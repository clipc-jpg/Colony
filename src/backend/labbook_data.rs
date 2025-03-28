


use std::fmt;
use std::path::PathBuf;

use dioxus::prelude::*;
use itertools::Itertools;
use serde::{Serialize, Deserialize};
use serde_json;
use uuid::Uuid;
use regex::Regex;

use crate::pages;
use crate::pages::ProjectPageWidgetTrait;
use crate::JobId;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SerializableProjectPageWidget{
    ParagraphElement(SerializableParagraphElement),
    OrderedListElement(SerializableOrderedListElement),
    UnorderedListElement(SerializableUnorderedListElement),
    GridElement(SerializableGridElement),
    ImageElement(SerializableImageElement),
    MeasurementsElement(SerializableMeasurementsElement),
    AnalysisElement(SerializableAnalysisElement),
    ResultsElement(SerializableResultsElement),
}

impl SerializableProjectPageWidget {
    pub fn from(other: &pages::ProjectPageWidget) -> Option<Self>{
        return match other {
            pages::ProjectPageWidget::None => { Option::None },
            pages::ProjectPageWidget::ParagraphElement(elem) => {
                Some(Self::ParagraphElement(SerializableParagraphElement::from(elem)))
            },
            pages::ProjectPageWidget::OrderedListElement(elem) => {
                Some(Self::OrderedListElement(SerializableOrderedListElement::from(elem)))
            },
            pages::ProjectPageWidget::UnorderedListElement(elem) => {
                Some(Self::UnorderedListElement(SerializableUnorderedListElement::from(elem)))
            },
            pages::ProjectPageWidget::GridElement(elem) => {
                Some(Self::GridElement(SerializableGridElement::from(elem)))
            },
            pages::ProjectPageWidget::ImageElement(elem) => {
                Some(Self::ImageElement(SerializableImageElement::from(elem)))
            },
            pages::ProjectPageWidget::MeasurementsElement(elem) => {
                Some(Self::MeasurementsElement(SerializableMeasurementsElement::from(elem)))
            },
            pages::ProjectPageWidget::AnalysisElement(elem) => {
                Some(Self::AnalysisElement(SerializableAnalysisElement::from(elem)))
            },
            pages::ProjectPageWidget::ResultsElement(elem) => {
                Some(Self::ResultsElement(SerializableResultsElement::from(elem)))
            },
            pages::ProjectPageWidget::PlaceHolderElement(_elem) => { Option::None },
        };
    }

    pub fn to_widget(&self) -> pages::ProjectPageWidget {
        return match self {
            Self::ParagraphElement(elem) => {
                pages::ProjectPageWidget::ParagraphElement(elem.to_widget())
            },
            Self::OrderedListElement(elem) => {
                pages::ProjectPageWidget::OrderedListElement(elem.to_widget())
            },
            Self::UnorderedListElement(elem) => {
                pages::ProjectPageWidget::UnorderedListElement(elem.to_widget())
            },
            Self::GridElement(elem) => {
                pages::ProjectPageWidget::GridElement(elem.to_widget())
            },
            Self::ImageElement(elem) => {
                pages::ProjectPageWidget::ImageElement(elem.to_widget())
            },
            Self::MeasurementsElement(elem) => {
                pages::ProjectPageWidget::MeasurementsElement(elem.to_widget())
            },
            Self::AnalysisElement(elem) => {
                pages::ProjectPageWidget::AnalysisElement(elem.to_widget())
            },
            Self::ResultsElement(elem) => {
                pages::ProjectPageWidget::ResultsElement(elem.to_widget())
            },
        };
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SerializableParagraphElement {
    pub uuid: Uuid,
    pub order: f64,
    pub text: String
}

impl SerializableParagraphElement {
    pub fn from(elem: &pages::ParagraphElement) -> Self {
        Self {
            uuid: elem.id(),
            order: elem.element_order(),
            text: (elem.text)().clone()
        }
    }
    pub fn to_widget(&self) -> pages::ParagraphElement {
        return pages::ParagraphElement {
            uuid: self.uuid,
            order: self.order,
            text: Signal::new(self.text.clone())
        };
    }
}


#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SerializableOrderedListElement {
    pub uuid: Uuid,
    pub order: f64,
    pub entries: Vec<String>
}

impl SerializableOrderedListElement {
    pub fn from(elem: &pages::OrderedListElement) -> Self {
        Self {
            uuid: elem.id(),
            order: elem.element_order(),
            entries: elem.entries.iter().map(|elem| elem.clone()).collect_vec(),
        }
    }
    pub fn to_widget(&self) -> pages::OrderedListElement {
        return pages::OrderedListElement {
            uuid: self.uuid,
            order: self.order,
            entries: self.entries.iter().map(|elem| elem.clone()).collect_vec(),
        };
    }
}

#[derive(Clone, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct SerializableUnorderedListElement {
    pub uuid: Uuid,
    pub order: f64,
    pub entries: Vec<String>
}

impl SerializableUnorderedListElement {
    pub fn from(elem: &pages::UnorderedListElement) -> Self {
        Self {
            uuid: elem.id(),
            order: elem.element_order(),
            entries: elem.entries.iter().map(|elem| elem.clone()).collect_vec(),
        }
    }
    pub fn to_widget(&self) -> pages::UnorderedListElement {
        return pages::UnorderedListElement {
            uuid: self.uuid,
            order: self.order,
            entries: self.entries.iter().map(|elem| elem.clone()).collect_vec(),
        };
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SerializableGridElement {
    pub uuid: Uuid,
    pub order: f64,
    pub elem_width: String,
    pub elem_height: String,
    pub elements: Vec<Box<SerializableProjectPageWidget>>
}

impl std::fmt::Debug for SerializableGridElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GridElement")
         .field("order", &self.order)
         .field("elem_width", &self.elem_width)
         .field("elem_height", &self.elem_height)
         .finish_non_exhaustive()
    }
}

