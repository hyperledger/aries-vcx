use diddoc_legacy::aries::diddoc::AriesDidDoc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InviterComplete {
    pub(crate) did_doc: AriesDidDoc,
}
