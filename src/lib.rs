pub mod error;
pub mod indexer;
pub mod kag;
pub mod model;
pub mod need;
pub mod ports;
pub mod process;
pub mod report;
pub mod search;

/// Compiles the README examples so the documented API cannot drift from the code.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
