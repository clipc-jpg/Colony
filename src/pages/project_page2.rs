



use std::fmt;
use std::path::PathBuf;
use std::cmp::Ordering;
use std::collections::HashMap;

use dioxus::prelude::*;
use dioxus_html::HasFileData;
use global_attributes::resize;
use itertools::Itertools;
use uuid::Uuid;

use crate::pages::*;
use crate::components::*;
use crate::backend::jobs::*;
use crate::persistent_state::exe_dir;
use crate::FrontendCommChannel;
use crate::SerializableParagraphElement;
use crate::SerializableProjectPageWidget;
use regex::Regex;

use crate::document::eval;

#[derive(Clone, Debug, PartialEq, IntoIterator)]
struct ProjectPageDataStore {
    #[into_iterator(owned, ref,  ref_mut)]
    pub items: HashMap<Uuid, SerializableProjectPageWidget>
}

impl ProjectPageDataStore {
    pub fn new() -> Self {
        ProjectPageDataStore { items: HashMap::new() }
    }

    pub fn get(&self id: Uuid) -> Option<&SerializableProjectPageWidget> {
        return self.items.get(id);
    }

    pub fn get_mut(&self id: Uuid) -> Option<&mut SerializableProjectPageWidget> {
        return self.items.get_mut(&id);
    }

