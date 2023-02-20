#![allow(unused_variables)]
#![allow(unused_must_use)]

use messages_macros::TransitiveTryFrom;

#[test]
fn try_from() {
    #[derive(TransitiveTryFrom)]
    #[transitive(B, C, D, E, F, G)] // impl TryFrom<G> for A
    struct A;
    #[derive(TransitiveTryFrom)]
    #[transitive_all(C, D, E, F, G)] // impl TryFrom<G> for E, D, C and B
    struct B;
    #[derive(TransitiveTryFrom)]
    #[transitive(D, E, F)] // impl TryFrom<F> for C
    #[transitive(F, G)] // impl TryFrom<G> for C => Since we already implement C -> F above, this works!
    struct C;
    struct D;
    struct E;
    struct F;
    struct G;

    impl TryFrom<G> for F {
        type Error = ();

        fn try_from(val: G) -> Result<Self, Self::Error> {
            Ok(F)
        }
    }

    impl TryFrom<F> for E {
        type Error = ();

        fn try_from(val: F) -> Result<Self, Self::Error> {
            Ok(E)
        }
    }

    impl TryFrom<E> for D {
        type Error = ();

        fn try_from(val: E) -> Result<Self, Self::Error> {
            Ok(D)
        }
    }

    impl TryFrom<D> for C {
        type Error = ();

        fn try_from(val: D) -> Result<Self, Self::Error> {
            Ok(C)
        }
    }

    impl TryFrom<C> for B {
        type Error = ();

        fn try_from(val: C) -> Result<Self, Self::Error> {
            Ok(B)
        }
    }

    impl TryFrom<B> for A {
        type Error = ();

        fn try_from(val: B) -> Result<Self, Self::Error> {
            Ok(A)
        }
    }

    A::try_from(G);

    B::try_from(D);
    B::try_from(E);
    B::try_from(F);
    B::try_from(G);

    C::try_from(F);
    C::try_from(G);
}
