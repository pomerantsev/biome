use biome_analyze::{
    Ast, Rule, RuleDiagnostic, context::RuleContext, declare_lint_rule,
};
use biome_console::markup;
use biome_js_syntax::JsIdentifierBinding;
use biome_rowan::AstNode;
use biome_rule_options::no_abcd::NoAbcdOptions;

declare_lint_rule! {
    /// Disallow `let` or `const` bindings named `abcd`.
    ///
    /// This is a test rule created for learning purposes.
    ///
    /// ## Examples
    ///
    /// ### Invalid
    ///
    /// ```js,expect_diagnostic
    /// let abcd = 1;
    /// ```
    ///
    /// ```js,expect_diagnostic
    /// const abcd = 2;
    /// ```
    ///
    /// ### Valid
    ///
    /// ```js
    /// let abc = 1;
    /// ```
    ///
    /// ```js
    /// const abcde = 2;
    /// ```
    ///
    pub NoAbcd {
        version: "next",
        name: "noAbcd",
        language: "js",
        recommended: false,
    }
}

impl Rule for NoAbcd {
    type Query = Ast<JsIdentifierBinding>;
    type State = ();
    type Signals = Option<Self::State>;
    type Options = NoAbcdOptions;

    fn run(ctx: &RuleContext<Self>) -> Option<Self::State> {
        let binding = ctx.query();
        let name = binding.name_token().ok()?;
        let name = name.text_trimmed();

        if name == "abcd" {
            Some(())
        } else {
            None
        }
    }

    fn diagnostic(ctx: &RuleContext<Self>, _state: &Self::State) -> Option<RuleDiagnostic> {
        let binding = ctx.query();

        Some(
            RuleDiagnostic::new(
                rule_category!(),
                binding.syntax().text_trimmed_range(),
                markup! {
                    "Don't use "<Emphasis>"abcd"</Emphasis>" as a variable name."
                },
            )
            .note(markup! {
                "This name is not descriptive enough."
            }),
        )
    }
}