    pub fn update(&mut self, update_variant: &ProjectPageWidgetUpdate) {

        let other_element = match update_variant {
            //AnalysisElement
            ProjectPageWidgetUpdate::LinkMeasurement(_id, elem_id) => {
                self.get(elem_id).filter(|elem| {
                    match elem {
                        SerializableProjectPageWidget::MeasurementsElement(_) => true,
                        _ => false
                    }
                }).cloned()
            },
            ProjectPageWidgetUpdate::LinkResult(_id, elem_id) => {
                self.get(elem_id).filter(|elem| {
                    match elem {
                        SerializableProjectPageWidget::ResultsElement(_) => true,
                        _ => false
                    }
                }).cloned()
            },
            ProjectPageWidgetUpdate::UnlinkElement(_id, elem_id) => { None }, //no retrieval necessary
            _ => { None } //Error
        };

        if let Some(mut item) = self.get_mut(update_variant.target_id()) {
            match item {
                SerializableProjectPageWidget::ParagraphElement(ref mut elem) => {
                    match update_variant {
                        ProjectPageWidgetUpdate::SetOrder(_id, order) => { elem.order = order; },
                        ProjectPageWidgetUpdate::SetText(_id, text) => { elem.text = text },
                        _ => {} // Error
                    }
                },
                SerializableProjectPageWidget::OrderedListElement(ref mut elem) => {
                    match update_variant {
                        ProjectPageWidgetUpdate::SetOrder(_id, order) => { elem.order = order; },
                        _ => {} //Error
                    }
                },
                SerializableProjectPageWidget::UnorderedListElement(ref mut elem) => {
                    match update_variant {
                        ProjectPageWidgetUpdate::SetOrder(_id, order) => { elem.order = order; },
                        _ => {} //Error
                    }
                },
                SerializableProjectPageWidget::GridElement(ref mut elem) => {
                    match update_variant {
                        ProjectPageWidgetUpdate::SetOrder(_id, order) => { elem.order = order; },
                        _ => {} //Error
                    }
                },
                SerializableProjectPageWidget::ImageElement(ref mut elem) => {
                    match update_variant {
                        ProjectPageWidgetUpdate::SetOrder(_id, order) => { elem.order = order; },
                        //ImageElement
                        ProjectPageWidgetUpdate::AddImageSrc(_id, src) => { elem.src.push(src); }
                        ProjectPageWidgetUpdate::RemoveImageSrc(_id, src) => { elem.src.retain(|elem| *elem != src ) }
                        ProjectPageWidgetUpdate::SetDisplayMode(_id, mode) => { elem.display_mode = format!("{mode}") },
                        _ => {} //Error
                    }
                },
                SerializableProjectPageWidget::MeasurementsElement(ref mut elem) => {
                    match update_variant {
                        ProjectPageWidgetUpdate::SetOrder(_id, order) => { elem.order = order; },
                        ProjectPageWidgetUpdate::SetText(_id, text) => { elem.description = text; },
                        //Measurements- and Resultselement
                        ProjectPageWidgetUpdate::SetDataSource(_id, src) => { elem.src = src; }
                        _ => {} //Error
                    }
                },
                SerializableProjectPageWidget::AnalysisElement(ref mut elem) => {
                    match update_variant {
                        ProjectPageWidgetUpdate::SetOrder(_id, order) => { elem.order = order; },
                        ProjectPageWidgetUpdate::SetText(_id, text) => { elem.description = text; },
                        //AnalysisElement
                        ProjectPageWidgetUpdate::LinkMeasurement(_id, elem_id) => {
                            if let Some(other) = other_element {
                                elem.output.push(other); elem.output.sort(); elem.output.dedup();
                            }
                        },
                        ProjectPageWidgetUpdate::LinkResult(_id, elem_id) => {
                            if let Some(other) = other_element {
                                elem.output.push(other); elem.output.sort(); elem.output.dedup();
                            }
                        },
                        ProjectPageWidgetUpdate::UnlinkElement(_id, elem_id) => {
                            elem.data.retain(|elem| elem.uuid != elem_id );
                            elem.output.retain(|elem| elem.uuid != elem_id );
                        }
                        ProjectPageWidgetUpdate::LinkContainer(_id, container_id) => {
                            if elem.container.is_none() { elem.container = Some(container_id) }
                            // TODO: Else?
                        },
                        ProjectPageWidgetUpdate::UnlinkContainer(_id, container_id) => {
                            if elem.container.is_some_and(|elem_id| elem_id ==  container_id) { elem.container = None }
                            // TODO: Else?
                        },
                        ProjectPageWidgetUpdate::LinkJob(_id, ,jid) => {
                            if elem.job.is_none() { elem.job = Some(jid) }
                            // TODO: Else?
                        },
                        ProjectPageWidgetUpdate::UnlinkJob(_id, jid)  => {
                            if elem.job.is_some_and(|elem_id| elem_id ==  jid) { elem.job = None }
                            // TODO: Else?
                        },
                        _ => {} //Error
                    }
                },
                SerializableProjectPageWidget::ResultsElement(ref mut elem) => {
                    match update_variant {
                        ProjectPageWidgetUpdate::SetOrder(_id, order) => { elem.order = order; },
                        ProjectPageWidgetUpdate::SetText(_id, text) => { elem.description = text; },
                        //Measurements- and Resultselement
                        ProjectPageWidgetUpdate::SetDataSource(_id, src) => { elem.src = src; }
                        _ => {} //Error
                    }
                },
            }
        }
    }
}

