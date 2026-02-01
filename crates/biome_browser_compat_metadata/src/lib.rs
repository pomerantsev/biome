//! Browser API metadata from @mdn/browser-compat-data.
//!
//! This crate provides a list of browser API names extracted from MDN's
//! browser-compat-data. The data is generated at build time directly from
//! `node_modules/@mdn/browser-compat-data/data.json`.

include!(concat!(env!("OUT_DIR"), "/browser_apis.rs"));
