use std::collections::HashMap;

use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OpdsAuth {
    None,

    Basic { username: String, password: String },

    Bearer { access_token: String },
}

impl OpdsAuth {
    pub fn apply(&self, builder: RequestBuilder) -> RequestBuilder {
        match self {
            OpdsAuth::None => builder,
            OpdsAuth::Basic { username, password } => builder.basic_auth(username, Some(password)),
            OpdsAuth::Bearer { access_token } => builder.bearer_auth(access_token),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExtraHeaders(pub HashMap<String, String>);

impl ExtraHeaders {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.0.insert(name.into(), value.into());
    }

    pub fn apply(&self, mut builder: RequestBuilder) -> RequestBuilder {
        for (k, v) in &self.0 {
            builder = builder.header(k, v);
        }
        builder
    }
}
