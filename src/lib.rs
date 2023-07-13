//! # Derivable Object Pool
//! 
//! This crate provides a trait that can be derived to implement an object pool
//! for a type with a single line of code. Allowing the user to forget about
//! the implementation details of the [`ObjectPool`] and focus on the important
//! parts of their code
//! 
//! This crate has the following features compared to other object pool crates:
//! - **Derivable**: The pool is simple to use and can be used with any type. Can 
//! be just derived using the [`#[derive(ObjectPool)]`](derive@ObjectPool) 
//! attribute macro.
//! - **Reusable**: The user can use the [`ObjectPool::new`] function to create
//! objects from the pool, which will reuse objects from the pool if possible.
//! This items are wrapped in a [`Reusable`] struct, which will be returned to
//! the pool when dropped.
//! - **Thread Safe**: The pool is thread-safe (through the use of a [`Mutex`])
//! and can be used in a multi-threaded environment.
//! - **Simple**: The user doesn't need to create a pool for each type manually
//! and can use the [`ObjectPool::new`] function to create objects from the
//! pool.
//! - **Flexible**: The user can configure the pool to use a custom generator
//! function (see attributes in [`#[derive(ObjectPool)]`](derive@ObjectPool)) or
//! just use the [`Default`] trait to create new objects.
//! 
//! # Example
//! 
//! ```
//! use derivable_object_pool::prelude::*;
//! 
//! #[derive(Default, ObjectPool)]
//! struct Test(i32);
//! 
//! fn main() {
//!     let mut obj = Test::new();
//!     obj.0 += 1;
//!     assert_eq!(obj.0, 1);
//!     drop(obj); // obj is returned to the pool
//!     assert_eq!(Test::pool().len(), 1);
//!     let mut obj = Test::new();
//!     assert_eq!(Test::pool().len(), 0);
//!     assert_eq!(obj.0, 1); 
//! }
//! ```
use std::borrow::{Borrow, BorrowMut};
use std::mem::{forget, ManuallyDrop};
use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, MutexGuard};

pub use derivable_object_pool_macros::ObjectPool;

/// Allows for the creation of objects that can be reused. This is useful for
/// objects that are expensive to create, but are used frequently. This trait
/// can be derived using the `#[derive(ObjectPool)]` attribute macro (for more
/// information, see the documentation for the [`ObjectPool`] trait)
/// 
/// The new objects will be created using a generator function, which can be
/// specified using the `#[generator(function_name)]` attribute macro on the
/// struct. If no generator is specified, the trait will use the [`Default`]
/// trait to create new objects.
/// 
/// # Example
/// 
/// Example without a generator:
/// ```
/// use derivable_object_pool::prelude::*;
/// 
/// #[derive(Default, ObjectPool)]
/// struct Test {
///     a: i32,
///     b: f64,
/// }
/// 
/// fn main() {
///     let obj = Test::new();
///     drop(obj); // obj is returned to the pool
///     let obj2 = Test::new(); // obj2 is the same object as obj
/// }
/// ```
/// 
/// Example with a generator:
/// ```
/// 
/// use derivable_object_pool::prelude::*;
/// 
/// #[derive(ObjectPool)]
/// #[generator(Test::new_item)]
/// struct Test {
///     a: i32,
///     b: f64,
/// }
/// 
/// impl Test {
///     fn new_item() -> Self {
///         Self {
///             a: 1,
///             b: 1.0,
///         }
///     }
/// }
/// 
/// fn main() {
///     let obj = Test::new();
///     drop(obj); // obj is returned to the pool
///     let obj2 = Test::new(); // obj2 is the same object as obj
/// }
/// ```
pub trait ObjectPool: Sized {
    /// Returns a reference to the pool for this type of object. This allows
    /// you to interact with the pool directly, if you need to.
    /// 
    /// # Example
    /// ```
    /// use derivable_object_pool::prelude::*;
    /// 
    /// #[derive(Default, ObjectPool)]
    /// struct Test;
    /// 
    /// fn main() {
    ///     let pool = Test::pool();
    ///     assert_eq!(pool.len(), 0);
    ///     let obj = Test::new();
    ///     drop(obj);
    ///     assert_eq!(pool.len(), 1);
    ///     pool.clear();
    ///     assert_eq!(pool.len(), 0);
    /// }
    /// ```
    fn pool<'a>() -> &'a Pool<Self>;

