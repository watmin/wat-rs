# Arc 119 — Pattern B Cache Services: Batch-Oriented Protocol Fix — INSCRIPTION

## The closing

Arc 119 closes 2026-05-06. Started 2026-05-01 as a "small Put
needs an ack-tx" finding during arc 109 K.holon-lru audit work;
through three rounds of gaze + user direction the scope grew
into a substrate-wide convention promotion. Six substrate steps
shipped 2026-05-01 across both crates. Step 7 (consumer sweep
with discipline correction) was paused on Pattern B's mid-flight
substrate-as-teacher diagnostics; the cascade — including arc
130 — closed the gaps that step 7 needed; **arc 130's HolonLRU
+ wat-lru test rebuilds (2026-05-06) absorbed step 7's
consumer-sweep work as a side effect of the rebuilds**.

**The substrate-wide convention promoted**: every wat-rs-shipped
service exposes only batch-oriented `get` / `put` interfaces.
Console is the single exception. After arc 119 + arc 130 shipped
together, the convention is substrate-uniform — every non-Console
service takes batches.

## What this arc shipped

### Pattern B convention promoted to substrate-wide

Codified in `docs/CONVENTIONS.md` § "Batch convention" (commit
`cc5a78f` 2026-05-01):

> Every wat-rs-shipped service exposes only batch-oriented
> `get` / `put` interfaces. Console is the single exception.

Five substrate services exist; three (Telemetry, Telemetry-Sqlite,
Console) already obeyed or were exempt (Console is the lone
fire-and-forget); arc 119 brought the remaining two (LRU +
HolonLRU) in line. Cross-referenced in `docs/ZERO-MUTEX.md`
§ "Batch granularity = lock granularity" + `docs/SERVICE-PROGRAMS.md`
§ "Reply shapes" (the two-reply-shape table that supersedes the
fire-and-forget Push row).

### Symmetric batch protocol — both services identical

Pre-arc-119, the two cache services had divergent shapes:
- `:wat::lru::*` — tagged-tuple Body<K,V> with unified Reply
- `:wat::holon::lru::*` — enum Request with NO Put reply path
  (fire-and-forget — violated ZERO-MUTEX § Mini-TCP)

Post-arc-119, both adopt the symmetric batch shape:
- **Get**: `(get probes)` → `Vec<Option<V>>` (Pattern B data-bearing)
- **Put**: `(put entries)` → `unit` (Pattern A unit-ack release)

Singletons become batches-of-one. Get is data-bearing; Put is
unit-ack release. The "lock granularity = batch granularity"
geometry from ZERO-MUTEX falls out cleanly.

### Substrate work shipped (steps 1-6, 2026-05-01)

For both `:wat::lru::*` and `:wat::holon::lru::*`:

- **Retired**: `Body<K,V>` typealias (LRU); fire-and-forget Put
  (HolonLRU)
- **Minted**: `Entry<K,V>` = `(K, V)` (LRU; HolonLRU's
  Entry is concrete `(HolonAST, HolonAST)`)
- **Reshaped Request**: enum `{Get(Vec<K>, GetReplyTx) | Put(Vec<Entry>, PutAckTx)}`
- **Reshaped GetReplyTx**: body widened from
  `Sender<Option<V>>` to `Sender<Vec<Option<V>>>`
- **Minted PutAck family**: `PutAckTx`, `PutAckRx`, `PutAckChannel`
  (`Sender<unit>` shape)
- **Driver loops**: per-request batch dispatch (Get iterates
  probes; Put iterates entries; both send their batch result
  on the corresponding back-edge)
- **Helper verbs**: `(get probes)` / `(put entries)` — bare
  verbs; argument type carries the plurality (substrate's
  unmarked-verb convention per `:wat::core::map`, `:wat::core::if`)

### Variant-scoped naming (Pattern A + Pattern B coexist)

Per the gaze pass-1 verdict: variant-scoped (`Get*Reply*` for
data-bearing back-edge + `Put*Ack*` for unit-ack release) wins
over unified Reply. INVENTORY § K's `Ack*` (unit) vs `Reply*`
(data) distinction is load-bearing; unified Reply enum forces
half its variants to be payload-less and lie about the family
they belong to.

Per pass-2: `PutAck*` over `PutReply*` — substrate's
load-bearing rule names families by what the back-edge CARRIES
(unit vs data), not which verb owns it. Put's back-edge is
`Sender<unit>` → joins the Ack family alongside
Telemetry / Console.

Per pass-3: bare verbs (`get` / `put`); enum-based Request
shape; `Entry<K,V>` for batch element name (cache-domain word
unambiguous across crates).

### The vocare spell (born here)

Arc 119 surfaced the load-bearing principle that became its own
spell:

> All code is measurable from the caller's perspective. That's
> the interface to confirm.

