
use std::path::PathBuf;

use std::process::{Command, Child};
use std::os::windows::process::CommandExt;


use crate::backend::backend_misc::CREATE_NO_WINDOW;
use crate::backend::persistent_state::exe_dir;


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

pub fn install_wsl_only_child()->Result<Child, std::io::Error> {
    return Command::new("cmd")
                    .creation_flags(CREATE_NO_WINDOW) // create no window
                    .args(["/C", "wsl --install --no-launch -d Ubuntu-22.04"])
                    .env("WSL_UTF8", "1")
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .spawn();
}

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
    let distro_tar_path = rfd::FileDialog::new()
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


//################################################################################
//## Setting up Singularity
//################################################################################



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
    //todo()!;
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

