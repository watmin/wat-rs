# Arc 214 Slice 3 — Stone E-2 — EXPECTATIONS

## Independent prediction

- **Runtime band:** 25-40 min Mode A. Larger than E-1 because E-2 bundles two converging concerns (persistent ring + Receiver method extraction) and refactors Select::select substantially. The skeleton is exhaustive but borrow scoping has more moving parts.
- **LOC changed:** ~80-130 net delta (2 new methods + 1 new struct field + reflexive rebuild block + Select::select body refactor + ~5 call-site changes in Receiver::recv/try_recv).
- **New files:** 1 (SCORE doc only).
- **Surprises expected:** MEDIUM. Borrow scoping for `Select.ring` borrow vs `Receiver.ring` borrow is the highest risk; reflexive rebuild block needs to release the borrow before submission setup; submission setup borrows again; Read step borrows the Receiver's ring (different RefCell) so releases the Select borrow first.

## Honest-delta watch

### Risk 1 — RefCell borrow scoping (Select.ring vs Receiver.ring)

**What:** `Select.ring` (RefCell<Option<(IoUring, u32)>>) and `Receiver.ring` (RefCell<IoUring>) are different RefCells but Rust's borrow checker treats `&mut self` borrows of Select carefully. The Select::select body must scope the Select-ring borrow so it releases BEFORE calling `rx.read_into_acc()` (which borrows the Receiver's ring). If sonnet holds the Select-ring borrow across the Read step, the test may compile but the borrow's lifetime is misleading; if sonnet drops the borrow inside the wrong scope, the borrow checker rejects.

**Mitigation:** BRIEF skeleton uses explicit `{ ... }` scope blocks to bound the Select-ring borrow.

### Risk 2 — Reflexive rebuild logic

**What:** At loop top, compute `needed_capacity`; compare to stored capacity in the Option; rebuild if mismatch or if None. The BRIEF skeleton uses pattern matching on `ring_slot.as_ref()` to handle both cases. If sonnet uses `if let Some(...)` without the None case, lazy init never fires. If sonnet uses `==` instead of `!=` for the mismatch check, rebuild semantics flip.

**Mitigation:** BRIEF skeleton spells out the match arms explicitly. Same logic as E-1's lazy-init pattern; sonnet has the precedent.

### Risk 3 — IoUring construction failure mid-rebuild

**What:** `IoUring::new(needed_capacity)` returns `io::Result<IoUring>`. The rebuild block must propagate the Err as `SelectOutcome::SubstrateError(e)`. If sonnet uses `.unwrap()` or `.expect()`, a real kernel error panics. The BRIEF spec says `return SelectOutcome::SubstrateError(e)` matching the pre-E-2 per-call error path.

**Mitigation:** BRIEF skeleton uses `match` with explicit `Err(e) => return SelectOutcome::SubstrateError(e)`.

### Risk 4 — Unwrap on the Option<(IoUring, u32)>

**What:** After the reflexive rebuild block, the ring is guaranteed `Some(_)`. The BRIEF uses `ring_slot.as_mut().unwrap().0` to get `&mut IoUring`. If sonnet uses `.expect("...")` with a misleading message, the panic semantic is mostly the same but the message must be honest. If sonnet introduces a defensive `if let Some` instead of unwrap, the dead `else` branch is reap-flag material.

**Mitigation:** BRIEF skeleton uses `unwrap()` and inscribes the SAFETY-of-unwrap claim (reflexive rebuild guarantees Some).

### Risk 5 — Receiver methods visibility

