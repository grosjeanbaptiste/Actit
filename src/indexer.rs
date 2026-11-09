use crate::error::Error;
use crate::model::Candidate;
use crate::ports::Catalog;
use serde::Deserialize;

/// Transport seam: the only network access of the catalogue.
pub trait Http {
    fn get(&self, url: &str) -> Result<String, String>;
}

pub struct IndexerCatalog<'a> {
    base: String,
    http: &'a dyn Http,
}

#[derive(Deserialize)]
struct Row {
    id: String,
    registry: String,
    url: String,
    docs_url: Option<String>,
    #[serde(default)]
    description: String,
}

impl<'a> IndexerCatalog<'a> {
    pub fn new(base: &str, http: &'a dyn Http) -> Self {
        Self {
            base: base.trim_end_matches('/').to_string(),
            http,
        }
    }
}

impl Catalog for IndexerCatalog<'_> {
    fn packages(&self, term: &str) -> Result<Vec<Candidate>, Error> {
        let url = format!("{}/v0/packages?q={}", self.base, percent_encode(term));
        let body = self.http.get(&url).map_err(Error::Catalog)?;
        let rows: Vec<Row> = serde_json::from_str(&body)
            .map_err(|e| Error::Catalog(format!("invalid response from {url}: {e}")))?;
        Ok(rows.into_iter().map(candidate).collect())
    }
}

fn candidate(row: Row) -> Candidate {
    Candidate {
        id: row.id,
        registry: row.registry,
        url: row.url,
        docs_url: row.docs_url,
        description: row.description,
    }
}

fn percent_encode(text: &str) -> String {
    let mut out = String::new();
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    include!("indexer_tests.rs");
}
