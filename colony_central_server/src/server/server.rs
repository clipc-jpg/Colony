





use std::sync::{Arc, Mutex};

use actix_cors::Cors;
use actix_web::{get, post, web, web::Data, App, HttpServer, Responder};
use actix_web::web::PayloadConfig;



//################################################################################
//## Starting the server
//################################################################################

pub async fn start_actix_server(address: String, port: u16,
                                server_state: AppState) {

    //TODO: implement timeout with a tokio::select! statement or similar
    tokio::spawn( async move {
        let state_data = Data::new(Mutex::new(server_state));

        println!("Listening on {}:{}...", &address, &port);
        let server_result = HttpServer::new(move || {
            let cors = Cors::default()
            //.allowed_origin("http://localhost:9283")
            .allowed_header(actix_web::http::header::AUTHORIZATION)
            .allowed_header(actix_web::http::header::ACCEPT)
            .allowed_header(actix_web::http::header::CONTENT_TYPE)
            .allowed_methods(vec!["GET", "POST"])
            .max_age(300);
            let payload_config = PayloadConfig::new(32*1024);

            App::new()
            .wrap(cors)
            .app_data(payload_config)
            .app_data(Data::clone(&state_data))
            .service(health_check)
        })
        .bind((address,port));


        //TODO: handle server start failure
        server_result.unwrap().run().await.ok();
    });
}




//################################################################################
//## Async server Task Execution loop
//################################################################################




pub enum LocalBackendRequest {
    InspectSingularityContainer(PathBuf),
    RunSingularityRun,
    RunSingularityApp,
    DownloadContent,
    MoveContent,
    StartLocalWebServer,
    InformLocalWebserverStarted,
    InformContainerWebserverStarted,
    AcceptConfiguration,
    SendConfiguration,
    ExportAnalysisIntoRepository,
    SendJobInfo,
    SendJobOutput,
    StopProcess,
    StopAllProcesses,
    StopProgram,
    InspectFilesystem,
    ListDirectory,
}

pub enum SingularityQuery {
    AppList,
    Inspection,
    RunHelp,
    AppHelp(String),
    RunLabels,
    AppLabels(String),
    AppRequirements,
    AppConfigurationOptions
}


struct BackendCommChannel {
    pub receiver: std::sync::mpsc::Receiver<T>
    pub sender:   std::sync::mpsc::Sender<T>
}

