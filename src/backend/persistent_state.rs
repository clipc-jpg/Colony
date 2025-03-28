

use std::path::PathBuf;

use std::env;
use std::sync::OnceLock;
use std::fs;
use std::fs::File;
use std::io::BufReader;

use serde::{Deserialize, Serialize};
use uuid::Uuid;




//################################################################################
//## Persistent State
//## (will be saved regularly and loaded upon app start)
//################################################################################

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PersistentStateUpdate {
    //containers
    AddContainer(PathBuf),
    RemoveContainerByPath(PathBuf),
    RemoveContainer(Uuid),
    // last_selected_container_dir
    SetlastSelectedContainerDir(Option<PathBuf>),
    // protocols
    //AddRecentProtocol(PathBuf),
//    DeleteProtocol(PathBuf),
//    ArchiveProtocol(PathBuf),
//    UnarchiveProtocol(PathBuf)
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct BackendContainerDescription {
    pub id: Uuid,
    pub path: PathBuf,
}

impl PartialOrd for BackendContainerDescription {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        return self.path.partial_cmp(&other.path);
    }
}

impl Ord for BackendContainerDescription {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        return self.path.cmp(&other.path);
    }
}

impl BackendContainerDescription {
    pub fn from_path(pth: PathBuf) -> Self {
        return BackendContainerDescription { id: Uuid::new_v4(), path: pth };
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct PersistentStateDeserialzed {
    pub containers: Option<Vec<PathBuf>>,
    pub containersv2: Option<Vec<BackendContainerDescription>>,
    pub last_selected_container_dir: Option<PathBuf>,
    pub recent_protocols: Option<Vec<PathBuf>>,
    pub archived_protocols: Option<Vec<PathBuf>>
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct PersistentState {
    #[serde(rename = "containersv2")]
    pub containers: Vec<BackendContainerDescription>,
    pub last_selected_container_dir: Option<PathBuf>,
    pub recent_protocols: Vec<PathBuf>,
    pub archived_protocols: Vec<PathBuf>
}

impl From<PersistentStateDeserialzed> for PersistentState {
    fn from(other: PersistentStateDeserialzed) -> Self {

        let mut containers = other.containersv2.unwrap_or_default();
        if let Some(mut other_containers) = other.containers {
            other_containers.sort();
            other_containers.dedup();

            for cont_path in other_containers.into_iter() {
                let mut new_bcd = None as Option<BackendContainerDescription>;
                if !containers.iter().any(|bcd| bcd.path == cont_path) {
                    new_bcd = Some(BackendContainerDescription::from_path(cont_path));
                }
                if let Some(new_elem) = new_bcd {
                    containers.push(new_elem);
                }
            }
        }

        Self {
            containers,
            last_selected_container_dir: other.last_selected_container_dir,
            recent_protocols: other.recent_protocols.unwrap_or_default(),
            archived_protocols: other.archived_protocols.unwrap_or_default(),
        }
    }
}


impl PersistentState {
    pub fn from_file(path: &Option<PathBuf>) -> Self {
        return match path {
            Some(pathstr) => {
                match File::open(&pathstr) {
                    Ok(persistent_file) => {
                        let filereader = BufReader::new(persistent_file);
                        let state: PersistentStateDeserialzed = serde_json::from_reader(filereader)
                                    .expect(&format!("Mismatched File Format or Contents in: {:?}", &pathstr));

                        PersistentState::from(state)
                    }
                    Err(_e) => {
                        println!("Could not find configuration file at: {:?}. Starting with new configuration.", &pathstr);
                        PersistentState {
                            containers: Vec::new() as Vec<BackendContainerDescription>,
                            last_selected_container_dir: None,
                            recent_protocols: Vec::new(),
                            archived_protocols: Vec::new()
                        }
                    }
                }
            },
            None => {
                PersistentState {
                    containers: Vec::new() as Vec<BackendContainerDescription>,
                    last_selected_container_dir: None,
                    recent_protocols: Vec::new(),
                    archived_protocols: Vec::new()
                }
            }
        };
    }

    pub fn write_to_file(&self, pathstr: &PathBuf) {
        let json_result = serde_json::to_string_pretty(&self);

        match json_result {
            Ok(json) => {
                match fs::write(pathstr, json) {
                    Ok(_) => { println!("Persistent state has been saved to disk!") },
                    Err(e) => {
                        println!("Error overwriting config at {:?}. Error: {}", pathstr, e);
                    }
                }
            },
            Err(e) => {
                println!("Error overwriting config at {:?}. Error: {}", pathstr, e);
            }
        }
    }
}

static EXE_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();

pub fn exe_dir() -> &'static Option<PathBuf>{
    return EXE_DIR.get_or_init(|| {
        match env::current_exe() {
            Ok(ptb) => {
                match ptb.parent() {
                    Some(pth) => Some(PathBuf::from(pth)),
                    None => Some(ptb)
                }
            }
            Err(_e) => {
                match env::var("COLONY_CONFIGDIR") {
                    Ok(val) => {
                        let pth = PathBuf::from(val);
                        if pth.is_dir() {Some(pth)} else {
                            println!(r#"Environment variable "COLONY_CONFIGDIR" is not a valid directory. Cannot read/write persistent session data."#);
                            None
                        }
                    },
                    Err(_e) => {
                        println!(r#"Environment variable "COLONY_CONFIGDIR" has not been set. Cannot read/write persistent session data."#);
                        None
                    },
                }
            }
        }
    })
}

static CONFIG_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

pub fn config_path() -> &'static Option<PathBuf> {
    return CONFIG_PATH.get_or_init(|| {
        match exe_dir() {
            Some(ptb) => {
                Some(ptb.join("colony_launcher_configuration.json"))
            },
            None => None
        }
    })
}


















