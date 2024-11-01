#[allow(warnings)]
mod bindings;
mod clock;
mod bit_cask_key;
mod config;
mod merge_config;
mod field_generator;
mod entry;
mod segment;
mod store;
mod errors;

use bindings::Guest;

struct Component;

impl Guest for Component {
    /// Say hello!
    fn hello_world() -> String {
        "Hello, World!".to_string()
    }
}

bindings::export!(Component with_types_in bindings);
