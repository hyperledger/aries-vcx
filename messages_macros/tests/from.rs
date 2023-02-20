#![allow(unused_variables)]
#![allow(unused_must_use)]

use messages_macros::TransitiveFrom;

#[test]
fn from() {
    #[derive(TransitiveFrom)]
    #[transitive(B, C, D, E, F, G)] // impl From<A> for G
    struct A;
    #[derive(TransitiveFrom)]
    #[transitive_all(C, D, E, F, G)] // impl From<B> for D, E, F and G
    struct B;
    #[derive(TransitiveFrom)]
    #[transitive(C, D, E, F)] // impl From<C> for F
    #[transitive(F, G)] // impl From<C> for G => Since we already implement C -> F above, this works!
    struct C;
    struct D;
    struct E;
    struct F;
    struct G;

    impl From<A> for B {
        fn from(val: A) -> B {
            B
        }
    }

    impl From<B> for C {
        fn from(val: B) -> C {
            C
        }
    }

    impl From<C> for D {
        fn from(val: C) -> D {
            D
        }
    }

    impl From<D> for E {
        fn from(val: D) -> E {
            E
        }
    }

    impl From<E> for F {
        fn from(val: E) -> F {
            F
        }
    }

    impl From<F> for G {
        fn from(val: F) -> G {
            G
        }
    }

    G::from(A);

    D::from(B);
    E::from(B);
    F::from(B);
    G::from(B);

    F::from(C);
    G::from(C);
}
