use biome_analyze::{Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule};
use biome_console::markup;
use biome_js_syntax::JsIdentifierBinding;
use biome_rowan::AstNode;
use biome_rule_options::no_abcd::NoAbcdOptions;

use biome_browser_compat_metadata::{api_not_available, get_api_support, get_api_version_added};

declare_lint_rule! {
    /// Disallow using browser APIs not available in target browsers.
    ///
    /// This rule flags any `let`, `const`, or `var` binding whose name
    /// matches a browser API that is not available in the specified target
    /// browser versions.
    ///
    /// When target browsers are specified, the rule only reports APIs that
    /// were added AFTER the target version (i.e., not available in that version).
    ///
    /// ## Options
    ///
    /// ```json
    /// {
    ///   "targets": {
    ///     "chrome": "50",
    ///     "firefox": "45"
    ///   }
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// With `targets: { "chrome": "50" }`:
    ///
    /// ```js,expect_diagnostic
    /// // PaymentRequest was added in Chrome 60, not available in Chrome 50
    /// const PaymentRequest = {};
    /// ```
    ///
    /// ### Valid
    ///
    /// With `targets: { "chrome": "50" }`:
    ///
    /// ```js
    /// // fetch was added in Chrome 42, available in Chrome 50
    /// const fetch = () => {};
    /// ```
    ///
    pub NoAbcd {
        version: "next",
        name: "noAbcd",
        language: "js",
        recommended: false,
    }
}

pub struct State {
    api_name: Box<str>,
    unavailable_in: Vec<(Box<str>, Box<str>)>, // (browser, target_version)
}

impl Rule for NoAbcd {
    type Query = Ast<JsIdentifierBinding>;
    type State = State;
    type Signals = Option<Self::State>;
    type Options = NoAbcdOptions;

    fn run(ctx: &RuleContext<Self>) -> Option<Self::State> {
        let binding = ctx.query();
        let name = binding.name_token().ok()?;
        let name = name.text_trimmed();

        // Check if this is a known browser API
        let _support = get_api_support(name)?;

        let options = ctx.options();

        // If no targets specified, don't report anything
        // (in a real rule, you might want different default behavior)
        let targets = options.targets.as_ref()?;

        // Check if the API is unavailable in any of the target browsers
        let mut unavailable_in = Vec::new();

        for (browser, target_version) in targets.iter() {
            // Get when this API was added to this browser
            if let Some(version_added) = get_api_version_added(name, browser) {
                // Check if it was added AFTER the target version (not available)
                if api_not_available(version_added, target_version) {
                    unavailable_in.push((browser.clone(), target_version.clone()));
                }
            } else {
                // API not supported at all in this browser
                unavailable_in.push((browser.clone(), target_version.clone()));
            }
        }

        if unavailable_in.is_empty() {
            // API is available in all target browsers
            None
        } else {
            Some(State {
                api_name: name.into(),
                unavailable_in,
            })
        }
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let binding = ctx.query();

        // Build the list of browsers where the API is unavailable
        let browser_list: Vec<String> = state
            .unavailable_in
            .iter()
            .map(|(browser, version)| format!("{browser} {version}"))
            .collect();
        let browsers_str = browser_list.join(", ");

        Some(
            RuleDiagnostic::new(
                rule_category!(),
                binding.syntax().text_trimmed_range(),
                markup! {
                    "The "<Emphasis>{&state.api_name}</Emphasis>" API is not available in: "{&browsers_str}"."
                },
            )
            .note(markup! {
                "This API was added after the target browser version(s) you specified."
            }),
        )
    }
}
