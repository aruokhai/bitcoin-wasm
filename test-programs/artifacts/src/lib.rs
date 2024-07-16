#[allow(warnings)]
mod bindings;

use bindings::Guest;

use store_test::{test_delete, test_insert};
mod store_test;

struct Component;

impl Guest for Component {
    /// Say hello!
    fn test_store()  {
        test_insert();
        test_delete();
        
    }
}

bindings::export!(Component with_types_in bindings);
