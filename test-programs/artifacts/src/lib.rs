#[allow(warnings)]
mod bindings;

use bindings::Guest;

use bindings::component::store::types::{KeyValuePair, Store};

struct Component;

impl Guest for Component {
    /// Say hello!
    fn test_store()  {
        let new_store = Store::new();
        new_store.insert(&KeyValuePair{ key: "hello".to_owned(), value: "world".to_owned()}).unwrap();
        assert_eq!(new_store.search(&"hello".to_owned()).unwrap().value, "world".to_owned())
    }
}

bindings::export!(Component with_types_in bindings);
