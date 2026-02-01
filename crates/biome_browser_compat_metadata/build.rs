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

    // Extract API names from the "api" key
    let api_names: Vec<&String> = data
        .get("api")
        .and_then(|v| v.as_object())
        .map(|obj| obj.keys().collect())
        .unwrap_or_default();

    let mut code = String::new();
    writeln!(code, "/// Browser API names from @mdn/browser-compat-data.").unwrap();
    writeln!(code, "/// Total: {} APIs.", api_names.len()).unwrap();
    writeln!(code, "pub const BROWSER_APIS: &[&str] = &[").unwrap();
    for name in &api_names {
        writeln!(code, "    \"{name}\",").unwrap();
    }
    writeln!(code, "];").unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();
    fs::write(PathBuf::from(out_dir).join("browser_apis.rs"), code)?;

    Ok(())
}
