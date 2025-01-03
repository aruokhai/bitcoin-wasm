#[allow(warnings)]
mod bindings;

use bindings::Guest;


mod coin_selection;
mod utils;
mod types;
mod errors;

mod tx_builder;
mod wallet;


struct Component;

impl Guest for Component {
    /// Say hello!
    fn hello_world() -> String {
        "Hello, World!".to_string()
    }
}

bindings::export!(Component with_types_in bindings);
