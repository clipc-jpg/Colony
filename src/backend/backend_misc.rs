

use std::error::Error;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::io::{Read, Write};
use std::process::{Child, Command, ChildStdout};
use std::io::BufRead;
use std::io::BufReader;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use fs_extra::dir::CopyOptions;
use itertools::Itertools;

use crate::persistent_state::exe_dir;

extern crate fs_extra;


const CREATE_NO_WINDOW: u32 = 0x08000000;


//################################################################################
//## file management
//################################################################################

#[cfg(not(target_arch = "wasm32"))]
pub fn choose_sif_file(starting_dir: &Option<PathBuf>) -> Option<PathBuf> {
    println!("Choosing sif file");
    let path = match starting_dir {
        Some(ptb) => ptb.clone(),
        None => std::env::current_dir().unwrap()
    };

    println!("Opening file dialog at {:?}",&path);
    let res = rfd::FileDialog::new()
        .set_directory(&path)
        .add_filter("SIF files", &["sif"])
        .add_filter("All files", &["*"])
        .pick_file();

    println!("The user chose: {:#?}", res);

    return res;
}

#[cfg(not(target_arch = "wasm32"))]
pub fn choose_file(starting_dir: &Option<PathBuf>) -> Option<PathBuf> {
    let path = match starting_dir {
        Some(ptb) => ptb.clone(),
        None => std::env::current_dir().unwrap()
    };

    let res = rfd::FileDialog::new()
        .set_directory(&path)
        .add_filter("All files", &["*"])
        .pick_file();

    println!("The user chose: {:#?}", res);

    return res;
}

#[cfg(not(target_arch = "wasm32"))]
pub fn choose_files(starting_dir: &Option<PathBuf>) -> Option<Vec<PathBuf>> {
    let path = match starting_dir {
        Some(ptb) => ptb.clone(),
        None => std::env::current_dir().unwrap()
    };

    let res = rfd::FileDialog::new()
        .set_directory(&path)
        .add_filter("All files", &["*"])
        .pick_files();

    println!("The user choose: {:#?}", res);

    return res;
}

#[cfg(not(target_arch = "wasm32"))]
pub fn choose_directory(starting_dir: &Option<PathBuf>) -> Option<PathBuf> {
    let path = match starting_dir {
        Some(ptb) => ptb.clone(),
        None => std::env::current_dir().unwrap()
    };

    let res = rfd::FileDialog::new()
        .set_directory(&path)
        .pick_folder();

    println!("The user choose: {:#?}", res);

    return res;
}

#[cfg(not(target_arch = "wasm32"))]
pub fn choose_directories(starting_dir: &Option<PathBuf>) -> Option<Vec<PathBuf>> {
    let path = match starting_dir {
        Some(ptb) => ptb.clone(),
        None => std::env::current_dir().unwrap()
    };

    let res = rfd::FileDialog::new()
        .set_directory(&path)
        .pick_folders();

    println!("The user choose: {:#?}", res);

    return res;
}


//################################################################################
//## interacting with child processes
//################################################################################

//pub fn bufread_child_stdout(mut target: SyncSignal<String>,
//                         mut process: Child) {
//    let stdout = process.stdout.take().unwrap();
//
//    // Stream output.
//    let lines = BufReader::new(stdout).lines();
//    for line in lines {
//        let ln = line.unwrap();
//        println!("{}", &ln);
//        //target.write().push_str("\t"); does not work, currently
//        target.write().push_str(&ln);
//        target.write().push_str("\n");
//    };
//}

//pub async fn bufread_child_stdout_async(mut target: SyncSignal<String>,
//                         mut process: Child)->JoinHandle<()> {
//    return tokio::spawn( async move {
//        let stdout = process.stdout.take().unwrap();
//
//        // Stream output.
//        let lines = BufReader::new(stdout).lines();
//        for line in lines {
//            let ln = line.unwrap();
//            println!("{}", &ln);
//            //target.write().push_str("\t"); does not work, currently
//            target.write().push_str(&ln);
//            target.write().push_str("\n");
//        }
//    });
//}

