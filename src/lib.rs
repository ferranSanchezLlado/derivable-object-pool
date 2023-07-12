use std::borrow::{Borrow, BorrowMut};
use std::mem::{forget, ManuallyDrop};
use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, MutexGuard};

pub use object_pool_derive::ObjectPool;

pub trait ObjectPool: Default {
    fn pool<'a>() -> &'a Pool<Self>;
    #[inline]
    fn new() -> Reusable<Self> {
        let mut pool = Self::pool().get_pool();
        match pool.is_empty() {
            true => Reusable::new(Self::default()),
            false => Reusable::new(pool.pop().unwrap()),
        }
    }
}

pub struct Pool<T> {
    pool: Mutex<Vec<T>>,
}

impl<T: Default> Pool<T> {
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            pool: Mutex::new(Vec::new()),
        }
    }
}

impl<T> Pool<T> {
    #[inline]
    fn get_pool(&self) -> MutexGuard<'_, Vec<T>> {
        self.pool.lock().unwrap()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.get_pool().len()
    }

    #[inline]
    pub fn insert(&self, item: T) {
        self.get_pool().push(item);
    }

    #[inline]
    pub fn clear(&self) {
        self.get_pool().clear();
    }
}

pub struct Reusable<T: ObjectPool> {
    item: ManuallyDrop<T>,
}

impl<T: ObjectPool> Reusable<T> {
    #[inline]
    const fn new(item: T) -> Self {
        Self {
            item: ManuallyDrop::new(item),
        }
    }

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
            .get_pool()
            .push(unsafe { ManuallyDrop::take(&mut self.item) });
    }
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
}
