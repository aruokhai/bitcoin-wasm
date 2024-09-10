use bindings::component::tbdex::types::{Client, Error as TBdexError, };
use crate::bindings;


pub fn test_tbdex(){
    println!("here gotten here");
    let pfi_url = "did:dht:zkp5gbsqgzn69b3y5dtt5nnpjtdq6sxyukpzo68npsf79bmtb9zy";
    let client = Client::new(pfi_url, "mock-idv.tbddev.org", "1234567");
    let conversion_offer = client.get_offer().unwrap();
    let coversion_res = client.convert(&conversion_offer.id, "1000", "12345678").unwrap();
    println!("conversion offer {}", coversion_res);


}