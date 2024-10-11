#![no_std]
#![cfg_attr(not(test), no_main)]

#[cfg(test)]
extern crate alloc;

pub mod entry;
pub mod error;

#[cfg(not(test))]
use ckb_std::default_alloc;
#[cfg(not(test))]
ckb_std::entry!(program_entry);
#[cfg(not(test))]
default_alloc!();

pub fn program_entry() -> i8 {
    match entry::entry() {
        Ok(_) => 0,
        Err(e) => match e {
            error::Error::CkbSysError(_) => 1,
            _ => 100,
        },
    }
}
