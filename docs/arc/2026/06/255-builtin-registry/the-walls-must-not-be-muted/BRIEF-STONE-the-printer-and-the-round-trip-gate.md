# BRIEF — the `#wat.doc/Row` printer, and the round trip that proves the migration

Design: `[[DESIGN-STONE-the-printer-and-the-round-trip-gate]]` (same dir) — read it first; it
carries the two decisions and, more importantly, the blind spot the gate must not inherit.
Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`; use `git -C` for any git read.

Two parts. Part 2 is the deliverable; Part 1 exists to make Part 2 possible.

---

## PART 1 — the printer, in `wat-doc`, beside the two readers

```
crates/wat-doc/src/lib.rs:225    pub struct DocComment    — the one shape
crates/wat-doc/src/lib.rs:506    parse(raw: &str)         — the @ reader
crates/wat-doc/src/lib.rs:1025   from_metadata(&WatAST)   — the map reader ("the ONE decoder")
```

Add the inverse: `print(doc: &DocComment) -> String`, emitting a `#wat.doc/Row { … }` block.

**The worked example is already on disk.** `src/intrinsic/char.rs` carries the first hand-written
row in the new form, and it parses today. Your printer's output for `:wat::core::char` should be
recognisably that shape — read it before writing anything.

**The docstring is the whole difficulty.** It must be a LITERAL multi-line string whose
continuation lines carry the map's margin — Clojure's docstring shape, and the **exact inverse** of
`crates/wat-macros/src/edn_doc.rs:72` `dedent` ("the fence's own least-indented line sets it,
exactly like Python's `textwrap.dedent`"). Read that function; you are writing its mirror.

⛔ **Do NOT touch `wat-edn`'s `write-pretty`.** It feeds diagnostics, IPC, and every golden `.edn`
under `tests/`; emitting literal newlines there would churn goldens tree-wide. This is a separate,
named emitter. (Measured: `write-pretty` escapes newlines today —
`wat-scripts/scratch-pad/255-how-does-write-pretty-handle-a-docstring.wat`.)

**Values are EDN ns/name keywords, not wat FQDNs** — `:wat.core/foldl`, not `:wat::core::foldl`.
`::` is a lexer error in EDN. This is not a convention choice: it is the wire format's own canonical
rendering, and `edn::write` already performs it losslessly
(`wat-scripts/scratch-pad/255-does-edn-round-trip-a-wat-keyword.wat`).

## PART 2 — the round-trip gate, and it must not inherit `char`'s blind spot

```
from_metadata(edn_to_watast(wat_edn::parse(print(doc))))  ==  doc
```

A test that holds this for real rows. `crates/wat-macros/src/edn_doc.rs` already has the
`wat_edn::Value → WatAST` transcoder; use it rather than writing a second one.

⛔ **`char` exercised `@added @arg @ret @example` and the five axes — and NOTHING else.** A gate
built only from what `char` has would pass while silently losing seven fields. It must cover:

```
@see           258 uses   src/collection/transform.rs has it
@yields         11 uses   src/intrinsic/witness.rs
@example-norun 139 uses   src/intrinsic/kernel/resource.rs
@syntax         37 uses
@deprecated      0 uses   ⛔ NO LIVE ROW EXISTS — see below
```

★ `src/intrinsic/holon/hologram.rs` carries **`yields` and `example-norun` together** — the richest
live witness I found. Use it, plus at least one row with `@see` and one with `@syntax`.

⚠ **`@deprecated` has no live user anywhere in the tree.** Either reach it with a constructed
`DocComment`, or state plainly in your report that the field is UNCOVERED. Do not let it pass by
silence — an uncovered field in a round-trip gate is exactly the shape that ships a quiet loss.

**Prove the gate is not vacuous.** Break the printer deliberately — drop a field, mangle the
docstring margin — and watch the gate go RED naming it. Report that red's text verbatim, then
restore. A gate never seen failing is a claim, not a gate.

---

## STOP TRIGGERS — rejections. Ship nothing, report, let me re-plan.

**STOP-1 — no second EDN writer.** If the emitter seems to need changes inside `wat-edn`'s writer,
STOP. A second authority on how EDN is written is the defect this campaign exists to remove, in
mirror image.

**STOP-2 — a field that cannot round-trip is THE finding.** If any `DocComment` field survives
`print` but not the read back, STOP and name the field. Do not special-case it, do not drop it from
the gate. That is the most valuable thing this stone can produce.

**STOP-3 — no existing row may change.** Every `@`-form row and `char`'s `#wat.doc/Row` keep working
untouched. The registry census must still read **571 rows · 85 SpecialForm · 52 alias**.

**STOP-4 — a red is a red.** Do NOT re-run. Copy the failing test's whole stdout+stderr block
verbatim, name the exact assertion that fired, report. Never weaken an assertion to make it pass.

## What you run, and what you do not

Yours, in the FOREGROUND: `cargo build --release`, `cargo test --release -p wat-doc`,
`cargo test --release -p wat-macros`, `target/release/wat --check <file>`, scoped
`cargo nextest run --release -E '<expr>'`, and the probes at `wat-scripts/scratch-pad/255-*.wat`.
**Do not run the full floor** — the orchestrator runs it centrally once the tree is quiescent, and
it now includes a doctest stage; do not disable it. Do not commit, push, stash, or revert.

## Report

The printer's output for `:wat::core::char`, verbatim, beside the hand-written row it should match ·
which rows the gate covers and which fields each one exercises · whether `@deprecated` is covered or
uncovered, said plainly · the sabotage red's text verbatim · any field that does not round-trip ·
anything that surprised you.
