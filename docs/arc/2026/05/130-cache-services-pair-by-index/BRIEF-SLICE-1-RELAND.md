# Arc 130 Slice 1 — RELAND Brief — Test File from Empty Slate

**Drafted 2026-05-02 (evening)** after the original sweep
shipped a substrate reshape + a complectens-violating test
file. Diagnosis: the test file inherited a 4-helper /
5-deftest layered structure shaped pre-arc-130 — the helpers
themselves bake in the OLD substrate's channel-pair semantics.
Patching couldn't repair this. The reland WIPES the test file
and BUILDS it bottom-up from empty, one stepping stone at a
time, per the /complectens spell.

The substrate reshape that the prior sweep shipped STAYS on
disk. The reland tests THIS substrate. If a stepping stone
reveals a substrate bug, the failing stone's NAME names the
broken behavior. We diagnose + fix at that point. Do NOT
modify the substrate during this reland — substrate fixes
are a separate arc.

## Goal

Build `crates/wat-lru/wat-tests/lru/CacheService.wat` from
EMPTY slate using the complectens discipline. Each layer
proves the SMALLEST POSSIBLE next thing; verify with
`cargo test` after EACH layer added; STOP at first red.

## Working directory

`/home/watmin/work/holon/wat-rs/`

## Required pre-reads (in order)

1. **`.claude/skills/complectens/SKILL.md`** — THE SPELL.
   Read it cover-to-cover. The four questions on test-file
   structure (Obvious / Simple / Honest / Good UX), the
   one-file-per-source rule, the per-layer-deftest rule,
   the no-forward-references rule, the deftest-body 3-7 lines
   rule. Every line of this brief operates within the spell.

2. **`crates/wat-holon-lru/wat-tests/proofs/arc-119/step-A-spawn-shutdown.wat`**
   — the SMALLEST POSSIBLE wat-rs proof of "service spawn +
   shutdown lifecycle works." Note its structure: ONE deftest,
   inner-let* nesting, single time-limit annotation. This is
   Layer 0's archetype.

3. **`crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat`**
   — Layer 0 + ONE single put via the helper verb. Adds ONE
   thing. Currently uses pre-arc-130 helper-verb shape (with
   ack-pair) — the SHAPE is the lesson, not the verb signature.

4. **`docs/arc/2026/04/107-option-result-expect/INSCRIPTION.md`**
   — the wat-rs arc that DECLARED the stepping-stones
   discipline. Read the "What this arc closes" section + the
   proof_004 cascade analysis. The discipline's WHY.

5. **`crates/wat-lru/wat/lru/CacheService.wat`** — the
   substrate file your tests will exercise. Read its
   typealiases, helper-verb signatures, spawn factory body,
   driver loop. Note the post-arc-130 shape: helper verbs take
   `Handle<K,V> = (ReqTx, ReplyRx)`; substrate owns the reply
   channels; spawn returns `(HandlePool<Handle>, Thread)`.

6. **`docs/arc/2026/05/130-cache-services-pair-by-index/DESIGN.md`**
   — the substrate redesign reference. Slice 1 section. The
   typealiases + helper-verb signatures that landed.

## What to do

### Step 1 — wipe the existing test file

`crates/wat-lru/wat-tests/lru/CacheService.wat` is poison
(complectens-violating; assumes pre-arc-130 channel-pair shape
in its helpers). DELETE all its content. Replace with an empty
file (or just a header comment block).

### Step 2 — build the layers, ONE AT A TIME

Each layer is ONE NAMED HELPER + ONE DEFTEST. Body of each
helper: single outer let*, 3-7 bindings. Body of each deftest:
3-7 lines composing the helper. Each helper composes ONLY
from helpers ABOVE it (no forward references; no late
dependencies).

**Suggested layer granularity** (you may refine, but don't
SKIP — each layer adds ONE NEW THING):

- **Layer 0** — `:test::lru-spawn-and-drop`. Just call
  `:wat::lru::spawn 1 1 null-reporter null-cadence`. Bind the
  Tuple result. Project pool, driver. Drop pool. Join driver.
  Asserts: lifecycle works WITHOUT pop, finish, send, or recv.
  THIS IS THE NARROWEST POSSIBLE PROOF.

