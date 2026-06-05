# BRIEF — Stone 249.3b — `->`/`->>` reborn as wat macros; HARD-CUT `thread_desugar`

**Arc:** 249 (total-pure-macros). **Design:** `DESIGN-STONE-249.3.md` §3.
**Contract:** `tests/probe_arc249_threading.rs` — the 5 threading mints (currently passing via the Rust `thread_desugar`; they must STAY green through the swap to wat).
**Probe-validated encodings:** `tests/probe_arc249_threading_in_wat.rs` rows A/B (`->>` list-step), F (`->` list-step) — the wat macro bodies below are those exact, now-green forms.
**You write wat + substrate Rust. Do NOT commit. Do NOT run git. Leave the core.wat re-ward to the orchestrator.**

The engine vocabulary is in place (249.3a + 249.3a-ii): `~@`-splice, `:wat::core::List?`, and `first`/`rest` over form-values, all fenced. Threading is now expressible as ~8 lines of wat. This stone moves the desugar LOGIC from Rust to wat and HARD-CUTs the Rust implementation.

---

## Change 1 — the two wat macros in `wat/core.wat`

Add, near the existing `(:wat::core::defmacro :wat::core::defn …)` at core.wat:163 (threading is core dialect — it lives beside `defn`; intueri verdict):

```wat
;; Thread-first `->`: inject acc as the FIRST arg of each step.
;;   (-> x (f a b) g)  =>  (g (f x a b))
;; A list step `(f a…)` => `(f acc a…)`; a bare symbol/keyword step `f` => `(f acc)`.
(:wat::core::defmacro :wat::core::-> [acc <- :wat::holon::HolonAST & steps <- :AST<wat::holon::Holons>]
  -> :AST<wat::holon::HolonAST>
  (:wat::core::foldl
    (:wat::core::fn [a <- :wat::holon::HolonAST step <- :wat::holon::HolonAST]
       -> :wat::holon::HolonAST
       (:wat::core::if (:wat::core::List? step) -> :AST<wat::holon::HolonAST>
          `(~(:wat::core::Option/expect -> :wat::holon::HolonAST (:wat::core::first step) "-> step has no head") ~a ~@(:wat::core::rest step))
          `(~step ~a)))
    acc
    steps))

;; Thread-last `->>`: inject acc as the LAST arg of each step.
;;   (->> x (f a b) g)  =>  (g (f a b x))
;; A list step `(f a…)` => `(f a… acc)`; a bare symbol/keyword step `f` => `(f acc)`.
(:wat::core::defmacro :wat::core::->> [acc <- :wat::holon::HolonAST & steps <- :AST<wat::holon::Holons>]
  -> :AST<wat::holon::HolonAST>
  (:wat::core::foldl
    (:wat::core::fn [a <- :wat::holon::HolonAST step <- :wat::holon::HolonAST]
       -> :wat::holon::HolonAST
       (:wat::core::if (:wat::core::List? step) -> :AST<wat::holon::HolonAST>
          `(~@step ~a)
          `(~step ~a)))
    acc
    steps))
```

These register via the baked-stdlib loader's `register_stdlib` (registry.rs:77 — bypasses the reserved-prefix gate for trusted stdlib, exactly as `:wat::core::defn` does). The exact `<- :AST<…>` type annotations match the probe-green macros (rows A/B/F) — mirror them precisely.

## Change 2 — the bare-head routing seam in `src/macros/expand.rs`

The contract calls threading with a BARE symbol head (`(-> 5 …)`, `(->> [1 2 3] …)`), not `(:wat::core::-> …)`. Today expand.rs:144-154 recognizes the bare `->`/`->>` head and calls `thread_desugar`. Replace that call with a thin REWRITE to the keyword macro, then re-dispatch through the existing registered-macro path:

```rust
if let Some(WatAST::Symbol(head, head_span)) = expanded_children.first() {
    let kw = match head.as_str() {
        "->" => Some(":wat::core::->"),
        "->>" => Some(":wat::core::->>"),
        _ => None,
    };
    if let Some(kw) = kw {
        // Arc 249 Stone 249.3b — bare threading head → its :wat::core:: keyword macro.
        // The desugar LOGIC now lives in wat/core.wat; this is the thin syntax-level
        // seam (Clojure-faithful bare `->`/`->>` call surface). Re-dispatch hits the
        // registered-macro path below (registry.get → expand_macro_call).
        let mut rewritten = expanded_children.clone();
        rewritten[0] = WatAST::Keyword(kw.to_string().into(), head_span.clone());
        return expand_form(WatAST::List(rewritten, list_span), registry, depth + 1, env, sym);
    }
}
```

(Match the actual `WatAST::Keyword` / `Identifier` constructor signatures in the file — adapt the `.into()` to whatever the codebase uses for keyword text. The point is: bare head → keyword head → re-expand.)

## Change 3 — HARD-CUT `thread_desugar`

Delete the entire `thread_desugar` function (expand.rs:215-267) AND its leading doc-comment block (the `// ─── Arc 249 — threading macros` banner at ~190-214). After deletion, `grep -rn "thread_desugar" src/` must return ZERO matches. The logic is gone; only the wat macros + the syntax seam remain. (#65 — the desugar carried threading to the point it could be replaced; it falls willingly, honored in `git log`.)

## Verification (the scorecard — run every row yourself, report actual output)

1. **The contract** — `cargo test --release --test probe_arc249_threading` : ALL 5 mints green (the regression `regression_fn_first_map_no_threading` + the 4 threading mints `mint_thread_last_single_step` / `mint_thread_last_pipeline` / `mint_thread_first_injects_first` / `mint_thread_last_injects_last` / `mint_bare_symbol_step`). They pass today via Rust; they must STILL pass via the wat macros. Confirm zero `#[ignore]`.
2. **The desugar is gone** — `grep -rn "thread_desugar" src/` returns nothing.
3. **Engine + form-vocab probes** — `cargo test --release --test probe_arc249_macro_engine` (gates A–E) + `cargo test --release --test probe_arc249_threading_in_wat` (rows A/B/C/E/F) all green.
4. **Library** — `cargo test --release --lib -p wat` → ≥ 898/0/1.
5. **Full workspace sanity** — `cargo build --release` clean; `cargo clippy --release -p wat` zero new warnings on touched lines.

Report each row with actual output. If a contract mint goes red, STOP and report the exact diagnostic — do NOT adjust the contract or work around it (it is the spec; a red mint means the wat macro or the seam is wrong).

## Notes
- Bash + cargo work; use them freely.
- Files: `wat/core.wat` (the 2 macros) + `src/macros/expand.rs` (the seam rewrite + the `thread_desugar` deletion). No other files.
- `wat/core.wat` is a WARDED home (arc 245) — your additions drift its stamp; the orchestrator re-wards it (with a new threading deftest) after verifying your work. Make the macros correct and obvious; mirror the probe-green forms exactly.
- The macros are PURE-TOTAL programs over forms — they use only `foldl`/`if`/`List?`/`first`/`rest`/`Option/expect`/quasiquote, all on the engine's allow-list. The fence guarantees no effect can leak at expand time.
