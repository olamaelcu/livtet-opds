use std::collections::HashMap;

use livtet_opds_client::SearchTemplate;

fn template_with(template: &str) -> SearchTemplate {
    SearchTemplate {
        template: template.to_string(),
        variables: extract_var_names(template),
    }
}

fn extract_var_names(template: &str) -> Vec<String> {
    let mut vars = Vec::new();
    if let Some(start) = template.find('{')
        && let Some(end) = template[start..].find('}')
    {
        let content = &template[start + 1..start + end];
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

#[test]
fn expand_encodes_space_as_plus() {
    let tmpl = template_with("/search{?query}");
    let mut params = HashMap::new();
    params.insert("query".into(), "hello world".into());

    let result = tmpl.expand(&params).unwrap();
    assert!(
        result.contains("query=hello+world"),
        "space should be encoded as +, got: {result}"
    );
}

#[test]
fn expand_percent_encodes_ampersand_and_equals() {
    let tmpl = template_with("/search{?query}");
    let mut params = HashMap::new();
    params.insert("query".into(), "a&b=c".into());

    let result = tmpl.expand(&params).unwrap();
    assert!(
        !result.contains("a&b=c"),
        "raw & and = must be percent-encoded, got: {result}"
    );
    assert!(result.contains("%26"), "expected %26 for &, got: {result}");
    assert!(result.contains("%3D"), "expected %3D for =, got: {result}");
}

#[test]
fn expand_percent_encodes_angle_brackets() {
    let tmpl = template_with("/search{?query}");
    let mut params = HashMap::new();
    params.insert("query".into(), "<script>".into());

    let result = tmpl.expand(&params).unwrap();
    assert!(result.contains("%3C"), "expected %3C for <, got: {result}");
    assert!(result.contains("%3E"), "expected %3E for >, got: {result}");
}

#[test]
fn expand_percent_encodes_utf8() {
    let tmpl = template_with("/search{?query}");
    let mut params = HashMap::new();
    params.insert("query".into(), "étui".into());

    let result = tmpl.expand(&params).unwrap();
    assert!(
        result.contains("%C3%A9"),
        "expected UTF-8 percent-encoding for é, got: {result}"
    );
}

#[test]
fn expand_with_empty_params_leaves_placeholders_as_literal() {
    let tmpl = template_with("/search{?query}");
    let params = HashMap::new();

    let result = tmpl.expand(&params).unwrap();
    assert_eq!(
        result, "/search",
        "empty params should return prefix only, got: {result}"
    );
}

#[test]
fn expand_with_non_query_start_template_returns_unchanged() {
    let tmpl = template_with("/search{#fragment}");
    let mut params = HashMap::new();
    params.insert("fragment".into(), "test".into());

    let result = tmpl.expand(&params).unwrap();
    assert_eq!(
        result, "/search{#fragment}",
        "non-query template should pass through unchanged, got: {result}"
    );
}
