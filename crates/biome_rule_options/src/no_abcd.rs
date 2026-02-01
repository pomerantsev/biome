use biome_deserialize_macros::{Deserializable, Merge};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

/// Options for the `noAbcd` rule.
///
/// Allows specifying target browser versions. The rule will only report
/// APIs that are not available in the specified browser versions.
///
/// ## Example configuration
///
/// ```json
/// {
///   "targets": {
///     "chrome": "50",
///     "firefox": "45",
///     "safari": "10.1"
///   }
/// }
/// ```
#[derive(Default, Clone, Debug, Deserialize, Deserializable, Merge, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct NoAbcdOptions {
    /// Target browser versions. Keys are browser names (e.g., "chrome", "firefox", "safari"),
    /// values are version strings (e.g., "50", "10.1").
    ///
    /// Only APIs that are not available in ALL specified browsers will be reported.
    /// If no targets are specified, the rule reports all browser API shadowing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub targets: Option<FxHashMap<Box<str>, Box<str>>>,
}
