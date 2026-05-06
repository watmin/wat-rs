# Arc 130 — Test File Rebuild BRIEF (post-arc-110-aware)

**Drafted 2026-05-06** after the substrate consumer sweep
(`Vector/len` → `Vector/length`) shipped Mode B clean and surfaced
the cascade's NEXT chain link: arc 110's silent-disconnect→panic-
loud discipline interacts with the existing test's
raw-send-no-recv pattern. The user's direction:

> *"i think we just need to rewrite the tests from the ground us
> [up] (using the pattern on how to write good tests) to have the
> cache service pass handles around correctly"*

The original BRIEF-SLICE-1-RELAND.md (sibling brief) was drafted
2026-05-02 BEFORE arc 110's discipline was fully internalized; it
proposed a Layer 3 of "raw-send-no-recv" which post-arc-110 IS
incoherent (arc 110 panics by construction when reply-rx is
dropped pre-recv). That brief is preserved as historical record;
this brief supersedes it for the test-file rebuild scope.

## Goal

Wipe `crates/wat-lru/wat-tests/lru/CacheService.wat` and rebuild
bottom-up per the `/complectens` spell, with layers that respect
the post-arc-130 substrate's pair-by-index discipline AND arc
110's silent-disconnect→panic-loud discipline. Each layer adds
ONE new thing; each layer carries its own deftest; failure trace
names the broken layer by helper-function name.

## The substrate shape (verified pre-brief)

The post-arc-130 wat-lru cache service substrate at
`crates/wat-lru/wat/lru/CacheService.wat`:

- **Handle<K,V> = (ReqTx, ReplyRx)** — client side; line 74. Pop
  one handle from HandlePool; the two ends are halves of TWO
  DIFFERENT pre-allocated channels (request + reply).
- **DriverPair<K,V> = (ReqRx, ReplyTx)** — driver side; line 78.
  Driver holds N pairs by index; select fires at index i; same
  index locates the ReplyTx for delivery.
- **Reply<V>** enum unifies GetResult + PutAck; line 57. Both
  verbs share ONE reply channel per slot.
- **Request<K,V>** enum: Get(Vec<K>) | Put(Vec<Entry<K,V>>)
- **spawn** signature: `(:wat::lru::spawn cap par reporter cadence)
  -> Spawn<K,V> = (HandlePool<Handle<K,V>>, Thread<unit,unit>)`
- **Helper verbs**: `:wat::lru::get` and `:wat::lru::put` take
  a Handle + payload; they internally do send-AND-recv per arc
  110's contract; helper signatures live in CacheService.wat.

The substrate is in SOUND state for happy-path scenarios. The
prior consumer sweep (`Vector/len` → `Vector/length`) closed
the substrate-vocabulary gap. Send-AND-recv flows work.

## Arc 110's contract (the design constraint)

**Silent disconnect → loud panic.** When the driver tries to send
a reply on a reply-tx whose reply-rx has been dropped, the driver
PANICS with a meaningful message rather than silently discarding
the reply. This is intentional substrate discipline.

**Implication for tests:**
- Layers that send WITHOUT recv'ing → driver will panic when
  trying to deliver. This is NOT a substrate bug; it is the
  contract.
- Tests must EITHER recv before drop (happy path) OR explicitly
  probe arc 110's panic (assertion matches the specific panic
  message).

The original BRIEF's "raw-send-no-recv" Layer 3 cannot be a
cleanly-passing layer post-arc-110. This brief drops that layer
shape; layers that send must also recv.

## Pre-reads (mandatory in this order before writing)

1. **`.claude/skills/complectens/SKILL.md`** — THE SPELL. Cover
   to cover. The four questions on test-file structure
   (Obvious / Simple / Honest / Good UX), the per-layer-deftest
   rule, the no-forward-references rule, the deftest-body 3-7
   lines rule, the one-outer-let* rule. Every line of this brief
   operates within the spell.

2. **`crates/wat-lru/wat/lru/CacheService.wat`** — the substrate
   you are testing. Read all of it: typealiases (lines 50-110),
   helper verbs `:wat::lru::get` + `:wat::lru::put`, spawn
   factory, driver loop. Note the post-arc-130 shape: Handle =
   (ReqTx, ReplyRx); DriverPair = (ReqRx, ReplyTx); Reply<V>
   enum.

3. **`crates/wat-holon-lru/wat-tests/proofs/arc-119/step-A-spawn-shutdown.wat`**
   — the smallest possible service-spawn-shutdown proof. Layer 0's
   archetype.

4. **`docs/arc/2026/04/110-kernel-comm-expect/INSCRIPTION.md`**
   — arc 110's discipline declaration. Read the "no silent
   disconnects" framing.

