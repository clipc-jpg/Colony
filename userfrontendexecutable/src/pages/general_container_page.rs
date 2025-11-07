




use std::collections::HashMap;
use std::path::{Path, PathBuf};

use dioxus::prelude::*;
use itertools::Itertools;
use tokio;
use uuid::Uuid;
extern crate chrono;
use chrono::prelude::DateTime;
use chrono::Utc;
use std::time::{UNIX_EPOCH, Duration};

use crate::pages::*;
use crate::backend::*;
use crate::backend::persistent_state::exe_dir;
use crate::components::*;

use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter};


#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ContainerDescription {
    pub id: Uuid,
    pub path: PathBuf,
    pub title: Option<String>,
    pub labels: Option<String>,
    pub help_section: Option<String>,
    pub apps: Option<Vec<String>>
}

impl ContainerDescription {
    pub fn from_backend(bcd: crate::backend::persistent_state::BackendContainerDescription) -> Self {
        return Self { id: bcd.id, path: bcd.path, title: None, labels: None, help_section: None, apps: None }
    }
}


#[derive(Clone, PartialEq, Debug)]
pub enum ContainerPageUpdate {
    //SetSelectedContainer(Uuid), // maybe not necessary, only one possible simultaneously
    //SetSelectedApp(String), // maybe not necessary, only one possible simultaneously
    SetSelectedContainerWorkdir(Uuid, PathBuf),
    SetSelectedContainerConfigPath(Uuid, PathBuf),
    AddContainer(PathBuf),
    RemoveContainer(PathBuf),
    StartSelfConfigurator(PathBuf),
}


#[derive(Clone, PartialEq, Debug)]
struct ContainerPageJobState {
    pub container_id: Uuid,
    pub container_path: PathBuf,
    pub job_id: JobId,
    pub state: crate::backend::JobState
}

