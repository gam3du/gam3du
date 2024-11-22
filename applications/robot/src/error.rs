use std::{
    fmt::{self, Display},
    process::ExitCode,
};

use winit::error::EventLoopError;

// pub(crate) type ApplicationResult<T> = Result<T, ApplicationError>;

#[derive(Debug)]
pub(crate) enum ApplicationError {
    #[allow(
        unused,
        reason = "This is a placeholder for errors that still need to be categorized"
    )]
    Todo(String),
    // BuildRuntime(std::io::Error),
    EventLoop(EventLoopError),
}

impl Display for ApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApplicationError::Todo(message) => write!(formatter, "other error: {message}"),
            // ApplicationError::BuildRuntime(error) => {
            //     write!(formatter, "failed to build async runtime: {error}")
            // }
            ApplicationError::EventLoop(message) => {
                write!(formatter, "event loop error: {message}")
            }
        }
    }
}

impl From<ApplicationError> for ExitCode {
    fn from(value: ApplicationError) -> Self {
        match value {
            ApplicationError::Todo(_) => ExitCode::FAILURE,
            // ApplicationError::BuildRuntime(_) => ExitCode::from(2),
            ApplicationError::EventLoop(_) => ExitCode::from(3),
        }
    }
}

#[cfg(target_family = "wasm")]
impl From<ApplicationError> for wasm_bindgen::JsValue {
    fn from(value: ApplicationError) -> Self {
        value.to_string().into()
    }
}

impl From<EventLoopError> for ApplicationError {
    fn from(value: EventLoopError) -> Self {
        Self::EventLoop(value)
    }
}
