//! Browser API metadata from @mdn/browser-compat-data.
//!
//! This crate provides browser API support data extracted from MDN's
//! browser-compat-data. The data is generated at build time directly from
//! `node_modules/@mdn/browser-compat-data/data.json`.

include!(concat!(env!("OUT_DIR"), "/browser_apis.rs"));

/// Look up browser support information for an API by name.
pub fn get_api_support(api_name: &str) -> Option<&'static BrowserSupport> {
    BROWSER_APIS
        .binary_search_by_key(&api_name, |(name, _)| *name)
        .ok()
        .map(|idx| &BROWSER_APIS[idx].1)
}

/// Get the version_added for a specific API and browser.
/// Returns None if the API doesn't exist or the browser doesn't support it.
///
/// This is a convenience wrapper around `get_api_support` + `get_version_added`.
pub fn get_api_version_added(api_name: &str, browser: &str) -> Option<&'static str> {
    let support = get_api_support(api_name)?;
    get_version_added(support, browser)
}

/// Compare two version strings.
/// Returns true if `version_added` > `target_version` (i.e., API was added AFTER the target).
/// This means the API is NOT available in the target browser version.
///
/// Examples:
/// - compare_versions("60", "50") -> true (60 > 50, not available)
/// - compare_versions("42", "50") -> false (42 <= 50, available)
/// - compare_versions("10.1", "10") -> true (10.1 > 10, not available)
/// - compare_versions("10.1", "10.2") -> false (10.1 <= 10.2, available)
pub fn api_not_available(version_added: &str, target_version: &str) -> bool {
    let added_parts = parse_version(version_added);
    let target_parts = parse_version(target_version);

    // Pad shorter version with zeros for comparison
    let max_len = added_parts.len().max(target_parts.len());
    let added_padded = pad_version(&added_parts, max_len);
    let target_padded = pad_version(&target_parts, max_len);

    // Compare component by component
    for (added, target) in added_padded.iter().zip(target_padded.iter()) {
        if added > target {
            return true; // version_added is higher, API not available
        }
        if added < target {
            return false; // version_added is lower, API is available
        }
    }

    // All parts equal
    false
}

/// Pad a version vector with zeros to the specified length.
fn pad_version(parts: &[u32], len: usize) -> Vec<u32> {
    let mut padded = parts.to_vec();
    padded.resize(len, 0);
    padded
}

/// Parse a version string into numeric components.
/// "10.1.2" -> [10, 1, 2]
/// "42" -> [42]
fn parse_version(version: &str) -> Vec<u32> {
    version
        .split('.')
        .filter_map(|part| part.parse::<u32>().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_not_available() {
        // API added in version 60, targeting version 50 -> not available
        assert!(api_not_available("60", "50"));

        // API added in version 42, targeting version 50 -> available
        assert!(!api_not_available("42", "50"));

        // API added in version 10.1, targeting version 10 -> not available
        assert!(api_not_available("10.1", "10"));

        // API added in version 10.1, targeting version 10.2 -> available
        assert!(!api_not_available("10.1", "10.2"));

        // API added in version 10, targeting version 10 -> available (equal)
        assert!(!api_not_available("10", "10"));

        // API added in version 10.0, targeting version 10 -> available
        assert!(!api_not_available("10.0", "10"));
    }

    #[test]
    fn test_get_api_support() {
        // fetch should exist
        let fetch = get_api_support("fetch");
        assert!(fetch.is_some());
        assert_eq!(fetch.unwrap().chrome, Some("42"));

        // nonexistent API
        assert!(get_api_support("nonexistent_api_xyz").is_none());
    }

    #[test]
    fn test_get_api_version_added() {
        assert_eq!(get_api_version_added("fetch", "chrome"), Some("42"));
        assert_eq!(get_api_version_added("fetch", "firefox"), Some("39"));
        assert_eq!(get_api_version_added("fetch", "invalid_browser"), None);
        assert_eq!(get_api_version_added("nonexistent", "chrome"), None);
    }
}
