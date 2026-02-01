# Biome Compat Rule Exploration

## Goal

Decide if creating a browser compatibility rule in Biome is viable. Identify challenges. If viable and challenges are clear, write an RFC.

**Success criteria:** Only present RFC to the community if it's rock solid.

---

## Learning Steps

### 1. Testing changes locally
- [x] How to build Biome locally
- [x] How to run a local build against test files
- [x] How to write test cases for quick prototyping

### 2. Creating a new rule
- [x] Understand the rule structure (`declare_lint_rule!`, `impl Rule`)
- [x] Create a minimal "hello world" rule
- [x] Understand how rules are registered and enabled

### 3. Adding rule options for browser targets
- [x] Update `build.rs` to generate richer data structure with browser support info
  - Generate `BrowserSupport` struct with all 17 browsers as fields
  - Generate `BROWSER_APIS: &[(&str, BrowserSupport)]` with version_added data
  - Browser list generated dynamically from MDN data (future-proof)
- [x] Define rule options format in `no_abcd.rs`
  - `targets: HashMap<String, String>` for browser → version mapping
  - Example: `{ "chrome": "50", "firefox": "45", "safari": "10.1" }`
- [x] Implement version comparison logic
  - Parse versions like "42", "10.1", handle `false` (not supported)
  - Compare: target version vs. version_added
- [x] Update rule logic
  - Warn only if API's version_added > target version (API not available yet)
  - Check against all specified target browsers

### 4. Using external data in a rule
- [x] Study `biome_aria_metadata` crate (build.rs, generated code)
- [x] Understand how `UseValidAriaRole` uses this data
- [x] Prototype: embed a small dataset and query it from a rule

---

## Architecture Notes

### Why not a plugin?