pub async fn bufread_child_stdout_into_messages(
                         output_collection: &mut Arc<Mutex<Vec<String>>>,
                         process_stdout: ChildStdout) {

    // Stream output.
    let lines = BufReader::new(process_stdout).lines();
    for line in lines {
        let mut n = 0;
        while n<3 {
            if let Err(ref _unusable) = line {continue;}
            match output_collection.lock() {
                Ok(ref mut mutex) => {
                    let ln = line.unwrap();
                    mutex.push(ln);
                    break;
                },
                Err(_) => {n += 1;}
            }
        }
    };
}


// TODO: test implementation with rsync -ah --info=progress2 [source] [destination] and curl -o destination File://source
pub fn bufread_child_stdout_bytes_into_messages(
                         output_collection: &mut Arc<Mutex<Vec<String>>>,
                         process: &mut Child) {
    let mut stdout = process.stdout.take().unwrap();

    bufread_stdout_bytes_into_messages(output_collection, &mut stdout);
}

pub fn bufread_stdout_bytes_into_messages(
                         output_collection: &mut Arc<Mutex<Vec<String>>>,
                         stdout: &mut ChildStdout) {

    let mut accumulated_buf = Vec::new() as Vec<u8>;
    let mut reserve_buf = Vec::new() as Vec<u8>;
    let mut current_line = String::new();
    let mut current_line_revised = Vec::new() as Vec<char>;

    for byte in stdout.bytes() {
        let byte = match byte {
            Ok(b) => b,
            Err(_) => continue
        };
        accumulated_buf.push(byte);

        // all chunks contain valid utf8 and invalid utf8
        // all but the last invalid parts will be converted to the invalid character "\u{FFFD}"
        // all valid chunks need to split by the Carriage return character, in order to backtrack inputs as e.g. progrss bars do

        let mut chunks = accumulated_buf.utf8_chunks().collect_vec();
        let last_chunk = chunks.pop();

        for chunk in &chunks {
            current_line.push_str(chunk.valid());
            if !chunk.invalid().is_empty() { current_line.push_str("\u{FFFD}"); }
        }
        chunks.clear();
        match last_chunk {
            Some(chunk) => {
                current_line.push_str(chunk.valid());
                for byte in chunk.invalid().bytes() {
                    match byte {
                        Ok(b) => { reserve_buf.push(b); },
                        Err(_) => continue
                    }
                }
            },
            None => {}
        }

        std::mem::swap(&mut accumulated_buf, &mut reserve_buf); reserve_buf.clear();

        // mutating output collection
        // the following code section is actually wrong, since backspace usually deletes a codepoint (==char)
        // but sometimes, grapheme clusters (or parts of them) are considered inseparable and deleted as a whole
        // Todo: find source that explains most correct approach (e.g. approach of browsers or text editors)
        current_line_revised.clear();
        for ch in current_line.chars() {
            if ch == char::from_u32(8).expect("Char 8 should have converted to BACKSPACE") { // Backspace
                // if no chars is left, then the previous lines remain untouched
                // cannot dsiplay multiline outputs (e.g. TUIs)
                current_line_revised.pop();
            } else if ch == char::from_u32(13).expect("Char 13 should have converted to CARRIAGE RETURN") {  // Carriage Return: do nothing
            } else if ch == char::from_u32(10).expect("Char 10 should have converted to LINE FEED") { // Line Feed
                match output_collection.lock() {
                    Ok(mut lines) => {
                        lines.push(current_line_revised.iter().collect());
                        current_line_revised.clear();
                    },
                    Err(_) => {break;}
                }
            } else if false && current_line_revised.len() > 1000 {
                match output_collection.lock() {
                    Ok(mut lines) => {
                        lines.push(current_line_revised.iter().collect());
                        current_line_revised.clear();
                    },
                    Err(_) => {break;}
                }
            } else {
                current_line_revised.push(ch);
            }
        }

        current_line = current_line_revised.iter().collect();

        match output_collection.lock() {
            Ok(mut lines) => {
                if lines.len() == 0 {
                    lines.push(current_line.clone());
                } else {
                    match lines.last_mut() {
                        Some(last_line) => {
                            current_line.clone_into(last_line);
                        },
                        None => {} // impossible
                    }
                }
            },
            Err(_) => {break;} // lock is poisened and will never return again
        }
    }
}



