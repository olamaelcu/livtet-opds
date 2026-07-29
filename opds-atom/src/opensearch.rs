//! OpenSearch 1.1 description document support.
//!
//! Clients use this document to discover the search template for an OPDS 1.x
//! catalog; servers use it to advertise search.

use livtet_opds_types::OpdsError;
use quick_xml::{
    XmlVersion,
    events::{BytesEnd, BytesStart, Event},
    reader::Reader,
    writer::Writer,
};

/// Parsed OpenSearch description document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenSearchDescription {
    pub short_name: String,
    pub description: String,
    pub template_url: String,
    pub template_type: String,
}

/// Render a minimal OpenSearch description document.
///
/// The returned string is a complete XML document with a single `Url`
/// element that points at `template_url`. `template_url` should contain a
/// `{searchTerms}` placeholder when used with OPDS 1.x clients.
pub fn render_description(template_url: &str, short_name: &str, description: &str) -> String {
    let mut writer = Writer::new(Vec::new());

    let _ = writer.write_event(Event::Decl(quick_xml::events::BytesDecl::new(
        "1.0",
        Some("UTF-8"),
        None,
    )));

    let mut root = BytesStart::new("OpenSearchDescription");
    root.push_attribute(("xmlns", "http://a9.com/-/spec/opensearch/1.1/"));
    let _ = writer.write_event(Event::Start(root));

    let _ = writer.write_event(Event::Start(BytesStart::new("ShortName")));
    let _ = writer.write_event(Event::Text(quick_xml::events::BytesText::new(short_name)));
    let _ = writer.write_event(Event::End(BytesEnd::new("ShortName")));

    let _ = writer.write_event(Event::Start(BytesStart::new("Description")));
    let _ = writer.write_event(Event::Text(quick_xml::events::BytesText::new(description)));
    let _ = writer.write_event(Event::End(BytesEnd::new("Description")));

    let _ = writer.write_event(Event::Start(BytesStart::new("InputEncoding")));
    let _ = writer.write_event(Event::Text(quick_xml::events::BytesText::new("UTF-8")));
    let _ = writer.write_event(Event::End(BytesEnd::new("InputEncoding")));

    let _ = writer.write_event(Event::Start(BytesStart::new("OutputEncoding")));
    let _ = writer.write_event(Event::Text(quick_xml::events::BytesText::new("UTF-8")));
    let _ = writer.write_event(Event::End(BytesEnd::new("OutputEncoding")));

    let mut url = BytesStart::new("Url");
    url.push_attribute(("type", template_type_value()));
    url.push_attribute(("template", template_url));
    let _ = writer.write_event(Event::Empty(url));

    let _ = writer.write_event(Event::End(BytesEnd::new("OpenSearchDescription")));

    String::from_utf8(writer.into_inner()).unwrap_or_default()
}

/// Parse an OpenSearch description document.
#[cfg_attr(test, mutants::skip)]
pub fn parse_description(body: &str) -> Result<OpenSearchDescription, OpdsError> {
    let mut reader = Reader::from_str(body);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut short_name = None;
    let mut description = None;
    let mut template_url = None;
    let mut template_type = None;
    let mut current_element: Option<String> = None;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                current_element = Some(local_name(e.name().as_ref()).to_lowercase());
            }
            Ok(Event::Empty(e)) => {
                let name = e.name();
                let local = local_name(name.as_ref());
                if local.eq_ignore_ascii_case("url") {
                    for attr in e.attributes() {
                        let attr = attr.map_err(|e| OpdsError::Xml(format!("{e}")))?;
                        let attr_name = local_name(attr.key.as_ref());
                        let value = attr
                            .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
                            .map_err(|e| OpdsError::Xml(format!("{e}")))?;
                        if attr_name.eq_ignore_ascii_case("template") && template_url.is_none() {
                            template_url = Some(value.into_owned());
                        } else if attr_name.eq_ignore_ascii_case("type") && template_type.is_none()
                        {
                            template_type = Some(value.into_owned());
                        }
                    }
                }
            }
            Ok(Event::Text(e)) => {
                if let Some(ref name) = current_element {
                    let text = e
                        .xml10_content()
                        .map_err(|e| OpdsError::Xml(format!("{e}")))?
                        .into_owned();
                    match name.as_str() {
                        "shortname" if short_name.is_none() => short_name = Some(text),
                        "description" if description.is_none() => description = Some(text),
                        _ => {}
                    }
                }
            }
            Ok(Event::End(_)) => {
                current_element = None;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OpdsError::Xml(format!("opensearch parse error: {e}"))),
            _ => {}
        }
    }

    Ok(OpenSearchDescription {
        short_name: short_name.unwrap_or_default(),
        description: description.unwrap_or_default(),
        template_url: template_url.unwrap_or_default(),
        template_type: template_type.unwrap_or_else(|| template_type_value().to_string()),
    })
}

