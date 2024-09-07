
use bindings::component::store::types::{KeyValuePair, Store, Error};

use crate::bindings;

pub fn test_store(){
    test_insert();
    test_delete();
} 


 fn test_insert() {
    let mut key_value:  Vec<(String,String)> = vec![];
    for i in 1..20 {
        let k = i * 3;
        key_value.push((i.to_string(), k.to_string()))
    }

    let new_store = Store::new();
    for (key,value) in key_value.iter() {
        new_store.insert(&KeyValuePair{ key: key.to_owned(), value: value.to_owned()}).unwrap();
    }
    for (key,value) in key_value.iter() {
        assert_eq!(new_store.search(key).unwrap().value, value.to_owned());
        println!("working motherfucker")
    }
}

 fn test_delete() {
    let mut key_value:  Vec<(String,String)> = vec![];
    for i in 1..20 {
        let k = i * 3;
        key_value.push((i.to_string(), k.to_string()))
    }

    let new_store = Store::new();
    for (key,value) in key_value.iter() {
        new_store.insert(&KeyValuePair{ key: key.to_owned(), value: value.to_owned()}).unwrap();
    }
    for (key,value) in key_value.iter() {
        new_store.delete(key).unwrap();
        assert!(matches!(new_store.search(key), Err(_)))
    }
}

