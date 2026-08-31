# BRIEF — STONE: the bare-symbol shorthand dies

Kill the bare-symbol `Some`/`Ok`/`Err` shorthand outright. Arc 109 slice 1h retired it **at
constructor sites only**; the match-pattern half was never closed, and the constructor half is still
reachable at runtime past the checker. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-bare-symbol-shorthand-dies.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or
`scripts/floor.sh`. You may run the pre-existing `target/release/wat` for a fast read, and you run
the codemod (step 1) because that is how `.wat` migrations are applied. **You may not spawn
sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit,
push, stash, revert, or `git checkout --` anything.

## Read in order

1. The DESIGN above — the pinned contract decision and the load-bearing ORDER.
2. `docs/arc/2026/06/255-builtin-registry/NOTE-the-bare-symbol-constructors-are-retired-at-the-door-and-live-behind-it.md`
   — the measurements, including the `eval-ast!` probe that proves the runtime path live.
3. `wat/fix.wat`'s header — the codemod framework, and its **BOOTSTRAP / STASH-DANCE** note, which is
   the documented path when a codemod ships alongside a checker change that outlaws the old form.
4. `wat-scripts/fixes/rename-sort-prime-to-native.wat` — a recent, small, one-rule codemod to copy
   for shape.
5. `src/check.rs:5896,5921,5941` and `:6206` — the arms that ACCEPT a bare-symbol pattern head today.
6. `src/runtime.rs:5183,5186,5189` (`eval_list`, constructors) and `:16102,16135,16164,16192,16195,16196`
   (`try_match_pattern`, patterns) — the two families to delete.
7. `src/remedy/retirement.rs` — the existing `Some`/`Ok`/`Err` retirement rows and their remedy text,
   which the pattern door must reuse rather than reinvent.

## The work — the ORDER is load-bearing

### 1 — migrate the corpus, by codemod

Five bare-symbol **pattern** sites, in three files:

```
wat-scripts/perf/grid/where-control.wat
tests/cli/wat_cli__programs_are_atoms.wat
tests/cli/wat_cli__presence_proof.wat
```

Write `wat-scripts/fixes/bare-symbol-shorthand-to-fqdn.wat`, dry-run it against `/tmp` copies and
`diff`, confirm the diff is only the intended rename, then apply and re-run once to confirm zero
changes. ⛔ Do not hand-edit the `.wat`, and do not reach for sed/python.

Also migrate the Rust inline-wat fixture in `tests/function/wat_arc170_closure_extraction.rs` — that
one is a Rust string, so it is an ordinary edit, not codemod territory.

### 2 — close the pattern door

The `check.rs` arms that accept a bare-symbol pattern head must refuse it, with the **same remedy the
constructor door already gives** — reuse the existing retirement machinery and its message; do not
write a second, differently-worded refusal for the same retirement.

### 3 — delete both families of runtime arms

Constructors (`eval_list`) and patterns (`try_match_pattern`). After this, the runtime cannot
evaluate the shorthand even when reached without the checker — which is the heresy's actual death,
and the acceptance row that matters most.

## Blast radius

`wat-scripts/fixes/bare-symbol-shorthand-to-fqdn.wat` (new) · three `.wat` corpus files ·
`tests/function/wat_arc170_closure_extraction.rs` · `src/check.rs` · `src/runtime.rs`. No changes to
`:wat::core::None`, to any registration, or to `src/remedy/retirement.rs`'s existing rows.

## STOP triggers — each rejects; ship nothing further on that point and report

**STOP-1 — order.** If closing the door first makes the corpus uncheckable and you cannot get the
codemod to run, STOP and report. `wat/fix.wat`'s BOOTSTRAP / STASH-DANCE note is the supported path;
hand-editing `.wat` to escape the ordering is not.

**STOP-2 — one refusal, not two.** If reusing the constructor door's remedy for the pattern door
proves impossible, STOP and report what blocked it. Two differently-worded refusals for one
retirement is the drift this stone exists to end.

**STOP-3 — a sixth site.** The population is 5 pattern sites plus 1 Rust fixture, validated against
the checker rather than grep (a first pass counted 20 constructor sites; 14 were comment lines and
the rest prose). If you find a site the design did not name, STOP and report it — do not migrate it
silently, because the count being wrong is itself the finding.

**STOP-4 — `None` is not in scope.** Its `eval_list` occurrence is a pattern-clause head inside
`match`'s own implementation, excluded by `meter-2` with a cited reason. If your change appears to
require touching it, STOP and report.

## Report

Per-file diff summary; the codemod's dry-run diff and its second-run idempotence result; and the
output of these three probes against the pre-existing binary (noting it lacks your Rust changes), so
the orchestrator can compare after rebuild:

```
(:wat::core::match (:wat::core::Some 5) ((Some v) v) (:wat::core::None 0))   -- pattern door
(:wat::eval-ast! (:wat::core::quote (Some 99)))                             -- runtime path
(:wat::core::Some 1)                                                        -- must still work
```

Then the part the orchestrator cannot reconstruct: what surprised you — an arm whose deletion
reached further than expected, a refusal path that did not reuse cleanly, or a site the validated
population still missed.
