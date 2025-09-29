



use std::path::PathBuf;

use uuid::Uuid;
use rusqlite::{Connection, Result, params};
use serde_json;
use serde::{Serialize, Deserialize};


use plugin_interface_elements::{elements_v1, TargetSystem, VersionedRequest, VersionedResponse};
use crate::server::exe_dir;


#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct HardTypedDBAccess {
    conn: Connection,
}

// How does versioning come into play?
impl HardTypedDBAccess {

    pub fn new() -> Self {
        let mut dbpath = exe_dir().clone().unwrap_or_else(|| &PathBuf::from("~"));
        dbpath.push("central-server_message_db.sqlite");
        let conn = Connection::open(dbpath);

        return Self { conn };
    }


    //################################################################################
    //## Checking Database integrity
    //################################################################################

    fn check_database_schema(&mut self) -> Result<()> {

        // squlite_schema schema:
        // CREATE TABLE sqlite_schema(
        //        type text,        # table, view, trigger, index,---
        //        name text,        # sqlite-internal name
        //        tbl_name text,    # for tables, the identifier used in queries
        //        rootpage integer, # internal index for a btree
        //        sql text          # (modified) original statement that created the table
        //    );

        let mut tbl_query = conn.prepare("SELECT type, name, tbl_name, sql from sqlite_schema")?;

        let tbls = stmt.query_map([], |row| {
            Ok(SqliteTable {
                id: row.get(0)?,
                name: row.get(1)?,
                tbl_name: row.get(2)?,
                schema: row.get(3)?
            })
        })?;

        todo!();
        // validate that all necessary tables exist (and possibly, that the schema is correct)
    }




    fn create_database(&mut self) -> Result<()> {

        // connection
        self.conn.execute(
            "CREATE TABLE requests (
                request_id TEXT PRIMARY KEY,
                api_version INTEGER NOT NULL,
                origin TEXT NOT NULL,
                destination TEXT NOT NULL,
                payload TEXT,
                start_time INTEGER NOT NULL,
                end_time INTEGER
            )", () // empty list of parameters.
        )?;

