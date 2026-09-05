# AMEND — the docstring's continuation lines must ALIGN. That needs a string-local margin, not a fence-wide one.

> **Builder, 2026-09-05**, reading the printer's output: *"that doc string doesn't have the indent
> it should…. the second lines are not aligned to the start of the string."*

He is right, and the SCORE's defence of flush-left is a correct defence of the **wrong reader**.

## THE MECHANISM — why flush-left is currently FORCED

`crates/wat-macros/src/edn_doc.rs:71` `dedent` computes **one common margin** across every
non-blank line of the fence and strips it uniformly — `textwrap.dedent`, exactly as its own doc
says. Inside a fence:

```
margin+0    #wat.doc/Row {
margin+2      :doc "first line
margin+8            continuation ALIGNED under the string content   ← what the builder wants
margin+2      :added "1.0.0"
margin+0    }
```

The common margin is `margin+0` (set by the braces). Stripping it leaves the continuation carrying
**6 literal spaces inside the prose** — and `wat_edn`'s lexer admits a raw newline in `"…"` byte for
byte, so those spaces become data. Byte-identity dies.

So flush-left is not a style grok chose. **It is the only shape a fence-wide dedent can round-trip.**
The defect is the dedenter, not the printer.

★ This is EXPECTATIONS' trap door 1, firing exactly where it was aimed: *"`dedent` strips the
least-indented line's margin. If a docstring's first line has different indentation from its
continuations, print→dedent may not be the identity. This is where the stone most likely breaks,
and it IS the finding."* It is the finding. The stone found it by being honest about the output.

## THE OPTIONS — four questions

| | Obvious? | Simple? | Honest? | Good UX? |
|---|:---:|:---:|:---:|:---:|
| **C1 · flush-left continuations** (today) | **NO** | YES | YES | **NO** |
| **C2 · a string-local margin — a real heredoc** | YES | YES | YES | YES |
| **C3 · `:doc` as a vector of lines** | **NO** | YES | **NO** | **NO** |

**C1 fails Obvious by demonstration** — the builder read the output and named it wrong on sight.
That is what Obvious is *for*; there is no stronger evidence available. It fails Good UX for the
reason the whole migration exists: a declaration nobody wants to read is the crutch coming back.

**C3 fails Honest**: `:doc` is a String, and `from_metadata` reads it as one. A vector spelling is a
second spelling for one field — the defect this campaign removes — and it makes prose worse to
author, not better.

## C2 — THE RULE, stated so it can be checked

```
PRINTER   a multi-line string's continuation lines are indented to the column where the
          string's CONTENT begins — one past the opening `"`.

READER    after the existing fence-wide dedent, for each multi-line string, strip from every
          continuation line exactly the number of columns at which that string's content
          begins on its OWN opening line.
```

Both sides compute the same number from the same place, so it is **exactly invertible** — and that
is testable directly, which is the acceptance:

```
dedent_stringwise(print(doc))  parses to  doc          for a doc whose prose has BLANK lines,
                                                        LEADING whitespace of its own, and an
                                                        embedded `"` escape
```

⚠ **What this costs, stated plainly:** the dedenter stops being pure `textwrap` and becomes
string-aware — it must track quote boundaries and `\"` escapes to know which lines are
continuations. That is roughly twenty lines and it is the *"smart heredoc"* the builder named. It
stays entirely inside `edn_doc.rs`. **`wat-edn` is still not touched** — B2 holds.

⛔ **The trap inside the trap:** prose whose own lines carry MEANINGFUL leading whitespace (an
indented code sample inside a docstring) must survive. The reader strips a FIXED count — the
string's opening column — never "all leading whitespace", and never a per-line minimum. A
per-line-minimum rule would silently eat a code sample's own indentation. The acceptance row above
names that case for exactly this reason.

## SCOPE

This amends the printer and `edn_doc::dedent` only. It does not reopen `@alias`, `@syntax`,
`:wat::doc::Row` as a runtime type, or the sweep. `src/intrinsic/char.rs`'s fence will need its
continuation lines re-indented to match the new rule — that is the worked example moving with the
form, not a second migration.
