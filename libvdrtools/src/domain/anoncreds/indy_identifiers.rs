use lazy_static::lazy_static;
use regex::Regex;

use super::{
    super::crypto::did::DidValue, credential_definition::CredentialDefinitionId,
    revocation_registry_definition::CL_ACCUM, schema::SchemaId,
};

const NAMESPACE_RE: &str = r"[a-z][a-z0-9_:-]*";
const DID_RE: &str = r"[1-9A-HJ-NP-Za-km-z]*"; //base58
const SCHEMA_TYPE: &str = super::schema::SchemaId::PREFIX;
const SCHEMA_NAME_RE: &str = r"[^/]*";
const SCHEMA_VER_RE: &str = r"[^/]*";
const SCHEMA_SEQ_NO_RE: &str = r"[0-9]*";

lazy_static! {
    static ref SCHEMA_RE: String =
        format!("(did:indy(:{NAMESPACE_RE})?:{DID_RE}){SCHEMA_TYPE}({SCHEMA_NAME_RE})/({SCHEMA_VER_RE})");
    static ref SCHEMA_REF_RE: String = format!("({SCHEMA_SEQ_NO_RE}|{})", *SCHEMA_RE);
}
const CREDDEF_TYPE: &str = super::credential_definition::CredentialDefinitionId::PREFIX;
const CREDDEF_TAG_RE: &str = r".*";

pub fn try_parse_indy_schema_id(id: &str) -> Option<(String, String, String)> {
    let id_re = format!("^{}$", *SCHEMA_RE);
    let id_re = Regex::new(id_re.as_str()).unwrap();
    if let Some(captures) = id_re.captures(id) {
        trace!("try_parse_indy_schema_id: captures {:?}", captures);
        if let (Some(did), Some(name), Some(ver)) = (captures.get(1), captures.get(3), captures.get(4)) {
            return Some((
                did.as_str().to_owned(),
                name.as_str().to_owned(),
                ver.as_str().to_owned(),
            ));
        }
    }
    None
}

pub fn try_parse_indy_creddef_id(id: &str) -> Option<(String, String, String)> {
    let schema_ref_re = &*SCHEMA_REF_RE;
    let id_re = format!("^(did:indy(:{NAMESPACE_RE})?:{DID_RE}){CREDDEF_TYPE}({schema_ref_re})/({CREDDEF_TAG_RE})$");
    let id_re = Regex::new(id_re.as_str()).unwrap();

    if let Some(captures) = id_re.captures(id) {
        trace!("try_parse_indy_creddef_id: captures {:?}", captures);
        if let (Some(did), Some(seq_no), Some(tag)) = (captures.get(1), captures.get(3), captures.get(9)) {
            return Some((
                did.as_str().to_owned(),
                seq_no.as_str().to_owned(),
                tag.as_str().to_owned(),
            ));
        }
    }

    None
}

pub fn try_parse_indy_rev_reg(id: &str) -> Option<(DidValue, CredentialDefinitionId, String, String)> {
    let creddef_name_re = r"[^/]*";
    let tag_re = r"[^/]*";
    let schema_ref_re = &*SCHEMA_REF_RE;
    let id_re = format!(
        "^(did:indy(:{NAMESPACE_RE})?:{DID_RE})/anoncreds/v0/REV_REG_DEF/{schema_ref_re}/({creddef_name_re})/\
         ({tag_re})$"
    );
    let id_re = Regex::new(id_re.as_str()).unwrap();

    if let Some(captures) = id_re.captures(id) {
        trace!("try_parse_indy_rev_reg: captures {:?}", captures);
        if let (Some(did), Some(schema_id), Some(creddef_name), Some(tag)) =
            (captures.get(1), captures.get(3), captures.get(8), captures.get(9))
        {
            let did = DidValue(did.as_str().to_owned());
            let schema_id = SchemaId(schema_id.as_str().to_owned());
            let creddef_id = CredentialDefinitionId::new(
                &did,
                &schema_id,
                super::credential_definition::CL_SIGNATURE_TYPE,
                creddef_name.as_str(),
            )
            .ok()?;
            return Some((did, creddef_id, CL_ACCUM.to_owned(), tag.as_str().to_owned()));
        }
    }

    None
}

#[test]
fn test_try_parse_valid_indy_creddefid_works_for_sub_ledger() {
    let (did, schema_seq_no, tag) =
        try_parse_indy_creddef_id("did:indy:sovrin:5nDyJVP1NrcPAttP3xwMB9/anoncreds/v0/CLAIM_DEF/56495/npdb").unwrap();
    assert_eq!(did, "did:indy:sovrin:5nDyJVP1NrcPAttP3xwMB9".to_owned());
    assert_eq!(schema_seq_no, "56495".to_owned());
    assert_eq!(tag, "npdb".to_owned());
}

#[test]
fn test_try_parse_valid_indy_creddefid_works() {
    let (did, schema_seq_no, tag) =
        try_parse_indy_creddef_id("did:indy:NcYxiDXkpYi6ov5FcYDi1e/anoncreds/v0/CLAIM_DEF/1/tag").unwrap();
    assert_eq!(did, "did:indy:NcYxiDXkpYi6ov5FcYDi1e".to_owned());
    assert_eq!(schema_seq_no, "1".to_owned());
    assert_eq!(tag, "tag".to_owned());

    let (did, schema_ref, tag) = try_parse_indy_creddef_id(
        "did:indy:NcYxiDXkpYi6ov5FcYDi1e/anoncreds/v0/CLAIM_DEF/did:indy:NcYxiDXkpYi6ov5FcYDi1e/anoncreds/v0/SCHEMA/\
         gvt/1.0/tag",
    )
    .unwrap();
    assert_eq!(did, "did:indy:NcYxiDXkpYi6ov5FcYDi1e".to_owned());
    assert_eq!(
        schema_ref,
        "did:indy:NcYxiDXkpYi6ov5FcYDi1e/anoncreds/v0/SCHEMA/gvt/1.0".to_owned()
    );
    assert_eq!(tag, "tag".to_owned());
}

#[test]
fn test_try_parse_valid_indy_revreg_works() {
    let (did, creddef, _, tag) =
        try_parse_indy_rev_reg("did:indy:NcYxiDXkpYi6ov5FcYDi1e/anoncreds/v0/REV_REG_DEF/1/creddef_name/TAG1").unwrap();
    assert_eq!(did.0, "did:indy:NcYxiDXkpYi6ov5FcYDi1e".to_owned());
    assert_eq!(
        creddef.0,
        "did:indy:NcYxiDXkpYi6ov5FcYDi1e/anoncreds/v0/CLAIM_DEF/1/creddef_name".to_owned()
    );
    assert_eq!(tag, "TAG1".to_owned());

    let (did, creddef, _, tag) = try_parse_indy_rev_reg(
        "did:indy:NcYxiDXkpYi6ov5FcYDi1e/anoncreds/v0/REV_REG_DEF/did:indy:NcYxiDXkpYi6ov5FcYDi1e/anoncreds/v0/SCHEMA/\
         gvt/1.0/creddef_name/TAG1",
    )
    .unwrap();
    assert_eq!(did.0, "did:indy:NcYxiDXkpYi6ov5FcYDi1e".to_owned());
    assert_eq!(
        creddef.0,
        "did:indy:NcYxiDXkpYi6ov5FcYDi1e/anoncreds/v0/CLAIM_DEF/did:indy:NcYxiDXkpYi6ov5FcYDi1e/anoncreds/v0/SCHEMA/\
         gvt/1.0/creddef_name"
            .to_owned()
    );
    assert_eq!(tag, "TAG1".to_owned());
}
