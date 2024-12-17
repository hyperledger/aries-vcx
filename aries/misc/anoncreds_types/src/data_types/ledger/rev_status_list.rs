use crate::{
    cl::{Accumulator, RevocationRegistry as CryptoRevocationRegistry},
    data_types::{
        identifiers::{issuer_id::IssuerId, rev_reg_def_id::RevocationRegistryDefinitionId},
        ledger::rev_reg::RevocationRegistry,
    },
    Result,
};

/// Data model for the revocation status list as defined in the [Anoncreds V1.0
/// specification](https://hyperledger.github.io/anoncreds-spec/#creating-the-initial-revocation-status-list-object)
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationStatusList {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev_reg_def_id: Option<RevocationRegistryDefinitionId>,
    pub issuer_id: IssuerId,
    #[serde(with = "serde_revocation_list")]
    pub revocation_list: bitvec::vec::BitVec,
    #[serde(
        rename = "currentAccumulator",
        alias = "accum",
        skip_serializing_if = "Option::is_none"
    )]
    pub accum: Option<Accumulator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
}

impl From<&RevocationStatusList> for Option<CryptoRevocationRegistry> {
    fn from(value: &RevocationStatusList) -> Self {
        value.accum.map(From::from)
    }
}

impl From<&RevocationStatusList> for Option<RevocationRegistry> {
    fn from(value: &RevocationStatusList) -> Self {
        value.accum.map(|registry| RevocationRegistry {
            value: registry.into(),
        })
    }
}

impl RevocationStatusList {
    pub const fn state(&self) -> &bitvec::vec::BitVec {
        &self.revocation_list
    }

    pub fn accum(&self) -> Option<Accumulator> {
        self.accum
    }

    pub fn set_registry(&mut self, registry: CryptoRevocationRegistry) -> Result<()> {
        self.accum = Some(registry.accum);
        Ok(())
    }

    pub fn new(
        rev_reg_def_id: Option<&str>,
        issuer_id: IssuerId,
        revocation_list: bitvec::vec::BitVec,
        registry: Option<CryptoRevocationRegistry>,
        timestamp: Option<u64>,
    ) -> Result<Self> {
        Ok(Self {
            rev_reg_def_id: rev_reg_def_id
                .map(RevocationRegistryDefinitionId::new)
                .transpose()?,
            issuer_id,
            revocation_list,
            accum: registry.map(|r| r.accum),
            timestamp,
        })
    }
}

pub mod serde_revocation_list {
    use bitvec::vec::BitVec;
    use serde::{
        de::{Deserializer, Error as DeError, SeqAccess, Visitor},
        ser::{SerializeSeq, Serializer},
    };

    pub fn serialize<S>(state: &bitvec::vec::BitVec, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = s.serialize_seq(Some(state.len()))?;
        for element in state {
            let e = i32::from(*element);
            seq.serialize_element(&e)?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<bitvec::vec::BitVec, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct JsonBitStringVisitor;

        impl<'de> Visitor<'de> for JsonBitStringVisitor {
            type Value = bitvec::vec::BitVec;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    formatter,
                    "a seq containing revocation state, i.e. [1, 0, 1]"
                )
            }

            fn visit_seq<S>(self, mut v: S) -> Result<Self::Value, S::Error>
            where
                S: SeqAccess<'de>,
            {
                // TODO: do we have a min size for this?
                let mut bv = BitVec::with_capacity(v.size_hint().unwrap_or_default());
                while let Some(ele) = v.next_element()? {
                    match ele {
                        0 => bv.push(false),
                        1 => bv.push(true),
                        _ => {
                            return Err(S::Error::custom("invalid revocation state"));
                        }
                    }
                }
                Ok(bv)
            }
        }
        deserializer.deserialize_seq(JsonBitStringVisitor)
    }
}

#[cfg(test)]
mod rev_reg_tests {
    use bitvec::prelude::*;

    use super::*;

    const REVOCATION_LIST: &str = r#"
        {
            "revRegDefId": "reg",
            "revocationList": [1, 1, 1, 1],
            "issuerId": "mock:uri",
            "currentAccumulator":  "1 1379509F4D411630D308A5ABB4F422FCE6737B330B1C5FD286AA5C26F2061E60 1 235535CC45D4816C7686C5A402A230B35A62DDE82B4A652E384FD31912C4E4BB 1 0C94B61595FCAEFC892BB98A27D524C97ED0B7ED1CC49AD6F178A59D4199C9A4 1 172482285606DEE8500FC8A13E6A35EC071F8B84F0EB4CD3DD091C0B4CD30E5E 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000",
			 "timestamp": 1234
        }"#;

    const REVOCATION_LIST_WITHOUT_ISSUER_ID: &str = r#"
        {
            "revRegDefId": "reg",
            "revocationList": [1, 1, 1, 1],
            "currentAccumulator":  "1 1379509F4D411630D308A5ABB4F422FCE6737B330B1C5FD286AA5C26F2061E60 1 235535CC45D4816C7686C5A402A230B35A62DDE82B4A652E384FD31912C4E4BB 1 0C94B61595FCAEFC892BB98A27D524C97ED0B7ED1CC49AD6F178A59D4199C9A4 1 172482285606DEE8500FC8A13E6A35EC071F8B84F0EB4CD3DD091C0B4CD30E5E 2 095E45DDF417D05FB10933FFC63D474548B7FFFF7888802F07FFFFFF7D07A8A8 1 0000000000000000000000000000000000000000000000000000000000000000",
			 "timestamp": 1234
        }"#;

    #[test]
    fn json_rev_list_can_be_deserialized() {
        let des = serde_json::from_str::<RevocationStatusList>(REVOCATION_LIST).unwrap();
        let expected_state = bitvec![1;4];
        assert_eq!(des.state(), &expected_state);
    }

    #[test]
    fn json_rev_list_can_not_be_deserialized_without_issuer_id() {
        let res = serde_json::from_str::<RevocationStatusList>(REVOCATION_LIST_WITHOUT_ISSUER_ID);
        assert!(res.is_err());
    }

    #[test]
    fn test_revocation_list_roundtrip_serde() {
        let des_from_json = serde_json::from_str::<RevocationStatusList>(REVOCATION_LIST).unwrap();
        let ser = serde_json::to_string(&des_from_json).unwrap();
        let des = serde_json::from_str::<RevocationStatusList>(&ser).unwrap();
        let ser2 = serde_json::to_string(&des).unwrap();
        assert_eq!(ser, ser2)
    }
}