//################################################################################
//## General WSL command
//################################################################################

pub fn run_wsl_command(command: &str) -> Result<Child,std::io::Error> {

    println!("Command: /C wsl -d ColonyWSL -e {}", &command);
    //return Command::new("cmd")
//                .args(["/C", "wsl", "-e", &command])
//                .args(command_args)
//                .spawn();
    return Command::new("cmd")
                .creation_flags(CREATE_NO_WINDOW) // create no window
                .arg(format!("/C wsl -d ColonyWSL -e {}", &command))
                .env("WSL_UTF8", "1")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn();
}

//################################################################################
//## Utilities
//################################################################################

pub fn wslify_windows_path(windows_path: &str) -> String {
    let result = windows_path.replace("C:\\","/mnt/c/").replace("\\","/");
    return result;
}

pub fn unwslify_wsl_linux_path(wsl_linux_path: &str) -> String {
    let result = wsl_linux_path.replace("/mnt/c/", "C:\\").replace("/","\\");
    return result;
}

//TODO: config file may contain relative paths
pub fn copy_config_and_file_references(workdir: &PathBuf, config_file_path: &PathBuf) -> Result<(), Box<dyn Error>> {

    let mut output_dir = workdir.clone(); output_dir.push(PathBuf::from_str("output").unwrap());
    if !output_dir.is_dir() { std::fs::create_dir_all(output_dir)?; }

    let mut target_dir = workdir.clone(); target_dir.push(PathBuf::from_str("input").unwrap());
    if !target_dir.is_dir() { std::fs::create_dir_all(&target_dir)?; }

    let config_file = std::fs::File::open(config_file_path)?;
    let config_file_reader = BufReader::new(config_file);
    let mut configuration: serde_json::Value = serde_json::from_reader(config_file_reader)?;

    let mut to_be_moved = Vec::new() as Vec<PathBuf>;

    let mut json_stack = Vec::new() as Vec<&mut serde_json::Value>;
    json_stack.push(&mut configuration);
    while !json_stack.is_empty() {
        let current_search: &mut serde_json::Value = match json_stack.pop() {
            Some(val) => val,
            None => break
        };

        if current_search.is_object() {
            if let Some(mut_object) = current_search.as_object_mut() {
                for (key, mut elem) in mut_object.iter_mut() {
                    if key.ends_with("_path") && elem.is_string() {
                        if let serde_json::Value::String(ref mut string_val) = &mut elem {
                            let native_path = unwslify_wsl_linux_path(&string_val);
                            to_be_moved.push( PathBuf::from(&native_path));
                            let path_val = PathBuf::from_str(string_val).unwrap();
                            let mut pth = PathBuf::from("./input"); pth.push(path_val.components().last().unwrap());
                            string_val.clone_from(&wslify_windows_path(&pth.as_os_str().to_string_lossy().to_string()));
                        }
                    } else if elem.is_object() || elem.is_array() {
                        json_stack.push(elem);
                    }
                }
            }
        } else if current_search.is_array() {
             if let Some(mut_array) = current_search.as_array_mut() {
                 for elem in mut_array.iter_mut() {
                     if elem.is_object() || elem.is_array() {
                         json_stack.push(elem);
                     }
                 }
             }
        }
    }

    //check if everything is ok
    for pth in to_be_moved.iter_mut() {
        if pth.is_absolute() {
            //todo: unwslify paths, if necessary
            if pth.components().last().is_none() {
                //pth probably is root and we assume that is a mistake
                //since copying a whole file system makes sense basically never
                return Err(format!("Path is malformed: {:?}", &pth).into());
            }
        } else if pth.is_relative() {
            let mut target = target_dir.clone();
            target.push(&mut *pth);
            if !target.exists() {
                return Err(format!("Given Path is relative, but does not already exist in target directory {:?}", &pth).into());
            }
        }
    }

    // relative paths have been checked to be at their destination
    // ergo, they do not need to be processed anymore
    to_be_moved.retain(|pth| pth.is_absolute());
    fs_extra::copy_items(&to_be_moved, &target_dir, &CopyOptions::new().overwrite(true))?;
    // overwrite copied configuration with modified configuration

    let mut config_target = workdir.clone();
    config_target.push(config_file_path.components().last().ok_or_else(|| format!("File path malformed: {:?}", &config_file_path))?);
    println!("Copying configuration file: {:?} => {:?}", &config_file_path, &config_target);
    let config_target_file = std::fs::File::create(config_target)?;
    //let config_target_writer = BufStream::new(BufWriter::new(configuration));
    serde_json::to_writer_pretty(config_target_file, &configuration)?;

    println!("Done copying files!");
    return Ok(());
}


