


use std::fmt;
use std::path::PathBuf;
use std::cmp::Ordering;

use dioxus::prelude::*;
use dioxus_html::HasFileData;
use uuid::Uuid;

use crate::pages::*;
use crate::components::*;
use crate::backend::jobs::*;
use crate::persistent_state::exe_dir;
use crate::FrontendCommChannel;
use regex::Regex;

use crate::document::eval;

//#[derive(Clone, Copy)]
//struct SharedState {
//    dropped_files: Signal<Option<Vec<String>>>
//}

#[component]
pub fn LabBookPage(app_state: Signal<AppState>, comm_with_backend: Signal<FrontendCommChannel>) -> Element {

    //let widgets = use_signal(|| Vec::new() as Vec<Box<dyn ProjectPageWidget>>);

    //use_context_provider(|| SharedState { dropped_files: Signal::new(None as Option<Vec<String>>) });

    let mut widgets: Signal<Vec<ProjectPageWidget>> = use_signal(|| {
        let mut v = Vec::new() as Vec<ProjectPageWidget>;
        v.push(
            ProjectPageWidget::ParagraphElement(
                ParagraphElement {
                    uuid: Uuid::new_v4(),
                    order: 0.0,
                    text: Signal::new("Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua. Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua. Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua. ".to_string())
                }
            )
        );
        v.push(
            ProjectPageWidget::ParagraphElement(
                ParagraphElement {
                    uuid: Uuid::new_v4(),
                    order: 1.0,
                    text: Signal::new("Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua. Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua. Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua. ".to_string())
                }
            )
        );
        v.push(
            ProjectPageWidget::PlaceHolderElement(
                PlaceHolderElement { uuid: Uuid::new_v4(), order: 0.8 }
            )
        );
        v.push(
            ProjectPageWidget::MeasurementsElement(
                MeasurementsElement {
                    uuid: Uuid::new_v4(),
                    order: 5.0,
                    src: PathBuf::new(),
                    description: "Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua. Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua. ".to_string()
                }
            )
        );

        v
    });

    let widget_store_opened = use_signal(|| false);
    let mut dragged_widget = use_signal(|| ProjectPageWidgetChoice::None);
    let mut next_widget_position = use_signal(|| DragStatus::None);

    let max_order = use_memo(move || widgets().iter()
                                            .map(|elem| elem.element_order())
                                            .reduce(|acc, elem| acc.max(elem))
                                            .unwrap_or(0.0) );


    let widgets_ordered = use_memo(move || {

        let new_elem = match (*dragged_widget.peek(), next_widget_position()) {
            (ProjectPageWidgetChoice::None, _) => Option::None,
            (_, DragStatus::PlanDeletion) => Option::None,
            (_, DragStatus::DeleteElement(_)) => Option::None,
            //(_, DragStatus::Debug(_)) => Option::None,
            (_, DragStatus::None) =>            Some(ProjectPageWidget::PlaceHolderElement(PlaceHolderElement::new())),
            (_, DragStatus::DragHovered(pos)) => {
                let mut new_elem = ProjectPageWidget::PlaceHolderElement(PlaceHolderElement::new());
                new_elem.set_element_order(pos);
                Some(new_elem)
            },
            (elem, DragStatus::DragEnded(pos)) => {
                let mut new_elem = elem.to_projectpagewidget().expect("ProjectPageWidgetChoice::None should not occur here");
                new_elem.set_element_order(pos);
                Some(new_elem)
            },
        };

        match next_widget_position() {
            DragStatus::DragEnded(_) => {
                dragged_widget.set(ProjectPageWidgetChoice::None);
            },
            _ => {}
        }

        widgets.write().retain(|elem| {
            match elem {
                ProjectPageWidget::PlaceHolderElement(_) => false,
                ProjectPageWidget::None => false,
                elem => match next_widget_position() {
                    DragStatus::DeleteElement(delete_id) => {
                        delete_id != elem.id()
                    },
                    _ => true
                },
            }
        });
        if let Some(elem) = new_elem {
            widgets.write().push(elem);
        }

        widgets.write().sort_by(|w1,w2| {
            let w1o = w1.element_order();
            let w2o = w2.element_order();
            if w1o < w2o {Ordering::Less} else if w1o == w2o {Ordering::Equal} else {Ordering::Greater}
        });

        widgets.write().iter_mut().enumerate().for_each(|(k, elem)| {
            let pos = u32::try_from(k).map(|u| f64::from(u)).unwrap_or(f64::INFINITY);
            elem.set_element_order(pos);
        });

        widgets.peek().clone()
    });

    rsx! {
        div {
            class: "project-page background",
            // background_image: BACKGROUND_IMG,
            div {
                class: "project-page logo-navbar-row",
                //"Logo-Navbar-Row"
                div {
                    class: "project-page logo-row",
                    IMILogo { class: "project-page navbar-logo" }
                    //p {"Logo-Row"}
                }
                div {
                    class: "project-page large-navbar",
                    //p {"Navbar"}
                    HomeButton {app_state: app_state, class: "project-page"}
                }
            }
            div {
                class: "project-page content-area",
                WidgetStore { opened: widget_store_opened, dragged_widget, drag_status: next_widget_position }
                div {
                    class: "project-page project-area",
                    div {
                        class: "project-page title-area",
                        h1 {
                            class: "project-page title",
                            "Project Title"
                        }
                        button {
                            class: "project-page import-template-button primary-button",
                            "From Template"
                        }
                    }
                    div {
                        class: "project-page project-decription-area",
                        h3 {
                            class: "project-page project-description-title",
                            "Project description:"
                        }
                        textarea {
                            class: "project-page project-description-text",
                            //r#type: "text",
                            //name: "project-description"
                        }
                    }
                    div {
                        class: "project-page customized-area",
                        {widgets_ordered.read().iter()
                            .map(|w| w.to_element(next_widget_position))
                        }
                        div {
                            class: "project-page customized-area-bottom-padding",
                            onmouseover: move |_| {
                                if next_widget_position() != DragStatus::PlanDeletion {
                                    next_widget_position.set(DragStatus::DragHovered(max_order()+1.0));
                                }
                            }
                        }
                    }
                    div {
                        class: "project-page footer-area",
                        onmouseover: move |_| {
                            if next_widget_position() != DragStatus::PlanDeletion {
                                next_widget_position.set(DragStatus::DragHovered(max_order()+1.0));
                            }
                        },
                        button {
                            class: "project-page export-button primary-button",
                            "Export"
                        }
                    }
                }
            }
        }
    }
}

//################################################################################
//## Widget store
//################################################################################

