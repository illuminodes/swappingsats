#![warn(
    clippy::all,
    clippy::missing_errors_doc,
    clippy::style,
    clippy::unseparated_literal_suffix,
    clippy::pedantic,
    clippy::nursery
)]

mod bindings;
pub use bindings::*;

#[cfg(test)]
mod test_component;
