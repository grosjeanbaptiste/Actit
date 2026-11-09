use crate::error::Error;
use crate::name::valid;
use crate::ports::RepoNames;

/// Repositories, under any owner, whose name contains a text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    /// What GitHub reports for the query; may exceed `names`, which holds the first 100.
    pub total: u64,
    pub names: Vec<String>,
}

pub fn list(name: &str, names: &impl RepoNames) -> Result<Listing, Error> {
    if !valid(name) {
        return Err(Error::InvalidName(name.to_string()));
    }
    names.contains(name)
}

/// Same convention as a need: 0 found, 1 not found.
pub fn exit_code(listing: &Listing) -> i32 {
    i32::from(listing.total == 0 && listing.names.is_empty())
}

pub fn render(name: &str, listing: &Listing) -> String {
    if exit_code(listing) == 1 {
        return format!("NONE {name} (no GitHub repository name contains it)\n");
    }
    let mut text = format!(
        "CONTAINS {name}: GitHub reports {} matches, {} listed\n",
        listing.total,
        listing.names.len()
    );
    for full in &listing.names {
        text.push_str(&format!("- {full}\n"));
    }
    text
}

#[cfg(test)]
mod tests {
    include!("contains_tests.rs");
}
