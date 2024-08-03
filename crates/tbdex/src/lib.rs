#[allow(warnings)]
mod bindings;

use bindings::Guest;

mod web5;
mod request;
mod json;
struct Component;

impl Guest for Component {
    /// Say hello!
    fn hello_world() -> String {
        "Hello, World!".to_string()
    }
}

bindings::export!(Component with_types_in bindings);
