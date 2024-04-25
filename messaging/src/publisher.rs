use crate::errors::MessagingError;
use async_trait::async_trait;
use opentelemetry::Context;
use std::collections::HashMap;

#[cfg(feature = "mocks")]
use mockall::*;

#[derive(Clone)]
pub enum HeaderValues {
    ShortString(String),
    LongString(String),
    Int(i8),
    LongInt(i32),
    LongLongInt(i64),
    Uint(u8),
    LongUint(u32),
    LongLongUint(u64),
}

impl From<HeaderValues> for String {
    fn from(val: HeaderValues) -> Self {
        match val {
            HeaderValues::ShortString(v) => v,
            HeaderValues::LongString(v) => v,
            HeaderValues::Int(v) => v.to_string(),
            HeaderValues::LongInt(v) => v.to_string(),
            HeaderValues::LongLongInt(v) => v.to_string(),
            HeaderValues::Uint(v) => v.to_string(),
            HeaderValues::LongUint(v) => v.to_string(),
            HeaderValues::LongLongUint(v) => v.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct PublishMessage {
    pub from: String,
    pub to: String,
    pub key: String,
    pub msg_type: String,
    pub data: Box<[u8]>,
    pub headers: Option<HashMap<String, HeaderValues>>,
}

impl PublishMessage {
    pub fn new<T>(
        from: T,
        to: T,
        key: T,
        msg_type: T,
        data: &[u8],
        headers: Option<HashMap<String, HeaderValues>>,
    ) -> Self
    where
        T: Into<String>,
    {
        PublishMessage {
            to: to.into(),
            from: from.into(),
            key: key.into(),
            msg_type: msg_type.into(),
            data: data.into(),
            headers,
        }
    }
}

#[cfg_attr(feature = "mocks", automock)]
#[async_trait]
pub trait Publisher: Send + Sync {
    async fn publish(&self, ctx: &Context, msg: &PublishMessage) -> Result<(), MessagingError>;
}