A consumer of HologramCacheService calls
`(HologramCacheService/get handle probes)`. They do not
hand-build Request enum constructors. They do not call raw
`:wat::kernel::send`. The wat-tests in a consumer crate's
`wat-tests/` directory should look like consumer call sites.
Anything else is at the wrong vantage and teaches the next
reader the wrong shape.

This became the **vocare ward** (commit `6083070` 2026-05-01):
the first ward that defends caller-perspective. Codified in
`docs/CONVENTIONS.md` § "Caller-perspective verification" +
`.claude/skills/vocare/SKILL.md`. Wire-protocol pedagogy lives
in `wat-rs/wat-tests/service-template.wat` (where the caller IS
the implementer); consumer-crate tests do not mirror that style.

The vocare-audit performed during arc 119 (commit `864d334`):
full wat-rs codebase audit confirmed consumer-vantage discipline
is solid across the rest of the workspace; HologramCacheService
was the lone violator pre-arc-130-rebuild.

### Step 7 — consumer sweep absorbed by arc 130's rebuilds

Step 7 was originally framed as "wrap singletons in batch-of-one
+ thread the new ack-tx." Per the user's caller-perspective
direction, that mechanical rewrite would land tests still at
the wrong vantage. The discipline correction: rewrite consumer
tests to call helper verbs only, not raw kernel send/recv.

That rewrite is exactly what **arc 130's slice 1 + slice 2 test
rebuilds** delivered (2026-05-06):

| Step 7's named test | Where it lives now (post-arc-130) |
|---|---|
| `test-cache-service-put-then-get-round-trip` | wat-lru L3 `helper-put-then-get` |
| `test-step3-put-only` | HolonLRU L2 `helper-put-one` |
| `test-step4-put-get-roundtrip` | HolonLRU L3 `helper-put-then-get` |
| `test-step5-multi-client-via-constructor` | HolonLRU L6 `multi-client` |
| `test-step6-lru-eviction-via-service` | HolonLRU L5 `eviction` |