    /// Creates a new object. If there are any objects in the pool, one of them
    /// will be returned. Otherwise, a new object will be created using the 
    /// generator function.
    /// 
    /// # Example
    /// ```
    /// use derivable_object_pool::prelude::*;
    /// 
    /// #[derive(Default, ObjectPool)]
    /// struct Test(i32);
    /// 
    /// fn main() {
    ///     let mut obj = Test::new();
    ///     assert_eq!(obj.0, 0);
    ///     obj.0 = 1;
    ///     drop(obj);
    ///     let obj = Test::new();
    ///     assert_eq!(obj.0, 1);
    /// }
    /// ```
    #[must_use]
    #[inline]
    fn new() -> Reusable<Self> {
        let mut pool = Self::pool().get_pool();
        match pool.pop() {
            Some(item) => Reusable::new(item),
            None => Reusable::new((Self::pool().generator)()),
        }
    }
}

/// A pool of objects that can be reused. This is useful for objects that are
/// expensive to create, but are used frequently. This struct can be created
/// using the [`Pool::new`] function. However, it is highly recommended that
/// you use the [`ObjectPool`] trait instead, as it is much easier to use.
/// 
/// 
/// # Example
/// 
/// Example without deriving [`ObjectPool`]:
/// 
/// ```
/// use derivable_object_pool::prelude::*;
/// 
/// #[derive(Default)]
/// struct Test;
/// 
/// static POOL: Pool<Test> = Pool::new(Test::default);
/// 
/// impl ObjectPool for Test {
///     fn pool<'a>() -> &'a Pool<Self> {
///         &POOL
///     }
/// }
/// 
/// fn main() {
///    let obj = Test::new();
///    drop(obj); // obj is returned to the pool
///    assert_eq!(POOL.len(), 1);
/// }
/// ```
pub struct Pool<T> {
    /// The pool of objects that can be reused. The pool uses a [`Mutex`] to
    /// ensure that it is thread-safe.
    pool: Mutex<Vec<T>>,
    /// The generator function that is used to create new objects.
    generator: fn() -> T,
}

impl<T> Pool<T> {
    /// Creates a new pool of objects. The pool will use the specified generator
    /// function to create new objects.
    #[must_use]
    #[inline]
    pub const fn new(generator: fn() -> T) -> Self {
        Self {
            pool: Mutex::new(Vec::new()),
            generator,
        }
    }

    /// Returns a locked reference to the pool. This is used internally by the
    /// rest of the library, but it can also be used to interact with the pool
    /// directly.
    #[inline]
    fn get_pool(&self) -> MutexGuard<'_, Vec<T>> {
        self.pool.lock().unwrap()
    }

    /// Returns the number of objects in the pool.
    #[inline]
    pub fn len(&self) -> usize {
        self.get_pool().len()
    }

    /// Returns `true` if the pool is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.get_pool().is_empty()
    }

    /// Inserts an object into the pool while taking ownership of it.
    #[inline]
    pub fn insert(&self, item: T) {
        self.get_pool().push(item);
    }

    /// Removes all objects from the pool.
    #[inline]
    pub fn clear(&self) {
        self.get_pool().clear();
    }

    /// Removes an object from the pool and returns the object while taking
    /// ownership of it.
    #[inline]
    pub fn remove(&self) -> Option<T> {
        self.get_pool().pop()
    }
}

impl<T: ObjectPool> Pool<T> {
    /// Removes an object from the pool and returns a resuable wrapper for it,
    /// which will return the object to the pool when it is dropped.
    #[inline]
    pub fn remove_reusable(&self) -> Option<Reusable<T>> {
        self.remove().map(Reusable::new)
    }
}

