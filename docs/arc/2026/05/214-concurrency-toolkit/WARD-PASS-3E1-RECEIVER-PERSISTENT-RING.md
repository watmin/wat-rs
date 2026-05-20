# Arc 214 Slice 3 Stone E-1 — WARD PASS

Per the kernel-impeccability protocol (INTERSTITIAL § 2026-05-19): per-stone trust gate = BRIEF scorecard verification + ward pass before commit. This doc captures the ward round-trip for Stone E-1 (Receiver persistent ring at capacity 4).

**9 wards cast** (broader than prior arc 214 stones' 5-ward pattern). User direction 2026-05-19 mid ward-pass: *"only 5 spells?... is our reference wrong?.. we've got 16 of them now... we need to run the ones that are applicable."* The kernel-impeccability protocol inscribed 2026-05-19 had referenced the 5-ward subset; the datamancy grimoire now holds 16. Reviewed each spell for applicability to Stone E-1's diff; 9 cast, 7 deferred as non-applicable to E-1's scope.

## Spells cast (9)

| Spell | Domain | Applicable why |
|---|---|---|
| **intueri** (gaze) | Names speaking, function size, comments, structure | Always-on for kernel additions |
| **struere** (forge) | Values/places (Hickey), types-enforce (Beckman), composition | Always-on for kernel additions |
| **purgare** (reap) | Dead code, unused fields, scaffolding, orphan code | Always-on for kernel additions |
| **solvere** (sever) | Tangled concerns, misplaced logic, duplicated encoding | Always-on for kernel additions |
| **temperare** (temper) | Redundant computation, residual heat, fail-stop panics in hot paths | Stone E-1 CLOSES 2 temperare runes; cast validates the closure |
| **conferre** (compare) | Spec ↔ code divergence | New per-stone discipline; verify implementation against BRIEF + EXPECTATIONS + DESIGN |
| **mora** (hunt the pause) | Time discipline; sleep is a guess; events arrive via the wire | Stone E-1's entire point is reducing per-call ring construction (a hidden pause) |
| **perspicere** (see-through) | Deeply-nested type expressions hiding nouns | E-1 introduces new types/signatures (`RefCell<IoUring>`, helper sigs) |
| **nesciens** (teachability) | Fresh reader walks the path; what cannot be reached | E-1 changed code structure + doc comments; verify the TCO discipline teaches from the code |

## Spells NOT cast (7) + why

| Spell | Why not E-1 |
|---|---|
| **cernere** (sift phantom forms) | Wat language conformance; Stone E-1 is Rust |
| **complectens** (test-shape) | E-1 makes ZERO test changes |
| **probare** (substance vs prose) | Rust code is substance-rich by nature |
| **secare** (atomic ordering) | E-1 touched no atomics |
| **sequi** (state-monad/ambient) | E-1 didn't change the SHUTDOWN_BROADCAST_READ_FD ambient |
| **vigilia** (comprehensive cast) | Running individual spells in parallel IS the vigilia for this stone |
| **vocare** (caller-perspective tests) | No test changes |

## Round 1 — initial ward pass (after sonnet SCORE Mode A)

Targets:
- `src/comms/process.rs` — module-level doc updated; `Receiver<T>` gains `ring: RefCell<IoUring>` field; manual `impl Debug` added (IoUring is !Debug); helpers `wait_for_data_or_cascade` + `uring_read_into_acc` refactored to take `&RefCell<IoUring>`; Receiver::recv + try_recv + Clone updated; pair() factory constructs ring; Select::select's Read step (line 809) passes `&rx.ring`; 2 `rune:temperare(no-reactor)` runes deleted from helpers; Select's POLL_ADD rune at line 719 preserved.

Sonnet's report: Mode A 36/36 with ONE honest delta — `IoUring` doesn't implement `Debug`; `#[derive(Debug)]` failed to compile; manual `impl Debug for Receiver<T>` rendering the ring as `"IoUring"` placeholder was added (13 lines).

### intueri — 3 L2 findings

| # | Site | Level | Observation |
|---|---|---|---|
| 1 | process.rs:199-200 | L2 | Struct-level doc says "Read (2 SQEs) and POLL_ADD pair (4 SQEs)"; field-level doc directly below (line 215) says correct counts "(1 SQE) and (2 SQEs)". `uring_read_into_acc` pushes 1 SQE; `wait_for_data_or_cascade` pushes 2 SQEs. Capacity 4 is fine; the arithmetic supporting it is mis-stated. |
| 2 | process.rs:28-29 | L2 | Module doc reads "Only NO persistent ring on Select / reflexive rebuild-on-mismatch (Stone E-2)" — "Only NO" parses as grammatical contradiction. |
| 3 | process.rs:719 | L2 | `rune:temperare(no-reactor)` text says "will persistify this ring per Receiver/Select" — implies BOTH still pending. Post-E-1 the Receiver ring IS persistified; only Select remains. |

### struere — 1 L2 finding

| Site | Level | Observation |
|---|---|---|
| process.rs:199-200 | L2 | Same as intueri #1. Struct-level doc contradicts field-level doc. |

**Other forge dimensions CLEAN:** RefCell<IoUring> matches the existing RefCell<Vec<u8>> precedent (honest interior-mutability choice); ring lifetime matches kernel-resource ownership reality (per-Receiver, dropped at Receiver Drop); helper signatures cleanly separate caller ownership from helper borrow; manual Debug honestly handles IoUring's !Debug; Clone's fresh-ring-per-clone enforces !Sync structurally; SAFETY comments on unsafe blocks preserved and honest.

### purgare — 1 L2 finding

| Site | Level | Observation |
|---|---|---|
| process.rs:227 (Debug impl) | L2 | Manual `impl Debug for Receiver<T>` not exercised by any test or downstream struct today. Required structurally (IoUring is !Debug; derive would fail; Sender<T> derives Debug at line 87; symmetry demands Receiver<T> have it too). Missing `rune:purgare(public-api)` exemption naming the rationale. |

**CONFIRMED LIVE:** `ring: RefCell<IoUring>` field, refactored helpers, all imports, no TODOs/FIXMEs, CommSender/CommReceiver trait impls.

### solvere — 1 L2 finding

| Site | Level | Observation |
|---|---|---|
| process.rs:698, 747, 809, 820 | L2 | Select reaches into Receiver internals at 4 sites: `rx.read_fd.as_raw_fd()`, `rx.accumulator`, `&rx.ring`. Select decomposes Receiver into raw parts and passes those parts to free functions rather than calling methods. If Receiver's internal representation changes (e.g., wraps read_fd in a newtype), Select breaks. Resolves naturally at Stone E-2 by introducing `Receiver::read_into_acc(&self) -> Result<usize, ()>` and `Receiver::take_buffered_frame(&self) -> Option<Vec<u8>>` methods. The braid is within-module + explicitly transient per `rune:temperare(no-reactor)` at line 719. |

**Receiver's 4 fields (read_fd / accumulator / ring / _phantom) hang straight; helpers compose cleanly; Clone + pair() construct without braiding; Manual Debug separates cleanly.**

### temperare — 1 L2 finding + closures verified

| Site | Level | Observation |
|---|---|---|
| process.rs:199-200 | L2 | Same as intueri #1. Doc inconsistency about SQE counts. |

**Closures verified:**
- `uring_read_into_acc`: per-call `IoUring::new(2)` retired; takes `&RefCell<IoUring>`; calls `borrow_mut()`. CLOSED.
- `wait_for_data_or_cascade`: per-call `IoUring::new(4)` retired; takes `&RefCell<IoUring>`; calls `borrow_mut()`. CLOSED.
- Select POLL_ADD rune at line 719: PRESERVED + well-formed + still applicable (E-2 territory).

**Loop-invariant audit:** `current_broadcast_fd()` hoisted at line 262 (recv) + line 709 (Select::select) — correctly invariant. Receiver's ring capacity (4) right-sized for operation set. Clone's `IoUring::new(4).expect(...)` is one-time spawn-time cost (not hot path). pair()'s factory cost is honest. NO new heat introduced.

### conferre — 2 L2 findings

| # | Site | Level | Observation |
|---|---|---|---|
| 1 | process.rs:62 (module doc audience) | L2 | "Substrate-internal Rust code (Stone D's `Select`, Stone E's tunable, Slice 4's kernel dispatcher)" — names "Stone E's tunable" as a future audience member. Per DESIGN.md § "Stone E forward-correction (2026-05-19)": "Tunable rejected; setter not minted... E-3 (originally: config tunable) DIES." The tunable is DEAD, not deferred. Code comment is stale. |
| 2 | process.rs:199-200 | L2 | Same as intueri #1. Conferre adds root-cause analysis: DESIGN.md line 176 (in the SUPERSEDED section) uses ambiguous "SQEs" accounting (likely counting ring slots, not submitted SQEs). The struct-level doc inherited the wrong counts; the field-level doc used the correct submitted-SQE counts. |

**TCO discipline VERIFIED:** FDs persist (read_fd is OwnedFd field); ring persists alongside (RefCell<IoUring> field); helpers borrow then release; ring drops at Receiver Drop. ✓

**"No tunable" VERIFIED:** `grep "set-process-tier-uring-depth" src/` → 0 hits. ✓

**Scorecard rows 1-36 VERIFIED:** all 36 EXPECTATIONS rows pass against the implementation. The manual Debug impl is a known divergence already inscribed in SCORE Surprise 1; no further action needed.

### mora — CLEAN (1 out-of-scope observation)

- The 2 per-call `IoUring::new` constructions eliminated by Stone E-1 are CONFIRMED GONE from both helper bodies. The "Per-call IoUring::new(N) is retired" comments at lines 469 + 592 honestly document the closure.
- No `thread::sleep`, `recv_timeout`, busy-wait loops, or duration-as-mechanism patterns anywhere in the file.
- `Receiver::clone`'s `IoUring::new(4).expect(...)` is a one-time spawn-time cost (not a wait). Not flagged.
- `pair<T>()`'s `IoUring::new(4)` is a one-time factory cost. Not flagged.
- Select POLL_ADD per-call ring at line 720: `rune:temperare(no-reactor)` correctly names it as E-2 territory. Not a mora finding here.
- All blocking sites (`submit_and_wait(1)`, `libc::poll(timeout=0)`) wait on kernel fd-events. Honest event-waits.

**Out-of-scope observation:** the rune at line 719 is `rune:temperare(no-reactor)` not `rune:mora(no-reactor)`. If rune namespacing is strict, mora could ride alongside as a sibling rune. Out of E-1 scope; decide in E-2 context.

### perspicere — 1 L1 + 1 L2 + 1 out-of-scope observation

| # | Site | Level | Observation |
|---|---|---|---|
| 1 | process.rs:213 (field) + process.rs:597-599 (helper sig) | **L1** | `acc: &RefCell<Vec<u8>>` in helper signature + `accumulator: RefCell<Vec<u8>>` in struct field — the noun `Accumulator` is missing as a type-level name. Mint `type Accumulator = RefCell<Vec<u8>>;` at module scope. Field becomes `accumulator: Accumulator`; helper sig becomes `acc: &Accumulator`. |
| 2 | process.rs:843 (pair() return) | L2 | `pair<T>() -> std::io::Result<(Sender<T>, Receiver<T>)>` — 3 logical layers. The noun `ChannelPair<T>` could be aliased. Marginal: `pair()` is called once + destructured immediately; alias would be read-once-then-forgotten at each call site. Human judgment: mint or apply `rune:perspicere(read-once)`. |

**Out-of-scope observation:** helper signatures use full path `std::cell::RefCell<IoUring>` despite `use std::cell::RefCell` at line 65. Import hygiene; not perspicere's concern but worth a wash while touching the signatures.

### nesciens — 2 L2 findings

| # | Site | Level | Observation |
|---|---|---|---|
| 1 | process.rs:199 (struct doc) vs process.rs:215 (field doc) | L2 | Same factual contradiction as intueri/struere/temperare/conferre #1. Fresh reader hits the SQE-count contradiction at the struct doc vs field doc; cannot resolve without reading implementation. |
| 2 | process.rs:17-29 (module doc) | L2 | Module doc names WHAT Stone E-1 does ("Receiver owns persistent IoUring (capacity 4) for its lifetime; helpers operate on the Receiver's ring instead of per-call construction") but does NOT name WHY — the TCO discipline (FDs are persistent state; io_urings are ephemeral frames) lives in DESIGN.md but is unreachable from the file alone. Fresh reader has the mechanism but not the model. |

**Aggregate from nesciens:** the file converges (TCO discipline IS reachable by induction from per-Receiver + per-clone fresh-ring + borrowed-in-helper + per-call-in-Select), but the architectural NAME lives only in DESIGN.md. One-sentence pointer would close the gap.

---

## Convergence summary

| Finding | Spells flagging | Level |
|---|---|---|
| Struct doc SQE counts wrong (lines 199-200) | intueri + struere + temperare + nesciens + conferre | **L2 × 5** — strongest cascade of the night |
| Stale "Stone E's tunable" in module doc audience (line 62) | conferre | L2 |
| Missing `Accumulator` typealias | perspicere | **L1** |
| Module doc "Only NO" grammatical stumble (lines 28-29) | intueri | L2 |
| Stale rune text at line 719 | intueri | L2 |
| Module doc lacks TCO discipline pointer to DESIGN.md | nesciens | L2 |
| Manual Debug impl lacks `rune:purgare(public-api)` | purgare | L2 |
| Select braids into Receiver internals (4 sites) | solvere | L2 (E-2 territory) |
| `ChannelPair<T>` typealias debatable | perspicere | L2 (marginal) |
| Full-path RefCell in helper sigs | perspicere (out-of-scope) | observation |
| Rune namespacing on line 719 | mora (out-of-scope) | observation |

**Substrate-as-teacher cascade firing strongly:** the SQE-count finding has 5-spell convergence. Independent observers seeing the same truth IS the discipline working as designed.

## Orchestrator design decisions

**Decision 1: 5-spell convergence on struct doc SQE counts — APPLY.**
- 5 spells (intueri + struere + temperare + nesciens + conferre) all flag the same factual error. Trivial doc fix.
- Conferre's recommendation: also update BRIEF line 77 (preserve spec coherence with code).
- DESIGN.md line 176 is in a SUPERSEDED section preserved per `feedback_inscription_immutable` — DO NOT touch (historical record).
- Four-questions YES YES YES YES.

**Decision 2: perspicere L1 (Accumulator typealias) — APPLY.**
- Mint `type Accumulator = RefCell<Vec<u8>>;` at module scope.
- Update struct field declaration; update `uring_read_into_acc` parameter type.
- Four-questions YES YES YES YES; small change; noun surfaces at type level.

**Decision 3: intueri stale rune text at line 719 — APPLY.**
- Update from "per Receiver/Select; per-call construction is the pre-Stone-E placeholder" to language naming post-E-1 state honestly (Receiver done; Select pending E-2).

**Decision 4: intueri module doc "Only NO" stumble — APPLY.**
- Restate cleanly as "Select's POLL_ADD ring remains per-call — Stone E-2 territory (reflexive rebuild-on-mismatch with grow OR shrink)."

**Decision 5: nesciens TCO discipline pointer — APPLY.**
- Add one sentence in module doc with DESIGN.md cross-reference.

**Decision 6: purgare unexercised Debug impl — APPLY rune.**
- Add `rune:purgare(public-api)` above the impl with rationale (symmetry with Sender<T> derive; IoUring !Debug; load-bearing even though no current caller).

**Decision 7: perspicere L2 (ChannelPair) — APPLY `rune:perspicere(read-once)` NOT alias.**
- Factory called once with immediate destructure; alias would be read-once-then-forgotten at call sites.
- Rune inscribes the deliberate non-mint so future readers see we considered it.

**Decision 8: solvere L2 (Select braids) — DEFER to E-2 with explicit inscription HERE.**
- Solvere explicitly named E-2 as the natural resolution point.
- E-2's BRIEF will introduce `Receiver::read_into_acc(&self) -> Result<usize, ()>` and `Receiver::take_buffered_frame(&self) -> Option<Vec<u8>>` methods; Select's Read step + accumulator scan collapse to method calls; the braid retires.
- The `rune:temperare(no-reactor)` at line 719 already names the transient state. No additional rune needed.

**Decision 9: conferre L2 (stale "Stone E's tunable" audience) — APPLY.**
- Replace "Stone D's `Select`, Stone E's tunable, Slice 4's kernel dispatcher" with "Stone D's `Select`, Stone E-2's reflexive-rebuild ring persistification, Slice 4's kernel dispatcher".

**Decision 10: perspicere out-of-scope (full-path `std::cell::RefCell`) — APPLY as wash.**
- While touching helper signatures for Decision 2, also short-form `&std::cell::RefCell<IoUring>` → `&RefCell<IoUring>` for consistency with the `use std::cell::RefCell` import at line 65.

## Fix pass — orchestrator-direct

10 edits applied. Test count: 34/34 PASS (unchanged); cargo build clean.

| # | Fix | File:line | Spells closed |
|---|---|---|---|
| 1 | Module doc "Only NO" stumble + TCO discipline pointer to DESIGN.md | process.rs:17-29 | intueri + nesciens |
| 2 | Audience section: "Stone E's tunable" → "Stone E-2's reflexive-rebuild ring persistification" | process.rs:62-64 | conferre |
| 3 | Mint `type Accumulator = RefCell<Vec<u8>>;` at module scope (after imports) | process.rs:~75 | perspicere (L1) |
| 4 | Struct doc SQE counts: "(2 SQEs) and (4 SQEs)" → "(1 SQE) and (2 SQEs)" | process.rs:198-200 | intueri + struere + temperare + nesciens + conferre (5-spell cascade) |
| 5 | Struct field declaration: `accumulator: RefCell<Vec<u8>>` → `accumulator: Accumulator` | process.rs:213 | perspicere (L1 part 2) |
| 6 | `rune:purgare(public-api)` above manual Debug impl | process.rs:227 | purgare |
| 7 | `wait_for_data_or_cascade` signature: `&std::cell::RefCell<IoUring>` → `&RefCell<IoUring>` | process.rs:500-503 | perspicere (out-of-scope wash) |
| 8 | `uring_read_into_acc` signature: `acc: &std::cell::RefCell<Vec<u8>>` → `acc: &Accumulator`; `ring: &std::cell::RefCell<IoUring>` → `ring: &RefCell<IoUring>` | process.rs:619-623 | perspicere (L1 + out-of-scope wash) |
| 9 | Rune text at line 719: update to name post-E-1 state honestly | process.rs:719-723 | intueri |
| 10 | `rune:perspicere(read-once)` above `pair<T>()` | process.rs:~843 | perspicere (L2 — judgment to NOT mint alias) |

**Also updated for spec coherence (per conferre's recommendation):**

| # | Fix | File |
|---|---|---|
| BRIEF-1 | Update BRIEF line 77 to match struct doc fix (so the spec stays coherent) | BRIEF-214-SLICE-3E1-RECEIVER-PERSISTENT-RING.md |

## Verification

```
cargo build --release             # CLEAN (5 pre-existing dead_code warnings unrelated to comms)
cargo test --release --test comms # 34/34 PASS (unchanged from pre-fix)
cargo test --release --test probe_channel_primitive --test probe_pidfd_primitive  # 3/3 + 2/2 PASS
```

## Deferred to Stone E-2 (explicit inscription)

**Select-reaches-into-Receiver braid** (solvere L2):

Sites: `process.rs:698, 747, 809, 820` — Select accesses `rx.read_fd`, `rx.accumulator`, `rx.ring` directly instead of calling Receiver methods.

**E-2 plan** (informational; E-2 BRIEF will formalize):
- Mint `Receiver::read_into_acc(&self) -> Result<usize, ()>` — encapsulates the Read-step that today calls `uring_read_into_acc(rx.read_fd.as_raw_fd(), &rx.accumulator, &rx.ring)`.
- Mint `Receiver::take_buffered_frame(&self) -> Option<Vec<u8>>` — encapsulates the fast-path check that today calls `take_frame(&mut rx.accumulator.borrow_mut())`.
- Select's body composes via these methods; the 4 field-access sites collapse.
- The `rune:temperare(no-reactor)` at line 719 already names the transient state; no additional rune needed.

Per `feedback_inscription_immutable`: the braid stays inscribed here as historical record + the E-2 resolution plan stays inscribed as forward commitment.

## Round 2 — not needed

All findings addressed in Round 1 fix pass. No remaining L1 or L2 from any of 9 wards (except the Select braid, explicitly deferred to E-2 per solvere's own framing). Workspace test counts preserved (34/34 + 3/3 + 2/2). Cargo build clean.

## Cross-references

- SCORE-214-SLICE-3E1-RECEIVER-PERSISTENT-RING.md — sonnet's scorecard (Mode A; 36/36)
- BRIEF-214-SLICE-3E1-RECEIVER-PERSISTENT-RING.md — work order (updated for spec coherence at line 77)
- EXPECTATIONS-214-SLICE-3E1-RECEIVER-PERSISTENT-RING.md — 36-row scorecard
- DESIGN.md § "Stone E forward-correction (2026-05-19)" — TCO discipline + reflexive rebuild reframe
- WARD-PASS-3D2-SELECT.md — prior ward pass (5-ward pattern; E-1 broadens to 9)
- INTERSTITIAL § 2026-05-19 "Kernel impeccability via ward pass" — protocol that demanded this pass
- `feedback_inscription_immutable` — historical record discipline
- `feedback_attack_foundation_cracks` — substrate trust is binary; ward findings are defect candidates
- `feedback_substrate_owns_not_callers_match` — solvere's E-2 plan codifies this at the Receiver/Select boundary
