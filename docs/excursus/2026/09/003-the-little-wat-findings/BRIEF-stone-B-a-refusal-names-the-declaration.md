# BRIEF — STONE B: a type refusal names the user's declaration

**Against `reason/little-wat-findings`.** Read `DESIGN-stone-B-a-refusal-names-the-declaration.md`
beside this first — it holds the root, the one contract decision, and the honest boundary.

## The work in one paragraph

Retain the declaration span the registry is already handed, and use it (plus the function body's
span) so that `UnknownNamedType` and `ImpureFieldInPureAggregate` name the **user's file and
declaration** instead of `src/check.rs`. Two banked tests flip from "still names wat-rs" to the
control's shape. No new public API, no span on `TypeExpr`.

## Read in order, and why you are sent there

1. **`src/types.rs:1095-1140`** — `register`, `register_with_span`, `register_stdlib_with_span`,
   `register_validated`. ⭐ **The span already arrives here**; arc 138 threaded it for
   registration-time errors and then dropped it. `register_validated` also already computes
   `let name = def.name().to_string()` — your key.
2. **`src/types.rs:818-840`** — `TypeEnv`'s fields. Add the side-table here.
   ⛔ **Do NOT add a span field to `AggregateDef`/`EnumDef`** — that is 14 construction sites for
   a value the registry can hold once.
3. **`src/check.rs:15220-15240`** — `validate_aggregate_containment`, F-114's site. It holds
   `name`; swap `rust_caller_span!()` for the lookup.
4. **`src/check.rs:15274-15300`** — `validate_named_type_annotations` and its `refuse` closure.
   ⚠ `refuse` takes only a `path: String` today; it needs the span too.
5. **`src/check.rs:15369-15384`** — the `functions_iter()` arm. **This is the one F-006 actually
   hits** (its program declares a `defn`, not a type). Use `func.body`'s span —
   `FunctionBody::Wat(Arc<WatAST>)` carries one; `FunctionBody::Native` does not.
6. **`tests/diagnostics/probe_ex003_diagnostic_locates_the_user.rs`** — the banked probe. Its
   `f006_*` test currently asserts the defect and **must flip to the control's shape**; its
   `control_*` test must stay green untouched, and is your proof you did not break user spans.

## Implementation sketch

```rust
// src/types.rs — TypeEnv
decl_spans: HashMap<String, Span>,

// in register_validated, where `name` is already in hand:
self.decl_spans.insert(name.clone(), span.clone());

// a reader, used by both walks:
pub(crate) fn decl_span(&self, name: &str) -> Option<&Span> { self.decl_spans.get(name) }
```

```rust
// src/check.rs — the closure grows a span
let refuse = |path: String, span: Span| Err(TypeError::new(span, TypeErrorKind::UnknownNamedType { path }));
```

## Blast radius

`src/types.rs` (one field, one insert, one accessor) · `src/check.rs` (two walks) ·
`tests/diagnostics/probe_ex003_diagnostic_locates_the_user.rs` (flip one assertion).
**No `wat/`. No new public API. No change to `AggregateDef`/`EnumDef`/`TypeDef`.**

## STOP triggers

1. **A lookup returns `None` for a type a user declared.** That means the span is not arriving
   where the DESIGN says it does. STOP and report which registration path bypassed
   `register_validated` — do not paper over it with a sentinel fallback and move on.
2. **`TypeExpr` starts looking like it needs a span.** It is affirmatively out of scope; the
   boundary is in the DESIGN. Report the argument, change nothing.
3. **The `control_*` test reddens.** That is a cure breaking user spans — the exact thing the
   control exists to catch. STOP.
4. **Any gate in `tests/lint/` reddens that you did not add.** Capture the whole untruncated
   block, name the test and arm, report. ⛔ Do not re-run first.

## ⛔ Staging, and why this paragraph exists

`tests/lint/every_tracked_wat_parses.rs` enumerates with **`git ls-files`**. An UNSTAGED file is
not in its corpus, so running it before `git add` proves nothing — it returns a green
indistinguishable from proof. This excursus has already landed one red exactly that way.
**`git add` first, then run the gates, then commit.**

Gates a changed/added file under `tests/` answers to: `no_loose_string_assert` ·
`every_discovering_gate_declares_how_it_knows_it_reached_something` · `every_tracked_wat_parses` ·
`no_inlined_wat_in_tests` · `no_inlined_edn` · `every_wat_bad_fixture_actually_fails`.

## Prove it

- ⭐ **Mutation:** point the lookup at the sentinel again, watch **both** flipped tests go RED,
  restore. A gate that has never failed is not a gate.
- The `control_*` test passes throughout, untouched.
- Report the verbatim `:location` of both refusals after the cure.
