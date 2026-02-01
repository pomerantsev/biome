//! Generates browser API metadata from @mdn/browser-compat-data

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::{env, fmt::Write};

const MDN_DATA: &str = "../../node_modules/@mdn/browser-compat-data/data.json";

fn main() -> io::Result<()> {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed={MDN_DATA}");

    let json = fs::read_to_string(MDN_DATA)?;
    let data: BTreeMap<String, serde_json::Value> = serde_json::from_str(&json)?;

    // Extract browser names from the "browsers" key (sorted for deterministic output)
    let browsers: Vec<&String> = data
        .get("browsers")
        .and_then(|v| v.as_object())
        .map(|obj| obj.keys().collect())
        .unwrap_or_default();

    // Extract API data
    let apis = data
        .get("api")
        .and_then(|v| v.as_object())
        .expect("Missing 'api' key in MDN data");

    let mut code = String::new();

    // Generate BrowserSupport struct
    writeln!(code, "/// Browser support information for an API.").unwrap();
    writeln!(code, "/// Generated from @mdn/browser-compat-data.").unwrap();
    writeln!(code, "#[derive(Debug, Clone, Copy)]").unwrap();
    writeln!(code, "pub struct BrowserSupport {{").unwrap();
    for browser in &browsers {
        let field_name = browser.replace('-', "_"); // e.g., "chrome-android" -> "chrome_android"
        writeln!(code, "    pub {field_name}: Option<&'static str>,").unwrap();
    }
    writeln!(code, "}}").unwrap();
    writeln!(code).unwrap();

    // Generate list of valid browser names (for validation)
    writeln!(code, "/// Valid browser names from MDN data.").unwrap();
    writeln!(code, "pub const BROWSER_NAMES: &[&str] = &[").unwrap();
    for browser in &browsers {
        writeln!(code, "    \"{browser}\",").unwrap();
    }
    writeln!(code, "];").unwrap();
    writeln!(code).unwrap();

    // Generate BROWSER_APIS constant
    writeln!(
        code,
        "/// Browser API support data from @mdn/browser-compat-data."
    )
    .unwrap();
    writeln!(code, "/// Total: {} APIs.", apis.len()).unwrap();
    writeln!(
        code,
        "pub const BROWSER_APIS: &[(&str, BrowserSupport)] = &["
    )
    .unwrap();

    for (api_name, api_value) in apis {
        // Get the __compat.support object for this API
        let support = api_value
            .get("__compat")
            .and_then(|c| c.get("support"))
            .and_then(|s| s.as_object());

        writeln!(code, "    (\"{api_name}\", BrowserSupport {{").unwrap();

        for browser in &browsers {
            let field_name = browser.replace('-', "_");
            let version = support
                .and_then(|s| s.get(*browser))
                .and_then(|v| extract_version_added(v));

            match version {
                Some(v) => writeln!(code, "        {field_name}: Some(\"{v}\"),").unwrap(),
                None => writeln!(code, "        {field_name}: None,").unwrap(),
            }
        }

        writeln!(code, "    }}),").unwrap();
    }

    writeln!(code, "];").unwrap();
    writeln!(code).unwrap();

    // Generate get_version_added function with dynamic match arms
    writeln!(code, "/// Get the version_added for a specific API and browser.").unwrap();
    writeln!(code, "/// Returns None if the API doesn't exist or the browser doesn't support it.").unwrap();
    writeln!(code, "pub fn get_version_added(support: &BrowserSupport, browser: &str) -> Option<&'static str> {{").unwrap();
    writeln!(code, "    match browser {{").unwrap();
    for browser in &browsers {
        let field_name = browser.replace('-', "_");
        writeln!(code, "        \"{browser}\" => support.{field_name},").unwrap();
    }
    writeln!(code, "        _ => None,").unwrap();
    writeln!(code, "    }}").unwrap();
    writeln!(code, "}}").unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();
    fs::write(PathBuf::from(out_dir).join("browser_apis.rs"), code)?;

    Ok(())
}

/// Extract version_added from a support entry.
/// The entry can be an object like `{"version_added": "42"}` or an array of such objects.
/// Returns None if version_added is false or missing.
fn extract_version_added(value: &serde_json::Value) -> Option<&str> {
    // If it's an array, take the first entry (primary support)
    let obj = if let Some(arr) = value.as_array() {
        arr.first()?.as_object()?
    } else {
        value.as_object()?
    };

    let version = obj.get("version_added")?;

    // version_added can be a string ("42"), boolean (false), or null
    if let Some(s) = version.as_str() {
        Some(s)
    } else {
        None // false or null means not supported
    }
}
