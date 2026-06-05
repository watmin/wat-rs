# BRIEF — Stone 249.4a — keyword/of reborn as a wat macro; HARD-CUT construct_keyword_of

**Arc:** 249 (total-pure-macros). **Probe-validated encoding:** `tests/probe_arc249_4_rehome_in_wat.rs` row B (`diag_keyword_of_full` — the KW_OF_MACRO body is the macro you'll promote; RED at HEAD pending the engine extension).
**Contract:** `tests/wat_arc170_program_contracts.rs` (keyword/of builds channel types at macro-expand time — these must stay green).
**You write substrate Rust + wat. Do NOT commit. Do NOT run git. Leave the core.wat re-ward to the orchestrator.**

`keyword/of` builds the parametric keyword `:Head<arg1,arg2>` (head + args, colons stripped) — currently the Rust built-in `construct_keyword_of` (expand.rs:192). The total-pure engine + the keyword-form vocabulary make it expressible in wat. Three changes, in order.

## Change 1 — engine extension: `keyword/to-string` over a keyword FORM

`eval_keyword_to_string` (`src/runtime.rs:8606`) handles a keyword VALUE today. A macro program body binds keyword args as `Value::wat__WatAST(Keyword)` form-values — and `keyword/to-string` rejects them (`expected keyword, got wat::WatAST`). Add a `Value::wat__WatAST(ast)` arm: when `&*ast` is `WatAST::Keyword(text, _)`, return the SAME string the keyword-value arm returns (leading colon stripped — see the keyword-value path + the test `keyword_to_string_strips_leading_colon` at runtime.rs:30856). Mirror the value arm's exact stripping. (This is the keyword analog of 249.3a-ii's first/rest-over-form arms.)

**Verify Change 1 alone:** un-`#[ignore]` row B (`diag_keyword_of_full`) → `cargo test --release --test probe_arc249_4_rehome_in_wat diag_keyword_of_full` should now produce `Ok("foo<bar,baz>")`. If instead it reveals a NEW gap (string::join arg-order, or keyword/from-string not re-adding the colon on output), STOP and report the exact error — that is the next gap to close, not something to work around.

## Change 2 — the wat `:wat::core::keyword/of` macro in `wat/core.wat`

Once row B is green, promote that macro body to `wat/core.wat` (beside the threading macros / defn — core dialect) as the real `:wat::core::keyword/of`:

```wat
;; keyword/of — build the parametric keyword `:Head<arg1,arg2>` from keyword args
;; (head + args, leading colons stripped). Pure-total program over forms.
(:wat::core::defmacro :wat::core::keyword/of
  [head <- :wat::holon::HolonAST & args <- :AST<wat::holon::Holons>]
  -> :AST<wat::holon::HolonAST>
  (:wat::core::let [head-text (:wat::core::keyword/to-string head)
                    arg-texts (:wat::core::map
                                (:wat::core::fn [a <- :wat::holon::HolonAST] -> :wat::core::String
                                   (:wat::core::keyword/to-string a))
                                args)
                    joined (:wat::core::string::join arg-texts ",")
                    full (:wat::core::string::concat head-text
                           (:wat::core::string::concat "<"
                             (:wat::core::string::concat joined ">")))]
    `~(:wat::core::keyword/from-string full)))
```

(Adjust to whatever row B proved actually works — the probe is the source of truth. Registers via `register_stdlib`, like the threading macros + defn.)

## Change 3 — HARD-CUT `construct_keyword_of`

In `src/macros/expand.rs`: DELETE the keyword/of recognition arm (the `if head == ":wat::core::keyword/of" { return construct_keyword_of(...) }` block near expand.rs:133) AND the `construct_keyword_of` function (expand.rs:192 + its banner). keyword/of now dispatches through the registered-macro path (registry.get on the `:wat::core::keyword/of` keyword head), same as every other macro. `grep -rn "construct_keyword_of" src/` must return ZERO.

## Verification (run every row yourself; report actual output)

1. **The contract** — `cargo test --release --test wat_arc170_program_contracts` : all green. keyword/of builds the channel types via the wat macro now. **THE KEY RISK:** keyword/of currently fires in TEMPLATE positions too (inside quasiquotes, post-unquote-resolution — expand.rs comment). As a registered macro it should still fire there (the registered-macro fixpoint re-expands keyword-headed results). If a channel-type contract goes red because keyword/of does NOT fire in a template position, STOP and report it precisely — that is a real diagnostic about the rehome's reach, NOT something to work around.
2. **Probe** — `cargo test --release --test probe_arc249_4_rehome_in_wat` : row B green (un-ignored); rows A/C unaffected.
3. **Cut confirmed** — `grep -rn "construct_keyword_of" src/` → zero.
4. **Engine + threading intact** — `cargo test --release --test probe_arc249_macro_engine` + `probe_arc249_threading` + `probe_arc249_threading_in_wat` green.
5. **Library** — `cargo test --release --lib -p wat` → ≥ 898/0/1.
6. **Build + clippy** — `cargo build --release` clean; `cargo clippy --release -p wat` zero new warnings on touched lines.

## Reporting
- Leave ALL git to me. Report: exact edits per file, each verification row's command + output, the `grep construct_keyword_of` result, `git status --short` + `git diff --stat`.
- The probe (row B) is the source of truth for the macro body — if reality differs from the BRIEF's draft, follow the probe and report the delta.
- Files: `src/runtime.rs` (Change 1) + `wat/core.wat` (Change 2) + `src/macros/expand.rs` (Change 3). `wat/core.wat` is a warded home (245) — I re-ward after.