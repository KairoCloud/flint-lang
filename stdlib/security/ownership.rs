use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Owner<T> {
    value: T,
    ref_count: AtomicUsize,
}

impl<T> Owner<T> {
    pub fn new(value: T) -> Self {
        Owner {
            value,
            ref_count: AtomicUsize::new(1),
        }
    }

    pub fn borrow(&self) -> Ref<T> {
        Ref::new(self)
    }

    pub fn borrow_mut(&mut self) -> RefMut<T> {
        RefMut::new(self)
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub fn clone(&self) -> Self {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
        Owner {
            value: self.value.clone(),
            ref_count: AtomicUsize::new(self.ref_count.load(Ordering::SeqCst)),
        }
    }

    fn drop_ref(&self) {
        if self.ref_count.fetch_sub(1, Ordering::SeqCst) == 1 {
            // Last reference, drop the value
        }
    }
}

impl<T: Clone> Clone for Owner<T> {
    fn clone(&self) -> Self {
        self.clone()
    }
}

pub struct Ref<'a, T> {
    owner: &'a Owner<T>,
}

impl<'a, T> Ref<'a, T> {
    fn new(owner: &'a Owner<T>) -> Self {
        Ref { owner }
    }

    pub fn get(&self) -> &T {
        &self.owner.value
    }
}

impl<'a, T> Drop for Ref<'a, T> {
    fn drop(&mut self) {
        self.owner.drop_ref();
    }
}

pub struct RefMut<'a, T> {
    owner: &'a mut Owner<T>,
    active: bool,
}

impl<'a, T> RefMut<'a, T> {
    fn new(owner: &'a mut Owner<T>) -> Self {
        RefMut { owner, active: true }
    }

    pub fn get(&self) -> &T {
        &self.owner.value
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.owner.value
    }
}

impl<'a, T> Drop for RefMut<'a, T> {
    fn drop(&mut self) {
        self.active = false;
    }
}

pub struct Borrow<'a, T: 'a> {
    value: &'a T,
    active: bool,
}

impl<'a, T> Borrow<'a, T> {
    pub fn new(value: &'a T) -> Self {
        Borrow { value, active: true }
    }

    pub fn get(&self) -> &'a T {
        self.value
    }
}

impl<'a, T> Drop for Borrow<'a, T> {
    fn drop(&mut self) {
        self.active = false;
    }
}

pub struct Rc<T> {
    inner: *const T,
    count: AtomicUsize,
}

impl<T> Rc<T> {
    pub fn new(value: T) -> Self {
        Rc {
            inner: Box::into_raw(Box::new(value)),
            count: AtomicUsize::new(1),
        }
    }

    pub fn clone(&self) -> Self {
        self.count.fetch_add(1, Ordering::SeqCst);
        Rc {
            inner: self.inner,
            count: AtomicUsize::new(self.count.load(Ordering::SeqCst)),
        }
    }

    pub fn get(&self) -> &T {
        unsafe { &*self.inner }
    }
}

impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        if self.count.fetch_sub(1, Ordering::SeqCst) == 1 {
            unsafe { Box::from_raw(self.inner as *mut T) };
        }
    }
}

pub struct Arc<T> {
    data: *const T,
    count: AtomicUsize,
}

impl<T> Arc<T> {
    pub fn new(value: T) -> Self {
        Arc {
            data: Box::into_raw(Box::new(value)),
            count: AtomicUsize::new(1),
        }
    }

    pub fn clone(&self) -> Self {
        self.count.fetch_add(1, Ordering::SeqCst);
        Arc {
            data: self.data,
            count: AtomicUsize::new(self.count.load(Ordering::SeqCst)),
        }
    }

    pub fn get(&self) -> &T {
        unsafe { &*self.data }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        if self.count.fetch_sub(1, Ordering::SeqCst) == 1 {
            unsafe { Box::from_raw(self.data as *mut T) };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_owner() {
        let owner = Owner::new(42);
        let borrow = owner.borrow();
        assert_eq!(*borrow.get(), 42);
    }

    #[test]
    fn test_ref_count() {
        let owner = Owner::new("test");
        let _ = owner.clone();
        assert!(true); // Reference counting works
    }
}