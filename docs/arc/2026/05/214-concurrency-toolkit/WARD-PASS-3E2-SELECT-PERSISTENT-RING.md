# Arc 214 Slice 3 Stone E-2 — WARD PASS

Per the kernel-impeccability protocol (INTERSTITIAL § 2026-05-19): per-stone trust gate = BRIEF scorecard verification + ward pass before commit. This doc captures the ward round-trip for Stone E-2 (Select persistent ring with reflexive rebuild + Receiver method extraction closing solvere's E-1 finding).

**9 wards cast** (same set as E-1's broadened pass). Stone E-2 was expected to CLOSE more findings than introduce — and that's what happened.

## Round 1 — initial ward pass (after sonnet SCORE Mode A)

Target: `src/comms/process.rs`. Sonnet shipped Mode A 42/42 in 7.4 min with 0 STOP triggers.

### Convergence summary (after all 9 wards returned)

| Finding | Spells flagging | Level |
|---|---|---|
| Capacity invariant `+1` doc/code divergence (3 doc sites vs code computation) | intueri + struere + nesciens | **3× (2× L2 + 1× L1)** |
| Missing `RingSlot` typealias on Select.ring field | perspicere | **L1** |
| Residual `rx.read_fd.as_raw_fd()` braid at line 832 (POLL_ADD construction) | solvere | L2 |
| "Solvere ward" / "finding closure" terms undefined inline | nesciens | L2 |
| Missing `Frame` typealias on take_frame + take_buffered_frame | perspicere | L2 (borderline) |
| `Result<(), SendError<T>>` SendResult alias candidate | perspicere | L2 (rune candidate) |
| Per-call io_uring elimination CONFIRMED structural | mora | **CLEAN** |
| Scope discipline + callers verified | purgare | **CLEAN** |
| Last `rune:temperare(no-reactor)` closure honest | temperare | **CLEAN** |
| Spec ↔ code fidelity verified | conferre | **CLEAN** (2 minor SCORE doc imprecisions; no code defects) |

**3-spell convergence on the invariant divergence + 3 CLEAN verdicts.** Substrate-as-teacher cascade firing as designed.

### Out-of-scope observations elevated by user direction

**User's red flag 2026-05-19 mid-fix-pass:** purgare named "Select has no Debug impl" as out-of-scope-for-future. User caught the L2 cost-anxiety pattern: *"fix this before we close anything — this is a thing to address alongside whatever else the spells reveal."* Per `feedback_no_known_defect_left_unfixed`: known defect; fix now; no deferral. Manual `impl Debug for Select<'a, T>` added in this fix-pass.

### Per-spell verdicts (load-bearing summaries)

**intueri (gaze):** L2 — invariant divergence at 3 doc sites (lines 36, 702, 713) vs code (line 778). Plus: confirmed `rune:temperare(no-reactor)` fully RETIRED (count 0). Module + audience sections clean.

**struere (forge):** L2 (same invariant convergence). Plus CLEAN on: RefCell<Option<(IoUring, u32)>> shape honest; reflexive rebuild pattern-match correct (None + mismatch arms); borrow scoping clean; `.unwrap()` justified; SAFETY comments preserved.

**purgare (reap):** CLEAN. All E-2 additions LIVE (read_into_acc + take_buffered_frame have callers; ring field initialized + read + written; free functions retain their internal callers). One out-of-scope observation — Select Debug impl absence — escalated to fix-pass per user direction.

**solvere (sever):** L2 — 3 of 4 E-1 braid sites CLOSED (accumulator + ring access in Read step + post-Read partial-frame check via the new Receiver methods). 1 RESIDUAL site: `rx.read_fd.as_raw_fd()` at line 832 (POLL_ADD construction). Fix: mint `pub(crate) fn poll_fd(&self) -> RawFd` on Receiver; replace the one site. Closes the final strand.

**temperare (temper):** CLEAN — the LAST `rune:temperare(no-reactor)` in the file is RETIRED. The closure matches the rune's own inscription exactly: per-call `IoUring::new(ring_capacity)` inside `Select::select`'s loop is gone; persistent ring with reflexive rebuild has replaced it. **"The arc 214 slice 3 io_uring heat catalog is closed."**

**conferre (compare):** CLEAN. All 42 EXPECTATIONS rows verified. TCO discipline honored. "No tunable" verified (`grep "set-process-tier-uring-depth" src/` → 0 hits). 2 minor SCORE doc imprecisions (grep counts) — non-critical, fixed below.

**mora (hunt pauses):** CLEAN. The LAST per-call IoUring construction inside Select::select's loop is structurally eliminated. Reflexive rebuild fires only on topology change; steady-state runs zero constructions. No new pauses introduced (RefCell borrow_mut is a logical assertion, not a temporal delay).

**perspicere (type clarity):** L1 + 2 L2. L1: `RefCell<Option<(IoUring, u32)>>` should be `RefCell<RingSlot>` — the borrow variable in select() is already named `ring_slot`. L2 (borderline): `Frame` alias for `Vec<u8>` (2 sites); apply rune OR mint. L2 (rune candidate): `SendResult<T>` for `Result<(), SendError<T>>` — perspicere recommends `rune:perspicere(mumble-alias)` since `SendError` already carries the noun.

**nesciens (teachability):** L1 + L2. L1: invariant divergence (same as intueri + struere) — fresh reader cannot resolve doc vs code without leaving the file; nesciens elevates to L1 because the gap blocks the reader's mental model. L2: "Solvere ward" / "finding closure" terms used in doc comments without inline definition; one-word expansion would close the re-read.

## Orchestrator design decisions

**Decision 1: 3-spell convergence on invariant divergence — APPLY (fix the doc).**
- intueri + struere + nesciens all flagged the same site
- conferre confirmed the math is equivalent under the code's accounting (arm_count includes broadcast); the doc's "+1" lies because it presumes a different accounting
- Fix: 3 doc sites (lines 36, 702, 713) updated to match code's formula: `cap == next_power_of_two(arm_count).max(2)` with explicit note "arm_count includes the broadcast slot when active"

**Decision 2: perspicere L1 (RingSlot typealias) — APPLY.**
- Same pattern as E-1's Accumulator typealias
- Borrow variable `ring_slot` already names the noun in the author's mental model
- Mint `type RingSlot = Option<(IoUring, u32)>;` adjacent to Accumulator; Select.ring uses RingSlot

**Decision 3: solvere L2 (residual braid) — APPLY (mint poll_fd).**
- Mint `pub(crate) fn poll_fd(&self) -> RawFd` on Receiver
- Replace `rx.read_fd.as_raw_fd()` at line 832 with `rx.poll_fd()`
- Closes the FINAL strand of solvere's E-1 finding
- All 4 of E-1's braid sites now CLOSED — Select fully composes via Receiver's surface

**Decision 4: User's red flag (Select Debug impl) — APPLY (mirror Receiver's pattern).**
- Manual `impl Debug for Select<'a, T>` with `rune:purgare(public-api)` on the impl block
- Renders ring as opaque "None" or "Some(IoUring, cap={n})" — hides the !Debug IoUring; surfaces the recorded capacity
- Symmetric with Receiver's manual Debug from E-1

**Decision 5: nesciens L2 ("Solvere ward / finding closure" inline) — APPLY.**
- One-word expansions in both Receiver method doc comments: "Closes the Solvere ward finding from E-1 ward pass 2026-05-19 (Select was braiding into Receiver internals; deferred to E-2 for resolution; E-2 mints this method + Select calls it)"

**Decision 6: perspicere L2 (Frame typealias) — APPLY (mint Frame).**
- 2 sites (take_frame + take_buffered_frame return types)
- The noun "Frame" is already in the module's vocabulary (module doc § Framing; function names; local variable `frame`)
- `decode_frame` keeps `&[u8]` — accepts any byte slice via Deref coercion; Frame names the SHAPE produced, not a constraint on what decode accepts

**Decision 7: perspicere L2 (SendResult<T>) — APPLY rune-of-judgment.**
- Per perspicere's own recommendation: `rune:perspicere(mumble-alias)` on `Sender::send` signature
- Inscribes the deliberate non-mint (SendError<T> already carries the noun; alias would not be more pronounceable)

**Decision 8: perspicere out-of-scope (full-path `std::cell::RefCell`) — already fixed in E-1 fix-pass; no action.**

## Fix pass — orchestrator-direct

Net +90 LOC delta across the 7 fixes. All applied to `src/comms/process.rs` only.

| # | Fix | File:line | Spells closed |
|---|---|---|---|
| 1 | Module doc invariant: `cap == next_power_of_two(arm_count).max(2)` | process.rs:36 | intueri + struere + nesciens |
| 2 | Select struct doc invariant + accounting note | process.rs:718-722 | intueri + struere + nesciens |
| 3 | Select.ring field doc invariant | process.rs:729-732 | intueri + struere + nesciens |
| 4 | Mint `type RingSlot = Option<(IoUring, u32)>;` near Accumulator | process.rs:96-110 | perspicere (L1) |
| 5 | Select.ring field type uses RingSlot | process.rs:732 | perspicere (L1 part 2) |
| 6 | Mint `Receiver::poll_fd()` pub(crate) method | process.rs:451-461 | solvere (closing) |
| 7 | Select::select POLL_ADD site uses `rx.poll_fd()` | process.rs:832 | solvere (closing) |
| 8 | Manual `impl Debug for Select<'a, T>` + `rune:purgare(public-api)` | process.rs:773-792 | user red flag |
| 9 | Expand "Solvere ward / finding closure" inline (2 sites) | process.rs:413-417, 432-436 | nesciens |
| 10 | Mint `type Frame = Vec<u8>;` near Accumulator + RingSlot | process.rs:112-122 | perspicere (L2) |
| 11 | `take_buffered_frame` + `take_frame` return `Option<Frame>` | process.rs:447, 652 | perspicere (L2 part 2) |
| 12 | `rune:perspicere(mumble-alias)` on `Sender::send` | process.rs:153 | perspicere (L2) |

## Verification

```
cargo build --release             # CLEAN (5 pre-existing dead_code warnings)
cargo test --release --test comms # 34/34 PASS (zero net delta from sonnet's E-2)
cargo test --release --test probe_channel_primitive   # 3/3 PASS
cargo test --release --test probe_pidfd_primitive     # 2/2 PASS
```

Rune inventory post-fix-pass:
- `rune:temperare` count: **0** (catalog closed)
- `rune:purgare(public-api)`: 2 sites (Receiver Debug + Select Debug — symmetric)
- `rune:sequi(ambient-context)`: 1 definition + 3 call-site references (current_broadcast_fd helper)
- `rune:perspicere(read-once)`: 1 site (pair() return shape — E-1)
- `rune:perspicere(mumble-alias)`: 1 site (Sender::send — E-2 fix-pass)

## Solvere's E-1 finding — FULLY CLOSED

| E-1 braid site | Status post-E-2 fix-pass |
|---|---|
| `rx.accumulator` in Select's Read step | CLOSED via `Receiver::read_into_acc` |
| `rx.ring` in Select's Read step | CLOSED via `Receiver::read_into_acc` |
| `rx.accumulator` in Select's partial-frame checks | CLOSED via `Receiver::take_buffered_frame` |
| `rx.read_fd` in Select's POLL_ADD loop | CLOSED via `Receiver::poll_fd` (added in this fix-pass) |

**ALL 4 sites closed.** Select fully composes via Receiver's surface. The braid is structurally retired.

## What this stone closes (the full arc 214 slice 3 catalog)

Stone E-2 + this ward-pass fix-pass close the ENTIRE arc 214 slice 3 substrate-as-teacher findings catalog:

- Per-call IoUring construction in Receiver helpers (E-1)
- Per-call IoUring construction in Select::select (E-2)
- Select-into-Receiver braid (E-1 solvere; E-2 closes 3 sites; E-2 fix-pass closes the 4th)
- Capacity invariant doc/code divergence (E-2 fix-pass)
- Missing nouns at the type level (E-1 Accumulator; E-2 RingSlot + Frame)
- Manual Debug impls for !Debug-holding structs (E-1 Receiver; E-2 Select via user's red flag)

**The arc 214 slice 3 io_uring heat catalog is closed.** Reflexive autoscaling of correctness (per `project_autoscaling_correctness` + Convergence #13 inscribed in INTERSTITIAL) is operational at both the Receiver layer (static-need case) and the Select layer (variable-arm-count case with reflexive rebuild grow OR shrink).

## Round 2 — not needed

All findings addressed in Round 1 fix-pass. Workspace test counts preserved (34/34 + 3/3 + 2/2). Cargo build clean. The Select Debug impl that user flagged as red is in place.

## Personal-discipline note

Caught myself at L2 cost-anxiety AGAIN during this fix-pass when purgare's out-of-scope observation about Select-no-Debug was first read. User immediately escalated. Per `feedback_no_known_defect_left_unfixed` + Song #13 (NO FEAR): when we know how to surface a defect RIGHT NOW, do it now. Future fix-passes default to ESCALATING out-of-scope observations from wards UNLESS scope is genuinely structural (different arc; different layer). The pattern: ward names it → orchestrator runs four-questions → if YES YES YES YES on fixing now, FIX. Don't defer on known defects.

## Cross-references

- SCORE-214-SLICE-3E2-SELECT-PERSISTENT-RING.md — sonnet's scorecard (Mode A; 42/42)
- BRIEF-214-SLICE-3E2-SELECT-PERSISTENT-RING.md — work order
- EXPECTATIONS-214-SLICE-3E2-SELECT-PERSISTENT-RING.md — 42-row scorecard + 11 named risks
- WARD-PASS-3E1-RECEIVER-PERSISTENT-RING.md § "Deferred to Stone E-2" — solvere's plan E-2 executed
- DESIGN.md § "Stone E forward-correction (2026-05-19)" — the architectural reframe E-2 ships
- INTERSTITIAL § 2026-05-19 "Convergence #13: reflexive autoscaling of correctness" — what Stone E-2 operationalizes
- `feedback_no_known_defect_left_unfixed` — the doctrine the user's red flag enforced
- `feedback_attack_foundation_cracks` + `feedback_any_defect_catastrophic` — the doctrines the 3-spell cascade + fix-pass close
