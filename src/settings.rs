use crate::error::Error;

/// Every tunable of the CLI. The `Default` values are the only place they are written down.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub indexer: String,
    pub kag: String,
    pub gh: String,
    pub limit: usize,
    pub min_confidence: f32,
    pub min_coverage: f32,
    pub min_term_length: usize,
    pub stopwords_file: Option<String>,
    /// `registry=ecosystem` pairs added to (and overriding) the built-in kag ecosystems.
    pub ecosystems: Vec<(String, String)>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            indexer: "https://indexer.acteble.workers.dev".into(),
            kag: "kag".into(),
            gh: "gh".into(),
            limit: 5,
            min_confidence: 0.5,
            min_coverage: 0.5,
            min_term_length: 3,
            stopwords_file: None,
            ecosystems: Vec::new(),
        }
    }
}

const OPTIONS: &[&str] = &[
    "indexer",
    "kag",
    "gh",
    "limit",
    "min-confidence",
    "min-coverage",
    "min-term-length",
    "stopwords",
    "ecosystem",
];

impl Settings {
    /// Reads `--option value` pairs anywhere in `args`; flag beats `ACTIT_<OPTION>` beats default.
    /// Returns the settings and the arguments that are not settings, in order.
    pub fn parse(
        args: &[String],
        env: &dyn Fn(&str) -> Option<String>,
    ) -> Result<(Self, Vec<String>), Error> {
        let mut settings = Self::default();
        for option in OPTIONS.iter().filter(|o| **o != "ecosystem") {
            if let Some(value) = env(&env_name(option)) {
                settings.set(option, &value)?;
            }
        }
        if let Some(list) = env("ACTIT_ECOSYSTEMS") {
            for pair in list.split(',').filter(|p| !p.is_empty()) {
                settings.set("ecosystem", pair)?;
            }
        }
        let mut rest = Vec::new();
        let mut args = args.iter();
        while let Some(arg) = args.next() {
            match arg.strip_prefix("--").filter(|name| OPTIONS.contains(name)) {
                Some(name) => {
                    let value = args
                        .next()
                        .ok_or_else(|| Error::Usage(format!("--{name} needs a value")))?;
                    settings.set(name, value)?;
                }
                None => rest.push(arg.clone()),
            }
        }
        Ok((settings, rest))
    }

    fn set(&mut self, option: &str, value: &str) -> Result<(), Error> {
        let bad = |what: &str| Error::Usage(format!("--{option}: {what}, got {value:?}"));
        match option {
            "indexer" => self.indexer = value.into(),
            "kag" => self.kag = value.into(),
            "gh" => self.gh = value.into(),
            "stopwords" => self.stopwords_file = Some(value.into()),
            "limit" => self.limit = value.parse().map_err(|_| bad("a whole number"))?,
            "min-term-length" => {
                let n: usize = value.parse().map_err(|_| bad("a whole number"))?;
                if n == 0 {
                    return Err(bad("at least 1"));
                }
                self.min_term_length = n;
            }
            "min-confidence" => self.min_confidence = ratio(value).ok_or_else(|| bad("0 to 1"))?,
            "min-coverage" => self.min_coverage = ratio(value).ok_or_else(|| bad("0 to 1"))?,
            _ => {
                let (registry, eco) = value
                    .split_once('=')
                    .filter(|(r, e)| !r.is_empty() && !e.is_empty())
                    .ok_or_else(|| bad("registry=ecosystem"))?;
                self.ecosystems.push((registry.into(), eco.into()));
            }
        }
        Ok(())
    }
}

fn ratio(value: &str) -> Option<f32> {
    value.parse().ok().filter(|v| (0.0..=1.0).contains(v))
}

fn env_name(option: &str) -> String {
    format!("ACTIT_{}", option.to_uppercase().replace('-', "_"))
}

#[cfg(test)]
mod tests {
    include!("settings_tests.rs");
}