/// A wrapper for an object that will return the object to the pool when it is
/// dropped. This is useful for objects that are expensive to create, but are
/// used frequently. This struct can be created using the 
/// [`Pool::remove_reusable`] function. However, it is highly recommended that
/// you use the [`ObjectPool::new`] function instead, as it will reuse objects
/// from the pool if possible.
/// 
/// The object implements [`Deref`] and [`DerefMut`] to allow you to access the
/// object inside the wrapper. It also implements [`Borrow`] and [`BorrowMut`]
/// to allow you to access the object inside the wrapper immutably or mutably.
/// Finally, it implements [`AsRef`] and [`AsMut`] to allow you to access the
/// object inside the wrapper immutably or mutably.
/// 
/// # Example
/// 
/// ```
/// use derivable_object_pool::prelude::*;
/// 
/// #[derive(Default, ObjectPool)]
/// struct Test(i32);
/// 
/// fn test(obj: &mut Test) {
///     obj.0 += 1;
/// }
/// 
/// fn main() {
///    let mut obj = Test::new();
///    assert_eq!(obj.0, 0);
///    test(&mut obj);
///    assert_eq!(obj.0, 1);
/// }
/// ```
#[repr(transparent)] 
pub struct Reusable<T: ObjectPool> {
    /// The wrapped object. This is a `ManuallyDrop` to ensure that the object
    /// is not dropped when the wrapper is dropped.
    item: ManuallyDrop<T>,
}

impl<T: ObjectPool> Reusable<T> {
    /// Creates a new reusable wrapper for the specified object.
    #[inline]
    const fn new(item: T) -> Self {
        Self {
            item: ManuallyDrop::new(item),
        }
    }

    /// Returns the owned object inside the wrapper. This will return the object
    /// without returning it to the pool. This is useful if you want to take
    /// ownership of the object.
    pub fn into_inner(mut self) -> T {
        let ret = unsafe { ManuallyDrop::take(&mut self.item) };
        forget(self);
        ret
    }
}

impl<T: ObjectPool> Borrow<T> for Reusable<T> {
    #[inline]
    fn borrow(&self) -> &T {
        &self.item
    }
}

impl<T: ObjectPool> BorrowMut<T> for Reusable<T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.item
    }
}

impl<T: ObjectPool> AsRef<T> for Reusable<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        &self.item
    }
}

impl<T: ObjectPool> AsMut<T> for Reusable<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        &mut self.item
    }
}

impl<T: ObjectPool> Deref for Reusable<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<T: ObjectPool> DerefMut for Reusable<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<T: ObjectPool> Drop for Reusable<T> {
    #[inline]
    fn drop(&mut self) {
        T::pool()
            .insert(unsafe { ManuallyDrop::take(&mut self.item) });
    }
}

impl<T: ObjectPool> From<T> for Reusable<T> {
    #[inline]
    fn from(item: T) -> Self {
        Self::new(item)
    }
}

/// This is the prelude for the `derivable-object-pool` crate. It contains the
/// main traits and structs that you will need to use the crate. It is
/// recommended that you import this prelude at the top of your file.
pub mod prelude {
    pub use crate::{ObjectPool, Pool, Reusable};
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;

    #[derive(Default, ObjectPool)]
    struct Test {
        a: i32,
        b: f64,
        c: bool,
        d: Vec<usize>,
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

    #[derive(ObjectPool)]
    #[generator(Test2::new_item)]
    /// This is a different attribute: a comment, tests the macro ignores it properly
    struct Test2 {
        a: i32,
        b: f64,
        c: bool,
        d: Vec<usize>,
    }

    impl Test2 {
        fn new_item() -> Self {
            Self {
                a: 0,
                b: 0.0,
                c: false,
                d: Vec::new(),
            }
        }
    }

    #[test]
    fn new_objects_with_generator() {
        let obj = Test2::new();
        drop(obj);
        assert_eq!(1, Test2::pool().len());

        let obj = Test2::new();
        assert_eq!(0, Test2::pool().len());
        let obj2 = Test2::new();

        drop(obj);
        drop(obj2);

        assert_eq!(2, Test2::pool().len());
    }
}
