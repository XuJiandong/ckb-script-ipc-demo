#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
pub mod channel;
pub mod error;
pub mod io;
pub mod io_impl;
pub mod ipc;
pub mod packet;
pub mod pipe;
pub mod utils;
pub mod vlq;
