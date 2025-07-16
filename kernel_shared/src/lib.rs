//! Shared code for both kernel and kernel loader.

#![no_std]
#![warn(missing_docs, clippy::missing_docs_in_private_items)]
#![feature(step_trait)]
#![feature(let_chains)]
#![feature(ptr_metadata)]
#![feature(iter_intersperse)]
#![feature(abi_x86_interrupt)]

pub mod io;
pub mod logger;
pub mod mem;
pub mod x86;

/// Size of kernel heap in bytes
pub const HEAP_SIZE: usize = 128 * 1024; // 128 KiB

/// Size of kernel stack in bytes
pub const STACK_SIZE: usize = 128 * 1024; // 128 KiB