**What:** New methods `Receiver::read_into_acc` + `Receiver::take_buffered_frame` are `pub(crate)`. Per the substrate-internal audience pattern (matches the existing `take_frame` free function's implicit visibility). If sonnet makes them `pub`, they leak into the user-facing API; if sonnet makes them private (no visibility modifier), Select can still call them within the same module — actually that works too. `pub(crate)` is the explicit "substrate-internal" marker.

**Mitigation:** BRIEF explicit `pub(crate)`.

### Risk 6 — Receiver::recv + try_recv refactor doesn't introduce regression

**What:** Replacing `uring_read_into_acc(read_fd, &self.accumulator, &self.ring)` with `self.read_into_acc()` should be behavior-identical (the method is a thin wrapper). But if sonnet inadvertently changes the call-site error mapping (e.g., from `.map_err(|_| RecvError)?` to a different shape), tests fail.

**Mitigation:** BRIEF says "behavior-identical refactor"; the method returns the same `Result<usize, ()>` shape.

### Risk 7 — Rune retirement completeness

**What:** The `rune:temperare(no-reactor)` at line ~742 retires entirely. The SURROUNDING doc-comment (which references "Stone E-2 (task #394) will persistify this ring per Receiver/Select") was forward documentation that now points at completed work. Both retire. If sonnet leaves the doc-comment but removes the rune, the doc-comment is stale.

**Mitigation:** BRIEF says "the doc-comment surrounding the rune retires alongside it."

### Risk 8 — Free functions stay

**What:** The free functions `take_frame` and `uring_read_into_acc` are still called by the Receiver methods internally; AND `wait_for_data_or_cascade` is still called by Receiver::recv. None retire. If sonnet purgare-flags any of them as dead and deletes, the build breaks.

**Mitigation:** BRIEF STOP triggers spell this out.

### Risk 9 — Test count preservation

**What:** All 34 existing tests must continue to pass. No new tests. If sonnet adds a probe test out of habit, test count grows; ward may flag (purgare).

**Mitigation:** BRIEF explicitly says "NO new tests."

### Risk 10 — Doc comment scope creep

**What:** Doc comments need updates on Receiver methods + Select struct field + Select::select body + module-level. If sonnet rewrites doc comments on UNTOUCHED items (e.g., Sender), gaze may flag inconsistency or scope drift.

**Mitigation:** BRIEF lists exactly which doc comments to update.

### Risk 11 — Preserving Stones A-E1 unchanged

**What:** E-2 ADDS methods to Receiver + a field to Select; refactors recv/try_recv/select internally. ALL other behavior preserved exactly. Specifically: Receiver's fields (read_fd, accumulator, ring, _phantom) unchanged in count/shape; Receiver::clone unchanged; pair() factory unchanged.

**Mitigation:** BRIEF's STOP triggers list every helper / method as UNCHANGED except the named ones.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | Module-level doc: "(through Stone E-1)" → "(through Stone E-2)" naming persistent Select ring + Receiver method extraction | YES |
| 2 | Audience section: Stone E-2 entry removed from the "future" list | YES |
| 3 | `Receiver::read_into_acc(&self) -> Result<usize, ()>` minted as `pub(crate)` method with doc comment | YES |
| 4 | `Receiver::take_buffered_frame(&self) -> Option<Vec<u8>>` minted as `pub(crate)` method with doc comment | YES |
| 5 | `Receiver::recv` fast-path uses `self.take_buffered_frame()` (2 sites) instead of `take_frame(...)` | YES |
| 6 | `Receiver::recv` Read step uses `self.read_into_acc()` instead of `uring_read_into_acc(...)` | YES |
| 7 | `Receiver::try_recv` fast-path uses `self.take_buffered_frame()` instead of `take_frame(...)` | YES |
| 8 | `Receiver::try_recv` Read step uses `self.read_into_acc()` instead of `uring_read_into_acc(...)` | YES |
| 9 | `Select<'a, T>` struct gains `ring: RefCell<Option<(IoUring, u32)>>` field with doc comment | YES |
| 10 | `Select::new()` initializes `ring: RefCell::new(None)` | YES |
| 11 | `Select::select` fast-path uses `rx.take_buffered_frame()` instead of `take_frame(...)` | YES |
| 12 | `Select::select` reflexive rebuild block at loop top: pattern-match `ring_slot.as_ref()`; rebuild on None OR capacity mismatch; assign `Some((r, needed_capacity))` | YES |
| 13 | `Select::select` rebuild failure path: `return SelectOutcome::SubstrateError(e)` (not unwrap/expect) | YES |
| 14 | `Select::select` inner submission block borrows `self.ring` via `borrow_mut()`; scope releases before Read step | YES |
| 15 | `Select::select` submission block: POLL_ADDs pushed through the persistent ring (not a fresh IoUring) | YES |
| 16 | `Select::select` Read step uses `rx.read_into_acc()` (not free function `uring_read_into_acc(...)`) | YES |
| 17 | `Select::select` partial-frame post-Read check uses `rx.take_buffered_frame()` (not free function `take_frame(...)`) | YES |
| 18 | `rune:temperare(no-reactor)` at line ~742 RETIRED + surrounding doc-comment retired | YES |
| 19 | Per-call `IoUring::new(ring_capacity)` inside `Select::select`'s loop ELIMINATED | YES |
| 20 | `cargo build --release` clean | YES |
| 21 | `cargo test --release --test comms` 34/34 PASS (zero net delta from E-1) | YES |
| 22 | `cargo test --release --test probe_channel_primitive` 3/3 PASS unchanged | YES |
| 23 | `cargo test --release --test probe_pidfd_primitive` 2/2 PASS unchanged | YES |
| 24 | NO new probe tests added | YES |
| 25 | Stone A helpers (`take_frame`) UNCHANGED (still called by Receiver::take_buffered_frame internally) | YES |
| 26 | Stone B `PollOutcome` + `wait_for_data_or_cascade` UNCHANGED (still called by Receiver::recv) | YES |
| 27 | Stone C `decode_frame` + `Sender::send` + Sender side of `pair<T>()` UNCHANGED | YES |
| 28 | Stone D1 close/len/CommSender/CommReceiver trait impls UNCHANGED | YES |
| 29 | Stone D2 `Select::new` / `Select::recv` UNCHANGED in signature (Select::new adds 1 line for ring init; Select::recv body unchanged) | YES |
| 30 | Stone E-1 Receiver field set (read_fd / accumulator / ring / _phantom) UNCHANGED — methods ADDED only | YES |
| 31 | Stone E-1 Receiver::clone UNCHANGED | YES |
| 32 | Stone E-1 `pair<T>()` factory UNCHANGED | YES |
| 33 | Stone E-1 free function `uring_read_into_acc` UNCHANGED (still called internally by Receiver::read_into_acc) | YES |
| 34 | Free function `take_frame` UNCHANGED (still called internally by Receiver::take_buffered_frame) | YES |
| 35 | NO config tunable code added (no `set-process-tier-uring-depth!` setter, atomic, validation) | YES |
| 36 | NO `wat_arc170_program_contracts` re-run | YES |
| 37 | Dirty tree (src/fork.rs + src/spawn_process.rs + arc 213 δ-2 docs) UNTOUCHED | YES |
| 38 | `src/typed_channel.rs`, `src/edn_shim.rs`, `src/comms/mod.rs`, `src/comms/thread.rs`, `Cargo.toml` UNTOUCHED | YES |
| 39 | Zero modifications outside `src/comms/process.rs` + SCORE doc | YES |
| 40 | SAFETY comments on all unsafe blocks PRESERVED + still honest (lifetimes still hold despite borrow rescoping) | YES |
| 41 | Every new public/pub(crate) item has a doc comment | YES |
| 42 | NO commit (orchestrator commits after verify + ward pass) | YES |

## Mode classification

- **Mode A:** all 42 criteria satisfied
- **Mode B (acceptable):**
  - Risk 1 fires (borrow scoping): compile error; sonnet adjusts scope block placement
  - Risk 2 fires (rebuild logic off-by-one): one Select test hangs or returns wrong outcome
  - Risk 4 fires (unwrap message wrong/missing): minor doc; gaze flag in ward pass
  - One probe test fails: sonnet investigates
- **Mode C (failure):**
  - Touched any file outside `src/comms/process.rs` + SCORE doc
  - Added a config tunable
  - Added new probe tests
  - Modified Stone A `take_frame` body
  - Modified Stone B `wait_for_data_or_cascade` body
  - Modified Receiver field set
  - Kept per-call `IoUring::new(ring_capacity)` inside Select::select's loop
  - Deleted free functions `uring_read_into_acc` / `take_frame` / `wait_for_data_or_cascade` (still have internal callers)

## Calibration metadata

- **Orchestrator confidence:** MEDIUM-HIGH. E-2 has more moving parts than E-1; the borrow scoping for Select.ring vs Receiver.ring is the highest risk; the BRIEF skeleton spells out scope blocks explicitly. The Receiver method extraction is mechanical; the reflexive rebuild is small and well-bounded.
- **Risk factors:** Borrow scoping; rebuild logic correctness; unwrap discipline post-rebuild; free function preservation.

## Ward pass prediction (9 wards per kernel impeccability protocol, broadened in E-1 ward pass)

Same 9 wards as E-1: intueri + struere + purgare + solvere + temperare + conferre + mora + perspicere + nesciens.

- gaze (intueri): 0-1 (smaller surface than E-1's struct + helpers + clone; one new struct field + 2 new methods + Select::select body refactor)
- forge (struere): 1-2 (RefCell<Option<(IoUring, u32)>> is the new shape; possible mumble on tuple-of-(ring, cap) vs separate fields)
- reap (purgare): 0-1 (free functions stay; rune retires cleanly; possible doc-mention residue)
- sever (solvere): CLEAN (this stone CLOSES solvere's E-1 finding; Select stops braiding into Receiver)
- temper (temperare): CLEAN (this stone CLOSES the last rune:temperare(no-reactor); reflexive rebuild IS the discipline)
- conferre: 0-1 (verify implementation matches BRIEF + DESIGN; possible documentation drift)
- mora: CLEAN (per-call ring construction inside select() loop ELIMINATED; one more pause class structurally gone)
- perspicere: 0-1 (RefCell<Option<(IoUring, u32)>> is 2-3 layers; could surface a noun like `MaybePersistedRing` but acceptable depth; rune-of-judgment if marginal)
- nesciens: 0-1 (fresh reader walks the file; does the reflexive rebuild block teach itself; module doc points at DESIGN.md already)

Total predicted: 1-5 findings. Round 2 unlikely. Multiple "CLEAN" predictions because Stone E-2 CLOSES findings rather than introducing them.

## Cross-references

- BRIEF-214-SLICE-3E2-SELECT-PERSISTENT-RING.md — this stone's work order
- DESIGN.md § "Stone E forward-correction (2026-05-19) — TCO discipline + reflexive rebuild" — the architectural reframe E-2 ships
- BRIEF + SCORE + WARD-PASS for E-1 — the pre-E-2 foundation (Receiver persistent ring; helper signatures take `&RefCell<IoUring>`)
- WARD-PASS-3E1-RECEIVER-PERSISTENT-RING.md § "Deferred to Stone E-2" — solvere's plan E-2 executes (Receiver methods)
- `feedback_iterative_complexity` — Stone E split into E-1 + E-2 per four-questions
- `feedback_substrate_owns_not_callers_match` — Receiver method extraction codifies this at the Receiver/Select boundary
