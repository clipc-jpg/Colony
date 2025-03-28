

use std::path::PathBuf;
#[allow(unused)]
use std::collections::HashMap;

use dioxus::prelude::*;
use itertools::Itertools;

use crate::pages::*;
use crate::components::*;
use crate::backend::{FrontendCommChannel, BackendRequest, BackendResponse, SingularityQuery};
use crate::backend;
use crate::SingularityResponse;
use crate::backend::persistent_state::PersistentStateUpdate;


#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StartingContainerDestination {
    GenericConfigurator,
    ColonyConfigurator
}

//################################################################################
//## whole window components
//################################################################################

#[component]
pub fn EntryPage(app_state: Signal<AppState>, comm_with_backend: Signal<FrontendCommChannel>) -> Element {

    let mut container_paths = use_signal(|| None as Option<Vec<PathBuf>>);
    let mut last_selected_container_dir = use_signal(|| None as Option<PathBuf>);

    let _init_paths = use_future(move || async move {
        println!("Initializing entry page state!");
        if container_paths.peek().is_none() { comm_with_backend.read().send(BackendRequest::ReadPersistentState).ok(); }

        loop {
            let now = tokio::time::Instant::now();
            let dt = std::time::Duration::from_millis(200);
            tokio::time::sleep_until(now + dt).await;

            match comm_with_backend.read().try_receive() {
                Ok(msg) => {
                    match msg {
                        BackendResponse::PersistentState(state) => {
                            println!("Received message with persistent state");
                            last_selected_container_dir.set(state.last_selected_container_dir);
                            let backend_container_pathss = state.containers.into_iter()
                                    .map(|bcd| bcd.path)
                                    .collect_vec();
                            container_paths.set(Some(backend_container_pathss));
                            break;
                        },
                        _ => {}
                    }
                },
                Err(_) => {}
            }
        }
    });

    //let mut container_paths = use_signal(|| load_persistent_state(&config_path()).containers);
    let mut containerpath_updates = use_signal(|| Vec::new() as Vec<PersistentStateUpdate>);

    let _update_container = use_effect(move || {

        for update in containerpath_updates.read().iter() {
            match update {
                PersistentStateUpdate::AddContainer(ptb) => {
                    container_paths.with_mut(|cpaths| {
                        if cpaths.is_some() {
                            let cpths = cpaths.as_mut().unwrap();
                            cpths.push(ptb.clone());
                            comm_with_backend.read().send(BackendRequest::UpdatePersistentState(PersistentStateUpdate::AddContainer(ptb.clone()))).ok();
                        }
                    })
                },
                PersistentStateUpdate::RemoveContainerByPath(ptb) => {
                    container_paths.with_mut(|cpaths| {
                        if cpaths.is_some() {
                            let cpths = cpaths.as_mut().unwrap();
                            cpths.retain(|elem| *elem != *ptb);
                            comm_with_backend.read().send(BackendRequest::UpdatePersistentState(PersistentStateUpdate::RemoveContainerByPath(ptb.clone()))).ok();
                        }
                    })
                },
                _ => {} // not used on this page
            }
        }

        let remove_updates = !containerpath_updates.is_empty();
        if remove_updates { containerpath_updates.write().clear(); }
    });

    let container_names = use_memo(move || {
        let mut filenames = container_paths().unwrap_or_else(Vec::new)
                            .into_iter()
                            .map(|pb|  {
                                match pb.as_path().file_stem() {
                                    Some(osstr) => {osstr.to_string_lossy().to_string()},
                                    None => {pb.to_string_lossy().to_string()}
                                }
                            })
                            .collect::<Vec<_>>();
        let mut name_counts = container_paths().unwrap_or_else(Vec::new).into_iter().map(|_| 0 as usize).collect::<Vec<_>>();
        for k1 in 0..filenames.len() {
            for k2 in 0..k1 {
                if filenames[k1] == filenames[k2] {
                    name_counts[k1] += 1;
                }
            }
        };
        let cpaths = container_paths().unwrap_or_else(Vec::new);
        for k in 0..name_counts.len() {
            if name_counts[k] > 0 {
                filenames[k] = cpaths[k].to_string_lossy().to_string();
            }
        };
        filenames
    });

    let mut selected_container = use_signal(|| None as Option<String>);

    let _update_last_selected_container_dir = use_effect(move || {
        let new_val = last_selected_container_dir().clone();
        comm_with_backend.read().send(BackendRequest::UpdatePersistentState(PersistentStateUpdate::SetlastSelectedContainerDir(new_val))).ok();
    });

    let start_button_destination_hashmap = use_signal(|| HashMap::new() as HashMap<String, StartingContainerDestination>);

    let mut start_button_destination = use_signal(|| None as Option<(String, StartingContainerDestination)>);
    let _on_selcted_container_changed = use_memo(move || {
        start_button_destination.set(None);
        if let Some(selected_cont) = selected_container() {
            spawn(async move {
                let sbdh_binding = start_button_destination_hashmap();

                if sbdh_binding.contains_key(&selected_cont) {
                    let dest = sbdh_binding.get(&selected_cont).unwrap();
                    start_button_destination.set(Some((selected_cont.clone(), dest.clone())));
                } else {
                    comm_with_backend.read().send(BackendRequest::QuerySingularity(PathBuf::from(selected_cont.clone()), SingularityQuery::AppList)).ok();

                     let dest = loop {
                        let now = tokio::time::Instant::now();
                        let dt = std::time::Duration::from_millis(100);
                        tokio::time::sleep_until(now + dt).await;

                        match comm_with_backend.read().try_receive() {
                            Ok(BackendResponse::SingularityInfo(_container, Some(SingularityResponse::AppList(appl_string)))) => {
                                // TODO: How is the string actually formatted?
                                // is it better to enforce json format and deserialize the string into a hash map first?
                                break if appl_string.iter().any(|s| s == "self-configurator") {
                                    println!("Next page: ColonyConfigurator");
                                    StartingContainerDestination::ColonyConfigurator
                                } else {
                                    println!("Next page: GenericConfigurator");
                                    StartingContainerDestination::GenericConfigurator
                                }
                            },
                            Ok(resp) => {
                                println!("Unexpected backend response {:?}", resp);
                            }
                            //TODO: define latest response time and implement resignation
                            Err(_) => {} //no message currently ready
                        }
                    };
                    #[allow(warnings)]
                    start_button_destination_hashmap.write_silent().insert(selected_cont.clone(), dest);
                    start_button_destination.set(Some((selected_cont, dest.clone())));
                }
            });
        }
    });



    rsx! {
        div {
            class: "entry-page background",
            // background_image: BACKGROUND_IMG,
            div {
                class: "entry-page main-columns",
                div {
                    class: "entry-page presentation-login-column",
                    div {
                        class: "entry-page logo-row",
                        IMILogo {class: "entry-page navbar-logo"}
                    }
                    div {
                        class: "entry-page presentation-login-block",

                        div {
                            class: "entry-page presentation-background-logo",
                        }

                        div {
                            class: "entry-page login-column",
                            h1 {class: "entry-page main-title", {"Colony"} }

                            div {
                                class: match selected_container() {
                                        Some(_) => "entry-page container-start-button-container-background",
                                        None => "entry-page container-start-button-background"
                                },
                                button {
                                    class: match selected_container() {
                                        Some(_) => "entry-page container-start-button primary-button",
                                        None => "entry-page container-start-button primary-button disabled-button"
                                    },
                                    onclick: move |_| {
                                        match start_button_destination() {
                                            None => {println!("No target page determined yet")}, //TODO: waiting instead, and resignation with error message
                                            Some((pth, nxt_page)) => {
                                                match nxt_page {
                                                    StartingContainerDestination::ColonyConfigurator => {
                                                      println!("Entering Colony Configurator page");
                                                      app_state.set(AppState::ColonyStartPage(PathBuf::from(pth)));
                                                    },
                                                    StartingContainerDestination::GenericConfigurator => {
                                                      println!("Entering Generic Configurator page");
                                                        app_state.set(AppState::LocalStartPage(PathBuf::from(pth)));
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    {"Start container"}
                                }
                            }
                        }
                    }
                }

                //right = message column
                div {
                    class: "entry-page message-column",
                    //h3 { class: "entry-page message-column-header", {"Message column"} }
                    h3 { class: "entry-page message-column-header", {"Containers"} }
                    div {
                        class: "entry-page message-container",
                        ul { class: "entry-page message-ul",
                            {container_paths().clone().unwrap_or_else(Vec::new).into_iter()
                                              .zip(container_names().into_iter())
                                              .sorted().dedup()
                                              //TODO: use lossy utf-8 conversion
                                              .filter(|(pth,_)| pth.is_file() && !pth.to_str().is_none())
                                              .map(|(pth,ctn)| {
                                    let pth_string = Some(String::from(pth.to_string_lossy()));
                                    let path_separator = if std::env::consts::OS == "windows" {"\\"} else {"/"};
                                    let first_pth_component =
                                        match pth.components().next() {
                                            Some(c) => {
                                                match c.as_os_str().to_str() {
                                                    Some(s) => { format!("{}{}", s, if std::env::consts::OS == "windows" {"\\\\"} else {"/"}) },
                                                    None => {String::from("UNREADABLE")}
                                                }
                                            }
                                            None => {String::from("EMPTY PATH")}
                                    };

                                    rsx! {
                                        li {
                                            class: if pth_string == selected_container()
                                                     {"entry-page message-li message-li-selected"}
                                                else {"entry-page message-li"},
                                            onclick: move |_| {
                                                if pth_string == selected_container() {
                                                    *selected_container.write() = None;
                                                } else {
                                                    *selected_container.write() = pth_string.clone();
                                                }
                                            },
                                            {rsx! {
                                                h4 { class: "entry-page message-li-container-name", "{ctn}" }
                                                if pth_string == selected_container() {
                                                     {first_pth_component}
                                                     {
                                                        #[allow(unstable_name_collisions)]
                                                        pth.components()
                                                            .skip(1)
                                                            .map(|c| {
                                                                match c.as_os_str().to_str() {
                                                                    Some(s) => {s},
                                                                    None => {"UNREADABLE"}
                                                                }
                                                            })
                                                            .filter(|s| !s.is_empty() && s != &path_separator)
                                                            .map(|s|  rsx! { {s} } )
                                                            .intersperse(rsx! { wbr {} {path_separator} })
                                                    }
                                                }
                                            }}
                                        }
                                    }
                                })
                            }
                        }
                    }
                    button {
                        class: "entry-page add-container-button primary-button",
                        onclick: move |_| async move {
                            println!("Selecting sif file at {:?}", &last_selected_container_dir());
                            let file = backend::choose_sif_file(&last_selected_container_dir());
                            match file {
                                Some(mut pthbuf) => {
                                    //container_paths.write().push(pthbuf.clone());
                                    containerpath_updates.write().push(PersistentStateUpdate::AddContainer(pthbuf.clone()));
                                    pthbuf.pop();
                                    last_selected_container_dir.set(Some(pthbuf));
                                },
                                None => {}
                            }
                            println!("{:?}", container_paths());
                        },
                        {"Add local container"}
                    }
                    button {
                        class: match selected_container() {
                            Some(_) => "entry-page remove-container-button primary-button",
                            None => "entry-page remove-container-button primary-button disabled-button"
                        },
                        onclick: move |_| {
                            match selected_container() {
                                Some(pthbuf) => {
                                    containerpath_updates.write().push(PersistentStateUpdate::RemoveContainerByPath(PathBuf::from(pthbuf)));
                                    selected_container.set(None);
                                },
                                None => {}
                            }
                            println!("{:?}", container_paths());
                        },
                        {"Remove selected container"}
                    }
                }
            }
        }
    }
}






















