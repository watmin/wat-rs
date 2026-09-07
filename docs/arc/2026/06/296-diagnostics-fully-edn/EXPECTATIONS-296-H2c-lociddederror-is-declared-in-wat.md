# EXPECTATIONS — 296 H-2c: LociDiedError is declared in wat

Written BEFORE the strike.

⚠ **A GREEN FLOOR IS NOT THE BAR HERE.** Retyping three string literals in the new shape turns the
floor green and leaves the defect class untouched. Rows 1 and 2 exist so the floor cannot be reached
by the patch.

| # | what | command | expected |
|---|---|---|---|
| 1 | ⛔ the enum is DECLARED IN WAT | `grep -n 'defenum.*LociDiedError' wat/kernel/*.wat` | one hit. Zero hits = the stone did not happen, whatever the floor says |
| 2 | ⛔ Rust SOURCES from it | `grep -n 'wat_enum_from!' -A3 src/**/*.rs \| grep -c LociDiedError` | ≥1 — a generated enum, not a hand-maintained list |
| 3 | the hand-typed identity is gone | `grep -rn '"wat.kernel.LociDiedError"' src/ --include=*.rs` | **0** outside comments |
| 4 | the producer goes through the writer | `git diff src/process/verbs.rs` | the hand-built `Tag::ns(...) + Vector` is gone |
| 5 | the chain decodes | `cargo nextest run --release -E 'test(cache_probe_startup_error)'` | passes — the emitted chain STRICT-decodes to typed records |
| 6 | the message survives as DATA | `cargo nextest run --release -E 'test(deftest_wat_tests_test_test_assert_eq_fail_populates_message)'` | passes with the MESSAGE, not the stringified envelope |
| 7 | H-2's writer did not move | `cargo nextest run --release -E 'test(probe_arc296_h2)'` | 3/3, `#[ignore]` count 0 (STOP-4) |
| 8 | the floor | `./scripts/floor.sh` unpiped, Summary line | `0 failed` |
| 9 | clippy | `cargo clippy --release --all-targets --workspace` | 0 errors |
| 10 | the drift gate is GONE, not passing | the SCORE | if a gate existed to keep a Rust list in step with wat, it is DELETED — the precedent's own words: *"the drift gate that used to guard them is gone rather than merely passing"* |

## RUNTIME PREDICTION

**30–50 min.** The declaration plus the generated enum is the `intrinsic/mod.rs` shape and is quick;
the cost is the three consumer/producer sites and the stdlib-freeze dance if the new declaration must
be visible to the build that consumes it (STOP-2).

## TRAP DOORS

- **The stash dance.** `wat/kernel/diagnostics.wat` is frozen into the binary at build time. A new
  `defenum` there is invisible to the embedded copy until rebuilt, and `wat_enum_from!` reads the
  file at COMPILE time — so the ordering matters. `wat/fix.wat:22-53` is the written procedure.
- **`builtin_enum_variant_names` panics on disagreement** — that panic is a wall doing its job, not
  an obstacle. If it fires, the declaration is wrong.
- **The 24 failures are ONE root.** If fixing it leaves a residue, the residue is a SECOND finding
  and belongs in the SCORE by name — not absorbed.
- **ServiceEvent (7 arms) and sqlite::Cell (2) share the hand table.** They are not in scope, but if
  the compiler drags them in, that is STOP-5 and a report, not a widened strike.
