#![allow(dead_code)] // Scaffolded code - types defined but not yet wired up

pub mod config;
pub mod domain;
pub mod error;
pub mod ports;
pub mod services;

pub use error::{HelixDocsError, Result};
