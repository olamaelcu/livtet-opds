use std::collections::HashMap;

use livtet_opds_types::{Feed, OpdsError};

use crate::Client;

#[derive(Debug, Clone)]
pub struct SearchTemplate {
    pub template: String,
    pub variables: Vec<String>,
}

impl SearchTemplate {
    pub fn from_feed(feed: &Feed) -> Option<Self> {
        feed.search_template().map(|link| {
            let variables = extract_variables(&link.href);
            Self {
                template: link.href.clone(),
                variables,
            }
        })
    }

    pub fn expand(&self, params: &HashMap<String, String>) -> Result<String, OpdsError> {
        let result = self.template.clone();

        if let Some(start) = result.find('{') {
            let prefix = &result[..start];
            let template_vars = &result[start..];

            let trimmed = template_vars.trim_start_matches('{').trim_end_matches('}');
            if !trimmed.starts_with('?') && !trimmed.starts_with('&') {
                return Ok(result);
            }

            let operator = if trimmed.starts_with('?') { '?' } else { '&' };
            let var_list = &trimmed[1..];
            let var_names: Vec<&str> = var_list.split(',').collect();

            let mut query_parts = Vec::new();
            for (i, name) in var_names.iter().enumerate() {
                let name = name.trim();
                if let Some(value) = params.get(name) {
                    let separator = if i == 0 { operator } else { '&' };
                    let encoded = url_encode(value);
                    query_parts.push(format!("{}{}={}", separator, name, encoded));
                }
            }

            if query_parts.is_empty() {
                return Ok(prefix.to_string());
            }

            return Ok(format!("{}{}", prefix, query_parts.join("")));
        }

        Ok(result)
    }
}

fn extract_variables(href: &str) -> Vec<String> {
    let mut vars = Vec::new();
    if let Some(start) = href.find('{')
        && let Some(end) = href[start..].find('}')
    {
        let content = &href[start + 1..start + end];
        if let Some(stripped) = content
            .strip_prefix('?')
            .or_else(|| content.strip_prefix('&'))
        {
            for var in stripped.split(',') {
                vars.push(var.trim().to_string());
            }
        }
    }
    vars
}

fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            ' ' => out.push('+'),
            c if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') => out.push(c),
            c => {
                let mut buf = [0u8; 4];
                let bytes = c.encode_utf8(&mut buf).as_bytes();
                for b in bytes {
                    out.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    out
}

pub async fn search(client: &Client, base_path: &str, query: &str) -> Result<Feed, OpdsError> {
    let feed = client.fetch_feed(base_path).await?;
    let template = SearchTemplate::from_feed(&feed)
        .ok_or_else(|| OpdsError::MissingField("search template".to_string()))?;

    let mut params = HashMap::new();
    params.insert("query".to_string(), query.to_string());

    let url = template.expand(&params)?;
    let path = url::Url::parse(&url)
        .ok()
        .and_then(|u| {
            u.path()
                .strip_prefix(client.base_path())
                .map(|p| p.to_string())
        })
        .unwrap_or_else(|| url.clone());

    client.fetch_feed(&path).await
}
