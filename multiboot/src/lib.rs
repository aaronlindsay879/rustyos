//! Library for constructing a multiboot2 header, and parsing the returned data structure

#![no_std]
#![feature(const_trait_impl, const_slice_make_iter, used_with_arg)]
#![warn(missing_docs, clippy::missing_docs_in_private_items)]

pub mod header;
pub mod prelude;
