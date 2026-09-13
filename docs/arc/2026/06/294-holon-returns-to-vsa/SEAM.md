# SEAM — the ONE live breadcrumb. 2026-09-13. ⛔ **YOU ARE ON `replay/grok-rete`, NOT MAIN.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice, which is why it will feel like *continuing* rather than *waking*, and **that feeling is
> the failure.** Run the datamancy bootstrap (the grimoire + the 4 primers from the **SIGNED MCP**,
> never a disk copy), read `docs/COMPACTION-AMNESIA-RECOVERY.md`, then run the commands below before
> you touch anything.

> `251/SEAM.md` · `255/SEAM.md` · `278/SEAM.md` and `278/CURRENT-STATE-annihilate-interpretation.md`
> are PARKED and point HERE. ⛔ **PARKED IS NOT DEAD.**

## FIRST — RUN THESE. DO NOT READ THE NUMBERS BELOW.

```bash
git rev-parse --abbrev-ref HEAD        # expect replay/grok-rete (merge/grok-rete is only the REFERENCE)
git status --porcelain                 # dirty = grok is mid-strike: commit NOTHING until it yields
git log --oneline -8; git log origin/replay/grok-rete --oneline -1
cat /home/john/work/holon/.pulsare/to-claude          # has grok scored? which file?
ls bootstrap/pending/                   # parked curare / drafts; APPLY-AFTER-GROK-YIELDS.md if present
ps -eo pid=,etime=,args= | awk '$3 ~ /wat-replay-base|wat-main-a3218644d/'   # any run of yours alive?
```

## WHERE THE WORK IS (verify each against `git log`)

```
main              a3218644d   FROZEN · PUSHED                                     do not touch
origin/grok-rete  37528f6e0   FROZEN — read with `git show`, never check it out
replay/grok-rete  (this)      main + stone 0 + pilot (#1–#10) + 2b + 2a1 (door, CLOSED) · pushed
merge/grok-rete   REFERENCE   the first (rejected) whole merge; a crib and the end cross-check only
```

