# BRIEF — the docstring becomes a real heredoc

Design: `[[AMEND-the-docstring-needs-a-string-local-margin]]` (same dir). It carries the mechanism,
the four questions, and the rule. Read it first. The previous stone is committed and green at
`0582f1919`; this amends it.

**Builder, on reading the printer's output:** *"that doc string doesn't have the indent it should….
the second lines are not aligned to the start of the string."*

## THE WORK

`print` currently emits a multi-line `:doc` with **flush-left** continuation lines. That is forced
by `crates/wat-macros/src/edn_doc.rs:71` `dedent`, which strips ONE fence-wide common margin — align
the continuations and those columns land inside the prose as data. Your SCORE defended this
correctly; it is a correct defence of the wrong reader.

Replace the fence-wide rule with a **string-local** one:

```
PRINTER   a multi-line string's continuation lines indent to the column where the string's
          CONTENT begins — one past the opening `"`.

READER    after the existing fence-wide dedent, strip from each continuation line exactly the
          number of columns at which that string's content begins on its OWN opening line.
```

Both sides compute the same number from the same place, so it is exactly invertible.

**Rooms:** `crates/wat-macros/src/edn_doc.rs:44` `extract_edn_fence` and `:71` `dedent` (the reader
half — `dedent` becomes string-aware, tracking quote boundaries and `\"` escapes so it knows which
lines are continuations) · `crates/wat-doc/src/print.rs` (the printer half) ·
`src/intrinsic/char.rs` (the worked example — its fence's `:doc` continuations must be re-indented
to the new rule; that is the example moving with the form, not a second migration).

## ⛔ THE TRAP INSIDE THE TRAP — this is the acceptance, not a footnote

Prose whose own lines carry **meaningful** leading whitespace — an indented code sample inside a
docstring — must survive untouched. The reader strips a **FIXED count** (that string's opening
column), never "all leading whitespace", and never a per-line minimum. **A per-line-minimum rule
would silently eat a code sample's own indentation** — the same class of quiet loss this whole
effort exists to stop.

Prove it. A round-trip witness whose prose contains: a BLANK line, a line with its own LEADING
whitespace beyond the margin, and an embedded `\"` escape. If any of those cannot survive, STOP and
name it — that is the finding, not a case to special-case.

## THE GOLDEN THAT PINS THE OLD BEHAVIOUR

`crates/wat-doc/src/print_tests__flush_left_docstring.edn` pins flush-left. **Replace its content
with the aligned shape and rename it** to say what it now asserts. Do not delete it — it is the
witness that the margin rule holds, and it changes meaning rather than ceasing to matter.

## STOP TRIGGERS

**STOP-1 — `wat-edn` stays untouched.** B2 held last time; it holds now. If the rule seems to need
a change inside `wat-edn`'s lexer or writer, STOP.
**STOP-2 — meaningful indentation must survive.** See above. Not negotiable and not runnable-around.
**STOP-3 — the round trip stays exact.** Every gate row from the last stone keeps passing, byte for
byte. If alignment costs byte-identity anywhere, the rule is wrong and I want to hear it.
**STOP-4 — a red is a red.** Do not re-run. Capture the whole block, name the arm, report.

## ⚠ RUN THE LINT BINARY THIS TIME

`cargo test -p wat-doc` and `-p wat-macros` do **not** build `tests/lint/` — that is why 20 loose
assertions reached my floor last round. Before you yield, run:

```
cargo nextest run --release --test lint
```

Everything else stands: FOREGROUND, block on it, no full floor (mine, centrally), no commit/push/
stash/revert.

## Report

The printer's `:wat::core::char` output, verbatim, showing the aligned continuations · the
whitespace-witness row and what its prose contained · the `--test lint` result · anything that
surprised you.
