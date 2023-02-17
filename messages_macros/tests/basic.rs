#![allow(unused_variables)]

use messages_macros::TransitiveFrom;

#[test]
fn basic() {
    #[derive(TransitiveFrom)]
    #[transitive(B, C, D, E, F)] // impl From<A> for F
    struct A;
    #[derive(TransitiveFrom)]
    #[transitive(C, D, E)] // impl From<B> for E
    #[transitive(E, F)] // impl From<B> for F => Since we already implement B -> E above, this works!
    struct B;
    struct C;
    struct D;
    struct E;
    struct F;

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

    let a = A;
    let f = F::from(a);

    let b = B;
    let e = E::from(b);
    let f = F::from(B);
}
