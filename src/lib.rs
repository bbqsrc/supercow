// Copyright 2016 Jason Lingle
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::{Borrow, ToOwned};
use std::convert::From;
use std::cmp;
use std::fmt;
use std::mem;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::slice;

/// Miscelaneous things used to integrate other code with Supercow, but which
/// is not of interest to end users.
pub mod aux {
    use std::borrow::Borrow;
    use std::rc::Rc;
    use std::sync::Arc;

    /// Marker trait indicating a `Deref`-like which always returns the same
    /// reference.
    ///
    /// This is not indended for general use outside Supercow. Notably, `Box`
    /// and mundane references satisfy this trait's requirements, but
    /// deliberately do not implement it. It is also not a subtrait of `Deref`
    /// due to some additional special logic around boxes.
    ///
    /// ## Unsafety
    ///
    /// Behaviour is undefined if the implementation does not always return the
    /// same reference from `deref()` for any particular implementing value
    /// (including if that value is moved).
    pub unsafe trait ConstDeref {
        type Target : ?Sized;
        fn const_deref(&self) -> &Self::Target;
    }

    unsafe impl<T : ?Sized> ConstDeref for Rc<T> {
        type Target = T;
        fn const_deref(&self) -> &T { self }
    }

    unsafe impl<T : ?Sized> ConstDeref for Arc<T> {
        type Target = T;
        fn const_deref(&self) -> &T { self }
    }

    unsafe impl<T : ConstDeref + ?Sized> ConstDeref for Box<T> {
        type Target = T::Target;
        fn const_deref(&self) -> &T::Target {
            (**self).const_deref()
        }
    }

    /// Marker trait for `Borrow`s which always return the same reference, even
    /// after mutation (including entirely replacing the value).
    ///
    /// This is needed by `Supercow::to_mut`.
    ///
    /// ## Unsafety
    ///
    /// Behaviour is undefined if `borrow()`, when passed the same reference at
    /// different times, returns different references.
    pub unsafe trait ConstBorrow<T : ?Sized>: Borrow<T> { }
    unsafe impl<T : ?Sized> ConstBorrow<T> for T { }
    unsafe impl<T> ConstBorrow<[T]> for [T;0] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;1] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;2] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;3] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;4] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;5] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;6] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;7] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;8] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;9] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;10] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;11] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;12] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;13] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;14] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;15] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;16] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;17] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;18] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;19] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;20] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;21] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;22] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;23] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;24] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;25] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;26] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;27] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;28] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;29] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;30] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;31] { }
    unsafe impl<T> ConstBorrow<[T]> for [T;32] { }

    /// Marker trait identifying a reference type which begins with an absolute
    /// address and contains no other address-dependent information.
    ///
    /// `Supercow` expects to be able to read the first pointer-sized value of
    /// such a reference and perform address arithmetic upon it.
    ///
    /// There is no utility of applying this trait to anything other than a
    /// const reference.
    ///
    /// ## Unsafety
    ///
    /// Behaviour is undefined if a marked type does not begin with a real
    /// pointer to a value (with the usual exception of ZSTs) or if other parts
    /// of the type contain address-dependent information.
    ///
    /// Behaviour is undefined if the reference has any `Drop` implementation,
    /// should a future Rust version make such things possible.
    pub unsafe trait PointerFirstRef { }
    unsafe impl<'a, T : Sized> PointerFirstRef for &'a T { }
    unsafe impl<'a, T> PointerFirstRef for &'a [T] { }
    unsafe impl<'a> PointerFirstRef for &'a str { }
    unsafe impl<'a> PointerFirstRef for &'a ::std::ffi::CStr { }
    unsafe impl<'a> PointerFirstRef for &'a ::std::ffi::OsStr { }
    unsafe impl<'a> PointerFirstRef for &'a ::std::path::Path { }
}

use self::aux::*;

