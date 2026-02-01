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

### 3. Adding external config to a rule
- [ ] Understand how `NoUndeclaredDependencies` reads `package.json`
- [ ] Understand the service-based architecture (`ServiceBag`, `FromServices`)
- [ ] Prototype: create a rule that reads a simple config value

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

- [ ] Build a working rule with all pieces integrated
- [ ] Test against local JS/TS code
- [ ] Write docs following Biome's patterns
- [ ] Write tests following Biome's patterns
- [ ] Evaluate: how much code? how well does it fit?

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
