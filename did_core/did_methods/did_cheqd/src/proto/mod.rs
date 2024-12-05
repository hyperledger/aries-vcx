//! module structure wrapper over the generated proto types

pub mod cheqd {
    pub mod did {
        pub mod v2 {
            include!("cheqd.did.v2.rs");
        }
    }
    pub mod resource {
        pub mod v2 {
            include!("cheqd.resource.v2.rs");
        }
    }
}

pub mod cosmos {
    pub mod base {
        pub mod query {
            pub mod v1beta1 {
                include!("cosmos.base.query.v1beta1.rs");
            }
        }
    }
}
