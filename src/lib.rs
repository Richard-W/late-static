//! Initialize variables at runtime which then behave like static variables.
//!
//! ```rust
//! extern crate late_static;
//! use late_static::LateStatic;
//!
//! struct Foo {
//!     pub value: u32,
//! }
//!
//! static FOO: LateStatic<Foo> = LateStatic::new();
//!
//! fn main() {
//!     unsafe {
//!         FOO.assign(Foo { value: 42 });
//!     }
//!     println!("{}", FOO.value);
//! }
//! ```
#![cfg_attr(not(test), no_std)]

use core::cell::UnsafeCell;

/// Static value that is manually initialized at runtime.
pub struct LateStatic<T> {
    val: UnsafeCell<Option<T>>
}

unsafe impl<T: Send> core::marker::Send for LateStatic<T> {}
unsafe impl<T: Send> core::marker::Sync for LateStatic<T> {}

impl<T> LateStatic<T> {
    /// Construct a LateStatic.
    pub const fn new() -> Self {
        LateStatic {
            val: UnsafeCell::new(None),
        }
    }

    /// Assign a value to the late static.
    ///
    /// This only works once. A second call to assign for a given variable will panic.
    ///
    /// This is completely unsafe if there is even the slightest chance of another
    /// thread trying to dereference the variable.
    pub unsafe fn assign(&self, val: T) {
        let option: &mut Option<T> = &mut *self.val.get();
        if option.is_some() {
            panic!("Second assignment to late static");
        }
        else {
            *option = Some(val);
        }
    }
}

impl<T> core::ops::Deref for LateStatic<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            let option: &Option<T> = &*self.val.get();
            match option {
                Some(ref val) => val,
                None => panic!("Dereference of late static before a value was assigned"),
            }
        }
    }
}

impl<T> core::ops::DerefMut for LateStatic<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe {
            let option: &mut Option<T> = &mut *self.val.get();
            match option {
                Some(ref mut val) => val,
                None => panic!("Dereference of late static before a value was assigned"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static ASSIGN_ONCE_TEST: LateStatic<u32> = LateStatic::new();
    #[test]
    fn assign_once() {
        unsafe {
            ASSIGN_ONCE_TEST.assign(42);
        }
    }


    static ASSIGN_TWICE_TEST: LateStatic<u32> = LateStatic::new();
    #[test]
    #[should_panic]
    fn assign_twice() {
        unsafe {
            ASSIGN_TWICE_TEST.assign(42);
            ASSIGN_TWICE_TEST.assign(37);
        }
    }

    struct Foo {
        pub value: u32,
    }

    static DEREF_CONST_TEST: LateStatic<Foo> = LateStatic::new();
    #[test]
    fn deref_const() {
        unsafe {
            DEREF_CONST_TEST.assign(Foo { value: 42 });
        }
        assert_eq!(DEREF_CONST_TEST.value, 42);
    }

    static mut DEREF_MUT_TEST: LateStatic<Foo> = LateStatic::new();
    #[test]
    fn deref_mut() {
        unsafe {
            DEREF_MUT_TEST.assign(Foo { value: 42 });
            assert_eq!(DEREF_MUT_TEST.value, 42);
            DEREF_MUT_TEST.value = 37;
            assert_eq!(DEREF_MUT_TEST.value, 37);
        }
    }
}