- **Layer 1** — `:test::lru-spawn-pop-drop`. Layer 0 + pop
  ONE handle from pool. Drop handle (let it fall out of
  scope). Drop pool. Join driver. Asserts: pop works; handle
  drop is benign for driver shutdown.

- **Layer 2** — `:test::lru-spawn-pop-finish-drop`. Layer 1
  + call `HandlePool::finish pool` between pop and pool drop.
  Asserts: finish on a partially-popped pool works.

- **Layer 3** — `:test::lru-raw-send-no-recv`. Layer 2 + on
  the popped handle, raw `(:wat::kernel::send (first handle)
  (Request::Get []))`. Don't recv anything. Drop. Join.
  Asserts: send on req-tx succeeds; driver consumes the Request
  and continues; shutdown still clean.

- **Layer 4** — `:test::lru-raw-send-raw-recv`. Layer 3 + raw
  `(:wat::kernel::recv (second handle))` after the send. Match
  the result; assert it's `Ok(Some(Reply::GetResult []))` —
  the empty-probes reply. Drop. Join. Asserts: driver replies
  on the matching reply-tx; recv on reply-rx receives it. THIS
  IS THE LAYER MOST LIKELY TO SURFACE THE SUSPECTED SUBSTRATE
  BUG (driver dropping ReplyTx prematurely).

- **Layer 5** — `:test::lru-helper-get-empty`. Switch from
  raw send/recv to the helper verb: `(:wat::lru::get handle
  [])`. Asserts: helper verb returns empty `Vec<Option<V>>`.

- **Layer 6** — `:test::lru-helper-put-one`. Helper verb
  `(:wat::lru::put handle [(k, v)])` returns unit. Asserts:
  put on a single entry succeeds.

- **Layer 7** — `:test::lru-helper-put-then-get`. Compose
  Layers 5+6: put one entry, get the same key, assert results
  vec contains `Some(Some(v))`.

(If a layer reveals the substrate is broken, you'll stop at
that layer. The remaining layers are the goal — what we'd
have if substrate were intact.)

### Step 3 — workflow per layer

For EACH layer:

1. Add the helper define + deftest (in that order; helper
   defined BEFORE deftest; helper composes only from
   helpers above).
2. Run: `cargo test --release -p wat-lru`.
3. Read the output. The new deftest must report `... ok`.
4. If the deftest passes: continue to next layer.
5. If the deftest FAILS: STOP. Do not write Layer N+1.
   Report:
   - Which layer failed (by name).
   - The exact failure mode (panic message + line number).
   - Your hypothesis on which substrate behavior is broken.
   - Your judgment on whether the failure is an instance of
     the suspected substrate bug (driver removing slots
     prematurely on `Ok(None)` from select) OR a different
     gap.

The first failing layer is the diagnostic. Don't try to
diagnose further or fix the substrate — surface the failure
+ stop.

### Step 4 — if all 8 layers pass

Report total test count, all-green confirmation, file LOC,
and how each layer reads top-down.

## Constraints

- **Test-only edits.** ONE file changes:
  `crates/wat-lru/wat-tests/lru/CacheService.wat`. Any other
  file modification = STOP and report.

- **Substrate is OFF LIMITS.** If the substrate is broken,
  the first failing layer surfaces that. Don't modify
  `crates/wat-lru/wat/lru/CacheService.wat`. Don't modify any
  other crate. Don't modify any Rust source.

- **Workspace stays GREEN as far as possible.** Other
  workspace tests should not break (unless your new test
  file syntactically defeats the build — in which case fix the
  syntax). The 4 currently-failing LRU tests will be GONE
  after Step 1's wipe; the workspace cleanup is a side effect.

- **Each helper: ONE outer let\*.** Per `feedback_simple_forms_per_func.md`
  + the spell's "deftest body 3-7 lines" rule. If a layer
  needs nested complexity, decompose into a sub-helper.

- **Each helper has its OWN deftest.** No helpers without
  proofs (Level 2 mumble).

- **Top-down: no forward references.** Helper at line N must
  not reference helpers defined at line N+M. The dependency
  graph runs upward only.

- **Run `cargo test --release -p wat-lru` after EACH layer.**
  This is the load-bearing discipline. Don't add three layers
  then run; don't trust your reading of the code without
  running. The `cargo test` after each layer IS the proof
  that the stepping stone is sound.