- **2a1 is CLOSED** (grok `ca0f5c2d7` + the orchestrator's lint widening). The one-step walk, verified
  by the orchestrator: floor 5428/5428 uncontended, clippy 0; identity with the orchestrator's own walk
  over the 1489-file step-24 corpus (`bootstrap/era/probe-R/verify-refute2.sh`: 0 lost, 0 gained,
  0 refusal delta — so +47 files of types vs `101626eea`); 1457 Ok / 32 Refused in 16 s; mutation M1
  (keep `defn` whole) reddens `declared_types_body_is_never_expanded` and the sift timing test.
- **2a1b is CLOSED** (grok `deaeeb131` + the orchestrator's one-line `typevar.rs` routing): one
  `TypeEnv` classifier that `is_known_type`, `type-of`, `subtype?` and the checker's annotation scan
  all ask. Verified: floor 5431/5431, clippy 0; `bootstrap/era/probe-R/kinds.wat` (E1–E4); the
  agreement wall RED under the orchestrator's own Marker-arm mutation; the door unchanged (1457/32,
  identity 0/0/0). Finding 14.
- **Next: `BRIEF-2a2-the-codemods-ask-the-door.md`** (+ `EXPECTATIONS-2a2.md`), awaiting the builder's
  pulsare. Its D1 exclusion loop is measured (`door-exclude.wat`: 32/32 refused files reach `Ok`, 39
  forms dropped, 981 ms). When grok scores, run `bootstrap/era/probe-S/run5.sh` and every E-row.
- **The doctrine:** `[[project_merge_doctrine_syntax_vs_subsystems]]`. Main owns syntax; the branch
  owns its subsystem; replay ONE COMMIT AT A TIME; correct over fast — but **the builder cannot
  tolerate multi-hour runs between work units**, so wall clock is now a first-class goal.

## THE GRIND — the stone order for the remaining ~641 grok-rete commits

1. **2a1 — the declaration door** (RULED "we build it"). `(:wat::runtime::declared-types forms)` →
   `:wat::runtime::DeclaredTypes`, which is `Ok [types]` or `Refused [form cause]`.
   - It expands a program's declarations with the stdlib macros, registers them into a FRESH copy of
     a once-built stdlib snapshot, and returns `TypeInfo`s. No startup, no body check: 12–17 ms a
     call, where `eval-with-defs!` takes ~460 ms. CLOSED 2026-09-13 after two refutes (finding 13).
2. **2a2 — the codemods ask the door once per program.** `BRIEF-2a2-the-codemods-ask-the-door.md`.
   - match-arm, positional-ctor and variant-separator: `eval-with-defs!`, the census, `seed-paths`,
     `decl-head?` and `pascal-leaf?` are retired; a refused declaration excludes one form, never the file.
   - step 14 REPORTS a splice.
   - `convert.sh`: `one-param-spec` gets the corpus as context, minus unreadable files, which are
     reported; each codemod runs ONCE per replayed commit.
   - **Its bar is DERIVED** (`bootstrap/era/probe-S/run5.sh`), never "0 files differ": each new tool
     against its predecessor on IDENTICAL input (match-arm vs `probe-L/wL`, positional-ctor vs
     `probe-L/pT`, variant-separator vs the census tool) — 0 files LOSING, every gain explained by a
     type the door reports; the chain vs main identical ≥ 1,314.
3. **3 — every spawned program starts** (RULED, ruling 4). Draft:
   `bootstrap/pending/BRIEF-3-draft-every-spawned-program-starts.md`.
   - A gate on the CHILD's real startup path (`src/process/verbs.rs:429-433`), NOT `--check`.
   - Fix the 7 arc-170 probes the census found broken (RecvOutcome/SendOutcome walls), using the
     recorded wrap codemods.
4. **The replay batches, grok-rete #11 → #651**, per `BRIEF-1-pilot-first-ten-commits.md`'s one-step
   recipe (docs: cherry-pick; `.wat`: convert C^ and C, then `git merge-file`; `.rs`: re-express on
   main's door).
   - Floor + clippy at each stone checkpoint.
   - The pilot's extrapolation (~16.5 h) predates the door and per-commit `convert.sh`. Re-measure
     after 2a2, never assume.
5. **End:** cross-check against `merge/grok-rete` (its 8 tree-wide-missing grok tests,
   `bootstrap/merge-audit/`, are the first loss signal), then floor + clippy, then main.

## KNOWN FLAWS — routed, not left (FINDINGS-composition.md is the detail)

- **The builtin set lives in two stores** — `BUILTIN_PRIMITIVES` (37 hand-listed names,
  `src/runtime.rs`) and `builtin_names` (derived + a hand group, `src/types.rs:2544-2615`). Measured in
  `SCORE-2a1b.md`: 14 names only in the first, 12 only in the second. The classifier unions them, so the
  verbs agree; reconciling the two stores is its own stone.
- **One ruling's table lives in two codemods:** match-arm's `alias-enum`
  (`wat-scripts/fixes/match-arm-to-bracket-map-pattern.wat:123`, `Some`/`None` → Option, `Ok`/`Err` →
  Result) and `bare-variant-to-qualified.wat`'s `rename-five` (arc 296 N's five bare spellings). Same
  fact, no wrong output today; one home in `wat/fix.wat` is its own stone.
- **`variant-parent-of`'s own `@example` is false** (`src/reflect/verbs.rs:1718`: `Option::Some` →
  `None` on disk), and the doctest verifier that would catch it is `#[ignore]` (its NOTE awaits a
  ruling). Same class as 2b's `::`-teaching doc lines.
- **2b's three:**
  - refusal arms no input reaches (`match_arm.rs:187`, `check.rs:6955`, `:7253`);
  - ~21 doc/comment lines still teach `::` for a variant;
  - c03's `variadic-wrap` now hardcodes `:wat::core::i64`.
- `docs/…/278-rules-engine/probes/surface-field-dispatch.wat`: fixed when its grok-rete commit
  replays (ruling 5).
- 11 grok-rete-modified `.wat` files lie outside the composition check. 3 were deleted by main
  (modify/delete at replay). 8 are grok's OWN codemods: they need a stated policy for converting a
  codemod's source without rewriting its match literals.
