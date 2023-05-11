#[derive(Debug)]
pub struct Finished {
    pub(crate) credential_id: String,
}

impl Finished {
    pub fn new(credential_id: String) -> Self {
        Finished { credential_id }
    }
}
