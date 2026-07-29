use livtet_opds_client::OpdsAuth;
use reqwest::Client;

fn request_builder() -> reqwest::RequestBuilder {
    Client::new().get("http://localhost/test")
}

#[test]
fn none_returns_builder_unchanged() {
    let builder = request_builder();
    let result = OpdsAuth::None.apply(builder);

    let req = result.build().unwrap();
    assert!(
        !req.headers().contains_key("Authorization"),
        "OpdsAuth::None must not set an Authorization header"
    );
}

#[test]
fn basic_sets_authorization_header() {
    let builder = request_builder();
    let auth = OpdsAuth::Basic {
        username: "user".into(),
        password: "pass".into(),
    };
    let result = auth.apply(builder);

    let req = result.build().unwrap();
    let header = req
        .headers()
        .get("Authorization")
        .expect("Basic auth must set Authorization header");
    let value = header.to_str().unwrap();
    assert!(
        value.starts_with("Basic "),
        "expected 'Basic' scheme, got: {value}"
    );
    assert_eq!(&value[6..], "dXNlcjpwYXNz");
}

#[test]
fn bearer_sets_authorization_header() {
    let builder = request_builder();
    let auth = OpdsAuth::Bearer {
        access_token: "tok".into(),
    };
    let result = auth.apply(builder);

    let req = result.build().unwrap();
    let header = req
        .headers()
        .get("Authorization")
        .expect("Bearer auth must set Authorization header");
    assert_eq!(header.to_str().unwrap(), "Bearer tok");
}

#[test]
fn apply_adds_alongside_existing_authorization() {
    let builder = request_builder().header("Authorization", "Bearer old");
    let auth = OpdsAuth::Bearer {
        access_token: "new".into(),
    };
    let result = auth.apply(builder);

    let req = result.build().unwrap();
    let all: Vec<_> = req.headers().get_all("Authorization").iter().collect();
    assert!(
        all.len() >= 2,
        "apply should add a new Authorization header alongside the existing one, got {all:?}"
    );
    let values: Vec<&str> = all.iter().map(|v| v.to_str().unwrap()).collect();
    assert!(values.contains(&"Bearer new"));
}