//################################################################################
//## Install WSL and import Colony, if necessary
//## TODO: commands sensible on windows only -> move into separate module
//################################################################################


pub fn check_wsl() -> Result<std::process::Output, std::io::Error> {
    let output = Command::new("cmd")
                        .creation_flags(CREATE_NO_WINDOW) // create no window
                        .env("WSL_UTF8", "1")
                        .args(["/C", "wsl --status"])
                        .output();
    match output {
        Ok(ref output) => {
            let out = std::str::from_utf8(&output.stdout);
            match out {
                Ok(string) => {
                    println!("{}", string);
                },
                Err(_) => {
                    println!("Error parsing wsl status output");
                }
            }
            println!("Exit code: {}", output.status);
        },
        Err(_) => {println!("Error checking wsl");}
    }
    return output;
}

pub fn request_wsl_only_child() -> Result<Child, std::io::Error> {
    let mut child_result =  Command::new("cmd")
                                .creation_flags(CREATE_NO_WINDOW) // create no window
                                .args(["/C", "wsl"])
                                .env("WSL_UTF8", "1")
                                .stdin(std::process::Stdio::piped())
                                .stdout(std::process::Stdio::piped())
                                .spawn();

    if let Ok(ref mut child) = child_result {
        let child_stdin = child.stdin.take();
        match child_stdin {
            Some(mut std_in) => {
                match std_in.write(&[0,65]) {
                    Ok(_) => {},
                    Err(e) => {return Err(std::io::Error::new(std::io::ErrorKind::Other, "Child process requesting wsl could not be written to"));}
                }
            },
            None => {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "Child process requesting wsl could not be written to"));
            }
        };
    };
    return child_result;
}

pub fn install_wsl_distribution_only_child()->Result<Child, std::io::Error> {
    return Command::new("cmd")
                .creation_flags(CREATE_NO_WINDOW) // create no window
                .args(["/C", "wsl --install --no-launch -d Ubuntu-22.04"])
                .env("WSL_UTF8", "1")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn();
}

pub fn install_wsl_only_child_pseudointeractive()->Result<Child, std::io::Error> {
    let mut installation = Command::new("cmd")
                .creation_flags(CREATE_NO_WINDOW) // create no window
                .args(["/C", "wsl --install -d Ubuntu-22.04"])
                .env("WSL_UTF8", "1")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn();
    match installation {
        Ok(ref mut child) => {
            let mut installation_stdin = child.stdin.take().unwrap();
            println!("Writing into process...");
            // type in username
            installation_stdin.write("user\r\n".as_bytes());
            println!("Written into process!");
            // type in password
            installation_stdin.write("passowrd\r\n".as_bytes());
            println!("Written into process!");
            // retype password
            installation_stdin.write("passowrd\r\n".as_bytes());
            println!("Written into process!");
            // exit distribution
            installation_stdin.write("exit\r\n".as_bytes());
            println!("Written into process!");
            child.stdin = Some(installation_stdin);
        },
        Err(_) => {}
    }
    println!("Function returns: install_wsl_only_child_pseudointeractive");
    return installation;
}

