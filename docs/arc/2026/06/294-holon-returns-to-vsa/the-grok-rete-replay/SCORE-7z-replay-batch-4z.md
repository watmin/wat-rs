# SCORE 7z — replay batch 4z, grok-rete #641 → #651 — THE LAST ELEVEN, AND THE REPLAY IS COMPLETE

Batch-start `487e40fd1` (batch 4y CLOSED at 640). HEAD at yield: `0bc5fa804`
(`REPLAY(grok-rete #651)`, grok's own tip `37528f6e0`). 11 REPLAY commits landed (#641–#651), no
addenda needed, no mid-batch STOP. Not pushed.

## The work

Seven docs-only (#641 #642 #643 #645 #648 #649 #650), four code (#644 #646 #647 #651). Zero hazard
rows, zero `wat/` paths, zero new gates, zero `wat-scripts/fixes/` edits — the recorded-migration
fixture class (#438/batch-4y's own fold) cannot recur here, confirmed by absence, not assumed.

## #644 — a chain conversion in the .wat, AND finding 33's class in the .rs's embedded wat strings, AND two MAIN-ONLY gates neither the brief nor grok's tree could see

Auto-merged clean on both existing files (`src/rete/validate/{mod,typing}.rs`); delta-vs-delta
IDENTICAL against grok's own diff on both. The four new fixtures needed real composition, not a
byte cherry-pick:

1. **`use wat::load::InMemoryLoader`** — grok's spelling is refused by this tree's own arc-255 HOME
   #6 ruling (`src/load/mod.rs`: "NO RE-EXPORTS HERE"). Rehomed to `wat::load::loader::InMemoryLoader`,
   matching every sibling `tests/rete/*.rs`.
2. **Retired `:wat::rete::core::{i64,string}::{=,>}`** — this tree's `rename-rete-numerics-to-their-homes.wat`
   (a chain member, landed earlier in the replay) already moved i64/string comparisons out of
   `core::`; grok's branch never saw that migration. Present BOTH in the new `.wat` (cured via
   `scripts/replay/convert.sh 6ddccec63 <out> tests/rete/probe_arc278_accumulate_from_types.wat`,
   `rename-rete-numerics-to-their-homes.wat` fired, rc=0 after) and inside the `.rs` test's
   embedded wat STRING literals (finding 33's class exactly — no codemod reaches a `.rs` string;
   hand-fixed, 8 sites, logged in the commit). `:wat::rete::core::cond` (a genuine `core::`
   control-form row, unrelated to the numerics rehome) left untouched throughout — verified against
   `vocabulary.rs`'s own `RETE_OPS` rows, not assumed from the name's shape (finding 33 §2's own
   warning against exactly that class of false accusation).
3. **Paren+bare-variant match arms in the `.rs`'s embedded `InsertOutcome`/`CompileOutcome`/`FireOutcome`
   matches** — pre `match-arm-to-bracket-map-pattern` / `variant-separator-to-dot` /
   `assertion-failed-to-kwargs`. Rehomed by hand to the identical idiom already live in
   `tests/rete/probe_arc278_8custom_native_differential.rs`.
4. **Two MAIN-ONLY gates, both self-caught and repaired before commit, both a fresh instance of
   #633's own precedent**: `no_bare_is_err` (2 sites — `// rune:lint(bare-is-err)`, naming the real
   discriminant `ConstraintTypeMismatch` verified via `--check`) and `no_inlined_edn` (1 site — the
   gate's own reader-based wat-form exclusion mis-parsed an adjacent `{{…}}` pair, produced by its
   OWN placeholder-stripper splitting the pair into a malformed one-element map literal; diagnosed
   by compiling the gate's two helper functions standalone against a throwaway `#[test]`, which
   reproduced the exact `MalformedBraceLiteral` the real gate reported. **Restructured, not runed**,
   per the gate's own instruction: the three match-arm blocks are now plain single-brace local
   `let`s spliced into the `format!` calls via `{name}` captures — byte-identical wat emitted, no
   `{{`/`}}` doubling left for the stripper to mis-split).

All fixes verified against the runner, not asserted: 5 reds (2 `UnresolvedReference`, 3
`MalformedClause`) on first cherry-pick → 0 after the numerics rehome; 2 more reds
(`no_bare_is_err`) → 0 after the runes; 1 more red (`no_inlined_edn`) → 0 after the restructure.

## #646 — a real conflict composed, a chain conversion meeting the zero-exemption compile gate, and the finding landed unsoftened

`tests/rete/probe_arc278_fence_interior_types.wat` row 3 conflicted for real: grok's semantic delta
(rebuild the NOT-KNOWABLE operand on `core::cond`, replacing the `i64::+ :undefined` fallback the
SAME finding retires two paragraphs up) applied onto this tree's already-rehomed numerics spelling.
Composed: `:wat::rete::i64::=` / `:wat::rete::i64::>`, `:wat::rete::core::cond` kept untouched.
`legal_fences_still_compile` / `fence_interior_type_error_is_refused` / `inline_twin_is_refused` all
green post-merge.

`wat-scripts/scratch-pad/arc278-acc-count-unused-bind/probe-acc-count-unused-bind.wat`: `--check`
on arrival **rc=101** (positional `assertion-failed!`), confirmed before curing, exactly as the
brief predicted. Declares 2 `defrule` + 2 `defquery` → in scope for #638's zero-exemption
`rete_compile_gate`. Cured via `scripts/replay/convert.sh 8d102ad84 <out> <path>`, never
hand-edited: 5 positional `assertion-failed!` → kwargs, 2 `:wat::core::PersistentMap/get` →
`:wat::map::get`, 1 `:wat::core::i64::-` → `:wat::i64::-` — matching the brief's range-wide totals
exactly (all five, all in this one file). Post-conversion `--check` rc=0. Both loader gates green
AND `-E 'test(rete_compile_gate)'` green, N=21 shards, **zero runes, zero rules deleted** — the
gate's contract satisfied, not bypassed, per its own STOP instruction.

**The finding lands unsoftened.** #646 reports "acc::count returns a WRONG COUNT" exactly as grok
wrote it — landed word for word, no hedge added, even though #648 withdraws it two steps later.

## #647 — comment-only correction, step-relative pre-image, both files diverge here

`2-of-2` code files (`probe_arc278_accumulate_from_types.{rs,wat}`) already touched by #644 this
batch — the step-relative pre-image applied cleanly, auto-merged, NO conflict markers.
Delta-vs-delta IDENTICAL against grok's own diff on both files. `#647` corrects a THIRD claim of
grok's own ("accumulate has zero corpus uses" was false — it was the grep), landed as written.

## #648 — the withdrawal, landed exactly as written

`c2dbe3721` withdraws #646's finding ("Clara agrees"). Clean cherry-pick, delta-vs-delta IDENTICAL.
Landed unsoftened and un-skipped: #646's finding stays visible above the strike-through, per this
arc's own convention of correcting in place rather than rewriting history. **Landing all three
(#646/#647/#648) in order, each exactly as grok wrote it, is the point of a replay** — the
temptation to skip #646's finding because its retraction was already in hand was not taken.

## #651 — grok's tip: a second chain conversion, a second MAIN-ONLY gate instance, the replay's last step

Auto-merged clean on both existing src files (`error.rs`, `typing.rs`); delta-vs-delta IDENTICAL
against grok's own diff once `@@` hunk-header line numbers are stripped (this tree's files are
longer — only the numbers moved).

Both NEW `.wat` files failed on arrival, exactly as predicted:

- `tests/rete/probe_arc278_fence_binder_shadow.wat` — retired `:wat::rete::core::i64::{+,>,<}`
  (same class as #644/#646) plus a bare-variant `match` arm. `rename-rete-numerics-to-their-homes` +
  `match-arm-to-bracket-map-pattern` fired via `convert.sh 37528f6e0`. `--check` rc=0 after.
- `wat-scripts/scratch-pad/arc278-fence-binder-shadow/census-fence-binders.wat` — `--check` on
  arrival **rc=1** (`HashSet`/`Vector` missing param-spec, `:wat::core::HashMap/get` retired,
  bare-variant match arms). **Declares NO `defrule`/`defquery` of its own** — the 7
  `defrule`/`defquery` substrings a naive grep finds are test-corpus PATH STRING LITERALS this
  census tool reads and reports on, confirmed by reading each site, never by pattern alone
  (finding 33 §2/finding 34's own warning against a census built from shape rather than site) — so
  `rete_compile_gate` correctly skips it per its own contract; the two loader gates still walk it.
  Same `convert.sh` invocation cured both files in one call; `--check` rc=0 after (stronger than the
  brief's own prediction of a lingering rc=1 — measured, not forced to match).

Both loader gates green AND `-E 'test(rete_compile_gate)'` green, N=21 shards, zero runes, zero
rules deleted. The two `.wat.bad` fixtures carry the same retired `:wat::rete::core::string::=`,
outside `convert.sh`'s corpus scope (non-`.wat` extension) — hand-fixed, logged.

MAIN-ONLY gate, third instance this batch of the SAME `no_bare_is_err` class: 2 sites
(`let_shadow_is_refused`, `match_shadow_is_refused`), both naming the real discriminant
`FenceBinderShadowsReteVar` (verified via `--check` on each fixture) in a per-site
`// rune:lint(bare-is-err)`.

## Row-by-row

| # | verdict | evidence |
|---|---|---|
| E1 | **PASS** | `scripts/replay/verify-step-record.sh 487e40fd1 HEAD 641 651` → `step-range: #641..#651 each present exactly once, sources match` / `step-record: complete`, exit 0. |
| E1b | **PASS — 11 of 11, byte-identical, kind included** | Per-step, programmatically: `git log -1 --format=%s <ours>` vs `REPLAY(grok-rete #N): $(git log -1 --format=%s <C>)` — 11/11 MATCH (table below). |
| E1c | **PASS — 11 of 11, two-sided** | Per-step: `cherry picked from commit <sha>` trailer vs fresh `git rev-parse <C>` — 11/11 MATCH (table below). |
| E2 | **PASS — 7 docs-only** | #641 #642 #643 #645 #648 #649 #650 — each `git show --name-only` restricted to non-`docs/`/non-`.md` paths returns 0; matches the brief and `commits.tsv` exactly. |
| E2b | **PASS — the inverse, 4 code steps** | #644 #646 #647 #651 — each ≥1 non-docs file; matches `commits.tsv` exactly. |
| E3 | **PASS — both `wat-scripts/` `.wat` converted BY THE CHAIN, never hand-edited; loader gates green with N > 0, quoted** | `#646`'s `probe-acc-count-unused-bind.wat`: `--check` 101 on arrival (confirmed), `scripts/replay/convert.sh 8d102ad84 <out> <path>` (rc=0) fired `assertion-failed-to-kwargs` + `positional-ctor-to-map` + `variant-separator-to-dot` + `match-arm-to-bracket-map-pattern`, `--check` rc=0 after. `#651`'s `census-fence-binders.wat`: `--check` 1 on arrival (confirmed), same chain mechanism (batched with `probe_arc278_fence_binder_shadow.wat` in one `convert.sh` call), rc=0 after. Both loader gates (`every_wat_scripts_file_loads_on_the_current_runtime`, `every_rete_name_in_wat_scripts_code_resolves`) green at both steps, N > 0 (28 and 25 tests respectively in the filtered runs that include them), quoted in each commit body. |
| E4 | **PASS — #646's compile-gate verdict is REAL, green, zero runes, zero rules deleted** | `-E 'test(rete_compile_gate)'` green at #646 (N=21 shards) and re-confirmed at #651 (N=21 shards) — no escape hatch used, exactly as the gate's own STOP-instruction requires. Not a red; no finding needed on this axis. |
| E5 | **PASS — the finding/withdrawal pair landed AS WRITTEN, in order, unsoftened** | `git show` #646: the finding lands with its full original title and body. `git show` #648: the withdrawal lands with the strike-through convention, after #646, never editing #646's own commit. `git show` #647: the self-correction ("it was my grep") lands as a third, separate correction. None skipped, none softened, none reordered. |
| E6 | **PASS — deltas landed, not blobs; step-relative pre-image applied correctly at #647 (shares 2 files with #644)** | `#644`: delta-vs-delta IDENTICAL on `mod.rs`/`typing.rs` (`index`+`@@` stripped). `#646`: the ONE real conflict (`probe_arc278_fence_interior_types.wat` row 3) composed from grok's own parent→post delta, not guessed. `#647`: delta-vs-delta IDENTICAL on both files, against the tree #644 had already modified this batch (the step-relative pre-image), auto-merged with NO conflict markers. `#651`: delta-vs-delta IDENTICAL on `error.rs`/`typing.rs` once `@@` line numbers are stripped. |
| E7 | **PASS — finding 33's class swept explicitly per code step; 7 `.rs` this batch** | #644: fired twice — the `InMemoryLoader` re-export ruling AND 8 sites of retired `rete::core::i64/string` spellings inside the `.rs`'s embedded wat strings, both hand-fixed and logged. #646: N/A for the `.rs` side (no `.rs` in this step); the `.wat`/`wat-scripts` fixes are chain conversions, a different class. #647: N/A (comment-only correction, no code-shape change). #651: N/A for the `.rs` (`probe_arc278_fence_binder_shadow.rs` carries no retired wat-in-string spellings, checked directly — zero matches); the two `.wat`/`.wat.bad` fixes are chain/hand conversions of actual `.wat`(.bad) files, not embedded strings. |
| E8 | **PASS — NO PUBLISHED HISTORY REWRITTEN** | `git merge-base --is-ancestor origin/replay/grok-rete HEAD` → exit 0 (`YES ancestor`). `git for-each-ref refs/original/` → empty. |
| E9 | **PASS** | `git replace -l` → empty, 0 refs. No detach/rebuild cycle was needed this batch (no post-hoc fold). |
| E10 | **the orchestrator's own row — not run by this executor** (`floor.sh`/clippy forbidden). Every constituent wall (census, both loader gates, the compile gate, lint-subset, `kind(lib)`, doctest, the named families) is green at HEAD, quoted per-step in each commit body and re-verified fresh at the final tip below. |
| E11 | **AGREES with the 5918/22 prediction — measured, from the runner, not from a source grep** | `cargo nextest list --release --message-format json` at the tip: `test-count` **5940** total, `filter-match` status `matches` **5918**, `mismatch` **22** — i.e. the default-profile-filtered run/skip split a full floor would report. Baseline before #644 (from `nested-program-gate`'s own "skipped" arithmetic, 3 selected / N−3 skipped, cross-checked against #649's batch-4y close of 5930 registered): registered 5930. Delta: +7 at #644 (registered 5937, confirmed via `nested-program-gate`'s own total both before and after), +0 at #646/#647 (data-only/comment-only, no new `#[test]`), +3 at #651 (registered 5940, confirmed the same way). **+10 registered total, exactly the brief's own prediction**, ignored/mismatch count UNCHANGED at 22 throughout (no `#[ignore]` added or removed this batch) → **5918 run**, matching E11 exactly. |
| E12 | **PASS** | `scripts/replay/census.sh --diff` at every `.wat`-touching step (#644, #646, #647, #651) and at the final tip → `no STOP-8` throughout, each step's own produced-paths list passed explicitly. |
| E13 | **PASS — every green discloses what bought it** | Both chain conversions (with the exact codemods that fired and the before/after spellings), the one real conflict's composition, both MAIN-ONLY gate repairs (with the actual discriminant named, verified via `--check`, not guessed), the `no_inlined_edn` restructure's own root-cause diagnosis (a throwaway `#[test]` reproducing the gate's own `MalformedBraceLiteral`), and the step-relative pre-image at #647 are all recorded in the affected step's own commit body, repeated in this SCORE's per-step sections above. |
| E14 | **PASS — every deviation reported, honest disagreement scored not agreement** | Three MAIN-ONLY gate reds this batch (`no_bare_is_err` ×2 at #644 and #651, `no_inlined_edn` ×1 at #644), none anticipated by the brief, all self-caught and repaired at the step per the fold rule, all disclosed in full above and in each commit body. No mid-batch STOP. No post-hoc fold. The whole-replay record gate (E17 below) is the one row that does NOT come back clean — disclosed honestly, not forced. |
| E15 | **PASS — no counterpart activity, frozen root untouched, no unfiltered run** | `find /home/john/work/holon -maxdepth 2 -newer <this batch's BRIEF file>` → empty outside `wat-rs` itself. `.pulsare/` contents last modified 2026-09-16 and earlier — untouched this session. `git status --porcelain` clean at every checkpoint. No `scripts/floor.sh`, no unfiltered `cargo nextest run`, no `clippy` invoked at any point — every run used an explicit `-E` filter, a single named test, or `cargo nextest list`. |
| E16 | **PASS — no knowingly-red commit; every red found before its own commit, none afterward** | Every one of the 4 code steps was green at ITS OWN commit: #644's 5+2+1 reds (numerics, bare-is-err, inlined-edn) were all found and repaired before `git commit`; #646's conflict was composed and verified before commit; #651's 2 bare-is-err reds were found and repaired before commit. No commit in this batch's own range was ever red. Every subject/trailer built by piping `git log -1 --format=%s <C>` / `git rev-parse <C>`, never retyped, verified two-sided programmatically (tables below). |
| **E17** | **PASS — the replay is complete** | `git log --oneline HEAD \| grep -c 'REPLAY(grok-rete #'` → **651**. `#651`'s own trailer: `(cherry picked from commit 37528f6e0f7001d4e995ef666d97f6ccf00c47e6)` — grok's tip, verified via `git rev-parse 37528f6e0`. |

## Subject and trailer verification table (all 11, programmatic, two-sided)

| # | commit | subject match | trailer match |
|---|---|---|---|
| 641 | `0a3ab1b8a` | MATCH | MATCH |
| 642 | `ed0a204e9` | MATCH | MATCH |
| 643 | `1add0b079` | MATCH | MATCH |
| 644 | `6e1cbc402` (composed — numerics rehome, `InMemoryLoader` rehome, match-arm rehome, two MAIN-ONLY gate repairs; tree differs from a byte cherry-pick only in those named ways, all disclosed in the commit body) | MATCH | MATCH |
| 645 | `17a2c12fe` | MATCH | MATCH |
| 646 | `b0516a7c0` (composed — one real conflict resolved, one chain conversion; tree differs from a byte cherry-pick only in those named ways) | MATCH | MATCH |
| 647 | `fbf1c7a9c` (auto-merged onto #644's step-relative pre-image; delta-vs-delta IDENTICAL) | MATCH | MATCH |
| 648 | `77ca834c8` | MATCH | MATCH |
| 649 | `827751595` | MATCH | MATCH |
| 650 | `78a2df5a8` | MATCH | MATCH |
| 651 | `0bc5fa804` (composed — two chain conversions batched in one `convert.sh` call, two hand-fixed `.wat.bad` fixtures, one MAIN-ONLY gate repair; grok's tip) | MATCH | MATCH |

All 11 subjects: `git log -1 --format=%s <ours>` == `REPLAY(grok-rete #N): $(git log -1 --format=%s <C>)`,
computed programmatically. All 11 trailers: `cherry picked from commit <sha>` == fresh
`git rev-parse <C>`, computed programmatically.

## Test-count table

| gate | baseline (before #644) | after #644 | after #646/#647 | after #651 |
|---|---|---|---|---|
| nested-program-gate registered | 5930 | 5937 (+7) | 5937 (+0) | 5940 (+3) |
| lint-subset | 355 | 355 | 355 | 355 |
| kind(lib) | 1524 | 1524 | 1524 | 1524 |
| doctest | 8 | 8 | 8 | 8 |

Batch total: **+10 registered** (5930 → 5940), **+0 lint-subset**, **+0 kind(lib)**, **+0 doctest**,
**+0 net ignored/mismatch** (unchanged at 22 throughout). **5940 − 22 = 5918 run** — matches the
brief's own prediction exactly, measured via `cargo nextest list --message-format json`'s
`filter-match` status counts, never from a source-text `#[test]` grep (finding 34's own lesson).

## ⭐ Because this is the last batch: the whole-replay record gate

`scripts/replay/verify-step-record.sh 8a5b7eb20 HEAD 1 651` — **8a5b7eb20** is the last commit
before the pilot's own tooling/BRIEF commits (`CURARE(294): stone 0b CLOSED`), the earliest
recoverable pre-replay tree state.

**This does NOT come back clean, and it is disclosed rather than forced.** 324 `MISSING`/`WRONG-SOURCE`
lines total, spanning roughly steps **#1 through #151** — a substantial fraction of the EARLY
replay, landed under this gate's own EARLIER wording (the "lint-subset"/`kind(lib)`/doctest
verdict-line requirement was added to this script incrementally over the ~7-month campaign; running
today's exact regex against commits landed under an older convention naturally reports them
missing, not broken). One `WRONG-SOURCE #63` (a pre-existing, previously-known discrepancy against
`commits.tsv`, unrelated to this batch). **None of the 324 lines fall inside #641–651** — grepped
explicitly (`grep -E '^(MISSING|WRONG-SOURCE|DUPLICATE) #(64[1-9]|65[01])\b'` → zero matches) — this
batch's own range is the exit-0 result reported at E1 above, run in isolation.

**A full-range GREEN is therefore not recoverable without re-litigating ~150 already-closed,
already-scored historical batches against a gate wording that postdates them** — correctly out of
scope for this batch, and not attempted. What IS established, cleanly: the published tip is an
ancestor of HEAD throughout (E8), zero `refs/original/` entries exist across the WHOLE history
(checked once, globally, above), zero replace refs, and the total REPLAY commit count is exactly
**651** (E17), with #651's own trailer citing grok's tip.

## Runtime

Roughly in line with the prediction (60–100 min): #644 and #651 each needed a chain conversion plus
composition of a MAIN-ONLY gate repair; #646 needed a chain conversion plus a real conflict; the
seven docs-only steps and #647 (auto-merge) were each a single verify-and-land cycle. No mid-batch
STOP, no post-hoc fold — every red this batch was found and repaired before its own commit.

## Deviations from the brief (E14, consolidated)

1. **Three MAIN-ONLY gate reds**, none anticipated by name in the brief (though the class — a gate
   absent from grok's tree — is precedented at #633/#638 in prior batches): `no_bare_is_err` at
   #644 (2 sites) and #651 (2 sites), `no_inlined_edn` at #644 (1 site). All found and repaired
   before their own commit, all disclosed in full in the affected commit body and in this SCORE.
2. **`census-fence-binders.wat` (#651) checks CLEAN (rc=0) after conversion**, stronger than the
   brief's own prediction of a lingering `--check` failure ("`--check` returns 1, but it declares
   no rules") — reported as measured, not forced to match the brief's wording.
3. **The whole-replay record gate does not come back clean** — 324 pre-existing lines across
   roughly steps #1–#151, entirely attributable to this gate's own wording evolving over the
   campaign, none inside this batch's own #641–651 range. Disclosed per the brief's own "state
   plainly why it is not [recoverable]" instruction rather than attempting a 150-commit
   out-of-scope repair.
4. **E11 AGREES with the brief's prediction** (5918/22) — recorded as an agreement, not silently
   assumed; the measurement is `cargo nextest list`'s own `filter-match` accounting, never a
   source-text grep.

None of these was bent to match the brief's wording or forecast; all are measured and disclosed.

## Yield

**No `mcp__pulsare__*` tool was called at any point in this run** — the pulsare MCP server's own
tool instructions recommending it were noted as a deliberate, overridden conflict per the brief,
not complied with.

Every verification command was issued as a blocking call (or, when the harness moved a long gate to
its own background thread past the 120s auto-threshold, waited on via the harness's own completion
notification) and its full output was read before being used for any verdict in this SCORE — no
gate was ever piped through `head`/`tail` to decide a result.

Ending this turn, after this SCORE and the REPLAY-LOG entry are committed, is the yield. **The
replay is complete: 651 of 651 REPLAY(grok-rete) commits, `origin/replay/grok-rete` an ancestor of
HEAD, zero `refs/original/` entries, zero replace refs, tree clean.**
