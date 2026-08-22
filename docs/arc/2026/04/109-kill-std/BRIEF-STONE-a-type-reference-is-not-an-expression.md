# BRIEF — the expander declines `(Head :- [args])`

DESIGN: `DESIGN-STONE-a-type-reference-is-not-an-expression.md`. Read it first — it carries the
expansion that reframes this and the four options.

## The work, in one paragraph

`(:user::R :- [T])` is a TYPE REFERENCE, but the expander sees a list headed by a registered macro
(`defrecord` mints a kwargs companion under the bare name) and expands it into
`(:wat::core::kwargs-construct :user::R :- [T])`, which then lands in a type slot and fails. A form
whose element 1 is the `:-` keyword can never be a value expression, so the expander must decline it.

## The room — one guard, one site

```rust
// src/macros/expand.rs:520
if let Some(WatAST::Keyword(head, head_span)) = items.first() {
    if registry.contains(head) {
```

The guard belongs here: also require that `items.get(1)` is **not** the `:-` keyword.

⚠ `:-` lexes as a **KEYWORD**, not a Symbol — measured, and `src/types.rs`'s `is_binder_marker` is the
canonical test (`matches!(node, WatAST::Keyword(k, _) if k == ":-")`). A guard that matches a Symbol
will silently never fire, and every acceptance row will fail with the ORIGINAL error, which reads like
"not implemented yet" rather than "matched the wrong node kind." That mistake has already been made
once in this codebase, in `wat-source-derive`, where it sat for months.

★ The surrounding comment at that site already documents why this arose — *"the construction flip
turned every aggregate type-keyword into a MACRO"* (arc 294 item 9a). Extend that comment; do not
replace it.

## Why the shape test, and not a slot test

`(Head :- [args])` carries the marker at **index 1**. A DECLARATION — `(defn :name :- [T] …)` —
carries it at **index 2**. The two are distinguishable without knowing any head's grammar, which is
why this needs no per-head list. This arc has watched such a list be wrong twice.

## The census, already run — with its limits

```
(head :- …) shape, corpus-wide:  wat/ 0 · tests/ 0 · wat-scripts/ 9
distinct heads:  Peer(3) · Option(3) · Tuple(2) · Lru(1)   — every one a TYPE reference
```

**No macro call uses this shape.** What that census CANNOT see: a form split across lines, and
MACRO-GENERATED code (which appears in no file). The population is small only because ②-iii has not
run — after it there will be hundreds, all type references.

## What "done" looks like

1. `(:user::R :- [T])` as an annotation CHECKS, where `R` is `defrecord`-minted.
2. The same for `defstruct` and `holon::defrecord`.
3. ★ **`(:user::R :field v)` — the kwargs constructor — still works.**
4. `(:wat::cache::Entry :- [K V])` and `(:wat::spawn::Launched :- […])` check.
5. The already-passing spellings are undisturbed: builtin `(:wat::core::Vector :- [:i64])`,
   typealias `(:wat::cache::Lru :- […])`, defenum `(:wat::spawn::ServiceEvent :- […])`.
6. A DECLARATION still expands normally — `(:wat::core::defn :user::f :- [T] [x <- :T] -> :T x)`
   checks, i.e. the guard did not catch index-2 markers.

⚠ **Row 3 is the row that bites, and rows 1-2 cannot substitute for it.** Rows 1-2 measure that the
form is no longer expanded. **Only row 3 measures that it is still expanded where it should be.** A
change that simply stopped the companion macro from firing would pass 1, 2, 4 and 5 — and break the
construction ergonomics arc 294 item 9a exists to provide.

## Boundaries

- Do NOT run `scripts/floor.sh` or a full `cargo nextest` — the orchestrator measures centrally.
- Do NOT commit, push, stash, revert or amend. Leave everything in the working tree.
- Do NOT touch `wat/Record.wat` or any companion-macro emission. The shadowing is DELIBERATE
  (arc 294 item 9a moved the positional ctor to the prime `:ns::T'` so the bare name could be the
  kwargs macro). Reversing it is not on the table.
- Do NOT add a per-head allow-list or a slot table.
- `src/macros/expand.rs` should be the only file you need. If it is not, see STOP-3.

## Your own checks

`cargo build --bin wat`, then `target/debug/wat --check <file>` on files under
`wat-scripts/scratch-pad/`, plus `cargo nextest run --release -E 'binary_id(wat::macros)'` for a scoped
run. Prefix long commands with `systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0
timeout 900`. Diagnostics go to **stderr** — judge by exit code AND empty output, never grep alone.

Delete any scratch `.wat` that must fail; `tests/lint/wat_scripts_fixes_load.rs` type-checks
everything under `wat-scripts/`.

## STOP triggers — ship nothing further and report

- **STOP-1.** If row 3 fails — the kwargs constructor stops working — STOP and report. The guard is
  catching a real macro call, and its shape test is wrong.
- **STOP-2.** If the floor turns up a macro that stopped expanding, STOP and report WHICH, verbatim.
  The census found no such call, but it could not see generated code, and "a census scopes work in;
  it never scopes work out."
- **STOP-3.** If the guard needs to live anywhere other than that one dispatch site, STOP and report
  where and why. More than one site means the shape test is not sufficient and the design changes.

## Your report

The diff. Every acceptance row with verbatim output — row 3 especially. Whether any existing macro
stopped expanding. What surprised you. Anything you inspected and left alone, with the reason.