Each scenario survived the complectens rebuild as a layered
helper. The consumer-vantage discipline arc 119 step 7 named
("tests should look like consumer call sites; helper verbs only;
no raw kernel send/recv") IS what arc 130's rebuilds shipped —
explicit BRIEF row 8 + SCORE verification.

Arc 130's substrate reshape made this absorption necessary AND
clean: arc 130 changed the helper-verb signatures (3 channel ends
→ single Handle) so the test files HAD to be rebuilt anyway; the
discipline correction landed in the same rebuild.

The cascade closed both arcs' consumer-sweep work in one rebuild.

## What this arc closes — counted

- **HolonLRU's fire-and-forget Put discipline violation** —
  client now blocks on Put ack per ZERO-MUTEX § Mini-TCP
- **2 service surfaces unified** — LRU and HolonLRU now have
  identical protocol shapes (modulo concrete-typing in HolonLRU)
- **`:wat::lru::Body<K,V>` typealias retired** (LRU)
- **`:wat::holon::lru::*` enum Request reshaped** to symmetric
  batch (Get + Put both have back-edges)
- **`PutAckTx/Rx/Channel` family minted** in both crates
- **`Entry<K,V>` typealias minted** (LRU) + concrete Entry (HolonLRU)
- **GetReplyTx body widened** from `Sender<Option<V>>` to
  `Sender<Vec<Option<V>>>` (batch return)
- **Substrate-wide batch convention promoted** to
  `docs/CONVENTIONS.md` (every non-Console service takes batches)
- **vocare ward born** — first caller-perspective ward; the
  discipline `CONVENTIONS.md` § "Caller-perspective verification"
  codified
- **Step 7 (consumer sweep) absorbed** by arc 130's slice 1 +
  slice 2 test rebuilds

## What this arc unlocks

- **Arc 109 K.holon-lru slice** (#195) — was waiting on arc 119's
  surface stabilization. Now tractable. (Note: arc 130's substrate
  reshape SUPERSEDED part of K.holon-lru's planned `GetReplyPair
  → GetReplyChannel` rename — those typealiases were retired
  entirely. K.holon-lru's remaining work is the grouping-noun
  flatten only: `HologramCacheService::*` → `:wat::holon::lru::*`.)
- **Arc 109 v1 closure trajectory** clearer — major chain link
  closes
- **Substrate-wide batch convention** holds — future services
  (none currently planned, but if any spawn) inherit the pattern
- **vocare ward** propagates caller-perspective discipline
  into future test-writing across the workspace

## What this arc does NOT close

- **Arc 109 K.holon-lru slice** (#195) — separate arc 109 slice;
  not arc 119's deferral
- **Arc 109 v1 INSCRIPTION** — major future ship; not arc 119's
  scope
- **Lab consumer sweeps** — `holon-lab-trading/` is downstream,
  separate workspace, separate arc; not in arc 119's scope per
  step 7 BRIEF's explicit "Scope is wat-rs only" boundary

These are downstream arc trajectories, NOT deferrals OF arc 119.
Arc 119's substrate work + the convention promotion + step 7's
consumer-sweep absorption-via-arc-130 are all DONE per the
DESIGN's "What's wrong today" + "The fix" + "Execution checklist"
sections.

## The honest record — three gaze passes + user direction

Per `feedback_inscription_immutable.md` and
`project_failure_engineering.md`: the scope-growth path is
preserved for future readers.

The arc started narrow. The scope grew through honest gaze
ward criticism + user direction:

1. **Original gaze** (during K.holon-lru first-pass): called
   HologramCacheService's "fire-and-forget" Put acceptable.
2. **Orchestrator codified the lie** as intentional comment.
3. **User caught the lie** pointing at `docs/ZERO-MUTEX.md`:
   *"the client cannot continue until the server confirms...
   both directions are lock step"*
4. **Second-pass gaze** locked variant-scoped `PutAckTx` family
   over a unified Reply enum (Pattern A unit-ack as its own
   family).
5. **User noticed the deeper issue**: LRU and HolonLRU should
   be basically identical surfaces; today they diverge.
6. **User clarified the desired protocol**: batch-oriented;
   Get returns Vec<Option<V>>; Put returns unit.
7. **Third-pass gaze** locked the names against the batch
   protocol covering both services.

Plus the step 7 realization (2026-05-01): mechanical rewrite of
singletons-to-batch-of-1 would land tests at the wrong vantage;
the discipline correction is what mattered. That insight
became the vocare ward.

## Cross-references

- **DESIGN**: `docs/arc/2026/04/119-holon-lru-put-ack/DESIGN.md`
  — locked plan + 3 gaze verdicts
- **VOCARE-AUDIT**: `docs/arc/2026/04/119-holon-lru-put-ack/VOCARE-AUDIT.md`
  — full wat-rs codebase caller-perspective audit
- **Per-step BRIEFs**: BRIEF-LRU-RESHAPE, BRIEF-HOLON-LRU-RESHAPE,
  BRIEF-CONSUMER-SWEEP, BRIEF-STEPPING-STONES
- **Substrate-wide convention**: `docs/CONVENTIONS.md` § "Batch
  convention" + § "Caller-perspective verification"
- **Ward**: `.claude/skills/vocare/SKILL.md` — the ward born here
- **Cascade**: arc 130 (cache services pair-by-index — absorbed
  step 7's consumer-sweep work)
- **Arc 109 INVENTORY § K**: `docs/arc/2026/04/109-kill-std/INVENTORY.md`
  rows for LRU + HolonLRU updated to mark arc 119 ✓
- **Discipline**: `docs/COMPACTION-AMNESIA-RECOVERY.md` (this
  arc's execution checklist was the compaction-amnesia anchor
  Step 7's pause survived through);
  `feedback_inscription_immutable.md` (gaze-pass scope-growth
  preserved as honest record);
  `project_failure_engineering.md` (the wrong-layer-test
  diagnostic became the vocare ward)

## Calibration record

- **Arc started**: 2026-05-01 as small finding
- **Substrate work shipped**: 2026-05-01 (steps 1-6) — same day
- **Step 7 paused**: 2026-05-01 evening (mid-flight diagnostic
  pause; cascade-as-teacher)
- **Cascade arcs that shipped during pause**: many (arc 130 +
  the same 12 cascade arcs that arc 130 spawned, since arc 119
  is arc 130's parent dependency)
- **Step 7 absorbed**: 2026-05-06 by arc 130's test rebuilds
- **Closure**: 2026-05-06 (this paperwork)
- **Total arc duration**: ~6 days
- **Honest deltas surfaced**: 3 gaze rounds + step 7 wrong-vantage
  realization + step 7 absorption-by-arc-130

## Status

**Arc 119 closes here.** Both cache services obey the substrate-
wide batch convention. The fire-and-forget Put violation is gone.
The two services have identical protocol shapes (modulo
concrete-typing in HolonLRU). Caller-perspective verification
discipline is codified + the vocare ward defends it.

**Arc 109 v1 closure trajectory clearer.** Arc 119's chain link
closes; K.holon-lru slice (#195) becomes tractable.

The arc started narrow + grew through honest gaze ward criticism
+ user direction. The scope-growth path stays inscribed as
historical record per "what is inscribed is inscribed." The
methodology IS the proof.

---

*the small finding became a substrate-wide convention. the
mechanical rewrite became a discipline correction. the wrong-
vantage realization became a ward. the cascade closed everything
that mattered. forward progress only.*

**PERSEVERARE.**
