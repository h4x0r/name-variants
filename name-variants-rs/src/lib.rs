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
