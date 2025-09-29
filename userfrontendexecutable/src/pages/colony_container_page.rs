




use std::path::PathBuf;

use dioxus::prelude::*;
use tokio;

use crate::pages::*;
use crate::pages::local_container_page::*;
use crate::backend::persistent_state::exe_dir;
use crate::backend::*;
use crate::components::*;


#[component]
pub fn ColonyContainerPage(app_state: Signal<AppState>, comm_with_backend: Signal<FrontendCommChannel>) -> Element {

    let selected_container = use_signal(move || match app_state() {
        AppState::ColonyStartPage(ptb) => ptb,
        _ => panic!("Unreachable")
    });

    let container_description = use_signal(|| {
        ContainerDescription::empty(selected_container())
    });

    let workdir_path = use_signal(|| PathBuf::new());
    let mut config_pth = use_signal(|| PathBuf::new());

    let mut webserver_port = use_signal(|| None as Option<u16>);

    let mut hide_config_self_button = use_signal(|| false);

    //TODO: retrieve config
    //comm_with_backend.read().send(BackendRequest::StartLocalWebServer(app_state().clone(), selected_container().clone()));
    use_future(move || {
        let container_send = selected_container().clone();
        async move {
            loop {
                let now = tokio::time::Instant::now();
                let dt = std::time::Duration::from_millis(200);
                tokio::time::sleep_until(now + dt).await;
                match comm_with_backend.read().try_receive() {
                    Ok(BackendResponse::LocalWebServerStarted(None)) => {
                        //TODO: webserver failed to start, give error message
                        hide_config_self_button.set(false);
                    },
                    Ok(BackendResponse::LocalWebServerStarted(Some(_port))) => {
                        // portnumber (and later magic link suffix) need to be
                        // transmitted to container at some point
                        // Configuration messages should not be possible to be received before this section is matched
                        // send BackendRequest::GetWebServerAddress(JobId)
                        // match on BackendResponse::WebServerAddress
                        // set webserver_port to the contained value
                        comm_with_backend.read().send(BackendRequest::RunSingularityApp(PathBuf::from("~"), container_send.clone(), "self-configurator".to_string(), vec!["--colony-interop".to_string()])).ok();
                    },
                    Ok(BackendResponse::ContainerWebServerStarted(None)) => {
                        //TODO: webserver failed to start, give error message
                        hide_config_self_button.set(false);
                    },
                    Ok(BackendResponse::ContainerWebServerStarted(Some(port))) => {
                        // portnumber (and later magic link suffix) need to be
                        // transmitted to container at some point
                        // Configuration messages should not be possible to be received before this section is matched
                        webserver_port.set(Some(port));
                        hide_config_self_button.set(false);
                        // send BackendRequest::GetWebServerAddress(JobId)
                        // match on BackendResponse::WebServerAddress
                        // set webserver_port to the contained value
                    },
                    Ok(BackendResponse::Configuration(target_container, config)) => {
                        if target_container == container_send {
                            config_pth.set(PathBuf::from(config.clone()));
                        } else {
                            println!("Received config from a different container");
                        }
                    },
                    Ok(_) => {}, //TODO: send unrelated messages back or error out
                    Err(_) => {}
                }
            }
        }
    });

    rsx! {
        div {
            class: "colony-container-page background",
            // background_image: BACKGROUND_IMG,
            div {
                class: "colony-container-page logo-navbar-row",
                div {
                    class: "colony-container-page logo-row",
                    IMILogo { class: "navbar-logo" }
                }
                div {
                    class: "colony-container-page large-navbar",
                    HomeButton {app_state, class: "colony-container-page"}
                }
            }
            div {
                class: "colony-container-page content-area",
                div {
                    class: "colony-container-page overview-column",
                    DescriptionCard { selected_container: selected_container, container_description: container_description, comm_with_backend: comm_with_backend}
                }
                div {
                    class: "colony-container-page argument-column",
                    ConfigureSelfCard { class: "colony-container-page".to_string(), app_state, container: selected_container,
                           workdir_path, configuration_path: config_pth, hide_config_self_button, webserver_port, comm_with_backend }
                }
                div {
                    class: "colony-container-page button-column",
                    p {""}

                    button {
                        class: "colony-container-page start-container-button primary-button",
                        // checks if resource is available (e.g. Signal<bool> or Signal<Option<Instant>>)
                        // click locks the resouce
                        // second check currently is not necessary
                        // checks if container is there and if config file has been set, if not does not do anything
                        // otherwise spawns an async task
                        // task sends Runsingularity message with single argument (config path) to backend
                        // task enables some observerwidget/observerpage
                        // if appropriate (like page is not switched), task releases resource
                        onclick: move |_| {
                            spawn(async move {
                                let args = vec![
                                    wslify_windows_path(&config_pth().to_string_lossy().to_string())
                                ];
                                println!("Running with args: {:?}", args);
                                comm_with_backend.read().sender.send(
                                    BackendRequest::RunSingularity(workdir_path().clone(), selected_container().clone(), args)
                                ).ok();
                            });
                        },
                        {"Start Container"}
                    }
                }
            }
        }
    }
}


