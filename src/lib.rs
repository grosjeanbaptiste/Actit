pub mod args;
pub mod contains;
pub mod error;
pub mod github;
pub mod indexer;
pub mod kag;
pub mod model;
pub mod name;
pub mod need;
pub mod ports;
pub mod process;
pub mod report;
pub mod search;
pub mod settings;

/// Compiles the README examples so the documented API cannot drift from the code.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
