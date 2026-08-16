//! Type ids and downcasting for types with a single non-`'static` lifetime.
//!
//! [`std::any::Any`] only works for `'static` types, so it cannot describe the parse tree
//! nodes, tokens and token factories of this runtime: they are all parameterized over the
//! `'input` lifetime of the borrowed source text. This module provides the [`Tid`] trait,
//! which is `Any` for types with exactly one lifetime parameter.
//!
//! The trick is the [`TidAble::Static`] associated type: an implementation maps `T<'a>` to a
//! private, per-implementation `'static` witness type, and the type id of *that* witness is
//! used as the type id of `T<'a>`. Because [`Tid`] is invariant in `'a`, downcasting a
//! `dyn Tid<'a>` back to a `T<'a>` cannot change the lifetime, so it stays sound. The actual
//! type id still comes from [`std::any::TypeId`] — this is a lifetime-preserving wrapper
//! around `Any`, not a replacement for it.
//!
//! Implementations are written with the [`crate::tid!`] macro, which checks the shape of the `impl`
//! for you; hand-written `unsafe impl`s of [`TidAble`] are possible but easy to get wrong.
//!
//! Note that the lifetime is load-bearing, not ceremony. Keying the type id off
//! `T<'static>` and downcasting through a plain `TypeId` comparison — the only shape
//! [`Any`] alone could take here — compiles in safe Rust and lets a caller launder a local
//! borrow into a `&'static` reference. The `T: Tid<'a>` bound on the downcast methods, which
//! ties the target type to the trait object's own lifetime, is what rules that out.
//!
//! Vendored from the `better_any` crate (v0.2.0) by Konstantin Anisimov,
//! <https://github.com/rrevenantt/better_typeid>, dual licensed MIT OR Apache-2.0.
//! Reduced to what this runtime needs: the `Any` interoperability layer (`AnyExt`,
//! `TypeIdAdjuster`, `downcast_any_*`), the derive macro, `downcast_arc`/`downcast_move`,
//! `typeid_of`, and the blanket implementations for standard library wrapper types are all
//! dropped. What remains is the `Rc`/`Box`/`&`/`&mut` downcasts that match how this runtime
//! actually stores parse tree nodes, token factories and error strategies.

use std::any::{Any, TypeId};
use std::rc::Rc;

/// Indicates that this type can be substituted as a type parameter of another type
/// so that the resulting type can implement [`Tid`].
///
/// If you have no such generic types, use [`Tid`] everywhere and ignore this trait.
///
/// Only this trait is implemented on the user side; the others are blanket implementations
/// over `X: TidAble<'a>`.
///
/// Note that this trait interferes with object safety, so it should not be used as a super
/// trait when a trait object is needed. Formally it is still object safe, but a trait object
/// cannot be made without specifying the internal associated type, like
/// `dyn TidAble<'a, Static = SomeType>`, which makes the trait object useless.
///
/// # Safety
///
/// Soundness of downcasting relies on `Static` being a distinct type for every distinct
/// `Self`, and on `Self` having at most the single lifetime `'a`. Use the [`crate::tid!`] macro,
/// which guarantees both.
// The associated type allows the type id generator to be a private type,
// and allows the trait to be implemented for generic types.
// It has a lifetime and depends on `Tid` because it would be practically useless standalone:
// even though a type id could then be obtained for more types, any action based on it would
// be unsound without also checking lifetimes.
pub unsafe trait TidAble<'a>: Tid<'a> {
    /// Implementation detail
    #[doc(hidden)]
    type Static: ?Sized + Any;
}

