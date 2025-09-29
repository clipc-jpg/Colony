

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::process::Child;
use std::time::SystemTime;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use super::HardTypedDBAccess;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId {
    pub order_time: UnixTime,
    pub uuidv4: Uuid,
}

impl JobId {
    pub fn new() -> Self {
        return Self { order_time: UnixTime::now(), uuidv4: Uuid::new_v4() };
    }
}

impl Default for JobId {
    fn default() -> Self { return Self::new(); }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnixTime {
    pub seconds: u64
}

impl UnixTime {
    pub fn from(t: SystemTime) -> UnixTime {
        return UnixTime { seconds: t.duration_since(SystemTime::UNIX_EPOCH).expect("System Time appears to be before 1970").as_secs() };
    }

    pub fn now() -> UnixTime {
        return UnixTime::from(SystemTime::now());
    }
}




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


//################################################################################
//## Running jobs
//################################################################################

pub fn run_singularity_job() {

    let child_result = singularity_run_in_dir(&workdir, &container_path, container_args);

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

            todo!("run_singularity_job may need to report its completion to the runtime")

            //comm_with_frontend.send(BackendResponse::JobInfo(job_id, JobState::Running)).ok();
        },
        Err(e) => {
            println!("Error starting container: {:?}", &e);
        }
    }
}

//################################################################################
//## Background runtime
//################################################################################





