# Arc 171 — INSCRIPTION

**Status:** Shipped + closed 2026-05-10.

**What this arc closes:** the second of three threads of
lexical EDN-compliance opened by arc 170 REALIZATIONS pass
14. Thread 1 (slice 1f-W; `<>`-position comma↔underscore
wire encoding) shipped at `4278c4d`. Thread 3 (macro flavor
swap — Scheme `,`/`,@` → Clojure `~`/`~@`) is arc 172,
queued (DESIGN authored; slice 1 BRIEF authored; spawns
immediately after this arc closes).

## What shipped

**Apostrophe `'` is the universal separator inside
keyword/symbol bodies.** Commas no longer carry meaning in
keyword bodies except inside `(...)` tuples or `<...>`
parametrics.

### The user's locked convention table

```
:wat::core::op
:wat::core::op'2
:wat::core::op'i64'i64
:wat::core::op'i64'f64
:wat::core::op'f64'i64
:wat::core::op'f64'f64
```

Per user direction 2026-05-10. Apostrophe replaces both the
comma AND any dashes within the post-comma discriminator
suffix. Dashes elsewhere (verb identifiers like
`:wat::io::read-line`, `:wat::core::-`) stay as dashes.

### Three slices

| Slice | Scope | Result |
|---|---|---|
| 1 (`a40a40a`) | Lexer accepts `'` inside keyword body (purely additive; was already accepted via fall-through) — locked via 6 test cases + doc comment | Mode A, ~8 min sonnet, 9/9 rows |
| 2 (`9566d33`) | Consumer sweep: `,N`/`,xxx-yyy` → `'N`/`'xxx'yyy` across 45 files / ~440 sites (3× BRIEF estimate; sonnet caught broader sites) | Mode A, ~45 min sonnet, 9/9 rows |
| 3 (this commit) | Reject `,` inside keyword body at depth 0; preserve commas inside `(...)` / `<...>`; replace transition test with rejection test | Mode A, ~10 min sonnet, 10/10 rows |

## What got surfaced

### Scope grew 3× from BRIEF estimate

Slice 2's pre-grep showed ~440 sites across 45 files, not the
~167 estimate. Sonnet's broader regex caught
`:i64::+,2`-style patterns the BRIEF's regex missed. All
swept cleanly without scope expansion.

### Integrity-artifact regeneration handled in-band

`wat-tests/holon/eval-coincident.wat` contains Ed25519
signatures + SHA-256 digests of source strings. Sweeping the
source strings invalidated both; sonnet regenerated them in
lockstep with the source edits. Verified by orchestrator that
`EXPECTED_SRC_A_SIG` / `EXPECTED_SRC_B_SIG` constants in
`src/runtime.rs` match the new string literals in the wat file.

### Carve-outs compose without special casing

The slice 3 reject rule fires at `paren_depth == 0 &&
angle_depth == 0`. Commas inside `:(A,B,C)` keep working
because the depth tracker is already incremented when the
comma arrives. Same for `:HashMap<K,V>` (angle_depth tracker).
No special-case code needed; the existing depth invariants
carry the discipline.

### `keyword_comma_suffix_transition` retired

Slice 1 added this test as a transition-mode guard. Slice 3
replaces it with `keyword_comma_in_body_rejected` (asserts the
rejection works + the carve-outs hold). The transition mode is
complete; commas in keyword bodies are now an honest error.

## What does NOT change

- Commas inside `(...)` keyword tuples (e.g., `:(i64,String)`)
  — load-bearing tuple separator
- Commas inside `<...>` keyword parametrics (e.g.,
  `:HashMap<K,V>`) — type-arg separator per slice 1f-W
- Commas in other lexer contexts (string literals, etc.) —
  unaffected
- Commas at the main lex loop as Unquote / UnquoteSplicing
  tokens — RETIRED in arc 172 slice 1 (not this arc's
  domain)

## Why this matters

Per the user's principle (REALIZATIONS pass 14):

> *"type declarations may only be keywords. keywords may not
> contain underscores. underscores are reserved for swapping
> from commas when transmitting EDN ... further... symbols
> may not contain commas, however they can use underscores..."*

The substrate is now lexically EDN-compliant for keyword
bodies. Arc 172 closes the macro-syntax half. After both
arcs ship, **commas carry zero meaning outside the `(...)`
tuple and `<...>` parametric positions.** wat source
round-trips through EDN parsers without lossy reinterpretation.

## The four questions

**Obvious?** Yes. The user's convention table is unambiguous;
the apostrophe-everywhere rule reads in one sentence: "the
internal separator inside a keyword body is apostrophe."

**Simple?** Yes. The depth tracker already provided the
position-awareness; the slice 3 reject rule is one match arm.

**Honest?** Yes. The arc names every concern surfaced (scope
growth, integrity-artifact regeneration, carve-out
composition). No deferrals. No "future fix" language.

**Good UX?** Yes. Authors read `:op'i64'f64` and know exactly
what it means without needing to remember dual-character
conventions.

## Cross-references

- DESIGN.md (the broader arc context)
- SCORE-SLICE-1.md / SCORE-SLICE-2.md / SCORE-SLICE-3.md
  (per-slice calibration)
- Memory: `feedback_apostrophe_dispatch_separator.md` (user's
  locked convention table)
- Arc 170 REALIZATIONS pass 14 (the parent decision; framed
  arity-only; user broadened to all comma-in-keyword 2026-05-10)
- Arc 170 slice 1f-W (`4278c4d`) — sibling EDN-compliance
  work for `<>`-position commas (the FIRST thread)
- Arc 172 (DESIGN authored; slice 1 BRIEF authored) —
  Scheme → Clojure macros (the THIRD thread); spawns after
  this arc closes
- 058 FOUNDATION-CHANGELOG: row added 2026-05-10 marking
  arc 171 shipped

## Out of arc 171's scope; tracked in arc 172

Arc 172 (Scheme → Clojure macro flavor swap) is tracked at
`docs/arc/2026/05/172-scheme-to-clojure-macros/`. DESIGN
authored; slice 1 BRIEF authored. Arc 172 closes the THIRD
thread of REALIZATIONS pass 14's EDN-compliance work (commas
at the lex-loop level retire; `~`/`~@` become unquote tokens;
auto-gensym + `&form`/`&env` mint). Arc 172 is a separate arc;
arc 171's INSCRIPTION does NOT commit to its execution
timing — it is its own arc opening on its own terms.

## Closure-discipline pre-INSCRIPTION grep

Per recovery doc § 11 mandatory grep — run + verified clean
2026-05-10 before commit. The only matches were the grep
pattern itself (literal text quoted above the discipline
section) and this self-referential note; no deferral-prose
in arc 171's actual scope commitments. All scope items
shipped.
