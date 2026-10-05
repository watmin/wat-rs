# WEIGH — STONE 255.94: measure — a keyword that is not a name — ACCEPTED

**Executor: grok via pulsare, solo.** `0d13246b3` (example `examples/keyword_not_a_name.rs`, `keyword-not-a-name.tsv`,
SCORE). Weighed by the orchestrator on 2026-10-04. Nothing under `src/`, `crates/`, `wat/`, `tests/` changed (the SCORE's
`git diff` row); no floor needed.

## What it settles

- **Source `::` keywords are names:** 115,132 of 116,052 corpus hits and all 1,359 stdlib hits sit in name positions
  (call head, declaration, type, pattern, reference, quote, rete).
- **The `<` rows are an instrument defect, not data:** `:wat::core::<`, `:wat::i64::<=` and their twins are functions.
  `Name::enter`'s rendered-type test (`is_rendered_type_text`, `crates/wat-reader/src/identifier.rs:219`) refuses `<`
  because generics were once rendered `<T>`; that rendering is retired, so `<` in a name is a name character. Only `(`
  marks a rendered form. `:fn(…)` is the retired keyword-bodied fn type (the two negative proofs on KEEP).
- **Names are also used as values** (the 8 MIXED spellings: a declared fn stored in a map, `:wat::core::Option.None` as
  a map value, the metadata variants). At run time on `wat-tests`, value-position `::` keywords resolved to a function
  238,799 times, to `Option` 118,781, to a unit variant 1,685, and **stayed a keyword value 88,952 times** (spellings
  not recorded).
- **Names are built at run time as keyword values:** `keyword/from-string` builds 773 name-shaped service and variant
  paths per stdlib startup; reflection 292 (`variant-parent-of`, `compose-variant`, `metadata-of`, type records).
  A reader never sees these.
- **A `::` keyword can be an opaque token:** `:wat::kernel::__peer_crashed__` (`src/kernel/peer.rs:256`) is a sentinel
  built in Rust; a planted unresolved keyword was compared and hashed.

## Verdict

Accepted. The measurement says the reader alone cannot be the whole door: names also arrive at run time as keyword
values, and some `::` keywords are data. That needs the builder's ruling on what a keyword **is** (brought to the
builder 2026-10-04).
