# Derivable Object Pool

This crate provides a trait that can be derived to implement an object pool
for a type with a single line of code. Allowing the user to forget about
the implementation details of the [`ObjectPool`](https://docs.rs/derivable-object-pool/trait.ObjectPool.html)
and focus on the important parts of their code

This crate has the following features compared to other object pool crates:
- **Derivable**: The pool is simple to use and can be used with any type. Can 
be just derived using the [`#[derive(ObjectPool)]`](https://docs.rs/derivable_object_pool/derive.ObjectPool.html)
attribute macro.
- **Reusable**: The user can use the [`ObjectPool::new`](https://docs.rs/derivable-object-pool/trait.ObjectPool.html#method.new)
function to create objects from the pool, which will reuse objects from the pool
if possible. This items are wrapped in a [`Reusable`](https://docs.rs/derivable-object-pool/struct.Reusable.html)
struct, which will be returned to the pool when dropped.
- **Thread Safe**: The pool is thread-safe (through the use of a [`Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html))
and can be used in a multi-threaded environment.
- **Simple**: The user doesn't need to create a pool for each type manually
and can use the [`ObjectPool::new`](https://docs.rs/derivable-object-pool/trait.ObjectPool.html#method.new)
function to create objects from the pool.
- **Flexible**: The user can configure the pool to use a custom generator
function (see attributes in [`#[derive(ObjectPool)]`](https://docs.rs/derivable_object_pool/derive.ObjectPool.html))
or just use the [`Default`](https://doc.rust-lang.org/std/default/trait.Default.html)
trait to create new objects.

# Installation

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
derivable-object-pool = "0.1.0"
```

# Usage

Without specifying any attributes, the pool will use the [`Default`](https://doc.rust-lang.org/std/default/trait.Default.html) trait to create new objects:
```rust
use derivable_object_pool::prelude::*;

#[derive(Default, ObjectPool)]
struct Test(i32);

fn main() {
    let mut obj = Test::new();
    obj.0 += 1;
    assert_eq!(obj.0, 1);
    drop(obj); // obj is returned to the pool
    assert_eq!(Test::pool().len(), 1);
    let mut obj = Test::new();
    assert_eq!(Test::pool().len(), 0);
    assert_eq!(obj.0, 1); 
}
```

Or you can specify a custom generator function using the `#[generator]` attribute:
```rust
use derivable_object_pool::prelude::*;

#[derive(ObjectPool)]
#[generator(Test::new_item)]
struct Test(i32);

impl Test {
    fn new_item() -> Self {
        Test(10)
    }
}

fn main() {
    let mut obj = Test::new();
    obj.0 += 1;
    assert_eq!(obj.0, 11);
    drop(obj); // obj is returned to the pool
    assert_eq!(Test::pool().len(), 1);
    let mut obj = Test::new();
    assert_eq!(Test::pool().len(), 0);
    assert_eq!(obj.0, 11); 
}
```

# License

This project is licensed under the dual MIT/Apache-2.0 license. For more
information, see [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT).