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
//!         LateStatic::assign(&FOO, Foo { value: 42 });
//!     }
//!     println!("{}", FOO.value);
//! }
//! ```
#![cfg_attr(not(test), no_std)]

use core::cell::UnsafeCell;

/// Static value that is manually initialized at runtime.
pub struct LateStatic<T> {
    val: UnsafeCell<Option<T>>,
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
    /// # Safety
    ///
    /// This is completely unsafe if there is even the slightest chance of another
    /// thread trying to dereference the variable.
    pub unsafe fn assign(instance: &LateStatic<T>, val: T) {
        let option: &mut Option<T> = &mut *instance.val.get();
        if option.is_some() {
            panic!("Second assignment to late static");
        } else {
            *option = Some(val);
        }
    }

    /// Invalidate the late static by removing its inner value.
    ///
    /// # Safety
    ///
    /// This is completely unsafe if there is even the slightest chance of another
    /// thread trying to dereference the variable.
    pub unsafe fn clear(instance: &LateStatic<T>) {
        if !Self::has_value(instance) {
            panic!("Tried to clear a late static without a value");
        }
        let option: &mut Option<T> = &mut *instance.val.get();
        *option = None;
    }

    /// Whether a value is assigned to this LateStatic.
    ///
    /// # Safety
    ///
    /// This is completely unsafe if there is even the slightest chance of another
    /// thread trying to dereference the variable.
    pub unsafe fn has_value(instance: &LateStatic<T>) -> bool {
        let option: &Option<T> = &*instance.val.get();
        option.is_some()
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
            assert!(!LateStatic::has_value(&ASSIGN_ONCE_TEST));
            LateStatic::assign(&ASSIGN_ONCE_TEST, 42);
            assert!(LateStatic::has_value(&ASSIGN_ONCE_TEST));
        }
    }

    static ASSIGN_TWICE_TEST: LateStatic<u32> = LateStatic::new();
    #[test]
    #[should_panic]
    fn assign_twice() {
        unsafe {
            LateStatic::assign(&ASSIGN_TWICE_TEST, 42);
            LateStatic::assign(&ASSIGN_TWICE_TEST, 37);
        }
    }

    struct Foo {
        pub value: u32,
    }

    static DEREF_CONST_TEST: LateStatic<Foo> = LateStatic::new();
    #[test]
    fn deref_const() {
        unsafe {
            LateStatic::assign(&DEREF_CONST_TEST, Foo { value: 42 });
        }
        assert_eq!(DEREF_CONST_TEST.value, 42);
    }

    static mut DEREF_MUT_TEST: LateStatic<Foo> = LateStatic::new();
    #[test]
    fn deref_mut() {
        unsafe {
            LateStatic::assign(&DEREF_MUT_TEST, Foo { value: 42 });
            assert_eq!(DEREF_MUT_TEST.value, 42);
            DEREF_MUT_TEST.value = 37;
            assert_eq!(DEREF_MUT_TEST.value, 37);
        }
    }

    static mut DEREF_WITHOUT_VALUE: LateStatic<Foo> = LateStatic::new();
    #[test]
    #[should_panic]
    fn deref_without_value() {
        unsafe {
            #[allow(clippy::no_effect)]
            DEREF_WITHOUT_VALUE.value;
        }
    }

    static mut CLEAR_TEST: LateStatic<Foo> = LateStatic::new();
    #[test]
    fn clear() {
        unsafe {
            LateStatic::assign(&CLEAR_TEST, Foo { value: 42 });
            assert_eq!(CLEAR_TEST.value, 42);
            LateStatic::clear(&CLEAR_TEST);
            assert!(!LateStatic::has_value(&CLEAR_TEST));
        }
    }

    static mut CLEAR_WITHOUT_VALUE: LateStatic<Foo> = LateStatic::new();
    #[test]
    #[should_panic]
    fn clear_without_value() {
        unsafe {
            LateStatic::clear(&CLEAR_WITHOUT_VALUE);
        }
    }
}
