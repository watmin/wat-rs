# EXPECTATIONS — Stone 4c: truth maintenance / retraction

Independent scorecard, fixed BEFORE the strike. Weigh the fact-model fix (input distinct from derived) and the
transitive cascade hardest.

| # | what | command | expected |
|---|---|---|---|
| 1 | retraction TM (4 cases) | `cargo test --release -p wat --test probe_arc278_4c_retraction -- --include-ignored` | **4/4 GREEN** (input distinct; retract drops consequence; transitive; precise) |
| 2 | cascade still green | `cargo test --release -p wat --test probe_arc278_4b_cascade -- --include-ignored` | 4/4 |
| 3 | production-fire still green | `cargo test --release -p wat --test probe_arc278_4a_production_fire -- --include-ignored` | 4/4 |
| 4 | join / root / alpha | `…3b_hash_join / …3a_root_join / …2b_insert_alpha -- --include-ignored` | 4/4 · 3/3 · 3/3 |
| 5 | matcher / data model / compile | `…2a_alpha_match / …1a_data_model / …1b_compile -- --include-ignored` | 3/3 · 1/1 · 2/2 |
| 6 | load order | `cargo test --release --test test_stdlib_load_order \| grep result` | 1/0 |
| 7 | lib floor | `cargo test --release -p wat --lib 2>&1 \| grep "test result"` | 931/36 (UNCHANGED) |
| 8 | deftest floor | `cargo test --release --test test 2>&1 \| grep "test result"` | 264/1 (UNCHANGED) |
| 9 | build clean | `cargo build --release 2>&1 \| tail -2` | Finished; 25 warnings (NO new — pure WAT) |

## Trap-doors named — weigh hardest

- **The fact-model fix is the headline.** After `fire-rules`, `Session.facts` must be the INPUT facts only — no
  derived fact may leak in (probe Part A: `ColdAndWindy` count in `Session.facts` == 0, but == 1 in
  `production-memory`). If `fire-rules` still returns the closure as `facts`, retraction silently fails to
  cascade (the derived fact persists as if asserted). Read the new `fire-rules`: it captures `input` *before*
  `fire-fixpoint` and threads `input` (not `fired`'s facts) into the result.
- **`fire-fixpoint` is a verbatim rename.** The recursion body must be byte-identical to the old `fire-rules`
  (only the name + the recursive self-call rename). 4b cascade green (row 2) proves matching still sees
  input ∪ derived. If the body changed, something else moved — read the diff.
- **Transitive cascade.** `WeatherAlert` derives from `ColdAndWindy` which derives from `Temperature`. Retract
  `Temperature` → BOTH gone (probe Part C: WeatherAlert == 0). A retract that drops only the direct consequence
  but leaves the transitive one means the re-fire isn't recomputing from a clean input base.
- **Precision.** Retracting Oslo's `Temperature` must leave Bergen's `ColdAndWindy` standing (probe Part D:
  count == 1). An all-or-nothing wipe is wrong.
- **`retract` is by-value, stage-only.** It removes facts structurally `=` to the argument and does NOT fire
  (symmetric with `insert`); the caller re-fires. If `retract` itself re-fires, that's a scope/semantics drift.
- **No scope creep.** No `Snapshot`, no support-store / `matches`-chain cascade, no delta retraction, no
  `query`/`defrule`, no Rust.

## Weigh (orchestrator — extra rigorous)
1. Re-run rows 1-9 myself; 8/8 EXACTLY baseline (only row 1 flips RED→GREEN).
2. Read the diff: `fire-fixpoint` is a verbatim rename (incl. the recursive self-call); the new `fire-rules`
   captures `input` then restores `facts = input`; `retract` removes by structural `=` and only stages.
3. Reason about replay TM: with `facts` = input only, retract-then-fire recomputes the closure from a smaller
   input → consequences vanish, transitively, by construction. Confirm no path re-introduces derived facts into
   the retractable base.
4. Confirm `render-dag` fixture untouched; no Rust in the diff.
5. Commit SCOPED on green; push.
