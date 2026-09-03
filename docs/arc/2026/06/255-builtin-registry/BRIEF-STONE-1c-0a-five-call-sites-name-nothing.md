# BRIEF — STONE 1c-0a: two renames by codemod, and three artifacts to MEASURE

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1c-0a-five-call-sites-name-nothing.md`

## The work, in one paragraph

Five names are called in the `.wat` corpus and defined nowhere — no registry row, no dispatch
arm, no `CheckEnv` scheme, no `wat/` `defn`, not on the `RETIREMENT_TABLE`. They type-check today
only because `is_reserved_prefix` accepts anything under `:wat::`. **Two are namespace slips with
a registered target and you will fix them with a wat-fix codemod. Three are artifacts whose
load-bearing call names a verb that never existed, and for those your deliverable is a
MEASUREMENT and a REPORT — not a fix.**

## PART 1 — the two renames, via the self-hosted codemod

```
:wat::core::println     →  :wat::kernel::println
:wat::core::edn::write  →  :wat::edn::write
```

Four call sites, three files:
`wat-scripts/scratch-pad/probe-stone-2a-bracket-mechanics.wat:53` ·
`wat-scripts/scratch-pad/t-bare.wat:1` ·
`wat-scripts/probes/arc-170/probe-process-only.wat:6` ·
`wat-scripts/probes/arc-170/probe-edn.wat:2`

**`.wat` corpus rewrites go through the codemod — never a hand-edit, never python or sed.** This
is R21 and it is not negotiable for a multi-site structural change.

1. **Read `wat/fix.wat`'s header**, then read
   `wat-scripts/fixes/rename-core-string-to-string.wat` and
   `wat-scripts/fixes/deprime-telemetry-sqlite.wat` — the latter shows two
   `:wat::fix::rename-keyword-prefix` calls composed in one script, which is the exact shape you
   need for two renames in one migration.
2. Write **`wat-scripts/fixes/rename-slipped-core-heads-to-their-homes.wat`**, composing the two
   renames. Mind `rename-keyword-prefix`'s boundary rule — `wat/fix.wat:632`'s
   `rename-valid-match?`, and the boundary note at the top of `deprime-telemetry-sqlite.wat`.
   ⚠ `:wat::core::edn::write` must not be reached by a `:wat::core::` → `:wat::` style prefix
   rule that would also hit unrelated `:wat::core::` heads. Rename the **whole** head.
3. **Dry-run on a `/tmp` copy and `diff`** before touching the corpus. Confirm the diff is exactly
   four changed call sites and that comments, formatting and every other byte survive identically.
4. Apply: `printf '["path" …]\n' | cargo wat ./wat-scripts/fixes/rename-slipped-core-heads-to-their-homes.wat`
   listing **every** path. Re-run it to confirm idempotence (second run = 0 changes).
5. Commit the codemod itself as the recorded migration — it is an artifact, not a throwaway.

## PART 2 — the three artifacts: MEASURE, then REPORT. Do not fix, delete, or invent.

Each file below documents what it proves or measures. Each names a verb that does not exist, at
the exact point where the proof happens. **Your job is to establish, from the disk, what the
artifact claims and whether that claim can be supported — and to report it.** The disposition is
not yours to choose and not the rider's to guess.

| file | the call that names nothing | what to establish |
|---|---|---|
| `wat-scripts/scratch-pad/bench-reduce-foldl-vs-seqable-walk.wat:30` | `:wat::core::reduce-walk` | Read the whole header. What does ARM B claim to measure? Has this bench ever produced a number, and is any number from it quoted anywhere in `docs/`? What standing ruling does its header carry that would be lost if the file went away? |
| `wat-scripts/probes/arc-170/probe-cap2-process-grantpath.wat:10` | `:wat::spawn::process/grants` | The probe says it proves the PROCESS grant path runs end-to-end. Read `wat/spawn.wat`: what IS the real grant API, if any? Is there a spelling that would make this probe do what it says, or was the path never built? |
| `wat-scripts/scratch-pad/arc109-2iii-fn-bracket-destinations.wat:55` | `:wat::core::tuple-get` | Does the corpus have ANY way to read an element out of a `(:wat::core::Tuple :- [...])`? Search every rendering. If there is none, say so plainly — that is the finding. |

For each, report: the artifact's own stated claim (quoted), what you measured, whether the claim
is supportable, and the options you can see — **with no recommendation you cannot ground.**
"I cannot tell" is a correct and welcome answer.

## Blast radius

`wat-scripts/fixes/rename-slipped-core-heads-to-their-homes.wat` (new) and the **four** call sites
that codemod rewrites. **Nothing else.** No `src/` change. No registration. No file deleted. No
`.wat` hand-edited. The three PART 2 files are **read-only** for this stone.

## STOP triggers — halt and report, do not improvise

- **STOP-1.** The dry-run diff is not exactly the four intended call sites. Report the diff; do
  not narrow the rule by hand-editing a file the codemod missed.
- **STOP-2.** You are tempted to make a PART 2 file work — by writing the missing verb, changing
  the call to a different verb, or deleting the file. **Stop.** All three are explicitly refused
  by the DESIGN's four-questions table. Report instead.
- **STOP-3.** `:wat::rete::f64::>X` is NOT in this stone. It is a deliberate committed negative
  control (`wat-scripts/scratch-pad/probe-f64-comparator-bogus-head.wat`) whose header is this
  arc's founding evidence. Do not touch it; do not "fix" the typo.
- **STOP-4.** A test outside `every_wat_scripts_file_loads` goes red. Copy that test's entire
  stdout and stderr block verbatim from `.floor/latest/raw.log`, name the exact assertion that
  fired, and report — before re-running anything.

## Verification, in this order

```bash
./target/release/wat --check wat-scripts/scratch-pad/t-bare.wat            # and the other three
cargo build --release 2>&1 | tail -20
./scripts/floor.sh > /dev/null 2>&1; echo "EXIT=$?"
grep -E "^\s+Summary" .floor/latest/raw.log | tail -2
cargo clippy --all-targets -- -D warnings 2>&1 | tail -20
```

`every_wat_scripts_file_loads` must stay green — the four rewritten files still have to parse and
type-check. Read the Summary line, never a piped exit code.

## Acceptance — derived, not estimated

```
call sites rewritten          exactly 4, in 3 files, by codemod
codemod idempotent            second run reports 0 changes
GAP_A / GAP_B / DEBT          60 / 68 / 106  — ALL UNCHANGED, nothing is registered here
floor                    5127/5127 → 5127/5127
clippy                                    0
PART 2                        three written findings, zero files changed
```

## Working rules

Everything foreground. You may not spawn sub-agents. No worktrees, no `git stash`, no
`git revert`, no commit, no push — leave the tree dirty and report; the orchestrator commits.
The PART 2 report is this stone's primary deliverable; the two renames are the easy half.
