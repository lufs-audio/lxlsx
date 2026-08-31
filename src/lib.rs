//! lxlsx — a contract-verified, agent-first process registry for Excel workbooks.

pub mod error;
pub mod json;
pub mod reader;
pub mod registry;
pub mod render;
pub mod snapshot;
pub mod verify;

pub use error::Error;
pub use snapshot::Snapshot;
