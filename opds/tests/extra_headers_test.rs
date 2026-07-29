use livtet_opds_client::ExtraHeaders;
use reqwest::Client;

fn request_builder() -> reqwest::RequestBuilder {
    Client::new().get("http://localhost/test")
}

#[test]
fn new_starts_empty() {
    let headers = ExtraHeaders::new();
    assert!(headers.0.is_empty());
}

#[test]
fn add_then_apply_includes_header() {
    let mut headers = ExtraHeaders::new();
    headers.add("X-Custom", "value");

    let result = headers.apply(request_builder());
    let req = result.build().unwrap();
    let val = req
        .headers()
        .get("X-Custom")
        .expect("added header must be present");
    assert_eq!(val.to_str().unwrap(), "value");
}

#[test]
fn crlf_injection_rejected_by_reqwest() {
    let mut headers = ExtraHeaders::new();
    headers.add("X-Test", "x\r\nSet-Cookie: evil=1");

    let result = headers.apply(request_builder());
    let build_result = result.build();
    assert!(
        build_result.is_err(),
        "reqwest must reject header values containing CRLF"
    );
}
