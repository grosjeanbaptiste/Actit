use super::*;
use std::cell::RefCell;

struct FakeHttp {
    body: Result<String, String>,
    asked: RefCell<Vec<String>>,
}

impl Http for FakeHttp {
    fn get(&self, url: &str) -> Result<String, String> {
        self.asked.borrow_mut().push(url.to_string());
        self.body.clone()
    }
}

fn http(body: Result<&str, &str>) -> FakeHttp {
    FakeHttp {
        body: body.map(String::from).map_err(String::from),
        asked: RefCell::default(),
    }
}

const SERDE: &str = r#"[{"id":"serde","name":"serde","registry":"crates_io",
  "url":"https://crates.io/crates/serde","description":"Serialisation",
  "docs_url":"https://docs.rs/serde","license":"MIT"}]"#;

#[test]
fn term_is_percent_encoded_in_the_packages_url() {
    let fake = http(Ok("[]"));
    let catalog = IndexerCatalog::new("https://i.example/", &fake);
    catalog.packages("c++ é").unwrap();
    assert_eq!(fake.asked.borrow()[0], "https://i.example/v0/packages?q=c%2B%2B%20%C3%A9");
}

#[test]
fn packages_are_parsed_ignoring_unknown_fields() {
    let fake = http(Ok(SERDE));
    let found = IndexerCatalog::new("https://i.example", &fake).packages("serde").unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].registry, "crates_io");
    assert_eq!(found[0].docs_url.as_deref(), Some("https://docs.rs/serde"));
}

#[test]
fn missing_docs_url_and_description_are_allowed() {
    let body = r#"[{"id":"x","registry":"npm","url":"https://npm.example/x"}]"#;
    let found = IndexerCatalog::new("https://i", &http(Ok(body))).packages("x").unwrap();
    assert_eq!(found[0].docs_url, None);
    assert_eq!(found[0].description, "");
}

#[test]
fn transport_failure_is_an_error_naming_indexer() {
    let err = IndexerCatalog::new("https://i", &http(Err("timeout"))).packages("x").unwrap_err();
    assert_eq!(err, Error::Catalog("timeout".into()));
}

#[test]
fn malformed_json_is_an_error_not_an_empty_list() {
    let err = IndexerCatalog::new("https://i", &http(Ok("<html>"))).packages("x").unwrap_err();
    assert!(matches!(err, Error::Catalog(m) if m.contains("invalid response")));
}