//pub fn install_wsl_only() {
//    let output = install_wsl_only_child()
//                    .expect("Failed to install wsl")
//                    .wait_with_output()
//                    .expect("Failed to install wsl");
//    println!("{}", match std::str::from_utf8(&output.stdout) {
//                        Ok(out) => out,
//                        Err(_) => "Error at install_wsl_only"
//                   }
//    );
//    println!("Exit code: {}", output.status);
//}

pub fn update_wsl_only_child()->Result<Child, std::io::Error> {
    return Command::new("cmd")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["/C", "wsl --update"])
            .env("WSL_UTF8", "1")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn();
}

pub fn set_wsl_set_default_distro_child() -> Result<Child, std::io::Error> {
    return Command::new("cmd")
            .creation_flags(CREATE_NO_WINDOW)
            .args(["/C", "wsl --set-default Ubuntu-22.04"])
            .env("WSL_UTF8", "1")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn();
}

pub fn find_wsl_distros() -> Vec<String> {
    let output = Command::new("cmd")
                        .creation_flags(CREATE_NO_WINDOW)
                        .args(["/C", "wsl -l"])
                        .env("WSL_UTF8", "1")
                        .output()
                        .expect("Failed to check wsl status");
    let wsl_response = String::from_utf8_lossy(&output.stdout);
    println!("Installed WSL distributions: {}", &wsl_response);
    let distros  = if wsl_response.contains("DEFAULT_DISTRO_NOT_FOUND") {
        vec!["DEFAULT_DISTRO_NOT_FOUND".to_string()] as Vec<String>
    } else if wsl_response.contains("/wslstore") {
        vec![] as Vec<String>
    } else {
        wsl_response.lines().into_iter().skip(1).map(String::from).collect()
    };
    println!("Extracted WSL distributions: {:?}", &distros);
    println!("Exit code: {}", output.status);
    return distros;
}

pub fn install_wsl_ubuntu_child() -> Result<Child, std::io::Error> {
    return Command::new("cmd")
                .creation_flags(CREATE_NO_WINDOW)
                .args(["/C", "wsl --install --no-launch -d Ubuntu-22.04"])
                .env("WSL_UTF8", "1")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn();
}


pub fn wsl_export_ubuntu_child(distro_tar_path: &str) -> Result<Child, std::io::Error> {
    return Command::new("cmd")
                .creation_flags(CREATE_NO_WINDOW)
                .args(["/C", "wsl --export Ubuntu-22.04", distro_tar_path])
                                .env("WSL_UTF8", "1")
                                .stdin(std::process::Stdio::piped())
                                .stdout(std::process::Stdio::piped())
                                .spawn();
}

pub fn wsl_reimport_ubuntu_child(distro_tar_path: &str) -> Result<Child, std::io::Error> {
    return Command::new("cmd")
                .creation_flags(CREATE_NO_WINDOW)
                .args(["/C", "wsl --import ColonyWSL $HOME/ColonyWSL", distro_tar_path])
                .env("WSL_UTF8", "1")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn();
}

pub fn wsl_import_colony_wsl_child(distro_tar_path: &str) -> Result<Child, std::io::Error> {
    let target_dir = exe_dir().clone().unwrap_or_else(PathBuf::new);
    return Command::new("cmd")
                .creation_flags(CREATE_NO_WINDOW)
                .args(["/C", "wsl --import ColonyWSL", &target_dir.to_string_lossy().to_string(), distro_tar_path])
                .env("WSL_UTF8", "1")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn();
}