#[component]
pub fn ConfigureSelfCard(class: String, app_state: Signal<AppState>, container: Signal<PathBuf>,
                         workdir_path: Signal<PathBuf>,configuration_path: Signal<PathBuf>,
                         hide_config_self_button: Signal<bool>,
                         webserver_port: Signal<Option<u16>>, comm_with_backend: Signal<FrontendCommChannel>) -> Element {

    let workdirinputfieldclass = format!("{class} configure-self-card workdir");
    let configinputfieldclass = format!("{class} configure-self-card config");

    let config_self_button_hidden = use_memo(move || if hide_config_self_button() { "hidden" } else { "" });
    let config_self_suspense_hidden = use_memo(move || if hide_config_self_button() { "" } else { "hidden" });

    rsx! {
        div {
            class: format!("configure-self-card csc-container {class}"),
            DirectoryInputField { class: workdirinputfieldclass, title: "Working Directory".to_string(), data: workdir_path }
            FileInputField { class: configinputfieldclass, title: "Configuration".to_string(), data: configuration_path }

            div {
                class: format!("configure-self-button-suspense-icon medium-icon {class} {}", &config_self_suspense_hidden()),
                LoadingIcon { class: format!("") }
            }

            button {
                class: format!("configure-self-button primary-button {class} {}", &config_self_button_hidden()),
                onclick: move |_| {
                    spawn(async move {
                        // starts self-configurator
                        // waits for server to start (optional???)
                        // sets appstate to iframe with self-configurator url and backpage this appstate
                        // webserver reverts appstate to this webpage
                        // webserver tells this webpage there is a Configuration files
                        match webserver_port() {
                            Some(port) => {
                                //comm_with_backend.read().send(BackendRequest::RunSingularityApp(PathBuf::from("~"), container().clone(), "self-configurator".to_string(), vec!["--colony-interop".to_string()])).ok();
                                let ip_address = format!("http://localhost:{port}");
                                println!("Webserver supposedly at: {}", &ip_address);

                                println!("Connecting to webserver at: {}", &ip_address);
                                app_state.set(AppState::ContainerSelfConfiguratorPage(container().clone(), ip_address));
                            },
                            None => {
                                println!("No webserver running!");
                                comm_with_backend.read().send(BackendRequest::StartLocalWebServer(app_state().clone(), container().clone())).ok();
                                comm_with_backend.read().send(BackendRequest::RunSingularityApp(PathBuf::from("~"), container().clone(), "self-configurator".to_string(), vec!["--colony-interop".to_string()])).ok();
                                hide_config_self_button.set(true);

                            }
                        }
                    });
                },
                {"Start Self-Configuration"}
            }
        }
    }
}




#[component]
pub fn FileInputField(class: String, title: String, data: Signal<PathBuf>) -> Element {

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
                    value: data().to_string_lossy().to_string(),
                    oninput: move |event| {
                        data.set(event.data.value().into());
                    }
                }
                RequestFilePathButton { class: button_class, data: data,  starting_dir: starting_dir }
            }
        }
    }
}

#[component]
pub fn DirectoryInputField(class: String, title: String, data: Signal<PathBuf>) -> Element {

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
                    value: data().to_string_lossy().to_string(),
                    oninput: move |event| {
                        data.set(event.data.value().into());
                    }
                }
                RequestDirectoryPathButton { class: button_class, data: data,  starting_dir: starting_dir }
            }
        }
    }
}