/// Extension trait carrying the actual downcasting methods.
///
/// If `Self` is `Sized` then any of these calls is optimized to a no-op, because both `T` and
/// `Self` are known statically. That is useful in generic code that should behave differently
/// depending on the concrete type substituted for a type parameter.
pub trait TidExt<'a>: Tid<'a> {
    /// Returns true if the type behind `self` is `T`.
    fn is<T: Tid<'a>>(&self) -> bool {
        self.self_id() == T::id()
    }

    /// Attempts to downcast `self` to `T` behind a reference
    ///
    /// The `T: Tid<'a>` bound ties the target type to the trait object's own lifetime. That
    /// is the property which makes this sound and which [`Any`] could not provide: without
    /// it, a caller could name a longer-lived target type and launder a borrow of local data
    /// into a `&'static` reference.
    ///
    /// ```compile_fail
    /// use antlr4rust::{tid, Tid, TidExt};
    ///
    /// struct Borrowing<'a>(&'a str);
    /// tid!(Borrowing<'a>);
    /// trait Node<'a>: Tid<'a> {}
    /// tid! { impl<'a> TidAble<'a> for dyn Node<'a> + 'a }
    /// impl<'a> Node<'a> for Borrowing<'a> {}
    ///
    /// // rejected (E0521): naming `Borrowing<'static>` as the target requires `'a: 'static`
    /// fn launder<'a>(node: &(dyn Node<'a> + 'a)) -> Option<&'static str> {
    ///     node.downcast_ref::<Borrowing<'static>>().map(|it| it.0)
    /// }
    /// ```
    fn downcast_ref<'b, T: Tid<'a>>(&'b self) -> Option<&'b T> {
        // `Tid<'a>` is implemented only for types with lifetime `'a`,
        // so the cast back is safe because the lifetime invariant is preserved.
        if self.is::<T>() {
            Some(unsafe { &*(self as *const _ as *const T) })
        } else {
            None
        }
    }

    /// Attempts to downcast `self` to `T` behind a mutable reference
    fn downcast_mut<'b, T: Tid<'a>>(&'b mut self) -> Option<&'b mut T> {
        // see `downcast_ref`
        if self.is::<T>() {
            Some(unsafe { &mut *(self as *mut _ as *mut T) })
        } else {
            None
        }
    }

    /// Attempts to downcast `self` to `T` behind an [`Rc`]
    fn downcast_rc<T: Tid<'a>>(self: Rc<Self>) -> Result<Rc<T>, Rc<Self>> {
        if self.is::<T>() {
            unsafe { Ok(Rc::from_raw(Rc::into_raw(self) as *const _)) }
        } else {
            Err(self)
        }
    }

    /// Attempts to downcast `self` to `T` behind a [`Box`]
    fn downcast_box<T: Tid<'a>>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
        if self.is::<T>() {
            unsafe { Ok(Box::from_raw(Box::into_raw(self) as *mut _)) }
        } else {
            Err(self)
        }
    }
}

impl<'a, X: ?Sized + Tid<'a>> TidExt<'a> for X {}

/// Indicates that this type can be converted to a trait object carrying a type id while
/// preserving lifetime information. Extends [`Any`] functionality to types with a single
/// lifetime.
///
/// Use it as `dyn Tid<'a>`, or as a super trait when a trait object is needed.
/// Everywhere else use [`TidAble`].
///
/// The lifetime here is necessary to make `dyn Tid<'a> + 'a` invariant over `'a`.
///
/// # Safety
///
/// Implement through [`TidAble`] (via the [`crate::tid!`] macro) rather than directly: the blanket
/// implementation below is the only intended one.
pub unsafe trait Tid<'a>: 'a {
    /// Returns the type id of the type of `self`
    fn self_id(&self) -> TypeId;

    /// Returns the type id of this type
    fn id() -> TypeId
    where
        Self: Sized;
}

unsafe impl<'a, T: ?Sized + TidAble<'a>> Tid<'a> for T {
    #[inline]
    fn self_id(&self) -> TypeId {
        TypeId::of::<T::Static>()
    }

    #[inline]
    fn id() -> TypeId
    where
        Self: Sized,
    {
        TypeId::of::<T::Static>()
    }
}

