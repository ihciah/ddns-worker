//! Fork part of code from crate cloudflare because its wasm support is not well.
#![allow(unused)]

use std::{
    borrow::Cow,
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr},
};

use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::value::Value as JValue;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "type")]
#[allow(clippy::upper_case_acronyms)]
pub enum DnsContent<'a> {
    A { content: Ipv4Addr },
    AAAA { content: Ipv6Addr },
    CNAME { content: &'a str },
    NS { content: &'a str },
    MX { content: &'a str, priority: u16 },
    TXT { content: &'a str },
    SRV { content: &'a str },
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "type")]
#[allow(clippy::upper_case_acronyms)]
pub enum DnsContentOwned {
    A { content: Ipv4Addr },
    AAAA { content: Ipv6Addr },
    CNAME { content: String },
    NS { content: String },
    MX { content: String, priority: u16 },
    TXT { content: String },
    SRV { content: String },
}

#[derive(Deserialize, Debug)]
pub struct DnsRecord {
    /// DNS record name
    pub name: String,
    /// Time to live for DNS record. Value of 1 is 'automatic'
    pub ttl: u32,
    /// When the record was last modified
    pub modified_on: DateTime<Utc>,
    /// When the record was created
    pub created_on: DateTime<Utc>,
    /// Whether this record can be modified/deleted (true means it's managed by Cloudflare)
    pub proxiable: bool,
    /// Type of the DNS record that also holds the record value
    #[serde(flatten)]
    pub content: DnsContentOwned,
    /// DNS record identifier tag
    pub id: String,
    /// Whether the record is receiving the performance and security benefits of Cloudflare
    pub proxied: bool,
}

pub trait ApiResult: DeserializeOwned + std::fmt::Debug {}

impl ApiResult for DnsRecord {}
impl ApiResult for Vec<DnsRecord> {}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ListDnsRecordsOrder {
    Type,
    Name,
    Content,
    Ttl,
    Proxied,
}

#[derive(Serialize, Clone, Debug)]
pub enum OrderDirection {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

/// Used as a parameter to API calls that search for a resource (e.g. DNS records).
/// Tells the API whether to return results that match all search requirements or at least one (any).
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SearchMatch {
    /// Match all search requirements
    All,
    /// Match at least one search requirement
    Any,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug, Default)]
pub struct ListDnsRecordsParams<'a> {
    #[serde(flatten)]
    pub record_type: Option<DnsContent<'a>>,
    pub name: Option<&'a str>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub order: Option<ListDnsRecordsOrder>,
    pub direction: Option<OrderDirection>,
    #[serde(rename = "match")]
    pub search_match: Option<SearchMatch>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug)]
pub struct UpdateDnsRecordParams<'a> {
    /// Time to live for DNS record. Value of 1 is 'automatic'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u32>,
    /// Whether the record is receiving the performance and security benefits of Cloudflare
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied: Option<bool>,
    /// DNS record name
    pub name: &'a str,
    /// Type of the DNS record that also holds the record value
    #[serde(flatten)]
    pub content: DnsContent<'a>,
}

#[derive(Clone, Debug)]
pub enum Credentials {
    UserAuthKey { email: String, key: String },
    UserAuthToken { token: String },
    Service { key: String },
}

impl Credentials {
    pub fn headers(&self) -> Vec<(&'static str, Cow<'_, str>)> {
        match self {
            Self::UserAuthKey { email, key } => {
                vec![
                    ("X-Auth-Email", Cow::Borrowed(email.as_str())),
                    ("X-Auth-Key", Cow::Borrowed(key.as_str())),
                ]
            }
            Self::UserAuthToken { token } => {
                vec![(
                    "Authorization",
                    Cow::Owned(format!("Bearer {}", token.as_str())),
                )]
            }
            Self::Service { key } => vec![("X-Auth-User-Service-Key", Cow::Borrowed(key.as_str()))],
        }
    }
}

/// Note that APIError's `eq` implementation only compares `code` and `message`.
/// It does NOT compare the `other` values.
#[derive(Deserialize, Debug)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
    #[serde(flatten)]
    pub other: HashMap<String, JValue>,
}

/// Note that APIErrors's `eq` implementation only compares `code` and `message`.
/// It does NOT compare the `other` values.
#[derive(Deserialize, Debug, Default)]
pub struct ApiErrors {
    #[serde(flatten)]
    pub other: HashMap<String, JValue>,
    pub errors: Vec<ApiError>,
}

impl PartialEq for ApiErrors {
    fn eq(&self, other: &Self) -> bool {
        self.errors == other.errors
    }
}

impl PartialEq for ApiError {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code && self.message == other.message
    }
}

impl Eq for ApiError {}
impl Eq for ApiErrors {}
impl std::error::Error for ApiError {}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error {}: {}", self.code, self.message)
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ApiSuccess<ResultType> {
    pub result: ResultType,
    pub result_info: Option<JValue>,
    pub messages: JValue,
    pub errors: Vec<ApiError>,
}
