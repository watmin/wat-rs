# Arc 214 — Slice 2 forward-correction — Mini-TCP at depth 1 (drop bounded(N); pair() at capacity 1)

## Mission

Forward-correct Slice 2's thread-tier comms surface. The substrate currently ships TWO factories (`pair()` returning unbounded, `bounded(n)` returning bounded-n) and a usage pattern that has ZERO honest callers of the bounded(N) factory and `pair()` ships with the WRONG default (unbounded — sender never blocks).

**The trading-lab convergence (verbatim user 2026-05-19):**

> *"before wat-rs existed - we were in the holon-lab-trading and build mailboxes and whatever their opposite is - we found that only ever needed a depth of 1 for everything - this forces us into a lock step that has an organic nature to it... its breathes based on system load - its dynamic but predictable.. when we had the option to send N things and then block we have massive perf hits - i think the thread comms need to be like process comms - you may only send one thing and must immediately read back - either an ack or some data - this is the only supported pattern - mini-tcp everywhere - forcing us to be locked eliminates entire categories of problems"*

This stone collapses the thread-tier comms surface to ONE factory (`pair()`) returning a capacity-1 buffered-rendezvous channel — matching the universal mini-TCP discipline that ZERO-MUTEX.md / arc 119 ack-tx / defservice / Counter actor / process tier (kernel-bounded pipes) ALL operate by. Stone closes BEFORE Slice 4 begins per "we cannot build upon a shaky foundation."

## Substrate-truth verified pre-spawn

- **`src/comms/thread.rs:276-283`** — `pub fn pair<T>()` body: `crossbeam_channel::unbounded()` + `(Sender { inner: tx }, Receiver { inner: rx })`. Flip to `crossbeam_channel::bounded(1)`; same wrapper construction.
- **`src/comms/thread.rs:285-292`** — `pub fn bounded<T>(capacity: usize)` body: `crossbeam_channel::bounded(capacity)` + wrapping. Delete entirely.
- **Grep evidence (2026-05-19):** `comms::thread::bounded` has ZERO downstream callers; `comms::thread::pair` has ZERO downstream callers outside its own probe. Pure surface clean-up; no migration cascade.
- **`tests/comms/thread.rs:14-15`** — `use wat::comms::{ReceiverIndex, RecvError, SelectOutcome, TryRecvError};` + `use wat::comms::thread::{bounded, pair, Select};`. Drop `bounded` from second line.
- **`tests/comms/thread.rs:25-35`** — `probe_slice2_bounded_round_trip`: calls `bounded::<i64>(4)`, sends 2, asserts len==2. **Premise dies** when bounded(N) retires AND pair=bounded(1) cannot hold 2 values. Delete the entire test function.
- **`tests/comms/thread.rs:17-23`** — `probe_slice2_unbounded_round_trip`: calls `pair::<i64>()`, sends 1, recvs 1. Behavior identical at bounded(1); test passes unchanged. **Naming is a lie post-flip** — rename to `probe_slice2_pair_round_trip` (FM 14 surface-retirement-leftovers discipline).
- **`tests/comms/thread.rs:1-10`** — module-header doc: "Ten tests covering: round-trip (unbounded + bounded), sender-drop, ..." → "Nine tests covering: round-trip, sender-drop, ..." (drop the "unbounded + bounded" phrase).
- **Other 9 probe tests (audit-verified):** none does multi-send-without-recv. All compatible with bounded(1) without modification. Verified via line-by-line inspection of test bodies (sender-drop, try_recv empty/disconnected, clone-multi-producer with intervening recvs, single-send-then-select, registration-only no-send, single-send-then-close).

## Concrete deliverables

### 1. Update `src/comms/thread.rs` module-level doc

The module-level doc currently describes the thread tier's surface. Add a section near the top (after the cascade-contract paragraph, before the factories) that names the mini-TCP discipline as load-bearing:

```rust
//! ## Mini-TCP at depth 1 (THE only pattern)
//!
//! Every channel constructed by `pair()` has capacity 1. The substrate
//! enforces the mini-TCP discipline structurally: producers may send one
//! value, then MUST recv an ack (or the next value) before sending again.
//! `send` blocks when one value is queued; `recv` unblocks the sender.
//!
//! This matches the process tier's kernel-bounded pipes (which have
//! similar structural backpressure) and the discipline named in
//! `docs/ZERO-MUTEX.md` § "Mini-TCP via paired channels" (line 252+).
//! Every load-bearing pattern this substrate ships (arc 119 ack-tx,
//! defservice Request/Reply, Counter actor, dispatch loops) operates
//! at this depth.
//!
//! No `bounded(N)` factory exists. The trading-lab convergence
//! (pre-wat-rs origin) proved that N > 1 produces massive perf hits +
//! entire categories of problems; mini-TCP at depth 1 is dynamic +
//! predictable + organic under load.
```

### 2. Flip `pair<T>` factory

Replace lines 276-283 (`pub fn pair<T>` body + its doc comment) with:

```rust
/// Construct a mini-TCP thread-tier channel pair at depth 1.
///
/// Capacity is 1: one value can be queued; `send` blocks when full;
/// `recv` unblocks the sender. THE only factory; THE only supported
/// communication pattern (see module-level doc § "Mini-TCP at depth 1").
///
/// Both endpoints are cascade-aware (Receiver's recv wakes on substrate
/// shutdown).
pub fn pair<T: Send + 'static>() -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = crossbeam_channel::bounded(1);
    (Sender { inner: tx }, Receiver { inner: rx })
}
```

### 3. Delete `bounded<T>` factory entirely

Remove lines 285-292 in their entirety:

```rust
/// Create a bounded thread-tier channel pair with the given capacity.
/// Senders block on `send` when the channel is full, providing back-pressure.
/// Cascade-on-send for blocking sends is future arc work; cascade on recv
/// is already wired.
pub fn bounded<T: Send + 'static>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = crossbeam_channel::bounded(capacity);
    (Sender { inner: tx }, Receiver { inner: rx })
}
```

No trailing whitespace; collapse the blank line if needed to keep the file clean.

### 4. Update `tests/comms/thread.rs`

**4a — Module-header doc (lines 1-10):** drop "(unbounded + bounded)" phrase + the "unbounded" name reference:

```rust
//! Arc 214 Slice 2 smoke probe — verify thread tier round-trip + cascade.
//!
//! Nine tests covering: round-trip, sender-drop, try_recv (empty +
//! disconnected), Clone semantics (sender + receiver), Select firing +
//! index ordering, close multi-clone behavior.
//!
//! SHUTDOWN_RX is NOT initialized in these tests (bootstrap fallback path).
//! The cascade-aware recv falls back to bare crossbeam recv, which is correct
//! for the test environment — the contract is verified structurally (the
//! select! pattern is in the code) rather than by triggering shutdown.
```

**4b — Import line (line 15):** drop `bounded`:

```rust
use wat::comms::thread::{pair, Select};
```

**4c — Rename `probe_slice2_unbounded_round_trip` → `probe_slice2_pair_round_trip` (lines 17-23):** match the post-flip substrate-truth (FM 14 surface-retirement-internals discipline):

```rust
#[test]
fn probe_slice2_pair_round_trip() {
    // Verifies the most basic contract: a value sent via mini-TCP depth-1 pair
    // is a value received.
    let (tx, rx) = pair::<i64>();
    tx.send(42).expect("send must succeed on live channel");
    assert_eq!(rx.recv().expect("recv must return the sent value"), 42);
}
```

**4d — Delete `probe_slice2_bounded_round_trip` entirely (lines 25-35):** the test premise (multi-enqueue len tracking on bounded(N)) dies with the substrate change. The test function and its preceding/following blank lines retire together.

### 5. Update `docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md`

**5a — Rust-side types listing (lines 84-104):** drop the `bounded` line; rewrite `pair` comment:

```rust
pub fn pair<T>() -> (Sender<T>, Receiver<T>);  // capacity-1 mini-TCP (see DESIGN § "Mini-TCP at depth 1 — universal symmetry")
// process tier — IDENTICAL surface; T bound differs; io_uring underneath; pair() returns io::Result<...> (libc::pipe(2) can fail)
```

(Delete the `pub fn bounded<T>(n: usize)` line entirely.)

**5b — Rename + rewrite § "Universe-residency + bounded() asymmetry"** (currently lines 106-141): rename to "Universe-residency + Mini-TCP at depth 1 (universal symmetry)" and rewrite. The new section's structure:

