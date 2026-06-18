# EXPECTATIONS — Stone 2a: `alpha-match`

Independent scorecard, fixed BEFORE the strike. The orchestrator re-runs each row + reads the diff; the
worker's report is a hypothesis until the disk confirms it.

| # | what | command | expected |
|---|---|---|---|
| 1 | matcher binds + constrains + rejects | `cargo test --release -p wat --test probe_arc278_2a_alpha_match -- --include-ignored` | **3/3 GREEN** |
| 2 | lib floor unmoved | `cargo test --release -p wat --lib 2>&1 \| grep "test result"` | 931 / 36 |
| 3 | deftest floor unmoved | `cargo test --release --test test 2>&1 \| grep "test result"` | 264 / 1 |
| 4 | load order green | `cargo test --release --test test_stdlib_load_order \| grep result` | 1 / 0 |
| 5 | build clean | `cargo build --release 2>&1 \| tail -2` | Finished; no NEW warnings (baseline 25) |

## Runtime prediction
~25-40 min. New home + one intrinsic + the matcher logic (the substance) + a check scheme. The field-read +
compare are mirrors of `eval_form_matches`; the novelty is the pure bindings-fold + the `<-`/FQDN classifier.

## Trap-doors named (weigh these hard)
- **PURITY.** The whole point. Confirm the diff calls NO `eval_inner` and threads NO `Environment` — operands
  resolved only from {bindings, field, literal}. If `eval_inner` appears, reject (it's `form::matches?`'s sin).
- **No-error semantics.** Wrong type / missing field / failed constraint → `None`, never a raised
  `RuntimeError`. Probe rows 2/3 are the canary; also spot-check a missing-field condition returns None.
- **Field-by-name correctness.** The registry index must map `:field` → the right `fields[idx]`. A
  silent off-by-one would bind the wrong value — read the field-lookup against `eval_form_matches`'s.
- **Bindings key form.** `"?t"` (string, leading `?` kept) — the probe's `PersistentMap/get "?t"` pins it.
- **No scope creep.** `where` must STOP (not a stubbed-true); cross-fact `?var` (join) must STOP. Confirm the
  worker didn't quietly implement either.
- **New home hygiene.** `src/rete/` mod wiring clean; no new warnings beyond the standing 25.

## Weigh (orchestrator, against own re-run + the diff)
1. Re-run rows 1-5 myself; rows 2/3/4 EXACTLY the baseline.
2. Read `src/rete/matcher.rs`: pure (no env/eval_inner), the classifier matches the DSL shapes, no-error
   semantics, field-read mirrors the registry path, `where`/join STOP-guarded.
3. Read the dispatch arm + check scheme.
4. Commit SCOPED on green; push.
