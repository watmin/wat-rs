# BRIEF 6 — replay batch 3: grok-rete #126 → #152

> Built on batch 2 (and 5b's fold) and every per-step wall so far: the lint subset, the census + stone
> 3's gate (STOP-8, STOP-10), the stdlib door (STOP-9), and 5b's library unit tests + doctests (STOP-11).
> Ends before #153, the first step that changes grok's own codemods.

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Verify with `pwd` and
`git rev-parse --abbrev-ref HEAD`. **Never use worktrees.** Do not touch `~/work/holon/` or `main`.

## The work

Replay grok-rete commits **#126 through #152** (`#126 = 5696835f1`), one at a time, in order, by
`BRIEF-1-pilot-first-ten-commits.md` § "One step" with every row it now carries.

The census (`bootstrap/era/replay-plan/commits.tsv`): 14 docs-only, 6 code touching only grok-rete files,
7 touching a file main also changed; 100 files, 54 `.rs`. **None touches `wat/`, `wat-scripts/fixes/`, or
positional-ctor's skip-listed paths.** The honest deleted-file census (`absent-on-main.tsv`, finding 21)
names ONE step in range: #126.

## #126 — the trap door (31 files, 25 `.rs`)

#126 (`edn::write` reports instead of aborting) edits two files main MOVED to new homes — the standing
`.rs` rule: main's version stands, grok's change is re-expressed on it:
- `src/edn_shim.rs` (+147/−74: `value_to_edn_string_lossy` added; `value_to_edn_string_with` reports) →
  main's `src/edn/render.rs` (`value_to_edn_string_with` at `:4111`), per main's `8ddccaaa3`
  ("EDN gets a home — five loose root files become src/edn/");
- `src/string_ops.rs` (`render_str_total` calls `_lossy`) → main's `src/string/mod.rs:59`, per
  `56eb6ab3a` ("string_ops.rs is gone — 29 verbs, five homes").
If a change cannot be re-expressed without a third behaviour, that is STOP-3.

## Checkpoints

After **#139** and after **#152**: `scripts/floor.sh` (read the Summary line) and
`cargo clippy --release --all-targets -- -D warnings`. A red floor is reported verbatim and never re-run;
a composition repair folds into the step that needs it (the 4b rule), never a commit after the batch.

## The log

`REPLAY-LOG.md`, one row per step, with every per-step wall's verdict.

## ⛔ STOP triggers — each is a REJECTION. Commit nothing further; report the verbatim evidence.

STOP-1 … STOP-11 as `BRIEF-1-pilot-first-ten-commits.md` § "STOP triggers" and BRIEF-5/5b state them.

## Tier

Commit each replayed step on green. **Do not push.** Yield after #152, or at the first STOP, with
`SCORE-6-replay-batch-3.md` and `REPLAY-LOG.md`.
