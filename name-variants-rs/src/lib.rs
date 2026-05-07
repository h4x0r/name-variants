//! Multilingual name romanization lookup tables.
//!
//! Maps romanization variants to their canonical native-script key so that
//! `Chen`, `Chan`, and `Tan` all resolve to `陈`.

mod generated;

/// Return the canonical script-form key for a name, or `None` if unknown.
///
/// Matching order:
/// 1. Exact match (handles native script input like `陈`)
/// 2. Lowercase match (handles `Chan`, `CHAN`)
/// 3. Token-by-token (handles `"Chan Wai Ming"` → checks `chan`)
///
/// # Examples
/// ```no_run
/// use name_variants::lookup_key;
/// assert_eq!(lookup_key("Chan"), Some("陈"));
/// assert_eq!(lookup_key("Smith"), None);
/// ```
pub fn lookup_key(text: &str) -> Option<&'static str> {
    // stub — always returns None until generated map is wired in
    let _ = text;
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Chinese ──────────────────────────────────────────────────────────────
    #[test]
    fn chan_and_chen_same_key() {
        assert_eq!(lookup_key("chan"), lookup_key("chen"));
    }

    #[test]
    fn chan_resolves_to_simplified_chinese() {
        assert_eq!(lookup_key("chan"), Some("陈"));
    }

    #[test]
    fn hui_and_xu_same_key() {
        // 許/许: Xu (Mandarin), Hui (Cantonese), Kho (Hokkien)
        assert_eq!(lookup_key("hui"), lookup_key("xu"));
    }

    #[test]
    fn wang_and_wong_same_key() {
        assert_eq!(lookup_key("wang"), lookup_key("wong"));
    }

    // ── Korean ───────────────────────────────────────────────────────────────
    #[test]
    fn park_and_bak_same_key() {
        assert_eq!(lookup_key("park"), lookup_key("bak"));
    }

    #[test]
    fn lee_and_rhee_same_key() {
        assert_eq!(lookup_key("lee"), lookup_key("rhee"));
    }

    // ── Arabic ───────────────────────────────────────────────────────────────
    #[test]
    fn muhammad_and_mohammed_same_key() {
        assert_eq!(lookup_key("muhammad"), lookup_key("mohammed"));
    }

    // ── Russian ──────────────────────────────────────────────────────────────
    #[test]
    fn ivanov_and_ivanoff_same_key() {
        assert_eq!(lookup_key("ivanov"), lookup_key("ivanoff"));
    }

    // ── Case-insensitive ─────────────────────────────────────────────────────
    #[test]
    fn uppercase_input_matches() {
        assert_eq!(lookup_key("CHAN"), Some("陈"));
    }

    #[test]
    fn mixed_case_input_matches() {
        assert_eq!(lookup_key("Chan"), Some("陈"));
    }

    // ── Multi-word token split ────────────────────────────────────────────────
    #[test]
    fn full_name_resolves_via_token_split() {
        // "chan wai ming" → first token "chan" matches
        assert_eq!(lookup_key("Chan Wai Ming"), Some("陈"));
    }

    #[test]
    fn korean_full_name_resolves() {
        assert_eq!(lookup_key("Park Ji-sung"), lookup_key("park"));
    }

    // ── Unknown names ────────────────────────────────────────────────────────
    #[test]
    fn unknown_returns_none() {
        assert_eq!(lookup_key("Smith"), None);
        assert_eq!(lookup_key("Kowalski"), None);
        assert_eq!(lookup_key(""), None);
    }

    #[test]
    fn all_unknown_tokens_returns_none() {
        assert_eq!(lookup_key("John Smith"), None);
    }
}