fn template_type_value() -> &'static str {
    "application/atom+xml;profile=opds-catalog;kind=acquisition"
}

fn local_name(name: &[u8]) -> &str {
    let s = std::str::from_utf8(name).unwrap_or("");
    s.rsplit_once(':').map_or(s, |(_, local)| local)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_description_contains_short_name() {
        let xml = render_description(
            "http://example.com/search?q={searchTerms}",
            "My Catalog",
            "Search my catalog",
        );
        assert!(xml.contains("My Catalog"));
        assert!(xml.contains("Search my catalog"));
        assert!(xml.contains("http://example.com/search?q={searchTerms}"));
    }

    #[test]
    fn render_description_has_opensearch_namespace() {
        let xml = render_description("/search", "S", "D");
        assert!(xml.contains("xmlns=\"http://a9.com/-/spec/opensearch/1.1/\""));
    }

    #[test]
    fn render_description_has_url_element() {
        let xml = render_description("/search", "S", "D");
        assert!(xml.contains("<Url"));
        assert!(xml.contains("template=\"/search\""));
    }

    #[test]
    fn render_description_has_short_name_element() {
        let xml = render_description("/s", "TestName", "D");
        assert!(xml.contains("<ShortName>TestName</ShortName>"));
    }

    #[test]
    fn render_description_has_description_element() {
        let xml = render_description("/s", "S", "TestDesc");
        assert!(xml.contains("<Description>TestDesc</Description>"));
    }

    #[test]
    fn parse_description_round_trips() {
        let xml = render_description(
            "http://example.com/opensearch?q={searchTerms}",
            "My Lib",
            "My Library Catalog",
        );
        let parsed = parse_description(&xml).expect("parse");
        assert_eq!(parsed.short_name, "My Lib");
        assert_eq!(parsed.description, "My Library Catalog");
        assert_eq!(
            parsed.template_url,
            "http://example.com/opensearch?q={searchTerms}"
        );
    }

    #[test]
    fn parse_description_defaults_template_type() {
        let xml = render_description("/s", "S", "D");
        let parsed = parse_description(&xml).expect("parse");
        assert_eq!(
            parsed.template_type,
            "application/atom+xml;profile=opds-catalog;kind=acquisition"
        );
    }

    #[test]
    fn parse_description_handles_empty_body() {
        let result = parse_description("");
        assert!(result.is_ok());
        let desc = result.unwrap();
        assert_eq!(desc.short_name, "");
        assert_eq!(desc.description, "");
        assert_eq!(desc.template_url, "");
    }

    #[test]
    fn parse_description_handles_no_url_element() {
        let result = parse_description(
            "<OpenSearchDescription><ShortName>S</ShortName></OpenSearchDescription>",
        );
        assert!(result.is_ok());
        let desc = result.unwrap();
        assert_eq!(desc.short_name, "S");
        assert_eq!(desc.template_url, "");
    }

    #[test]
    fn template_type_value_returns_acquisition() {
        assert_eq!(
            template_type_value(),
            "application/atom+xml;profile=opds-catalog;kind=acquisition"
        );
    }

    #[test]
    fn local_name_strips_prefix() {
        assert_eq!(local_name(b"atom:entry"), "entry");
        assert_eq!(local_name(b"dc:title"), "title");
    }

    #[test]
    fn local_name_returns_full_name_when_no_prefix() {
        assert_eq!(local_name(b"entry"), "entry");
        assert_eq!(local_name(b"feed"), "feed");
    }

    #[test]
    fn local_name_handles_empty() {
        assert_eq!(local_name(b""), "");
    }

    #[test]
    fn local_name_handles_non_utf8() {
        assert_eq!(local_name(b"\xff\xfe"), "");
    }
}
