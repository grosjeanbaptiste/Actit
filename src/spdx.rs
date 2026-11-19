//! SPDX licence expressions: `A OR B`, `A AND B`, `(…)`, `A WITH exception`, `A+`.
//! `AND` binds tighter than `OR`; an exception does not change the licence it is attached to.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Id(String),
    And(Vec<Expr>),
    Or(Vec<Expr>),
}

fn tokens(text: &str) -> Vec<String> {
    let spaced = text.replace('(', " ( ").replace(')', " ) ");
    spaced.split_whitespace().map(str::to_owned).collect()
}

fn is_word(token: &str, word: &str) -> bool {
    token.eq_ignore_ascii_case(word)
}

struct Parser {
    tokens: Vec<String>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> Option<&str> {
        self.tokens.get(self.at).map(String::as_str)
    }

    fn keyword(&mut self, word: &str) -> bool {
        let hit = self.peek().is_some_and(|t| is_word(t, word));
        self.at += usize::from(hit);
        hit
    }

    fn list(
        &mut self,
        word: &str,
        item: fn(&mut Self) -> Result<Expr, String>,
    ) -> Result<Vec<Expr>, String> {
        let mut parts = vec![item(self)?];
        while self.keyword(word) {
            parts.push(item(self)?);
        }
        Ok(parts)
    }

    fn or(&mut self) -> Result<Expr, String> {
        let mut parts = self.list("OR", Self::and)?;
        Ok(if parts.len() == 1 {
            parts.remove(0)
        } else {
            Expr::Or(parts)
        })
    }

    fn and(&mut self) -> Result<Expr, String> {
        let mut parts = self.list("AND", Self::atom)?;
        Ok(if parts.len() == 1 {
            parts.remove(0)
        } else {
            Expr::And(parts)
        })
    }

    fn atom(&mut self) -> Result<Expr, String> {
        match self.peek() {
            None => Err("expected a licence id, found the end".into()),
            Some("(") => {
                self.at += 1;
                let inner = self.or()?;
                if self.peek() == Some(")") {
                    self.at += 1;
                    return Ok(inner);
                }
                Err("missing closing parenthesis".into())
            }
            Some(")") => Err("unexpected `)`".into()),
            Some(id) if ["AND", "OR", "WITH"].iter().any(|w| is_word(id, w)) => {
                Err(format!("expected a licence id, found {id}"))
            }
            Some(id) => {
                let id = id.to_owned();
                self.at += 1;
                if self.keyword("WITH") {
                    // The exception id does not change the class of the licence.
                    match self.peek() {
                        Some(e)
                            if !["(", ")"].contains(&e)
                                && !is_word(e, "AND")
                                && !is_word(e, "OR") =>
                        {
                            self.at += 1;
                        }
                        _ => return Err("expected an exception id after WITH".into()),
                    }
                }
                Ok(Expr::Id(id))
            }
        }
    }
}

/// Parses a whole expression; trailing words are an error.
pub fn parse(text: &str) -> Result<Expr, String> {
    let mut parser = Parser {
        tokens: tokens(text),
        at: 0,
    };
    let expr = parser.or()?;
    match parser.peek() {
        None => Ok(expr),
        Some(")") => Err("unexpected `)`".into()),
        Some(other) => Err(format!("expected AND or OR, found {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(s: &str) -> Expr {
        Expr::Id(s.into())
    }

    #[test]
    fn precedence_parentheses_and_with() {
        assert_eq!(
            parse("A OR B AND C"),
            Ok(Expr::Or(vec![id("A"), Expr::And(vec![id("B"), id("C")])]))
        );
        assert_eq!(
            parse("(A OR B) AND C"),
            Ok(Expr::And(vec![Expr::Or(vec![id("A"), id("B")]), id("C")]))
        );
        assert_eq!(
            parse("GPL-2.0-only WITH Classpath-exception-2.0"),
            Ok(id("GPL-2.0-only"))
        );
        assert_eq!(parse("a or b"), Ok(Expr::Or(vec![id("a"), id("b")])));
    }

    #[test]
    fn malformed_input_is_an_error_with_a_reason() {
        for bad in ["", "(A", "A OR", "A B", "A )", "OR A", "A WITH", "A WITH )"] {
            assert!(parse(bad).is_err(), "{bad:?}");
        }
    }
}
