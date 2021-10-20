use crate::messages::proof_presentation::presentation_request::PresentationRequestData;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct PresentationRequestSet {
    pub presentation_request_data: PresentationRequestData,
}

impl PresentationRequestSet {
    pub fn new(presentation_request_data: PresentationRequestData) -> Self {
        Self { presentation_request_data }
    }
}