#[tokio::main]
pub async fn local_task_processing_loop(comm_with_frontend: BackendCommChannel) {

    let mut database = HardTypedDBAccess::new();

    let mut container_infos = ContainerJobStore::from(&mut database);
    let mut process_store = ProcessStore::from(&mut database);
    let mut process_outputs = JobOutputStore::from(&mut database);

    loop {
        let message = comm_with_frontend.receiver.recv();
        println!("Backend received message: {:?}", &message);
        /*match message {
            Ok(task) => match task {
                LocalBackendRequest::InspectSingularityContainer(container_path, query) => {
                    match query {
                        SingularityQuery::AppList => {
                            match backend::singularity_app_list(&container_path) {
                                Some(msg) => {comm_with_frontend.send(BackendResponse::SingularityInfo(container_path, Some(SingularityResponse::AppList(msg)))).ok();},
                                None => {comm_with_frontend.send(BackendResponse::SingularityInfo(container_path, None)).ok();}
                            }
                        },
                        SingularityQuery::Inspection => {
                            match backend::singularity_inspection(&container_path) {
                                Some(msg) => {comm_with_frontend.send(BackendResponse::SingularityInfo(container_path, Some(SingularityResponse::Inspection(msg)))).ok();},
                                None => {comm_with_frontend.send(BackendResponse::SingularityInfo(container_path, None)).ok();}
                            }
                        },
                        SingularityQuery::RunHelp => {todo!()},
                        SingularityQuery::AppHelp(_appname) => {todo!()},
                        SingularityQuery::RunLabels => {todo!()},
                        SingularityQuery::AppLabels(_appname) => {todo!()},
                        // Specific to our containers
                        SingularityQuery::AppRequirements => {
                            match backend::singularity_app_requirements(&container_path) {
                                Some(msg) => {comm_with_frontend.send(BackendResponse::SingularityInfo(container_path, Some(SingularityResponse::AppRequirements(msg)))).ok();},
                                None => {comm_with_frontend.send(BackendResponse::SingularityInfo(container_path, None)).ok();}
                            }
                        },
                        SingularityQuery::AppConfigurationOptions => {
                            match backend::singularity_app_configuration_options(&container_path) {
                                Some(msg) => {comm_with_frontend.send(BackendResponse::SingularityInfo(container_path, Some(SingularityResponse::AppConfigurationOptions(msg)))).ok();},
                                None => {comm_with_frontend.send(BackendResponse::SingularityInfo(container_path, None)).ok();}
                            }
                        },
                    }
                },
                LocalBackendRequest::RunSingularityRun(workdir, container_path, container_args) => {
                    println!("Backend tasked with: Starting Container {:?}", &container_path);
                    let child_result = backend::singularity_run_in_dir(&workdir, &container_path, container_args);

                    match child_result {
                        Ok(mut child) => {
                            //let jid = JobId::new(format!("Running: {}", &container_path));
                            let job_id = JobId::new();
                            let output_collector = process_outputs
                            .entry(job_id.clone())
                            .or_insert_with(|| Arc::new(Mutex::new(Vec::new())));
                            let mut output_collector = Arc::clone(&output_collector);
                            println!("Command: {:?}", &child);

                            let job_id_coll = container_infos.entry(PathBuf::from(container_path))
                            .or_insert(Arc::new(Mutex::new(Vec::new() as Vec<JobId>)));
                            job_id_coll.lock().map(|mut vec_jid| vec_jid.push(job_id.clone())).ok();

                            let mut child_stdout = child.stdout.take().unwrap();
                            tokio::spawn(async move {
                                backend::bufread_stdout_bytes_into_messages(&mut output_collector, &mut child_stdout);
                            });

                            process_store.insert(job_id.clone(), Arc::new(Mutex::new(child)));

                            comm_with_frontend.send(BackendResponse::JobInfo(job_id, JobState::Running)).ok();
                        },
                        Err(e) => {
                            println!("Error starting container: {:?}", &e);
                        }
                    }
                },
                LocalBackendRequest::RunSingularityApp(workdir, container_path, app_name, app_args) => {
                    println!("Backend tasked with: Starting App '{}' of container {:?}", &app_name, &container_path);
                    let child_result = backend::singularity_run_app_in_dir(&workdir, &container_path, &app_name, app_args);
                    match child_result {
                        Ok(mut child) => {
                            println!("Command: {:?}", &child);
                            let job_id = JobId::new();
                            let output_collector = process_outputs
                            .entry(job_id.clone())
                            .or_insert_with(|| Arc::new(Mutex::new(Vec::new())));
                            let mut output_collector = Arc::clone(&output_collector);

                            let job_id_coll = container_infos.entry(PathBuf::from(container_path))
                            .or_insert(Arc::new(Mutex::new(Vec::new() as Vec<JobId>)));
                            job_id_coll.lock().map(|mut vec_jid| vec_jid.push(job_id.clone())).ok();

                            let mut child_stdout = child.stdout.take().unwrap();
                            tokio::spawn(async move {
                                backend::bufread_stdout_bytes_into_messages(&mut output_collector, &mut child_stdout);
                            });

                            process_store.insert(job_id.clone(), Arc::new(Mutex::new(child)));

                            comm_with_frontend.send(BackendResponse::JobInfo(job_id, JobState::Running)).ok();
                        },
                        Err(_) => {}
                    }
                },
                LocalBackendRequest::DownloadContent(source_url, destination_path) => {

                    //let file_errored = false;
                    match std::fs::File::create(destination_path.clone()) {
                        Ok(target_file) => {
                            let resp = reqwest::get(&source_url)
                            .await;

                            match resp {
                                Ok(msg) => {
                                    let mut file_errored = false;

                                    {
                                        let mut writer = BufWriter::new(target_file);
                                        let mut byte_stream = msg.bytes_stream();

                                        while let Some(chunk_result) = byte_stream.next().await {
                                            match chunk_result {
                                                Ok(chunk) => {
                                                    match writer.write(&chunk) {
                                                        Ok(_write_len) => {},
                                                        Err(_) => {
                                                            file_errored = true;
                                                            break;
                                                        }
                                                    }
                                                },
                                                Err(_) => {file_errored = true; break;}
                                            }
                                        }
                                        writer.flush().ok();
                                    }

                                    if file_errored {
                                        std::fs::remove_file(destination_path).ok();
                                        comm_with_frontend.send(BackendResponse::FileCreated(None)).ok();
                                    }
                                    else {comm_with_frontend.send(BackendResponse::FileCreated(Some(destination_path))).ok();}
                                },
                                Err(_e) => {comm_with_frontend.send(BackendResponse::FileCreated(None)).ok();}
                            }
                        },
                        Err(_) => {comm_with_frontend.send(BackendResponse::FileCreated(None)).ok();}
                    }
                },
                LocalBackendRequest::MoveContent(source_path, destination_path) => {
                    let mut operation_succeeded = false;
                    if source_path.is_file() {
                        match std::process::Command::new("cmd /C mv -f")
                        .args([&source_path.to_string_lossy().into_owned(), &destination_path.to_string_lossy().into_owned()])
                        .output() {
                            Ok(_) => {operation_succeeded = true;},
                            Err(_) => {operation_succeeded = false;}
                        }

                    } else if source_path.is_dir() {
                        match std::process::Command::new("cmd /C mv -rf")
                        .args([&source_path.to_string_lossy().into_owned(), &destination_path.to_string_lossy().into_owned()])
                        .output() {
                            Ok(_) => {operation_succeeded = true;},
                            Err(_) => {operation_succeeded = false;}
                        }
                    }
                    if operation_succeeded {comm_with_frontend.send(BackendResponse::FileCreated(Some(destination_path))).ok();}
                    else {comm_with_frontend.send(BackendResponse::FileCreated(None)).ok();}
                },
                LocalBackendRequest::StartLocalWebServer(followup_page, comm_partner_container) => {
                    let job_id = JobId::new();
                    println!("Backend starts Webserver with JobId {:?}", &job_id);
                    let _output_collector = process_outputs
                    .entry(job_id.clone())
                    .or_insert_with(|| Arc::new(Mutex::new(Vec::new())));

                    let comm_with_backend = comm_with_frontend.backsender.clone();
                    backend::start_frontend_local_server(followup_page, comm_partner_container, job_id, comm_with_backend).await;

                    // webserver thread will notify backend about success

                    println!("Frontend-local webserver has been requested");
                },
                LocalBackendRequest::InformLocalWebserverStarted(maybe_port) => {
                    match maybe_port {
                        Ok(port_number) => {
                            comm_with_frontend.send(BackendResponse::LocalWebServerStarted(Some(port_number))).ok();
                            println!("Frontend-local webserver has started");
                        },
                        Err(_) => {
                            comm_with_frontend.send(BackendResponse::LocalWebServerStarted(None)).ok();
                            println!("Frontend-local webserver has failed to start");
                        }
                    }
                },
                LocalBackendRequest::InformContainerWebserverStarted(maybe_port) => {
                    match maybe_port {
                        Ok(port_number) => {
                            comm_with_frontend.send(BackendResponse::ContainerWebServerStarted(Some(port_number))).ok();
                            println!("Frontend-local webserver has started");
                        },
                        Err(_) => {
                            comm_with_frontend.send(BackendResponse::ContainerWebServerStarted(None)).ok();
                            println!("Frontend-local webserver has failed to start");
                        }
                    }
                },
                LocalBackendRequest::AcceptConfiguration(container, config_str) => {
                    println!("Received Config: {}", config_str);

                    comm_with_frontend.send(BackendResponse::Configuration(container, config_str)).ok();
                    println!("Configuration has been sent to frontend");

                },
                LocalBackendRequest::SendConfiguration() => {
                    println!("Not implemented: SendConfiguration");
                },
                LocalBackendRequest::ExportAnalysisIntoRepository(jid, workdir, container, config_file) => {

                    match (container.components().last(), config_file.components().last()) {
                        (Some(container_name), Some(_config_file_name)) => {
                            comm_with_frontend.send(BackendResponse::JobInfo(jid, JobState::Running)).ok();
                            let mut container_target = workdir.clone();
                            container_target.push(container_name);
                            println!("Copying container: {:?} => {:?}", &container, &container_target);
                            std::fs::copy(&container, &container_target).ok();

                            match backend::copy_config_and_file_references(&workdir, &config_file) {
                                Ok(_) => {
                                    println!("Copy Job succeded!");
                                    comm_with_frontend.send(BackendResponse::JobInfo(jid, JobState::Completed(ExitStatus::from_raw(0)))).ok();
                                },
                                Err(e) => {
                                    println!("Copy Job failed! Reason: {:?}", e);
                                    comm_with_frontend.send(BackendResponse::JobInfo(jid, JobState::Completed(ExitStatus::from_raw(1)))).ok();
                                }
                            }
                        },
                        _ => {
                            //TODO: BackendResponse contains Result
                        }
                    };

                },
                LocalBackendRequest::SendJobInfo(job_id) => {
                    let process_entry = process_store.get(&job_id);

                    if let Some(process) = process_entry {
                        match process.lock() {
                            Ok(mut proc) => {
                                match proc.try_wait() {
                                    Ok(Some(status)) => {
                                        comm_with_frontend.send(BackendResponse::JobInfo(job_id, JobState::Completed(status))).ok();
                                    },
                                    Ok(None) => {
                                        comm_with_frontend.send(BackendResponse::JobInfo(job_id, JobState::Running)).ok();
                                    }
                                    Err(_) => {
                                        comm_with_frontend.send(BackendResponse::JobInfo(job_id, JobState::Unknown)).ok();
                                    },
                                }
                            },
                            Err(_) => {
                                //Is this a definite failure or is this an actual JobState::Unknown?
                                comm_with_frontend.send(BackendResponse::JobInfo(job_id, JobState::Unknown)).ok();
                            }
                        }
                    } else {
                        comm_with_frontend.send(BackendResponse::JobInfo(job_id, JobState::JobNotListed)).ok();
                    }
                },
                LocalBackendRequest::SendJobOutput(job_id, offset) => {
                    let output = process_outputs.get(&job_id);
                    match output {
                        Some(output) => {
                            match output.lock() {
                                Ok(lock) => {
                                    //println!("Backend sending slice from output of length: {}", lock.len());
                                    let lines = lock.iter().skip(offset).map(|s| s.clone()).collect();
                                    comm_with_frontend.send(BackendResponse::JobOutput(job_id, lines)).ok();
                                },
                                Err(_) => {}
                            }
                        },
                        None => {
                            comm_with_frontend.send(BackendResponse::JobOutput(job_id, vec!["No Output".to_string()])).ok();
                        }
                    }
                },
                LocalBackendRequest::StopProcess(jid) => {
                    println!("Backend stops process with JobId {:?}", &jid);
                    process_store.store.entry(jid.clone())
                    .and_modify(|arc| { arc.lock().unwrap().kill().ok(); });

                    //TODO: store address of server properly (per job_id) or create yet another BackendRequest
                    let _resp = reqwest::get("http://127.0.0.1:9283/terminate")
                    .await;
                    comm_with_frontend.send(BackendResponse::StoppedProcess(jid)).ok();
                },
                LocalBackendRequest::StopAllProcesses => {

                    process_store.store.values_mut().for_each(|arc| {
                        if let Ok(mut child) = arc.lock() { child.kill().ok(); }
                    });

                    //TODO: store address of server properly (per job_id) or create yet another BackendRequest
                    let _resp = reqwest::get("http://127.0.0.1:9283/terminate")
                    .await;

                    comm_with_frontend.send(BackendResponse::StoppedAllProcesses).ok();
                },
                LocalBackendRequest::StopProgram => {
                    match backend::shut_down_colonywsl_child() {
                        Ok(mut child) => {
                            let begin = tokio::time::Instant::now();
                            loop {
                                let now = tokio::time::Instant::now();
                                let dt = std::time::Duration::from_millis(100);
                                tokio::time::sleep_until(now + dt).await;
                                match child.try_wait() {
                                    Ok(Some(_status)) => { break; }
                                    _ => {
                                        if begin.elapsed() > std::time::Duration::from_millis(1000) {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => { }
                    }
                    std::process::exit(0);
                },
                LocalBackendRequest::InspectFilesystem(_fs_type, _path) => {
                    todo!()
                },
                LocalBackendRequest::ListDirectory(fs, path) => {
                    println!("Backend tasked with: Listing directory {:?}", &path);
                    let result = match fs {
                        FilesystemData::Local => {
                            list_dir_local_fs(&fs, path )
                        },
                        FilesystemData::LocalWSL => {
                            list_dir_local_wsl_fs(&fs, path)
                        },
                        //TODO: actually communicate with helix (over async-ssh2-challenge)
                        FilesystemData::HelixSSH(_) => {
                            todo!()
                        },
                        _ => todo!(),
                    };
                    comm_with_frontend.send(BackendResponse::ListDirectory(result)).ok();
                }
                //_ => println!("Backend tasked with: some task")
            },
            Err(_) => {
                println!("Communication with the backend has been closed.");
                break;
            }
        }*/
    }
}



pub async fn start_actix_server(address: String, port: u16,
                                server_state: AppState) {

    //TODO: implement timeout with a tokio::select! statement or similar
    tokio::spawn( async move {
        let state_data = Data::new(Mutex::new(server_state));

        println!("Listening on {}:{}...", &address, &port);
        let server_result = HttpServer::new(move || {
            let cors = Cors::default()
            //.allowed_origin("http://localhost:9283")
            .allowed_header(actix_web::http::header::AUTHORIZATION)
            .allowed_header(actix_web::http::header::ACCEPT)
            .allowed_header(actix_web::http::header::CONTENT_TYPE)
            .allowed_methods(vec!["GET", "POST"])
            .max_age(300);
            let payload_config = PayloadConfig::new(32*1024);

            App::new()
            .wrap(cors)
            .app_data(payload_config)
            .app_data(Data::clone(&state_data))
            .service(health_check)
        })
        .bind((address,port));


        //TODO: handle server start failure
        server_result.unwrap().run().await.ok();
    });
}














