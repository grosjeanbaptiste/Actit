//! Licence verification: classify an SPDX expression and decide whether the tool is reusable.
use crate::error::Error;
use crate::model::{Finding, Report};
use crate::spdx::{self, Expr};

/// How restrictive a licence is, from least to most; `Unknown` is never reusable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    PublicDomain,
    Permissive,
    WeakCopyleft,
    StrongCopyleft,
    NetworkCopyleft,
    Restricted,
    Proprietary,
    Unknown,
}

const NAMES: [(Class, &str); 8] = [
    (Class::PublicDomain, "public-domain"),
    (Class::Permissive, "permissive"),
    (Class::WeakCopyleft, "weak-copyleft"),
    (Class::StrongCopyleft, "strong-copyleft"),
    (Class::NetworkCopyleft, "network-copyleft"),
    (Class::Restricted, "restricted"),
    (Class::Proprietary, "proprietary"),
    (Class::Unknown, "unknown"),
];

impl Class {
    pub fn wire(self) -> &'static str {
        NAMES
            .iter()
            .find(|(c, _)| *c == self)
            .map_or("unknown", |(_, n)| n)
    }

    pub fn parse(text: &str) -> Option<Class> {
        NAMES.iter().find(|(_, n)| *n == text).map(|(c, _)| *c)
    }

    /// Only these classes can ever be reusable: a tool whose licence forbids commercial use
    /// (non-commercial, source-available with a use limit, proprietary) or that cannot be
    /// identified is out, whatever the policy says.
    pub fn permits_commercial_use(self) -> bool {
        !matches!(
            self,
            Class::Restricted | Class::Proprietary | Class::Unknown
        )
    }
}

/// Known licence ids (lowercase) and their class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table(Vec<(String, Class)>);

impl Table {
    pub fn builtin() -> Table {
        match Table::parse(include_str!("../resources/licenses.txt")) {
            Ok(table) => table,
            // The embedded file is checked by a test: a failure here is a build defect.
            Err(e) => panic!("resources/licenses.txt is invalid: {e}"),
        }
    }

    /// One `id|class` per line; `#` comments and blank lines are skipped.
    pub fn parse(text: &str) -> Result<Table, Error> {
        let mut rows: Vec<(String, Class)> = Vec::new();
        for (n, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let bad = |why: &str| Error::License(format!("licence table line {}: {why}", n + 1));
            let (id, class) = line
                .split_once('|')
                .ok_or_else(|| bad("expected id|class"))?;
            let class = Class::parse(class.trim())
                .filter(|c| *c != Class::Unknown)
                .ok_or_else(|| bad("the class must be one of the policy classes"))?;
            let id = id.trim().to_lowercase();
            if id.is_empty() || id.contains('|') {
                return Err(bad("empty id"));
            }
            if rows.iter().any(|(known, _)| *known == id) {
                return Err(bad("duplicate id"));
            }
            rows.push((id, class));
        }
        Ok(Table(rows))
    }

    pub fn class_of(&self, id: &str) -> Option<Class> {
        let id = id.to_lowercase();
        self.0
            .iter()
            .find(|(known, _)| *known == id)
            .map(|(_, c)| *c)
    }

    fn lookup(&self, id: &str) -> Option<Class> {
        self.class_of(id)
            .or_else(|| id.strip_suffix('+').and_then(|base| self.class_of(base)))
    }
}

/// The verdict on one licence expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub expression: Option<String>,
    pub class: Class,
    pub allowed: bool,
    /// Why the class is `Unknown`, when it is.
    pub note: Option<String>,
}

/// Public domain and permissive licences.
pub fn default_allow() -> Vec<Class> {
    vec![Class::PublicDomain, Class::Permissive]
}

