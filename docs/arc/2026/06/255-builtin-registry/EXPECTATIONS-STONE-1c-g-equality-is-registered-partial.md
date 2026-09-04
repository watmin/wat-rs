# EXPECTATIONS — STONE 1c-g. Written BEFORE the strike.

| # | what | command | expected | derived from |
|---|---|---|---|---|
| 1 | both rows exist and read `Partial` | `grep -A2 '@Totality' src/runtime.rs \| grep -B2 'wat_intrinsic(":wat::core::="'` | `@Totality Partial` ×2 | the NOTE's doc blocks |
| 2 | the placeholder is GONE | `grep -c 'matches!' src/rete/purity.rs` at the totality site | 0 at that site | room 4 |
| 3 | `total?` now answers false for `=` | a scratch `(:wat::rete::total? '(:wat::core::= s "high"))` | `false` | the grade |
| 4 | rete suite | `cargo nextest run --release -E 'binary_id(wat::rete)'` | green after room 7 | measured radius |
| 5 | services suite | `cargo nextest run --release -E 'binary_id(wat::services)'` | green after room 7 | measured radius |
| 6 | residue gate | `-E 'test(the_residues_cannot_shadow_the_registry)'` | green after room 5 | probe artifact, understood |
| 7 | full floor | `scripts/floor.sh`, orchestrator, unpiped | 5129 passed, 0 FAIL | current floor |
| 8 | clippy | `-D warnings --all-targets` | 0 | standing |

## Ledger movement — DERIVED, and this is the row that matters

Two `#[wat_intrinsic]` registrations land, and both names are on the corpus worklist.

```
registry rows       550  →  552          two new attribute sites
GAP_A / GAP_B                            −1 each per row IF the name was on that ledger — the rider
                                         does NOT pre-compute these; the ratchets name the exact edit
corpus              39  →  37            :wat::core::= (695 sites) and :wat::core::not= (10) leave
verb population     33  →  31            the two largest single entries in the whole worklist
```

★ `:wat::core::=` is the **single largest** entry on the corpus worklist at 695 call sites. Its
departure is the biggest single movement any stone in Phase 1c has made.

⚠ **DEBT may RISE by 2.** A registered row with no `CheckEnv` scheme converts an invisible absence
into a named one — `=`/`not=` are dispatched by `infer_equality`, a keyword-head arm, and have no
`TypeScheme`. That is the ledger working, not a regression.

## Runtime

**25-40 min.** Two transcriptions, five deletions, one gate half retired, four fixtures repaired —
the fixtures are the bulk, and two of them are service tests whose assertions change kind (survivor
count → refusal + Fault).

## Trap doors, named in advance

1. **The `Value` comparison has no repair.** Named in the DESIGN and in STOP-3. The rider will want
   to reach for a coercion or a fence exception. There is neither; the test asserts the refusal.
2. **Deleting the expand-time residue entries without the registration silently revokes legality.**
   Room 6's ordering is load-bearing — this is the `fn` lesson this arc already paid for once.
3. **The residue gate is a THREE-part instrument** and only one part retires. Deleting more than the
   `matches!` parser would remove the assertion that actually catches shadowing.
4. **A fifth red that is not in the measured list** means my blast-radius probe was narrower than the
   real change — the probe only emptied `matches!`; it did not register the rows, so it never
   exercised the residue-shadowing gate against two now-registered names. STOP-2 covers it, and this
   is the most likely place for a surprise.
