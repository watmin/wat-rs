# EXPECTATIONS — the printer and the round-trip gate

Written BEFORE the strike. Every bar derived from a measured fact or a stated rule.

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the printer reproduces the hand-written row | print `:wat::core::char`, diff against `src/intrinsic/char.rs`'s fence | recognisably the same shape; every key present |
| 2 | the round trip holds on a RICH row | the gate, on `src/intrinsic/holon/hologram.rs` | `== doc`, with `yields` + `example-norun` exercised |
| 3 | the round trip holds on `@see` and `@syntax` rows | the gate, on rows carrying them | `== doc` |
| 4 | `@deprecated` is covered OR declared uncovered | the report | an explicit statement either way — **silence fails this row** |
| 5 | the gate is NOT vacuous | break the printer, run the gate | **RED**, naming the field or the margin |
| 6 | `wat-edn`'s writer is untouched | `git diff --stat crates/wat-edn/` | empty |
| 7 | no existing row changed | the registry census | **571 · 85 · 52**, unchanged |
| 8 | both crates' own tests hold | `cargo test --release -p wat-doc -p wat-macros` | green |
| 9 | the floor holds, doctests included | orchestrator, centrally, once | 5151/5151 or better, doctests exit 0 |
| 10 | clippy holds | `cargo clippy --release --all-targets -- -D warnings` | 0 |

Row 4 is the honesty row, and it is deliberately answerable by prose rather than a command:
`@deprecated` has **zero live users**, so no real row can exercise it. A gate that quietly omits a
field it cannot reach is the exact shape that ships a silent loss across 558 rows.

Row 5 is load-bearing. `no_rc_use.rs`'s doctrine — a gate never seen failing is a claim.

Row 7's derivation: the census measured **571 / 85 / 52** at `74bb197f8`. A printer and a test
change nothing about what is declared; movement here means Part 1 leaked into the corpus.

## Independent prediction

**45–75 minutes.** The printer is small. The margin-aware docstring is the fiddly part — it must be
the exact inverse of `dedent`, and "exact" is doing real work: a trailing-whitespace or
final-newline difference will surface as a round-trip failure, not as a cosmetic one.

## Trap doors — named before, not after

1. **The inverse may not be exact.** `dedent` strips the least-indented line's margin. If a
   docstring's own first line has different indentation from its continuations, print→dedent may not
   be the identity. This is where the stone most likely breaks, and it IS the finding, not a nuisance.
2. **`@example` vs `@example-norun`.** The run flag lives on `ExampleSubmission`, not as a separate
   field. If the printer emits both kinds identically, the flag is lost and row 2 catches it — but
   only because `hologram.rs` was chosen for carrying `example-norun`. That is why the row was named.
3. **`DocComment` may hold fields with no EDN spelling.** The transcoder is already known non-total
   (`Tagged`/`Inst`/`Uuid`/`BigDec`/namespaced-`Symbol`). A printer emitting only what the entry
   holds should not produce one — but "should not" is not "cannot", and STOP-2 covers it.
4. **The census (row 7) is a lagging indicator.** It reads the registry, not the doc text. A printer
   that is wrong but never invoked at compile time leaves it green. Rows 1–3 are the real check.

## What I will do on return

Re-run rows 1–8 myself before scoring. Rows 9 and 10 are mine alone and are the only verdict on
green. The rider's numbers are a hypothesis until a current `file:line` confirms them.