        self.conn.execute(
            "CREATE TABLE logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                request_id TEXT REFERENCES requests(id),
                     stream_type TEXT,                           -- stdout, stderr, log, event
                     time_stamp INTEGER,
                     raw_line TEXT,
                     json_payload TEXT,
                     FOREIGN KEY(process_id) REFERENCES process(id)
            )", ()
        )?;

        //
        self.conn.execute(
            "CREATE TABLE servers (
                server_id TEXT PRIMARY KEY,
                server_capabilities TEXT,
            )", ()
        )?;

        self.conn.execute(
            "CREATE TABLE containers (
                container_id TEXT PRIMARY KEY,
                container_path TEXT NOT NULL,
                container_apps TEXT,
            )", ()
        )?;

        // let me = Person {
        //     id: 0,
        //     name: "Steven".to_string(),
        //     data: None,
        // };
        // conn.execute(
        //     "INSERT INTO person (name, data) VALUES (?1, ?2)",
        //              (&me.name, &me.data),
        // )?;
        //
        // let mut stmt = conn.prepare("SELECT id, name, data FROM person")?;
        // let person_iter = stmt.query_map([], |row| {
        //     Ok(Person {
        //         id: row.get(0)?,
        //        name: row.get(1)?,
        //        data: row.get(2)?,
        //     })
        // })?;
        //
        // for person in person_iter {
        //     println!("Found person {:?}", person.unwrap());
        // }
        Ok(())
    }










    //################################################################################
    //## Requests table interactions
    //################################################################################

    pub fn register_plugintask_request(&mut self, request: &VersionedRequest) {
        match request {
            &VersionedRequest::ApiV1(ref plugin, ref requ_id, ref task_request) => {
                self.register_plugintask_request_v1(&plugin, &requ_id, &task_request);
            }
        }
    }

    fn register_plugintask_request_v1(&mut self,
                                      target_plugin: &TargetSystem,
                                      request_id: &elements_v1::RequestId,
                                      plugintask_request: &elements_v1::PluginTaskRequest) {

        let requ_id = serde_json::to_string(request_id);
        let origin = serde_json::to_string(TargetSystem::Frontend);
        let destination = serde_json::to_string(target_plugin);
        let payload = serde_json::to_string(plugintask_request);
        let start_time = crate::server::UnixTime::now().as_secs();

        self.conn.execute(
            "INSERT INTO requests (request_id, origin, destination, payload, start_time) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                     params![&requ_id, 1, &origin, &destination, &payload, &start_time],
        )?;
    }

    fn register_plugintask_response(&mut self, response: &VersionedResponse) {
        match response {
            VersionedResponse::ApiV1(ref targ_plugin, ref request_id, ref plugintask_response) => {
                register_plugintask_response_v1(self, &targ_plugin, &request_id, &plugintask_response);
            }
        }
    }

    fn register_plugintask_response_v1(&mut self,
                                      target_plugin: &TargetSystem,
                                      request_id: &elements_v1::RequestId,
                                      plugintask_request: &elements_v1::PluginTaskRequest) {

        let requ_id = serde_json::to_string(request_id.inner);
        let origin = serde_json::to_string(target_plugin);
        let destination = serde_json::to_string(TargetSystem::Frontend);
        let payload = serde_json::to_string(plugintask_request);
        let now = crate::server::UnixTime::now().as_secs();

        self.conn.execute(
            "INSERT INTO requests (request_id, api_version, origin, destination, payload, start_time, end_time) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                     params![&requ_id, 1, &origin, &destination, &payload, &now, &now],
        )?;

        //mark_plugintask_request_resolved_v1(&self, &request_id);
        self.conn.execute(
            "UPDATE requests SET end_time = (?1) WHERE request_id = (?2)", (&now, &requ_id)
        )?;
    }

    // Should this be handled internally?
    //pub fn mark_plugintask_requests_resolved(&mut self) {
    //
    //
    //}

    pub fn mark_request_resolved_v1(&mut self, request_id: &elements_v1::RequestId) {

        let requ_id = serde_json::to_string(request_id.inner);
        let now = crate::server::UnixTime::now().as_secs();

        self.conn.execute(
            "UPDATE requests SET end_time = (?1) WHERE request_id = (?2)", (&now, &requ_id)
        )?;
    }


    pub fn register_frontendtask_request(&mut self, request: VersionedRequest) {
        match request {
            &VersionedResponse::ApiV1(ref plugin, ref requ_id, ref task_request) => {
                self.register_plugintask_request_v1(&plugin, &requ_id, &task_request);
            }
        }
    }

    pub fn register_frontendtask_request_v1(&mut self,
                                            target_plugin: &TargetSystem,
                                            request_id: &elements_v1::RequestId,
                                            plugintask_request: &elements_v1::FrontendTaskRequest) {

        let requ_id = serde_json::to_string(request_id);
        let origin = serde_json::to_string(target_plugin);
        let destination = serde_json::to_string(TargetSystem::Frontend);
        let payload = serde_json::to_string(plugintask_request);
        let start_time = crate::server::UnixTime::now().as_secs();

        self.conn.execute(
            "INSERT INTO requests (request_id, api_version, origin, destination, payload, start_time) VALUES (?1, ?2)",
                          params![&requ_id, 1, &origin, &destination, &payload, &start_time],
        )?;
    }

    pub fn register_frontendtask_request(&mut self, request: VersionedResponse) {
        match request {
            &VersionedResponse::ApiV1(ref plugin, ref requ_id, ref task_request) => {
                self.register_plugintask_request_v1(&plugin, &requ_id, &task_request);
            }
        }
    }

    pub fn register_frontendtask_response_v1(&mut self,
                                             target_plugin: &TargetSystem,
                                             request_id: &elements_v1::RequestId,
                                             plugintask_request: &elements_v1::FrontendTaskRequest) {

        let requ_id = serde_json::to_string(request_id);
        let origin = serde_json::to_string(TargetSystem::Frontend);
        let destination = serde_json::to_string(target_plugin);
        let payload = serde_json::to_string(plugintask_request);
        let start_time = crate::server::UnixTime::now().as_secs();

        self.conn.execute(
            "INSERT INTO requests (request_id, api_version, origin, destination, payload, start_time) VALUES (?1, ?2)",
                          params![&requ_id, 1, &origin, &destination, &payload, &start_time],
        )?;
    }

    //################################################################################
    //## Logs table interactions
    //################################################################################

    pub fn append_logs() {

    }

    pub fn read_logs() {

    }



    //################################################################################
    //## Removing old data
    //################################################################################

    // separate methods for logs ?
    pub fn prune_archived_requests() {

    }


    //################################################################################
    //## Servers table interaction
    //################################################################################

    pub fn add_server_entry() {

    }

    pub fn remove_server_entry () {

    }





}













