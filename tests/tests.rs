#![allow(unused)]
use derivable_object_pool::prelude::*;

#[derive(Default, ObjectPool)]
struct Test {
    a: u32,
    b: u32,
}

#[test]
fn new_objects() {
    let obj = Test::new();
    drop(obj);
    assert_eq!(1, Test::pool().len());

    let obj = Test::new();
    assert_eq!(0, Test::pool().len());
    let obj2 = Test::new();

    drop(obj);
    drop(obj2);

    assert_eq!(2, Test::pool().len());
}
