# Arc 214 Slice 3 Stone A — WARD PASS

Per the kernel-impeccability protocol (INTERSTITIAL § 2026-05-19): per-stone trust gate = BRIEF scorecard verification + ward pass before commit. This doc captures the ward round-trip for Slice 3 Stone A (io_uring bytes proof of life).

5 wards (gaze + forge + reap + sever + temper) — same set established in Slice 2.

## Round 1 — initial ward pass (after sonnet SCORE Mode A)

Targets:
- `src/comms/process.rs` (~180 LOC; Sender + Receiver + take_frame + pair via libc::pipe + io_uring + RefCell accumulator)
- `tests/probe_comms_process.rs` (~110 LOC; 6 smoke tests)
- `src/comms/mod.rs` (5-line `pub mod process;` block)
- `Cargo.toml` (+1 line: `io-uring = "0.7"`)

### gaze — CLEAN

> "Module-level doc is exemplary. The Stone A scope block names every absent feature (cascade, generics, try_recv, persistent ring) with explicit forward-pointer to the stone that adds it. The why-not-cascade section names the observable failure mode (recv WILL HANG) and the condition under which it is acceptable... Six test names are full sentences in snake_case... Each name tells the WHAT being verified and the EXPECTED OUTCOME."

The BRIEF's pre-emption discipline worked. Every public item carries WHY-comments; test names are honest sentences; the `pub mod process;` block notes cascade-aware status as `(Stone B)` not aspirational.

### forge — 2 L1 findings (missing SAFETY comments)

| Site | Level | Observation |
|---|---|---|
| process.rs:213 | L1 | `libc::pipe(fds.as_mut_ptr())` in `pair()` — no SAFETY comment directly above the unsafe block. The comment at lines 217-219 covers the `OwnedFd::from_raw_fd` calls below it, but the `pipe` call itself has no annotation. |
| process.rs:81-87 | L1 | `libc::write(fd, ptr, len)` in `Sender::send` retry loop — no SAFETY comment naming the fd validity + pointer/length invariants + `framed` Vec lifetime. |

The forge ward declared "FORGE CLEAN" in its header line but then surfaced two real findings. Interpretation: header was premature; findings are load-bearing.

Hickey lens, Beckman lens, type discipline, newtype invariants ("can't construct Sender/Receiver without pair()"): ALL CLEAN. Other unsafe blocks (`OwnedFd::from_raw_fd` × 2 + io_uring push) had honest SAFETY comments.

### reap — CLEAN

> "Every public item (Sender, Receiver, pair) is alive — used in all six tests via pair() return. take_frame is private; called twice inside recv() (fast path + slow path). The accumulator is read AND written. No TODO/FIXME/unimplemented! markers."

Honest-delta from sonnet (trimmed unused `Sender` / `Receiver` imports from the test file because tests only call `pair()`) was correct judgment — types are still exercised through the return value of `pair()`, just not directly imported by name.

### sever — CLEAN

> "Sender: owns write_fd; concern = frame + write. Receiver: owns read_fd + accumulator; concern = read + split frames. take_frame: pure helper. pair: factory. No braided concerns. The recv() body cleanly separates the fast path (accumulator check) from the slow path (io_uring loop)."

Each of the 4 logical units represents one concern; the recv() fast/slow paths are structurally partitioned.

### temper — CLEAN (with explicit deferral acknowledgment)

> "Known-deferred items (acknowledged, not flagged): IoUring::new(2) per loop iteration → Stone E. Cascade multi-arm → Stone B. Generic T: HolonRepresentable → Stone C."

Surfaced an honest observation about `Sender::send` allocating `framed: Vec<u8>` per call. Categorized as "deferred to Stone C" because Stone C replaces newline framing with length-prefixed EDN bytes anyway — the allocation pattern changes entirely. Not flagged as a defect; explicitly intentional simplicity per the BRIEF.

No unintentional waste found in the hot path; `buf: [u8; 4096]` is stack; `take_frame` is allocation-aware (one suffix Vec allocation per complete frame, unavoidable).

## Orchestrator design decisions (judgment calls)

