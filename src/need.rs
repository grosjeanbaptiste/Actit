use crate::error::Error;
use crate::settings::Settings;

/// How a need is cut into searchable terms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermRules {
    pub min_length: usize,
    pub stopwords: Vec<String>,
}

impl Default for TermRules {
    fn default() -> Self {
        Self {
            min_length: Settings::default().min_term_length,
            stopwords: parse_stopwords(include_str!("../resources/stopwords.txt")),
        }
    }
}

/// One word per line, `#` comments and blank lines ignored, lowercased.
pub fn parse_stopwords(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_lowercase)
        .collect()
}

/// What the user wants a tool for, validated to hold at least one searchable term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Need {
    text: String,
    terms: Vec<String>,
}

impl Need {
    pub fn new(text: &str) -> Result<Self, Error> {
        Self::with_rules(text, &TermRules::default())
    }

    pub fn with_rules(text: &str, rules: &TermRules) -> Result<Self, Error> {
        let text = text.trim();
        if text.is_empty() {
            return Err(Error::EmptyNeed);
        }
        let terms = extract_terms(text, rules);
        if terms.is_empty() {
            return Err(Error::NoTerms(text.to_string()));
        }
        Ok(Self {
            text: text.to_string(),
            terms,
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn terms(&self) -> &[String] {
        &self.terms
    }

    /// Share of the need's terms that appear in `text`, case-insensitively.
    pub fn coverage(&self, text: &str) -> f32 {
        let text = text.to_lowercase();
        let hit = self
            .terms
            .iter()
            .filter(|t| text.contains(t.as_str()))
            .count();
        hit as f32 / self.terms.len() as f32
    }
}

fn extract_terms(text: &str, rules: &TermRules) -> Vec<String> {
    let mut terms: Vec<String> = Vec::new();
    for word in text.split(|c: char| !c.is_alphanumeric()) {
        let word = word.to_lowercase();
        if word.chars().count() >= rules.min_length
            && !rules.stopwords.contains(&word)
            && !terms.contains(&word)
        {
            terms.push(word);
        }
    }
    terms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_is_the_share_of_terms_found_in_the_text() {
        let need = Need::new("pdf text converter").unwrap();
        assert_eq!(need.coverage("A PDF reader"), 1.0 / 3.0);
        assert_eq!(need.coverage("nothing"), 0.0);
        assert_eq!(need.coverage("pdf TEXT converter"), 1.0);
    }

    #[test]
    fn rules_set_the_minimum_term_length() {
        let rules = TermRules {
            min_length: 5,
            ..TermRules::default()
        };
        let need = Need::with_rules("csv converter pipeline", &rules).unwrap();
        assert_eq!(need.terms(), ["converter", "pipeline"]);
    }

    #[test]
    fn custom_stopwords_replace_the_built_in_ones() {
        let rules = TermRules {
            min_length: 3,
            stopwords: vec!["pdf".into()],
        };
        let need = Need::with_rules("the pdf reader", &rules).unwrap();
        assert_eq!(need.terms(), ["the", "reader"]);
    }

    #[test]
    fn stopword_file_has_one_word_per_line_with_comments() {
        let words = parse_stopwords("# english\nThe\n\n  and \n#x\n");
        assert_eq!(words, ["the", "and"]);
    }

    #[test]
    fn new_uses_the_default_rules() {
        assert_eq!(
            Need::new("a pdf").unwrap(),
            Need::with_rules("a pdf", &TermRules::default()).unwrap()
        );
    }

    #[test]
    fn blank_need_is_refused() {
        assert_eq!(Need::new("  \n").unwrap_err(), Error::EmptyNeed);
    }

    #[test]
    fn terms_are_lowercase_unique_and_without_stopwords() {
        let need = Need::new("Convert a PDF to text, then PDF to the cloud").unwrap();
        assert_eq!(need.terms(), ["convert", "pdf", "text", "then", "cloud"]);
    }

    #[test]
    fn short_acronyms_are_kept() {
        let need = Need::new("csv to sql").unwrap();
        assert_eq!(need.terms(), ["csv", "sql"]);
    }

    #[test]
    fn need_of_only_stopwords_is_refused() {
        assert_eq!(
            Need::new("to the and").unwrap_err(),
            Error::NoTerms("to the and".into())
        );
    }

    #[test]
    fn accents_and_french_stopwords() {
        let need = Need::new("Analyser une phrase pour les élèves").unwrap();
        assert_eq!(need.terms(), ["analyser", "phrase", "élèves"]);
    }
}
