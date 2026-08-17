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
/// Implementors must guarantee both of the following. [`crate::tid!`] establishes them by
/// construction, which is why it is the only supported way to implement this trait.
///
/// 1. **`Static` is injective.** Two types that are not the same type constructor must not
///    map to the same `Static`. Every [`crate::tid!`] invocation mints a fresh private
///    witness struct inside its own `const _: () = { .. }` block, so no two invocations can
///    collide, and generic parameters are threaded through as `P::Static` so they stay
///    distinguishable. Violating this makes every downcast in [`TidExt`] type-confusable.
///
/// 2. **`Self` carries at most the single lifetime `'a`.** `Static` is `Self` with `'a`
///    replaced by `'static`, so the type id says nothing about lifetimes; the lifetime is
///    recovered purely from the `Tid<'a>` bound. A type with a second, independent lifetime
///    would have that lifetime erased with nothing to recover it from. Unifying several
///    lifetime parameters to the same `'a` (`tid! { impl<'a> TidAble<'a> for T<'a, 'a> }`)
///    is fine, because then there is only one lifetime to recover.
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
        if self.is::<T>() {
            // SAFETY: `is` compared the concrete type's `self_id` (a virtual call, so it is
            // the id of the value actually behind `self`, not of `Self`) against `T::id()`.
            // By the `TidAble` injectivity contract, equal ids mean the same type
            // constructor; by the `Tid<'a>` bound shared by `Self` and `T`, both are
            // instantiated at the same `'a`. Same constructor plus same lifetime is the same
            // type, so `T` is the concrete type and reinterpreting is valid.
            //
            // `Self` may be `?Sized` (typically `dyn ParserRuleContext<'a>`) while `T` is
            // always `Sized`; the cast drops the pointer metadata and keeps the data address,
            // which is the address of the `T` itself. The returned reference borrows `self`
            // for `'b`, so no lifetime is extended.
            Some(unsafe { &*(self as *const _ as *const T) })
        } else {
            None
        }
    }

    /// Attempts to downcast `self` to `T` behind a mutable reference
    fn downcast_mut<'b, T: Tid<'a>>(&'b mut self) -> Option<&'b mut T> {
        if self.is::<T>() {
            // SAFETY: as in `downcast_ref` for why `T` is the concrete type. Uniqueness of
            // the resulting `&mut T` follows from the `&'b mut self` receiver: it is the only
            // live reference to the value for `'b`, and it is consumed to produce this one.
            Some(unsafe { &mut *(self as *mut _ as *mut T) })
        } else {
            None
        }
    }

    /// Attempts to downcast `self` to `T` behind an [`Rc`]
    fn downcast_rc<T: Tid<'a>>(self: Rc<Self>) -> Result<Rc<T>, Rc<Self>> {
        if self.is::<T>() {
            // SAFETY: as in `downcast_ref`, `T` is the concrete type behind `self`, so the
            // allocation really is an `RcBox<T>`. `Rc::from_raw` recovers the box start by
            // subtracting the offset of the value field within `RcBox<T>`, which is the same
            // offset `Rc::into_raw` added, because it is the same `T`. The strong count is
            // carried over unchanged: `into_raw` forgets one owner and `from_raw` takes it
            // back, so the count is neither leaked nor double-decremented.
            unsafe { Ok(Rc::from_raw(Rc::into_raw(self) as *const _)) }
        } else {
            Err(self)
        }
    }

    /// Attempts to downcast `self` to `T` behind a [`Box`]
    fn downcast_box<T: Tid<'a>>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
        if self.is::<T>() {
            // SAFETY: as in `downcast_ref`, `T` is the concrete type behind `self`, so the
            // allocation was made for a `T` and `Box::from_raw` will free it with the same
            // layout `Box::into_raw` gave up. Ownership transfers exactly once.
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
/// `self_id` and `id` must both return the type id of `Self`'s own type, and must agree with
/// each other. [`TidExt`] treats a match between `self_id()` and `T::id()` as proof that the
/// value behind the reference really is a `T` and reinterprets its bytes accordingly, so an
/// implementation that returns some other type's id causes type confusion in entirely safe
/// caller code.
///
/// Note that the blanket implementation over [`TidAble`] does *not* seal this trait: a
/// hand-written `unsafe impl Tid` for a type that does not implement [`TidAble`] is accepted
/// by coherence, and a wrong `self_id` there is enough to break every downcast. Implement
/// [`TidAble`] through the [`crate::tid!`] macro instead and let the blanket implementation
/// supply `Tid`.
pub unsafe trait Tid<'a>: 'a {
    /// Returns the type id of the type of `self`
    fn self_id(&self) -> TypeId;

    /// Returns the type id of this type
    fn id() -> TypeId
    where
        Self: Sized;
}

// SAFETY: `Tid` requires `self_id`/`id` to return the id of `Self`'s own type and to agree.
// Both are defined here as `TypeId::of::<T::Static>()` — literally the same expression — so
// they agree by construction, and they identify `Self` exactly as long as `T::Static` is
// injective, which is obligation 1 of the `TidAble` contract.
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

    // A plain type with no parameters at all.
    ($struct: ident) => {
        // SAFETY: obligation 1 (injective `Static`) holds because `Static` is the type
        // itself, and distinct types have distinct `TypeId`s. Obligation 2 (at most one
        // lifetime) holds vacuously: this arm only matches a bare identifier, so the type has
        // no lifetime parameter. The `Static: Any` bound rejects a non-`'static` type here.
        unsafe impl<'a> $crate::TidAble<'a> for $struct {
            type Static = $struct;
        }
    };
    // A type whose only parameter is one lifetime.
    ($struct: ident < $lt: lifetime >) => {
        // SAFETY: obligation 1 holds because `Static` is `$struct<'static>`, and distinct
        // type constructors give distinct `TypeId`s at the same argument. Obligation 2 holds
        // because this arm only matches a single lifetime and no type parameters, and the
        // impl instantiates it at the trait's own `'a`.
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
    // The general case. SAFETY for the `unsafe impl` produced below:
    //
    // Obligation 1 (injective `Static`): `__TypeIdGenerator` is declared inside this
    // invocation's own `const _: () = { .. }` block, so every use of the macro mints a
    // distinct type that no other invocation can name — two different types can never share a
    // `Static`. Within one invocation, generic parameters are threaded through as
    // `$param::Static` (injective by induction on the same contract) and `'static`-bounded
    // parameters are passed through unchanged (injective via their own `TypeId`), so distinct
    // instantiations stay distinct.
    //
    // Obligation 2 (at most one lifetime): the matcher accepts exactly one lifetime, `$lt`,
    // and the impl is written for `__Alias<$lt, ..>`. Any lifetime appearing in the aliased
    // type must therefore be `$lt` itself, and `$lt` is instantiated at the trait's `$lt2`.
    // A second, independent lifetime cannot be expressed through this arm.
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