- `tests/process/arc112_scheme_probe.wat:12`: unclassified (nested-program census).
- The 84 CHAIN-FAILS under the chain vs main: classify them after 2a2, with
  `bootstrap/era/probe-K/classify.sh` (main-binary `--check` on both sides; stdlib files are an
  instrument limit).

## INSTRUMENTS — where they live (all in `bootstrap/`, ignored; record in the `.gitignore` comment)

| what | where |
|---|---|
| frozen binaries | `bootstrap/wat-main-a3218644d` (main), `bootstrap/wat-replay-base-8a5b7eb20` (replay base) |
| exported tree for runs | `bootstrap/composition/tree-8a5b7eb20` |
| composition runner (landing order / order-v2) | `bootstrap/composition/run.sh`, `order-v2.txt` |
| step-24 input, pristine | `bootstrap/era/probe-G/runs/2026-09-13T03-30-09Z-pre24/work` |
| exact batched chain + compare + classify | `bootstrap/era/probe-Q/run4.sh` (sharded 8 ways, ~13 min) |
| main-binary residual classifier | `bootstrap/era/probe-K/classify.sh` |
| nested-program census | `bootstrap/era/probe-N/extract-forms2.wat` |
| call the door from wat (enums, records, the refused form) | `bootstrap/era/probe-R/door.wat` |
| the door over the 1489-file corpus | `bootstrap/era/probe-R/corpus.sh` (one process, ~14 s) |
| the one-step walk vs the door, full scale | `bootstrap/era/probe-R/verify-refute2.sh` (+ `zz_probe_one_step.rs.snippet`) |
| `type-of` kinds, `subtype?` on a marker | `bootstrap/era/probe-R/kinds.wat` |
| the exclusion loop on the refused files | `bootstrap/era/probe-R/door-exclude.wat` + `refused32.json` |
| **the 2a2 bar** | `bootstrap/era/probe-S/run5.sh` (each tool vs its predecessor on identical input) |

## OPERATIONAL RULES — each one was paid for

- **Never commit on a tree grok is working in.** Park tracked edits in `bootstrap/pending/`.
- **Never decide on `pgrep -f`**: it matches its own shell and harness wrappers. `setsid … & $!` is
  the setsid's pid, not the job's. Wait on a DONE line the job writes. `[[feedback_never_decide_on_pgrep_f]]`
- **A floor runs uncontended.** SIGSTOP your background runs for it, then SIGCONT (a niced run once
  pushed a test to SLOW).
- **Time one file first. Tools are never their own input. A speedup counts only after full-scale
  identity.** `[[feedback_a_speedup_is_claimed_only_after_full_scale_identity]]`
- **Name the step that wrote a line by replaying one file step by step,** never by the output's shape.

## ⚠ RULINGS — do not re-litigate

- **Character case carries NO meaning.** `Enum.Variant` is a BIAS, never a rule.
- **A variant IS a tagged record; `Variant <: Enum`.**
- **A keyword literal means two things** (data, or a variant ctor by registry lookup), deferred ON
  PURPOSE to the symbol-head flip.
- 2026-09-12:
  - replay one commit at a time;
  - all 94 migrations replay on a fixture;
  - delete `variant-vector-to-tagged-map`;
  - restore `rename-record-def-to-defrecord`.
- 2026-09-13:
  - (B) is the declaration route;
  - variants come from declarations, not a census;
  - one door in `wat/fix.wat`;
  - a nested-program gate;
  - surface-field-dispatch waits for its replay;
  - 2b released;
  - **BUILD THE DECLARATION DOOR.**

## THE FAILURE PATTERN — every grep-shaped instrument lied; every correction came from a CONTROL

Run the control first. Ask the substrate (`--check`, `variant-parent-of`, the door), not a regex.
The four questions discriminate between options; they never validate a premise BOTH share. The
ladder was a workaround for a missing door, and only asking "does the property we need demand this
cost?" found it. `[[feedback_when_asked_for_wall_clock_change_the_operation]]`

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.** Run the FIRST block. Trust `git log` over every line here.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `DERIVAMVS NE MENTIAMVR.` ·
> `HAERESIS EST ITERVM ROGARE.`