/// `permissive,weak-copyleft`: a comma list of class names. A class that forbids commercial
/// use, or `unknown`, cannot be allowed.
pub fn parse_allow(list: &str) -> Result<Vec<Class>, Error> {
    list.split(',')
        .map(|word| {
            let word = word.trim();
            match Class::parse(word) {
                Some(class) if class.permits_commercial_use() => Ok(class),
                Some(Class::Unknown) | None => Err(Error::License(format!(
                    "{word:?} is not a licence class (one of: {})",
                    policy_names()
                ))),
                Some(_) => Err(Error::License(format!(
                    "{word:?} does not permit commercial use"
                ))),
            }
        })
        .collect()
}

fn policy_names() -> String {
    let names: Vec<&str> = NAMES
        .iter()
        .filter(|(c, _)| c.permits_commercial_use())
        .map(|(_, n)| *n)
        .collect();
    names.join(", ")
}

fn class_of_expr(expr: &Expr, table: &Table, unknown: &mut Vec<String>) -> Class {
    match expr {
        Expr::Id(id) => table.lookup(id).unwrap_or_else(|| {
            unknown.push(id.clone());
            Class::Unknown
        }),
        Expr::Or(parts) => {
            let classes: Vec<Class> = parts
                .iter()
                .map(|p| class_of_expr(p, table, unknown))
                .collect();
            classes
                .into_iter()
                .filter(|c| *c != Class::Unknown)
                .min()
                .unwrap_or(Class::Unknown)
        }
        Expr::And(parts) => {
            let classes: Vec<Class> = parts
                .iter()
                .map(|p| class_of_expr(p, table, unknown))
                .collect();
            classes.into_iter().max().unwrap_or(Class::Unknown)
        }
    }
}

fn unknown_check(expression: Option<String>, note: String) -> Check {
    unknown_class_check(expression, Class::Unknown, note)
}

/// A licence that is never reusable, with the reason.
fn unknown_class_check(expression: Option<String>, class: Class, note: String) -> Check {
    Check {
        expression,
        class,
        allowed: false,
        note: Some(note),
    }
}

/// `OR` takes the least restrictive alternative, `AND` the most; what cannot be read or
/// identified is `Unknown`, which is never allowed, whatever the policy.
pub fn check(expression: Option<&str>, table: &Table, allow: &[Class]) -> Check {
    let Some(text) = expression.map(str::trim).filter(|t| !t.is_empty()) else {
        return unknown_check(None, "no licence declared".into());
    };
    let shown = Some(text.to_owned());
    let expr = match spdx::parse(text) {
        Ok(expr) => expr,
        Err(why) => return unknown_check(shown, why),
    };
    let mut unknown = Vec::new();
    let class = class_of_expr(&expr, table, &mut unknown);
    if class == Class::Unknown {
        return unknown_check(shown, format!("unknown licence id {}", unknown.join(", ")));
    }
    if !class.permits_commercial_use() {
        return unknown_class_check(shown, class, "does not permit commercial use".into());
    }
    Check {
        expression: shown,
        class,
        allowed: allow.contains(&class),
        note: None,
    }
}

/// One check per finding, same order.
pub fn checks(report: &Report, table: &Table, allow: &[Class]) -> Vec<Check> {
    let one = |f: &Finding| check(f.candidate.license.as_deref(), table, allow);
    report.findings.iter().map(one).collect()
}

/// The findings whose licence allows reuse.
pub fn reusable<'a>(report: &'a Report, checks: &[Check]) -> Vec<&'a Finding> {
    let pairs = report.findings.iter().zip(checks);
    pairs.filter(|(_, c)| c.allowed).map(|(f, _)| f).collect()
}

/// `licence <expr>: <class> (reusable|not reusable)[, <note>]`
pub fn render_line(check: &Check) -> String {
    let what = check.expression.as_deref().unwrap_or("none declared");
    let verdict = if check.allowed {
        "reusable"
    } else {
        "not reusable"
    };
    let mut line = format!("licence {what}: {} ({verdict})", check.class.wire());
    if let Some(note) = &check.note {
        line.push_str(", ");
        line.push_str(note);
    }
    line
}

#[cfg(test)]
mod tests {
    include!("license_tests.rs");
}
