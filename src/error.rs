//! Error type and exit-code mapping.

/// Exit codes (SPEC.md). The fixed floor `0/2/5` plus domain codes `3/4`.
pub mod exit {
    pub const SUCCESS: i32 = 0;
    pub const RESERVED: i32 = 1;
    pub const USAGE: i32 = 2;
    pub const DB_UNAVAILABLE: i32 = 3;
    pub const PROCESS_NOT_FOUND: i32 = 4;
    pub const CONTRACT_VIOLATED: i32 = 5;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database unavailable: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("xlsx write error: {0}")]
    Xlsx(#[from] rust_xlsxwriter::XlsxError),
    #[error("xlsx read/verify error: {0}")]
    Verify(#[from] calamine::Error),
    #[error("process not found: {0}")]
    ProcessNotFound(String),
    #[error("contract violated: {0}")]
    Contract(String),
    #[error("invalid usage: {0}")]
    Usage(String),
}

impl Error {
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::Db(_) | Error::Io(_) => exit::DB_UNAVAILABLE,
            Error::Xlsx(_) | Error::Verify(_) | Error::Contract(_) => exit::CONTRACT_VIOLATED,
            Error::ProcessNotFound(_) => exit::PROCESS_NOT_FOUND,
            Error::Usage(_) => exit::USAGE,
        }
    }
}