- Keep the user-direction quote about universe-residency (lines 108-110)
- Keep the two-layer honesty table (lines 112-117)
- Update the three substrate-internal asymmetries list (lines 119-125): keep #1 (T bound) and #2 (pair() return type); REMOVE #3 (bounded() asymmetry) entirely
- Add a new "Mini-TCP at depth 1 (universal symmetry)" subsection inscribing the convergence:
  - Trading-lab pre-wat-rs origin
  - User verbatim quote about bounded(N) producing massive perf hits + entire categories of problems
  - Four-questions verdict against keeping bounded(N): FAILS YES YES YES YES
  - Four-questions verdict for pair() at bounded(1): YES YES YES YES
  - Cross-references (ZERO-MUTEX.md mini-TCP, arc 119, defservice, Counter, process tier kernel-bounded pipes)
- Cross-references at the end retain: project_universe_residency, project_autoscaling_correctness; ADD reference to this new forward-correction inscription (INTERSTITIAL § "2026-05-19 (Slice 2 forward-correction)").

**5c — Slice 2 description (lines 475-486):** add a one-line forward-correction note at the END of the slice description (do NOT edit the body; per `feedback_inscription_immutable` past descriptions stay):

```markdown
- Factories: `pair<T>()`, `bounded<T>(n)`

**FORWARD-CORRECTED 2026-05-19 (Slice 2 forward-correction stone):** `bounded()` factory retired; `pair()` returns capacity-1 mini-TCP. Trading-lab convergence + universal symmetry with process tier. See DESIGN § "Mini-TCP at depth 1 (universal symmetry)" + INTERSTITIAL § "2026-05-19 (Slice 2 forward-correction)".
```

**5d — Append a new forward-correction subsection at the END of the DESIGN.md file** (parallel structure to "Stone E forward-correction (2026-05-19) — TCO discipline + reflexive rebuild"):

```markdown
### Slice 2 forward-correction (2026-05-19) — Mini-TCP at depth 1 (universal symmetry)

Slice 4 prep surfaced that thread tier shipped TWO factories (`pair()` unbounded, `bounded(n)` opt-in) when the substrate's universal discipline is mini-TCP at depth 1. Inscribed forward per `feedback_inscription_immutable`.

**Four-questions verdict on `pub fn bounded<T>(n)`:**

- **Obvious?** NO — asymmetric with process tier (which has ONE factory, kernel-bounded pipes).
- **Simple?** NO — two factories; substrate-author choice carries semantic meaning; the meaning is "honor the discipline or violate it."
- **Honest?** NO — exposes a knob the substrate's own practice proved harmful (22 of 22 honest callers use `bounded(1)` only; the n parameter is vestigial).
- **Good UX?** NO — substrate-internal callers (brackets, services) could pick `bounded(64)` and break mini-TCP discipline; no structural guard.

**FAILS YES YES YES YES.** Factory retired.

**Four-questions verdict on `pub fn pair<T>()` returning bounded(1):**

- **Obvious?** YES — symmetric with process tier; ONE factory per tier.
- **Simple?** YES — N identical `pair()` call sites; no choice to make.
- **Honest?** YES — capacity-1 IS the mini-TCP discipline structurally enforced; senders cannot "send N then block" because send blocks at depth 1.
- **Good UX?** YES — substrate-author CANNOT pick wrong depth; lock-step by construction.

**YES YES YES YES.** pair() flipped from unbounded to bounded(1).

**Universal symmetry restored:**

| Tier | Factory | Discipline mechanism |
|---|---|---|
| Thread | `pair()` → bounded(1) | crossbeam capacity-1 buffer; send blocks at depth 1 |
| Process | `pair()` → io_uring + pipe | kernel `PIPE_BUF` bounded; send blocks when pipe full |
| Remote (future) | `pair()` → TBD | TBD; same mini-TCP discipline |

Both tiers now expose ONE factory whose underlying transport enforces the same mini-TCP semantics structurally. Programs running in any universe (thread / process / future remote) see identical send/recv semantics. The universe-residency principle + Convergence #13 (autoscaling of correctness) + this mini-TCP-at-depth-1 discipline compose into: substrate manages all transport details invisibly; user picks hosting env at the outside; one supported communication pattern; entire categories of problems eliminated structurally.

**Trading-lab origin (pre-wat-rs lineage):** the mini-TCP-at-depth-1 discipline predates wat-rs. The user built mailboxes (and their opposite) in `holon-lab-trading` and converged on depth-1 universally: it forces lock-step that breathes organically with system load — dynamic but predictable. N > 1 produced massive perf hits + entire categories of problems (cf. arc 119 ack-tx correction, defservice's Request/Reply convention, the Counter actor's match-arm three-line discipline). Thread tier now MATCHES what process tier always had (kernel-bounded pipes) + what every load-bearing pattern in this substrate ships.

**Cross-references:**
- `docs/ZERO-MUTEX.md` § "Mini-TCP via paired channels" (line 252-415) — the substrate-wide articulation
- arc 119 — `HologramCacheService Put ack-tx` (the mini-TCP discipline correction that named the pattern in-substrate)
- INTERSTITIAL § "2026-05-16 (deeper) — Control channels: Shutdown/Final convention" — Counter actor at mini-TCP depth 1
- INTERSTITIAL § "2026-05-19 (Slice 2 forward-correction)" — this stone's narrative
- INTERSTITIAL § "2026-05-19 — Universe-residency principle" — the principle this discipline operationalizes at the channel-factory layer
- INTERSTITIAL § "2026-05-19 — Convergence #13" — autoscaling of correctness (sibling discipline at the resource-management layer)
- `feedback_options_are_tangle` — the pattern bounded(N) was; rejected here
- `feedback_refuse_easy_solutions` — "keep bounded(N) for flexibility" would have been easy + dishonest
- `feedback_attack_foundation_cracks` + `feedback_any_defect_catastrophic` — the discipline that drove the immediate pivot when the asymmetry surfaced
- `feedback_inscription_immutable` — Slice 2's original description preserved; forward-correction inscribed here as new work
```

