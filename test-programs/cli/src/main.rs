#[allow(warnings)]
mod bindings;

use bindings::component::store::types::{KeyValuePair, Store};


fn main() {
    let new_store = Store::new();
        new_store.insert(&KeyValuePair{ key: "hello".to_owned(), value: "world".to_owned()}).unwrap();
        println!("{:?}",new_store.search(&"hello".to_owned()).unwrap());
        assert_eq!(new_store.search(&"hello".to_owned()).unwrap().value, "world".to_owned())
}
