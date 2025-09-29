




use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::data_elements::{elements_v1};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum TargetSystem {
    Frontend,
    LocalMachine,
    RemoteMachine(String)
}


#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag="version", content="data")]
pub enum VersionedRequest {
    ApiV1(TargetSystem, elements_v1::RequestId, elements_v1::TaskRequest)
}

impl VersionedRequest {
    pub fn target_plugin(&self) -> TargetSystem {
        return match self {
            VersionedRequest::ApiV1(target,_,_) => target.clone()
        }
    }
}

impl std::fmt::Display for VersionedRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Versioned API Request to: {:?}", &self.target_plugin())
    }
}

//################################################################################
//## Versioned Response
//################################################################################


#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag="version", content="data")]
pub enum VersionedResponse {
    ApiV1(TargetSystem, elements_v1::RequestId, elements_v1::PluginTaskResponse)
}

impl VersionedResponse {
    pub fn target_plugin(&self) -> TargetSystem {
        return match self {
            VersionedResponse::ApiV1(target, _, _) => target.clone()
        }
    }

    pub fn request_id(&self) -> elements_v1::RequestId {
        return match self {
            VersionedResponse::ApiV1(_, id, _) => *id
        }
    }
}

impl std::fmt::Display for VersionedResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Versioned API Response from: {:?}", &self.target_plugin())
    }
}



