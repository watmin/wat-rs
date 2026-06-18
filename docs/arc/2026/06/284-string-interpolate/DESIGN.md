# Arc 284 — `:wat::core::string::interpolate` — the pure-total interpolation intrinsic

> **STATUS: SHIPPED (2026-06-17).** Weighed on own build + eyeballed: runtime `(string::interpolate "{a} = {b} ({{esc}})" :a "count" :b 42)` -> `"count = 42 ({esc})"`; EXPAND-TIME (in a defmacro body) `(string::interpolate "{x}::Op::{x}" :x bs)` -> `svc::Op::svc` — the load-bearing property format cannot do. interpolate 2/2, format 3/3 (untouched), lib 929/36, deftest 263/1, deporder 0 — additive. RED probe `tests/probe_arc284_interpolate.rs` (`#[ignore]`'d):
> undefined at HEAD. intueri-named (`string::interpolate`, `{name}` surface — NOT `#{}`, which would
> falsely promise inline-eval). The Good-UX cure for expand-time string-building, AND the concat→format
> sweep's missing target for macro-body concats.

## Why (the gap, grounded)

Building strings at EXPAND time (inside defmacro bodies — Record::def, defservice build keyword names)
has no ergonomic tool: the `format` macro is refused at expand time (arc 249 F5 purity gate), and
`concat`/`join` are clunky for literal-interleaved-with-values templates. The arc-277 sweep proved this
hard: rewriting macro-body concats to `format` broke the whole stdlib. `interpolate` is the cure — same
template grammar as `format`, but a **pure-total intrinsic** (interpolates at call time), so it is
**expand-time-legal**. `format` stays the zero-cost macro for hot runtime paths; `interpolate` is the
universal/fallback usable everywhere. The concat→format/interpolate fix then picks the head by position:
runtime → `format` macro; expand-time → `string::interpolate` intrinsic; ONE template shape (both `{name}`).

## The surface (intueri-blessed)

```clojure
(:wat::core::string::interpolate "{greeting}, {name}! you have {count} messages"
  :name "ada" :greeting "hello" :count 3)
;=> "hello, ada! you have 3 messages"
```
- **NAMED `{name}` placeholders** + trailing `:name val` kwargs (out-of-order OK) — IDENTICAL grammar to
  the `format` macro (so the codemod emits one shape). NOT `#{}` (that sigil promises inline-eval, which
  is impossible here — a Level-1 lie; held in reserve for true inline-interp).
- **Unquoted render** — each value via the `:wat::core::str` semantics (String→itself, i64→digits,
  f64/bool/u8→their text). Reuse `eval_str`'s match (`src/runtime.rs:16886`).
- **`{{`/`}}` escape** — `{{`→`{`, `}}`→`}` (same as `format`, arc 279.1).
- **Strict** — every `{name}` MUST have a matching `:name` (else RuntimeError naming the placeholder);
  every `:name` MUST be consumed (else RuntimeError naming the unused kwarg). Same strictness as `format`,
  but at runtime (RuntimeError, not MacroError). `:name` keys repeated `{name}` allowed (dedup-on-use).
- **Pure + total** — deterministic, no IO → goes on the macro-eval allow-list (expand-time-legal).

## The build (THE CONTRACT) — 5 sites

1. **`src/string_ops.rs` — `eval_string_interpolate(args, list_span, env, sym) -> Result<Value, RuntimeError>`:**
   - `args[0]` → eval → must be `Value::String` (the template; else TypeMismatch).
   - `args[1..]` → MUST be an even count of (keyword, value) pairs (else MalformedForm "…:name value pairs").
     For each pair: key = the keyword's name with the leading `:` stripped (e.g. `:name` → `"name"`);
     value = eval then render UNQUOTED via the `eval_str` match (factor a shared `render_unquoted(Value)
     -> Result<String>` helper, or inline the same arms). Build a `HashMap<String,String>` name→rendered
     + track insertion for the unused-check.
   - **Parse the template char-by-char** (mirror the `format` macro's `wat/core.wat:543-736` state machine,
     in Rust): text accumulates; `{{`→`{`, `}}`→`}`; `{name}` → look up `name` in the map (missing →
     RuntimeError naming it); a lone `{`/`}` mid-template that isn't a placeholder/double → RuntimeError.
   - After: every kwarg key must have been referenced by some `{name}` (else RuntimeError "unused kwarg").
   - Return `Value::String(Arc::new(result))`.
2. **`src/runtime.rs` dispatch** — beside `:wat::core::string::concat` (`:4085`):
   `":wat::core::string::interpolate" => crate::string_ops::eval_string_interpolate(args, list_span, env, sym).map_err(Into::into),`
3. **`src/check.rs` — a custom infer arm** modeled on `infer_string_concat` (`:4177` + `infer_string_concat`
   def): `infer_string_interpolate` — arg[0] must unify with `String`; the rest are (keyword, value) pairs
   (validate keyword in key slots; value slot is any `str`-renderable / EdnRepresentable type — at minimum
   don't reject); returns `string_ty()`. Wire the arm in the `dispatch_keyword_head` infer match beside
   the concat arm.
4. **`src/macros/eval.rs` — `is_pure_total`** — add `| ":wat::core::string::interpolate"` beside
   `:wat::core::string::concat` (`:414`). THIS is what makes it expand-time-legal (the whole point).
5. **(optional, intueri's sharpener)** a one-line doc-comment at the intrinsic + at the `format` macro
   noting the pair: `format` = expand-time macro, zero runtime cost; `string::interpolate` = pure-total
   intrinsic, legal in macro bodies.

## Proof

- `tests/probe_arc284_interpolate.rs` (un-ignore):
  - runtime: `(string::interpolate "{a}::{b} {{lit}}" :a "x" :b 5)` → `"x::5 {lit}"`.
  - **expand-time**: a defmacro body using `interpolate` to build a name expands cleanly (the load-bearing
    property — proves it's on the allow-list).
- A `wat-tests/` deftest mirroring the runtime + strict cases.
- Floors: lib 929/36, deftest, deporder 0 — all unchanged (additive intrinsic).

## Out of scope (rejected)

- Changing the `format` macro (stays `{name}`, top-level, zero-cost). NOT touched.
- `#{}` inline-expression interpolation — impossible (wat strings opaque post-lex); the sigil is reserved.
- The position-gated concat→format/interpolate fix + the re-sweep — the NEXT stones; this ships the
  intrinsic the fix will target.

## Four questions

- **Obvious?** YES — `(string::interpolate "{a}" :a v)` reads as the string it makes; lives in the string
  family beside concat/join.
- **Simple?** YES — one intrinsic; reuses `eval_str`'s render + the `format` template grammar; one check arm.
- **Honest?** YES — pure-total by construction (so genuinely expand-time-legal, not guessed); `{name}` not
  `#{}` (no false inline-eval promise); strict matching (no silent drops).
- **Good UX?** YES — this is the question we were failing: expand-time code finally has a clean interpolator.

## Blast radius

`src/string_ops.rs` (the intrinsic + a `render_unquoted` helper) + `src/runtime.rs` (dispatch) +
`src/check.rs` (infer arm) + `src/macros/eval.rs` (allow-list) + un-ignore the probe + a `wat-tests/`
deftest. No wat-stdlib change (additive intrinsic). The `format` macro is untouched.