5. **`docs/arc/2026/04/107-option-result-expect/INSCRIPTION.md`**
   — the original wat-rs declaration of stepping-stones discipline.

6. **`docs/arc/2026/05/130-cache-services-pair-by-index/REALIZATIONS.md`**
   — the test-file-composition discipline named during arc 130's
   killed sweep. Read in full; this brief operates within its
   rules.

7. **`docs/arc/2026/05/130-cache-services-pair-by-index/SCORE-SUBSTRATE-CONSUMER-SWEEP.md`**
   — the immediate predecessor sweep (today). Read the "What Mode
   B revealed" section to understand what NOT to do at Layer 1.

## What to do

### Step 1 — wipe the existing test file

`crates/wat-lru/wat-tests/lru/CacheService.wat` currently has 98
LOC: a header + Layer 0 (spawn-and-drop) + Layer 1
(raw-send-no-recv with arc 110 collision). DELETE all content.
Replace with an empty file (or just a header comment block).

### Step 2 — build the layers, ONE AT A TIME

Each layer is ONE NAMED HELPER + ONE DEFTEST. Helper body: single
outer let*, 3-7 bindings. Deftest body: 3-7 lines composing the
helper. Each helper composes ONLY from helpers ABOVE it (no
forward references; no late dependencies).

**Suggested layer plan** (refine if substrate reality demands;
do not skip):

- **Layer 0** — `:test::lru-spawn-and-drop`. Call
  `(:wat::lru::spawn 16 1 :wat::lru::null-reporter
  (:wat::lru::null-metrics-cadence))`. Bind the Spawn<K,V> result.
  Project pool, driver. Pop one handle (required before finish per
  HandlePool contract). Finish pool. Let handle drop at inner
  scope exit → driver sees disconnect → outer Thread/join-result
  unblocks. Asserts: lifecycle works WITHOUT any request being sent.