#[component]
fn WidgetStore(opened: Signal<bool>, mut dragged_widget: Signal<ProjectPageWidgetChoice>, drag_status: Signal<DragStatus>) -> Element {
    let miniwidgets = make_widget_preview();

    rsx! {
        div {
            class: match opened() {
                true => "project-page widget-store opened",
                false => "project-page widget-store",
            },
            match opened() {
                true => rsx! {
                    svg {
                        width: "16",
                        height: "16",
                        fill: "currentColor",
                        class: "project-page close-widget-store-icon small-icon bi bi-chevron-compact-right",
                        view_box: "0 0 16 16",
                        onclick: move |_| {
                            drag_status.set(DragStatus::None);
                            opened.set(false);
                        },
                        path {
                            fill_rule: "evenodd",
                            d: "M6.776 1.553a.5.5 0 0 1 .671.223l3 6a.5.5 0 0 1 0 .448l-3 6a.5.5 0 1 1-.894-.448L9.44 8 6.553 2.224a.5.5 0 0 1 .223-.671"
                        }
                    }
                    div {
                        class: "project-page widget-store-icon-area",
                        {miniwidgets.read().iter().map(|w| rsx!{ {w.to_widget_icon(dragged_widget)} })}
                        svg {
                            width: "16",
                            height: "16",
                            fill: "currentColor",
                            class: match drag_status() {
                                DragStatus::PlanDeletion => "project-page delete-element-icon selected small-icon bi bi-x-circle-fill",
                                _ => "project-page delete-element-icon small-icon bi bi-x-circle-fill",
                            },
                            view_box: "0 0 16 16",
                            onclick: move |_| {
                                let new_status = match drag_status() {
                                    DragStatus::PlanDeletion => DragStatus::None,
                                    _ => DragStatus::PlanDeletion
                                };
                                drag_status.set(new_status);
                            },
                            path {
                                d: "M16 8A8 8 0 1 1 0 8a8 8 0 0 1 16 0M5.354 4.646a.5.5 0 1 0-.708.708L7.293 8l-2.647 2.646a.5.5 0 0 0 .708.708L8 8.707l2.646 2.647a.5.5 0 0 0 .708-.708L8.707 8l2.647-2.646a.5.5 0 0 0-.708-.708L8 7.293z"
                            }
                        }
                    }
                },
                false => rsx! {
                    svg {
                        width: "16",
                        height: "16",
                        fill: "currentColor",
                        class: "project-page open-widget-store-icon small-icon bi bi-chevron-compact-left",
                        view_box: "0 0 16 16",
                        onclick: move |_| opened.set(true),
                        path {
                            fill_rule: "evenodd",
                            d: "M9.224 1.553a.5.5 0 0 1 .223.67L6.56 8l2.888 5.776a.5.5 0 1 1-.894.448l-3-6a.5.5 0 0 1 0-.448l3-6a.5.5 0 0 1 .67-.223"
                        }
                    }
                }
            }
        }
    }
}

fn make_widget_preview() -> Signal<Vec<ProjectPageWidget>> {
    let widgets: Signal<Vec<ProjectPageWidget>> = Signal::new(
         {
        let mut v = Vec::new() as Vec<ProjectPageWidget>;
        v.push(
            ProjectPageWidget::ParagraphElement(
                ParagraphElement {uuid: Uuid::new_v4(), order: 1.0,  text: Signal::new(String::new()) }
            )
        );
        //v.push(
//            ProjectPageWidget::OrderedListElement(
//                OrderedListElement { order: 2.0, entries: Vec::new() as Vec<String> }
//            )
//        );
//        v.push(
//            ProjectPageWidget::UnorderedListElement(
//                UnorderedListElement { order: 3.0, entries: Vec::new() as Vec<String> }
//            )
//        );
        //v.push(
//            ProjectPageWidget::GridElement(
//                GridElement {
//                    uuid: Uuid::new_v4(),
//                    order: 4.0,
//                    elem_width: "4rem".to_string(),
//                    elem_height: "4rem".to_string(),
//                    elements: Vec::new()
//                }
//            )
//        );
        v.push(
            ProjectPageWidget::ImageElement(
                ImageElement {
                    uuid: Uuid::new_v4(),
                    order: 5.0,
                    src: Signal::new(Vec::new()),
                    display_mode: Signal::new(ImageDisplayMode::Grid(2))
                }
            )
        );
        v.push(
            ProjectPageWidget::MeasurementsElement(
                MeasurementsElement {
                    uuid: Uuid::new_v4(),
                    order: 4.0,
                    src: PathBuf::new(),
                    description: String::new()
                }
            )
        );
        v.push(
            ProjectPageWidget::AnalysisElement(
                AnalysisElement {
                    uuid: Uuid::new_v4(),
                    order: 5.0,
                    container: None,
                    data: Vec::new(),
                    output: Vec::new(),
                    job: None,
                    description: String::new()
                }
            )
        );
        v.push(
            ProjectPageWidget::ResultsElement(
                ResultsElement { uuid: Uuid::new_v4(), order: 6.0, src: PathBuf::new(), description: String::new() }
            )
        );

        v
    });

    return widgets;
}


//################################################################################
//## Trait for conversion into elements
//## and different pub structs holding information
//## drag events need to be defined here as well
//################################################################################


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ProjectPageWidgetChoice {
    None,
    ParagraphElement,
    OrderedListElement,
    UnorderedListElement,
    GridElement,
    ImageElement,
    MeasurementsElement,
    AnalysisElement,
    ResultsElement,
    //PlaceHolderElement, // not intended to exist
}

