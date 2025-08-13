pub mod client;
pub mod errors;
pub mod logs;
pub mod resources;
pub mod watch;
pub mod resource_map;
pub mod system_monitor;
pub mod resource_api;
pub mod shared_cache;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod serialization_test;

#[cfg(test)]
mod tauri_ipc_test;

pub use client::{K8sClient, K8sContext};
pub use errors::{K8sWatchError, K8sWatchResult};
pub use logs::LogStreamManager;
pub use resources::*;
pub use watch::*;
pub use resource_map::*;
pub use system_monitor::*;
pub use resource_api::*;
pub use shared_cache::*;