### 6. Inscribe in `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md`

Append a new entry AFTER the existing compaction breadcrumb (after line 6417). Section header:

```markdown
## 2026-05-19 (post-compaction, Slice 2 forward-correction) — Mini-TCP at depth 1: the trading-lab origin returns
```

Structure:
- User direction (verbatim): the full "before wat-rs existed - we were in the holon-lab-trading..." quote
- The recognition: Slice 4 prep surfaced the asymmetry as L1 lie; the trading-lab pattern returned to claim the foundation
- Grep evidence: 22 of 22 honest substrate callers use `bounded(1)`; `comms::thread::bounded` has ZERO downstream callers; `pair()` ships with the wrong default
- Four-questions verdict (as in DESIGN § 5d above; named here as the verdict that drove the stone)
- The tally of "shockingly stable" foundation pivots in arc 214: (1) Stone E tunable rejection, (2) bounded() process-tier rejection (in symmetric-honest framing), (3) THIS — bounded(N) thread-tier rejection. Three pivots, all in service of the same discipline: substrate manages what substrate manages; user/substrate-author can't pick wrong because the wrong choice doesn't exist.
- The universal symmetry recognition: thread tier now matches process tier (which always had this discipline via kernel-bounded pipes). Universe-residency principle made operational at the comms-factory layer.
- The four-questions are MANDATED reminder reinforced (this stone ran them inline, surfaced verdict, moved forward — per feedback_four_questions_inline)
- Cross-references: project_universe_residency, project_autoscaling_correctness, feedback_options_are_tangle, feedback_refuse_easy_solutions, feedback_attack_foundation_cracks, arc 119 (the in-substrate naming of mini-TCP), ZERO-MUTEX § "Mini-TCP via paired channels"

Closing line in the substrate-dreams voice: *"the substrate dreams the depth. The substrate dreams 1. So do we."*

## Verification

After all 6 deliverables ship:

1. **`cargo build --release`** — must complete clean (zero downstream callers of `bounded`; pair() flip is internal mechanism change).
2. **`cargo test --release --test thread -p wat`** — exactly 9 tests; all pass.
3. **`cargo test --release --workspace --no-fail-fast`** — workspace baseline preserved; only `probe_slice2_bounded_round_trip` retired (gone) + `probe_slice2_unbounded_round_trip` → `probe_slice2_pair_round_trip` (renamed).
4. **Manual grep verification:** `grep -rn "comms::thread::bounded" --include="*.rs"` returns ZERO matches across the workspace (factory truly gone; no orphan callers).

