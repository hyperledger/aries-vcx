use cosmrs::tx::MsgProto;

pub mod cheqdid {
    pub mod cheqdnode {
        pub mod cheqd {
            pub mod v1 {
                include!(concat!(
                env!("OUT_DIR"),
                "/prost/cheqdid.cheqdnode.cheqd.v1.rs"
                ));
            }
        }
    }
}

impl MsgProto for cheqdid::cheqdnode::cheqd::v1::MsgCreateDid {
    const TYPE_URL: &'static str = "/cheqdid.cheqdnode.cheqd.v1.MsgCreateDid";
}

impl MsgProto for cheqdid::cheqdnode::cheqd::v1::MsgUpdateDid {
    const TYPE_URL: &'static str = "/cheqdid.cheqdnode.cheqd.v1.MsgUpdateDid";
}
