# BRIEF — arc 109 Stone ②-i-b: the Tuple arm

Finish what ②-i (`0422b67ff`) scoped out. `type_expr_to_clojure_form`'s `TypeExpr::Tuple` arm is
the one arm that still splices its args FLAT and still hardcodes the `wat.type/Tuple` Symbol head
in BOTH modes. Give it exactly what the `Parametric` arm already has: **bracketed args in one
`WatAST::Vector`, and the head-spelling `mode` honoured.** Then stop the renderer's *input* from
being canonicalized, so a `nil` type keyword renders back as `nil` instead of as an empty tuple.

Design: `DESIGN-STONE-2i-b-the-tuple-arm.md` (sibling). Read it first — it carries the measurement
and the builder's ruling this brief implements.

## Read in order

1. **`src/edn_shim.rs:1322–1332`** — the `TypeExpr::Tuple` arm. This is the room. Note how the
   `Parametric` arm immediately above it (`1306–1312`) builds `head_node` through the 4-way ladder
   and then wraps args in ONE `WatAST::Vector`. That is the shape to mirror.
2. **`src/edn_shim.rs:1216–1221`** — the fn-doc bullet that currently documents Tuple as
   "OUT OF SCOPE for `mode`" and says the empty `:()` renders `(wat.type/Tuple)`. Both sentences
   become false with this change; the doc is part of the deliverable, not an afterthought.
3. **`src/types.rs:4334`** — `parse_type_expr_with_span`. The new sibling goes beside it.
4. **`src/types.rs:4728`** — the line that collapses `:wat::core::nil` into `Tuple(vec![])` when
   `canonicalize` is true. You are not editing this line; you are giving the renderer a path that
   does not reach it.
5. **`src/edn_shim.rs:1364`** — the single call site that must switch to the new entry point.
6. **`wat-scripts/scratch-pad/arc109-tuple-bracket-reader.wat`** — the committed probe proving the
   READER already takes the bracketed form, empty one included. You do not need to make the parser
   accept anything; it already does.

## The work

**(a) A non-canonicalizing entry point, and one call site onto it.**

In `src/types.rs`, beside `parse_type_expr_with_span`, add:

```rust
pub fn parse_type_expr_preserving_with_span(kw: &str, span: &Span) -> Result<TypeExpr, TypeError>
```

Byte-identical to `parse_type_expr_with_span` except it passes `canonicalize=false` to
`parse_type_inner`. It **still calls `reject_any`**. It returns `Result`, never `Option` — the
existing `canonicalize=false` path (`parse_type_expr_audit`, `src/types.rs:4561`) swallows parse
errors into `None`, which is why the verb cannot reuse it; the verb surfaces those errors to the
caller as a `MalformedForm`.

Then in `src/edn_shim.rs:1364`, `eval_keyword_to_type_form_impl` calls the new function instead of
`parse_type_expr`. That one line is the whole of (a). Both verbs — `keyword/to-type-form` and
`keyword/to-type-form-colon` — route through this shared impl, so both change together, which is
what we want.

**(b) The Tuple arm brackets and honours `mode`.**

The head becomes mode-dependent, mirroring the `Parametric` arm's case 1:

- `TypeFormHeadMode::Clojure` → `WatAST::Symbol("wat.type/Tuple")` (unchanged from today)
- `TypeFormHeadMode::Colon` → `WatAST::Keyword(":wat::core::Tuple")`

and the items go into ONE `WatAST::Vector` in the list's second position, unconditionally in both
modes, **at every arity including zero and one**. The full ladder, as the builder set it down — a
bare head is ILLEGAL at the top of it, and that is the whole point:

```
(:wat::core::Tuple)                                        ILLEGAL — never emit a bare head
(:wat::core::Tuple [])                                     empty
(:wat::core::Tuple [:wat::core::i64])                      1-ary
(:wat::core::Tuple [:wat::core::i64 :wat::core::String])   2-ary

(wat.type/Tuple [])
(wat.type/Tuple [wat.type/i64])
(wat.type/Tuple [wat.type/i64 wat.type/String])
```

The empty rung is a **first-class member of that ladder, not a defensive case**: `(wat.type/Tuple [])`
is legal, writable source today and the committed reader probe exercises it. (Only the KEYWORD
spelling `:()` is retired; that retirement is about a spelling, not about the type.)