- **STOP at first red.** Don't grind. Don't iterate on a
  failure. Report it.

- **No commits, no pushes.**

## What success looks like

There are TWO success modes:

**Mode A — all 8 layers pass:** the substrate is intact under
the post-arc-130 reshape. The test file is a worked
demonstration of the complectens discipline applied to a
service substrate. Slice 1 ships clean.

**Mode B — Layer N fails:** the substrate has a bug that
surfaces at Layer N. Layer N's name names the broken
behavior. The test file as it stands (Layers 0..N-1 +
Layer N's failing deftest) is a worked diagnostic. We open a
follow-on arc to fix the substrate at Layer N.

EITHER mode is a successful run of THIS reland brief. The
discipline is what we're proving — not that the substrate is
already perfect.

## Reporting back

Target ~250 words:

1. **Layer-by-layer pass/fail roll-up:**
   ```
   Layer 0 (lru-spawn-and-drop):           PASS
   Layer 1 (lru-spawn-pop-drop):           PASS
   Layer 2 (lru-spawn-pop-finish-drop):    PASS
   Layer 3 (lru-raw-send-no-recv):         PASS
   Layer 4 (lru-raw-send-raw-recv):        FAIL — recv returned Ok(None) at <line>
   Layer 5..7:                              NOT WRITTEN (stopped at first red)
   ```

2. **Final cargo test totals** for `cargo test --release -p
   wat-lru`: passed / failed / ignored.

3. **The failing layer's mechanics** (if any):
   - Helper body (the let* bindings).
   - Deftest body.
   - Failure message verbatim from cargo test.
   - Your hypothesis on the broken substrate behavior.

4. **The four questions verdict** on the test file YOU wrote:
   - Obvious? Does each deftest's failure trace name the
     broken layer?
   - Simple? Are all deftest bodies 3-7 lines?
   - Honest? Do helper names match bodies?
   - Good UX? Top-down readable, no forward refs?

5. **File LOC** total + LOC budget per layer (rough average
   should be 15-30 LOC per layer including helper + deftest).

## What this brief is testing (meta)

Two things at once:

1. **Does the post-arc-130 substrate work?** The first
   stepping stone that fails (if any) tells us where it breaks.
   If all layers pass, substrate is sound.

2. **Does the complectens discipline propagate to fresh
   agents via the spell + worked examples?** Sonnet has no
   conversation memory of the prior sweep's struggle. The
   discipline must propagate via `.claude/skills/complectens/SKILL.md`
   + `step-A-spawn-shutdown.wat` + `step-B-single-put.wat` +
   arc 107's INSCRIPTION + this brief alone.

If sonnet ships a clean stepping-stone test file (whether all
layers pass OR stops at the right red), the discipline holds.
If sonnet ships a complectens-violating file (missing
per-layer deftests; monolithic; forward refs), the spell needs
sharpening.

## Sequencing — what to do, in order

1. Read `.claude/skills/complectens/SKILL.md` cover to cover.
2. Read step-A-spawn-shutdown.wat + step-B-single-put.wat.
3. Read arc 107's INSCRIPTION (the wat-rs discipline
   declaration).
4. Read the substrate file `crates/wat-lru/wat/lru/CacheService.wat`
   — note the current shape (typealiases, helper-verb
   signatures, spawn factory, driver loop).
5. Read `docs/arc/2026/05/130-cache-services-pair-by-index/DESIGN.md`
   for the slice 1 substrate-redesign reference.
6. Run `cargo test --release -p wat-lru 2>&1 | tail -20` to see
   the current failure baseline (4 failing tests post-prior-sweep).
7. WIPE `crates/wat-lru/wat-tests/lru/CacheService.wat`.
   Replace with an empty file (or just a header).
8. **For each layer 0..7:**
   a. Add the helper + deftest.
   b. Run `cargo test --release -p wat-lru`.
   c. If the new deftest reports `... ok`: continue to next
      layer.
   d. If it FAILS: STOP. Report.
9. If all 8 layers pass: report success per "Reporting back."
10. Then DO NOT commit. Working tree stays modified for the
    orchestrator to score.