Biome plugins (GritQL) are designed for project-specific rules, not wide distribution. From the [distribution discussion](https://github.com/biomejs/biome/discussions/6265):

> "The initial idea behind this plugin system was to provide users the ability to create custom rules for their own projects. These rules would make sense only within an organisation or project, and shouldn't be shared or distributed."

GritQL also cannot access external data or configuration, which we need for browserslist and MDN compat data.

### Relevant prior art

- **eslint-plugin-compat** ([repo](https://github.com/amilajack/eslint-plugin-compat)) — our inspiration
- **biome_aria_metadata** — pattern for embedding external data
- **NoReactForwardRef** — pattern for reading package.json
- **NoUndeclaredDependencies** — pattern for manifest-based rules

### Scope

Starting trimmed-down:
- MDN compat data only (no caniuse-lite)
- JavaScript/TypeScript only
- CSS as a possible next step

---

## Prototype Plan

- [x] Build a working rule with all pieces integrated
- [x] Test against local JS/TS code
- [x] Write tests following Biome's patterns
- [ ] Write docs following Biome's patterns
- [ ] Evaluate: how much code? how well does it fit?

---

## Pre-RFC Tasks

### 1. Verify alignment with Biome philosophy
- [ ] Research Biome's stance on rules that require external configuration
- [ ] Check if there are precedents for rules with complex options like `targets`
- [ ] Understand how Biome handles rules that depend on project context (browserslist, tsconfig, etc.)
- [ ] Look for any discussions/issues about browser compat rules in Biome

### 2. Implement actual API usage detection
Current prototype only checks variable binding names. Real rule needs to detect actual API usage:
- [ ] Study how eslint-plugin-compat detects API usage
- [ ] `fetch()` — direct function calls
- [ ] `new PaymentRequest()` — constructor calls
- [ ] `navigator.serviceWorker` — property access on known objects
- [ ] `window.fetch` — property access with explicit global
- [ ] Decide: do we need type information, or can we do this with AST only?

### 3. Verify Renovate updates MDN data
- [ ] Check if Renovate is configured to update all npm dependencies
- [ ] If not, add `@mdn/browser-compat-data` to Renovate config

### 4. Polyfill support
Users shouldn't get warnings for APIs they've polyfilled.
- [ ] Study how eslint-plugin-compat handles polyfills
- [ ] Design config format for specifying polyfills
- [ ] Decide: list of API names? Or list of polyfill packages that map to APIs?

### 5. Other considerations
- [ ] **Rule naming**: `noAbcd` is a placeholder. Consider: `noUnsupportedBrowserApi`, `useCompatibleApi`, etc.
- [ ] **Error messages**: Should include which browsers don't support the API, maybe MDN links
- [ ] **Performance**: With 1077 APIs, ensure lookup is efficient (current binary search should be fine)
- [ ] **Scope of detection**: Start with global APIs only, or also detect prototype methods like `Array.prototype.includes`?

---

## Open Questions

(To be filled as we go)

---

## RFC Draft

(To be written after prototype validates the approach)

---

## Implementation

### noAbcd rule (learning exercise)

Created a minimal lint rule `noAbcd` that flags any `let`/`const`/`var` binding named `abcd`. This exercise helped understand the full workflow.

**Files created:**

| File | Purpose |
|------|---------|
| `crates/biome_js_analyze/src/lint/nursery/no_abcd.rs` | Rule implementation |
| `crates/biome_rule_options/src/no_abcd.rs` | Rule options (empty) |
| `crates/biome_js_analyze/tests/specs/nursery/noAbcd/invalid.js` | Test cases that trigger |
| `crates/biome_js_analyze/tests/specs/nursery/noAbcd/valid.js` | Test cases that pass |
| `crates/biome_js_analyze/tests/specs/nursery/noAbcd/*.snap` | Snapshot expectations |

**Files modified:**

| File | Change |
|------|--------|
| `crates/biome_diagnostics_categories/src/categories.rs` | Added `lint/nursery/noAbcd` |
| `crates/biome_rule_options/src/lib.rs` | Added `pub mod no_abcd;` |

**Key learnings:**

1. Rules use `declare_lint_rule!` macro + `impl Rule` trait
2. Query type determines what AST nodes to match (e.g., `Ast<JsIdentifierBinding>`)
3. `run()` returns `Some(state)` to signal an issue; `diagnostic()` builds the message
4. Tests go in `tests/specs/{group}/{ruleName}/`, snapshots via `cargo insta accept`
5. Must register diagnostic category and options module manually (or use codegen)

**Test project:**

Created `~/dev/playground/biome-examples/abcd-test/` to test the local Biome build:
- `example.ts` — file with `abcd` variable
- `biome.json` — enables `nursery/noAbcd`
- `package.json` — `npm run lint` runs local cargo build

### Using MDN compat data (external data prototype)

Extended the `noAbcd` rule to check against real browser API names from `@mdn/browser-compat-data`. The rule now flags any variable that shadows a browser API (e.g., `const fetch = ...`).

**New crate created:** `biome_browser_compat_metadata`

Following the `biome_aria_metadata` pattern:
- `build.rs` reads directly from `node_modules/@mdn/browser-compat-data/data.json`
- Generates Rust code at compile time (not committed)
- `lib.rs` uses `include!()` to embed the generated code

**Files:**

| File | Purpose |
|------|---------|
| `crates/biome_browser_compat_metadata/Cargo.toml` | Crate manifest with serde build-deps |
| `crates/biome_browser_compat_metadata/build.rs` | Reads MDN JSON, generates `BROWSER_APIS` const |
| `crates/biome_browser_compat_metadata/src/lib.rs` | Exports generated code |

**Key pattern:**

```
node_modules/@mdn/browser-compat-data/data.json
        ↓ (build.rs reads at compile time)
    OUT_DIR/browser_apis.rs
        ↓ (include!() in lib.rs)
    pub const BROWSER_APIS: &[&str] = &[...];
```

**Updated rule:** `no_abcd.rs` now imports `biome_browser_compat_metadata::BROWSER_APIS` and checks if variable names are in this list (1077 browser APIs).

**Dependency added:** `@mdn/browser-compat-data` as npm devDependency in root `package.json`. Renovate will auto-update it.

### Browser version targeting (compat prototype)

Extended the rule to check browser API availability against user-specified target versions. The rule now only reports APIs that are NOT available in the target browser versions.

**Config format:**

```json
{
  "linter": {
    "rules": {
      "nursery": {
        "noAbcd": {
          "level": "error",
          "options": {
            "targets": {
              "chrome": "50",
              "firefox": "45"
            }
          }
        }
      }
    }
  }
}
```

**Updated data generation:** `build.rs` now generates:

1. `BrowserSupport` struct with all 17 MDN browsers as fields (dynamically generated)
2. `BROWSER_APIS: &[(&str, BrowserSupport)]` with version_added data for each API
3. `BROWSER_NAMES: &[&str]` for validation
4. Helper functions: `get_api_support()`, `get_version_added()`, `api_not_available()`

**Version comparison:**

- Parses versions like "42", "10.1", "10.1.2"
- Compares numerically component by component
- Handles `None` (API not supported in browser)
- "10.0" == "10" (trailing zeros are equal)

**Rule logic:**

1. Check if variable name is a known browser API
2. If no targets specified, don't report (opt-in behavior)
3. For each target browser, check if API's version_added > target version
4. Report only if API is unavailable in at least one target browser

**Test results:**

With `targets: { "chrome": "50" }`:
- `PaymentRequest` (Chrome 60) → ❌ flagged
- `AbortController` (Chrome 66) → ❌ flagged
- `fetch` (Chrome 42) → ✅ not flagged
- `MutationObserver` (Chrome 18) → ✅ not flagged
