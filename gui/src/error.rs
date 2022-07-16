#[derive(thiserror::Error, Debug)]
pub enum WasmWorkerError {
    #[error("Unable to start worker, as worker is already running.")]
    AlreadyActive,
    #[error("Unable to start worker, unclear why")]
    DoesNotStart,
}
