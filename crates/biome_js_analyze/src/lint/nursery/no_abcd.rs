use biome_analyze::{
    Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_js_syntax::JsIdentifierBinding;
use biome_rowan::AstNode;
use biome_rule_options::no_abcd::NoAbcdOptions;

use biome_browser_compat_metadata::BROWSER_APIS;

declare_lint_rule! {
    /// Disallow variable bindings that shadow browser APIs.
    ///
    /// This rule flags any `let`, `const`, or `var` binding whose name
    /// matches a browser API from the MDN compatibility data.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```js,expect_diagnostic
    /// const fetch = () => {};
    /// ```
    ///
    /// ```js,expect_diagnostic
    /// let URL = "example";
    /// ```
    ///
    /// ### Valid
    ///
    /// ```js
    /// const myFetch = () => {};
    /// ```
    ///
    /// ```js
    /// let myURL = "example";
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

        if BROWSER_APIS.contains(&name) {
            Some(State {
                api_name: name.into(),
            })
        } else {
            None
        }
    }

    fn diagnostic(ctx: &RuleContext<Self>, state: &Self::State) -> Option<RuleDiagnostic> {
        let binding = ctx.query();

        Some(
            RuleDiagnostic::new(
                rule_category!(),
                binding.syntax().text_trimmed_range(),
                markup! {
                    "Don't shadow the browser API "<Emphasis>{&state.api_name}</Emphasis>"."
                },
            )
            .note(markup! {
                "Shadowing browser APIs can lead to confusion and unexpected behavior."
            }),
        )
    }
}
