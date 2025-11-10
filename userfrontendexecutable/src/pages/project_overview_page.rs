

use std::collections::HashMap;
use std::path::PathBuf;

use dioxus::prelude::*;

use crate::{AppState, FrontendCommChannel};
use crate::components::*;
use crate::backend;

use slotmap::{new_key_type, SlotMap};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum ContentType {
    Projects,
    Containers
}

#[derive(PartialEq, Hash, Debug, Clone)]
struct UserProject {
    pub name: String,
    pub description: String,
}

new_key_type! { struct ProjectKey; }

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
struct DummyContainer {
    pub name: String,
    pub path: PathBuf,
    pub id: ContainerId,
}

new_key_type! { struct ContainerKey; }

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct ContainerId {
    pub id: String, // could be cryptographic hash of Container instead, as soo as implemented
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
struct DummyExperiment {
    pub name: String,
    pub id: String,
    pub associated_container_id: ContainerId,
}

new_key_type! { struct ExperimentKey; }

#[component]
pub fn ProjectOverviewPage(app_state: Signal<AppState>, comm_with_backend: Signal<FrontendCommChannel>) -> Element {

    let selected_content_type = use_signal(|| ContentType::Projects);

    let mut project_list = use_signal(|| SlotMap::with_key() as SlotMap<ProjectKey,UserProject>);

    let _projects = use_signal(move || {
        (0..20).map(|index| {
            let description = "abc ".repeat(index*(index.max(10)));
            project_list.write().insert( UserProject { name: format!("Project {}", index+1), description } )
        }).collect::<Vec<_>>()
    } );

    let selected_project = use_signal(|| None as Option<ProjectKey>);


    let mut container_list = use_signal(|| SlotMap::with_key() as SlotMap<ContainerKey,DummyContainer>);

    let containers = use_signal(move || {
        (1..4).map(|id| {
            container_list.write().insert(
                DummyContainer {
                    name: format!("Container {}", id),
                    path: PathBuf::from(format!("/containers/container_{}", id)),
                    id: ContainerId { id: format!("{}", id) }
                })
        }).collect::<Vec<_>>()
    });

    let selected_container = use_signal(|| None as Option<ContainerKey>);

    let last_selected_container_dir = use_signal(|| None as Option<PathBuf>);

    let mut experiments = use_signal(|| SlotMap::with_key() as SlotMap<ExperimentKey, DummyExperiment>);

    let experiment_map = use_signal(move || {
        let ns = vec![3,1,0];

        let mut exps = HashMap::new() as HashMap<ContainerId, Vec<ExperimentKey>>;
        let mut k = 0;
        for (&cont_arena_id, &n) in containers().iter().zip(ns.iter()) {

            match container_list().get(cont_arena_id) {
                None => continue,
                Some(container) => {
                    let mut experiment_entries = Vec::new() as Vec<ExperimentKey>;

                    for _ in 0..n {
                        k += 1;
                        let dummyexp = DummyExperiment {
                            name: format!("Analysis {}", k),
                            id: format!("{}", k),
                            associated_container_id: container.id.clone()
                        };
                        experiment_entries.push( experiments.write().insert(dummyexp) );
                    }
                    exps.insert(container.id.clone(), experiment_entries);
                }
            }
        }
        exps
    });

    let selected_experiment = use_signal(|| None as Option<ExperimentKey>);

    rsx! {
        div {
            class: "project-overview-page background",
            // background_image: BACKGROUND_IMG,
            div {
                class: "project-overview-page logo-navbar-row",
                //"Logo-Navbar-Row"

                div {
                    class: "project-overview-page logo-row",
                    IMILogo { class: "navbar-logo" }
                    //p {"Logo-Row"}
                }
                div {
                    class: "project-overview-page large-navbar",
                    //p {"Navbar"}
                    HomeButton {app_state: app_state, class: "project-overview-page"}
                }
            }
            div {
                class: "project-overview-page content-area",
                ContentSelectionColumn { selected_content_type }
                match selected_content_type() {
                    ContentType::Projects => rsx! {
                        ProjectList { project_list, selected_project }
                    },
                    ContentType::Containers => rsx! {
                        ContainerOverview {
                            container_list, selected_container, last_selected_container_dir,
                            experiments, experiment_map, selected_experiment
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ContentSelectionColumn(selected_content_type: Signal<ContentType>) -> Element {

    rsx! {
        div {
            class: "project-overview-page content-selection-column",
            div {
                class: "project-overview-page content-selection-list",
                ContentTypeListEntry { selected_content_type, content_type: ContentType::Projects }
                ContentTypeListEntry { selected_content_type, content_type: ContentType::Containers }
            }
        }
    }
}

#[component]
fn ContentTypeListEntry(selected_content_type: Signal<ContentType>, content_type: ContentType) -> Element {
    rsx! {
        div {
            class: match selected_content_type() {
                _type if _type == content_type => "project-overview-page content-type-list-entry selected",
                _ => "project-overview-page content-type-list-entry"
            },
            onclick: move |_| {
                selected_content_type.set(content_type)
            },
            match content_type {
                ContentType::Projects => rsx! { h4 { class: "project-overview-page content-type-title", "Projects" } },
                ContentType::Containers => rsx! { h4 { class: "project-overview-page content-type-title", "Containers" } },
            }
        }
    }
}

#[component]
fn ProjectList(project_list: Signal<SlotMap<ProjectKey,UserProject>>, selected_project: Signal<Option<ProjectKey>>) -> Element {
    rsx! {
        div {
            class: "project-overview-page project-list",
            {
                project_list().iter()
                    .map(|(id, &ref elem)| rsx! {
                        ProjectWidget { class: "project-overview-page".to_string(), project: (id, elem.clone()), selected_project }
                    })
            }
        }
    }
}

#[component]
fn ProjectWidget(class: String, project: (ProjectKey, UserProject), selected_project: Signal<Option<ProjectKey>>) -> Element {
    rsx! {
        div {
            class: format!("{} project-widget-container", class),
            div {
                class: format!("{} project-widget-title-description-container", class),
                onclick: move |_| {
                    if selected_project() == Some(project.0) {
                        selected_project.set(None)
                    } else {
                        selected_project.set(Some(project.0))
                    }
                },
                div {
                    class: format!("{} project-widget-title-container", class),
                    h2 { class: format!("{} project-widget-title", class), {project.1.name}  }
                }
                div {
                    class: format!("{} project-widget-description-container", class),
                    p { class: format!("{} project-widget-title", class), {project.1.description} }
                }
            }
            svg {
                width: "16",
                height: "16",
                fill: "currentColor",
                class: format!("{} project-widget-delete-button mini-icon bi bi-x-square", class),
                view_box: "0 0 16 16",
                path {
                    d: "M14 1a1 1 0 0 1 1 1v12a1 1 0 0 1-1 1H2a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1zM2 0a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V2a2 2 0 0 0-2-2z"
                }
                path {
                    d: "M4.646 4.646a.5.5 0 0 1 .708 0L8 7.293l2.646-2.647a.5.5 0 0 1 .708.708L8.707 8l2.647 2.646a.5.5 0 0 1-.708.708L8 8.707l-2.646 2.647a.5.5 0 0 1-.708-.708L7.293 8 4.646 5.354a.5.5 0 0 1 0-.708"
                }
            }
        }
    }
}


#[component]
fn ContainerOverview(container_list: Signal<SlotMap<ContainerKey,DummyContainer>>,
                     selected_container: Signal<Option<ContainerKey>>,
                     last_selected_container_dir: Signal<Option<PathBuf>>,
                     experiments: Signal<SlotMap<ExperimentKey, DummyExperiment>>,
                     experiment_map: Signal<HashMap<ContainerId, Vec<ExperimentKey>>>,
                     selected_experiment: Signal<Option<ExperimentKey>>) -> Element {

    let experiment_list = match selected_container() {
        None => {
            let mut el = experiments().iter().map(|(id, &ref elem)| (id, elem.clone())).collect::<Vec<_>>();
            el.sort_unstable_by(|elem1, elem2| elem1.1.name.cmp(&elem2.1.name) );
            el
        },
        Some(cont_arena_id) => {
            match container_list().get(cont_arena_id) {
                None => {
                    selected_container.set(None);
                    let mut el = experiments().iter().map(|(id, &ref elem)| (id, elem.clone())).collect::<Vec<_>>();
                    el.sort_unstable_by(|elem1, elem2| elem1.1.name.cmp(&elem2.1.name) );
                    el
                },
                Some(container) => {
                    let mut el = experiment_map.write()
                        .entry(container.id.clone())
                        .or_insert_with(|| Vec::new() as Vec<ExperimentKey>)
                        .iter()
                        .filter_map(|key| experiments().get(*key).map(|elem| (*key, elem.clone())))
                        .collect::<Vec<(ExperimentKey, DummyExperiment)>>();
                    el.sort_unstable_by(|elem1, elem2| elem1.1.name.cmp(&elem2.1.name) );

                    if let Some(selected_exp) = selected_experiment() {
                        if !el.iter().any(|(key, _exp)| *key == selected_exp) {
                            selected_experiment.set(None);
                        }
                    }

                    el
                }
            }
        }
    };

    rsx! {
        div {
            class: "project-overview-page container-overview",
            div {
                class: "project-overview-page container-overview-box-aligner",
                div {
                    class: "project-overview-page container-selection-column",
                    div {
                        class: "project-overview-page container-selection-column-title-container-area",
                        h4 {
                            class: "project-overview-page container-selection-column-title",
                            "Container List"
                        }
                        div {
                            class: "project-overview-page container-selection-column-container-area",
                            {
                                container_list().iter().map(|(container_key, &ref cont)| {
                                    rsx! { ContainerWidget { container: cont.clone(), container_key, selected_container } }
                                })
                            }
                        }
                    }
                    AddContainerButton { containers: container_list, last_selected_container_dir }
                }
                div {
                    // there is apparently a bug here, and should read "project-overview-page experiment-selection-column"
                    // although this and "project-overview-page experiment-selection-column"
                    // are styled exactly the same, they behave differently
                    class: "project-overview-page experiment-selection-column",
                    div {
                        class: "project-overview-page experiment-selection-column-title-container-area",
                        h4 {
                            class: "project-overview-page experiment-selection-column-title",
                            "Previous Analyses"
                        }
                        div {
                            class: "project-overview-page experiment-selection-column-container-area",
                            {
                                experiment_list.into_iter().map(|(experiment_key, experiment)| {
                                    rsx! { ExperimentWidget { experiment, experiment_key, selected_experiment } }
                                })
                            }
                        }
                    }
                    NewExperimentButton { container_list, selected_container, experiments, selected_experiment }
                    InspectExperimentButton { container_list, selected_container, experiments, selected_experiment }
                }
            }
        }
    }
}

#[component]
fn ContainerWidget(container: DummyContainer, container_key: ContainerKey, selected_container: Signal<Option<ContainerKey>>) -> Element {
    rsx! {
        div {
            class: match selected_container() {
                Some(_key) if _key == container_key => "project-overview-page container-widget selected",
                _ => "project-overview-page container-widget"
            },
            onclick: move |_| {
                if selected_container() == Some(container_key) {
                    selected_container.set(None)
                } else {
                    selected_container.set(Some(container_key))
                }
            },
            { container.name }
        }
    }
}

//TODO: put Experiments in Slotmap
#[component]
fn ExperimentWidget(experiment: DummyExperiment, experiment_key: ExperimentKey, selected_experiment: Signal<Option<ExperimentKey>>) -> Element {
    rsx! {
        div {
            class: match selected_experiment() {
                Some(_key) if _key == experiment_key => "project-overview-page experiment-widget selected",
                _ => "project-overview-page experiment-widget"
            },
            onclick: move |_| {
                if selected_experiment() == Some(experiment_key) {
                    selected_experiment.set(None)
                } else {
                    selected_experiment.set(Some(experiment_key))
                }
            },
            { experiment.name.clone() }
        }
    }
}

#[component]
fn AddContainerButton(containers: Signal<SlotMap<ContainerKey, DummyContainer>>,
                      last_selected_container_dir: Signal<Option<PathBuf>>) -> Element {
    rsx! {
        button {
            class: "project-overview-page add-container-button primary-button",
            onclick: move |_| async move {
                let last_dir = last_selected_container_dir();
                // code is misleading; execution will happen in the frontend thread
                let _file = backend::choose_sif_file(&last_dir).await;
                //match file {
//                    Some(mut pthbuf) => {
//                        container_paths.write().push(pthbuf.clone());
//                        pthbuf.pop();
//                        last_selected_container_dir.set(Some(pthbuf));
//                        match config_path() {
//                            Some(cpath) => {
//                                write_persistent_state(&cpath, PersistentState {containers: container_paths(), last_selected_container_dir: last_selected_container_dir() })
//                            },
//                            None => {}
//                        }
//
//                    },
//                    None => {}
//                }
                println!("{:?}", containers().values());
            },
            {"Add local container"}
        }
    }
}

#[component]
fn NewExperimentButton(container_list: Signal<SlotMap<ContainerKey,DummyContainer>>,
                       selected_container: Signal<Option<ContainerKey>>,
                       experiments: Signal<SlotMap<ExperimentKey, DummyExperiment>>,
                       selected_experiment: Signal<Option<ExperimentKey>>) -> Element {
    rsx! {
        button {
            class: match selected_experiment() {
                Some(_) => "project-overview-page new-experiment-button primary-button hidden",
                None => "project-overview-page new-experiment-button primary-button",
            },
            onclick: move |_| {

            },
            {"New Experiment"}
        }
    }
}

#[component]
fn InspectExperimentButton(container_list: Signal<SlotMap<ContainerKey,DummyContainer>>,
                       selected_container: Signal<Option<ContainerKey>>,
                       experiments: Signal<SlotMap<ExperimentKey, DummyExperiment>>,
                       selected_experiment: Signal<Option<ExperimentKey>>) -> Element {
    rsx! {
        button {
            class: match selected_experiment() {
                Some(_) => "project-overview-page inspect-experiment-button primary-button",
                None => "project-overview-page inspect-experiment-button primary-button hidden",
            },
            onclick: move |_| {

            },
            {"Inspect Experiment"}
        }
    }
}