pub fn wsl_manually_import_distro() -> Result<Child, std::io::Error> {
    let distro_tar_path =
            rfd::FileDialog::new()
                .add_filter("Tar files", &["tar"])
                .add_filter("All Files", &["*"])
                .pick_file()
                //.ok_or(Err(std::io::Error::new(std::io::ErrorKind::Other, "No File Selected")))?;
                .ok_or(std::io::Error::new(std::io::ErrorKind::Other, "No File Selected"))?;
    println!("Distro_tar_path: {:?}", &distro_tar_path);
    let target_dir = exe_dir().clone().unwrap_or_else(PathBuf::new);
    return Command::new("cmd")
                .creation_flags(CREATE_NO_WINDOW)
                .env("WSL_UTF8", "1")
                //.args(["/C", "wsl --import ColonyWSL $HOME/ColonyWSL", &format!(r#"{:?}"#, &distro_tar_path)])
                .args(["/C", "wsl --import", "ColonyWSL", &target_dir.to_string_lossy().to_string(),  &distro_tar_path.to_string_lossy().to_string()])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn();
}

//pub fn wsl_export_reimport_ubuntu(distro_tar_path: &str) {
//    let output1 = wsl_export_ubuntu_child(distro_tar_path)
//                        .expect("Failed to export ubuntu")
//                        .wait_with_output()
//                        .expect("Failed to export ubuntu");
//    println!("Exit code export: {}", output1.status);
//
//
//    let output2 = wsl_reimport_ubuntu_child(distro_tar_path)
//                        .expect("Failed to reimport ubuntu")
//                        .wait_with_output()
//                        .expect("Failed to reimport ubuntu");
//    println!("Exit code reimport: {}", output2.status);
//}
//
//pub fn wsl_import_colony_distro(distro_tar_path: &str) {
//    //TODO: give file dialog, if automatism fails
//    let exe_pth = env::current_exe().expect("Failed to find Path of executable!");
//    println!("Executable located at: {}", exe_pth.display());
//    let homedir = match env::var("$HOME") {
//        Ok(val) => val,
//        Err(e) => {println!("Environment variable 'HOME' not found: {e}"); String::from(".")},
//    };
//    println!("Installing custom WSL distribution located at {}...", distro_tar_path);
//    let wsl_cmd = format!("wsl --import ColonyWSL {}/ColonyWSL {}", homedir, distro_tar_path);
//    let output = Command::new("cmd")
//                        .args(["/C", &wsl_cmd])
//                        .output()
//                        .expect("Failed to import custom wsl distribution");
//    println!("{}", std::str::from_utf8(&output.stdout).unwrap());
//    println!("Exit code: {}", output.status);
//}

pub fn check_singularity_version() -> Result<std::process::Output, std::io::Error> {
    let output = Command::new("cmd")
                        .creation_flags(CREATE_NO_WINDOW)
                        .args(["/C", "wsl -d ColonyWSL -e singularity --version"])
                        .env("WSL_UTF8", "1")
                        .stdin(std::process::Stdio::piped())
                        .stdout(std::process::Stdio::piped())
                        .output();
    match output {
        Ok(ref output) => {
            println!("{}", std::str::from_utf8(&output.stdout).unwrap());
            println!("Exit code: {}", output.status);
        },
        Err(_) => {println!("Error checking singularity version");}
    }
    return output;
}

pub fn install_singularity_child()->Result<Child, std::io::Error> {
    return Command::new("cmd")
                .creation_flags(CREATE_NO_WINDOW)
                .args(["/C", "wsl -d ColonyWSL -e singularity --version"])
                .env("WSL_UTF8", "1")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn();
}

pub fn shut_down_colonywsl_child() -> Result<Child, std::io::Error> {
    return Command::new("cmd")
                .creation_flags(CREATE_NO_WINDOW)
                .args(["/C", "wsl --terminate ColonyWSL"])
                .env("WSL_UTF8", "1")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn();
}


