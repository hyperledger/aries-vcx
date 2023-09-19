pub mod formats;
pub mod holder;
pub mod issuer;

mod messages {
    pub struct ProposeCredentialV2;
    pub struct OfferCredentialV2;
    pub struct RequestCredentialV2;
    pub struct IssueCredentialV2;
}
