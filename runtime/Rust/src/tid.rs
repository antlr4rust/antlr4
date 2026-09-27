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
/// Implementors must ensure that `Static` identifies `Self` exactly: for every lifetime `'a`,
/// if `X: TidAble<'a>` and `Y: TidAble<'a>` have the same `Static`, then `X` and `Y` are the
/// same type, lifetimes included. The blanket [`Tid`] implementation, and through it every
/// downcast in [`TidExt`], relies on this, so violating it makes them type-confusable. That
/// requires both of the following. [`crate::tid!`] establishes them by construction, which is
/// why it is the only supported way to implement this trait.
///
/// 1. **Distinct types get distinct `Static`s**, both across implementations and across the
///    instantiations of one generic implementation. `tid!(Type)` uses `Type` itself and
///    `tid!(Type<'a>)` uses `Type<'static>`. The general form mints a fresh private witness
///    struct inside the invocation's own `const _: () = { .. }` block, and threads generic
///    parameters through as `P::Static`, or unchanged when they are `'static`, so different
///    instantiations stay distinct. A hand-written implementation must likewise pick a
///    `Static` that no other implementation uses, such as a private type of its own.
///
/// 2. **`Self` carries at most the single lifetime `'a`**, besides `'static`. The lifetime is
///    recovered purely from the `Tid<'a>` bound, because `Static` is `'static` and so cannot
///    tell apart types that differ in any lifetime: a type with a second, independent lifetime
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
            // SAFETY: `is` checked that `self_id` on `self` returned `T::id()`, where `Self`
            // and `T` are both `Tid<'a>` and `T` is sized. By the `Tid` contract, the value
            // behind `self` is then a `T`, lifetimes included. That holds whichever `self_id`
            // the call resolved to when `Self` is a `dyn Trait` (the concrete type's, through
            // the vtable, or `dyn Trait`'s own if it implements `TidAble`), because every
            // implementation is bound by that contract.
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
            // SAFETY: as in `downcast_ref`, the value behind `self` is a `T`. `Rc::from_raw`
            // takes a pointer returned by `Rc<U>::into_raw` (here `U` is `Self`, with the
            // global allocator) and requires `U`, or for an unsized `U` its data pointer, to
            // "have the same size and alignment as `T`", which holds because the value is a
            // `T`. The strong count is carried over unchanged: `into_raw` gives up this
            // handle's count and `from_raw` takes it back, so the count is neither leaked nor
            // double-decremented.
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
/// [`TidExt`] reinterprets a value as a `T` whenever `self_id` on it returns `T::id()`, so
/// implementors must ensure that such a match only happens for a `T`: for every lifetime `'a`,
/// every sized `T: Tid<'a>` and every `S: Tid<'a> + ?Sized`, if `self_id` called on an `&S`
/// returns `T::id()`, then the value behind that reference is a `T`, lifetimes included.
/// Otherwise entirely safe caller code gets type confusion.
///
/// For a single implementation, that means:
///
/// - `id` identifies `Self` exactly among all implementors for the same `'a`: base it on a
///   `'static` type that no other implementation uses, including as a `TidAble::Static`. A
///   [`TypeId`] only exists for `'static` types, so it cannot record any lifetime of `Self`:
///   `Self` may carry no lifetime but `'a`, which the `Tid<'a>` bound pins, and `'static`.
/// - `self_id` returns the `id` of the type of the value behind `self`, or an id that no sized
///   type's `id` returns.
///
/// The blanket implementation over [`TidAble`] meets this given the `TidAble` contract, and the
/// vtable of a `dyn Trait` with `Trait: Tid<'a>` carries the concrete type's `self_id`. Note
/// that the blanket implementation does *not* seal this trait: a hand-written
/// `unsafe impl Tid` for a type that does not implement [`TidAble`] is accepted by coherence,
/// and a wrong `self_id` there is enough to break every downcast. Implement [`TidAble`] through
/// the [`crate::tid!`] macro instead and let the blanket implementation supply `Tid`.
pub unsafe trait Tid<'a>: 'a {
    /// Returns the type id of the type of `self`
    fn self_id(&self) -> TypeId;

    /// Returns the type id of this type
    fn id() -> TypeId
    where
        Self: Sized;
}

// SAFETY: `Tid` requires that `self_id` on an `&S` can return a sized `T`'s `id` only if the
// value behind it is a `T`. This implementation returns the `TypeId` of the implementing type's
// `Static` from both methods, and equal `TypeId`s mean the same type ("a globally unique
// identifier for a type", per the std docs). So if `S` and `T` both implement `Tid` here, a
// match means `S::Static` and `T::Static` are the same type, hence `S` and `T` are the same
// type, lifetimes included, by the `TidAble` contract; `S` is then sized, and the value behind
// an `&S` is an `S`. If either implements `Tid` by hand instead, the ids cannot match, because
// that implementation may not reuse a `TidAble::Static`.
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
        // SAFETY: `Static` is `Self`, and the `Static: Any` bound makes this compile only for
        // a `'static` `Self`, so `Self` has no lifetime but `'static` (obligation 2). No other
        // implementation shares this `Static` (obligation 1): another `tid!` producing it
        // would implement `TidAble` for an overlapping type, which coherence rejects, and a
        // hand-written implementation may not reuse it.
        unsafe impl<'a> $crate::TidAble<'a> for $struct {
            type Static = $struct;
        }
    };
    // A type whose only parameter is one lifetime.
    ($struct: ident < $lt: lifetime >) => {
        // SAFETY: `Self` is `$struct<'a>`, instantiated at the trait's own `'a`, and this arm
        // matches no other parameter, so `Self` has no lifetime but `'a` and `'static`
        // (obligation 2). For a given `'a` that is a single type, and its `Static`,
        // `$struct<'static>`, is shared with no other implementation (obligation 1): another
        // `tid!` producing it would implement `TidAble` for an overlapping type, which
        // coherence rejects, and a hand-written implementation may not reuse it.
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
    // Obligation 1 (distinct `Static`s): `__TypeIdGenerator` is declared inside this
    // invocation's own `const _: () = { .. }` block, so every use of the macro mints a
    // distinct type that no other invocation can name — two different types can never share a
    // `Static`. Within one invocation, generic parameters are threaded through as
    // `$param::Static` (injective by induction on the same contract) and `'static`-bounded
    // parameters are passed through unchanged (injective via their own `TypeId`), so distinct
    // instantiations stay distinct.
    //
    // Obligation 2 (at most one lifetime): the matcher accepts exactly one lifetime, `$lt`,
    // and the impl is written for `__Alias<$lt, ..>`. Any free lifetime in the aliased type
    // must therefore be `$lt` or `'static`, and `$lt` is instantiated at the trait's
    // `$lt2`. Type parameters add none: non-`'static` ones are bound by `TidAble<$lt>`, so by
    // the same obligation they carry only `$lt`, and the others are `'static`. A second,
    // independent lifetime cannot be expressed through this arm.
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