//################################################################################
//################################################################################
//##
//## Interacting with Singularity
//##
//################################################################################
//################################################################################

//################################################################################
//## query a singularity container for its capabilities
//## TODO: commands depend on operating system
//## TODO: !!!! ensure shell escaping
//################################################################################

//TODO: introduce error enum, since having no app list is different from not being able to read from it
//TODO: serialize response into struct

fn singularity_command(args: Vec<&str>) -> Option<String>  {

    let output_res = Command::new("cmd")
                        .creation_flags(CREATE_NO_WINDOW)
                        .args(&args)
                        .output();

    return match output_res {
        Ok(out) => match out.status.code() {
            Some(0) => {
                let s = std::string::String::from_utf8_lossy(&out.stdout);
                println!("Command output is: {}", &s);
                Some(s.into_owned())
            },
            Some(k) => {
                println!("Command resulted in exit code {}", k);
                let s = std::string::String::from_utf8_lossy(&out.stderr);
                println!("Command output is: {}", &s);
                Some(s.into_owned())
            },
            None => {
                println!("Command did not properly terminate!");
                None
            }
        },
        Err(e) => {
            println!("Error executing {:?}: {:?}", &args, &e);
            None
        }
    }
}

pub fn singularity_inspection(container_path: &PathBuf) -> Option<serde_json::Value> {
    let pth = wslify_windows_path(&container_path.to_string_lossy().to_string());
    let response = singularity_command(vec!["/C", "wsl -d ColonyWSL -e singularity inspect --all", &pth]);

    let json: Option<serde_json::Value> = match response {
        Some(s) => {
            match serde_json::from_str(&s) {
                Ok(js) => {
                    println!("Success interpreting json data");
                    Some(js)
                },
                Err(_) => {
                    println!("Error interpreting json data: {:?}", &s);
                    None
                }
            }
        },
        None => {
            println!("Error reading json data");
            None
        }
    };

    return json;
}

pub fn singularity_app_list(container_path: &PathBuf) -> Option<Vec<String>> {
    let pth = wslify_windows_path(&container_path.to_string_lossy().to_string());
    let response = singularity_command(vec!["/C", "wsl -d ColonyWSL -e", "singularity", "inspect", "--list-apps", &pth]);
    let result = response.map(|resp| resp.lines().map(String::from).collect_vec());
    return result;
}

#[allow(unused)]
pub fn singularity_help(container_path: &PathBuf) -> Option<String> {
    let pth = wslify_windows_path(&container_path.to_string_lossy().to_string());
    return singularity_command(vec!["/C", "wsl -d ColonyWSL -e", "singularity help", &pth]);
}

#[allow(unused)]
pub fn singularity_label(container_path: &PathBuf) -> Option<String> {
    let pth = wslify_windows_path(&container_path.to_string_lossy().to_string());
    return singularity_command(vec!["/C", "wsl -d ColonyWSL -e", "singularity inspect --json --labels", &pth]);
}

//TODO: serialize response into struct
pub fn singularity_app_requirements(container_path: &PathBuf) -> Option<String> {
    let pth = wslify_windows_path(&container_path.to_string_lossy().to_string());
    return singularity_command(vec!["/C", "wsl -d ColonyWSL -e", "singularity run --app app-requirements", &pth]);
 }

//TODO: serialize response into json
pub fn singularity_app_configuration_options(container_path: &PathBuf) -> Option<String> {
    let pth = wslify_windows_path(&container_path.to_string_lossy().to_string());
    return singularity_command(vec!["/C", "wsl -d ColonyWSL -e", "singularity run --app app-configurations", &pth]);
 }

// pub fn singularity_app_help(container_path: &str, app: &str) -> Option<String> {
//     return singularity_command(vec!["/C", "wsl -d ColonyWSL -e", "singularity help --app", &app, &container_path]);
// }

