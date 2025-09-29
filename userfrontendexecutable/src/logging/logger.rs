


use std::fs::File;
use tracing::{info, error};
use tracing_subscriber::{fmt, prelude::*};
use tracing_appender::non_blocking;

/// Initialize global logging: stdout + global.log
pub fn init_global(global_logfile: &str) {
    // stdout layer
    let stdout_layer = fmt::layer()
    .with_writer(std::io::stdout)
    .with_target(true);

    // global file layer
    let file = File::create(global_logfile).expect("Failed to create global.log");
    let (file_writer, _guard) = non_blocking(file);
    let file_layer = fmt::layer()
    .with_writer(file_writer)
    .with_target(true);

    // Build and set global subscriber
    tracing_subscriber::registry()
    .with(stdout_layer)
    .with(file_layer)
    .init();
}

/// Create a per-job logger that writes to a job-specific logfile
pub struct JobLogger {
    pub job_id: usize,
    _guard: tracing_appender::non_blocking::WorkerGuard,
}

impl JobLogger {
    pub fn new(job_id: usize, logfile: &str) -> Self {
        let file = File::create(logfile).expect("Failed to create job log file");
        let (file_writer, guard) = non_blocking(file);

        // File layer for this job, filters events by job_id field
        let job_layer = fmt::layer()
        .with_writer(file_writer)
        .with_target(true)
        .with_filter(tracing_subscriber::filter::filter_fn(move |meta| {
            meta.fields()
            .field("job_id")
            .map_or(false, |f| f.to_string() == job_id.to_string())
        }));

        // Attach the job layer to the global subscriber
        tracing_subscriber::registry().with(job_layer);

        JobLogger {
            job_id,
            _guard: guard,
        }
    }

    pub fn info(&self, msg: &str) {
        tracing::info!(target = "job", job_id = self.job_id, "{}", msg);
    }

    pub fn error(&self, msg: &str) {
        tracing::error!(target = "job", job_id = self.job_id, "{}", msg);
    }
}