#[component]
pub fn GeneralContainerPage(app_state: Signal<AppState>, comm_with_backend: Signal<FrontendCommChannel>) -> Element {

    let mut containers = use_signal(|| None as Option<HashMap<Uuid, ContainerDescription>>);
    let container_path_to_id = use_memo(move|| {

        if let Some(store) = containers() {
            for descr in store.values() {
                println!("Container: {:?}", descr);
            }
        }
        containers().map(|store| store.iter().map(|(id, descr)| (descr.path.clone(), *id) ).collect::<HashMap<_,_>>() )
    });

    let mut selected_container = use_signal(|| None as Option<Uuid>);
    let selected_app = use_signal(|| None as Option<String>);

    let mut workdir_paths = use_signal(|| HashMap::new() as HashMap<Uuid, Option<PathBuf>>);
    let mut config_paths = use_signal(|| HashMap::new() as HashMap<Uuid, Option<PathBuf>>);
    //TODO: args must be chosen both per container and per app
    let container_args = use_signal(|| HashMap::new() as HashMap<Uuid, Vec<(i32, String)>>);

    let mut running_job = use_signal(|| None as Option<ContainerPageJobState>);
    let mut copy_files_job = use_signal(|| None as Option<crate::backend::JobId>);

    let container_paths = use_memo(move || {
        let result = containers().map(|store| store.values().map(|descr| descr.path.clone()).collect_vec() );
        println!("Containers on GeneralStartPage: {result:?}");
        result
    });

    let selected_container_path = use_memo(move || {
        selected_container().map(|id| {
            containers().map(|store| store.get(&id).map(|descr| descr.path.clone()) ).flatten()
        }).flatten()
    });

    let selected_cont_workdir = use_memo(move || {
        selected_container().map(|id| {
            workdir_paths().get(&id).cloned().flatten()
        }).flatten()
          .filter(|p| Path::new(p).is_dir())
    });

    let selected_cont_config = use_memo(move || {
        selected_container().map(|id| {
            config_paths().get(&id).cloned().flatten()
        }).flatten()
          .filter(|p| Path::new(p).is_file())
    });

    let mut last_selected_container_dir = use_signal(|| None as Option<PathBuf>);
    let mut webserver_port = use_signal(|| None as Option<u16>);
    let job_observer_visible = use_signal(|| false);
    let mut backend_output = use_signal(|| Vec::new() as Vec<String>);

    let _clear_backend_output = use_effect(move || {
        if running_job().is_none() {
            backend_output.with_mut(|v| {
                // writing existing output_to file
                let outputfile_dir = selected_cont_workdir().or_else(|| exe_dir().clone()).expect("Could not determine File to write logs to.");

                // Try with timestamp first
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
                let mut filename = format!("raw_output_{}.log", timestamp);

                let mut counter = 1;
                let mut path = PathBuf::from(&outputfile_dir);
                path.push(filename);

                // If somehow file exists already, bump counter
                while path.exists() {
                    filename = format!("raw_output_{}_{}.log", timestamp, counter);
                    path = PathBuf::from(&filename);
                    counter += 1;
                }

                // Create file (fails if someone else just created it in between)
                let mut file = OpenOptions::new()
                    .write(true)
                    .create_new(true) // fail if exists
                    .open(&path);

                if file.is_ok() {
                    let mut writer = BufWriter::new(file.unwrap());
                    for line in v.iter() {
                        writer.write_all(line.as_bytes()).ok();
                        writer.write_all(b"\n").ok();
                    }
                }

                v.clear();
            })
        }
    });

    let _poll_backend_loop = use_future(move || {  async move {
            println!("Initializing entry page state!");
            if containers.peek().is_none() {
                println!("Sending Backend Request for Initialization");
                comm_with_backend.read().send(BackendRequest::ReadPersistentState).ok();
            }

            loop {
                let now = tokio::time::Instant::now();
                let dt = std::time::Duration::from_millis(200);
                tokio::time::sleep_until(now + dt).await;

                //Polling for answers
                match comm_with_backend.read().try_receive() {
                    Ok(BackendResponse::PersistentState(state)) => {
                        println!("Received message with persistent state");
                        // TODO: Does this introduce bugs, if PersistentState had already been read?
                        // what if a file/path is replaced by a container of newer version?
                        containers.set(Some(HashMap::new()));
                        state.containers.into_iter().map(ContainerDescription::from_backend)
                                .map(|descr| (descr.id, descr))
                                .for_each(|(id, mut descr)| {
                                    let container_path = descr.path.clone();
                                    spawn(async move {
                                        comm_with_backend.read().send(BackendRequest::QuerySingularity(container_path, SingularityQuery::AppList)).ok();
                                    });
                                    if descr.title.is_none() {
                                        let title = descr.path.components().last()
                                                         .map(|comp| comp.as_os_str().to_string_lossy().to_string())
                                                         .unwrap_or_else(|| "Unnamed".to_string());
                                        descr.title = Some(title);
                                    }
                                    containers.with_mut(|store| store.get_or_insert_with(HashMap::new).insert(id, descr));
                                });
                        last_selected_container_dir.set(state.last_selected_container_dir);
                        println!("Processed message with persistent state");
                    },
                    Ok(BackendResponse::LocalWebServerStarted(None)) => {
                        //TODO: webserver failed to start, give error message
                        //todo!();
                        webserver_port.set(None);
                    },
                    Ok(BackendResponse::LocalWebServerStarted(Some(port))) => {
                        //TODO: webserver successfully started, ignore for now
                        //TODO: start-self-configurator button should turn into a spinning icon now, at latest
                        //todo!();
                        webserver_port.set(Some(port));
                    },
                    Ok(BackendResponse::ContainerWebServerStarted(None)) => {
                        //TODO: webserver failed to start, give error message
                    },
                    Ok(BackendResponse::ContainerWebServerStarted(Some(_port))) => {
                        // portnumber (and later magic link suffix) need to be
                        // transmitted to container at some point
                        // Configuration messages should not be possible to be received before this section is matched
                        //webserver_port.set(Some(port)); // This is wrong (webserver)
                        // send BackendRequest::GetWebServerAddress(JobId)
                        // match on BackendResponse::WebServerAddress
                        // set webserver_port to the contained value
                    },
                    Ok(BackendResponse::Configuration(target_container, configuration)) => {
                        let cont_id = container_path_to_id().map(|store| store.get(&target_container).cloned()).flatten();
                        match cont_id {
                            None => {},
                            Some(id) => {
                                config_paths.write().insert(id, Some(PathBuf::from(configuration)));
                            }
                        }
                    },
                    Ok(BackendResponse::AddedNewContainer(bcd)) => {
                        println!("Adding new container...");
                        containers.with_mut(|store_maybeinit| {
                            if store_maybeinit.is_some() {

                                //this must not be - the backend gives out ids
                                let mut new_container = ContainerDescription::from_backend(bcd);
                                let container_id = new_container.id;
                                let container_path = new_container.path.clone();
                                let new_container_title = Some(new_container.path.components().last().map(|comp| {
                                        comp.as_os_str().to_string_lossy().to_string()
                                    }).unwrap_or_else(|| "Unnamed".to_string()));
                                new_container.title = new_container_title;
                                // hash collisions need to be prevented inside the backend
                                store_maybeinit.as_mut().unwrap().insert(container_id, new_container);
                                println!("Added new container at: {:?}", &container_path);
                                spawn(async move {
                                    comm_with_backend.read().send(BackendRequest::QuerySingularity(container_path, SingularityQuery::AppList)).ok();
                                });
                            } else {
                                println!("Could not add container; store had not been initialized");
                            }
                        });
                    },
                    Ok(BackendResponse::SingularityInfo(target_container, Some(SingularityResponse::AppList(app_list)))) => {
                        let cont_id = container_path_to_id().map(|store| store.get(&target_container).cloned()).flatten();

                        match cont_id {
                            None => {},
                            Some(id) => {
                                containers.with_mut(|store_option| {
                                    if let Some(ref mut store) = store_option {
                                        store.entry(id).and_modify(|descr| descr.apps = Some(app_list));
                                    }
                                });
                            }
                        }
                    },
                    Ok(BackendResponse::JobInfo(job_id, job_state)) => {
                        //TODO: abstraction is wrong, incorrect information may be displayed
                        //TODO: what if the user attempts to run multiple jobs simultaneously?

                        //secondly, request new update
                        match job_state {
                            //do nothing
                            JobState::Completed(_) => {
                                if copy_files_job().is_some() && running_job().is_some() {
                                    if copy_files_job().unwrap() == running_job().unwrap().job_id &&
                                        selected_container().is_some_and(|cont_id| cont_id == running_job().unwrap().container_id)
                                    {
                                        copy_files_job.set(None);
                                        match (selected_cont_workdir(), selected_cont_config()) {
                                            (Some(workdir), Some(config_file)) => {
                                                if config_file.components().last().is_some() {
                                                    let config_file_name = config_file.components().last().unwrap().clone();
                                                    let mut new_config_path = workdir.clone();
                                                    new_config_path.push(config_file_name.clone());

                                                    config_paths.with_mut(|store| {
                                                        store.insert(selected_container().unwrap(), Some(new_config_path));
                                                    });
                                                }
                                            },
                                            _ => {}
                                        };
                                    }
                                    // else do not note anything has changed
                                }
                            },
                            JobState::JobNotListed => {
                                if running_job().is_some_and(|job| job.job_id == job_id ) { running_job.set(None); }
                            },
                            //continue polling
                            _ =>  { comm_with_backend.read().send(BackendRequest::SendJobInfo(job_id)).ok(); }//TODO: reliably handle failures without spamming the queue
                        }

                        // firstly, accept and process message
                        if running_job().is_some_and(|prev_job_state| prev_job_state.job_id == job_id) {
                            running_job.with_mut(|prev_job_state| {
                                if let Some(prev_job_state) = prev_job_state {
                                    //println!("Setting Job {:?} to: {:?}", &prev_job_state.job_id, &job_state);
                                    prev_job_state.state = job_state;
                                }
                            });
                        } else { // Job may be the first one, or a different one: INCORRECT BEHAVIOUR
                            match (selected_container(), selected_container_path()) {
                                (Some(id), Some(path)) => {
                                    let new_job_state = ContainerPageJobState {
                                        container_id: id, //maybeuninit
                                        container_path: path.clone(), //maybeuninit
                                        job_id,
                                        state: job_state,
                                    };
                                    //println!("Setting Job {:?} to: {:?}", &job_id, &new_job_state);
                                    running_job.set(Some(new_job_state));
                                },
                                _ => {
                                    println!("Retrieveing Job state errored: No container selected!")
                                }
                            }
                        }
                    },
                    Ok(BackendResponse::JobOutput(backend_job_id, output)) if running_job().is_some_and(|job| job.job_id == backend_job_id) => {
                        backend_output.with_mut(move |lines| {
                            for new_line in output.into_iter() {
                                lines.push(new_line);
                            }
                        });
                        comm_with_backend.read().send(crate::BackendRequest::SendJobOutput(backend_job_id, backend_output().len())).ok();
                    },
                    Ok(response) => {println!("GeneralContainerPage received: {response:?}");}, //TODO: send unrelated messages back or error out
                    Err(_) => {}
                }

            }
        }}
    );

    let mut stop_entire_program_overlay_visible = use_signal(|| false);

    let mut page_updates = use_signal(|| Vec::new() as Vec<ContainerPageUpdate>);

    let _update_event = use_future(move || async move {
        loop {
            let now = tokio::time::Instant::now();
            let dt = std::time::Duration::from_millis(500);
            tokio::time::sleep_until(now + dt).await;

            if running_job().is_some_and(|job| job.state == JobState::JobNotListed) { running_job.set(None); }

            if !page_updates.read().is_empty() {
                for update in page_updates().iter() {
                    match update {
                        ContainerPageUpdate::SetSelectedContainerWorkdir(container_id, workdir) => {
                            workdir_paths.with_mut(|store| {
                                store.insert(*container_id, Some(workdir.clone()));
                            });
                        },
                        ContainerPageUpdate::SetSelectedContainerConfigPath(container_id, config_path) => {
                            config_paths.with_mut(|store| {
                                store.insert(*container_id, Some(config_path.clone()));
                            });
                        }
                        ContainerPageUpdate::AddContainer(ptb) => {
                            println!("containers: {:?}", containers());
                            // only the backend may give containers  ids => send path to backend first
                            // (backend has full knowledge and may help limit duplications)

                            comm_with_backend.read().send(BackendRequest::UpdatePersistentState(persistent_state::PersistentStateUpdate::AddContainer(ptb.clone()))).ok();

                        },
                        ContainerPageUpdate::RemoveContainer(ptb) => {
                            println!("containers: {:?}", containers());
                            containers.with_mut(|store_maybeinit| {
                                if store_maybeinit.is_some() {
                                    let key_maybe = store_maybeinit.as_ref().unwrap().iter()
                                                .find(|(&_key, &ref descr)| descr.path == *ptb)
                                                .map(|(key, _descr)| key.clone());
                                    if let Some(key) = key_maybe {
                                        store_maybeinit.as_mut().unwrap().remove(&key);
                                        comm_with_backend.read().send(BackendRequest::UpdatePersistentState(persistent_state::PersistentStateUpdate::RemoveContainer(key))).ok();
                                    }
                                }
                            })
                        },
                        ContainerPageUpdate::StartSelfConfigurator(container_pth) => {
                            if webserver_port().is_none() {
                                println!("No webserver running!");
                                comm_with_backend.read().send(BackendRequest::StartLocalWebServer(app_state().clone(), container_pth.clone())).ok();
                            }
                            while webserver_port().is_none() {
                                let now = tokio::time::Instant::now();
                                let dt = std::time::Duration::from_millis(200);
                                tokio::time::sleep_until(now + dt).await;
                            }

                            comm_with_backend.read().send(BackendRequest::RunSingularityApp(PathBuf::from("~"), container_pth.clone(), "self-configurator".to_string(), vec!["--colony-interop".to_string()])).ok();
                            let ip_address = format!("http://localhost:{:?}", &webserver_port());
                            println!("Webserver supposedly at: {}", &ip_address);
                            let ip_address = format!("http://localhost:9283/");

                            println!("Connecting to webserver at: {}", &ip_address);
                            app_state.set(AppState::ContainerSelfConfiguratorPage(container_pth.clone(), ip_address));
                        },
                    }
                }

                page_updates.with_mut(|updates| updates.clear());
            }
        }
    });


    rsx! {
        div {
            class: "general-container-page background",
            // background_image: BACKGROUND_IMG,
            div {
                class: "general-container-page logo-navbar-row",
                div {
                    class: "general-container-page logo-row",
                    IMILogo { class: "navbar-logo" }
                }
                div {
                    class: "general-container-page large-navbar",
                    HomeButton {app_state, class: "general-container-page"}
                }
            }
            div {
                class: "general-container-page content-area",
                StopEntireProgramOverlay { comm_with_backend, running_job, is_visible: stop_entire_program_overlay_visible }
                JobObserverOverlay { is_visible: job_observer_visible, running_job, working_directory: selected_cont_workdir, backend_output, stop_entire_program_overlay_visible, comm_with_backend }
                div {
                    class: "general-container-page overview-column",
                    {
                        match containers() {
                            None => rsx! { div {} },
                            Some(conts) => rsx! {
                                {
                                    conts.iter().map(|cont_descr| {
                                        let (_id, container_description) = cont_descr;
                                        rsx! {
                                            ContainerAppCard {
                                                class: "general-container-page".to_string(),
                                                app_state: app_state,
                                                page_updates: page_updates,
                                                container: container_description.clone(),
                                                selected_container: selected_container,
                                                selected_app: selected_app,
                                                comm_with_backend: comm_with_backend
                                            }
                                        }
                                    })
                                }
                            }
                        }
                    }
                    button {
                        class: "general-container-page entry-page add-container-button primary-button",
                        onclick: move |_| async move  {
                            //println!("Button clicked!");
                            println!("Selecting sif file at {:?}", &last_selected_container_dir());
                            let file = crate::backend::choose_sif_file(&last_selected_container_dir());
                            match file {
                                Some(mut pthbuf) => {
                                    page_updates.with_mut(|v| v.push(ContainerPageUpdate::AddContainer(pthbuf.clone())));
                                    pthbuf.pop();
                                    last_selected_container_dir.set(Some(pthbuf));
                                },
                                None => {}
                            }
                            println!("Containers: {:?}", container_paths());
                        },
                        {"Add local container"}
                    }

                    button {
                        class: match selected_container() {
                            Some(_) => "general-container-page remove-container-button primary-button",
                            None => "general-container-page remove-container-button primary-button disabled-button hidden"
                        },
                        onclick: move |_| {
                            match selected_container_path() {
                                Some(pthbuf) => {
                                    page_updates.with_mut(|v| v.push(ContainerPageUpdate::RemoveContainer(PathBuf::from(pthbuf))));
                                    selected_container.set(None);
                                },
                                None => {}
                            }
                            println!("{:?}", container_paths());
                        },
                        {"Remove selected container"}
                    }
                }
                div {
                    class: "general-container-page argument-column",
                    {
                        if selected_container().is_some() && selected_app().is_some_and(|app| app == "self-configurator") {
                            rsx! {
                                ConfigureSelfCard {
                                    class: "general-container-page".to_string(),
                                    app_state,
                                    selected_container,
                                    container_path: selected_container_path,
                                    workdir_path: selected_cont_workdir,
                                    configuration_path: selected_cont_config,
                                    webserver_port,
                                    comm_with_backend,
                                    page_updates,
                                }
                            }
                            // implicitly, main app will be chosen
                        } else if selected_container().is_some() {
                            rsx! {
                                CustomConfigCard {
                                    container_id: selected_container().unwrap(),
                                    workdir_path: selected_cont_workdir,
                                    container_args,
                                    page_updates
                                }
                            }
                        } else { rsx! { } }
                    }

                }
                div {
                    class: "general-container-page button-column",
                    p {""}

                    RunningContainerCard { job: running_job, working_directory: selected_cont_workdir, backend_output, comm_with_backend, job_observer_visible  }

                    button {
                        class: match (running_job(), selected_app(), selected_cont_workdir(), selected_cont_config()) {
                            // The button is hidden while a job is running
                            (Some(_), _, _, _) => "general-container-page export-analysis-button primary-button disabled-button hidden",
                            // The button may be shown, but only if the selected app is named "self-configurator"
                            (None, Some(app), None, _) if app == "self-configurator" => "general-container-page export-analysis-button primary-button disabled-button",
                            (None, Some(app), _, None) if app == "self-configurator" => "general-container-page export-analysis-button primary-button disabled-button",
                            // The button is only enabled if all information is set correctly
                            (None, Some(app), Some(_workdir), Some(_config_path)) if app == "self-configurator" => "general-container-page export-analysis-button primary-button",
                            // If the selected app is anything other than "self.configurator", the button is hidden
                            // also, rust needs a fallback case here
                            _ => "general-container-page export-analysis-button primary-button disabled-button hidden"
                        },
                        onclick: move |_| {
                            spawn(async move {
                                match (running_job(), selected_container_path(), selected_cont_workdir(), selected_cont_config()) {
                                    (None, Some(container), Some(workdir), Some(config_path)) => {
                                        let jid = crate::backend::JobId::new();
                                        match comm_with_backend.read().send(BackendRequest::ExportAnalysisIntoRepository(jid, workdir.clone(), container.clone(), config_path.clone())) {
                                            Ok(_) => { copy_files_job.set(Some(jid)); },
                                            Err(_) => { } //TODO
                                        }
                                    },
                                    _ => {}
                                }
                            });
                        },
                        {"Export Analysis"} br {} {"Into Repository"}
                    }
                    button {
                        class: match (running_job(), selected_app(), selected_cont_workdir(), selected_cont_config()) {
                            // This button is visible at all times
                            // While a job is running, the button is shown, but disabled
                            (Some(_), _, _, _) => "general-container-page start-container-button primary-button disabled-button",
                            // If an app is selected, and not "self-configurator", only the working directory is necessary to enable the button
                            (None, Some(app), Some(_workdir), _) if app != "self-configurator" => "general-container-page start-container-button primary-button",
                            // If an app is selected, and it is "self-configurator", both the working directory and the config file need to be present
                            (None, Some(app), Some(_workdir), Some(_config_path)) if app == "self-configurator" => "general-container-page start-container-button primary-button",
                            _ => "general-container-page start-container-button primary-button disabled-button"
                        },
                        onclick: move |_| {
                            spawn(async move {
                                if running_job().is_none() {
                                    match selected_app() {
                                        None => {
                                            match (selected_container(), selected_container_path(), selected_cont_workdir()) {
                                                (Some(cont_id), Some(cont_pth), Some(workdir))  => {
                                                    let args_maybe = container_args().get(&cont_id)
                                                                        .map(|vals| {
                                                                            vals.iter()
                                                                                .map(|(_, arg)| arg.clone())
                                                                                .collect_vec()
                                                                });
                                                    if let Some(args) = args_maybe {
                                                        println!("Running with args: {:?}", args);
                                                        comm_with_backend.read().sender.send(
                                                            BackendRequest::RunSingularity(workdir.clone(), cont_pth.clone(), args)
                                                        ).ok();
                                                    }
                                                },
                                                _ => { println!("Main app: Container: {:?}, Path: {:?}, Workdir: {:?}", &selected_container(), &selected_container_path(), &selected_cont_workdir()) }
                                            }
                                        },
                                        Some(app) if app == "self-configurator" => {
                                            match (selected_container_path(), selected_cont_workdir(), selected_cont_config()) {
                                                (Some(cont_pth), Some(workdir), Some(config))  => {
                                                    let args = vec![
                                                        wslify_windows_path(&config.to_string_lossy().to_string())
                                                    ];
                                                    println!("Running with args: {:?}", args);
                                                    comm_with_backend.read().sender.send(
                                                        BackendRequest::RunSingularity(workdir.clone(), cont_pth.clone(), args)
                                                    ).ok();
                                                },
                                                _ => { println!("Self-Configurator: Path: {:?}, Workdir: {:?}, Config: {:?}", &selected_container_path(), &selected_container_path(), &selected_cont_workdir()) }
                                            }
                                        },
                                        Some(app) => {
                                            match (selected_container(), selected_container_path(), selected_cont_workdir()) {
                                                (Some(cont_id), Some(cont_pth), Some(workdir))  => {
                                                    let args_maybe = container_args().get(&cont_id)
                                                                        .map(|vals| {
                                                                            vals.iter()
                                                                                .map(|(_, arg)| arg.clone())
                                                                                .collect_vec()
                                                                });
                                                    if let Some(args) = args_maybe {
                                                        println!("Running with args: {:?}", args);
                                                        comm_with_backend.read().sender.send(
                                                            BackendRequest::RunSingularityApp(workdir.clone(), cont_pth.clone(), app, args)
                                                        ).ok();
                                                    }
                                                },
                                                _ => { println!("App: {}, Container: {:?}, Path: {:?}, Workdir: {:?}", &app, &selected_container(), &selected_container_path(), &selected_cont_workdir()) }
                                            }
                                        }
                                    }
                                }
                            });
                        },
                        {"Start Container"}
                    }
                    button {
                        class: match running_job() {
                            Some(_) => "general-container-page stop-container-button primary-button",
                            None => "general-container-page stop-container-button primary-button hidden"
                        },
                        onclick: move |_| {
                            stop_entire_program_overlay_visible.set(true);
                            spawn(async move {
                                match running_job() {
                                    Some(job_state) => {
                                        match job_state.state {
                                            crate::backend::JobState::Submitted => {
                                                comm_with_backend.read().send(BackendRequest::StopProcess(job_state.job_id)).ok();
                                            },
                                            crate::backend::JobState::Scheduled => {
                                                comm_with_backend.read().send(BackendRequest::StopProcess(job_state.job_id)).ok();
                                            },
                                            crate::backend::JobState::Running => {
                                                comm_with_backend.read().send(BackendRequest::StopProcess(job_state.job_id)).ok();
                                            },
                                            _ => {},
                                        }
                                    },
                                    _ => {}
                                }
                            });
                        },
                        {"Stop Container"}
                    }
                }
            }
        }
    }
}


