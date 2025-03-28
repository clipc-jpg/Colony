use std::path::PathBuf;


use dioxus::prelude::*;
use tokio;

use crate::pages::*;
use crate::pages::colony_container_page::*;
//use crate::backend;
use crate::backend::*;
use crate::components::*;

#[component]
pub fn LocalContainerPage(app_state: Signal<AppState>, comm_with_backend: Signal<FrontendCommChannel>) -> Element {

    let selected_container = use_signal(move || match app_state() {
        AppState::LocalStartPage(ptb) => ptb,
        _ => panic!("Unreachable")
    });
    let mut container_args = use_signal(|| Vec::new() as Vec<(i32,String)>);

    let container_description = use_signal(|| {
        ContainerDescription::empty(selected_container())
    });

    let workdir_path = use_signal(|| PathBuf::new());

    rsx! {
        div {
            class: "local-container-page background",
            // background_image: BACKGROUND_IMG,
            //"Background"
            div {
                class: "local-container-page logo-navbar-row",
                //"Logo-Navbar-Row"
                div {
                    class: "local-container-page logo-row",
                    IMILogo { class: "navbar-logo" }
                    //p {"Logo-Row"}
                }
                div {
                    class: "local-container-page large-navbar",
                    //p {"Navbar"}
                    StartLocalServerButton {class: "local-container-page", followup_page: app_state, comm_partner_container: selected_container, comm_with_backend: comm_with_backend}
                    SelfConfigDummyButton {app_state, class: "local-container-page"}
                    HtmlWrapperButton {app_state, class: "local-container-page", url: "https://icons.getbootstrap.com/"}
                    //LinkButton {app_state, class: "local-container-page", url: "https://icons.getbootstrap.com/"}
                    //LinkButton {app_state, class: "local-container-page", url: "https://helix-monitoring.bwservices.uni-heidelberg.de/"}
                    LinkButton {app_state, class: "local-container-page", url: "https://www.protocols.io/"}
                    TestButton {app_state, class: "local-container-page"}
                    ProjectOverviewPageButton {app_state, class: "local-container-page"}
                    ProjectPageButton {app_state, class: "local-container-page"}
                    SelfConfigButton {app_state, class: "local-container-page", container: selected_container(), url: "http://localhost:8080/"}
                    HomeButton {app_state, class: "local-container-page"}
                }
            }
            div {
                class: "local-container-page content-area",
                //"Content Area"
                div {
                    class: "local-container-page overview-column",
                    //p {"Overview-Column"}
                    DescriptionCard { selected_container: selected_container, container_description: container_description, comm_with_backend: comm_with_backend}
                }
                div {
                    class: "local-container-page argument-column",

                    div {
                        class: "local-container-page argument-title",
                        h3 {"Container Arguments"}

                        svg {
                            fill: "currentColor",
                            class: "bi bi-file-plus-fill small-icon",
                            view_box: "0 0 16 16",
                            onclick: move |_| {
                                let args = container_args();
                                let k = args.iter().map(|(i,_)| i).max().unwrap_or(&0);
                                container_args.write().push((k+1, String::new()));
                                println!("{:?}", container_args());
                            },
                            path {
                                d: "M12 0H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V2a2 2 0 0 0-2-2M8.5 6v1.5H10a.5.5 0 0 1 0 1H8.5V10a.5.5 0 0 1-1 0V8.5H6a.5.5 0 0 1 0-1h1.5V6a.5.5 0 0 1 1 0"
                            }
                        }
                    }

                    div {
                        class: "local-container-page alignment-container",
                        DirectoryInputField { class: "local-container-page".to_string(), title: "Working Directory".to_string(), data: workdir_path }
                        {
                            let l = container_args().len();
                            (0..l).map(|i| {
                                let arg_key = container_args()[i].0;
                                rsx! {
                                    InputField { arg_id: arg_key, arguments: container_args}
                                }
                            })
                        }
                    }
                }
                div {
                    class: "local-container-page button-column",
                    //p {"Button-Column"}
                    p {""}

                    button {
                        class: "local-container-page start-container-button primary-button",
                        onclick: move |_| {
                            spawn(async move {
                                let args = container_args().into_iter().map(|(_,s)| s).collect();
                                //backend::start_container(&pth, args);
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


//################################################################################
//## container description column
//################################################################################


#[derive(PartialEq, Eq, Clone)]
pub struct ContainerDescription {
    pub path: PathBuf,
    pub title: String,
    pub labels: String,
    pub help_section: String
}

impl ContainerDescription {
    pub fn empty(path: PathBuf) -> Self {
        return Self {path, title: "".to_string(), labels: "".to_string(), help_section: "".to_string()};
    }
}

#[component]
pub fn DescriptionCard(selected_container: Signal<PathBuf>, container_description: Signal<ContainerDescription>, comm_with_backend: Signal<FrontendCommChannel>) -> Element {

    use_future(move || {
        async move {

            match container_description().path.file_name() {
                Some(nm) => {container_description.write().title = nm.to_string_lossy().into_owned()},
                None => {}
            };

            println!("Requesting singularity inspection...");
            comm_with_backend.read().send(BackendRequest::QuerySingularity(selected_container(), SingularityQuery::Inspection)).ok();
            loop {
                let now = tokio::time::Instant::now();
                let dt = std::time::Duration::from_millis(200);
                tokio::time::sleep_until(now + dt).await;

                //TODO: make labels an array, then show each key-value-pair in the description card
                match comm_with_backend.read().try_receive() {
                    Ok(msg) => match msg {
                        BackendResponse::SingularityInfo(_container, Some(SingularityResponse::Inspection(json))) => {
                            //container_description.write().labels = format!("{}",json["data"]["attributes"]["labels"]);
                            let labels = json["data"]["attributes"]["labels"]
                                .as_object().unwrap()
                                .iter()
                                .filter(|(k,_)| !k.contains("org."))
                                .map(|(k,v)| {
                                    format!("{}: {}", k.clone(), v.as_str().unwrap())
                                })
                                .collect::<Vec<_>>()
                                .join("\n");
                            container_description.write().labels = labels;
                        },
                        BackendResponse::SingularityInfo(_container, None) => {println!("Singularity Inspection failed");},
                        msg => {
                            println!("Received something unexpected: {:?}", &msg);
                            comm_with_backend.read().reinsert_message(msg).ok();
                        }
                    },
                    Err(_) => {}//{println!("Received Nothing");}
                }
            }

        }
    });

    rsx! {
        div {
            class: "local-container-page description-card container",
            h1 {
                class: "local-container-page description-card title",
                {container_description().title}
            }
            h3 {
                class: "local-container-page description-card labels",
                {container_description().labels}
            }
            p {
                class: "local-container-page description-card help-section",
                {container_description().help_section}
            }
        }
    }
}

//################################################################################
//## argument column
//################################################################################

#[component]
pub fn InputField(arg_id: i32, arguments: Signal<Vec<(i32, String)>>) -> Element {

    let index: i32 = match arguments().into_iter().position(|(id,_)| id ==arg_id) {
        Some(k) => k as i32,
        None => -1
    };

    rsx! {
        h3 {
            class: "input-field-title",
            {
                match index {
                    -1 => "".to_string(),
                    k => format!("Argument {}:", k+1)
                }
            }
        }
        div {
            class: "input-field-container",
            input {
                class: "input-field",
                value: {
                    match index {
                        -1 => "".to_string(),
                        k => arguments()[k as usize].1.clone()
                    }
                },
                oninput: move |event| {
                    arguments.with_mut(|v| {
                        match index {
                            -1 => {},
                            k => {
                                v[k as usize] = (v[k as usize].0, event.data.value());
                            }
                        }
                    });
                }
            }

            div {
                class: "input-delete-icon small-icon icon-container",
                onclick: move |_| {
                    println!("Clicked argument: {}", arg_id);
                    arguments.set(arguments()
                                    .into_iter()
                                    .filter(|elem| elem.0 != arg_id)
                                    .collect());
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

















