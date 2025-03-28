

use std::path::PathBuf;
use std::io::Write;
use std::sync::Mutex;
use std::sync::mpsc::Sender;

use actix_cors::Cors;
use actix_web::{get, post, web, web::Data, App, HttpServer, Responder};
use actix_web::web::PayloadConfig;
use actix_web::dev::Server;
use crate::{pages::*, wslify_windows_path, JobId};
use crate::backend;
use backend::BackendRequest;
use backend::persistent_state::exe_dir;


//################################################################################
//################################################################################
//################################################################################
//################################################################################
//##
//## TODO
//## rewrite entire server / replace it with a pipe (stdin/stdout)-based communication protocol
//##
//################################################################################
//################################################################################
//################################################################################
//################################################################################


// create Invalidation errors
// have functions return invalidationerrors


//must contain request ids
//#[derive(Debug, Clone)]
//pub enum SelfConfigRequest {
//    ChooseFile(u8),
//    ChooseFiles(u8),
//    ChooseDirectory(u8),
//    ChooseDirectories(u8), // should this be included?
//}

//impl SelfConfigRequest {
//    pub fn from_message(message: String) -> Result<Self,()> {
//        let pattern = Regex::new(r"(\d+):([^:]+)").unwrap();
//        let Some(caps) = pattern.captures(&message) else { return Err(()); };
//        let Ok(request_id): Result<u8,_> = caps[1].parse::<u8>() else { return Err(()) };
//        //let request_id = Ok(request_id) else { return Err(()) };
//        let request_name = &caps[2];
//
//        return match request_name {
//            "ChooseFile" => Ok(SelfConfigRequest::ChooseFile(request_id)),
//            "ChooseFiles" => Ok(SelfConfigRequest::ChooseFiles(request_id)),
//            "ChooseDirectory" => Ok(SelfConfigRequest::ChooseDirectory(request_id)),
//            "ChooseDirectories" => Ok(SelfConfigRequest::ChooseDirectories(request_id)),
//            _ => Err(())
//        }
//    }
//}


//#[derive(Debug, Clone)]
//pub enum SelfConfigResponse {
//    ChosenFile(u8, PathBuf),
//    ChosenFiles(u8, Vec<PathBuf>),
//    ChosenDirectory(u8, PathBuf),
//    ChosenDirectories(u8, Vec<PathBuf>), // should this be included?
//}

//impl SelfConfigResponse {
//    pub fn to_string(&self) -> String {
//        match self {
//            SelfConfigResponse::ChosenFile(id, path) => {
//                format!("{}:ChosenFile:{:?}", id, path)
//            },
//            SelfConfigResponse::ChosenFiles(id, paths) => {
//                format!("{}:ChosenFiles:{:?}", id, paths)
//            },
//            SelfConfigResponse::ChosenDirectory(id, path) => {
//                format!("{}:ChosenDirectory:{:?}", id, path)
//            },
//            SelfConfigResponse::ChosenDirectories(id, paths) => {
//                format!("{}:ChosenDirectories:{:?}", id, paths)
//            },
//        }
//    }
//}


//pub fn extract_ip_addr(message: &str) -> Option<IpAddr> {
//
//    let ipv4_regex = regex::Regex::new("(([0-9]|[1-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])\\.){3}([0-9]|[1-9][0-9]|1[0-9][0-9]|2[0-4][0-9]|25[0-5])").unwrap();
//    let ipv6_regex = regex::Regex::new("((([0-9a-fA-F]){1,4})\\:){7}([0-9a-fA-F]){1,4}").unwrap();
//
//    return ipv4_regex.find(&message)
//                     .map(|regex_match| regex_match.as_str().parse().expect("Invalid Ipv4 regex"))
//                     .or_else(|| {
//                         ipv6_regex.find(&message)
//                                   .map(|regex_match| regex_match.as_str().parse().expect("Invalid Ipv6 regex"))
//                    });
//}

//################################################################################
//## container self configurators can call into a locally hosted webserver
//## this enables native functionality
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
async fn download_json_config(body: String,
                              followup_page: Data<Mutex<AppState>>,
                              payload_sender: Data<Mutex<PathBuf>>,
                              this_servers_job_id: Data<Mutex<JobId>>,
                              payload_receiver: Data<Mutex<Sender<BackendRequest>>>
                              ) -> impl Responder {
    println!("Payload received: {}", &body);
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
                    file.write_all(&body.into_bytes()).ok();
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



















