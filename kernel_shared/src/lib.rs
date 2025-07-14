//! Shared code for both kernel and kernel loader.

#![no_std]
#![warn(missing_docs, clippy::missing_docs_in_private_items)]
#![feature(step_trait)]
#![feature(let_chains)]
#![feature(ptr_metadata)]

pub mod io;
pub mod logger;
pub mod mem;
