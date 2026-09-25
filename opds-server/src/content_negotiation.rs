//! Content negotiation helpers for OPDS feeds.
//!
//! Determines whether a client should receive OPDS 2 (JSON) or OPDS 1
//! (Atom) based on the `Accept` header.

use mime::{APPLICATION_JSON, Mime, TEXT_XML};

pub const MIME_OPDS_JSON: &str = "application/opds+json";
pub const MIME_ATOM: &str = "application/atom+xml";

/// Renderer selected by content negotiation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Renderer {
    Json,
    Atom,
}

impl Renderer {
    pub fn content_type(&self) -> &'static str {
        match self {
            Renderer::Json => "application/opds+json",
            Renderer::Atom => "application/atom+xml;profile=opds-catalog",
        }
    }
}

/// Pick a renderer from an `Accept` header value.
///
/// If the header is missing or only contains `*/*`, defaults to JSON.
pub fn pick_renderer(accept: Option<&str>) -> Renderer {
    let Some(accept) = accept else {
        return Renderer::Json;
    };

    let mut best: Option<(f64, Renderer)> = None;
    for part in accept.split(',') {
        let mut iter = part.trim().split(';');
        let media = iter.next().unwrap_or("").trim();
        let mut q: f64 = 1.0;
        for token in iter {
            let token = token.trim();
            if let Some(value) = token.strip_prefix("q=") {
                q = value.parse().unwrap_or(0.0);
            }
        }

        let renderer = match media {
            "*/*" => Renderer::Json,
            _ if matches_mime(media, &APPLICATION_JSON) || media == MIME_OPDS_JSON => {
                Renderer::Json
            }
            _ if is_xml(media) => Renderer::Atom,
            _ => continue,
        };

        if best.is_none_or(|(best_q, _)| q > best_q) {
            best = Some((q, renderer));
        }
    }

    best.map(|(_, r)| r).unwrap_or(Renderer::Json)
}

fn matches_mime(media: &str, expected: &Mime) -> bool {
    media
        .parse::<Mime>()
        .ok()
        .is_some_and(|m| m.type_() == expected.type_() && m.subtype() == expected.subtype())
}

fn is_xml(media: &str) -> bool {
    media == MIME_ATOM
        || media
            .parse::<Mime>()
            .ok()
            .is_some_and(|m| m.type_() == "application" && m.subtype() == "xml")
        || matches_mime(media, &TEXT_XML)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_accept_defaults_json() {
        assert_eq!(pick_renderer(None), Renderer::Json);
    }

    #[test]
    fn json_wins_over_atom_when_higher_q() {
        assert_eq!(
            pick_renderer(Some(
                "application/opds+json;q=1.0, application/atom+xml;q=0.9"
            )),
            Renderer::Json
        );
    }

    #[test]
    fn atom_explicitly_selected() {
        assert_eq!(pick_renderer(Some("application/atom+xml")), Renderer::Atom);
    }
}