- **Layer 1** — `:test::lru-helper-get-empty`. Layer 0 + call
  `(:wat::lru::get handle (:wat::core::Vector :wat::core::String))`
  on the popped handle BEFORE finishing the pool. The helper does
  send-AND-recv internally (per arc 110's contract). Asserts:
  helper returns empty `Vec<Option<V>>` for empty probes; driver
  shuts down cleanly after pool finish + handle drop.

- **Layer 2** — `:test::lru-helper-put-one`. Layer 1 + a single
  `(:wat::lru::put handle <entries-of-one>)` call. Asserts:
  helper returns unit; put on a fresh cache succeeds.

- **Layer 3** — `:test::lru-helper-put-then-get`. Compose Layers
  1+2: put one entry, get the same key, assert the results vec
  contains the put-value wrapped `(Some Some(v))`. THIS IS THE
  HAPPY PATH ROUND TRIP — proves the substrate works end-to-end
  for a single key.

- **Layer 4** — `:test::lru-helper-get-many-keys`. Same shape as
  Layer 3 but with multiple keys; asserts the result vec aligns
  with the probe vec by index.

- **Layer 5** — `:test::lru-helper-put-batch`. Multiple
  entries in one put; asserts unit; subsequent get on those keys
  returns Some.

- **Layer 6** (optional, only if time) — eviction probe. Spawn
  with capacity 2; put 3 distinct keys; first key should evict.

If the substrate reveals a happy-path bug at any layer, STOP at
that layer. The failing layer's name names the bug. Surface it
+ stop. Do not modify the substrate.

### Step 3 — workflow per layer

For EACH layer:

1. Add the helper define + deftest (helper defined BEFORE
   deftest; helper composes only from helpers above).
2. Run `cargo test --release -p wat-lru`.
3. Read the output. The new deftest must report `... ok`.
4. If the deftest passes: continue to next layer.
5. If the deftest FAILS: STOP. Do not write Layer N+1.
   Report:
   - Which layer failed (by helper name).
   - The exact failure mode (panic message + line number).
   - Your hypothesis on which substrate behavior is broken.
   - Whether the failure surfaces an arc 110 / arc 117 / arc 126
     interaction that suggests a test-design refinement, or
     whether it surfaces a genuine substrate bug.

The first failing layer is the diagnostic. Don't try to diagnose
further or fix the substrate — surface the failure + stop.

## Constraints

- **Test-only edits.** ONE file changes:
  `crates/wat-lru/wat-tests/lru/CacheService.wat`. Any other
  file modification = STOP and report.

- **Substrate is OFF LIMITS.** If the substrate is broken,
  the first failing layer surfaces that. Don't modify
  `crates/wat-lru/wat/lru/CacheService.wat`. Don't modify any
  other crate. Don't modify any Rust source.

- **Each helper: ONE outer let\*.** Per
  `feedback_simple_forms_per_func.md` + the spell's
  "deftest body 3-7 lines" rule. If a layer needs nested
  complexity, decompose into a sub-helper.

- **Each helper has its OWN deftest.** No helpers without
  proofs (Level 2 mumble).

- **Top-down: no forward references.** Helper at line N must
  not reference helpers defined at line N+M. The dependency
  graph runs upward only.

- **Run `cargo test --release -p wat-lru` after EACH layer.**
  Don't add three layers then run; don't trust your reading of
  the code without running.

- **STOP at first red.** Don't grind. Don't iterate on a
  failure. Report it.

- **Use the helper verbs (`:wat::lru::get`, `:wat::lru::put`)
  as the primary interface.** Raw `:wat::kernel::send` /
  `:wat::kernel::recv` is the substrate plumbing; the helpers
  are the contract surface; tests should exercise the contract.

- **No commits, no pushes.** Working tree stays modified for the
  orchestrator to score.

- **No deferral language in the file's header comments.** Per
  FM 11: if you're tempted to write "future arc," "deferred to,"
  "TODO," etc. — STOP. Either ship it or affirmatively scope-bound it.

## Out of scope

- HolonLRU (`crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`)
  — that's slice 2's test rebuild; same shape, different crate;
  separate brief.
- Retiring the 6 `:should-panic("channel-pair-deadlock")`
  annotations on HologramCacheService tests — slice 3 closure
  work; these tests still hold structural value while slice 2
  is in flight.
- Continuing the original BRIEF-SLICE-1-RELAND's Layer 3
  "raw-send-no-recv" pattern — incompatible with arc 110;
  intentionally dropped from this brief.
- Slice 1 INSCRIPTION — orchestrator paperwork after this sweep
  scores.

## Reporting

Target ~250 words:

1. **Layer-by-layer pass/fail roll-up:**
   ```
   Layer 0 (lru-spawn-and-drop):           PASS
   Layer 1 (lru-helper-get-empty):         PASS
   Layer 2 (lru-helper-put-one):           PASS
   Layer 3 (lru-helper-put-then-get):      PASS
   Layer 4 (lru-helper-get-many-keys):     FAIL — <reason>
   Layer 5..6:                             NOT WRITTEN
   ```

2. **Final cargo test totals** for `cargo test --release -p
   wat-lru`: passed / failed / ignored.

3. **The failing layer's mechanics** (if any):
   - Helper body (the let* bindings).
   - Deftest body.
   - Failure message verbatim from cargo test.
   - Your hypothesis.

4. **The four questions verdict** on the test file YOU wrote:
   - Obvious? Does each deftest's failure trace name the
     broken layer?
   - Simple? Are all deftest bodies 3-7 lines?
   - Honest? Do helper names match bodies?
   - Good UX? Top-down readable, no forward refs?

5. **File LOC** total + LOC budget per layer.

## What success looks like

**Mode A — all 4+ shipped layers pass:** the substrate works
end-to-end for the happy path. The test file is a worked
demonstration of the complectens discipline applied to the
post-arc-130 cache service. Slice 1's test side ships clean.

**Mode B — Layer N fails:** the substrate has a happy-path bug
at Layer N. Layer N's name names the broken behavior. The test
file as it stands (Layers 0..N-1 + Layer N's failing deftest)
is a worked diagnostic. Open a follow-on arc / decision for the
substrate fix.

**Mode C — sonnet violates complectens:** monolithic body, missing
per-layer deftests, forward refs. Hard scorecard rows fail.
Reland with sharper brief.

EITHER Mode A OR Mode B is a successful run. The discipline is
what we're proving.

## Why this brief matters for the cooperation

User direction 2026-05-06 (verbatim):

> *"if i can teach you and you can teach sonnet - then i have
> full clarity of my ask"*

The substrate consumer sweep this morning shipped Mode B clean —
chain held end-to-end, surfaced arc 110's interaction. The user
named the next move: rewrite the tests from the ground up using
the complectens pattern, with handles passed correctly.

This brief is the orchestrator's restatement. If sonnet ships
Mode A clean, the chain holds:
- User → Orchestrator (the user teaching the rebuild direction)
- Orchestrator → Sonnet (this brief)
- Sonnet → Reality (the test file ships with N happy-path layers)

If Mode B: the substrate has a bug at Layer N. The diagnostic is
clean; we open a follow-on. The cascade continues.

If Mode C: the brief was unclear; sharpen + reland.

Each link's success calibrates the cooperation. None alone
suffices; all three together prove mutual agreement.
