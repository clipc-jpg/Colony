






use std::path::PathBuf;
use std::str::FromStr;   //Pathbuf::from_str

use std::process::{Command, Child};
use std::os::windows::process::CommandExt; // Command.creation_flags
use std::io::{Write, BufReader};

use std::sync::Mutex;
use std::sync::mpsc::Sender;

use fs_extra::dir::CopyOptions;

use actix_cors::Cors;
use actix_web::{get, post, web, web::Data, App, HttpServer, Responder};
use actix_web::web::PayloadConfig;
use itertools::Itertools; // Iterator.collect_vec();


use crate::{pages::*, JobId};
use crate::backend;
use crate::backend::backend_misc::{wslify_windows_path, unwslify_wsl_linux_path, CREATE_NO_WINDOW};
use crate::backend::BackendRequest;
use crate::backend::persistent_state::exe_dir;



//################################################################################
//## Singularity commands
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
                            .stdout(std::process::Stdio::piped())
                            .stderr(std::process::Stdio::piped())
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
    .arg("2>&1")
    .stdin(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
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
    .creation_flags(CREATE_NO_WINDOW)
    .current_dir(&working_directory)
    .args(["/C", "wsl -d ColonyWSL --shell-type standard", "singularity", "run", "--pwd", &work_dir, "--bind", "/mnt:/mnt", &container_pth])
    .args(container_args)
    //.arg("2>&1")
    .stdin(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .stdout(std::process::Stdio::piped());

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
    //.arg("2>&1")
    .stdin(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
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

    let singularity_cmd = format!(
        r#"wsl -d ColonyWSL -e "singularity run --pwd {} --bind /mnt:/mnt --app {} {} {} 2>&1 | tee raw_output.log""#,
        wsl_workdir,
        app,
        container_pth,
        container_args.join(" "),
    );

    return Command::new("cmd")
    .creation_flags(CREATE_NO_WINDOW)
    //.args(["/C", &singularity_cmd])
    .args(["/C", "wsl -d ColonyWSL -e", "singularity", "run", "--pwd", &wsl_workdir, "--bind", "/mnt:/mnt", "--app", &app, &container_pth])
    .args(container_args)
    //.arg("2>&1 | tee raw_output.log")
    .stdin(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .stdout(std::process::Stdio::piped())
    .spawn();
}

#[allow(unused)]
pub fn singularity_test(container_path: &PathBuf) -> Result<Child,std::io::Error> {
    let container_pth = wslify_windows_path(&container_path.to_string_lossy().to_string());
    return Command::new("cmd")
    .creation_flags(CREATE_NO_WINDOW)
    .args(["/C", "wsl -d ColonyWSL -e", "singularity", "test", &container_pth])
    .arg("2>&1")
    .spawn();
}





//################################################################################
//## Frontend-side Utilities
//################################################################################

//TODO: config file may contain relative paths
pub fn copy_config_and_file_references(workdir: &PathBuf, config_file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {

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
//## Frontend-Server with API for Singularity containers to connect with
//################################################################################


pub async fn start_frontend_local_server(followup_page: AppState, comm_partner_container: PathBuf,
                                         this_servers_job_id: JobId,
                                         comm_with_backend: Sender<BackendRequest>) {

    //TODO: implement timeout with a tokio::select! statement or similar
    tokio::spawn( async move {
        let fup_data = Data::new(Mutex::new(followup_page));
        let tsj_data = Data::new(Mutex::new(this_servers_job_id));
        let cpc_data = Data::new(Mutex::new(comm_partner_container));
        let cwb_data = Data::new(Mutex::new(comm_with_backend.clone()));
        let adr = "127.0.0.1";
        let port = 20311;

        println!("Making termination request...");
        reqwest::get("http://127.0.0.1:9283/terminate").await.ok();
        println!("Completed termination request");


        println!("Listening on {}:{}...", &adr, &port);
        let server_result = HttpServer::new(move || {
            let cors = Cors::default()
            .allowed_origin("http://localhost:9283")
            .allowed_origin("http://127.0.0.1:9283")
            .allowed_headers(vec![actix_web::http::header::AUTHORIZATION, actix_web::http::header::ACCEPT])
            .allowed_header(actix_web::http::header::CONTENT_TYPE)
            .allowed_methods(vec!["GET", "POST"])
            .max_age(300);
            let payload_config = PayloadConfig::new(32*1024);

            App::new()
            .wrap(cors)
            .app_data(payload_config)
            .app_data(Data::clone(&fup_data))
            .app_data(Data::clone(&tsj_data))
            .app_data(Data::clone(&cpc_data))
            .app_data(Data::clone(&cwb_data))
            .service(health_check)
            .service(pick_local_file)
            .service(pick_local_files)
            .service(pick_local_directory)
            .service(pick_local_directories)
            .service(download_json_config)
        })
        .bind((adr,port));


        //TODO: handle server start failure
        comm_with_backend.send(BackendRequest::InformLocalWebserverStarted(Ok(port))).ok();
        server_result.unwrap().run().await.ok();

        //match server_result {
        //            Ok(server) => {
        //                let srv = Some(server.run());
        //                println!("Server is running!");
        //                comm_with_backend.send(BackendRequest::InformLocalWebserverStarted(Ok(port))).ok();
        //            },
        //            Err(err) => {
        //                let err_boxed = Err(std::sync::Arc::new(Box::new(err)));
        //                println!("Server is NOT running!");
        //                comm_with_backend.send(BackendRequest::InformLocalWebserverStarted(err_boxed)).ok();
        //            }
        //        };



    });

    //return Ok(port);
}



//################################################################################
//## Frontend-Server Enpoint definitions
//################################################################################


#[get("/")]
async fn health_check() -> impl Responder {
    format!("Thie web server is up and running.")
}

#[get("/choosefile/{request_id}/{filename}")]
async fn pick_local_file(path: web::Path<(String,String)>) -> impl Responder {
    let (_request_id, _filename) = path.into_inner();
    let filepath = match backend::choose_file(exe_dir()) {
        Some(pth) => wslify_windows_path(&pth.to_string_lossy().to_string()),
        None => "".to_string()
    };
    format!("{filepath}")
}

#[get("/choosefiles/{request_id}/{filename}")]
async fn pick_local_files(path: web::Path<(String,String)>) -> impl Responder {
    let (_request_id, _filename) = path.into_inner();
    let filepaths = match backend::choose_files(exe_dir()) {
        Some(pths) => pths.into_iter().map(|pth| wslify_windows_path(&pth.to_string_lossy().to_string())).collect::<Vec<_>>(),
        None => vec!{"".to_string()}
    };
    format!("{:?}", filepaths)
}

#[get("/choosedirectory/{request_id}/{filename}")]
async fn pick_local_directory(path: web::Path<(String,String)>) -> impl Responder {
    let (_request_id, _filename) = path.into_inner();
    let dirpath = match backend::choose_directory(exe_dir()) {
        Some(pth) => wslify_windows_path(&pth.to_string_lossy().to_string()),
        None => "".to_string()
    };
    let resp_body = format!("{dirpath}");
    actix_web::HttpResponse::Ok().body(resp_body)
}

#[get("/choosedirectories/{request_id}/{filename}")]
async fn pick_local_directories(path: web::Path<(String,String)>) -> impl Responder {
    let (_request_id, _filename) = path.into_inner();
    let filepaths = match backend::choose_directories(exe_dir()) {
        Some(pths) => pths.into_iter().map(|pth| wslify_windows_path(&pth.to_string_lossy().to_string())).collect::<Vec<_>>(),
        None => vec!["".to_string()]
    };
    format!("{:?}", filepaths)
}

#[post("/config/json")]
async fn download_json_config(body: String, // alternatively, web::Json<serde_json::Value>
                              followup_page: Data<Mutex<AppState>>,
                              payload_sender: Data<Mutex<PathBuf>>,
                              this_servers_job_id: Data<Mutex<JobId>>,
                              payload_receiver: Data<Mutex<Sender<BackendRequest>>>
) -> impl Responder {
    println!("Payload received: {}", &body.to_string());
    let followup_page = followup_page.lock().unwrap();
    let payload_sender = payload_sender.lock().unwrap();
    let jid = this_servers_job_id.lock().unwrap();
    let payload_receiver = payload_receiver.lock().unwrap();

    let file_dest = rfd::FileDialog::new()
    .set_directory(exe_dir().clone().unwrap_or_else(PathBuf::new))
    .add_filter("JSON files", &["json"])
    .add_filter("All files", &["*"])
    .save_file();

    match file_dest {
        Some(pth) => {
            println!("Config accepted by user");
            payload_receiver.send(BackendRequest::SetAppState((*followup_page).clone())).ok();

            match std::fs::File::create(pth.clone()) {
                Ok(mut file) => {
                    file.write_all(&body.to_string().into_bytes()).ok();
                },
                Err(e) => {println!("Error writing to file: {:?}", e);}
            }

            payload_receiver.send(BackendRequest::AcceptConfiguration((*payload_sender).clone(), pth.to_string_lossy().to_string())).ok();
            payload_receiver.send(BackendRequest::StopProcess(jid.clone())).ok();
        },
        None => {println!("Config reception cancelled by user");}
    }

    "response"
}











