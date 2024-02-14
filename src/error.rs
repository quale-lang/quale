//! This file contains all error types for qcc.
use std::fmt::{Debug, Display};

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(crate) trait Error: Debug + Display {}

pub fn eprintln<T>(result: Result<T>) {
    match result {
        Ok(_) => unreachable!(),
        Err(e) => eprintln!("qcc: {e}"),
    }
}