#[component]
pub fn ContainerAppCard(class: String, app_state: Signal<AppState>, page_updates: Signal<Vec<ContainerPageUpdate>>,
                        container: ContainerDescription,
                        selected_container: Signal<Option<Uuid>>, selected_app: Signal<Option<String>>,
                        comm_with_backend: Signal<FrontendCommChannel>) -> Element {

    let container_id = container.id;
    let container_selected = if selected_container().is_some_and(|selected| container.id == selected) { "selected" } else { "" };
    let apps_hidden = if selected_container().is_some_and(|selected| container.id == selected) { "" } else { "" };
    let app_list_initialized = container.apps.is_some();

    let mut pseudo_real_time = use_signal(||0 as i64);
    let icon_spin = use_memo(move || {
        let deg = pseudo_real_time * 15;
        format!("rotate({}deg)", deg)
    });

    use_future(move || async move {
        loop {
            let now = tokio::time::Instant::now();
            let dt = std::time::Duration::from_millis(30);
            tokio::time::sleep_until(now + dt).await;
            *pseudo_real_time.write() += 1;
        }
    });

    rsx! {
        div {
            class: format!("{class} container-app-card container-app-card-container {container_selected}"),
            h1 {
                class: format!("{class} container-app-card container-title"),
                {container.title}
            }
            div {
                class: format!("{class} container-app-card indent-container {apps_hidden}"),
                onclick: move |_| {
                    selected_container.set(Some(container_id));
                },
                div {
                    class: match selected_app() {
                        Some(_) => { format!("{class} container-app-card app-card") },
                        None => { format!("{class} container-app-card app-card {container_selected}") }
                    },
                    onclick: move |_| {
                        selected_app.set(None); // Main will always be chosen, indirectly
                    },
                    h3 {
                        class: format!("{class} container-app-card app-title"),
                        {"Main"}
                    }
                }
                div {
                    class: match app_list_initialized {
                        true => { format!("{class} container-app-card app-list-loading-icon-container medium-icon spinning-icon unclickable hidden") },
                        false => { format!("{class} container-app-card app-list-loading-icon-container medium-icon spinning-icon unclickable") }
                    },
                    transform: icon_spin,
                    LoadingIcon { class: format!("{class} container-app-card app-list-loading-icon medium-icon spinning-icon unclickable") }
                }
                {
                    match container.apps {
                        None => rsx! { div {} },
                        Some(app_list) => {
                            rsx! {
                                {
                                    app_list.into_iter().map(|name| {
                                        let selected = if selected_app().is_some_and(|app| app == name) { container_selected } else { "" };
                                        rsx! {
                                            div {
                                                class: format!("{class} container-app-card app-card {selected}"),
                                                onclick: move |_| {
                                                    if selected_app().is_some_and(|app| app == name) {
                                                        selected_app.set(None);
                                                    } else { selected_app.set(Some(name.to_string())); }
                                                },
                                                h3 {
                                                    class: "{class} container-app-card app-title",
                                                    {name.clone()}
                                                }
                                            }
                                        }
                                    })
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ConfigureSelfCard(class: String, app_state: Signal<AppState>,
                         selected_container: Signal<Option<Uuid>>,
                         container_path: Memo<Option<PathBuf>>,
                         workdir_path: Memo<Option<PathBuf>>,
                         configuration_path: Memo<Option<PathBuf>>,
                         webserver_port: Signal<Option<u16>>,
                         page_updates: Signal<Vec<ContainerPageUpdate>>,
                         comm_with_backend: Signal<FrontendCommChannel>) -> Element {

    let workdirinputfieldclass = format!("{class} configure-self-card workdir");
    let configinputfieldclass = format!("{class} configure-self-card config");

    let hidden = if container_path().is_none() { "hidden" } else { "" };
    let mut self_configurator_is_starting = use_signal(|| false);
    let self_config_button_disabled = use_memo(move || {
        if self_configurator_is_starting() { "disabled" } else { "" }
    });
    let waiting_for_self_configurator_hidden = use_memo(move || {
        if self_configurator_is_starting() { "" } else { "hidden" }
    });

    rsx! {
        div {
            class: format!("configure-self-card csc-container {class} {hidden}"),
            DirectoryInputField {
                class: workdirinputfieldclass,
                title: "Working Directory".to_string(),
                data: workdir_path,
                oninput: move |event_data| {
                    if let Some(container_id) = selected_container() {
                        let update = ContainerPageUpdate::SetSelectedContainerWorkdir(container_id, event_data);
                        page_updates.with_mut(|queue| queue.push(update) );
                    }
                }
            }
            FileInputField {
                class: configinputfieldclass,
                title: "Configuration".to_string(),
                data: configuration_path,
                oninput: move |event_data| {
                    if let Some(container_id) = selected_container() {
                        let update = ContainerPageUpdate::SetSelectedContainerConfigPath(container_id, event_data);
                        page_updates.with_mut(|queue| queue.push(update) );
                    }
                }
            }

            button {
                class: format!("configure-self-button primary-button {class} {self_config_button_disabled}"),
                onclick: move |_| {
                    spawn(async move {
                        match container_path() {
                            Some(cont) => {
                                page_updates.push(ContainerPageUpdate::StartSelfConfigurator(cont.clone()));
                                self_configurator_is_starting.set(true);
                            },
                            None => { println!("Error: self-configurator issued, but no no container had been selected"); }
                        }
                    });
                },
                match self_configurator_is_starting() {
                    false => rsx! { {"Create Configuration"} },
                    true => rsx! { SelfConfigSpinningIcon { class: class.clone(), hidden: waiting_for_self_configurator_hidden() } }
                }
            }

        }
    }
}

#[component]
pub fn SelfConfigSpinningIcon(class: String, hidden: String) -> Element {
    let mut pseudo_real_time = use_signal(||0 as i64);
    let icon_spin = use_memo(move || {
        let deg = pseudo_real_time * 15;
        format!("rotate({}deg)", deg)
    });

    use_future(move || async move {
        loop {
            let now = tokio::time::Instant::now();
            let dt = std::time::Duration::from_millis(30);
            tokio::time::sleep_until(now + dt).await;
            *pseudo_real_time.write() += 1;
        }
    });

    rsx! {
        div {
            //class: format!("waiting-for-self-configurator-container {class} {waiting_for_self_configurator_hidden}"),
            class: format!("waiting-for-self-configurator-container {class} rotating medium-icon icon-container {hidden}"),
            transform: icon_spin,
            LoadingIcon { class: "general-container-page spinning-icon medium-icon unclickable".to_owned() }
        }
    }
}

#[component]
pub fn CustomConfigCard(container_id: Uuid,
                        workdir_path: Memo<Option<PathBuf>>,
                        container_args: Signal<HashMap<Uuid, Vec<(i32, String)>>>,
                        page_updates: Signal<Vec<ContainerPageUpdate>>) -> Element {

    use_future(move || async move {
        container_args.with_mut(|store| {
            store.entry(container_id).or_insert_with(Vec::new);
        });
    });

    rsx! {
        div {
            class: "general-container-page argument-title",
            h3 {"Container Arguments"}

            svg {
                fill: "currentColor",
                class: "bi bi-file-plus-fill small-icon",
                view_box: "0 0 16 16",
                onclick: move |_| {
                    if container_args().keys().any(|key| *key == container_id) {
                        let k = container_args().get(&container_id).map(|v| {
                                v.iter().map(|(i,_)| i).max().unwrap_or(&0).clone()
                        }).unwrap().clone();
                        container_args.with_mut(|store| {
                            store.entry(container_id)
                                .and_modify(|v| { v.push((k+1, String::new())); } );
                        });
                    }
                    println!("{:?}", container_args());
                },
                path {
                    d: "M12 0H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V2a2 2 0 0 0-2-2M8.5 6v1.5H10a.5.5 0 0 1 0 1H8.5V10a.5.5 0 0 1-1 0V8.5H6a.5.5 0 0 1 0-1h1.5V6a.5.5 0 0 1 1 0"
                }
            }
        }

        div {
            class: "general-container-page alignment-container",
            DirectoryInputField {
                class: "general-container-page".to_string(),
                title: "Working Directory".to_string(),
                data: workdir_path,
                oninput: move |event_data| {
                    let update = ContainerPageUpdate::SetSelectedContainerWorkdir(container_id, event_data);
                    page_updates.with_mut(|queue| queue.push(update) );
                }
            }
            {
                if container_args().keys().any(|key| *key == container_id) {
                    rsx! {
                        {
                            let l = container_args().get(&container_id).unwrap().len();
                            (0..l).map(|i| {
                                let arg_key = container_args().get(&container_id).unwrap()[i].0;
                                rsx! {
                                    InputField { container_id, arg_id: arg_key, container_args: container_args }
                                }
                            })
                        }
                    }
                } else { rsx! { } }

            }
        }
    }
}


#[component]
pub fn FileInputField(class: String, title: String,
                      data: Memo<Option<PathBuf>>,
                      oninput: EventHandler<PathBuf>) -> Element {

    let starting_dir = use_signal(|| exe_dir().clone());
    let button_class = format!("{class} file-input");

    rsx! {
        div {
            class: format!("file-input fpi-container {class}"),
            h3 {
               class: format!("file-input title {class}"),
               {title}
            }
            div {
                class: format!("file-input input-field-container {class}"),
                input {
                    class: "file-input input-field",
                    class: "{class}",
                    r#type: "text", // TODO: when run without external launcher, type may be "file"
                    value: data().map(|ptb| ptb.to_string_lossy().to_string()).unwrap_or_else(String::new),
                    oninput: move |event| oninput.call(PathBuf::from(event.data.value())),
                }
                RequestFilePathButton2 {
                    class: button_class,
                    starting_dir: starting_dir,
                    result_handler: move |ptb| oninput.call(ptb)
                }
            }
        }
    }
}

#[component]
pub fn DirectoryInputField(class: String, title: String,
                           data: Memo<Option<PathBuf>>,
                           oninput: EventHandler<PathBuf>) -> Element {

    let starting_dir = use_signal(|| exe_dir().clone());
    let button_class = format!("{class} file-input");

    rsx! {
        div {
            class: format!("file-input fpi-container {class}"),
            h3 {
               class: format!("file-input title {class}"),
               {title}
            }
            div {
                class: format!("file-input input-field-container {class}"),
                input {
                    class: "file-input input-field",
                    class: "{class}",
                    r#type: "text", // TODO: when run without external launcher, type may be "file"
                    value: data().map(|ptb| ptb.to_string_lossy().to_string()).unwrap_or_else(String::new),
                    oninput: move |event| oninput.call(PathBuf::from(event.data.value())),
                }
                RequestDirectoryPathButton2 {
                    class: button_class,
                    starting_dir: starting_dir,
                    result_handler: move |ptb| oninput.call(ptb)
                }
            }
        }
    }
}








#[component]
pub fn InputField(container_id: Uuid, arg_id: i32, container_args: Signal<HashMap<Uuid, Vec<(i32, String)>>>) -> Element {

    if container_args().get(&container_id).is_none() {
        container_args.with_mut(|store| {
            store.entry(container_id).or_insert_with(Vec::new);
        })
    }

    let index: i32 = match container_args().get(&container_id).unwrap()
                                      .into_iter().position(|(id,_)| *id ==arg_id) {
        Some(k) => k as i32,
        None => -1
    };

    rsx! {
        h3 {
            class: "general-container-page input-field-title",
            {
                match index {
                    -1 => "".to_string(),
                    k => format!("Argument {}:", k+1)
                }
            }
        }
        div {
            class: "general-container-page general-input input-field-container",
            input {
                class: "general-container-page general-input input-field",
                value: {
                    match index {
                        -1 => "".to_string(),
                        k => container_args().get(&container_id).unwrap()[k as usize].1.clone()
                    }
                },
                oninput: move |event| {
                    container_args.with_mut(|store| {
                        store.entry(container_id).and_modify(|v| {
                            match index {
                                -1 => {},
                                k => {
                                    v[k as usize] = (v[k as usize].0, event.data.value());
                                }
                            }
                        });
                    });
                }
            }

            div {
                class: "general-container-page input-delete-icon small-icon icon-container",
                onclick: move |_| {
                    println!("Clicked argument: {}", arg_id);
                    container_args.with_mut(|store| {
                        store.entry(container_id)
                             .and_modify(|collection| collection.retain(|elem| elem.0 != arg_id) );
                    });
                },
                svg {
                    fill: "currentColor",
                    class: "bi bi-file-x-fill",
                    view_box: "0 0 16 16",
                    path {
                        d: "M12 0H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V2a2 2 0 0 0-2-2M6.854 6.146 8 7.293l1.146-1.147a.5.5 0 1 1 .708.708L8.707 8l1.147 1.146a.5.5 0 0 1-.708.708L8 8.707 6.854 9.854a.5.5 0 0 1-.708-.708L7.293 8 6.146 6.854a.5.5 0 1 1 .708-.708"
                    }
                }
            }
        }
    }
}





#[component]
fn RunningContainerCard(job: Signal<Option<ContainerPageJobState>>,
                        working_directory: Memo<Option<PathBuf>>,
                        backend_output: Signal<Vec<String>>,
                        comm_with_backend: Signal<FrontendCommChannel>,
                        job_observer_visible: Signal<bool>) -> Element {

    let hidden = if job().is_none() { "hidden" } else { "" };



    let job_order_time = job().map(|job_state| {
        let order_time = job_state.job_id.order_time;
        let order_system_time = UNIX_EPOCH + Duration::from_secs(order_time.seconds);
        let datetime = DateTime::<Utc>::from(order_system_time);
        let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
        timestamp_str
    });

    let mut pseudo_real_time = use_signal(||0 as i64);
    let icon_spin = use_memo(move || {
        let deg = pseudo_real_time * 15;
        format!("rotate({}deg)", deg)
    });

    let runtime_duration = use_memo(move || {
        pseudo_real_time(); // unused read hopefully never gets compiled away
        job().map(|job_state| {
            let order_time = job_state.job_id.order_time;
            let order_system_time = UNIX_EPOCH + Duration::from_secs(order_time.seconds);
            let duration_string = order_system_time.elapsed().ok().map(|duration| {
                let seconds = duration.as_secs() % 60;
                let minutes = (duration.as_secs() / 60) % 60;
                let hours = (duration.as_secs() / 60) / 60;
                let days = ((duration.as_secs() / 60) / 60) / 24;
                format!("{}D:{:0>2}H:{:0>2}M:{:0>2}S", days, hours, minutes, seconds)
            });
            duration_string
        }).flatten()
    });

    use_future(move || async move {
        loop {
            let now = tokio::time::Instant::now();
            let dt = std::time::Duration::from_millis(30);
            tokio::time::sleep_until(now + dt).await;
            *pseudo_real_time.write() += 1;
        }
    });

    rsx! {
        div {
            class: format!("general-container-page running-container-card card-container {hidden}"),
            match job().map(|job_state| job_state.state) {
                Some(JobState::Completed(_exit_status)) => {
                    rsx! {
                        h2 { text_align: "center",  "Job has completed" }
                        div {
                            display: "flex",
                            align_items: "center",
                            justify_content: "center",
                            onclick: move |_| {
                                job.set(None);
                            },
                            StoppedIcon { class: "general-container-page spinning-icon large-icon unclickable".to_owned() }
                        }
                        {
                            if job().is_some() {
                                rsx! {
                                    div {
                                        onclick: move |_| {
                                            job_observer_visible.set(true);
                                        },
                                        JobOutputWidget {
                                            class: "general-container-page running-container-card",
                                            job_id: job().map(|job_state| job_state.job_id).unwrap(),
                                            working_directory,
                                            displayed_text: backend_output,
                                            comm_with_backend
                                        }
                                    }
                                }
                            } else {
                                rsx! { }
                            }
                        }
                    }
                },
                Some(JobState::JobNotListed) => {
                    rsx!{ }
                }
                None => {
                    rsx! { }
                }
                _ => {
                    rsx! {
                        h2 { text_align: "center", "Job is running..." }
                        div {
                            display: "flex",
                            align_items: "center",
                            justify_content: "center",
                            transform: icon_spin,
                            onclick: move |_| {
                                job_observer_visible.set(true);
                            },
                            LoadingIcon {class: "general-container-page spinning-icon large-icon unclickable".to_owned()}
                        }
                        {
                            if job().is_some() {
                                rsx! {
                                    h4 {
                                        text_align: "center",
                                        { format!("Job started: {}", &job_order_time.unwrap()) }
                                    }
                                    h4 {
                                        text_align: "center",
                                        { runtime_duration }
                                    }
                                }
                            } else {
                                rsx! { }
                            }
                        }
                        {
                            if job().is_some() {
                                rsx! {
                                    div {
                                        onclick: move |_| {
                                            job_observer_visible.set(true);
                                        },
                                        JobOutputWidget {
                                            class: "general-container-page running-container-card",
                                            job_id: job().map(|job_state| job_state.job_id).unwrap(),
                                            working_directory,
                                            displayed_text: backend_output,
                                            comm_with_backend
                                        }
                                    }
                                }
                            } else {
                                rsx! { }
                            }
                        }
                    }
                }
            }
        }
    }
}



#[component]
fn JobObserverOverlay(is_visible: Signal<bool>,
                      running_job: Signal<Option<ContainerPageJobState>>,
                      working_directory: Memo<Option<PathBuf>>,
                      backend_output: Signal<Vec<String>>,
                      stop_entire_program_overlay_visible: Signal<bool>,
                      comm_with_backend: Signal<FrontendCommChannel>) -> Element {

    let running_job_id = use_memo(move || running_job().map(|job_state| job_state.job_id));

    match running_job_id() {
        Some(jid) => {
            rsx! {
                div {
                    class: match is_visible() {
                        true => "general-container-page job-observer-overlay",
                        false => "general-container-page job-observer-overlay-disabled hidden",
                    },

                    JobOutputWidget {
                        class: "general-container-page job-observer-overlay",
                        job_id: jid,
                        working_directory,
                        displayed_text: backend_output,
                        comm_with_backend
                    }
                    div {
                        class: "general-container-page job-observer-button-row",
                        button {
                            class: "general-container-page close-observer-overlay-button primary-button",
                            onclick: move |_| {
                                is_visible.set(false);
                            },
                            { "Close Overlay" }
                        }
                        button {
                            class: match running_job() {
                                Some(_) => "general-container-page job-observer-stop-container-button primary-button",
                                None => "general-container-page job-observer-stop-container-button primary-button hidden"
                            },
                            onclick: move |_| {
                                stop_entire_program_overlay_visible.set(true);
                                spawn(async move {
                                    match running_job() {
                                        Some(job_state) => {
                                            match job_state.state {
                                                crate::backend::JobState::Submitted => {
                                                    comm_with_backend.read().send(BackendRequest::StopProcess(job_state.job_id)).ok();
                                                },
                                                crate::backend::JobState::Scheduled => {
                                                    comm_with_backend.read().send(BackendRequest::StopProcess(job_state.job_id)).ok();
                                                },
                                                crate::backend::JobState::Running => {
                                                    comm_with_backend.read().send(BackendRequest::StopProcess(job_state.job_id)).ok();
                                                },
                                                _ => {},
                                            }
                                        },
                                        _ => {}
                                    }
                                });
                            },
                            {"Stop Container"}
                        }
                    }
                }
            }
        },
        None => {
            rsx! { }
        }
    }
}

// A hacky solution since I am not confident tunneling processes through windows c-api will work flawlessly
// Subprocesses on windows are tied to the console of the parent process, ergo one needs the process id of the console to be closed
// Sending a stop process signal (SIGINT, SIGTERM, SIGKILL) on windows is NOT possible
// In the future Colony will start some kind of server inside wsl once and all process manipulations shall be done inside linux
// Closing the entire application should (hopefully) eliminate all jobs inside wsl linux
#[component]
fn StopEntireProgramOverlay(comm_with_backend: Signal<FrontendCommChannel>, running_job: Signal<Option<ContainerPageJobState>> ,is_visible: Signal<bool>) -> Element {


    let no_overlay = rsx! { };
    let actual_overlay = rsx! {
        div {
            class: "general-container-page stop-program-overlay",
            div {
                class: "general-container-page stop-program-info-container",
                div {
                    h3 {
                        class: "general-container-page stop-program-info",
                        "Stopping all spawned processes currently requires terminating the whole program."
                        br { }
                        "Do you want to continue?"
                    }
                }
                div {
                    class: "general-container-page button-alignment",
                    button {
                        class: "general-container-page stop-program-button primary-button",
                        onclick: move |_| {
                            running_job.set(None);
                            comm_with_backend.read().send(BackendRequest::StopProgram).ok();
                        },
                        "Exit Program"
                    }
                    button {
                        class: "general-container-page cancel-stop-program-button primary-button",
                        onclick: move |_| {
                            is_visible.set(false)
                        },
                        "Cancel"
                    }
                }
            }
        }
    };

    match is_visible() {
        true => actual_overlay,
        false => no_overlay
    }
}