/// Safe implementation interface for [`Tid`]/[`TidAble`].
///
/// It uses the syntax of a regular Rust `impl` block, but with parameters restricted enough to
/// be sound. In particular it is restricted to a single lifetime parameter per block, and
/// additional bounds must go in `where` clauses. In trivial cases just the type signature can
/// be used.
///
/// ```rust
/// # use antlr4rust::tid;
/// struct S;
/// tid!(S);
///
/// struct F<'a>(&'a str);
/// tid!(F<'a>);
///
/// struct Bar<'x, 'y, X, Y>(&'x str, &'y str, X, Y);
/// tid! { impl<'b, X, Y> TidAble<'b> for Bar<'b, 'b, X, Y> }
///
/// trait Test<'a> {}
/// tid! { impl<'b> TidAble<'b> for dyn Test<'b> + 'b }
/// ```
///
/// The implementation by default adds a `TidAble<'a>` bound on every generic parameter.
/// This can be opted out of by specifying a `'static` bound on the corresponding type
/// parameter. Note that because of declarative macro limitations it must be specified
/// directly on the type parameter and **not** in a `where` clause:
///
/// ```rust
/// # use antlr4rust::tid;
/// struct Test<'a, X: ?Sized>(&'a str, Box<X>);
/// tid! { impl<'a, X: 'static> Tid<'a> for Test<'a, X> where X: ?Sized }
/// ```
#[macro_export]
macro_rules! tid {

    ($struct: ident) => {
        unsafe impl<'a> $crate::TidAble<'a> for $struct {
            type Static = $struct;
        }
    };
    ($struct: ident < $lt: lifetime >) => {
        unsafe impl<'a> $crate::TidAble<'a> for $struct<'a> {
            type Static = $struct<'static>;
        }
    };
    // no static parameters case
    (impl <$lt:lifetime $(,$param:ident)*> $tr:ident<$lt2:lifetime> for $($struct: tt)+ ) => {
        $crate::tid!{ inner impl <$lt $(,$param)* static> $tr<$lt2> for $($struct)+  }
    };

    // inner submacro is used to check/fix/error on whether correct trait is being implemented
    (inner impl <$lt:lifetime $(,$param:ident)* static $( $static_param:ident)* > Tid<$lt2:lifetime> for $($struct: tt)+ ) => {
        $crate::tid!{ inner impl <$lt $(,$param)* static $( $static_param)*> TidAble<$lt2> for $($struct)+  }
    };
    (inner impl <$lt:lifetime $(,$param:ident)* static $( $static_param:ident)* > TidAble<$lt2:lifetime> for $($struct: tt)+ ) => {
        const _:() = {
            use core::marker::PhantomData;
            type __Alias<$lt $(,$param)* $(,$static_param)*>  = $crate::before_where!{ $($struct)+ };
            pub struct __TypeIdGenerator<$lt $(,$param:?Sized)* $(,$static_param:?Sized)*>
                (PhantomData<& $lt ()> $(,PhantomData<$param>)* $(,PhantomData<$static_param>)*);
            $crate::impl_block!{
                after where {  $($struct)+ }
                {unsafe impl<$lt $(,$param:$crate::TidAble<$lt>)* $(,$static_param: 'static)* > $crate::TidAble<$lt2> for __Alias<$lt $(,$param)* $(,$static_param)*>}

                {
                    type Static = __TypeIdGenerator<'static $(,$param::Static)* $(,$static_param)*>;
                }
            }
        };
    };
    (inner impl <$lt:lifetime $(,$param:ident)* static $( $static_param:ident)* > $tr:ident<$lt2:lifetime> for $($struct: tt)+ ) => {
        compile_error!{" wrong trait, should be TidAble or Tid "}
    };

    // temp submacro is used to separate 'static type parameters from other ones
    (temp $(,$param:ident)* static $(,$static_param:ident)* impl <$lt:lifetime , $token:ident : 'static $($tail: tt)+ ) => {
        $crate::tid!{ temp $(,$param)* static  $(,$static_param)* , $token  impl <$lt $($tail)+}
    };
    (temp $(,$param:ident)* static $(,$static_param:ident)* impl <$lt:lifetime , $token:ident $($tail: tt)+ ) => {
        $crate::tid!{ temp $(,$param)* ,$token static $(,$static_param)* impl <$lt $($tail)+ }
    };
    (temp $(,$param:ident)* static $(,$static_param:ident)* impl <$lt:lifetime> $($tail: tt)+ ) => {
        $crate::tid!{ inner impl <$lt $(,$param)* static $( $static_param)* > $($tail)+ }
    };
    ( impl $($tail: tt)+) => {
        $crate::tid!{ temp static impl $($tail)+ }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! before_where {
    (inner { $($processed:tt)* } where     $($tokens:tt)* ) => { $($processed)* };
    (inner { $($processed:tt)* } $token:tt $($tokens:tt)* ) => {
        $crate::before_where!(inner { $($processed)* $token }  $($tokens)*)
    };
    (inner { $($processed:tt)* } ) => { $($processed)* };
    ($($tokens:tt)*) => {$crate::before_where!(inner {} $($tokens)*)};
}

// creates the actual impl block while also extracting tokens after `where`
#[doc(hidden)]
#[macro_export]
macro_rules! impl_block {
    (
        after where {}
        {$($imp:tt)*}
        { $($block:tt)* }
    ) => {
        $($imp)*

        {
            $($block)*
        }
    };
    (
        after where { where $($bounds:tt)* }
        {$($imp:tt)*}
        { $($block:tt)* }
    ) => {
        $($imp)*
            where $($bounds)*
        {
            $($block)*
        }
    };
    (
        after where {$token:tt $($tokens:tt)*}
        {$($imp:tt)*}
        { $($block:tt)* }
    ) => {
        $crate::impl_block!{
            after where { $($tokens)*}
            {$($imp)*}
            { $($block)* }

        }
    };
}

/// Alias of the [`crate::tid!`] macro, for compatibility with the `better_any` naming.
pub use crate::tid as type_id;

// `better_any` also ships blanket implementations for `Box`, `Rc`, `RefCell`, `Cell`, `Arc`,
// `Mutex`, `RwLock`, `Vec`, `Option`, `Result`, `dyn Tid`, `&T` and `&mut T`. None of them are
// reachable from this runtime: every type that needs a type id here names itself through the
// `tid!` macro, and the wrapper types this runtime does use (`Box<dyn ErrorStrategy>`,
// `Rc<dyn ParserRuleContext>`, `Box<CommonToken>`, `&'input CommonToken`) either carry their
// own implementation or never need one. They were dropped rather than carried along; add back
// only the specific one a use case demands, since the orphan rule means downstream crates
// cannot write these themselves.

#[cfg(test)]
mod tests {
    use core::marker::PhantomData;

    use super::*;

    struct Plain;
    tid!(Plain);

    struct Borrowing<'a>(&'a str);
    tid!(Borrowing<'a>);

    struct Generic<'a, T>(PhantomData<(&'a str, T)>);
    tid! { impl<'a, T> TidAble<'a> for Generic<'a, T> }

    trait Node<'a>: Tid<'a> {}
    tid! { impl<'a> TidAble<'a> for dyn Node<'a> + 'a }
    impl<'a> Node<'a> for Borrowing<'a> {}
    impl<'a> Node<'a> for Plain {}

    #[test]
    fn distinct_types_have_distinct_ids() {
        assert_ne!(Plain::id(), Borrowing::id());
        assert_ne!(
            Generic::<'_, Plain>::id(),
            Generic::<'_, Borrowing<'_>>::id()
        );
    }

    #[test]
    fn downcast_ref_through_trait_object() {
        let text = String::from("input");
        let node: &(dyn Node<'_> + '_) = &Borrowing(&text);
        assert!(node.downcast_ref::<Borrowing<'_>>().is_some());
        assert!(node.downcast_ref::<Plain>().is_none());
    }

    #[test]
    fn downcast_rc_through_trait_object() {
        let text = String::from("input");
        let node: Rc<dyn Node<'_> + '_> = Rc::new(Borrowing(&text));
        assert!(node.clone().downcast_rc::<Plain>().is_err());
        let concrete = node.downcast_rc::<Borrowing<'_>>().ok().unwrap();
        assert_eq!(concrete.0, "input");
    }

    // The borrowed data outlives the trait object, which is what makes downcasting back to
    // `Borrowing<'a>` sound: `dyn Tid<'a>` is invariant in `'a`, so the lifetime cannot widen.
    #[test]
    fn downcast_preserves_lifetime() {
        fn extract<'a>(node: &(dyn Node<'a> + 'a)) -> Option<&'a str> {
            node.downcast_ref::<Borrowing<'a>>().map(|it| it.0)
        }
        let text = String::from("input");
        assert_eq!(extract(&Borrowing(&text)), Some("input"));
    }
}
