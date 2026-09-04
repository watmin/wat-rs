# EXPECTATIONS — STONE 1c-f. Written BEFORE the strike.

Every bar below is DERIVED from a rule or from the orchestrator's own probe run, never estimated.
The one row that cannot be derived is marked as a measurement, not a prediction —
`[[feedback_an_acceptance_row_is_a_pin_unless_it_derives_its_bar]]`.

| # | what | command | expected | derived from |
|---|---|---|---|---|
| 1 | the aimed-probe canary fires | `--check` a 2-arity `(reduce f coll)` | **RED**, `expected 3 argument(s); got 2` | probe run 2 |
| 2 | Stream + PersistentVector sites survive | `--check` `probe_arc278_0d_transform_dispatch_parity.wat` | **exit 0** (RED ×6 without room 1) | probe run 2 |
| 3 | the other six call-site files | `--check` each | **exit 0** | probe run 2 |
| 4 | exactly ONE file needs augmenting | the arity census | `probe-118B2-rider-verification.wat` only | census, 19×3-arity / 1×2-arity |
| 5 | the corpus loader stays green | `cargo nextest run --release -E 'test(every_wat_scripts_file_loads)'` | pass | the gate parses every `wat-scripts/**/*.wat` |
| 6 | rete purity suite after the arm deletions | `cargo nextest run --release -E 'binary_id(wat::rete)'` | **MEASUREMENT, not a prediction** | door-3 hypothesis is untested |
| 7 | full floor | `scripts/floor.sh`, orchestrator, unpiped | 5129 passed, 0 FAIL | current floor |
| 8 | clippy | `-D warnings --all-targets` | 0 | standing |

## Ledger movement

**None predicted, and that is derived, not hedged.** This stone registers no new row: `defalias`
writes into `sym.functions`, never into the intrinsic registry (proven by the precedent
`:wat::core::count`, a wat-side alias for `length` with no registry row). So:

```
GAP_A 49 · GAP_B 44 · DEBT 119 · TYPES_UNCHECKED 10 · KNOWN_UNREVIEWED 13     ← UNCHANGED
corpus 39 (33 verb · 6 non-verb)                                              ← UNCHANGED
```

⚠ A stone that moves no ledger is normally suspect (the campaign's own rule). This one is exempt
**with a stated reason**: it is a step-3 *duplicate dies* move plus a step-1 *stale authority
corrected* move, and neither is what the four absence ledgers count. If the rider reports a ledger
change, that is a surprise worth investigating, not a bonus.

## Runtime

**12-20 min.** Two `cargo build --release` cycles (~20s each, measured during the lair study) plus
one scoped nextest, four small edits, and one prose block rewritten.

## Trap doors, named in advance

1. **The stale binary.** The single most likely failure. `wat/seq.wat` is `include_str!`ed; the
   orchestrator's own first probe run produced seven false greens against a stale binary, and a
   deliberately-undefined verb in the stdlib also read `exit=0`. Row 1 exists to catch exactly this.
2. **Room 5 comes back RED.** Then the arms are live and `head_ok`'s door 3 does not intercept the
   way the design hypothesises. Restoring both arms is the correct outcome and the stone still
   ships. The hypothesis is the orchestrator's, not the rider's, and it is the orchestrator's to be
   wrong about.
3. **A second 2-arity caller.** Would mean the corrected tokenizer is still wrong. The first
   tokenizer returned impossible "7-arity"/"8-arity" rows; the second agreed with an eyeball on all
   20 sites, but agreement of two instruments sharing an input is not corroboration —
   `[[feedback_two_instruments_agreeing_is_not_corroboration]]`. STOP-2 covers it.
4. **The `Seqable` widening admits something `infer_foldl` would refuse.** The scheme is only read
   for alias derivation (direct calls are intercepted), so the blast radius is aliases of `foldl` —
   of which, after this stone, there is exactly one. Named, bounded, and the floor is the check.
