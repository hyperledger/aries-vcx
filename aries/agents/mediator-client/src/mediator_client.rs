use messages::msg_fields::protocols::out_of_band::invitation::Invitation as OOBInvitation;
use reqwest::{self, Url};

use crate::errors::MediatorClientResult;
pub struct MediatorClient {
    url: Url,
    client: reqwest::Client,
}

impl MediatorClient {
    pub fn new(url: &str) -> MediatorClientResult<Self> {
        Ok(Self {
            url: url.parse()?,
            client: reqwest::Client::new(),
        })
    }

    pub async fn register(&self) -> MediatorClientResult<OOBInvitation> {
        let endpoint = self.url.join("/register.json")?;

        Ok(self.client.get(endpoint).send().await?.json().await?)
    }
}