#[component]
pub fn ProjectPage(app_state: Signal<AppState>, comm_with_backend: Signal<FrontendCommChannel>)  -> Element {

    let mut data_store = use_signal(|| {
        ProjectPageDataStore::new()
    });

    let widgets = use_memo(move || {
        let result = data_store.read().items.values()
                .sorted_by_key(|elem| elem.element_order())
                .map(|elem| elem.id())
                .collect_vec();

        // enforce ordering that simplifies future reordering logic
        for (k, id) in result.iter().enumerate() {
            data_store.write_silent().update(&ProjectPageWidgetUpdate::SetOrder(id, 2+2*k));
        }
    });

    let _init_data = use_future(move || async move {
        let new_elem = SerializableProjectPageWidget::ParagraphElement(
            SerializableParagraphElement {
                uuid: Uuid::new_v4(),
                order: 2.0,
                text: "Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua. Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua. Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua. ".to_string()
            }
        );
        data_store.write().items.insert(new_elem.uuid, new_elem);
        let new_elem = SerializableProjectPageWidget::ParagraphElement(
            SerializableParagraphElement {
                uuid: Uuid::new_v4(),
                order: 4.0,
                text: "Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua. Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua. Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua. ".to_string()
            }
        );
        data_store.write().items.insert(new_elem.uuid, new_elem);
    });

    let data_updates = use_signal(|| Vec::new() as Vec<ProjectPageWidgetUpdate>)

    let _update_data = use_effect(move || {
        if !data_updates.read().is_empty() {

            for update in data_updates.read().iter() {
                data_store.write().update(update);
            }

            data_updates.write().clear();
        }
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
fn WidgetStore(opened: Signal<bool>, page_state: Signal<ProjectPageUserInteraction>) -> Element {
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


//################################################################################
//## creating projectpage widgets
//################################################################################


#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ProjectPageWidgetChoice {
    None,
    ParagraphElement,
    //OrderedListElement,
    //UnorderedListElement,
    //GridElement,
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
            //Self::OrderedListElement => Some(ProjectPageWidget::OrderedListElement(OrderedListElement::new())),
            //Self::UnorderedListElement => Some(ProjectPageWidget::UnorderedListElement(UnorderedListElement::new())),
            //Self::GridElement => Some(ProjectPageWidget::GridElement(GridElement::new())),
            Self::ImageElement => Some(ProjectPageWidget::ImageElement(ImageElement::new())),
            Self::MeasurementsElement => Some(ProjectPageWidget::MeasurementsElement(MeasurementsElement::new())),
            Self::AnalysisElement => Some(ProjectPageWidget::AnalysisElement(AnalysisElement::new())),
            Self::ResultsElement => Some(ProjectPageWidget::ResultsElement(ResultsElement::new())),
        }
    }
}

//################################################################################
//## ProjectPageWidgets as enum
//################################################################################


pub trait ProjectPageWidgetTrait {
    fn to_element(&self, next_widget_position: Signal<DragStatus>) -> Element;
    fn id(&self) -> Uuid;
    fn element_order(&self) -> i64;
    fn set_element_order(&mut self, new_order_key: f64);
    fn to_widget_icon(&self, dragged_element: Signal<ProjectPageWidgetChoice>) -> Element;
}


#[derive(Debug, Clone, PartialEq)]
pub enum ProjectPageWidget {
    None,
    ParagraphElement(ParagraphElement),
    //OrderedListElement(OrderedListElement),
//    UnorderedListElement(UnorderedListElement),
//    GridElement(GridElement),
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
            //Self::OrderedListElement(elem) => { elem.to_element(next_widget_position) },
            //Self::UnorderedListElement(elem) => { elem.to_element(next_widget_position) },
            //Self::GridElement(elem) => { elem.to_element(next_widget_position) },
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
            //Self::OrderedListElement(elem) => { elem.id() },
//            Self::UnorderedListElement(elem) => { elem.id() },
//            Self::GridElement(elem) => { elem.id() },
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
            //Self::OrderedListElement(elem) => { elem.element_order() },
//            Self::UnorderedListElement(elem) => { elem.element_order() },
//            Self::GridElement(elem) => { elem.element_order() },
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
            //Self::OrderedListElement(elem) => { elem.set_element_order(new_order_key) },
//            Self::UnorderedListElement(elem) => { elem.set_element_order(new_order_key) },
//            Self::GridElement(elem) => { elem.set_element_order(new_order_key) },
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
            //Self::OrderedListElement(elem) => { elem.to_widget_icon(dragged_element) },
//            Self::UnorderedListElement(elem) => { elem.to_widget_icon(dragged_element) },
//            Self::GridElement(elem) => { elem.to_widget_icon(dragged_element) },
            Self::ImageElement(elem) => { elem.to_widget_icon(dragged_element) },
            Self::MeasurementsElement(elem) => { elem.to_widget_icon(dragged_element) },
            Self::AnalysisElement(elem) => { elem.to_widget_icon(dragged_element) },
            Self::ResultsElement(elem) => { elem.to_widget_icon(dragged_element) },
            Self::PlaceHolderElement(elem) => { elem.to_widget_icon(dragged_element) },
        }
    }
}