## STOP triggers

- **If `cargo build` fails after `bounded` deletion** → a downstream caller exists that pre-spawn grep missed. STOP; report the call site; do not work around (the grep premise needs investigation).
- **If any of the other 9 probe tests fail unexpectedly** → indicates a hidden multi-send-without-recv pattern. STOP; report the failing test + the relevant lines; do not bump capacity or work around.
- **If any wat-level test or example references `:wat::kernel::bounded` or `:wat::comms::thread::bounded` keyword form** → wat-layer caller exists. STOP and report (Slice 2 was substrate-internal only; wat layer shouldn't have surfaced this yet but verify).
- **If DESIGN.md's § "Universe-residency + bounded() asymmetry" rewrite produces inconsistencies with other DESIGN sections** (e.g., line 96-97 Rust-side types listing references bounded) → STOP and report what other section needs updating; do not silently flow-through inconsistent state.
- **If INTERSTITIAL's existing structure (heading levels, voice, cross-reference patterns) suggests a different framing than the proposed entry** → STOP and surface alternative.

## Discipline anchors

This stone operates under:

- **`feedback_attack_foundation_cracks`** — the crack surfaced during Slice 4 prep; fix immediately, not "future arc"
- **`feedback_any_defect_catastrophic`** — substrate trust is binary; pivot when defect surfaces; don't build Slice 4 on shaky foundation
- **`feedback_no_known_defect_left_unfixed`** — known defect (asymmetric bounded; wrong pair() default); no excuse to defer
- **`feedback_options_are_tangle`** — bounded(N) IS the option-tangle pattern; collapse to one canonical mechanism
- **`feedback_simple_is_uniform_composition`** — pair() at bounded(1) for ALL callers IS simple; uniform composition
- **`feedback_refuse_easy_solutions`** — "keep bounded() for flexibility" would be easy + dishonest
- **`feedback_substrate_owns_not_callers_match`** — substrate enforces mini-TCP depth structurally; callers can't pick wrong
- **`feedback_inscription_immutable`** — Slice 2's original description stays; forward-correction is a NEW commit
- **`feedback_four_questions_inline`** — the four-questions ran inline per the discipline; verdict surfaced; forward motion
- **`feedback_four_questions_yes_no`** — atomic YES/NO per candidate; no comparison-shopping
- **`feedback_iterative_complexity`** — ONE coherent concern per stone; the N uniform changes are mechanical composition

Per the kernel impeccability protocol (INTERSTITIAL § "2026-05-19 — Kernel impeccability via ward pass (NEW PROTOCOL)"), this stone gets a **9-ward parallel pass after sonnet ships** (intueri + struere + purgare + solvere + temperare + conferre + mora + perspicere + nesciens). The cargo build + test gate is necessary but not sufficient.

## Out of scope (do NOT touch)

- **Process tier (`src/comms/process.rs`)** — already symmetric (one `pair()` factory; kernel-bounded pipes). NO changes.
- **Foundation primitives (`src/comms/mod.rs`)** — traits + error types unchanged.
- **Other workspace code referencing `crossbeam_channel::bounded`** — those call crossbeam directly (not `comms::thread::bounded`); out of scope.
- **The `bounded(4)` test fixtures in `tests/wat_arc170_typed_channel_pipes.rs`** — those test `typed_channel`'s arc 213 χ work using crossbeam directly; NOT calls to `comms::thread::bounded`; untouched.
- **wat-level surface** — Slice 2 was substrate-internal; no wat-layer surface to change.
- **Slice 4 work** — DOES NOT START until this stone closes + ward-passes + commits.

## Time budget

- Code changes (3 files; ~50 lines net delta): 5-10 min
- DESIGN.md updates (3 sections + 1 new subsection): 15-25 min
- INTERSTITIAL new entry: 10-15 min
- cargo build + test verification: 2-5 min
- SCORE doc: 10-15 min
- **Total: 45-70 min Mode A**

If sonnet runs > 90 min: STOP via wakeup; report progress; orchestrator decides continue vs Mode B-time-violation.
