# EXPECTATIONS 7g — replay batch 4g, grok-rete #261 → #280 (written BEFORE the strike)

The orchestrator re-runs every row on the SCORE commit, uncontended. Fixed now so the result cannot move
the goalposts.

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | 20 steps, correctly subjected | `verify-step-record.sh <start> HEAD 261 280` | exit 0; `step-range: #261..#280 each present exactly once, sources match` |
| E2 | docs-only steps are docs-only | `git show --name-only` on all **15** (#261 #263 #264 #265 #267 #268 #269 #271 #272 #273 #275 #276 #277 #279 #280) | only `docs/`/`.md` |
| E3 | **the #274 meta-gate is GREEN at HEAD, both arms** | `cargo nextest run --release -E 'test(every_discovering_gate_declares_how_it_knows_it_reached_something)'` | 1 test run, PASS — neither `undeclared` nor `hollow` fires |
| E4 | **all ten repair targets carry a declaration** | `grep -LiE 'non-vacuity\|rune:lint\(vacuity-guard\)'` over the ten named files | **empty output** — no file lacks one |
| E5 | **the declarations are not HOLLOW** | for each of the ten: marker line, then an assertion within the 12 lines below | every one has an assertion under its marker |
| E6 | ⚠ **#278's dead path was NOT recreated** | `test -e wat-scripts/scratch-pad/probe-f64-comparator-bogus-head.wat` | **absent** (exit 1) |
| E7 | ⚠ **no INERT rune was added to the moved copy** | `grep -c 'rune:lint(rete-name-unminted)' tests/resolve/probe_arc255_the_blanket_hides_a_phantom_head__bogus_rete_head.wat` | **0** — the gate never walks `tests/`, so a rune there is decoration |
| E8 | **the #278 gate is green** | `-E 'test(every_rete_name_in_wat_scripts_code_resolves)'` plus its three sibling controls | all green, **N > 0 selected** |
| E9 | `wat-scripts/fixes/` edited at **#278 and nowhere else** | `git log --format='%h %s' --name-only <start>..HEAD -- wat-scripts/fixes/` | only #278's commit appears |
| E10 | every produced `.wat` checks | `--check` each (#262's 6, #278's 5) | rc 0, or a deliberate `.bad` with its reason |
| E11 | named tests at the code steps | `-E` per step (#262 #266 #270 #274 #278) | green, **N > 0 selected** |
| E12 | **#274's repair landed AT #274, not folded backward** | `git show --name-only` on the #274 REPLAY commit | it carries the repaired `tests/lint/*.rs` files itself |
| E13 | finding 33's class was actively looked for | the REPLAY-LOG / SCORE | each rename-ish step records whether the `.rs`/`.sh` side was grepped — "not applicable" is an answer, silence is not |
| E14 | the checkpoint | `scripts/floor.sh` + `cargo clippy --release --all-targets -- -D warnings` | green; clippy 0 |
| E15 | **test-count delta ACCOUNTED FOR** | predict from the diff BEFORE the floor runs | predicted == actual, exactly |
| E16 | spot re-run of the walls | orchestrator re-runs lint-subset + `kind(lib)` + doctests + stone-3 at HEAD vs the last code step's verdict lines | identical numbers |
| E17 | no knowingly-red commit | no repair commit after #280 | none |
| E18 | every artifact a body names exists | each `.census/…txt` cited | all present on disk |
| E19 | **every repair visible to `push`** | `git replace -l`; gate under `GIT_NO_REPLACE_OBJECTS=1` | **0 replace refs**; gate exit 0 either way |
| E20 | **no verdict line is WRAPPED** | grep each pattern across the range | every required line matches on ONE line |

## ⛔ Every `-E` filter must be confirmed to SELECT A NON-ZERO COUNT

A nextest filterset matching nothing runs zero tests and **exits 0** — a mis-aimed probe is
indistinguishable from a working gate. Read each run's own `N tests run` and require N > 0.

## ⛔ E4/E5 are the rows that catch a lazy #274

A `// NON-VACUITY:` comment placed above nothing, or a guard written as `assert!(violations.is_empty())`,
satisfies a careless grep and proves nothing — that is the exact defect `every_ungated_wat_checks.rs`
already carries, and the meta-gate's own `hollow` arm exists to refuse it. **E5 re-reads each declaration
for an assertion underneath it.** A gate whose only assertion is that its violation list is empty has NOT
declared non-vacuity: it passes when its walk finds nothing at all.

**Runtime prediction:** ~90–130 min. Fifteen docs cherry-picks are quick; the weight is #274 (20 `.rs`
PLUS a ten-file repair the orchestrator predicted but did not perform), #262 (15 files), #278 (7 files
with a dropped hunk).

**Trap doors named in advance:**
- **#274 — the meta-gate WILL go red on arrival, with 9 undeclared + 1 hollow.** That is the step's
  declared work, not a STOP. It repairs at #274 (the #184 precedent), never folded backward, and never by
  weakening or allowlisting the gate.
- **#278 — the probe hunk has no landing site.** Drop it; do not recreate the dead path; do not add an
  inert rune to the moved copy under `tests/resolve/`.
- **#278 edits three recorded codemods** — expected, and the only `wat-scripts/fixes/` touch in the range.
- **#262 modifies two files created at #258**, inside batch 4f. Absent ⇒ 4f's #258 is at fault, not you.
- **finding 33's class**: any step mentioning a rename must have the `.rs`/`.sh` side grepped.

**What would make me reject the batch:** the meta-gate weakened, allowlisted, or its red deferred; a
`// NON-VACUITY:` marker with no assertion beneath it; the #278 probe recreated at its dead path; an inert
rune under `tests/resolve/`; any `refs/replace/` entry; or a `.wat` left failing `--check` without a
stated, checkable reason.
