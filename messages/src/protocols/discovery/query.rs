use crate::{
    a2a::{A2AMessage, MessageId},
    concepts::timing::Timing,
    timing_optional,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Query {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}

timing_optional!(Query);

impl Query {
    pub fn create() -> Query {
        Query::default()
    }

    pub fn set_query(mut self, query: Option<String>) -> Self {
        self.query = query;
        self
    }

    pub fn set_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }

    pub fn to_a2a_message(&self) -> A2AMessage {
        A2AMessage::Query(self.clone()) // TODO: THINK how to avoid clone
    }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
    use super::*;

    pub fn _query_string() -> String {
        String::from("https://didcomm.org/")
    }

    pub fn _comment() -> String {
        String::from("I'm wondering if we can...")
    }

    pub fn _query() -> Query {
        Query {
            id: MessageId::id(),
            query: Some(_query_string()),
            comment: Some(_comment()),
            timing: None,
        }
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use super::*;
    use crate::protocols::discovery::query::test_utils::{_comment, _query, _query_string};

    #[test]
    fn test_query_build_works() {
        let query: Query = Query::default()
            .set_query(Some(_query_string()))
            .set_comment(Some(_comment()));

        assert_eq!(_query(), query);
    }
}
