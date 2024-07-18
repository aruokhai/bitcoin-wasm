#[allow(warnings)]
mod bindings;

use bindings::Guest;

struct Component;

mod p2p;
mod buffer;

impl Guest for Component {
    /// Say hello!
    fn hello_world() -> String {
        "Hello, World!".to_string()
    }
}

bindings::export!(Component with_types_in bindings);
