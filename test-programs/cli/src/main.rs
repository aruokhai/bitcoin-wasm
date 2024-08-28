#[allow(warnings)]
mod bindings;

use bindings::component::artifacts::test_store;


fn main() {
    let tester = test_store();
}
