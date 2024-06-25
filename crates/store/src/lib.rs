#[allow(warnings)]
mod bindings;

use bindings::Guest;

struct Component;

mod node_type;
mod page_layout;
mod page;
mod node;
mod error;
mod btree;

impl Guest for Component {
    /// Say hello!
    fn hello_world() -> String {
        "Hello, World!".to_string()
    }
}

bindings::export!(Component with_types_in bindings);
