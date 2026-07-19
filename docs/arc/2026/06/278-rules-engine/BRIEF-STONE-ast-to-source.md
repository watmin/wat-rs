# BRIEF — Stone 1: `:wat::core::ast->source` (the sift Predicate's enabling primitive)

> Executor tier: sonnet shadowdancer. Orchestrator weighs by its own re-run.
> Design settled (DESIGN-sift-server-side-filter.md, "Predicate-form delivery"): B — a `WatAST → verbatim-::-source`
> printer. A (span→source) is infeasible; B is notation-agnostic (prints the AST's verbatim strings) so it survives
> the medium-term Clojure flip untouched. Disconfirming probe already run + grounded (see "The gap, proven" below).

## The work (one paragraph)
Add `:wat::core::ast->source` — a runtime primitive that serializes a `Value::wat__WatAST` back to **verbatim wat
source** (a `String`), printing every `::` keyword/symbol untouched. It is the resurrection the retired
`wat_ast_to_source` note explicitly invites (`crates/wat-reader/src/ast.rs:459-466`). It is the sibling of
`write-forms` — but `write-forms` goes through `watast_to_edn` + `wat_edn::write`, which **dials `::` → `.`** (GROUNDED:
`write-forms` on a `::`-form emits `:wat.core/fn`, not `:wat::core::fn`). `ast->source` must **walk the AST directly**
and emit the raw token text, so `read-string(ast->source(form))` reproduces the SAME form.

## Read in order (the rooms, each with why)
1. `crates/wat-reader/src/ast.rs:440-466` — the current `WatAST` variant set (`variant_name`) + the retired-printer
   note. **The current enum has 13 variants** (IntLit, FloatLit, RationalLit, BigIntLit, BoolLit, StringLit, NilLit,
   Keyword, Symbol, List, Vector, Map, Set), each carrying a `Span`.
2. `scratchpad/old_ast.rs` (the RETIRED printer, extracted from `b5bca8be^:src/ast.rs` lines 85-160) — the reference
   shape to resurrect: `IntLit → to_string`, `FloatLit → format!("{:?}", x)` (**keeps `3.0` as `3.0`, not `3`**),
   `BoolLit → true/false`, `StringLit → re-quoted + escaped (\\ \" \n \r \t)`, `Keyword → verbatim`,
   `Symbol → ident.name`, `List → "(" items-space-joined ")"`. **It predates 6 variants** — you must ADD
   `RationalLit`, `BigIntLit`, `NilLit → "nil"`, `Vector → "[…]"`, `Map → "{k v …}"`, `Set → "#{…}"`; and the Symbol
   field is now `ident.as_str()` (each variant now also carries a `Span`, which the printer ignores).
3. `src/edn_shim.rs:454-481` (`eval_write_forms`) + `:651-681` (`eval_ast_name`) — the EXACT arg-unwrap + return
   pattern to mirror: `require_one_arg(OP, …)?` → `match Value::wat__WatAST(a) => a.as_ref()` else `TypeMismatch` →
   return `TrackedValue::new(Value::String(Arc::new(text)), Provenance::RuntimeBuilt{producer: OP, call_span})`.
   NOTE the doc at `:445-451`: `write-forms` dials `::`→`.` — do NOT reuse `watast_to_edn`/`wat_edn::write`.
4. `src/edn_shim.rs:483-528` (`eval_ast_children`) — the sibling that already switches on List/Vector/Set/Map;
   reference for the child-container arms.
5. `src/runtime.rs:3976-3986` — the dispatch arms (`":wat::core::write-forms" => eval_write_forms`, etc.). Add
   `":wat::core::ast->source" => eval_ast_to_source` beside them.
6. `src/check.rs:19296-19306` — the `write-forms` `TypeScheme` (`params: [:wat::WatAST]`, `ret: :wat::core::String`).
   Register `":wat::core::ast->source"` IDENTICALLY (same `WatAST → String` shape).

## Implementation sketch (fill it; do not invent the shape)
```rust
// src/edn_shim.rs, beside eval_write_forms
pub fn eval_ast_to_source(args, list_span, env, sym) -> Result<TrackedValue, RuntimeError> {
    const OP: &str = ":wat::core::ast->source";
    let v = require_one_arg(OP, args, env, sym, list_span)?;
    let ast: &WatAST = match &v { Value::wat__WatAST(a) => a.as_ref(),
        other => return Err(/* TypeMismatch, expected ":wat::WatAST" — copy eval_write_forms */) };
    let mut out = String::new();
    write_wat_source(ast, &mut out);   // recursive, verbatim
    Ok(TrackedValue::new(Value::String(Arc::new(out)), Provenance::RuntimeBuilt { producer: OP, call_span: list_span.clone() }))
}
fn write_wat_source(ast: &WatAST, out: &mut String) { /* resurrect write_wat_ast + the 6 new variants */ }
```
**GROUND the Rational / BigInt / Set / Map literal spellings against the parser** (`crates/wat-reader/src/` reader /
`src/parser`) so `read-string` re-reads each to the identical node — the RED gate's round-trip forces this.

## Blast radius (bounded)
`src/edn_shim.rs` (one new fn + one helper) · `src/runtime.rs` (one dispatch arm) · `src/check.rs` (one `env.register`)
· `crates/wat-reader/src/ast.rs:459-466` (update the note — the primitive is reintroduced, in edn_shim). **No new
types. Do NOT touch `watast_to_edn` / `wat_edn::write` / `write-forms`.**

## STOP triggers (halt + surface; do not improvise)
- **STOP-1:** if any variant's parser spelling cannot be made to round-trip (`read-string(ast->source(x))` ≠ `x`),
  STOP and surface which variant + why — do not ship a spelling that re-reads to a different node.
- **STOP-2:** if the checker/purity fence rejects a caller of `ast->source` as impure, STOP and report — `ast->source`
  is pure ∧ deterministic; if a `pure?` entry is needed it belongs in the purity allowlist, but confirm the need first.

## The RED gate (install + make green)
Install the disconfirming probe as a co-located fixture — **copy the exact idiom of
`tests/rete/probe_arc278_accessor_purity.{rs,wat}`** (`call_beside(file!(), ":user::…")` → bool; `build.rs`
auto-wires it). Name it `tests/rete/probe_arc278_ast_to_source.{rs,wat}`. Assert:
1. **round-trip**: `(= form (first (ast->children (read-string (ast->source form)))))` = `true`, over a form that
   exercises List + Vector + Keyword + Symbol + a literal (the sift predicate shape:
   `(:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::+ x 1))`).
2. **verbatim `::`**: `(string::contains? (ast->source form) "::")` = `true` (the anti-`write-forms` assertion — the
   `::` is NOT dialed to `.`).
The scratchpad worked reference is `scratchpad/probe-ast-source-gap.wat` (program form; adapt to the `call_beside`
entry-fn form).

## Expectations
| what | command | expected |
|---|---|---|
| the gate is green | `cargo test --release -p wat ast_to_source` | pass |
| nothing else breaks | `cargo nextest run --release` (Summary line) | 0 new failures; only the standing `no_inlined_wat` at 351 |
Runtime: ~5–8 min. Trap-door: a literal spelling that round-trips wrong (STOP-1).