impl SerializableGridElement {
    pub fn from(elem: &pages::GridElement) -> Self {
        Self {
            uuid: elem.id(),
            order: elem.element_order(),
            elem_width: elem.elem_width.clone(),
            elem_height: elem.elem_height.clone(),
            elements: elem.elements.iter()
                        .filter_map(|elem| SerializableProjectPageWidget::from(&**elem) )
                        .map(|elem| Box::new(elem) )
                        .collect_vec(),
        }
    }
    pub fn to_widget(&self) -> pages::GridElement {
        return pages::GridElement {
            uuid: self.uuid,
            order: self.order,
            elem_width: self.elem_width.clone(),
            elem_height: self.elem_height.clone(),
            elements: self.elements.iter().map(|elem| Box::new(elem.to_widget()) ).collect_vec(),
        };
    }
}




#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SerializableImageElement {
    pub uuid: Uuid,
    pub order: f64,
    pub src: Vec<String>,
    pub display_mode: String
}

impl SerializableImageElement {
    pub fn from(elem: &pages::ImageElement) -> Self {
        Self {
            uuid: elem.id(),
            order: elem.element_order(),
            src: elem.src.iter().map(|elem| elem.clone()).collect_vec(),
            display_mode: match (elem.display_mode)() {
                pages::ImageDisplayMode::Grid(k) => format!("Grid({})", k)
            }
        }
    }
    pub fn to_widget(&self) -> pages::ImageElement {

        let grid_pattern = Regex::new(r#"Grid\((\d+)\)"#).unwrap();

        return pages::ImageElement {
            uuid: self.uuid,
            order: self.order,
            src: Signal::new(self.src.iter().map(|elem| elem.clone()).collect_vec()),
            display_mode: if let Some(mtch) = grid_pattern.find(&self.display_mode) {
                    let k = mtch.as_str().parse::<usize>().unwrap_or(2);
                    Signal::new(pages::ImageDisplayMode::Grid(k))
                } else {
                    Signal::new(pages::ImageDisplayMode::Grid(2))
            }
        };
    }
}


#[derive(Clone, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct SerializableMeasurementsElement {
    pub uuid: Uuid,
    pub order: f64,
    pub src: PathBuf,
    pub description: String
}

impl SerializableMeasurementsElement {
    pub fn from(elem: &pages::MeasurementsElement) -> Self {
        Self {
            uuid: elem.id(),
            order: elem.element_order(),
            src: elem.src.clone(),
            description: elem.description.clone(),
        }
    }
    pub fn to_widget(&self) -> pages::MeasurementsElement {
        return pages::MeasurementsElement {
            uuid: self.uuid,
            order: self.order,
            src: self.src.clone(),
            description: self.description.clone(),
        };
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SerializableAnalysisElement {
    pub uuid: Uuid,
    pub order: f64,
    pub container: Option<Uuid>,
    pub data: Vec<SerializableMeasurementsElement>,
    pub output: Vec<SerializableResultsElement>,
    pub job: Option<JobId>,
    pub description: String
}

impl SerializableAnalysisElement {
    pub fn from(elem: &pages::AnalysisElement) -> Self {
        Self {
            uuid: elem.id(),
            order: elem.element_order(),
            container: elem.container,
            data: elem.data.iter().map(|elem| SerializableMeasurementsElement::from(elem)).collect_vec(),
            output: elem.output.iter().map(|elem| SerializableResultsElement::from(elem)).collect_vec(),
            job: elem.job.clone(),
            description: elem.description.clone(),
        }
    }
    pub fn to_widget(&self) -> pages::AnalysisElement {
        return pages::AnalysisElement {
            uuid: self.uuid,
            order: self.order,
            container: self.container,
            data: self.data.iter().map(|elem| elem.to_widget()).collect_vec(),
            output: self.output.iter().map(|elem| elem.to_widget()).collect_vec(),
            job: self.job.clone(),
            description: self.description.clone(),
        };
    }
}


#[derive(Clone, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct SerializableResultsElement {
    pub uuid: Uuid,
    pub order: f64,
    src: PathBuf,
    description: String
}

impl SerializableResultsElement {
    pub fn from(elem: &pages::ResultsElement) -> Self {
        Self {
            uuid: elem.id(),
            order: elem.element_order(),
            src: elem.src.clone(),
            description: elem.description.clone(),
        }
    }
    pub fn to_widget(&self) -> pages::ResultsElement {
        return pages::ResultsElement {
            uuid: self.uuid,
            order: self.order,
            src: self.src.clone(),
            description: self.description.clone(),
        };
    }
}

#[derive(Clone, PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct SerializableProjectPage {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub elements: Vec<SerializableProjectPageWidget>
}


impl SerializableProjectPage {
    pub fn new(title: String, description: String, elements: Vec<pages::ProjectPageWidget>) -> Self {
        return Self {
            id: Uuid::new_v4(),
            title,
            description,
            elements: elements.iter()
                        .filter_map(SerializableProjectPageWidget::from)
                        .collect_vec()
        }
    }
}






