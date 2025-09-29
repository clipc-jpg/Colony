
#![allow(unused)]


use std::error::Error;
use std::sync::Arc;
use std::path::PathBuf;

use uuid::Uuid;
use chrono::{DateTime, Utc};
use actix_web::{web, HttpResponse};

use plugin_interface_elements::{elements_v1, TargetSystem, VersionedRequest, VersionedResponse};





#[actix_web::get("/api")]
pub async fn api_endpoint(bodydata: web::Json<VersionedRequest>,
                          //local_message_db: web::Data<HardTypedDBAccess>,
                          local_job_queue: web::Data<LocalJobQueue>,
                          remote_process_store: web::Data<RemoteProcessStore>)  -> Result<HttpResponse, actix_web::Error> {
     let binding = &(*bodydata);
     match binding {
          &VersionedRequest::ApiV1(TargetSystem::LocalMachine,ref request) => {

               // TODO: log raw request
               // local_message_db.log_request()
               let response_body = api_endpoint_localmachine_logic_v1(request);
               return HttpResponse::Ok().json(response_body);
          },
          &VersionedRequest::ApiV1(TargetSystem::RemoteMachine(_), _) => {
               return HttpResponse::UnprocessableEntity().finish();
          },
     }
}

pub fn api_endpoint_localmachine_logic_v1(bodydata: &elements_v1::PluginTaskRequest)  -> elements_v1::PluginTaskResponse {
     match bodydata {
          elements_v1::PluginTaskRequest::AddServerAccess(addserveraccess_data) => {
               let response_data = elements_v1::AddServerAccessResponse {
                    localhost_port: Err(elements_v1::RemoteOperationError::NotSupported)
               };
               let response_body = elements_v1::PluginTaskResponse::AddServerAccess(response_data);

               return response_body;
          },
          elements_v1::PluginTaskRequest::EditServerAccess(editserveraccess_data) => {
               let response_data = elements_v1::EditServerAccessResponse {
                    localhost_port: Err(elements_v1::RemoteOperationError::NotSupported)
               };
               let response_body = elements_v1::PluginTaskResponse::EditServerAccess(response_data);

               return response_body;
          },
          elements_v1::PluginTaskRequest::EditServerConfiguration(editserverconfiguration_data) => {
               let response_data = elements_v1::EditServerConfigurationResponse {
                    localhost_port: Err(elements_v1::RemoteOperationError::NotSupported)
               };
               let response_body = elements_v1::PluginTaskResponse::EditServerConfiguration(response_data);

               return response_body;
          },
          elements_v1::PluginTaskRequest::ConnectToServer(connecttoserver_data) => {
               let response_data = elements_v1::ConnectToServerResponse {
                    success: Err(elements_v1::RemoteOperationError::NotSupported)
               };
               let response_body = elements_v1::PluginTaskResponse::ConnectToServer(response_data);

               return response_body;
          },
          elements_v1::PluginTaskRequest::DisconnectFromServer(disconnectfromserver_data) => {
               let response_data = elements_v1::DisconnectFromServerResponse {
                    success: Err(elements_v1::RemoteOperationError::NotSupported)
               };
               let response_body = elements_v1::PluginTaskResponse::DisconnectFromServer(response_data);

               return response_body;
          },
          elements_v1::PluginTaskRequest::DisconnectFromAllServers(disconnectfromallservers_data) => {
               let response_data = elements_v1::DisconnectFromAllServersResponse {
                    success: Err(elements_v1::RemoteOperationError::NotSupported)
               };
               let response_body = elements_v1::PluginTaskResponse::DisconnectFromAllServers(response_data);

               return response_body;
          },
          elements_v1::PluginTaskRequest::ListDirectory(listdirectory_data) => {
               let read_result = std::fs::read_dir(&listdirectory_data.directory);

               let content = match read_result {
                    Ok(entries) => {
                         entries.into_iter().filter_map(|res| res.map(|entry| {
                              elements_v1::FsElement::try_from(entry.path()).ok()
                         } ))
                    },
                    Err(e) => {
                         Err(RemoteOperationError::InternalFailure(format!("{:?}", &e)))
                    }
               };

               let response_data = elements_v1::ListDirectoryResponse { content };
               let response_body = elements_v1::PluginTaskResponse::ListDirectory(response_data);

               return response_body;

          },
          elements_v1::PluginTaskRequest::MoveFile(movefile_data) => {
               // TODO: log Job creation

               let elements_v1::MoveFile { source, target } = *movefile_data;
               match std::fs::rename(source, target) {
                    Ok(()) => {
                         // TODO: log job completion
                         let response_data = elements_v1::MoveFileResponse { destination: Ok(target) };
                         let response_body = elements_v1::PluginTaskResponse::MoveFile(response_data);
                         return response_body;
                    },
                    // TODO handle errors that will make copy fail as well here already? Or wait for copy to fail?
                    Err(_e) => {
                         todo!()
                         // Step2 create entry in database

                         // Step3 schedule subprocess
                         // step3b  have subprocess enter job completion and add completion message to queue
                    }
               }
          },
          elements_v1::PluginTaskRequest::CopyFile(copyfile_data) => { },
          elements_v1::PluginTaskRequest::DeleteFile(deletefile_data) => { },
          elements_v1::PluginTaskRequest::ShowFileMetadata(showfilemetadata_data) => { },
          elements_v1::PluginTaskRequest::DownloadData(downloaddata_data) => { },
          elements_v1::PluginTaskRequest::RunSingularityJob(runsingularityjob_data) => { },
          elements_v1::PluginTaskRequest::ShowSingularityJobLogs(showsingularityjoblogs_data) => { },
          elements_v1::PluginTaskRequest::ShowSingularityJobsRunning(showsingularityjobsrunning_data) => { },
          elements_v1::PluginTaskRequest::EnqueueMultipleJobs(enqueuemultiplejobs_data) => { },
          elements_v1::PluginTaskRequest::StopRunningJobs(stoprunningjobs_data) => { },
          elements_v1::PluginTaskRequest::SendMessages(sendmessages_data) => { },
          elements_v1::PluginTaskRequest::Terminate(terminate_data) => { },
     }
     return Ok(HttpResponse::Ok().finish());
}





























