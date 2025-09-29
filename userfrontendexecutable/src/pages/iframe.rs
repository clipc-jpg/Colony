

use std::path::PathBuf;

use dioxus::prelude::*;

use crate::components::*;
use crate::pages::*;
use crate::backend::*;



#[component]
pub fn IFramePage(app_state: Signal<AppState>, title: String, source: String) -> Element {
    rsx! {
        div {
            class: "iframe background",
            // background_image: BACKGROUND_IMG,
            div {
                class: "iframe logo-navbar-row",
                //"Logo-Navbar-Row"
                div {
                    class: "iframe logo-row",
                    IMILogo { class: "navbar-logo" }
                    //p {"Logo-Row"}
                }
                div {
                    class: "iframe large-navbar",
                    //p {"Navbar"}
                    HomeButton {app_state: app_state, class: "iframe"}
                }
            }
            iframe {
                class: "iframe frame",
                title: title,
                src: source
            }
        }
    }
}


//################################################################################
//## Special Iframe for container self configuration
//## It can react appropriately to backend events
//################################################################################


#[component]
pub fn ContainerSelfConfiguratorPage(app_state: Signal<AppState>, comm_with_backend: Signal<FrontendCommChannel>, title: String, container: PathBuf, address: String) -> Element {

    use_future(move || {
        let container_send = container.clone();
        async move {
            loop {
                let now = tokio::time::Instant::now();
                let dt = std::time::Duration::from_millis(200);
                tokio::time::sleep_until(now + dt).await;

                // 1. wait for JobStarted message
                match comm_with_backend.try_read() {
                    Ok(message) => {
                        match message.try_receive() {
                            Ok(msg) => {
                                println!("Iframe received message");
                                match msg {
                                    BackendResponse::Configuration(target_container,config) => {
                                        println!("Iframe received Configuration");
                                        if target_container == container_send {
                                            comm_with_backend.read().send(BackendRequest::StartJob(container_send.clone(), config)).ok();
                                        } else {
                                            println!("Received configuration from a different container");
                                        }
                                    },
                                    BackendResponse::JobInfo(_job_id, _) => {
                                        // 2. redirect to GenericOberverPage, pass JobId
                                        println!("Iframe received JobInfo");
                                        //app_state.set(AppState::GenericObserverPage(job_id));
                                    },
                                    BackendResponse::SetAppState(new_page) => {
                                        app_state.set(new_page);
                                    }
                                    _ => {println!("Need to redirect unexpected messages");}
                                }
                            },
                            Err(_) => {}
                        }
                    },
                    Err(_) => {}
                }
            }
        }
    });

    rsx! {
        h1 { "This is the Self-Configurator" }
        IFramePage { app_state: app_state, title: title, source: address }
    }
}
















