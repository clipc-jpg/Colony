
use std::collections::HashMap;
use std::os::windows::fs::FileTypeExt;
use std::os::windows::process::ExitStatusExt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::io::{BufWriter, Write};
use std::process::Child;
use std::process::ExitStatus;

use dioxus::signals::SyncSignal;
use itertools::Itertools;
use uuid::Uuid;
use futures_util::StreamExt;
use regex::Regex;

use crate::pages::AppState;
use crate::{backend, singularity_run};
use crate::backend::utils::*;
use crate::backend::jobs::*;
use crate::persistent_state::exe_dir;
use crate::components::FilesystemData;

use crate::backend::persistent_state::{config_path, PersistentState, PersistentStateUpdate};

use super::persistent_state;

#[derive(Debug)]
pub struct CommChannel<T1,T2> {
    id: Uuid,
    pub receiver: Receiver<T1>,
    pub sender: Sender<T2>,
    backsender: Sender<T1>
}

impl<T1,T2> CommChannel<T1,T2> {

    pub fn try_receive(&self) -> Result<T1, mpsc::TryRecvError> {
        return self.receiver.try_recv();
    }

    #[allow(dead_code)]
    pub fn receive(&self) -> Result<T1, mpsc::RecvError> {
        return self.receiver.recv();
    }

    pub fn send(&self, msg: T2) -> Result<(), mpsc::SendError<T2>> {
        return self.sender.send(msg);
    }

    pub fn reinsert_message(&self, msg: T1) -> Result<(), mpsc::SendError<T1>> {
        return self.backsender.send(msg);
    }
}

//impl<T1, T2:Clone> Clone for CommChannel<T1,T2> {
//    fn clone(&self) -> Self {
//        return Self {id: self.id, receiver: self.receiver, sender: self.sender.clone()};
//    }
//}

impl<T1, T2> PartialEq for CommChannel<T1, T2> {
    fn eq(&self, ch2: &Self) -> bool {
        return self.id == ch2.id;
    }
    fn ne(&self, ch2: &Self) -> bool {
        return self.id != ch2.id;
    }
}