//################################################################################
//## Individual Widgets
//################################################################################

#[derive(Clone, PartialEq, Debug)]
pub struct ParagraphElement {
    pub uuid: Uuid,
    pub order: f64,
    pub text: String
}

impl ParagraphElement {
    pub fn new() -> Self {
        return Self {
            uuid: Uuid::new_v4(),
            order : 0.0,
            text: "Lorem ipsum dolor sit amet, consectetur adipisici elit, sed eiusmod tempor incidunt ut labore et dolore magna aliqua.".to_string()
        };
    }
}


impl ProjectPageWidgetTrait for ParagraphElement {
    fn to_element(&self, widget_id: Uuid, widget_data: ProjectPageWidget,
                         widget_updates: Signal<Vec<ProjectPageWidgetUpdate>>) -> Element {

        //let update_data = use_context::<UpdateData>();
        let update_data = use_signal(|| false);
        use_effect(move || {
            // just setting a signal to anything will trigger the update
            update_data.read();
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
                    widget_updates.write(ProjectPageWidgetUpdate::SetText(widget_id, par_content[1].to_string()))
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


//################################################################################
//## PageState and Updates
//################################################################################

[derive(Clone, Copy, Debug, PartialEq)]
enum ProjectPageUserInteraction {

}




[derive(Clone, Copy, Debug, PartialEq)]
enum ProjectPageWidgetUpdate {
    // First component is the id of the element to be modified, the reast are arguments
    //Broadly applicable
    SetOrder(Uuid, usize),
    SetText(Uuid, String),
    //ImageElement
    AddImageSrc(Uuid, String),
    RemoveImageSrc(Uuid, String),
    SetDisplayMode(Uuid, ImageDisplayMode),
    //Measurements- and Resultselement
    SetDataSource(Uuid, PathBuf),
    //AnalysisElement
    LinkMeasurement(Uuid, Uuid), // Self, Other
    LinkResult(Uuid, Uuid), // Self, Other
    UnlinkElement(Uuid, Uuid), // Self, Other
    LinkContainer(Uuid, Uuid), // Self, ContainerID
    UnlinkContainer(Uuid, Uuid), // Self, ContainerID
    LinkJob(Uuid, JobId),
    UnlinkJob(Uuid, JobId) // JobId may be used to check for correctness later
}

impl ProjectPageWidgetUpdate {
    pub fn target_id(&self) -> Uuid {
        return match self {
            ProjectPageWidgetUpdate::SetOrder(id, _) => { id },
            ProjectPageWidgetUpdate::SetText(id, _) => { id },
            //ImageElement
            ProjectPageWidgetUpdate::AddImageSrc(id, _) => { id },
            ProjectPageWidgetUpdate::RemoveImageSrc(id, _) => { id },
            ProjectPageWidgetUpdate::SetDisplayMode(id, _) => { id },
            //Measurements- and Resultselement
            ProjectPageWidgetUpdate::SetDataSource(id, _) => { id },
            //AnalysisElement
            ProjectPageWidgetUpdate::LinkMeasurement(id, _) => { id },
            ProjectPageWidgetUpdate::LinkResult(id, _) => { id },
            ProjectPageWidgetUpdate::UnlinkElement(id, _) => { id },
            ProjectPageWidgetUpdate::LinkContainer(id, _) => { id },
            ProjectPageWidgetUpdate::UnlinkContainer(id, _) => { id },
            ProjectPageWidgetUpdate::LinkJob(id, _) => { id },
            ProjectPageWidgetUpdate::UnlinkJob(id, _) => { id },
        };
    }
}

































