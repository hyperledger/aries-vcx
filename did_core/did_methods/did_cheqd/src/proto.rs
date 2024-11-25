pub mod cheqd {
    pub mod did {
        pub mod v2 {
            tonic::include_proto!("cheqd.did.v2");
        }
    }
    pub mod resource {
        pub mod v2 {
            tonic::include_proto!("cheqd.resource.v2");
        }
    }
}

pub mod cosmos {
    pub mod base {
        pub mod query {
            pub mod v1beta1 {
                tonic::include_proto!("cosmos.base.query.v1beta1");
            }
        }
    }
}
