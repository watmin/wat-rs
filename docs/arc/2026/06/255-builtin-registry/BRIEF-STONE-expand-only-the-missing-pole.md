# BRIEF — STONE 1 of 2: `ExpandOnly` — mint the pole, derive the branch

Mint the missing `ExpandTime` pole, re-declare `macro-error` onto it, and let the doc gate's third
branch **derive** from the coordinate. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-expand-only-the-missing-pole.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
You may run the pre-existing `./target/release/wat` and `--check` freely; that binary does NOT
contain your changes. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything.

⚠ **The working tree is already dirty and the floor is currently RED on one test** —
`purity_mandated_examples`, which is what this stone fixes. That is expected, not your doing. Two
other reds from the same wave were already repaired by the orchestrator. **Do not "fix" anything you
did not come here to change**; if you see unrelated modified files, leave them alone.

## Read in order

1. The DESIGN above — why this is a missing coordinate and not an exemption.
2. `wat/runtime-meta.wat:248` — the `ExpandTime` `defenum`, and read `:RuntimeOnly`'s doc sentence
   carefully. **Your new variant is its mirror**, and the doc you write must say so — not a synonym
   for `Legal`.
3. `crates/wat-doc/src/lib.rs:2239` — `wat_enum_from!(pub enum ExpandTime, "../../wat/runtime-meta.wat", …)`.
   **wat is the source of truth**: the Rust enum and its name-parsing are GENERATED from the
   `defenum` at build time. You add the variant in ONE place — the `.wat` file.
4. `src/intrinsic/macro_error.rs:70` — the `@ExpandTime Legal` line to change, and `:28`'s
   "Macro-body-only — legal ONLY where…" which is the evidence it was wrong.
5. `src/intrinsic/mod.rs:1305` — `purity_mandated_examples`, the gate to give a third branch.
6. `src/macros/eval.rs:424-426` — `is_expand_time_legal`. **Read STOP-1 before you touch this.**

## The work

### 1 — mint the pole, in wat

Add `:ExpandOnly` to `wat/runtime-meta.wat`'s `ExpandTime` `defenum` with a doc comment in the shape
of its siblings. It must state the mirror explicitly: **`RuntimeOnly`'s opposite** — the verb has no
runtime call site at all; its only legitimate caller is a `defmacro` body during expansion. Name it
for what the verb IS, the way `RuntimeOnly`'s own comment says it was named.

### 2 — let the compiler find the rest

Adding the variant will break every **exhaustive** `match` over `ExpandTime`. That is the census, and
it is free — fix each site the compiler names. Known ones (verify, don't trust this list):
`crates/wat-macros/src/wat_intrinsic.rs:682-685`, `crates/wat-macros/src/wat_special_form.rs:86-89`
(a SECOND copy of the same match — both are real), and an exhaustive round-trip test in
`crates/wat-doc/src/lib.rs`'s `probe_expand_time_axis`.

### 3 — re-declare `macro-error`

`@ExpandTime Legal` → `@ExpandTime ExpandOnly`, with grounding prose citing `macro_error.rs:28` and
`src/value/signal.rs:529` ("evaluated at expand time, NEVER post-expansion").

### 4 — the doc gate's third branch

`purity_mandated_examples` currently reads:

```rust
if is_pure_and_det { assert!(has_run) }
else { assert!(has_norun); assert!(!has_run) }
```

Give it a branch that **derives from the coordinate**: a verb whose `expand_time` is `ExpandOnly` has
no runtime call site, so a runnable `@example` is impossible and `@example-norun` is its correct and
REQUIRED form. ⚠ **Make it checkable in BOTH directions**, exactly as the existing two branches are:
`ExpandOnly` + a runnable example must FAIL. A branch that only relaxes is a hole.

## Blast radius

`wat/runtime-meta.wat` (one variant) · `crates/wat-macros/src/{wat_intrinsic,wat_special_form}.rs`
(one match arm each) · `crates/wat-doc/src/lib.rs` (whatever the compiler names) ·
`src/intrinsic/macro_error.rs` (one declaration + prose) · `src/intrinsic/mod.rs` (the third branch) ·
`src/macros/eval.rs` (STOP-1). No `.wat` corpus change. No verb bodies move.

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — THE ONE THE COMPILER CANNOT CATCH, and it breaks `macro-error` if you miss it.**
`src/macros/eval.rs:426` is `matches!(e.expand_time, ExpandTime::Legal | ExpandTime::Preserving)`.
A `matches!` does **not** go non-exhaustive when a variant is added — it silently returns `false`.
So `ExpandOnly` would be **refused inside macro bodies**, which is `macro-error`'s ONLY legitimate
call site. **`is_expand_time_legal` must ACCEPT `ExpandOnly`** — an expand-time-only verb is by
definition expand-time legal. This is the single measured `matches!` site on this axis; if you find a
second, STOP and report it.

**STOP-2 — the branch must bite in both directions.** If you cannot make `ExpandOnly` + a runnable
`@example` fail the gate, STOP and report why. A one-way relaxation is the hole this stone exists to
close, not a smaller version of the fix.

**STOP-3 — no runtime behaviour changes in this stone.** A top-level `(:wat::core::macro-error "x")`
must still pass `--check` and still raise at run, exactly as today. **Refusing it is STONE 2's job.**
If you find yourself adding a check-time refusal, stop — that is the next stone and bundling it makes
a red un-attributable.

**STOP-4 — one verb, not a census.** `macro-error` is the only verb on disk claiming macro-body-only
(measured). If you find a second candidate, that is a **finding to report**, not a declaration to
change.

**STOP-5 — wat is the source of truth.** If you find yourself hand-writing the variant into a Rust
`enum ExpandTime` declaration, stop and re-read `wat_enum_from!` at `crates/wat-doc/src/lib.rs:2239`.
The Rust side is generated from the `.wat`; adding it twice is the drift this mechanism exists to
prevent.

## Report

Per-file diff summary; the variant's name and the doc sentence you wrote for it; **every site the
compiler forced you to touch** (this is the census the orchestrator cannot reconstruct); how you made
the third branch bite in both directions and how you proved it; confirmation that
`is_expand_time_legal` accepts the new variant; and what surprised you — a site the brief did not
name, a second `matches!`, or a generated file that did not regenerate.
