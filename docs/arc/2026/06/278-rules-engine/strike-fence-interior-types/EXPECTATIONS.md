# EXPECTATIONS — the fence interior is type-checked

Written **before** the strike. The probe (`d8068e26a`) already fixes most of these; the rows that
are not the probe are the ones that decide whether the cure is real.

| what | command | expected |
|---|---|---|
| the gap arm unlocks and passes | `cargo nextest run --release -E 'test(fence_interior)'` | **3 tests, 3 passed** (2 today) |
| the over-rejection guard holds | same run | `legal_fences_still_compile` **PASS** — including row 3's computed operand |
| the inline twin is untouched | same run | `inline_twin_is_refused` **PASS** |
| the bank is cleared, not worked around | `grep -c 'bad-is-banked' tests/rete/probe_arc278_fence_interior_types_fence.wat.bad` | **0** |
| the `.wat.bad` gate now demands the fixture FAIL | `cargo nextest run --release -E 'test(every_wat_bad_fixture_actually_fails)'` | **27 passed** — with the rune gone, this gate is what proves the fixture genuinely fails now |
| ⛔ **the contract decision held** | read the new variant in `src/rete/validate/error.rs` | it carries **no `fact_type` field**. A sentinel string is the row this scorecard exists to catch |
| the retired claim is recorded, not erased | `git diff src/rete/validate/mod.rs` | the `:282` comment is **rewritten** to say what the arm now does. A deleted comment loses the record that "interior out of scope" was once asserted |
| ⛔ **MUTATION A — the check is load-bearing** | make the new interior walk a no-op **in the live source**, re-run `-E 'test(fence_interior)'` | `fence_interior_type_error_is_refused` **REDs**. Restore ⇒ green. Mutate the shipping code, never a copy of it |
| ⛔ **MUTATION B — trap 2 is actually cured** | a fence fixture whose operand is a **keyword constant** of the wrong type | **refused.** Then restore `is_non_field_keyword`'s suppression and re-run ⇒ it must go **silent**. One mutation cannot prove a two-arm cure, and this is the arm the naive reuse drops |
| the census is reported | the floor's red list, or its absence | either a **verbatim list** of every `.wat` that newly fails, or an explicit *"zero — no other fence interior is ill-typed"* |
| floor | `./scripts/floor.sh`, foreground; read `.floor/latest/clean.log` | **0 failed.** Passed **≥ 5493**, and skipped goes **20 → 19** — that transition is the bank clearing, and it is the row to check, not the total |
| clippy | `cargo clippy --all-targets --release -- -D warnings` | rc=0 |

## Runtime prediction

**60–90 minutes.** Two floors at ~8 min each dominate; the walk itself is small. The census is the
variance — if STOP-1 fires the strike ends early with a list instead of a cure, and that is a
success, not a failure.

## Trap doors

- ⛔ **Reusing `is_non_field_keyword` verbatim.** With empty `field_names` it returns `true` for
  *every* keyword, silently exempting the whole keyword-constant class. The cure would ship a hole
  of the same shape as the one it cures, and every row above except MUTATION B would still be
  green. This is the single most likely way this strike fails while looking finished.
- **A sentinel `fact_type`** (`""`, `"<where>"`) to reuse `ConstraintTypeMismatch`. One line
  smaller, and it prints a location that is not where the mistake is.
- **Modelling `let`/`match`/`if` in the walk.** `src/rete/clause.rs:242` already enumerates those shapes; a
  second copy is a second grammar to drift. Walk every list node instead.
- **Asserting "it errored".** A malformed fixture fails for many reasons. Assert the error KIND —
  the same standard `probe_arc278_D10_then_field_types.rs` holds itself to.
- **Treating a large census as a crisis.** It is the progress meter, and it is the deliverable the
  finding predicted: *"nothing has ever looked at the other fence interiors."* Report it; do not
  start migrating.
- **Believing this brief.** Every line was measured once, by one hand, on one box, at `d8068e26a`.
  Reproduce the two-position drive before you lean on any of it — and if the clause-level twin at
  `src/rete/validate/mod.rs:456` turns out to matter, that refutation is worth more than the cure.