pub fn commchannel_pair<T1,T2>() -> (CommChannel<T1,T2>, CommChannel<T2,T1>) {
    let (sender1, receiver1) = mpsc::channel();
    let (sender2, receiver2) = mpsc::channel();

    let com1 = CommChannel {id: Uuid::new_v4(), receiver: receiver1, sender: sender2.clone(), backsender: sender1.clone()};
    let com2 = CommChannel {id: Uuid::new_v4(), receiver: receiver2, sender: sender1, backsender: sender2};

    return (com1, com2);
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum BackendRequest {
    //Installation
    CheckWSL(Arc<Mutex<Vec<String>>>,SyncSignal<usize>), // No PartialEq; SyncSignal is unsafe to use
    CheckWSL_v2(Arc<Mutex<Vec<String>>>,SyncSignal<usize>), // No PartialEq; SyncSignal is unsafe to use
    //Have frontend change the rendered page
    SetAppState(AppState),
    //Reading, Updating, Writing persistent state
    ReadPersistentState,
    UpdatePersistentState(backend::persistent_state::PersistentStateUpdate),
        // backend decides when to save state
    //interacting with singularity containers locally
    QuerySingularity(PathBuf, SingularityQuery), // container path, query object
    RunSingularity(PathBuf, PathBuf, Vec<String>), // workdir, container path, container_args
    RunSingularityApp(PathBuf, PathBuf, String, Vec<String>), // workdir, containerpath, app name, app arguments
    StartLocalWebServer(AppState, PathBuf), // page to return to after completion and PathBuf is communication partner container identified by path
    InformLocalWebserverStarted(Result<u16, Arc<Box<std::io::Error>>>),
    InformContainerWebserverStarted(Result<u16, Arc<Box<std::io::Error>>>),
    AcceptConfiguration(PathBuf, String),
    SendConfiguration(),
    ExportAnalysisIntoRepository(JobId, PathBuf, PathBuf, PathBuf), // workdir == export target, container path, configuration file (also contains content to
    // Starting and stopping jobs
    // this request is especially security critical and may be abused
    StartJob(PathBuf, String),
    SendJobInfo(JobId),
    SendJobOutput(JobId, usize),
    StopProcess(JobId),
    StopAllProcesses,
    StopProgram,

    // file interactions and remote file interactions
    InspectFilesystem(FilesystemData, PathBuf),
    ListDirectory(FilesystemData, PathBuf),
    MoveContent(PathBuf, PathBuf),
    DownloadContent(String, PathBuf),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BackendResponse {
    //Installation
    InstallStepCompleted(InstallationState),
    //Have frontend change the rendered page
    SetAppState(AppState),
    //Reading, Updating, Writing persistent state
    PersistentState(PersistentState), // Updates either all return the newly updated state, or nothing at all
    AddedNewContainer(persistent_state::BackendContainerDescription),
    //interacting with singularity containers locally
    SingularityInfo(PathBuf, Option<SingularityResponse>),
    LocalWebServerStarted(Option<u16>), // port number
    ContainerWebServerStarted(Option<u16>), // port number
    Configuration(PathBuf, String), // container identified by path, and the corresponding configuration
    ExportedAnalysisIntoRepository(Option<PathBuf>), // workdir == export target, if successfull
    // Starting and stopping jobs
    JobInfo(JobId, JobState),
    JobOutput(JobId, Vec<String>),
    StoppedProcess(JobId),
    StoppedAllProcesses,
    // TODO: declare Error types
    // file interactions and remote file interactions
    FileCreated(Option<PathBuf>),
    #[allow(unused)]
    FileList(Vec<String>),
    ListDirectory(Result<DirectoryContents, Result<DirectoryContents, ()>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectoryContents {
    pub root: PathBuf,
    pub files: Vec<String>,
    pub directories: Vec<String>
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstallationState {
    InstallationStarted,
    InstallingWSL,
    ImportingDistribution,
    DistributionWasNotFound,
    DistributionNeedsToBeSelected,
    InstallingSingularity,
    InstallationEnded,
    InstallationFailed,
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub enum SingularityQuery { // container identifier is in the Query enum
    AppList,
    AppRequirements,
    AppConfigurationOptions,
    Inspection,
    RunHelp,
    AppHelp(String),
    RunLabels,
    AppLabels(String),
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub enum SingularityResponse {
    AppList(Vec<String>),
    AppRequirements(String),
    AppConfigurationOptions(String),
    Inspection(serde_json::Value),
    RunHelp(String),
    AppHelp(String),
    RunLabels(String),
    AppLabels(String),
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub enum JobQuery {
    JobState(JobId),
    JobOutput(JobId),
}

#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub enum JobResponse {
    JobScheduled(JobId),
    JobState(JobId, JobState),
    JobOutput(JobId, String, String) // StdOut, StdErr
}


#[allow(unused)]
#[derive(Debug, Clone, PartialEq)]
pub enum JobState {
    Scheduled,
    Submitted,
    Running,
    Completed(ExitStatus),
    Unknown,
    JobNotListed
}

pub type FrontendCommChannel = CommChannel<BackendResponse, BackendRequest>;
pub type BackendCommChannel = CommChannel<BackendRequest, BackendResponse>;


//pub enum Process {
//    RunningProcess(Child, Option<String>), // process, Configuration-String if it is a colony container job
//    CompletedProcess(Child, Option<String>),
//    FailedProcess(Child, Option<String>), //TODO: needs more info like Errorcode, Errormessage
//    ToBeScheduled,
//}
//type ProcessStore = HashMap<JobId, Arc<Mutex<Process>>>;              //TODO: are configurations stored here, too?

type ContainerJobStore = HashMap<PathBuf, Arc<Mutex<Vec<JobId>>>>;
type JobOutputStore = HashMap<JobId, Arc<Mutex<Vec<String>>>>;


struct ProcessStore {
        store: HashMap<JobId, Arc<Mutex<Child>>>
}

#[allow(unused)]
impl ProcessStore {
    pub fn new() -> ProcessStore {
        return ProcessStore { store: HashMap::new() };
    }

    pub fn insert(&mut self, key: JobId, ch: Arc<Mutex<Child>>) {
        self.store.insert(key, ch);
    }

    pub fn get(&mut self, jbd: &JobId) -> Option<&Arc<Mutex<Child>>> {
        return self.store.get(jbd);
    }

    pub fn remove(&mut self, jbd: &JobId) -> Option<Arc<Mutex<Child>>>{
        return self.store.remove(&jbd);
    }
}

impl Drop for ProcessStore {
    fn drop(&mut self) {
        self.store.values_mut().for_each(|arc| {
            if let Ok(mut child) = arc.lock() {
                child.kill().ok();
            }
        });
    }
}

//struct JobStore {
//    store: HashMap<JobId, JobData>,
//}
//
//impl JobStore {
//    pub fn new() -> JobStore {
//        return JobStore { store: HashMap::new() };
//    }
//
//    pub fn get(&mut self, jbd: &JobId) -> Option<&JobData> {
//        //return self.store.entry(jbd).or_insert_with(|| JobData::new());
//        return self.store.get(jbd);
//    }
//
//    pub fn remove(&mut self, jbd: &JobId) -> Option<JobData> {
//        return self.store.remove(&jbd);
//    }
//}

//################################################################################
//## running the background thread content
//################################################################################
#[tokio::main]
pub async fn listen(comm_with_frontend: BackendCommChannel) {

    let mut container_infos = ContainerJobStore::new();
    let mut process_store = ProcessStore::new();
    let mut process_outputs = JobOutputStore::new();

    let mut persistent_state = PersistentState::from_file(config_path());

    loop {
        let message = comm_with_frontend.receiver.recv();
        //println!("Backend received message: {:?}", message);
        match message {
            Ok(task) => match task {
                BackendRequest::CheckWSL(mut output_collection, _child_output_count) => {
                    let msg = "Backend tasked with: Checking WSL";
                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                    println!("{}", &msg);
                    match backend::check_wsl() {
                        Ok(out) => {
                            let msg = String::from_utf8(out.stdout.clone())
                                        .unwrap_or_else(|_| {
                                            let u8s = out.stdout.clone();
                                            let u16s: Vec<u16> = u8s
                                                .chunks_exact(2)
                                                .into_iter()
                                                .map(|a| u16::from_ne_bytes([a[0], a[1]]))
                                                .collect();
                                            String::from_utf16(u16s.as_slice())
                                                .unwrap_or_else(|_| String::from("Error parsing WSL status output"))
                                        });
                            println!("{}", msg);
                            output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                        },
                        Err(_) => {
                            let msg = "Error checking WSL!";
                            println!("{}", &msg);
                            output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                        }
                    };
                    comm_with_frontend.send(BackendResponse::InstallStepCompleted(InstallationState::InstallationStarted)).ok();


                    let msg = "Backend tasked with: Installing WSL";
                    println!("{}", &msg);
                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                    let msg = "Installing WSL...";
                    println!("{}", &msg);
                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                    //match backend::install_wsl_only_child() {
                    match backend::request_wsl_only_child() {
                        Ok(mut child) => {
                            backend::bufread_child_stdout_bytes_into_messages(&mut output_collection, &mut child)
                        },
                        Err(_) => {
                            let msg = "Error installing WSL";
                            println!("{}", &msg);
                            output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                        }
                    }
                    let msg = "Installing WSL Distribution...";
                    println!("{}", &msg);
                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                    match backend::install_wsl_distribution_only_child() {
                        Ok(mut child) => {
                            backend::bufread_child_stdout_bytes_into_messages(&mut output_collection, &mut child)
                        },
                        Err(_) => {
                            let msg = "Error installing WSL";
                            println!("{}", &msg);
                            output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                        }
                    }
                    let msg = "Updating WSL...";
                    println!("{}", &msg);
                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                    match backend::update_wsl_only_child() {
                        Ok(mut child) => {
                            backend::bufread_child_stdout_bytes_into_messages(&mut output_collection, &mut child);
                        },
                        Err(_) => {
                            let msg = "Error updating WSL";
                            println!("{}", &msg);
                            output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                        }
                    }
                    comm_with_frontend.send(BackendResponse::InstallStepCompleted(InstallationState::InstallingWSL)).ok();


                    let msg = "Backend tasked with: Exporting and Reimporting WSL";
                    println!("{}", &msg);
                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                    let distros = backend::find_wsl_distros();
                    if !distros.contains(&"Ubuntu".to_owned()) {

                        if distros.contains(&"DEFAULT_DISTRO_NOT_FOUND".to_string()) {
                            let msg = "Setting Default Distribution to Ubuntu...";
                            println!("{}", &msg);
                            match backend::set_wsl_set_default_distro_child() {
                                Ok(mut child) => {
                                    backend::bufread_child_stdout_bytes_into_messages(&mut output_collection, &mut child);
                                },
                                Err(_) => {
                                    let msg = "Error setting Default Distribution to Ubuntu";
                                    println!("{}", &msg);
                                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                                }
                            }
                        }


                        let msg = "Installing Ubuntu-Distribution...";
                        println!("{}", &msg);
                        output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                        //################################################################################
                        //##  TODO: print to board
                        //################################################################################
                        //backend::install_wsl_ubuntu();

                        match backend::install_wsl_ubuntu_child() {
                            Ok(mut child) => {
                                backend::bufread_child_stdout_bytes_into_messages(&mut output_collection, &mut child);
                            },
                            Err(_) => {
                                let msg = "Error installing Ubuntu Distribution";
                                println!("{}", &msg);
                                output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                            }
                        }

                    };
                    if !distros.contains(&"ColonyWSL".to_owned()) {
                        //TODO: export WSL, reimport WSL
                        //################################################################################
                        //## TODO: in case exe_dir could not be determined:
                        //## TODO: create temporary file instead
                        //## TODO: reserve temporary file
                        //## TODO: delete temporary file
                        //################################################################################
                        let path = match exe_dir() {
                          Some(exe_pth) => {let mut pth = exe_pth.clone(); pth.push("ColonyWSL.tar"); pth},
                          None => {let mut pth = PathBuf::new(); pth.push("ColonyWSL.tar"); pth}
                        };
                        let msg = "Exporting and Reimporting Ubuntu-Distribution...";
                        println!("{}", &msg);
                        output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                        let pathstr = path.to_string_lossy();
                        //backend::wsl_export_reimport_ubuntu(&pathstr);
                        match backend::wsl_export_ubuntu_child(&pathstr) {
                            Ok(mut child) => {
                                //backend::bufread_child_stdout(backend_output, child);
                                backend::bufread_child_stdout_bytes_into_messages(&mut output_collection, &mut child);
                            },
                            Err(_) => {
                                let msg = "Error reimporting Ubuntu Distribution under new name";
                                println!("{}", &msg);
                                output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                            }
                        }

                        match backend::wsl_reimport_ubuntu_child(&pathstr) {
                            Ok(mut child) => {
                                //backend::bufread_child_stdout(backend_output, child);
                                backend::bufread_child_stdout_bytes_into_messages(&mut output_collection, &mut child);
                            },
                            Err(_) => {
                                let msg = "Error reimporting Ubuntu Distribution under new name";
                                println!("{}", &msg);
                                output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                            }
                        }
                    };
                    comm_with_frontend.send(BackendResponse::InstallStepCompleted(InstallationState::ImportingDistribution)).ok();


                    let msg = "Backend tasked with: Installing Singularity";
                    println!("{}", &msg);
                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                    match backend::check_singularity_version() {
                        Ok(response) => {
                            let msg = format!("Singularity installed: {}", match std::str::from_utf8(&response.stdout) {
                                Ok(s) => s,
                                Err(_) => "Something went wrong"
                            });
                            println!("{}", &msg);
                            output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                        },
                        Err(_) => {
                            let msg = "Installing Singularity...";
                            println!("{}", &msg);
                            output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                            //backend::install_singularity().ok();

                            match backend::install_singularity_child() {
                                Ok(mut child) => {
                                    //backend::bufread_child_stdout(backend_output, child);
                                    backend::bufread_child_stdout_bytes_into_messages(&mut output_collection, &mut child);
                                    let msg = "Singularity has been installed";
                                    println!("{}", &msg);
                                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                                },
                                Err(_) => {
                                    let msg = "Error installing singularity in WSL";
                                    println!("{}", &msg);
                                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                                }
                            }
                        }
                    }
                    comm_with_frontend.send(BackendResponse::InstallStepCompleted(InstallationState::InstallingSingularity)).ok();


                    let msg = "Backend tasked with: Finishing Installation";
                    println!("{}", &msg);
                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();

                    comm_with_frontend.send(BackendResponse::InstallStepCompleted(InstallationState::InstallationEnded)).ok();


                    let msg = "Backend finished Installation";
                    println!("{}", &msg);
                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                },
                BackendRequest::CheckWSL_v2(mut output_collection, _child_output_count) => {
                    let msg = "Backend tasked with: Checking WSL";
                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                    println!("{}", &msg);
                    match backend::check_wsl() {
                        Ok(out) => {
                            let msg = String::from_utf8(out.stdout.clone())
                                        .unwrap_or_else(|_| {
                                            let u8s = out.stdout.clone();
                                            let u16s: Vec<u16> = u8s
                                                .chunks_exact(2)
                                                .into_iter()
                                                .map(|a| u16::from_be_bytes([a[0], a[1]]))
                                                .collect();
                                            String::from_utf16(u16s.as_slice())
                                                .unwrap_or_else(|_| String::from("Error parsing WSL status output"))
                                        });
                            println!("{}", msg);
                            output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                            comm_with_frontend.send(BackendResponse::InstallStepCompleted(InstallationState::InstallationStarted)).ok();
                        },
                        Err(_) => {
                            let msg = "Error checking WSL!";
                            println!("{}", &msg);
                            output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                            comm_with_frontend.send(BackendResponse::InstallStepCompleted(InstallationState::InstallationFailed)).ok();
                        }
                    };

                    let msg = "Backend tasked with: Installing WSL";
                    println!("{}", &msg);
                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                    let msg = "Installing WSL...";
                    println!("{}", &msg);
                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                    match backend::request_wsl_only_child() {
                        Ok(mut child) => {
                            backend::bufread_child_stdout_bytes_into_messages(&mut output_collection, &mut child)
                        },
                        Err(_) => {
                            let msg = "Error installing WSL";
                            println!("{}", &msg);
                            output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                        }
                    }
                    let msg = "Installing WSL Distribution...";
                    println!("{}", &msg);
                    //match backend::install_wsl_only_child() {
                    match backend::install_wsl_distribution_only_child() {
                        Ok(mut child) => {
                            backend::bufread_child_stdout_bytes_into_messages(&mut output_collection, &mut child)
                        },
                        Err(_) => {
                            let msg = "Error installing WSL";
                            println!("{}", &msg);
                            output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                            comm_with_frontend.send(BackendResponse::InstallStepCompleted(InstallationState::InstallationFailed)).ok();
                        }
                    }
                    let msg = "Updating WSL...";
                    println!("{}", &msg);
                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                    match backend::update_wsl_only_child() {
                        Ok(mut child) => {
                            backend::bufread_child_stdout_bytes_into_messages(&mut output_collection, &mut child);
                        },
                        Err(_) => {
                            let msg = "Error updating WSL";
                            println!("{}", &msg);
                            output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                            comm_with_frontend.send(BackendResponse::InstallStepCompleted(InstallationState::InstallationFailed)).ok();
                        }
                    }
                    comm_with_frontend.send(BackendResponse::InstallStepCompleted(InstallationState::InstallingWSL)).ok();

                    let msg = "Backend tasked with: Importing WSL Linux Distribution";
                    println!("{}", &msg);
                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                    let distros = backend::find_wsl_distros();
                    if !distros.iter().any(|distro| distro.starts_with("ColonyWSL")) {
                        let mut distro_path = exe_dir().clone().unwrap_or_else(PathBuf::new);
                        distro_path.push("assets");
                        distro_path.push("ColonyWSL.tar");
                        if !distro_path.is_file() {
                            comm_with_frontend.send(BackendResponse::InstallStepCompleted(InstallationState::DistributionWasNotFound));
                            std::thread::sleep(std::time::Duration::from_millis(1500));
                            match backend::wsl_manually_import_distro() {
                                Ok(mut child) => {
                                    backend::bufread_child_stdout_bytes_into_messages(&mut output_collection, &mut child);
                                },
                                Err(_) => {
                                    let msg = "Error importing Linux Distribution";
                                    println!("{}", &msg);
                                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                                    comm_with_frontend.send(BackendResponse::InstallStepCompleted(InstallationState::InstallationFailed)).ok();
                                }
                            }
                        }
                        else {
                            println!("Trying to import: {:?}", &distro_path);
                            let pathstr = distro_path.to_str().unwrap();
                            match backend::wsl_import_colony_wsl_child(&pathstr) {
                                Ok(mut child) => {
                                    //backend::bufread_child_stdout(backend_output, child);
                                    backend::bufread_child_stdout_bytes_into_messages(&mut output_collection, &mut child);
                                },
                                Err(_) => {
                                    let msg = "Error importing ColonyWSL Distribution";
                                    println!("{}", &msg);
                                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();
                                    comm_with_frontend.send(BackendResponse::InstallStepCompleted(InstallationState::InstallationFailed)).ok();
                                }
                            }
                        }

                    }
                    let distros = backend::find_wsl_distros();
                    if !distros.iter().any(|distro| distro.starts_with("ColonyWSL")) {
                        comm_with_frontend.send(BackendResponse::InstallStepCompleted(InstallationState::InstallationFailed)).ok();
                    } else {
                        comm_with_frontend.send(BackendResponse::InstallStepCompleted(InstallationState::ImportingDistribution)).ok();
                    }

                    let msg = "Backend tasked with: Finishing Installation";
                    println!("{}", &msg);
                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();

                    comm_with_frontend.send(BackendResponse::InstallStepCompleted(InstallationState::InstallationEnded)).ok();


                    let msg = "Backend finished Installation";
                    println!("{}", &msg);
                    output_collection.lock().as_mut().map(|outp| outp.push(String::from(msg))).ok();

                },
                BackendRequest::SetAppState(app_state) => {
                    comm_with_frontend.send(BackendResponse::SetAppState(app_state)).ok();
                },
                BackendRequest::ReadPersistentState => {
                    println!("Sending Peristent State response!");
                    comm_with_frontend.send(BackendResponse::PersistentState(persistent_state.clone())).ok();
                },
                BackendRequest::UpdatePersistentState(update) => {
                    println!("Persistent State will be updated!");

                    match update {
                        PersistentStateUpdate::AddContainer(ptb) => {
                            println!("Adding container {:?}", &ptb);
                            let bcd = crate::backend::persistent_state::BackendContainerDescription::from_path(ptb);
                            persistent_state.containers.push(bcd.clone());

                            // only retain unique entries (sorting is stable)
                            persistent_state.containers.sort_by(|elem1, elem2| elem1.path.cmp(&elem2.path));
                            persistent_state.containers.dedup_by(|elem1, elem2| elem1.path == elem2.path);

                            if persistent_state.containers.iter().any(|elem| elem.id == bcd.id) {
                                comm_with_frontend.send(BackendResponse::AddedNewContainer(bcd)).ok();
                            }
                        },
                        PersistentStateUpdate::RemoveContainerByPath(ptb) => {
                             println!("Removing container {:?}", &ptb);
                             persistent_state.containers.retain(|elem| elem.path != ptb);
                        },
                        PersistentStateUpdate::RemoveContainer(container_id) => {
                            println!("Removing container {:?}", &container_id);
                            persistent_state.containers.retain(|elem| elem.id != container_id);
                        },
                        PersistentStateUpdate::SetlastSelectedContainerDir(ptb) => {
                            println!("Setting last_selected_container_dir {:?}", &ptb);
                            persistent_state.last_selected_container_dir = ptb;
                        },
                        //PersistentStateUpdate::AddRecentProtocol(_ptb) => { todo!() },
//                        PersistentStateUpdate::DeleteProtocol(_ptb) => { todo!() },
//                        PersistentStateUpdate::ArchiveProtocol(_ptb) => { todo!() },
//                        PersistentStateUpdate::UnarchiveProtocol(_ptb) => { todo!() },
                    }

                    match config_path() {
                        Some(cpath) => { persistent_state.write_to_file(cpath); },
                        None => {}
                    }
                },
                BackendRequest::QuerySingularity(container_path, query) => {
                    match query {
                        SingularityQuery::AppList => {
                            match backend::singularity_app_list(&container_path) {
                                Some(msg) => {comm_with_frontend.send(BackendResponse::SingularityInfo(container_path, Some(SingularityResponse::AppList(msg)))).ok();},
                                None => {comm_with_frontend.send(BackendResponse::SingularityInfo(container_path, None)).ok();}
                            }
                        },
                        SingularityQuery::Inspection => {
                            match backend::singularity_inspection(&container_path) {
                                Some(msg) => {comm_with_frontend.send(BackendResponse::SingularityInfo(container_path, Some(SingularityResponse::Inspection(msg)))).ok();},
                                None => {comm_with_frontend.send(BackendResponse::SingularityInfo(container_path, None)).ok();}
                            }
                        },
                        SingularityQuery::RunHelp => {todo!()},
                        SingularityQuery::AppHelp(_appname) => {todo!()},
                        SingularityQuery::RunLabels => {todo!()},
                        SingularityQuery::AppLabels(_appname) => {todo!()},
                        // Specific to our containers
                        SingularityQuery::AppRequirements => {
                            match backend::singularity_app_requirements(&container_path) {
                                Some(msg) => {comm_with_frontend.send(BackendResponse::SingularityInfo(container_path, Some(SingularityResponse::AppRequirements(msg)))).ok();},
                                None => {comm_with_frontend.send(BackendResponse::SingularityInfo(container_path, None)).ok();}
                            }
                        },
                        SingularityQuery::AppConfigurationOptions => {
                            match backend::singularity_app_configuration_options(&container_path) {
                                Some(msg) => {comm_with_frontend.send(BackendResponse::SingularityInfo(container_path, Some(SingularityResponse::AppConfigurationOptions(msg)))).ok();},
                                None => {comm_with_frontend.send(BackendResponse::SingularityInfo(container_path, None)).ok();}
                            }
                        },
                    }
                },
                BackendRequest::RunSingularity(workdir, container_path, container_args) => {
                    println!("Backend tasked with: Starting Container {:?}", &container_path);
                    let child_result = backend::singularity_run_in_dir(&workdir, &container_path, container_args);

                    match child_result {
                        Ok(mut child) => {
                            //let jid = JobId::new(format!("Running: {}", &container_path));
                            let job_id = JobId::new();
                            let output_collector = process_outputs
                                                    .entry(job_id.clone())
                                                    .or_insert_with(|| Arc::new(Mutex::new(Vec::new())));
                            let mut output_collector = Arc::clone(&output_collector);
                            println!("Command: {:?}", &child);

                            let job_id_coll = container_infos.entry(PathBuf::from(container_path))
                                .or_insert(Arc::new(Mutex::new(Vec::new() as Vec<JobId>)));
                            job_id_coll.lock().map(|mut vec_jid| vec_jid.push(job_id.clone())).ok();

                            let mut child_stdout = child.stdout.take().unwrap();
                            tokio::spawn(async move {
                                backend::bufread_stdout_bytes_into_messages(&mut output_collector, &mut child_stdout);
                            });

                            process_store.insert(job_id.clone(), Arc::new(Mutex::new(child)));

                            comm_with_frontend.send(BackendResponse::JobInfo(job_id, JobState::Running)).ok();
                        },
                        Err(e) => {
                            println!("Error starting container: {:?}", &e);
                        }
                    }
                },
                BackendRequest::RunSingularityApp(workdir, container_path, app_name, app_args) => {
                    println!("Backend tasked with: Starting App '{}' of container {:?}", &app_name, &container_path);
                    let child_result = backend::singularity_run_app_in_dir(&workdir, &container_path, &app_name, app_args);
                    match child_result {
                        Ok(mut child) => {
                            println!("Command: {:?}", &child);
                            let job_id = JobId::new();
                            let output_collector = process_outputs
                                                    .entry(job_id.clone())
                                                    .or_insert_with(|| Arc::new(Mutex::new(Vec::new())));
                            let mut output_collector = Arc::clone(&output_collector);

                            let job_id_coll = container_infos.entry(PathBuf::from(container_path))
                                .or_insert(Arc::new(Mutex::new(Vec::new() as Vec<JobId>)));
                            job_id_coll.lock().map(|mut vec_jid| vec_jid.push(job_id.clone())).ok();

                            let mut child_stdout = child.stdout.take().unwrap();
                            tokio::spawn(async move {
                                backend::bufread_stdout_bytes_into_messages(&mut output_collector, &mut child_stdout);
                            });

                            process_store.insert(job_id.clone(), Arc::new(Mutex::new(child)));

                            comm_with_frontend.send(BackendResponse::JobInfo(job_id, JobState::Running)).ok();
                        },
                        Err(_) => {}
                    }
                },
                BackendRequest::DownloadContent(source_url, destination_path) => {

                    //let file_errored = false;
                    match std::fs::File::create(destination_path.clone()) {
                        Ok(target_file) => {
                            let resp = reqwest::get(&source_url)
                                .await;

                            match resp {
                                Ok(msg) => {
                                    let mut file_errored = false;

                                    {
                                        let mut writer = BufWriter::new(target_file);
                                        let mut byte_stream = msg.bytes_stream();

                                        while let Some(chunk_result) = byte_stream.next().await {
                                            match chunk_result {
                                                Ok(chunk) => {
                                                    match writer.write(&chunk) {
                                                        Ok(_write_len) => {},
                                                        Err(_) => {
                                                            file_errored = true;
                                                            break;
                                                        }
                                                    }
                                                },
                                                Err(_) => {file_errored = true; break;}
                                            }
                                        }
                                        writer.flush().ok();
                                    }

                                    if file_errored {
                                        std::fs::remove_file(destination_path).ok();
                                        comm_with_frontend.send(BackendResponse::FileCreated(None)).ok();
                                    }
                                    else {comm_with_frontend.send(BackendResponse::FileCreated(Some(destination_path))).ok();}
                                },
                                Err(_e) => {comm_with_frontend.send(BackendResponse::FileCreated(None)).ok();}
                            }
                        },
                        Err(_) => {comm_with_frontend.send(BackendResponse::FileCreated(None)).ok();}
                    }
                },
                BackendRequest::MoveContent(source_path, destination_path) => {
                    let mut operation_succeeded = false;
                    if source_path.is_file() {
                        match std::process::Command::new("cmd /C mv -f")
                                .args([&source_path.to_string_lossy().into_owned(), &destination_path.to_string_lossy().into_owned()])
                                .output() {
                                    Ok(_) => {operation_succeeded = true;},
                                    Err(_) => {operation_succeeded = false;}
                        }

                    } else if source_path.is_dir() {
                        match std::process::Command::new("cmd /C mv -rf")
                                .args([&source_path.to_string_lossy().into_owned(), &destination_path.to_string_lossy().into_owned()])
                                .output() {
                                    Ok(_) => {operation_succeeded = true;},
                                    Err(_) => {operation_succeeded = false;}
                        }
                    }
                    if operation_succeeded {comm_with_frontend.send(BackendResponse::FileCreated(Some(destination_path))).ok();}
                    else {comm_with_frontend.send(BackendResponse::FileCreated(None)).ok();}
                },
                BackendRequest::StartLocalWebServer(followup_page, comm_partner_container) => {
                    let job_id = JobId::new();
                    println!("Backend starts Webserver with JobId {:?}", &job_id);
                    let _output_collector = process_outputs
                                                    .entry(job_id.clone())
                                                    .or_insert_with(|| Arc::new(Mutex::new(Vec::new())));

                    let comm_with_backend = comm_with_frontend.backsender.clone();
                    backend::start_frontend_local_server(followup_page, comm_partner_container, job_id, comm_with_backend).await;

                    // webserver thread will notify backend about success

                    println!("Frontend-local webserver has been requested");
                },
                BackendRequest::InformLocalWebserverStarted(maybe_port) => {
                    match maybe_port {
                        Ok(port_number) => {
                            comm_with_frontend.send(BackendResponse::LocalWebServerStarted(Some(port_number))).ok();
                            println!("Frontend-local webserver has started");
                        },
                        Err(_) => {
                            comm_with_frontend.send(BackendResponse::LocalWebServerStarted(None)).ok();
                            println!("Frontend-local webserver has failed to start");
                        }
                    }
                },
                BackendRequest::InformContainerWebserverStarted(maybe_port) => {
                    match maybe_port {
                        Ok(port_number) => {
                            comm_with_frontend.send(BackendResponse::ContainerWebServerStarted(Some(port_number))).ok();
                            println!("Frontend-local webserver has started");
                        },
                        Err(_) => {
                            comm_with_frontend.send(BackendResponse::ContainerWebServerStarted(None)).ok();
                            println!("Frontend-local webserver has failed to start");
                        }
                    }
                },
                BackendRequest::AcceptConfiguration(container, config_str) => {
                    println!("Received Config: {}", config_str);

                    comm_with_frontend.send(BackendResponse::Configuration(container, config_str)).ok();
                    println!("Configuration has been sent to frontend");

                },
                BackendRequest::SendConfiguration() => {
                    println!("Not implemented: SendConfiguration");
                },
                BackendRequest::ExportAnalysisIntoRepository(jid, workdir, container, config_file) => {

                    match (container.components().last(), config_file.components().last()) {
                        (Some(container_name), Some(_config_file_name)) => {
                            comm_with_frontend.send(BackendResponse::JobInfo(jid, JobState::Running)).ok();
                            let mut container_target = workdir.clone();
                            container_target.push(container_name);
                            println!("Copying container: {:?} => {:?}", &container, &container_target);
                            std::fs::copy(&container, &container_target).ok();

                            match backend::copy_config_and_file_references(&workdir, &config_file) {
                                Ok(_) => {
                                    println!("Copy Job succeded!");
                                    comm_with_frontend.send(BackendResponse::JobInfo(jid, JobState::Completed(ExitStatus::from_raw(0)))).ok();
                                },
                                Err(e) => {
                                    println!("Copy Job failed! Reason: {:?}", e);
                                    comm_with_frontend.send(BackendResponse::JobInfo(jid, JobState::Completed(ExitStatus::from_raw(1)))).ok();
                                }
                            }
                        },
                        _ => {
                            //TODO: BackendResponse contains Result
                        }
                    };

                },
                BackendRequest::StartJob(containerpth, config) => {
                    let job_id = JobId::new();
                    let output_collector = process_outputs
                                                    .entry(job_id.clone())
                                                    .or_insert_with(|| Arc::new(Mutex::new(Vec::new())));

                    match singularity_run(&containerpth, vec![config]) {
                        Ok(mut child) => {
                            let child_stdout = child.stdout.take().unwrap();
                            backend::bufread_child_stdout_into_messages(output_collector, child_stdout).await;
                            process_store.insert(job_id.clone(), Arc::new(Mutex::new(child)));
                        },
                        Err(_) => {}
                    }
                    comm_with_frontend.send(BackendResponse::JobInfo(job_id, JobState::Running)).ok();
                },
                BackendRequest::SendJobInfo(job_id) => {
                    let process_entry = process_store.get(&job_id);

                    if let Some(process) = process_entry {
                        match process.lock() {
                            Ok(mut proc) => {
                                match proc.try_wait() {
                                    Ok(Some(status)) => {
                                        comm_with_frontend.send(BackendResponse::JobInfo(job_id, JobState::Completed(status))).ok();
                                    },
                                    Ok(None) => {
                                        comm_with_frontend.send(BackendResponse::JobInfo(job_id, JobState::Running)).ok();
                                    }
                                    Err(_) => {
                                        comm_with_frontend.send(BackendResponse::JobInfo(job_id, JobState::Unknown)).ok();
                                    },
                                }
                            },
                            Err(_) => {
                                //Is this a definite failure or is this an actual JobState::Unknown?
                                comm_with_frontend.send(BackendResponse::JobInfo(job_id, JobState::Unknown)).ok();
                            }
                        }
                    } else {
                        comm_with_frontend.send(BackendResponse::JobInfo(job_id, JobState::JobNotListed)).ok();
                    }
                },
                BackendRequest::SendJobOutput(job_id, offset) => {
                    // println!("All process outputs:");
                    //
                    // for (k,v) in process_outputs.iter() {
                    //     let val_str = match v.lock() {
                    //         Ok(mtx) => {
                    //             format!("[{:?}]", mtx.join(", "))
                    //         },
                    //         Err(_) => String::new()
                    //     };
                    //     println!("\t{:?} : {:?}", k, val_str)
                    // }

                    let output = process_outputs.get(&job_id);
                    match output {
                        Some(output) => {
                            match output.lock() {
                                Ok(lock) => {
                                    //println!("Backend sending slice from output of length: {}", lock.len());
                                    let lines = lock.iter().skip(offset).map(|s| s.clone()).collect();
                                    comm_with_frontend.send(BackendResponse::JobOutput(job_id, lines)).ok();
                                },
                                Err(_) => {}
                            }
                        },
                        None => {
                            comm_with_frontend.send(BackendResponse::JobOutput(job_id, vec!["No Output".to_string()])).ok();
                        }
                    }
                },
                BackendRequest::StopProcess(jid) => {
                    println!("Backend stops process with JobId {:?}", &jid);
                    process_store.store.entry(jid.clone())
                                        .and_modify(|arc| { arc.lock().unwrap().kill().ok(); });

                    //TODO: store address of server properly (per job_id) or create yet another BackendRequest
                    let _resp = reqwest::get("http://127.0.0.1:9283/terminate")
                                .await;

                    //process_store.store.entry(jid)
//                                       .and_modify(|arc| {
//                            match arc.lock() {
//                                Ok(mtx) => {
//                                    if mtx.stdin.is_some() {
//                                        mtx.stdin.unwrap().write()
//                                    } else {
//
//                                    }
//                                },
//                                Err(_) => {
//
//                                }
//                            }
//                    });

                    comm_with_frontend.send(BackendResponse::StoppedProcess(jid)).ok();
                },
                BackendRequest::StopAllProcesses => {

                    process_store.store.values_mut().for_each(|arc| {
                        if let Ok(mut child) = arc.lock() { child.kill().ok(); }
                    });

                    //TODO: store address of server properly (per job_id) or create yet another BackendRequest
                    let _resp = reqwest::get("http://127.0.0.1:9283/terminate")
                                .await;

                    comm_with_frontend.send(BackendResponse::StoppedAllProcesses).ok();
                },
                BackendRequest::StopProgram => {
                    match backend::shut_down_colonywsl_child() {
                        Ok(mut child) => {
                            let begin = tokio::time::Instant::now();
                            loop {
                                let now = tokio::time::Instant::now();
                                let dt = std::time::Duration::from_millis(100);
                                tokio::time::sleep_until(now + dt).await;
                                match child.try_wait() {
                                    Ok(Some(_status)) => { break; }
                                    _ => {
                                        if begin.elapsed() > std::time::Duration::from_millis(1000) {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        Err(_) => { }
                    }
                    std::process::exit(0);
                },
                BackendRequest::InspectFilesystem(_fs_type, _path) => {
                    todo!()
                },
                BackendRequest::ListDirectory(fs, path) => {
                    println!("Backend tasked with: Listing directory {:?}", &path);
                    let result = match fs {
                        FilesystemData::Local => {
                            list_dir_local_fs(&fs, path )
                        },
                        FilesystemData::LocalWSL => {
                            list_dir_local_wsl_fs(&fs, path)
                        },
                        //TODO: actually communicate with helix (over async-ssh2-challenge)
                        FilesystemData::HelixSSH(_) => {
                            todo!()
                        },
                        _ => todo!(),
                    };
                    comm_with_frontend.send(BackendResponse::ListDirectory(result)).ok();
                }
                //_ => println!("Backend tasked with: some task")
            },
            Err(_) => {
                println!("Communication with the backend has been closed.");
                break;
            }
        }
    }
}



fn list_dir_local_fs(_fs: &FilesystemData, path: PathBuf) -> Result<DirectoryContents, Result<DirectoryContents, ()>> {
    let mut files = Vec::new() as Vec<String>;
    let mut directories = Vec::new() as Vec<String>;

    if let Ok(dir_entries) = std::fs::read_dir(&path) {
        for node in dir_entries {
            if let Ok(entry) = node {
                if let Ok(ftype) = entry.file_type() {
                    if ftype.is_dir() { directories.push(entry.file_name().to_string_lossy().to_string()) }
                    else if ftype.is_symlink() {
                        if ftype.is_symlink_dir() { directories.push(entry.file_name().to_string_lossy().to_string()) }
                        else { files.push(entry.file_name().to_string_lossy().to_string()) }
                    }
                    else { files.push(entry.file_name().to_string_lossy().to_string()) }
                } else { files.push(entry.file_name().to_string_lossy().to_string()) }
            }
        }
        Ok(DirectoryContents { root: path, files, directories })
    } else {
        println!(r#"Directory {:?} could not be read"#, &path);
        Err(Err(()))
    }
}

fn list_dir_local_wsl_fs(_fs: &FilesystemData, path: PathBuf) -> Result<DirectoryContents, Result<DirectoryContents, ()>> {
    //TODO: directory names may contain malicious code

    let pathnames = backend::run_wsl_command(&format!("ls -1Q {}", linux_path_display(&path)))
        .map_err(|_| ())
        .and_then(|child| { child.wait_with_output().map_err(|_| ()) })
        .and_then(|out| String::from_utf8(out.stdout).map_err(|_| ()))
        .map(|string| {
            println!("Received Command output: {}", &string);
            string.lines().map(|l| {
                l.strip_prefix("\"")
                    .and_then(|l2| l2.strip_suffix("\""))
                    .map(|s| s.to_string())
                    .unwrap_or_else(String::new)
            }).collect_vec()
        });
    let pathmeta = backend::run_wsl_command(&format!("ls -1lQocF {}", linux_path_display(&path)))
        .map_err(|_| ())
        .and_then(|child| { child.wait_with_output().map_err(|_| ()) })
        .and_then(|out| String::from_utf8(out.stdout).map_err(|_| ()))
        .map(|string| {
            println!("Received Command output: {}", &string);
            string.lines().map(String::from).collect_vec()
        });

    let matches = match (pathnames, pathmeta) {
        (Err(_), _) => { println!("Path names could not be requested"); Err(Err(())) },
        (_, Err(_)) => { println!("Path metadata could not be requested"); Err(Err(())) },
        (Ok(names), Ok(meta)) => {
                if names.len()+1 == meta.len() {
                let mut files = Vec::new() as Vec<String>;
                let mut directories = Vec::new() as Vec<String>;

                println!("Fitting paths to metadata...");

                let mut k: usize = 0;
                let error = loop {
                    if k >= names.len() {
                        println!("Completed fitting paths to metadata");
                        break Ok((files, directories));
                    }

                    let nm = &names[k];
                    let mta = &meta[k+1]; // first line represents summary statistics
                    let arrow_pattern = Regex::new("->").unwrap();
                    if arrow_pattern.find_iter(mta).count() > arrow_pattern.find_iter(nm).count() {
                        // this is a symlink and ls has indicated that with an arrow
                        // the resolved path may also contain an arrow
                        let symlink_pattern = Regex::new(&format!(r#"(?s)"{}" -> "(.*)"(=>|[\*/@|]?)"#, regex::escape(nm))).unwrap();

                        let info = symlink_pattern.captures(mta);

                        match info {
                            Some(mtch) => {
                                let _resolved_path = mtch.get(1);
                                let is_dir = mtch.get(2).map_or(false, |grp| grp.as_str() == "/");
                                if is_dir { directories.push(format!("{}", nm)) }
                                else { files.push(nm.clone()) }
                            },
                            None => {
                                println!("Could not parse metadata for symlink {:?} => {:?}", nm, mta);
                                break Err(Err(()))
                            }
                        }

                    } else {
                        // arrows cannot appear if not in the filename itself
                        let basic_pattern = Regex::new(&format!(r#"(?s)"{}"(=>|[\*/@|]?)"#, regex::escape(&nm))).unwrap();

                        let info = basic_pattern.captures(&mta);

                        match info {
                            Some(mtch) => {
                                let is_dir = mtch.get(1).map_or(false, |grp| grp.as_str() == "/");
                                if is_dir { directories.push(format!("{}", &nm)) }
                                else { files.push(nm.clone()) }
                            },
                            None => {
                                println!("Could not parse metadata for file/directory {:?} => {:?}", nm, mta);
                                break Err(Err(()))
                            }
                        }
                    }

                    k += 1;
                };

                error.map(|(files, directories)| DirectoryContents { root: path.clone(), files, directories } )
            } else if names.len() == 0 && meta.len() == 0 {
                println!("Outputs for names and metadata is zero, Directory {:?} probably does not exist", &path);
                Err(Err(()))
            } else {
                println!("Outputs for names and metadata do not fit (must differ by one), names: {}, meta: {}", names.len(), meta.len());
                Err(Err(()))
            }
        }
    };
    matches
}