// pub fn singularity_app_label(container_path: &str, app: &str) -> Option<String> {
//     return singularity_command(vec!["/C", "wsl -d ColonyWSL -e", "singularity inspect --json --labels --app", &app, &container_path]);
// }

// pub fn singularity_test_app(container_path: &str, app: &str) -> Result<Child,std::io::Error> {
//     return Command::new("cmd")
//                 .args(["/C", "wsl", "-e", "singularity", "test", "--app", &app, &container_path])
//                 .spawn();
// }

//################################################################################
//## running and testing a container
//## TODO: commands depend on operating system
//## TODO: !!!! ensure shell escaping
//################################################################################

pub fn singularity_run(container_path: &PathBuf, container_args: Vec<String>) -> Result<Child,std::io::Error> {
    let pth = wslify_windows_path(&container_path.to_string_lossy().to_string());
    println!("Command: /C wsl -d ColonyWSL -e singularity run --bind /mnt:/mnt {:?} {:?}", &pth, &container_args);
    return Command::new("cmd")
                .creation_flags(CREATE_NO_WINDOW)
                .args(["/C", "wsl -d ColonyWSL -e", "singularity", "run", "--bind", "/mnt:/mnt", &pth])
                .args(container_args)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn();
}

pub fn singularity_run_in_dir(working_directory: &PathBuf, container_path: &PathBuf, container_args: Vec<String>) -> Result<Child,std::io::Error> {
    let work_dir = wslify_windows_path(&working_directory.to_string_lossy().to_string());
    println!("Working Directory: {}", &work_dir);
    let container_pth = wslify_windows_path(&container_path.to_string_lossy().to_string());
    println!("Container Path: {}", &container_pth);
    let mut base_cmd = Command::new("cmd");
    let cmd = base_cmd
                .current_dir(&working_directory)
                .args(["/C", "wsl -d ColonyWSL --shell-type standard", "singularity", "run", "--pwd", &work_dir, "--bind", "/mnt:/mnt", &container_pth])
                .args(container_args)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                ;

    println!("Command: {:?}", &cmd);
    return cmd.spawn();
}

#[allow(unused)]
pub fn singularity_run_app(container_path: &PathBuf, app: &str, container_args: Vec<String>) -> Result<Child,std::io::Error> {
    let container_pth = wslify_windows_path(&container_path.to_string_lossy().to_string());
    return Command::new("cmd")
                .creation_flags(CREATE_NO_WINDOW)
                .args(["/C", "wsl -d ColonyWSL -e", "singularity", "run", "--app", &app, &container_pth])
                .args(container_args)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn();
}

pub fn singularity_run_app_in_dir(working_directory: &PathBuf, container_path: &PathBuf, app: &str, container_args: Vec<String>) -> Result<Child,std::io::Error> {
    let wsl_workdir = wslify_windows_path(&working_directory.to_string_lossy().to_string());
    let container_pth = wslify_windows_path(&container_path.to_string_lossy().to_string());
    println!("container app workdir: {:?}", &wsl_workdir);
    println!("container path: {:?}", &container_pth);
    println!("container app args: {:?}", &container_args);
    println!("Command: {:?}", &["--shell-type", "standard", "singularity", "run", "--pwd", &wsl_workdir, "--bind", "/mnt:/mnt", "--app", &app, &container_pth]);
    return Command::new("cmd")
                .creation_flags(CREATE_NO_WINDOW)
                .args(["/C", "wsl -d ColonyWSL -e", "singularity", "run", "--pwd", &wsl_workdir, "--bind", "/mnt:/mnt", "--app", &app, &container_pth])
                .args(container_args)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn();
}

#[allow(unused)]
pub fn singularity_test(container_path: &PathBuf) -> Result<Child,std::io::Error> {
    let container_pth = wslify_windows_path(&container_path.to_string_lossy().to_string());
    return Command::new("cmd")
                .creation_flags(CREATE_NO_WINDOW)
                .args(["/C", "wsl -d ColonyWSL -e", "singularity", "test", &container_pth])
                .spawn();
}















