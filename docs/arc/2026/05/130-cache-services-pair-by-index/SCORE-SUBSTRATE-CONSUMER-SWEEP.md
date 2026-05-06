# Arc 130 — Substrate Consumer Sweep — SCORE

**Sweep:** sonnet, agent `a0f8bfa98e9fc29e1`
**Wall clock:** ~100 seconds — well under the 20-min time-box
(used 8.3% of cap); UNDER the 5-10 min predicted band.
**Output verified:** orchestrator independently confirmed via
`git diff --stat` (2 files, 4 insertions / 4 deletions),
`grep -rn "Vector/len"` (zero remaining), and full workspace
`cargo test`.

**Verdict:** **MODE B CLEAN — RENAME SHIPPED, NEXT-LINK
SURFACED.** 7/8 hard rows pass; 3/3 soft rows pass. Row 4
("the previously-failing test now passes") did NOT hold —
this is the BRIEF's allowed Mode B outcome: rename worked
exactly as named, but verification surfaced a NEW failure mode
(arc 110's panic discipline, not a substrate-vocabulary bug).
Sonnet correctly stopped and surfaced rather than grinding.

The discipline held perfectly. The brief allowed Mode B as a
valid outcome ("If verification surfaces unexpected failures,
sonnet STOPS and surfaces the discrepancy. Do NOT modify
additional files to make tests pass."). Sonnet honored every
constraint.

## Hard scorecard (7/8 PASS — row 4 honest Mode B fail)

| # | Criterion | Result |
|---|---|---|
| 1 | File diff scope | ✅ EXACTLY 2 files modified per `git diff --stat`. NO test files. NO Rust source. NO `wat/core.wat` edits. |
| 2 | All 4 sites swept | ✅ All 4 explicit sites changed; verified via `grep -rn "Vector/len"` returning zero remaining matches in substrate. |
| 3 | Verbatim rename only | ✅ `git diff` shows each replaced line preserves all surrounding code, comments, indentation. Diff is 4 lines `-…/len…` / 4 lines `+…/length…`. |
| 4 | **Failing test now passes** | ❌ **MODE B FAIL** — `deftest_wat_lru_test_lru_raw_send_no_recv` still FAILS. The substrate-vocabulary error is GONE (the BRIEF's stated success criterion); the test's assertion still fails because the canary's design surfaces the cascade's NEXT link (arc 110's panic). NOT a regression — a new top-of-stack chain link revealed by closing this one. |
| 5 | No previously-passing test breaks | ✅ Workspace test count delta zero across all crates. The 8-passing/1-failing wat-lru profile preserved. No new failures elsewhere. |
| 6 | The 6 :should-panic tests preserve state | ✅ All 6 deadlock-class tests in `HologramCacheService.wat` remain `should panic … ok`. |
| 7 | No new substrate primitives or removals | ✅ Rename-only sweep. `register_builtin` set unchanged. |
| 8 | Honest report | ✅ Sonnet's report is exemplary — pre-flight crawl confirmed verbatim, edits enumerated, verification numbers precise, Mode B classification + cascade-implication analysis explicit, `:should-panic` preservation called out, working-tree status flagged. |

**Hard verdict:** 7/8 with row 4 being the BRIEF's allowed Mode
B outcome (not a discipline failure; sonnet did exactly what the
brief specified — stop at first red, surface, don't grind).

## Soft scorecard (3/3 PASS)

| # | Criterion | Result |
|---|---|---|
| 9 | Total LOC delta | ✅ 4 lines net (one per site). No scope creep. |
| 10 | clippy clean | ✅ wat-source-only edit; no Rust delta; clippy unchanged. |
| 11 | Stop-at-first-red discipline | ✅ Sonnet identified the new failure as a NEW chain link rather than a regression OR the original substrate gap. Did not modify any additional files to make the test pass. The brief's "do NOT modify additional files" + "no grinding" constraints fully honored. |

## What Mode B revealed — the next cascade link

Pre-sweep failure: `unknown function: :wat::core::Vector/len`
(substrate vocabulary error)

Post-sweep failure: `CacheService/handle: reply-tx disconnected
— client died mid-request?` (arc 110's "silent disconnect →
panic loud" discipline firing)

The test's design:
1. Spawn cache service with capacity 16, parallelism 1
2. Pop one handle = `(req-tx, reply-rx)`
3. Finish the pool (signal driver to drain + exit)
4. **Raw send Request::Get on req-tx**
5. **DROP THE HANDLE without recv'ing the reply** (handle goes
   out of scope at inner let* exit)
6. Match driver's join-result; assert error message is `""` (the
   diagnostic canary pattern — assertion ALWAYS fails to surface
   the actual error message)

Pre-sweep: driver hit `Vector/len` unknown before reaching the
reply step → died mid-Get-handler → assertion surfaced that.

Post-sweep: driver successfully processes Get (zero probes →
empty results vec), tries to send `Reply::GetResult` on the
reply-tx → reply-rx is dropped (handle dropped pre-recv) → arc
110's discipline at `CacheService.wat:226` panics LOUD with the
"reply-tx disconnected" message → assertion surfaces THAT.

## What this means for the BRIEF's mutual-agreement protocol

**The chain held — and surfaced more than the brief enumerated.**

User → Orchestrator: "obvious next move is the consumer sweep"
→ Orchestrator → Sonnet: this brief, with `Vector/len` →
`Vector/length` as the rename, the named test as the canary
→ Sonnet → Reality: rename shipped, named test's vocabulary
error gone, NEW chain link surfaced

The mutual-agreement chain holds:
- The substrate consumer sweep was the right move ✅
- The Vector/length name was correct ✅
- All 4 sites were correctly identified ✅
- Mode B was a valid outcome the brief allowed ✅

The brief's Mode B allowance protected the discipline from
turning into "grind to make the named test pass." Sonnet's
~100s ship + clean Mode B classification is the exact pattern
the brief specified.

## What's NEW (cascade analysis)

Pre-arc-110, the test design (raw-send-no-recv) might have been
viable as a probe. Post-arc-110, raw-send-no-recv ALWAYS triggers
the panic-loud discipline by construction.

Three honest paths emerge for Layer 1 (the test):

1. **Update Layer 1's assertion** — assert the panic equals arc
   110's specific message. Layer 1 becomes a documented test
   of arc 110's contract. Test passes; layer ships as
   "raw-send-no-recv triggers arc 110's panic loudly, as
   designed."

2. **Redesign Layer 1** — change to `raw-send-AND-recv`
   (originally Layer 4 in the BRIEF-SLICE-1-RELAND plan). Recv
   the reply before drop; driver shuts down cleanly; assertion
   passes naturally.

3. **Keep Layer 1 as eternal canary** — the assertion-against-
   empty pattern stays; Layer 1 perpetually fails to surface
   whatever next-link top-of-stack error exists. This is the
   substrate-as-teacher pattern in action, but it leaves the
   workspace with a permanent FAILED test which is honestly
   awkward.

Each option has different implications for the slice 1 RELAND.
**Decision belongs to the user, not this SCORE.**

## What does NOT need new work

- The substrate's `Vector/len` → `Vector/length` rename is
  COMPLETE across both wat-lru and wat-holon-lru substrate
  files. This is foundation work; permanent.
- Sonnet's report stands as the historical record per "what is
  inscribed is inscribed" — no need to amend.
- The arc 130 slice 1 RELAND's existing test file (98 LOC,
  Layer 0-1) stays as it is. Layer 0 still passes; Layer 1
  still fails (now with arc 110's message instead of Vector/len);
  the file IS the historical record of the partial reland.

## Calibration record

- **Predicted Mode A (~85%)**: ACTUAL Mode B (~10% predicted).
  The rename worked — sonnet's substrate diagnosis was perfect —
  but the brief's prediction that "the failing test's name names
  the broken behavior" held only for the substrate-vocabulary
  layer. The canary surfaced a deeper interaction (arc 110's
  contract vs the test's design) the brief did not enumerate.
- **Predicted runtime (5-10 min)**: ACTUAL ~100 seconds.
  Mechanical sweep; the simplest substantive sweep in the cascade.
- **Time-box (20 min)**: NOT triggered (used 8.3%).
- **Honest deltas (predicted 0)**: ACTUAL 2 — the cascade's NEXT
  link surfaced (arc 110 vs raw-send-no-recv); the test file's
  diagnostic comment is more stale than the brief named.
  Both surfaced cleanly without grinding.

## What this slice unlocks (forward progress only)

- Substrate's `Vector/length` consistency across wat-lru and
  wat-holon-lru — a foundation correction that survives forward
- The arc 130 slice 1 RELAND's substrate side is now CORRECT for
  the happy-path (Layer 4+ scenarios that recv before drop)
- The cascade's `:wat::core::Vector/len` chain link closes —
  arc 143's named "next link" is now closed
- The next-step design decision for Layer 1 (the test design vs
  arc 110's contract interaction) is named explicitly here for
  user direction

## Pivot signal analysis

NO discipline pivot. Sonnet executed the brief perfectly. The
"Mode B with surprise" outcome is valuable data, not a failure.

The cascade just demonstrated its compounding nature again — the
arc 130 pause taught arcs 110, 117, 126, 131, 132, 135, 138, 139,
140, 142, 143, 144, 146, 148, 150 (15+ arcs). Now the substrate
fix in this sweep surfaces ARC 110's interaction with the test's
design — arc 110 itself was a child of the cascade. The
substrate-as-teacher pattern is recursive.

The next decision is the user's: how should Layer 1 reckon with
arc 110? Three paths sketched above; user picks.

**The brief's protocol was honored. The chain held. The next
chain link is named.**