impl ProjectPageWidgetChoice {
    fn to_projectpagewidget(&self) -> Option<ProjectPageWidget> {
        match self {
            Self::None => Option::None,

            Self::ParagraphElement => Some(ProjectPageWidget::ParagraphElement(ParagraphElement::new())),
            Self::OrderedListElement => Some(ProjectPageWidget::OrderedListElement(OrderedListElement::new())),
            Self::UnorderedListElement => Some(ProjectPageWidget::UnorderedListElement(UnorderedListElement::new())),
            Self::GridElement => Some(ProjectPageWidget::GridElement(GridElement::new())),
            Self::ImageElement => Some(ProjectPageWidget::ImageElement(ImageElement::new())),
            Self::MeasurementsElement => Some(ProjectPageWidget::MeasurementsElement(MeasurementsElement::new())),
            Self::AnalysisElement => Some(ProjectPageWidget::AnalysisElement(AnalysisElement::new())),
            Self::ResultsElement => Some(ProjectPageWidget::ResultsElement(ResultsElement::new())),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DragStatus {
    None,
    DragHovered(f64),
    DragEnded(f64),
    PlanDeletion,
    DeleteElement(Uuid),
    //Debug(Uuid)
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectPageWidget {
    None,
    ParagraphElement(ParagraphElement),
    OrderedListElement(OrderedListElement),
    UnorderedListElement(UnorderedListElement),
    GridElement(GridElement),
    ImageElement(ImageElement),
    MeasurementsElement(MeasurementsElement),
    AnalysisElement(AnalysisElement),
    ResultsElement(ResultsElement),
    PlaceHolderElement(PlaceHolderElement),
}

impl ProjectPageWidgetTrait for ProjectPageWidget {
    fn to_element(&self, next_widget_position: Signal<DragStatus>) -> Element {
        match self {
            //Self::None => Err(RenderError::Aborted(())),
            Self::None => rsx! {},
            Self::ParagraphElement(elem) => { elem.to_element(next_widget_position) },
            Self::OrderedListElement(elem) => { elem.to_element(next_widget_position) },
            Self::UnorderedListElement(elem) => { elem.to_element(next_widget_position) },
            Self::GridElement(elem) => { elem.to_element(next_widget_position) },
            Self::ImageElement(elem) => { elem.to_element(next_widget_position) },
            Self::MeasurementsElement(elem) => { elem.to_element(next_widget_position) },
            Self::AnalysisElement(elem) => { elem.to_element(next_widget_position) },
            Self::ResultsElement(elem) => { elem.to_element(next_widget_position) },
            Self::PlaceHolderElement(elem) => { elem.to_element(next_widget_position) },
        }
    }

    fn id(&self) -> Uuid {
        match self {
            Self::None => {Uuid::nil()},
            Self::ParagraphElement(elem) => { elem.id() },
            Self::OrderedListElement(elem) => { elem.id() },
            Self::UnorderedListElement(elem) => { elem.id() },
            Self::GridElement(elem) => { elem.id() },
            Self::ImageElement(elem) => { elem.id() },
            Self::MeasurementsElement(elem) => { elem.id() },
            Self::AnalysisElement(elem) => { elem.id() },
            Self::ResultsElement(elem) => { elem.id() },
            Self::PlaceHolderElement(elem) => { elem.id() },
        }
    }

    fn element_order(&self) -> f64 {
        match self {
            Self::None => {20000.0},
            Self::ParagraphElement(elem) => { elem.element_order() },
            Self::OrderedListElement(elem) => { elem.element_order() },
            Self::UnorderedListElement(elem) => { elem.element_order() },
            Self::GridElement(elem) => { elem.element_order() },
            Self::ImageElement(elem) => { elem.element_order() },
            Self::MeasurementsElement(elem) => { elem.element_order() },
            Self::AnalysisElement(elem) => { elem.element_order() },
            Self::ResultsElement(elem) => { elem.element_order() },
            Self::PlaceHolderElement(elem) => { elem.element_order() },
        }
    }

    fn set_element_order(&mut self, new_order_key: f64) {
        match self {
            Self::None => {},
            Self::ParagraphElement(elem) => { elem.set_element_order(new_order_key) },
            Self::OrderedListElement(elem) => { elem.set_element_order(new_order_key) },
            Self::UnorderedListElement(elem) => { elem.set_element_order(new_order_key) },
            Self::GridElement(elem) => { elem.set_element_order(new_order_key) },
            Self::ImageElement(elem) => { elem.set_element_order(new_order_key) },
            Self::MeasurementsElement(elem) => { elem.set_element_order(new_order_key) },
            Self::AnalysisElement(elem) => { elem.set_element_order(new_order_key) },
            Self::ResultsElement(elem) => { elem.set_element_order(new_order_key) },
            Self::PlaceHolderElement(elem) => { elem.set_element_order(new_order_key) },
        }
    }

    fn to_widget_icon(&self, dragged_element: Signal<ProjectPageWidgetChoice>) -> Element {
        match self {
            //Self::None => Err(RenderError::Aborted(())),
            Self::None => rsx! {},
            Self::ParagraphElement(elem) => { elem.to_widget_icon(dragged_element) },
            Self::OrderedListElement(elem) => { elem.to_widget_icon(dragged_element) },
            Self::UnorderedListElement(elem) => { elem.to_widget_icon(dragged_element) },
            Self::GridElement(elem) => { elem.to_widget_icon(dragged_element) },
            Self::ImageElement(elem) => { elem.to_widget_icon(dragged_element) },
            Self::MeasurementsElement(elem) => { elem.to_widget_icon(dragged_element) },
            Self::AnalysisElement(elem) => { elem.to_widget_icon(dragged_element) },
            Self::ResultsElement(elem) => { elem.to_widget_icon(dragged_element) },
            Self::PlaceHolderElement(elem) => { elem.to_widget_icon(dragged_element) },
        }
    }
}


pub trait ProjectPageWidgetTrait {
    fn to_element(&self, next_widget_position: Signal<DragStatus>) -> Element;
    fn id(&self) -> Uuid;
    fn element_order(&self) -> f64;
    fn set_element_order(&mut self, new_order_key: f64);
    fn to_widget_icon(&self, dragged_element: Signal<ProjectPageWidgetChoice>) -> Element;
    //fn corresponding_choice(&self) ->ProjectPageWidgetChoice;
}

#[derive(Clone, PartialEq, Debug)]
pub struct ParagraphElement {
    pub uuid: Uuid,
    pub order: f64,
    pub text: Signal<String>
}

impl ParagraphElement {
    pub fn new() -> Self {
        return Self {
            uuid: Uuid::new_v4(),
            order : 0.0,
            text: Signal::new("Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua.".to_string())
        };
    }
}

impl ProjectPageWidgetTrait for ParagraphElement {
    fn to_element(&self, mut next_widget_position: Signal<DragStatus>) -> Element {
        let self_order = self.order;
        let self_id = self.uuid;
        let mut self_text = self.text;
        let mut to_be_deleted = use_signal(|| false);

        let mut debug_signal = use_signal(|| false);
        let mut debug_signal2 = use_signal(|| false);

        //let update_data = use_context::<UpdateData>();
        let update_data = use_signal(|| false);
        let mut auto_update_data = use_signal(|| false);
        use_effect(move || {
            // just setting a signal to anything will trigger the update
            update_data.read(); // update from outer logic (e.g. export)
            auto_update_data.read(); // will be written to by onmount-event, e.g. on every render and by onblur
            spawn( async move {
                let result = eval(&format!(r#"
                    let content = document.querySelector("[id='{self_id}']");
                    return content.innerHTML;
                "#));
                let par_pattern = Regex::new(r">(.*)\s*</p>").unwrap();
                let inner_html = result.await;
                println!("JavaSyript returned: {:?}", &inner_html);
                if let Some(par_content) = par_pattern.captures(&format!("{:?}", inner_html)) {
                    println!("Paragraph content: {:?}", &par_content[1]);
                    self_text.set(par_content[1].to_string());
                } else { println!("Error while parsing innerHTML!") }
            });
        });

        rsx! {
            div {
                id : format!("{}", self_id),
                class: match (to_be_deleted(), debug_signal(), debug_signal2()) {
                    (true,_, _) => "project-page paragraph-container widget-container to-be-deleted",
                    (false, true, _) => "project-page paragraph-container widget-container debug",
                    (false, false, true) => "project-page paragraph-container widget-container debug2",
                    _ => "project-page paragraph-container widget-container",
                },
                //these two do not work for some reason
                //ondragenter: move |evt| {
//                    println!("OnDragEnter fired on element: {}!", self_id);
//                    debug_signal2.set(true);
//                },
//                ondragend: move |evt| {
//                    println!("OnDragEnd fired on element: {}!", self_id);
//                    //next_widget_position.set(DragStatus::DragEnded(self_order));
//                },
                // these do work
                ondragover: move |_evt| {
                    println!("OnDragOver fired on element: {}!", self_id);
                    debug_signal2.set(true);
                },
                ondragleave: move |_evt| {
                    println!("OnDragLeave fired on element: {}!", self_id);
                    debug_signal2.set(false);
                },
                ondrop: move |_evt| {
                    println!("Drop fired on element: {}!", self_id);
                    debug_signal.toggle();
                },
                onmouseover: move |_evt| {
                    if next_widget_position() != DragStatus::PlanDeletion {
                        next_widget_position.set(DragStatus::DragHovered(self_order - 0.5));
                    } else { to_be_deleted.set(true); }
                },
                onmouseleave: move |_evt| {
                    if to_be_deleted() {to_be_deleted.set(false);}
                },
                onclick: move |_evt| async move {
                    if to_be_deleted() {
                        next_widget_position.set(DragStatus::DeleteElement(self_id));
                    }
                },
                onmounted: move |_element| {
                    auto_update_data.set(true); //value does not need to change
                },
                p {
                    class: "project-page paragraph-text",
                    contenteditable: true,
                    onkeydown: move |evt: KeyboardEvent| {
                        match evt.key() {
                            Key::Tab => {
                                evt.prevent_default();
                                evt.stop_propagation();
                            },
                            _ => {}
                        }
                    },
                    onblur: move |_| {
                        auto_update_data.set(true); //value does not need to change
                    },
                    {self_text.peek().clone()}
                }
            }
        }
    }

    fn id(&self) -> Uuid { return self.uuid; }

    fn element_order(&self) -> f64 { return self.order; }
    fn set_element_order(&mut self, new_order_key: f64) { self.order = new_order_key; }

    fn to_widget_icon(&self, mut dragged_element: Signal<ProjectPageWidgetChoice>) -> Element {
        let uuid = Uuid::new_v4();
        rsx! {
            div {
                id: format!("{}", uuid),
                class: match dragged_element() {
                    ProjectPageWidgetChoice::ParagraphElement => { "project-page icon-container selected" },
                    _ => { "project-page icon-container" }
                },
                //ondragstart works, but the rest of the events do not
                //draggable: true,
//                ondragstart: move |_| {
//                    dragged_element.set(ProjectPageWidgetChoice::ParagraphElement);
//                },
                onclick: move |_| {
                    match dragged_element() {
                        ProjectPageWidgetChoice::ParagraphElement => dragged_element.set(ProjectPageWidgetChoice::None),
                        _ => dragged_element.set(ProjectPageWidgetChoice::ParagraphElement)
                    }
                },
                svg {
                    width: "16",
                    height: "16",
                    fill: "currentColor",
                    class: "project-page paragraph-icon large-icon bi-body-text",
                    view_box: "0 0 16 16",
                    path {
                        fill_rule: "evenodd",
                        d: "M0 .5A.5.5 0 0 1 .5 0h4a.5.5 0 0 1 0 1h-4A.5.5 0 0 1 0 .5m0 2A.5.5 0 0 1 .5 2h7a.5.5 0 0 1 0 1h-7a.5.5 0 0 1-.5-.5m9 0a.5.5 0 0 1 .5-.5h5a.5.5 0 0 1 0 1h-5a.5.5 0 0 1-.5-.5m-9 2A.5.5 0 0 1 .5 4h3a.5.5 0 0 1 0 1h-3a.5.5 0 0 1-.5-.5m5 0a.5.5 0 0 1 .5-.5h5a.5.5 0 0 1 0 1h-5a.5.5 0 0 1-.5-.5m7 0a.5.5 0 0 1 .5-.5h3a.5.5 0 0 1 0 1h-3a.5.5 0 0 1-.5-.5m-12 2A.5.5 0 0 1 .5 6h6a.5.5 0 0 1 0 1h-6a.5.5 0 0 1-.5-.5m8 0a.5.5 0 0 1 .5-.5h5a.5.5 0 0 1 0 1h-5a.5.5 0 0 1-.5-.5m-8 2A.5.5 0 0 1 .5 8h5a.5.5 0 0 1 0 1h-5a.5.5 0 0 1-.5-.5m7 0a.5.5 0 0 1 .5-.5h7a.5.5 0 0 1 0 1h-7a.5.5 0 0 1-.5-.5m-7 2a.5.5 0 0 1 .5-.5h8a.5.5 0 0 1 0 1h-8a.5.5 0 0 1-.5-.5m0 2a.5.5 0 0 1 .5-.5h4a.5.5 0 0 1 0 1h-4a.5.5 0 0 1-.5-.5m0 2a.5.5 0 0 1 .5-.5h2a.5.5 0 0 1 0 1h-2a.5.5 0 0 1-.5-.5"
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct OrderedListElement {
    pub uuid: Uuid,
    pub order: f64,
    pub entries: Vec<String>
}

impl OrderedListElement {
    pub fn new() -> Self { return Self { uuid: Uuid::new_v4(), order: 0.0, entries: vec!["A".to_string(), "B".to_string(), "C".to_string()] }; }
}

impl ProjectPageWidgetTrait for OrderedListElement {
    fn to_element(&self, mut next_widget_position: Signal<DragStatus>) -> Element {
        let self_order = self.order;
        let self_id = self.uuid;
        let mut to_be_deleted = use_signal(|| false);

        rsx! {
            ol {
                class: match to_be_deleted() {
                    true => "project-page ordered-list widget-container to-be-deleted",
                    _ => "project-page ordered-list widget-container"
                },
                onmouseover: move |_evt| {
                    if next_widget_position() != DragStatus::PlanDeletion {
                        next_widget_position.set(DragStatus::DragHovered(self_order - 0.5));
                    } else { to_be_deleted.set(true); }
                },
                onmouseleave: move |_evt| {
                    if to_be_deleted() {to_be_deleted.set(false);}
                },
                onclick: move |_evt| {
                    if to_be_deleted() {
                        next_widget_position.set(DragStatus::DeleteElement(self_id));
                    }
                },
                {self.entries.iter().enumerate().map(|(k,elem)| rsx!{ li { p {{format!("{}. {}", k+1, &elem)}} } })}
            }
        }
    }

    fn id(&self) -> Uuid { return self.uuid; }

    fn element_order(&self) -> f64 { return self.order; }
    fn set_element_order(&mut self, new_order_key: f64) { self.order = new_order_key; }

    fn to_widget_icon(&self, mut dragged_element: Signal<ProjectPageWidgetChoice>) -> Element {
        let uuid = Uuid::new_v4();
        rsx! {
            div {
                id: format!("{}", uuid),
                class: match dragged_element() {
                    ProjectPageWidgetChoice::OrderedListElement => { "project-page icon-container selected" },
                    _ => { "project-page icon-container" }
                },
                onclick: move |_| {
                    match dragged_element() {
                        ProjectPageWidgetChoice::OrderedListElement => dragged_element.set(ProjectPageWidgetChoice::None),
                        _ => dragged_element.set(ProjectPageWidgetChoice::OrderedListElement)
                    }
                },
                svg {
                    width: "16",
                    height: "16",
                    fill: "currentColor",
                    class: "project-page ordered-list-icon large-icon bi bi-list-ol",
                    view_box: "0 0 16 16",
                    path {
                        fill_rule: "evenodd",
                        d: "M5 11.5a.5.5 0 0 1 .5-.5h9a.5.5 0 0 1 0 1h-9a.5.5 0 0 1-.5-.5m0-4a.5.5 0 0 1 .5-.5h9a.5.5 0 0 1 0 1h-9a.5.5 0 0 1-.5-.5m0-4a.5.5 0 0 1 .5-.5h9a.5.5 0 0 1 0 1h-9a.5.5 0 0 1-.5-.5"
                    }
                    path {
                        d: "M1.713 11.865v-.474H2c.217 0 .363-.137.363-.317 0-.185-.158-.31-.361-.31-.223 0-.367.152-.373.31h-.59c.016-.467.373-.787.986-.787.588-.002.954.291.957.703a.595.595 0 0 1-.492.594v.033a.615.615 0 0 1 .569.631c.003.533-.502.8-1.051.8-.656 0-1-.37-1.008-.794h.582c.008.178.186.306.422.309.254 0 .424-.145.422-.35-.002-.195-.155-.348-.414-.348h-.3zm-.004-4.699h-.604v-.035c0-.408.295-.844.958-.844.583 0 .96.326.96.756 0 .389-.257.617-.476.848l-.537.572v.03h1.054V9H1.143v-.395l.957-.99c.138-.142.293-.304.293-.508 0-.18-.147-.32-.342-.32a.33.33 0 0 0-.342.338zM2.564 5h-.635V2.924h-.031l-.598.42v-.567l.629-.443h.635z"
                    }
                }
            }
        }
    }
}



#[derive(Clone, PartialEq, Default, Debug)]
pub struct UnorderedListElement {
    pub uuid: Uuid,
    pub order: f64,
    pub entries: Vec<String>
}

impl UnorderedListElement {
    pub fn new() -> Self { return Self { uuid: Uuid::new_v4(), order: 0.0, entries: vec!["A".to_string(), "B".to_string(), "C".to_string()] }; }
}

impl ProjectPageWidgetTrait for UnorderedListElement {
    fn to_element(&self, mut next_widget_position: Signal<DragStatus>) -> Element {
        let self_order = self.order;
        let self_id = self.uuid;
        let mut to_be_deleted = use_signal(|| false);
        rsx! {
            ul {
                class: match to_be_deleted() {
                    true => "project-page unordered-list widget-container to-be-deleted",
                    _ => "project-page unordered-list widget-container"
                },
                onmouseover: move |_evt| {
                    if next_widget_position() != DragStatus::PlanDeletion {
                        next_widget_position.set(DragStatus::DragHovered(self_order - 0.5));
                    } else { to_be_deleted.set(true); }
                },
                onmouseleave: move |_evt| {
                    if to_be_deleted() {to_be_deleted.set(false);}
                },
                onclick: move |_evt| {
                    if to_be_deleted() {
                        next_widget_position.set(DragStatus::DeleteElement(self_id));
                    }
                },
                {self.entries.iter().map(|elem| {
                    rsx!{
                         li { p {
                             class: "project-page ordered-list item-text",
                             {format!("•  {}", &elem)}
                        } }
                    }
                })}
            }
        }
    }

    fn id(&self) -> Uuid { return self.uuid; }

    fn element_order(&self) -> f64 { return self.order; }
    fn set_element_order(&mut self, new_order_key: f64) { self.order = new_order_key; }

    fn to_widget_icon(&self, mut dragged_element: Signal<ProjectPageWidgetChoice>) -> Element {
        let uuid = Uuid::new_v4();
        rsx! {
            div {
                id: format!("{}", uuid),
                class: match dragged_element() {
                    ProjectPageWidgetChoice::UnorderedListElement => { "project-page icon-container selected" },
                    _ => { "project-page icon-container" }
                },
                onclick: move |_| {
                    match dragged_element() {
                        ProjectPageWidgetChoice::UnorderedListElement => dragged_element.set(ProjectPageWidgetChoice::None),
                        _ => dragged_element.set(ProjectPageWidgetChoice::UnorderedListElement)
                    }
                },
                svg {
                    width: "16",
                    height: "16",
                    fill: "currentColor",
                    class: "project-page unordered-list-icon large-icon bi bi-list-ul",
                    view_box: "0 0 16 16",
                    path {
                        fill_rule: "evenodd",
                        d: "M5 11.5a.5.5 0 0 1 .5-.5h9a.5.5 0 0 1 0 1h-9a.5.5 0 0 1-.5-.5m0-4a.5.5 0 0 1 .5-.5h9a.5.5 0 0 1 0 1h-9a.5.5 0 0 1-.5-.5m0-4a.5.5 0 0 1 .5-.5h9a.5.5 0 0 1 0 1h-9a.5.5 0 0 1-.5-.5m-3 1a1 1 0 1 0 0-2 1 1 0 0 0 0 2m0 4a1 1 0 1 0 0-2 1 1 0 0 0 0 2m0 4a1 1 0 1 0 0-2 1 1 0 0 0 0 2"
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct GridElement {
    pub uuid: Uuid,
    pub order: f64,
    pub elem_width: String,
    pub elem_height: String,
    pub elements: Vec<Box<ProjectPageWidget>>
}

impl GridElement {
    pub fn new() -> Self {
        return Self {
            uuid: Uuid::new_v4(),
            order: 0.0,
            elem_width: String::new(),
            elem_height: String::new(),
            elements: Vec::new(),
        };
    }
}

impl fmt::Debug for GridElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GridElement")
         .field("order", &self.order)
         .field("elem_width", &self.elem_width)
         .field("elem_height", &self.elem_height)
         .finish_non_exhaustive()
    }
}

impl ProjectPageWidgetTrait for GridElement {
    fn to_element(&self, mut next_widget_position: Signal<DragStatus>) -> Element {
        let self_order = self.order;
        let self_id = self.uuid;
        let mut to_be_deleted = use_signal(|| false);
        rsx! {
            div {
                class: match to_be_deleted() {
                    true => "project-page grid widget-container to-be-deleted",
                    _ => "project-page grid widget-container"
                },
                onmouseover: move |_evt| {
                    if next_widget_position() != DragStatus::PlanDeletion {
                        next_widget_position.set(DragStatus::DragHovered(self_order - 0.5));
                    } else { to_be_deleted.set(true); }
                },
                onmouseleave: move |_evt| {
                    if to_be_deleted() {to_be_deleted.set(false);}
                },
                onclick: move |_evt| {
                    if to_be_deleted() {
                        next_widget_position.set(DragStatus::DeleteElement(self_id));
                    }
                },
                {
                    self.elements.iter().map(|elem| rsx! {
                        div {
                            class: "project-page grid-element",
                            width: self.elem_width.clone(),
                            height: self.elem_height.clone(),
                            {elem.to_element(next_widget_position)}
                        }
                    })
                }
            }
        }
    }

    fn id(&self) -> Uuid { return self.uuid; }

    fn element_order(&self) -> f64 { return self.order; }
    fn set_element_order(&mut self, new_order_key: f64) { self.order = new_order_key; }

    fn to_widget_icon(&self, mut dragged_element: Signal<ProjectPageWidgetChoice>) -> Element {
        let uuid = Uuid::new_v4();
        rsx! {
            div {
                id: format!("{}", uuid),
                class: match dragged_element() {
                    ProjectPageWidgetChoice::GridElement => { "project-page icon-container selected" },
                    _ => { "project-page icon-container" }
                },
                onclick: move |_| {
                    match dragged_element() {
                        ProjectPageWidgetChoice::GridElement => dragged_element.set(ProjectPageWidgetChoice::None),
                        _ => dragged_element.set(ProjectPageWidgetChoice::GridElement)
                    }
                },
                svg {
                    width: "16",
                    height: "16",
                    fill: "currentColor",
                    class: "project-page grid-icon large-icon bi bi-grid-fill",
                    view_box: "0 0 16 16",
                    path {
                        d: "M1 2.5A1.5 1.5 0 0 1 2.5 1h3A1.5 1.5 0 0 1 7 2.5v3A1.5 1.5 0 0 1 5.5 7h-3A1.5 1.5 0 0 1 1 5.5zm8 0A1.5 1.5 0 0 1 10.5 1h3A1.5 1.5 0 0 1 15 2.5v3A1.5 1.5 0 0 1 13.5 7h-3A1.5 1.5 0 0 1 9 5.5zm-8 8A1.5 1.5 0 0 1 2.5 9h3A1.5 1.5 0 0 1 7 10.5v3A1.5 1.5 0 0 1 5.5 15h-3A1.5 1.5 0 0 1 1 13.5zm8 0A1.5 1.5 0 0 1 10.5 9h3a1.5 1.5 0 0 1 1.5 1.5v3a1.5 1.5 0 0 1-1.5 1.5h-3A1.5 1.5 0 0 1 9 13.5z"
                    }
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ImageDisplayMode {
    Grid(usize)
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ImageElement {
    pub uuid: Uuid,
    pub order: f64,
    pub src: Signal<Vec<String>>,
    pub display_mode: Signal<ImageDisplayMode>
}

impl ImageElement {
    pub fn new() -> Self {
        return Self {
            uuid: Uuid::new_v4(),
            order: 0.0,
            src: Signal::new(Vec::new()),
            display_mode: Signal::new(ImageDisplayMode::Grid(2))
        };
    }
}

impl ProjectPageWidgetTrait for ImageElement {

    fn to_element(&self, mut next_widget_position: Signal<DragStatus>) -> Element {
        let self_order = self.order;
        let self_id = self.uuid;
        let mut images = self.src.clone();
        let display_mode = self.display_mode.clone();

        let mut to_be_deleted = use_signal(|| false);
        let mut dragged_over = use_signal(|| false);

        let div_class = use_memo(move || {
            let class_append1 = match *display_mode.read() {
                ImageDisplayMode::Grid(_) => "image-grid",
            };

            let class_append2 = match (to_be_deleted(), dragged_over()) {
                (true, _) => "to-be-deleted",
                (false, true) => "dragged_over",
                _ => ""
            };

            let div_class = format!("project-page image-widget-container {} {}", class_append1, class_append2);
            div_class
        });

        //let grid_cols = match display_mode() {
//            ImageDisplayMode::Grid(n_cols) => {
//                let n_cols = if images().is_empty() { 1 } else { n_cols };
//                format!("repeat({}, 1fr)", n_cols)
//            },
//        };

        let grid_cols = use_memo(move || {
            match *display_mode.read() {
                ImageDisplayMode::Grid(n_cols) => {
                    let n_cols = if images.read().is_empty() { 1 } else { n_cols };
                    format!("repeat({}, 1fr)", n_cols)
                },
            }
        });

        println!("Contained images: {:?}", images.read());

        // use asset handler here causes crashes, since this element is conditionally created
        // it needs to be in the topscope app and yet does not work there, either
        // ergo, images need to be copied over into the exe's directoriy in order to be rendered
//        use_asset_handler("images", |request, response| {
//            let bytes = std::fs::read(request.uri().path()).unwrap_or_else(|_| Vec::new() as Vec<u8>);
//            if bytes.is_empty() {println!("Reading data at {} failed!", request.uri().path());}
//            else {println!("Asset handler successfully called. Read {} bytes.", bytes.len())}
//            response.respond(Response::new(bytes));
//        });

        rsx! {
            div {
                class: div_class(),
                display: "grid",
                grid_template_columns: grid_cols(),
                grid_auto_rows: "auto",
                gap: "0.5rem",
                justify_items: "center",
                align_items: "center",
                onmouseover: move |_evt| {
                    if next_widget_position() != DragStatus::PlanDeletion {
                        next_widget_position.set(DragStatus::DragHovered(self_order - 0.5));
                    } else { to_be_deleted.set(true); }
                },
                onmouseleave: move |_evt| {
                    if to_be_deleted() {to_be_deleted.set(false);}
                },
                onclick: move |_evt| {
                    if to_be_deleted() {
                        next_widget_position.set(DragStatus::DeleteElement(self_id));
                    }
                },
                ondragover: move |evt| {
                    evt.prevent_default();
                    evt.stop_propagation();
                    println!("OnDragOver fired on element: {}!", self_id);
                    dragged_over.set(true);
                },
                ondragleave: move |evt| {
                    evt.prevent_default();
                    evt.stop_propagation();
                    println!("OnDragLeave fired on element: {}!", self_id);
                    dragged_over.set(false);
                },
                ondrop: move |evt: Event<DragData>| {
                    evt.prevent_default();
                    evt.stop_propagation();
                    println!("Drop fired on element: {}!", self_id);
                    if let Some(file_engine) = evt.data().files() {

                        let copied_images_dir = exe_dir().clone().unwrap_or_else(PathBuf::new).join("copied_images");
                            //TODO: handle fallible case
                        if !copied_images_dir.is_dir() { let _ = std::fs::create_dir_all(&copied_images_dir); }

                        for fil in file_engine.files().iter() {
                            let basename = match fil.replace("\\", "/").split_terminator("/").last() {
                                Some(name) => name.to_string(),
                                None => fil.clone()
                            };
                            let target = copied_images_dir.join(&basename);

                            println!("Copying {} to {:?}!", &fil, &target);
                            match std::fs::copy(fil, target) {
                                Ok(_) => println!("Copy succeeded!"),
                                Err(_) => println!("Copy failed!")
                            }

                            images.write().push(basename);
                        }

                        //images.write().append(&mut file_engine.files());
                    }
                    dragged_over.set(false);
                },

                {
                    images.peek().clone().iter().map(|pth| {
                        rsx! {
                            img {
                                class: "project-page image-widget",
                                //onmouseover: move |_evt| {
//                                    if next_widget_position() != DragStatus::PlanDeletion {
//                                        next_widget_position.set(DragStatus::DragHovered(self_order - 0.5));
//                                    } else { to_be_deleted.set(true); }
//                                },
//                                onmouseleave: move |_evt| {
//                                    if to_be_deleted() {to_be_deleted.set(false);}
//                                },
//                                onclick: move |_evt| {
//                                    if to_be_deleted() {
//                                        next_widget_position.set(DragStatus::DeleteElement(self_id));
//                                    }
//                                },
                                //src: format!("file://{}", pth.replace("\\", "/"))
                                //src: "dna_background_v2 - Kopie.jpg"
                                src: format!("/copied_images/{}", &pth)
                            }
                        }
                    })
                }
                svg {
                    width: "16",
                    height: "16",
                    fill: "currentColor",
                    class: "project-page add-image-icon medium-icon bi bi-plus-square-fill",
                    view_box: "0 0 16 16",
                    onclick: move |_evt| {
                        let init_dir = exe_dir().clone().unwrap_or_else(PathBuf::new);
                        let filepicker_result = rfd::FileDialog::new()
                            .set_directory(&init_dir)
                            //.add_filter("SIF files", &["sif"])
                            .add_filter("All files", &["*"])
                            .pick_files();
                        if let Some(image_paths) = filepicker_result {
                            let copied_images_dir = exe_dir().clone().unwrap_or_else(PathBuf::new).join("copied_images");
                            //TODO: handle fallible case
                            if !copied_images_dir.is_dir() { let _ = std::fs::create_dir_all(&copied_images_dir); }

                            for fil in image_paths.iter().map(|pth| pth.to_string_lossy().to_string()) {
                                let basename = match fil.replace("\\", "/").split_terminator("/").last() {
                                    Some(name) => name.to_string(),
                                    None => fil.clone()
                                };
                                let target = copied_images_dir.join(&basename);

                                println!("Copying {} to {:?}!", &fil, &target);
                                match std::fs::copy(fil, target) {
                                    Ok(_) => println!("Copy succeeded!"),
                                    Err(_) => println!("Copy failed!")
                                }

                                images.write().push(basename);
                            }
                        }
                    },
                    path {
                        d: "M2 0a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V2a2 2 0 0 0-2-2zm6.5 4.5v3h3a.5.5 0 0 1 0 1h-3v3a.5.5 0 0 1-1 0v-3h-3a.5.5 0 0 1 0-1h3v-3a.5.5 0 0 1 1 0"
                    }
                }
            }
        }
    }

    fn id(&self) -> Uuid { return self.uuid; }

    fn element_order(&self) -> f64 { return self.order; }
    fn set_element_order(&mut self, new_order_key: f64) { self.order = new_order_key; }

    fn to_widget_icon(&self, mut dragged_element: Signal<ProjectPageWidgetChoice>) -> Element {
        let uuid = Uuid::new_v4();
        rsx! {
            div {
                id: format!("{}", uuid),
                class: match dragged_element() {
                    ProjectPageWidgetChoice::ImageElement => { "project-page icon-container selected" },
                    _ => { "project-page icon-container" }
                },
                onclick: move |_| {
                    match dragged_element() {
                        ProjectPageWidgetChoice::ImageElement => dragged_element.set(ProjectPageWidgetChoice::None),
                        _ => dragged_element.set(ProjectPageWidgetChoice::ImageElement)
                    }
                },
                svg {
                    width: "16",
                    height: "16",
                    fill: "currentColor",
                    class: "project-page image-icon large-icon bi bi-image-fill",
                    view_box: "0 0 16 16",
                    path {
                        d: "M.002 3a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2h-12a2 2 0 0 1-2-2zm1 9v1a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V9.5l-3.777-1.947a.5.5 0 0 0-.577.093l-3.71 3.71-2.66-1.772a.5.5 0 0 0-.63.062zm5-6.5a1.5 1.5 0 1 0-3 0 1.5 1.5 0 0 0 3 0"
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct MeasurementsElement {
    pub uuid: Uuid,
    pub order: f64,
    pub src: PathBuf,
    pub description: String
}


impl MeasurementsElement {
    pub fn new() -> Self { return Self { uuid: Uuid::new_v4(), order: 0.0, src: PathBuf::new(), description: "Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua.".to_string() }; }
}

impl ProjectPageWidgetTrait for MeasurementsElement {
    fn to_element(&self, mut next_widget_position: Signal<DragStatus>) -> Element {
        let self_order = self.order;
        let self_id = self.uuid;
        let mut to_be_deleted = use_signal(|| false);
        rsx! {
            div {
                class: match to_be_deleted() {
                    true => "project-page measurements-widget-container widget-container to-be-deleted",
                    _ => "project-page measurements-widget-container widget-container"
                },
                onmouseover: move |_evt| {
                    if next_widget_position() != DragStatus::PlanDeletion {
                        next_widget_position.set(DragStatus::DragHovered(self_order - 0.5));
                    } else { to_be_deleted.set(true); }
                },
                onmouseleave: move |_evt| {
                    if to_be_deleted() {to_be_deleted.set(false);}
                },
                onclick: move |_evt| {
                    if to_be_deleted() {
                        next_widget_position.set(DragStatus::DeleteElement(self_id));
                    }
                },
                svg {
                    width: "16",
                    height: "16",
                    fill: "currentColor",
                    class: "project-page measurements-widget-icon large-icon bi bi-clipboard2-data-fill",
                    view_box: "0 0 16 16",
                    path {
                        d: "M10 .5a.5.5 0 0 0-.5-.5h-3a.5.5 0 0 0-.5.5.5.5 0 0 1-.5.5.5.5 0 0 0-.5.5V2a.5.5 0 0 0 .5.5h5A.5.5 0 0 0 11 2v-.5a.5.5 0 0 0-.5-.5.5.5 0 0 1-.5-.5"
                    }
                    path {
                        d: "M4.085 1H3.5A1.5 1.5 0 0 0 2 2.5v12A1.5 1.5 0 0 0 3.5 16h9a1.5 1.5 0 0 0 1.5-1.5v-12A1.5 1.5 0 0 0 12.5 1h-.585q.084.236.085.5V2a1.5 1.5 0 0 1-1.5 1.5h-5A1.5 1.5 0 0 1 4 2v-.5q.001-.264.085-.5M10 7a1 1 0 1 1 2 0v5a1 1 0 1 1-2 0zm-6 4a1 1 0 1 1 2 0v1a1 1 0 1 1-2 0zm4-3a1 1 0 0 1 1 1v3a1 1 0 1 1-2 0V9a1 1 0 0 1 1-1"
                    }
                }
                p {
                    class: "project-page measurements-widget-text",
                    contenteditable: true,
                    {self.description.clone()}
                }
            }
        }
    }

    fn id(&self) -> Uuid { return self.uuid; }

    fn element_order(&self) -> f64 { return self.order; }
    fn set_element_order(&mut self, new_order_key: f64) { self.order = new_order_key; }

    fn to_widget_icon(&self, mut dragged_element: Signal<ProjectPageWidgetChoice>) -> Element {
        let uuid = Uuid::new_v4();
        rsx! {
            div {
                id: format!("{}", uuid),
                class: match dragged_element() {
                    ProjectPageWidgetChoice::MeasurementsElement => { "project-page icon-container selected" },
                    _ => { "project-page icon-container" }
                },
                onclick: move |_| {
                    match dragged_element() {
                        ProjectPageWidgetChoice::MeasurementsElement => dragged_element.set(ProjectPageWidgetChoice::None),
                        _ => dragged_element.set(ProjectPageWidgetChoice::MeasurementsElement)
                    }
                },
                svg {
                    width: "16",
                    height: "16",
                    fill: "currentColor",
                    class: "project-page measurements-widget-icon large-icon bi bi-clipboard2-data-fill",
                    view_box: "0 0 16 16",
                    path {
                        d: "M10 .5a.5.5 0 0 0-.5-.5h-3a.5.5 0 0 0-.5.5.5.5 0 0 1-.5.5.5.5 0 0 0-.5.5V2a.5.5 0 0 0 .5.5h5A.5.5 0 0 0 11 2v-.5a.5.5 0 0 0-.5-.5.5.5 0 0 1-.5-.5"
                    }
                    path {
                        d: "M4.085 1H3.5A1.5 1.5 0 0 0 2 2.5v12A1.5 1.5 0 0 0 3.5 16h9a1.5 1.5 0 0 0 1.5-1.5v-12A1.5 1.5 0 0 0 12.5 1h-.585q.084.236.085.5V2a1.5 1.5 0 0 1-1.5 1.5h-5A1.5 1.5 0 0 1 4 2v-.5q.001-.264.085-.5M10 7a1 1 0 1 1 2 0v5a1 1 0 1 1-2 0zm-6 4a1 1 0 1 1 2 0v1a1 1 0 1 1-2 0zm4-3a1 1 0 0 1 1 1v3a1 1 0 1 1-2 0V9a1 1 0 0 1 1-1"
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct AnalysisElement {
    pub uuid: Uuid,
    pub order: f64,
    pub container: Option<Uuid>,
    pub data: Vec<MeasurementsElement>,
    pub output: Vec<ResultsElement>,
    pub job: Option<JobId>,
    pub description: String
}

impl AnalysisElement {
    pub fn new() -> Self {
        return Self {
            uuid: Uuid::new_v4(),
            order: 0.0,
            container: None,
            data: Vec::new(),
            output: Vec::new(),
            job: None,
            description: "Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua.".to_string(),
        };
    }
}

impl Default for AnalysisElement {
    fn default() -> Self { return Self::new(); }
}

impl ProjectPageWidgetTrait for AnalysisElement {
    fn to_element(&self, mut next_widget_position: Signal<DragStatus>) -> Element {
        let self_order = self.order;
        let self_id = self.uuid;
        let mut to_be_deleted = use_signal(|| false);
        rsx! {
            div {
                class: match to_be_deleted() {
                    true => "project-page analysis-widget-container widget-container to-be-deleted",
                    _ => "project-page analysis-widget-container widget-container"
                },
                onmouseover: move |_evt| {
                    if next_widget_position() != DragStatus::PlanDeletion {
                        next_widget_position.set(DragStatus::DragHovered(self_order - 0.5));
                    } else { to_be_deleted.set(true); }
                },
                onmouseleave: move |_evt| {
                    if to_be_deleted() {to_be_deleted.set(false);}
                },
                onclick: move |_evt| {
                    if to_be_deleted() {
                        next_widget_position.set(DragStatus::DeleteElement(self_id));
                    }
                },
                svg {
                    width: "16",
                    height: "16",
                    fill: "currentColor",
                    class: "project-page analysis-widget-icon large-icon bi bi-chevron-double-right",
                    view_box: "0 0 16 16",
                    path {
                        fill_rule: "evenodd",
                        d: "M3.646 1.646a.5.5 0 0 1 .708 0l6 6a.5.5 0 0 1 0 .708l-6 6a.5.5 0 0 1-.708-.708L9.293 8 3.646 2.354a.5.5 0 0 1 0-.708"
                    }
                    path {
                        fill_rule: "evenodd",
                        d: "M7.646 1.646a.5.5 0 0 1 .708 0l6 6a.5.5 0 0 1 0 .708l-6 6a.5.5 0 0 1-.708-.708L13.293 8 7.646 2.354a.5.5 0 0 1 0-.708"
                    }
                }
                p {
                    class: "project-page analysis-widget-text",
                    contenteditable: true,
                    {self.description.clone()}
                }
            }
        }
    }

    fn id(&self) -> Uuid { return self.uuid; }

    fn element_order(&self) -> f64 { return self.order; }
    fn set_element_order(&mut self, new_order_key: f64) { self.order = new_order_key; }

    fn to_widget_icon(&self, mut dragged_element: Signal<ProjectPageWidgetChoice>) -> Element {
        let uuid = Uuid::new_v4();
        rsx! {
            div {
                id: format!("{}", uuid),
                class: match dragged_element() {
                    ProjectPageWidgetChoice::AnalysisElement => { "project-page icon-container selected" },
                    _ => { "project-page icon-container" }
                },
                onclick: move |_| {
                    match dragged_element() {
                        ProjectPageWidgetChoice::AnalysisElement => dragged_element.set(ProjectPageWidgetChoice::None),
                        _ => dragged_element.set(ProjectPageWidgetChoice::AnalysisElement)
                    }
                },
                svg {
                    width: "16",
                    height: "16",
                    fill: "currentColor",
                    class: "project-page analysis-widget-icon large-icon bi bi-chevron-double-right",
                    view_box: "0 0 16 16",
                    path {
                        fill_rule: "evenodd",
                        d: "M3.646 1.646a.5.5 0 0 1 .708 0l6 6a.5.5 0 0 1 0 .708l-6 6a.5.5 0 0 1-.708-.708L9.293 8 3.646 2.354a.5.5 0 0 1 0-.708"
                    }
                    path {
                        fill_rule: "evenodd",
                        d: "M7.646 1.646a.5.5 0 0 1 .708 0l6 6a.5.5 0 0 1 0 .708l-6 6a.5.5 0 0 1-.708-.708L13.293 8 7.646 2.354a.5.5 0 0 1 0-.708"
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
pub struct ResultsElement {
    pub uuid: Uuid,
    pub order: f64,
    pub src: PathBuf,
    pub description: String
}

impl ResultsElement {
    pub fn new() -> Self { return Self { uuid: Uuid::new_v4(), order: 0.0, src: PathBuf::new(), description: "Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua.".to_string()}; }
}

impl ProjectPageWidgetTrait for ResultsElement {
    fn to_element(&self, mut next_widget_position: Signal<DragStatus>) -> Element {
        let self_order = self.order;
        let self_id = self.uuid;
        let no_content = self.description.is_empty();
        let mut to_be_deleted = use_signal(|| false);
        rsx! {
            div {
                class: match to_be_deleted() {
                    true => "project-page results-widget-container widget-container to-be-deleted",
                    _ => "project-page results-widget-container widget-container"
                },
                onmouseover: move |_evt| {
                    if next_widget_position() != DragStatus::PlanDeletion {
                        next_widget_position.set(DragStatus::DragHovered(self_order - 0.5));
                    } else { to_be_deleted.set(true); }
                },
                onmouseleave: move |_evt| {
                    if to_be_deleted() {to_be_deleted.set(false);}
                },
                onclick: move |_evt| {
                    if to_be_deleted() {
                        next_widget_position.set(DragStatus::DeleteElement(self_id));
                    }
                },
                svg {
                    width: "16",
                    height: "16",
                    fill: "currentColor",
                    class: "project-page results-widget-icon large-icon bi bi-clipboard2-check-fill",
                    view_box: "0 0 16 16",
                    path {
                        d: "M10 .5a.5.5 0 0 0-.5-.5h-3a.5.5 0 0 0-.5.5.5.5 0 0 1-.5.5.5.5 0 0 0-.5.5V2a.5.5 0 0 0 .5.5h5A.5.5 0 0 0 11 2v-.5a.5.5 0 0 0-.5-.5.5.5 0 0 1-.5-.5"
                    }
                    path {
                        d: "M4.085 1H3.5A1.5 1.5 0 0 0 2 2.5v12A1.5 1.5 0 0 0 3.5 16h9a1.5 1.5 0 0 0 1.5-1.5v-12A1.5 1.5 0 0 0 12.5 1h-.585q.084.236.085.5V2a1.5 1.5 0 0 1-1.5 1.5h-5A1.5 1.5 0 0 1 4 2v-.5q.001-.264.085-.5m6.769 6.854-3 3a.5.5 0 0 1-.708 0l-1.5-1.5a.5.5 0 1 1 .708-.708L7.5 9.793l2.646-2.647a.5.5 0 0 1 .708.708"
                    }
                }
                p {
                    class: match no_content {
                        false => "project-page results-widget-text",
                        true => "project-page results-widget-text no-content"
                    },
                    contenteditable: true,
                    {self.description.clone()}
                }
            }
        }
    }

    fn id(&self) -> Uuid { return self.uuid; }

    fn element_order(&self) -> f64 { return self.order; }
    fn set_element_order(&mut self, new_order_key: f64) { self.order = new_order_key; }

    fn to_widget_icon(&self, mut dragged_element: Signal<ProjectPageWidgetChoice>) -> Element {
        let uuid = Uuid::new_v4();
        rsx! {
            div {
                id: format!("{}", uuid),
                class: match dragged_element() {
                    ProjectPageWidgetChoice::ResultsElement => { "project-page icon-container selected" },
                    _ => { "project-page icon-container" }
                },
                onclick: move |_| {
                    match dragged_element() {
                        ProjectPageWidgetChoice::ResultsElement => dragged_element.set(ProjectPageWidgetChoice::None),
                        _ => dragged_element.set(ProjectPageWidgetChoice::ResultsElement)
                    }
                },
                svg {
                    width: "16",
                    height: "16",
                    fill: "currentColor",
                    class: "project-page results-widget-icon large-icon bi bi-clipboard2-check-fill",
                    view_box: "0 0 16 16",
                    path {
                        d: "M10 .5a.5.5 0 0 0-.5-.5h-3a.5.5 0 0 0-.5.5.5.5 0 0 1-.5.5.5.5 0 0 0-.5.5V2a.5.5 0 0 0 .5.5h5A.5.5 0 0 0 11 2v-.5a.5.5 0 0 0-.5-.5.5.5 0 0 1-.5-.5"
                    }
                    path {
                        d: "M4.085 1H3.5A1.5 1.5 0 0 0 2 2.5v12A1.5 1.5 0 0 0 3.5 16h9a1.5 1.5 0 0 0 1.5-1.5v-12A1.5 1.5 0 0 0 12.5 1h-.585q.084.236.085.5V2a1.5 1.5 0 0 1-1.5 1.5h-5A1.5 1.5 0 0 1 4 2v-.5q.001-.264.085-.5m6.769 6.854-3 3a.5.5 0 0 1-.708 0l-1.5-1.5a.5.5 0 1 1 .708-.708L7.5 9.793l2.646-2.647a.5.5 0 0 1 .708.708"
                    }
                }
            }
        }
    }
}



//################################################################################
//## widgets that may be part of a document temporarily
//## they need to be filtered out before export
//################################################################################


#[derive(Clone, PartialEq, Default, Debug)]
pub struct PlaceHolderElement {
    pub uuid: Uuid,
    pub order: f64,
}

impl PlaceHolderElement {
    pub fn new() -> Self { return Self { uuid: Uuid::new_v4(), order: 100000.0 } }
}

impl ProjectPageWidgetTrait for PlaceHolderElement {
    fn to_element(&self, mut next_widget_position: Signal<DragStatus>) -> Element {
        let self_order = self.order;
        rsx! {
            div {
                class: "project-page placeholder widget-container",
                svg {
                    onclick: move |_| {
                        let next_pos = match next_widget_position() {
                            DragStatus::DragHovered(pos) => pos,
                            _ => self_order
                        };
                        next_widget_position.set(DragStatus::DragEnded(next_pos));
                    },
                    width: "16",
                    height: "16",
                    fill: "currentColor",
                    class: "project-page placeholder-icon small-icon bi bi-plus-circle-fill",
                    view_box: "0 0 16 16",
                    path {
                        d: "M16 8A8 8 0 1 1 0 8a8 8 0 0 1 16 0M8.5 4.5a.5.5 0 0 0-1 0v3h-3a.5.5 0 0 0 0 1h3v3a.5.5 0 0 0 1 0v-3h3a.5.5 0 0 0 0-1h-3z"
                    }
                }
            }
        }
    }

    fn id(&self) -> Uuid { return self.uuid; }

    fn element_order(&self) -> f64 { return self.order; }
    fn set_element_order(&mut self, new_order_key: f64) { self.order = new_order_key; }

    //Todo: return appropriate Error instead
    fn to_widget_icon(&self, _dragged_element: Signal<ProjectPageWidgetChoice>) -> Element {
        //panic!("Placeholder should never be available in the widget store!")
        //empty output
        rsx! {}
    }
}