The 1-ary rung is real and distinct — measured, passing a bare `7` to a
`(wat.type/Tuple [wat.type/i64])` param is a TypeMismatch. Do not special-case it, and do not
special-case zero either: one code path, `args.len()` never consulted.

The builder's ruling, verbatim, 2026-08-20:

> *"nil is rust's unit… but `nil != ()` in wat. nil is not an empty list. `(wat.type/Tuple)` is
> illegal, it'd be `(wat.type/Tuple [])` to be an empty tuple."*

Items still recurse with the same `mode`, as they do today.

**(c) The four goldens that pin the old shape.**

Update each to the new expected bytes. These are single-line files:

| file | becomes |
|---|---|
| `tests/resolve/probe_arc251_keyword_to_type_form__contract-06-tuple.wat` | `(wat.type/Tuple [wat.type/i64 wat.type/String])` |
| `tests/resolve/probe_arc251_keyword_to_type_form__contract-07-empty-tuple.wat` | `(wat.type/Tuple [])` |
| `tests/resolve/probe_arc251_keyword_to_type_form__contract-08-nested-tuple.wat` | `(wat.type/Tuple [(wat.type/Vector [T]) wat.type/i64])` |
| `tests/reflection/wat_arc201_structured_signature_types__tuple.edn` | its `(wat.type/Tuple wat.type/i64 wat.type/String)` line becomes `(wat.type/Tuple [wat.type/i64 wat.type/String])` — keep the surrounding EDN and its indentation exactly as-is |

The fourth one is the reflection path (`runtime.rs:13034` / `runtime.rs:14649`), which shares this
renderer in Clojure mode. Its sibling `__parametric_fn.edn` already shows the bracketed
`(wat.type/Vector [wat.type/i64])` from ②-i — that is your confirmation the two goldens live on the
same code path and this one simply had not been reached yet.

**(d) One test name that becomes true.**

`tests/resolve/probe_arc251_keyword_to_type_form.rs:73` is
`contract_07_empty_tuple_is_not_nil` (its `:user::c07` fixture fn is at
`tests/resolve/probe_arc251_keyword_to_type_form.wat:15`). Today that name is a claim its own fixture cannot
distinguish — `:()` and `:wat::core::nil` both render `(wat.type/Tuple)`. After (a) they diverge
(`(wat.type/Tuple [])` vs `wat.type/nil`). Add a second assertion to that test that feeds
`":wat::core::nil"` through the same verb and asserts it does **not** render as any Tuple form —
add a `:user::c07b` fn to `tests/resolve/probe_arc251_keyword_to_type_form.wat` mirroring the
existing `:user::c07` but feeding `":wat::core::nil"`, and a golden sibling named exactly
`probe_arc251_keyword_to_type_form__contract-07b-nil-is-not-a-tuple.wat`. That is what makes the test's name earned
rather than asserted.

## Blast radius

`src/types.rs` (one new fn) · `src/edn_shim.rs` (one arm, one call site, the fn doc) ·
four golden files · one test file + its `.wat` fixture. **No new types. No change to
`parse_type_inner` or to `src/types.rs:4728`. No change to the `Parametric`, `Path`, or `Fn` arms —
the `Fn` arm in particular is correct and is not yours to touch.**

## STOP triggers

1. **STOP-1** — if bracketing the Tuple arm requires ANY change to `parse_type_node`
   (`src/types.rs:4460`+) or to the bracket-unwrap at `src/types.rs:4528` to make the emitted form
   read back, STOP and report. The probe says it does not; if the disk disagrees with the probe,
   that is a finding, not something to work around.
2. **STOP-2** — if switching the call site to the non-canonicalizing parse changes the rendering of
   anything OTHER than a `nil`-derived type (a scalar, a parametric, a type-var, a user type), STOP
   and report which input and both spellings. Only `nil` is supposed to move.
3. **STOP-3** — if you find a golden, fixture, or inline assertion pinning the old flat Tuple shape
   that is NOT one of the four named in (c), STOP and report it with its path. The list is the
   orchestrator's measurement; a fifth means the measurement was incomplete and the scope needs
   re-drawing before you edit it.

## How this lands

You are a rider. **Text edits only.** The orchestrator builds, runs the release floor, and runs
clippy — centrally, once, after the tree is quiescent. Do not run cargo, do not build, do not
commit, do not stash, do not revert. Run everything you do run in the FOREGROUND: your turn ends
when your edits are on disk and your report is written, and ending your turn ends you.

Report: the exact diff shape per file, anything that surprised you, and any STOP you hit.
