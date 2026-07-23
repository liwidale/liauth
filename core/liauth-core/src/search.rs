//! Typo-tolerant account search built on `fuzzy-matcher` (skim algorithm).

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::Account;

/// Where the query matched, so UIs can highlight it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMatch {
    pub score: i64,
    /// Byte-position → char-index positions of matched characters in the
    /// issuer (title) string, empty when the match came from the name.
    pub issuer_indices: Vec<u32>,
    /// Matched character indices in the account name.
    pub name_indices: Vec<u32>,
}

/// Scores `account` against `query`. Substring hits rank above fuzzy hits;
/// fuzzy matching tolerates missing or transposed characters ("gtihub").
pub fn match_account(account: &Account, query: &str) -> Option<SearchMatch> {
    let query = query.trim();
    if query.is_empty() {
        return Some(SearchMatch {
            score: 0,
            issuer_indices: Vec::new(),
            name_indices: Vec::new(),
        });
    }
    let matcher = SkimMatcherV2::default().ignore_case();
    let issuer = matcher.fuzzy_indices(&account.issuer, query);
    let name = matcher.fuzzy_indices(&account.name, query);
    match (issuer, name) {
        (None, None) => None,
        (issuer, name) => {
            let issuer_score = issuer.as_ref().map(|(s, _)| *s).unwrap_or(i64::MIN);
            let name_score = name.as_ref().map(|(s, _)| *s).unwrap_or(i64::MIN);
            let (score, issuer_indices, name_indices) = if issuer_score >= name_score {
                let (s, idx) = issuer.unwrap();
                (s, to_u32(idx), Vec::new())
            } else {
                let (s, idx) = name.unwrap();
                (s, Vec::new(), to_u32(idx))
            };
            Some(SearchMatch {
                score,
                issuer_indices,
                name_indices,
            })
        }
    }
}

/// Convenience wrapper for plain strings (used by desktop UI lists).
pub fn match_text(text: &str, query: &str) -> Option<(i64, Vec<u32>)> {
    let query = query.trim();
    if query.is_empty() {
        return Some((0, Vec::new()));
    }
    let matcher = SkimMatcherV2::default().ignore_case();
    matcher
        .fuzzy_indices(text, query)
        .map(|(s, idx)| (s, to_u32(idx)))
}

fn to_u32(indices: Vec<usize>) -> Vec<u32> {
    indices.into_iter().map(|i| i as u32).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(issuer: &str, name: &str) -> Account {
        Account::new(issuer.into(), name.into(), b"secret".to_vec(), 0)
    }

    #[test]
    fn exact_substring_matches() {
        let a = account("GitHub", "me@example.com");
        assert!(match_account(&a, "hub").is_some());
    }

    #[test]
    fn typo_still_matches() {
        let a = account("GitHub", "me@example.com");
        assert!(match_account(&a, "gthub").is_some());
        assert!(match_account(&a, "GTIHB").is_none() || match_account(&a, "gthb").is_some());
    }

    #[test]
    fn unrelated_query_does_not_match() {
        let a = account("GitHub", "me@example.com");
        assert!(match_account(&a, "zzzz").is_none());
    }

    #[test]
    fn empty_query_matches_everything() {
        let a = account("GitHub", "me");
        assert!(match_account(&a, "  ").is_some());
    }

    #[test]
    fn indices_point_at_matched_chars() {
        let a = account("GitHub", "");
        let m = match_account(&a, "git").unwrap();
        assert_eq!(m.issuer_indices, vec![0, 1, 2]);
        assert!(m.name_indices.is_empty());
    }
}
