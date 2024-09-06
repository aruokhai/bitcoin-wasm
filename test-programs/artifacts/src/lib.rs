#[allow(warnings)]
mod bindings;

use bindings::Guest;

use node_test::test_node;
use store_test::test_store;
mod store_test;
mod node_test;

struct Component;

impl Guest for Component {
    /// Say hello!
    fn test()  {
        test_store();
        test_node();
    }

}

bindings::export!(Component with_types_in bindings);