/// Defines a "feature set" for a custom `Supercow` type.
///
/// ## Syntax
///
/// ```
/// #[macro_use] extern crate supercow;
///
/// # pub trait SomeTrait { }
/// # pub trait AnotherTrait { }
///
/// supercow_features!(
///   /// Some documentation, etc, if desired.
///   pub trait FeatureName: SomeTrait, AnotherTrait);
/// supercow_features!(
///   pub trait FeatureName2: Clone, SomeTrait, AnotherTrait);
///
/// # fn main() { }
/// ```
///
/// ## Semantics
///
/// A public trait named `FeatureName` is defined which extends all the listed
/// traits, other than `Clone`, and in addition to `ConstDeref`. If listed,
/// `Clone` *must* come first. If `Clone` is listed, the trait gains a
/// `clone_boxed()` method and `Box<FeatureName>` is `Clone`. All types which
/// implement all the listed traits (including `Clone`) and `ConstDeref`
/// implement `FeatureName`.
#[macro_export]
macro_rules! supercow_features {
    // It's unclear why $req:path doesn't work, but apparently constraints
    // allow neither `path` nor `ty`.
    ($(#[$meta:meta])* pub trait $feature_name:ident: Clone $(, $req:ident)*) => {
        $(#[$meta])*
        pub trait $feature_name<'a>: $($req +)* $crate::aux::ConstDeref + 'a {
            fn clone_boxed
                (&self)
                 -> Box<$feature_name<'a, Target = Self::Target> + 'a>;
        }
        impl<'a, T : 'a + $($req +)* Clone + $crate::aux::ConstDeref>
        $feature_name<'a> for T {
            fn clone_boxed
                (&self)
                 -> Box<$feature_name<'a, Target = Self::Target> + 'a>
            {
                let cloned: T = self.clone();
                Box::new(cloned)
            }
        }
        impl<'a, S : 'a> Clone for Box<$feature_name<'a, Target = S> + 'a> {
            fn clone(&self) -> Self {
                $feature_name::clone_boxed(&**self)
            }
        }
    };

    ($(#[$meta:meta])* pub trait $feature_name:ident: $($req:ident),*) => {
        $(#[$meta])*
        pub trait $feature_name<'a>: $($req +)* $crate::aux::ConstDeref + 'a {
        }
        impl<'a, T : 'a + $($req +)* Clone + $crate::aux::ConstDeref>
        $feature_name<'a> for T {
        }
    };
}

supercow_features!(
    /// The default feature set for special `Supercow` references.
    pub trait DefaultFeatures: Clone);
supercow_features!(
    /// The feature set used for `ASupercow` references.
    pub trait SyncFeatures: Clone, Send, Sync);

pub struct Supercow<'a, OWNED, BORROWED : ?Sized = OWNED,
                    SPECIAL = Box<DefaultFeatures<'a, Target = BORROWED>>>
where BORROWED : 'a,
      &'a BORROWED : PointerFirstRef,
      SPECIAL : ConstDeref<Target = BORROWED> {
    // In order to implement `Deref` in a branch-free fashion that isn't
    // sensitive to the Supercow being moved, we set `ptr_mask` and
    // `ptr_displacement` such that
    // `target = &*((&self & sext(ptr_mask)) + ptr_displacement)`
    // (arithmetic in terms of bytes, obviously).
    //
    // So for the three cases:
    //
    // Owned => ptr_mask = ~0u, ptr_displacement = offsetof(self, Owned.0)
    // Borrowed, Special => ptr_mask = 0u, ptr_displacement = address
    //
    // `ptr_mask` is an i8 since that may allow better struct layout in the
    // future (since `SupercowData` has a decent amount of padding).
    //
    // In order to support DSTs, `ptr_displacement` is actually a reference to
    // `BORROWED`. We assume the first pointer-sized value is the actual
    // pointer (see `PointerFirstRef`). `ptr_displacement` may not actually be
    // dereferenced.
    ptr_displacement: &'a BORROWED,
    ptr_mask: i8,
    state: SupercowData<'a, OWNED, BORROWED, SPECIAL>,
}

enum SupercowData<'a, OWNED, BORROWED : 'a + ?Sized, SPECIAL> {
    Owned(OWNED),
    Borrowed(&'a BORROWED),
    Special(SPECIAL),
}
use self::SupercowData::*;

impl<'a, OWNED, BORROWED : ?Sized, SPECIAL> Deref
for Supercow<'a, OWNED, BORROWED, SPECIAL>
where BORROWED : 'a,
      &'a BORROWED : PointerFirstRef,
      SPECIAL : ConstDeref<Target = BORROWED> {
    type Target = BORROWED;
    #[inline]
    fn deref(&self) -> &BORROWED {
        let mask = self.ptr_mask as isize as usize;
        let self_address = self as *const Self as usize;

        let mut target_ref = self.ptr_displacement;
        unsafe {
            let target_address: &mut usize = mem::transmute(&mut target_ref);
            let nominal_address = *target_address;
            *target_address = (self_address & mask) + nominal_address;
        }
        target_ref
    }
}

impl<'a, OWNED, BORROWED : ?Sized, SPECIAL>
Supercow<'a, OWNED, BORROWED, SPECIAL>
where OWNED : Borrow<BORROWED>,
      BORROWED : 'a,
      &'a BORROWED : PointerFirstRef,
      SPECIAL : ConstDeref<Target = BORROWED> {
    pub fn owned(inner: OWNED) -> Self {
        Self::from_data(Owned(inner))
    }

    pub fn borrowed<T : Borrow<BORROWED> + ?Sized>(inner: &'a T) -> Self {
        Self::from_data(Borrowed(inner.borrow()))
    }

    pub fn special<T : Into<SPECIAL>>(inner: T) -> Self {
        Self::from_data(Special(inner.into()))
    }

    fn from_data(data: SupercowData<'a, OWNED, BORROWED, SPECIAL>) -> Self {
        let mut this = Supercow {
            ptr_mask: 0,
            ptr_displacement: unsafe { mem::uninitialized() },
            state: data,
        };
        this.set_ptr();
        this
    }

    fn set_ptr(&mut self) {
        {
            let borrowed_ptr = match self.state {
                Owned(ref r) => r.borrow(),
                Borrowed(r) => r,
                Special(ref s) => s.const_deref(),
            };
            // There's no safe way to propagate `borrowed_ptr` into
            // `ptr_displacement` since the former has a borrow scoped to this
            // function.
            unsafe {
                let dst: &mut [u8] = slice::from_raw_parts_mut(
                    &mut self.ptr_displacement as *mut&'a BORROWED
                        as *mut u8,
                    mem::size_of::<&'a BORROWED>());
                let src: &[u8] = slice::from_raw_parts(
                    &borrowed_ptr as *const&BORROWED as *const u8,
                    mem::size_of::<&'a BORROWED>());
                dst.copy_from_slice(src);
            }
        }

        // Use relative addressing if `ptr` is inside `self` and absolute
        // addressing otherwise.
        //
        // Ordinarily, `ptr` will always be inside `self` if the state is
        // `Owned`, and outside otherwise. However, it is possible to create
        // `Borrow` implementations that return arbitrary pointers, so we
        // handle the two cases like self instead.
        let self_start = self as *const Self as usize;
        let self_end = self_start + mem::size_of::<Self>();
        let addr: &mut usize = unsafe {
            mem::transmute(&mut self.ptr_displacement)
        };

        if *addr >= self_start && *addr < self_end {
            self.ptr_mask = !0;
            *addr -= self_start;
        } else {
            self.ptr_mask = 0;
        }
    }
}

impl<'a, OWNED, BORROWED : ?Sized, SPECIAL> From<OWNED>
for Supercow<'a, OWNED, BORROWED, SPECIAL>
where OWNED : Borrow<BORROWED>,
      BORROWED : 'a,
      &'a BORROWED : PointerFirstRef,
      SPECIAL : ConstDeref<Target = BORROWED> {
    fn from(inner: OWNED) -> Self {
        Self::from_data(SupercowData::Owned(inner))
    }
}

impl<'a, OWNED, BORROWED : ?Sized, SPECIAL> From<&'a OWNED>
for Supercow<'a, OWNED, BORROWED, SPECIAL>
where OWNED : Borrow<BORROWED>,
      BORROWED : 'a,
      &'a BORROWED : PointerFirstRef,
      SPECIAL : ConstDeref<Target = BORROWED> {
    fn from(inner: &'a OWNED) -> Self {
        Self::from_data(SupercowData::Borrowed(inner.borrow()))
    }
}

impl<'a, OWNED, BORROWED : ?Sized, SPECIAL>
Supercow<'a, OWNED, BORROWED, SPECIAL>
where OWNED : ConstBorrow<BORROWED>,
      BORROWED : 'a + ToOwned<Owned = OWNED>,
      &'a BORROWED : PointerFirstRef,
      SPECIAL : ConstDeref<Target = BORROWED> {
    pub fn to_mut(&mut self) -> &mut OWNED {
        let new = match self.state {
            Owned(ref mut r) => return r,
            Borrowed(r) => Self::owned(r.to_owned()),
            Special(ref s) => Self::owned(s.const_deref().to_owned()),
        };
        *self = new;
        match self.state {
            Owned(ref mut r) => r,
            _ => unreachable!(),
        }
    }
}

impl<'a, OWNED, BORROWED : ?Sized, SPECIAL> Clone
for Supercow<'a, OWNED, BORROWED, SPECIAL>
where OWNED : Clone,
      BORROWED : 'a,
      &'a BORROWED : PointerFirstRef,
      SPECIAL : Clone + ConstDeref<Target = BORROWED> {
    fn clone(&self) -> Self {
        Supercow {
            ptr_mask: self.ptr_mask,
            ptr_displacement: self.ptr_displacement,
            state: match self.state {
                Owned(ref o) => Owned(o.clone()),
                Borrowed(r) => Borrowed(r),
                Special(ref s) => Special(s.clone()),
            }
        }
    }
}

macro_rules! deleg_fmt {
    ($tr:ident) => {
        impl<'a, OWNED, BORROWED : ?Sized, SPECIAL> fmt::$tr
        for Supercow<'a, OWNED, BORROWED, SPECIAL>
        where BORROWED : 'a + fmt::$tr,
              &'a BORROWED : PointerFirstRef,
              SPECIAL : ConstDeref<Target = BORROWED> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                (**self).fmt(f)
            }
        }
    }
}
deleg_fmt!(Binary);
deleg_fmt!(Debug);
deleg_fmt!(Display);
deleg_fmt!(LowerExp);
deleg_fmt!(LowerHex);
deleg_fmt!(Octal);
deleg_fmt!(Pointer);
deleg_fmt!(UpperExp);
deleg_fmt!(UpperHex);

impl<'a, OWNED, BORROWED : ?Sized, SPECIAL, T> cmp::PartialEq<T>
for Supercow<'a, OWNED, BORROWED, SPECIAL>
where T : Deref<Target = BORROWED>,
      BORROWED : 'a + PartialEq<BORROWED>,
      &'a BORROWED : PointerFirstRef,
      SPECIAL : ConstDeref<Target = BORROWED> {
    fn eq(&self, other: &T) -> bool {
        **self == **other
    }

    fn ne(&self, other: &T) -> bool {
        **self != **other
    }
}

impl<'a, OWNED, BORROWED : ?Sized, SPECIAL> cmp::Eq
for Supercow<'a, OWNED, BORROWED, SPECIAL>
where BORROWED : 'a + Eq,
      &'a BORROWED : PointerFirstRef,
      SPECIAL : ConstDeref<Target = BORROWED> { }

impl<'a, OWNED, BORROWED : ?Sized, SPECIAL, T> cmp::PartialOrd<T>
for Supercow<'a, OWNED, BORROWED, SPECIAL>
where T : Deref<Target = BORROWED>,
      BORROWED : 'a + PartialOrd<BORROWED>,
      &'a BORROWED : PointerFirstRef,
      SPECIAL : ConstDeref<Target = BORROWED> {
    fn partial_cmp(&self, other: &T) -> Option<cmp::Ordering> {
        (**self).partial_cmp(other)
    }

    fn lt(&self, other: &T) -> bool {
        **self < **other
    }

    fn le(&self, other: &T) -> bool {
        **self <= **other
    }

    fn gt(&self, other: &T) -> bool {
        **self > **other
    }

    fn ge(&self, other: &T) -> bool {
        **self >= **other
    }
}

impl<'a, OWNED, BORROWED : ?Sized, SPECIAL> cmp::Ord
for Supercow<'a, OWNED, BORROWED, SPECIAL>
where BORROWED : 'a + cmp::Ord,
      &'a BORROWED : PointerFirstRef,
      SPECIAL : ConstDeref<Target = BORROWED> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        (**self).cmp(other)
    }
}

impl<'a, OWNED, BORROWED : ?Sized, SPECIAL> Hash
for Supercow<'a, OWNED, BORROWED, SPECIAL>
where BORROWED : 'a + Hash,
      &'a BORROWED : PointerFirstRef,
      SPECIAL : ConstDeref<Target = BORROWED> {
    fn hash<H : Hasher>(&self, h: &mut H) {
        (**self).hash(h)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ref_to_owned() {
        let x = 42u32;
        let a: Supercow<u32> = Supercow::borrowed(&x);
        assert_eq!(x, *a);
        assert_eq!(&x as *const u32 as usize,
                   (&*a) as *const u32 as usize);

        let mut b = a.clone();
        assert_eq!(x, *b);
        assert_eq!(&x as *const u32 as usize,
                   (&*b) as *const u32 as usize);

        *b.to_mut() = 56;
        assert_eq!(42, *a);
        assert_eq!(x, *a);
        assert_eq!(&x as *const u32 as usize,
                   (&*a) as *const u32 as usize);
        assert_eq!(56, *b);
    }

    #[test]
    fn supports_dst() {
        let a: Supercow<String, str> = Supercow::borrowed("hello");
        let b: Supercow<String, str> = Supercow::owned("hello".to_owned());
        assert_eq!(a, b);
    }
}
