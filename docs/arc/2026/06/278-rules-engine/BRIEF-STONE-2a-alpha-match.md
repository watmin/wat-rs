# BRIEF — Stone 2a: `alpha-match` (the rete single-fact matcher)

Single-hop sonnet in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** A Rust stone: one PURE
intrinsic in a NEW `src/rete/` home. Build, run the named tests, report verbatim. Another agent weighs.

## The work
Add `(:wat::rete::alpha-match [cond <- :wat::WatAST  fact <- :wat::Record] -> :wat::core::Option<wat::core::PersistentMap>)`:
given a condition form (DATA) and a fact (record), return `Some(bindings)` iff the fact's type == the
condition head AND every clause holds, else `None` (Clara no-error — never raise). PURE: no `Environment`, no
`eval_inner`. Bindings = a `PersistentMap` keyed by the logic-var name string (`"?t"`).

## Read FIRST (in order) and implement EXACTLY
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-2a-alpha-match.md` — the contract, the clause table, the
   PURITY crux (operands resolve ONLY from {bindings, field, literal}), the out-of-scope cuts.
2. `src/runtime.rs:10464` `eval_form_matches` + `:10559` `walk_match_clause` — **the reference for reading a
   struct's fields by name via the type registry** (`sym.types()` → `TypeDef::Struct` → field names →
   `struct_value.fields[idx]`) and **the numeric ordering compare** (the `Compare` arm ~:10615). MIRROR the
   field-read + comparison, but PURE: thread a bindings map, NOT an `Environment`; return `Option<map>`, NOT
   `bool`; resolve operands from bindings/field/literal, NEVER `eval_inner`. (This is why we don't reuse it.)
3. `src/runtime.rs:8708` `eval_quote` — `cond` arrives as `Value::wat__WatAST(Arc<WatAST>)`; extract the
   `WatAST` (a `List` `(:FactType clause…)`).
4. `src/form_match.rs:229` `logic_var_name` (`?`-symbol → name incl `?`) + `:239` `keyword_payload` (`:kw` →
   string) — both `pub`, REUSE them for the `?var` + `:field` extraction. Do NOT use `classify_clause` (it's
   form::matches?'s grammar — no `<-`, no FQDN ops).
5. `src/collection/mod.rs` + `src/collection/eval.rs` — the HOME pattern (mod.rs structure; how a collection
   intrinsic is dispatched from `runtime.rs` + carries a `check.rs` scheme). Mirror it for `src/rete/`.
6. `src/runtime.rs` dispatch (the `:wat::core::*` keyword-head match, e.g. ~:3817-3866) — add one arm
   `":wat::rete::alpha-match" => crate::rete::matcher::eval_alpha_match(args, list_span, env, sym)`.
7. `src/check.rs` `register_builtins` — register the scheme `[:wat::WatAST, :wat::Record] -> Option<PersistentMap>`
   (mirror a simple 2-arg intrinsic scheme; the `Option`/`PersistentMap` type heads exist).
8. `tests/probe_arc278_2a_alpha_match.rs` — remove the 3 `#[ignore]`s. It is your contract.

## The matcher (own classifier — classify each clause `List` by SHAPE)
- `[Symbol(?v), Symbol(<-), Keyword(:field)]` → **bind**: `bindings["?v"] = fact.<field>` (field index via the
  registry; field absent → `None`).
- `[Keyword(:wat::core::<op>), a, b]`, op ∈ `= not= < > <= >=` → **constraint**: resolve `a`,`b` (a `?v`-symbol
  → bindings; a `:field`-keyword → fact field; a literal → its `Value`), compare; false → `None`.
- `[Keyword(:wat::rete::and), sub…]` / `or` / `not` → combinators (thread bindings; `not` binds nothing).
- type-head: `(:wat::core::type fact)` (runtime.rs:12115, gives FQDN minus `:`) must == the cond head keyword
  minus `:`; else `None`.

## STOP triggers (HALT + report; do NOT improvise)
1. `(:wat::rete::where …)` clause → STOP (arbitrary-expr eval is stone 6; do not implement eval here).
2. A `?var` referenced in a constraint that was NOT bound earlier in THIS condition → that's the cross-fact
   JOIN (stone 3); STOP, report (do not invent join/unify semantics here).
3. Reading a struct field by name needs a registry path that doesn't exist / isn't reachable → STOP, name it.

## Verify (run each; paste VERBATIM)
```
cargo test --release -p wat --test probe_arc278_2a_alpha_match -- --include-ignored   # 3/3 GREEN
cargo test --release -p wat --lib 2>&1 | grep "test result"                          # 931/36 (UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"                           # 264/1 (UNCHANGED)
cargo test --release --test test_stdlib_load_order | grep result                     # 1/0
cargo build --release 2>&1 | tail -2                                                  # clean (no NEW warnings)
```
Report: the full `src/rete/matcher.rs` source + the dispatch arm + the check scheme + the mod wiring, all
outputs verbatim, any STOP hit. Un-ignore the 3 probe tests. No git.

## Blast radius
`src/rete/matcher.rs` + `src/rete/mod.rs` (new) · `src/lib.rs` (`mod rete;`) · `src/runtime.rs` (one dispatch
arm) · `src/check.rs` (one scheme) · the probe (un-ignore). NO new `Value` variant. NO `form_match.rs` change.
No git.
