/*!
# Theseus

Theseus is a library which provides utilities for launching minecraft, creating Modrinth mod packs,
and launching Modrinth mod packs
*/
#![warn(unused_import_braces)]
#![deny(unused_must_use)]

#[macro_use]
pub mod util; // [AR] Refactor

mod api;
mod error;
mod event;
mod launcher;
mod logger;
mod state;

pub use api::*;
pub use error::*;
pub use event::{
    EventState, LoadingBar, LoadingBarType, emit::cancel_loading_bar,
    emit::emit_loading, emit::init_loading,
};
pub use logger::start_logger;
pub use state::DirectoryInfo;
pub use state::State;
pub use state::db::repair_migration_state_from_disk;
pub use state::set_rpc_language;

pub const LAUNCHER_USER_AGENT: &str = concat!(
    "modrinth/theseus/",
    env!("CARGO_PKG_VERSION"),
    " (support@modrinth.com)"
);
