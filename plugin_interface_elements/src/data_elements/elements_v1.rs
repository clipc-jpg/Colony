

use std::path::PathBuf;
use std::fmt::Debug;

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};


//################################################################################
//## Metadata
//################################################################################

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct StandardMetadata {
    //TODO
}

//################################################################################
//## Error Types
//################################################################################

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum RemoteOperationError {
    NotSupported,
    ParsingError(String),
    IncorrectParameters(String),
    ClientServerInconsistency(String),
    InternalFailure(String)
}


//################################################################################
//## Identifiers
//################################################################################

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PluginId { 
    pub clear_name: String
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct RequestId { 
    pub inner: Uuid
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct ChannelId { 
    pub inner: Uuid
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct JobId { 
    pub id: Uuid, 
    pub generation_time: DateTime<Utc>
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct DownloadId { 
    pub inner: Uuid
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct MessageId { 
    pub inner: Uuid
}


//################################################################################
//## Request data types
//################################################################################


#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CentralServerRequest {
    pub target_plugin: PluginId, 
    pub request_id: RequestId, 
    pub plugin_request: PluginTaskRequest
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum TaskRequest {
    PluginTaskRequest(PluginTaskRequest),
    FrontendTaskRequest(FrontendTaskRequest),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum PluginTaskRequest {
    AddServerAccess(AddServerAccess),
    EditServerAccess(EditServerAccess),
    EditServerConfiguration(EditServerConfiguration),
    ConnectToServer(ConnectToServer),
    DisconnectFromServer(DisconnectFromServer),
    DisconnectFromAllServers(DisconnectFromAllServers),
    ListDirectory(ListDirectory),
    MoveFile(MoveFile),
    CopyFile(CopyFile),
    DeleteFile(DeleteFile),
    ShowFileMetadata(ShowFileMetadata),
    DownloadData(DownloadData),
    RunSingularityJob(RunSingularityJob),
    ShowSingularityJobLogs(ShowSingularityJobLogs),
    ShowSingularityJobsRunning(ShowSingularityJobsRunning),
    EnqueueMultipleJobs(EnqueueMultipleJobs),
    StopRunningJobs(StopRunningJobs),
    SendMessages(SendMessages),
    Terminate(Terminate),
}


#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum FrontendTaskRequest {
    RequestConfiguration(RequestConfiguration),
    HaveConfigurationStored(HaveConfigurationStored),
    OpenChatChannel(OpenChatChannel),
    CloseChatChannel(CloseChatChannel),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AddServerAccess { }          // Plugin handles all logic and sends a URL to be displayed in an iframe

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EditServerAccess { }         // Plugin handles all logic and sends a URL to be displayed in an iframe

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EditServerConfiguration { }  // Plugin handles all logic and sends a URL to be displayed in an iframe

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ConnectToServer {            //pub  Plugin handles all logic and connects to its
    pub server_name: String
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DisconnectFromServer { 
    pub server_name: String
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DisconnectFromAllServers { }





#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ListDirectory { 
    pub directory: std::path::PathBuf
}


#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct MoveFile { 
    pub source: std::path::PathBuf, 
    pub target: std::path::PathBuf
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CopyFile { 
    pub source: std::path::PathBuf, 
    pub target: std::path::PathBuf
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DeleteFile { 
    pub file_path: std::path::PathBuf
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ShowFileMetadata { 
    pub file_path: std::path::PathBuf
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DownloadData { 
    pub url: String, 
    pub auth: Option<DownloadAuth>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DownloadAuth { 
    pub username: String, 
    pub password: String
}


#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RunSingularityJob { 
    pub specification: RemoteSingularityJob
}

//pub TODO: is this too general?
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RemoteSingularityJob { 
    pub singularity_container: PathBuf, 
    pub configuration: PathBuf, 
    pub working_directory: PathBuf
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ShowSingularityJobLogs { 
    pub job: JobId
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ShowSingularityJobsRunning { }

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EnqueueMultipleJobs { }

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct StopRunningJobs { }



#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SendMessages { }



#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Terminate { }





#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RequestConfiguration { }

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct HaveConfigurationStored { }

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct OpenChatChannel { }

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CloseChatChannel { }




//################################################################################
//## Plugin Responses
//################################################################################

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum PluginTaskResponse {
    AddServerAccess(AddServerAccessResponse),
    EditServerAccess(EditServerAccessResponse),
    EditServerConfiguration(EditServerConfigurationResponse),
    ConnectToServer(ConnectToServerResponse),
    DisconnectFromServer(DisconnectFromServerResponse),
    DisconnectFromAllServers(DisconnectFromAllServersResponse),
    ListDirectory(ListDirectoryResponse),
    MoveFile(MoveFileResponse),
    CopyFile(CopyFileResponse),
    DeleteFile(DeleteFileResponse),
    ShowFileMetadata(ShowFileMetadataResponse),
    DownloadData(DownloadDataResponse),
    RunSingularityJob(RunSingularityJobResponse),
    ShowSingularityJobLogs(ShowSingularityJobLogsResponse),
    ShowSingularityJobsRunning(ShowSingularityJobsRunningResponse),
    EnqueueMultipleJobs(EnqueueMultipleJobsResponse),
    StopRunningJobs(StopRunningJobsResponse),
    SendMessages(SendMessagesResponse),
    Terminate(TerminateResponse),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum FrontendTaskResponse {
    RequestConfiguration(RequestConfigurationResponse),
    HaveConfigurationStored(HaveConfigurationStoredResponse),
    OpenChatChannel(OpenChatChannelResponse),
    CloseChatChannel(CloseChatChannelResponse),
}




#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AddServerAccessResponse { 
    pub localhost_port: Result<u16, RemoteOperationError>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EditServerAccessResponse { 
    pub localhost_port: Result<u16, RemoteOperationError>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EditServerConfigurationResponse { 
    pub localhost_port: Result<u16, RemoteOperationError>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ConnectToServerResponse { 
    pub success: Result<(), RemoteOperationError>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DisconnectFromServerResponse { 
    pub success: Result<(), RemoteOperationError>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DisconnectFromAllServersResponse { 
    pub success: Result<(), RemoteOperationError>
}


#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RequestConfigurationResponse { 
    pub configuration: Option<String>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct HaveConfigurationStoredResponse { 
    pub success: Result<(), RemoteOperationError>
}


#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct OpenChatChannelResponse { 
    pub channel_id: Result<ChannelId, RemoteOperationError>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CloseChatChannelResponse { 
    pub channel_id: Result<ChannelId, RemoteOperationError>
}


#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ListDirectoryResponse { 
    pub content: Result<Vec<FsElement>, RemoteOperationError>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum FsElement {
    File(PathBuf),
    Directory(PathBuf),
}

impl TryFrom<PathBuf> for FsElement {
    type Error = std::io::Error;

    fn try_from(pth: PathBuf) -> std::io::Result<Self> {
        let pth = pth.canonicalize()?;
        if pth.exists() {
            if pth.is_file() { Ok(FsElement::File(pth)) }
            else if pth.is_dir() { Ok(FsElement::Directory(pth)) }
            else  { return Err(std::io::Error::new(std::io::ErrorKind::Other, "Neither File nor Directory")) }
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Not found: {pth:?}")))
        }
    }
}


#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct MoveFileResponse { 
    pub destination: Result<PathBuf, RemoteOperationError>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CopyFileResponse { 
    pub destination: Result<PathBuf, RemoteOperationError>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DeleteFileResponse { 
    pub success: Result<(), RemoteOperationError>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ShowFileMetadataResponse { 
    pub meta: Result<StandardMetadata, RemoteOperationError>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct DownloadDataResponse { 
    pub id: Result<DownloadId, RemoteOperationError>
}


#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct RunSingularityJobResponse { 
    pub success: Result<JobId, RemoteOperationError>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ShowSingularityJobLogsResponse { 
    pub logs: Result<Vec<String>, RemoteOperationError>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ShowSingularityJobsRunningResponse { 
    pub running_jobs: Result<Vec<JobId>, RemoteOperationError>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EnqueueMultipleJobsResponse { 
    pub success: Result<(), RemoteOperationError>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct StopRunningJobsResponse { 
    pub success: Result<(), RemoteOperationError>
}



#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SendMessagesResponse { 
    pub requests: Vec<FrontendTaskRequest>, 
    pub messages: Vec<PluginMessage>
}



#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TerminateResponse { }


#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PluginMessage { 
    pub id: MessageId, 
    pub creation_time: DateTime<Utc>, 
    pub short_summary_title: String, 
    pub text: String
}