**Decision 1: forge L1 findings (missing SAFETY comments)** — FIX. The doctrine requires SAFETY comments AT each unsafe site, not nearby. The libc::pipe and libc::write blocks need SAFETY comments above each `unsafe {` block naming the FFI/lifetime invariants being asserted.

**Decision 2: forge's "FORGE CLEAN" header vs findings body** — Trust the findings, not the header. Two real L1 findings stand; round 2 verifies fixes.

## Fix pass — orchestrator-direct (2 surgical edits)

Per the new protocol's "orchestrator addresses OR redirects sonnet": for these small mechanical SAFETY-comment additions, orchestrator applied directly.

| # | Fix | File:line |
|---|---|---|
| 1 | Add SAFETY comment above `libc::pipe(fds.as_mut_ptr())` in `pair()` — names the stack-allocation invariant + fd-write semantics | process.rs:213 (now 213-216) |
| 2 | Add SAFETY comment above `libc::write(...)` in `Sender::send` retry loop — names fd validity (OwnedFd-managed), pointer/length validity, framed Vec lifetime | process.rs:81-86 (now 81-93) |

Mechanical verification post-fix:
- `cargo test --release --test probe_comms_process` 6/6 PASS (comment-only changes; no-op for test logic)

## Round 2 — forge re-pass (only the one ward had findings)

### forge re-pass — CLEAN

> "Both SAFETY comments from Round 1 are now present and correctly placed. Site 1 (pair, libc::pipe): names the invariant — fds is a valid [i32; 2] stack allocation whose lifetime covers the call; libc::pipe writes two fds into it. Honest. Site 2 (Sender::send, libc::write): names two invariants — fd is valid for the lifetime of self.write_fd (OwnedFd-managed, not closed until Drop), and the pointer from framed[written..] is valid for the remaining byte count because framed is a live Vec on the function's stack not freed until after the loop. Honest. FORGE CLEAN."

No new forge findings introduced. Types + composition + abstractions all hold.

## Verdict

**STONE A IMPECCABLE — all 5 wards clean on re-pass.**

- gaze: code speaks; module-level doc names every absent feature with forward-pointer; test names carry WHAT + EXPECTED OUTCOME
- forge: types enforce contracts (newtype + private fields + sole pair() constructor); SAFETY comments honest at every unsafe site
- reap: zero dead thoughts; honest-delta (trimmed unused imports) was correct judgment
- sever: zero braided concerns; Sender / Receiver / take_frame / pair each represent one concern; recv() fast/slow paths structurally partitioned
- temper: no unintentional waste in hot path; Stone B/C/E deferrals explicitly acknowledged; not over-engineering for Stone A scope

The kernel-impeccability protocol's per-stone trust gate fires GREEN: BRIEF scorecard Mode A (40/40 satisfied per sonnet's SCORE; one beyond-scope import-trim self-flagged + judged correct) + ward pass CLEAN (all 5 wards green on re-pass).

Stone A ready to commit. Stone B (cascade-aware multi-arm POLL_ADD on [data_fd, broadcast_fd]) is the next stepping stone in Slice 3.

## Cross-references

- BRIEF-214-SLICE-3A-IO-URING-BYTES.md — work order
- EXPECTATIONS-214-SLICE-3A-IO-URING-BYTES.md — 40-row scorecard + 8 risk pre-emption
- SCORE-214-SLICE-3A-IO-URING-BYTES.md — sonnet's Mode A report
- WARD-PASS-1-FOUNDATION-PRIMITIVES.md — Slice 1 round-trip
- WARD-PASS-2-THREAD-TIER.md — Slice 2 round-trip; 5-ward protocol established
- INTERSTITIAL § 2026-05-19 "Kernel impeccability via ward pass" — protocol doctrine
- `feedback_assertion_demands_evidence` — every ward finding is evidence; act on it
- `feedback_any_defect_catastrophic` — kernel defects intolerable; missing SAFETY comments count as defect candidates per substrate's `unsafe` discipline
- `feedback_iterative_complexity` — why 5 stones in Slice 3 (defended 2026-05-19; per-stone isolation is the diagnostic-value point)
