use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use typed_builder::TypedBuilder;

/// https://w3c-ccg.github.io/DID-Linked-Resources/
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DidResource {
    pub content: Vec<u8>,
    pub metadata: DidResourceMetadata,
}

/// https://w3c-ccg.github.io/DID-Linked-Resources/
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, TypedBuilder)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct DidResourceMetadata {
    // FUTURE - could be a map according to spec
    /// A string or a map that conforms to the rules of RFC3986 URIs which SHOULD directly lead to
    /// a location where the resource can be accessed.
    /// For example:
    /// did:example:46e2af9a-2ea0-4815-999d-730a6778227c/resources/
    /// 0f964a80-5d18-4867-83e3-b47f5a756f02.
    pub resource_uri: String,
    /// A string that conforms to a method-specific supported unique identifier format.
    /// For example, a UUID: 46e2af9a-2ea0-4815-999d-730a6778227c.
    pub resource_collection_id: String,
    /// A string that uniquely identifies the resource.
    /// For example, a UUID: 0f964a80-5d18-4867-83e3-b47f5a756f02.
    pub resource_id: String,
    /// A string that uniquely names and identifies a resource. This property, along with the
    /// resourceType below, can be used to track version changes within a resource.
    pub resource_name: String,
    /// A string that identifies the type of resource. This property, along with the resourceName
    /// above, can be used to track version changes within a resource. Not to be confused with
    /// mediaType.
    pub resource_type: String,
    /// (Optional) A string that identifies the version of the resource.
    /// This property is provided by the client and can be any value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_version: Option<String>,
    /// (Optional) An array that describes alternative URIs for the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub also_known_as: Option<Vec<Value>>,
    /// A string that identifies the IANA-media type of the resource.
    pub media_type: String,
    // TODO - check datetime serializes into XML-date-time
    /// A string that identifies the time the resource was created, as an XML date-time.
    pub created: DateTime<Utc>,
    /// (Optional) A string that identifies the time the resource was updated, as an XML date-time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<DateTime<Utc>>,
    /// A string that may be used to prove that the resource has not been tampered with.
    pub checksum: String,
    /// (Optional) A string that identifies the previous version of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_version_id: Option<String>,
    /// (Optional) A string that identifies the next version of the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_version_id: Option<String>,
}
