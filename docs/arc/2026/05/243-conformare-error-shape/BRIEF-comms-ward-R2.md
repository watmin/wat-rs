# BRIEF — comms/ ward R2 (vigilia 9-spell divergence sweep)

**Target:** `src/comms/{mod,thread,process}.rs` (+ 2 named lines in `src/runtime.rs` for one fd-hygiene fix).
**Goal:** close every L1 + L2 the live 9-spell vigilia raised so `src/comms/` converges L1+L2=0 and earns its vigilatum stamp — the **6th and FINAL** warded home. REMARKABLE bar: every L1 and L2 closes; L3 is taste, left.
**Mode:** sonnet writes the substrate; orchestrator re-casts the divergent spells + stamps. **Do NOT commit. Do NOT push. Do NOT write a vigilatum stamp.** Leave the tree dirty.

---

## Hard rules (the map carries the STOP triggers)

- **Anchor:** `pwd` MUST be `/home/watmin/work/holon/wat-rs`. Any `.claude/worktrees/` path → `cd` to the anchor, use `git -C /home/watmin/work/holon/wat-rs`. Never operate in a worktree.
- **Write scope:** `src/comms/*.rs` + `tests/comms/*.rs` (the comms integration suite — Fix 1's test retirement and Fix 4/5's match-arm updates land there) + exactly the fd-hygiene fix in `src/runtime.rs` (Fix 7, the `broadcast_w_fd` OwnedFd wrap at ~line 271/316). ANY edit reaching a file outside that set → STOP and report (do not silently edit outside scope — a prior round did this; it was caught and reverted). Orchestrator-verified cascade: `SelectOutcome` and `process::Sender`/`pair()` have NO consumers outside `src/comms/` + `tests/comms/` (grep-confirmed) — so the compiler-guided cascade from Fix 1/5/8 stays inside the authorized set. If the compiler names a site OUTSIDE it, the crawl was wrong → STOP and report; do not chase it.
- **No commit, no stamp, no push.** Leave the tree dirty for the orchestrator's re-cast.
- **cargo, not venv** (this is wat-rs Rust). `-p wat` on every cargo command; bare `cargo test --lib` runs 0 tests (false green) — never use it.
- **No runes** — every finding below is solvable + non-perf-impairing → it must be FIXED. (Exception: the temperare items are NOT in this brief at all — see "Left as L3".)

---

## The cast being closed (9 spells; secare CONVERGED; 7 L1 + 11 L2 across the rest)

The findings cluster into themes. Fixes are ordered: lead (make-impossible) → abstraction-seam → mechanical → doc-truth.

---

## Fix 1 — LEAD: remove `Clone` from `process::Sender` (closes struere L1 + 2 circumspicere PIPE_BUF findings)

**Finding (3 lenses, one defect):** `process::Sender<T>` is `Clone` (struct doc at process.rs:131-134 advertises "MPMC-style write fan-in"), but `send()` (process.rs:158-197) writes a frame via a single `libc::write` loop with NO size bound. POSIX guarantees atomicity only for writes ≤ `PIPE_BUF` (4096). Two cloned senders writing frames > PIPE_BUF concurrently → **silent interleave / wire corruption** (no error; the receiver's newline-frame parser decodes garbage or spins). The struct comment even claims "single contiguous write" — false for >PIPE_BUF. `PIPE_BUF` appears ONLY in doc prose; no const exists.

**Verified scope fact (orchestrator crawl):** NO production caller clones a `process::Sender` — grep across src/ + tests/ shows the only clone is comms' own test `probe_slice3d1_sender_clone_shares_write_end` (tests/comms/process.rs:221). Arc-214 DESIGN.md:573 documents the multi-producer pattern as **"one shared `Sender` + N cloned *Receivers*"** (cloned RECEIVERS, not senders); every "fan-in" in the design is Select over N peers (receiver-side). **Cloned process Senders are an unused capability that is also a silent-corruption trap.**

**Fix — make the corruption structurally impossible (✅✅✅), single-writer:**
- Remove `#[derive(... Clone ...)]` (or the manual `Clone` impl) from `process::Sender<T>` at process.rs:135-142. Keep `Debug`, `PhantomData`, etc.
- The frame-size atomicity concern then **vanishes** — a single writer cannot interleave with itself; any frame size is safe on a single fd. No size guard, no wire-format change needed.
- **Correct the now-false struct doc** (process.rs:131-134): drop "Clone via OwnedFd::try_clone; cloned senders share the same kernel pipe (MPMC-style write fan-in)"; replace with the single-writer truth: the endpoint owns the write-end; `close(self)` consumes it; the peer sees EOF when the (sole) Sender closes. Note that if multi-producer fan-in is ever needed it must be built interleave-safe (length-prefix framing), not via raw-write Clone — record this as the reason the capability was retired.
- **Retire the now-invalid test** `probe_slice3d1_sender_clone_shares_write_end` (tests/comms/process.rs ~221): it asserted the corruption-prone behavior. Remove it (and any `Sender::clone` reference it holds). If removing it drops a numbered test count, that delta is EXPECTED and explained here — state it in your report.
- The `send()` "single contiguous write" comment (process.rs:163) is now TRUE for the single-writer (no concurrent interleave) — leave the framing as-is, but if the comment over-claims atomicity-vs-PIPE_BUF, soften to "single write syscall (single-writer endpoint — no concurrent interleave)."

**Cascade:** removing `Clone` is compiler-checked — if any non-test site clones a `process::Sender`, the build fails and names it. If that happens, STOP and report (it would contradict the crawl); do not chase it outside `src/comms/` + the authorized test.

---

## Fix 2 — `thread::Select` shutdown re-read (closes sequi L2 + its tier-asymmetry L3)

**Finding (sequi):** `thread::Select::new()` reads `SHUTDOWN_RX` ONCE at construction and bakes `shutdown_arm` into the struct. A `Select` built before `init_shutdown_signal()` has `shutdown_arm = None` and NEVER wakes on shutdown for its whole lifetime — an init-order trap enforced by nothing. The process tier reads its broadcast fd FRESH every `select()` (no trap); the asymmetry is the L3.

**Fix — converge the tiers:** make `thread::Select::select()` read `crate::runtime::SHUTDOWN_RX.get()` FRESH on each call (matching `process::Select`'s fresh-read pattern), rather than relying on a `shutdown_arm` captured at `new()`. The shutdown arm is added to the crossbeam `select!` at call time if `SHUTDOWN_RX` is now initialized. This closes the init-order hole AND erases the temporal asymmetry between tiers in one change. Verify the existing thread-tier cascade tests still pass.

---

## Fix 3 — empty-`Select` guard (closes struere L2, both tiers)

**Finding (struere, process.rs ~837):** a `Select` with zero registered receivers AND no broadcast fd submits zero SQEs then calls `submit_and_wait(1)` → hangs forever. `thread::Select` has the symmetric hole (crossbeam panics "cannot select with no operations").

**Fix:** at `select()` entry (both tiers), guard against the empty case: if there are no user arms AND no shutdown/broadcast arm available, return a clear error/outcome rather than hanging or panicking. Prefer a structural fix if clean (e.g. the type requires ≥1 arm), but an explicit `assert!`/early-return with a named message at `select()` entry is acceptable here since the empty Select is a caller misuse, not a representable-good state. Document the chosen guard.

---

## Fix 4 — `CommReceiver::len()` trait-doc honesty (closes intueri L1 + struere L1, mod.rs:635)

**Finding (2 lenses):** the `CommReceiver::len()` trait doc promises "Number of values currently queued in the channel awaiting recv" — but `process::Receiver::len()` returns an accumulator-only count (kernel pipe bytes not yet drained are invisible). A generic caller over `impl CommReceiver<T>` forms a false belief. (thread tier IS exact; process tier is a lower bound.)

**Fix:** narrow the TRAIT-level doc (mod.rs ~635) to the contract ALL impls actually satisfy — e.g. "Number of values locally buffered and ready for immediate `recv`. Implementations MAY undercount when transport-buffered values are not yet locally drained (e.g. the process tier does not count kernel-pipe bytes)." Make the trait promise match the weakest honest impl. No code change to either impl — this is a contract-truth fix.

---

## Fix 5 — `SelectOutcome::SubstrateError` tier placement (closes solvere L1, mod.rs:743)

**Finding (solvere):** `SelectOutcome::SubstrateError` is a process-tier io_uring failure class living in the TIER-AGNOSTIC `SelectOutcome` in mod.rs. `thread::Select` structurally cannot produce it → thread-tier callers must handle an impossible arm (or hide it in a catch-all that masks real logic errors). A domain concept misplaced in the shared layer.

**Fix — PINNED shape (orchestrator-drawn; do not invent a different one):** make the io_uring failure the `Err` of a `Result`, so the impossible arm vanishes from the thread tier by construction (✅✅✅):
- `SelectOutcome<T>` (shared, mod.rs:~739) drops the `SubstrateError(std::io::Error)` variant → it becomes just `Recv { index, result } | Shutdown`. Both tiers can produce BOTH remaining variants, so the shared type is now honest.
- `process::Select::select()` changes return type from `SelectOutcome<T>` to `Result<SelectOutcome<T>, std::io::Error>`. The 5 current `return SelectOutcome::SubstrateError(e)` sites (process.rs:874/902/920/928/937) become `return Err(e)`; the success returns wrap in `Ok(...)`.
- `thread::Select::select()` KEEPS returning the bare `SelectOutcome<T>` (the thread tier has no io_uring failure mode — it is infallible beyond Recv/Shutdown). This asymmetry is HONEST: the tiers genuinely differ; solvere's L1 is precisely that forcing them into one type lied.
- **Cascade (all in `tests/comms/`):** `tests/comms/process.rs:310-315` matches the process outcome directly → update to handle the `Result` (`match sel.select() { Ok(SelectOutcome::Recv{..}) => …, Ok(SelectOutcome::Shutdown) => …, Err(e) => … }`); DELETE its now-impossible `SubstrateError` arm. `tests/comms/thread.rs:120` and `tests/comms/foundation.rs:81/95` — DELETE the `SelectOutcome::SubstrateError` match arms (the variant no longer exists; these were the dead arms solvere flagged). `foundation.rs` constructs `SelectOutcome` values directly — leave the `Recv`/`Shutdown` constructions, remove only SubstrateError references.
If anything forces a touch OUTSIDE `src/comms/` + `tests/comms/`, STOP and report (crawl says there is nothing — it's all in-suite).

---

## Fix 6 — Display impls for the error types (closes conformare ×3 L2, mod.rs)

**Finding (conformare):** `WireError`, `RecvError`, `TryRecvError`, `SendError<T>` have no `Display` impl — so `?`-chains into `Box<dyn Error>` and `format!("{}", e)` don't compile; messages are only reachable via bespoke accessors. (conformare confirmed these are correctly SPANLESS-by-domain — wire/channel errors have no wat-source span; Pattern A does NOT apply. This is purely additive Display-completeness.)

**Fix:** add `impl Display` for each, single-sourced from the documented variant meanings:
- `WireError` → write its message (`write_str(&self.0)` or equivalent).
- `RecvError` → static "channel disconnected".
- `TryRecvError` → match: Empty → "channel empty"; Disconnected → "channel disconnected".
- `SendError<T>` → static "send failed: channel disconnected" (the held `T` is not necessarily Display — do NOT bound T: Display; the message is static).
If a `std::error::Error` impl is idiomatic alongside Display and trivial, add it too — but Display is the requirement. Verify no new bound leaks onto callers.

---

## Fix 7 — fd-hygiene: OwnedFd guard + EINTR retry (closes circumspicere ×2 L2)

**Finding 7a (circumspicere, runtime.rs ~271):** the shutdown broadcast pipe write-end (`broadcast_w_fd`) is captured as a raw `i32` in the shutdown-worker closure; `libc::close()` is called only AFTER `trigger_shutdown()`. If the worker panics before that close, the write-end never closes → POLLHUP never fires → every `process::Receiver::recv()` blocks forever. No RAII guard.
**Fix 7a:** wrap `broadcast_w_fd` as an `OwnedFd` inside the worker closure so `Drop` closes it unconditionally on ANY exit path (including panic). This is the ONLY authorized edit outside `src/comms/` — exactly this fd at the `broadcast_w_fd` site (~runtime.rs:271/316). If the change is more than wrapping that fd's lifecycle, STOP and report.

**Finding 7b (circumspicere, process.rs:695-726):** `uring_read_into_acc`'s `submit_and_wait()` can return `EINTR` (signal during wait); the io-uring crate does not auto-retry; the code maps `Err` → `RecvError`, silently treating a signal interrupt as channel death. `send()` ALREADY retries EINTR correctly (process.rs:187) — the read path just doesn't.
**Fix 7b:** after `submit_and_wait` returns `Err`, check `e.raw_os_error() == Some(libc::EINTR)` and retry the wait loop rather than returning the error. Mirror the pattern `send()` already uses.

---

## Fix 8 — dead `LinkedList` impl (closes purgare L2, HARD CUT)

**Finding (purgare, mod.rs:286-320):** `impl<T> HolonRepresentable for std::collections::LinkedList<T>` has ZERO callers (grep-verified across src/ + tests/; not in the `assert_holon_representable` compile check). It claims "LinkedList can cross a comms boundary" when nothing does.
**Fix:** delete the impl block (mod.rs ~286-320). HARD CUT — no deprecation, no shim. If anything fails to compile after removal, it had a hidden caller → STOP and report (contradicts the crawl).

---

## Fix 9 — retract the false "Slice 6 structural wall" claim (closes circumspicere's HIGHEST-rank L1, mod.rs:29-32)

**Finding (circumspicere, claim-vs-code):** the module doc states "Callers cannot bypass the cascade because tier wrappers hide the underlying mechanism. Bare `crossbeam_channel::*` and bare `libc::pipe/read/write/poll/epoll/io_uring_*` are unreachable outside the tier wrapper modules (Slice 6 structural wall)." This is FALSE TODAY: bare `crossbeam_channel::*` is used in 8 files outside comms/, bare `libc::pipe/write/poll` in 5; and DESIGN.md:732 confirms "Slice 6 — Structural wall" is a FUTURE planned slice (reorganize src/ + `pub(crate)` discipline). A never-retractable shipped doc claiming a wall that isn't built is the highest-severity finding (a claim outliving its code).
**Fix (DOC-TRUTH, in-home):** retract the claim to honest present-tense. Replace the "Callers cannot bypass... (Slice 6 structural wall)" sentence with something like: "**Intended invariant (NOT YET ENFORCED — Slice 6, pending):** tier wrappers should be the only path to the underlying mechanism. Today bare `crossbeam_channel::*` / `libc::*` remain reachable elsewhere in the crate; the structural wall (`pub(crate)` reorg) lands in arc-214 Slice 6 (DESIGN.md § Slice 6)." Do NOT claim enforcement that doesn't exist. (Landing Slice 6 itself is OUT of this ward's scope — it's a tree-wide reorg, its own future stone. This fix only makes the doc honest.)

## Fix 10 — correct the "mini-TCP capacity-1" doc for the process tier (closes circumspicere L1, mod.rs:37-39 + thread.rs:42-43)

**Finding (circumspicere, claim-vs-code):** mod.rs:37-39 says the process tier pair is "capacity-1 ... `send` blocks when the buffer holds one value" — FALSE: a Linux pipe buffer is 65536 bytes, not PIPE_BUF=4096, so the process pipe accepts ~hundreds of frames before blocking; it is NOT capacity-1. The "mini-TCP at depth 1" *symmetry* claim across tiers does not hold for process. thread.rs:42-43 compounds it ("64KiB per PIPE_BUF" names 64KiB with the wrong constant).
**Fix (DOC-TRUTH):** correct both doc sites to distinguish the two facts honestly: thread tier IS capacity-1 (crossbeam `bounded(1)`); the process tier's pipe is kernel-buffered (~65536 bytes, many frames) and individual writes ≤ PIPE_BUF=4096 are atomic — which is NOT the same as capacity-1. State that the mini-TCP *discipline* (one send pairs with one recv) is a usage discipline the process tier does not structurally enforce at depth-1 (per the same honest framing DESIGN.md:909 already uses). Keep the thread-tier capacity-1 claim (it's true).

---

## Left as L3 (NOT fixed — recorded, contested honestly)

- **temperare ×2 (data-path decode):** `Vec<T>::from_holon_ast` traverses items 3× + sorts an already-sorted sequence (mod.rs:210-243); `extract_positional_binds` double-allocates per tuple decode (mod.rs:564-597). **Orchestrator verdict: L3, LEFT.** Rationale (let-the-need-reveal-through-work): the comms channel is depth-1 lock-step ("mini-TCP at depth 1") — throughput is bounded by the send/recv handshake, not per-message decode microopt; the waste is real but on a path whose necessity-to-optimize is unproven. temperare's own spell says most premature optimization lives at L3. If a profiler ever shows decode as a hot path, revisit. **Do NOT fix these in this sweep** — touching them adds risk for no proven gain. (Recorded here so the L3 is a conscious contest, not a silent drop.)

---

## Gates (all must pass before you report done)
```
cargo build -p wat
cargo test -p wat --test comms        # the comms integration suite (tests/comms/) — must stay green (minus the 1 retired clone test)
cargo test -p wat --lib               # root lib suite stays green (was 895/0/1)
cargo clippy -p wat --all-targets     # no NEW warnings from src/comms/ or the runtime.rs fd fix
```
If a gate fails, the failure IS the next instruction (substrate-as-teacher) — read it, fix the named site, re-run. STOP + report any red you cannot resolve, or any edit that would reach outside the authorized scope.

---

## Report (your final message — the SCORE)
1. **Files touched** (must be only `src/comms/*.rs` + the authorized `src/runtime.rs` fd fix + the authorized test retirement).
2. **Per-fix disposition:** each of Fixes 1-10 — DONE / how, with the line(s) changed. Note Fix 1's retired test + Fix 5's deleted match arms + any test-count delta explicitly (expected: comms 33 → 32 from the one retired clone test; Fix 5 deletes arms but not whole tests, so no further count change unless a test is structurally retired — explain if so).
3. **Gate results** with counts (`cargo test -p wat --test comms` N passed / M failed; `cargo test -p wat --lib` N passed).
4. **Cascade check:** confirm removing `Clone` (Fix 1) and the `LinkedList` impl (Fix 8) produced NO compile errors outside the authorized scope (i.e. the crawl held — no hidden callers). If either DID, report it and STOP rather than chasing outside scope.
5. **Dirty set:** `git status --porcelain` (only the authorized files; if anything else shows, you violated scope — STOP and say so).
6. Any honest delta / surprise.

Do NOT commit. The orchestrator re-casts the divergent spells (struere, solvere, sequi, intueri, conformare, purgare, circumspicere) on your dirty tree; converged L1+L2=0 → hashless ISO8601 vigilatum stamp + ONE atomic ward commit = the 6th and FINAL warded home.

---

## Shape + calibration (mirror the proven cadence)

- **Prior comparable:** this is the SAME shape as the `remedy/` ward sweep (BRIEF-remedy-ward-R5/R6). Mirror that SCORE structure: files-touched → per-fix disposition with line(s) → gate counts → cascade check → dirty-set → honest delta. The remedy ward's lead fix was also a make-illegal-states-impossible type change (`Typo(u32)` → `Typo(NonZeroU32)`); Fix 1 here (remove `Clone`) is the same failure-engineering move at the transport layer.
- **Baseline (orchestrator-verified, pre-strike):** `cargo test -p wat --test comms` = **33 passed / 0 failed**; `cargo test -p wat --lib` = **895 passed / 0 failed / 1 ignored**; tree clean. After Fix 1 retires `probe_slice3d1_sender_clone_shares_write_end`, comms drops to **32** — that is the ONE expected count change; any OTHER delta is unexplained and a STOP.
- **Nature of the work:** NOT new-mechanism territory. Fix 1 + Fix 5 are type/enum reshapes (compiler-guided cascade); Fixes 2/3 are control-flow guards; Fixes 4/9/10 are doc-truth; Fix 6 is additive `Display`; Fix 7 is fd-lifecycle + EINTR retry (mirror `send()`'s existing EINTR loop); Fix 8 is a deletion. All sites are named with `file:line` above — no hunting.
- **Calibration band:** ~20-35 min (10 fixes, ~3 files, mostly mechanical). If you pass ~60 min or hit a red you cannot resolve, STOP and report rather than thrash — the failure is data.
- **Substrate-as-teacher:** removing `Clone` (Fix 1) + the `LinkedList` impl (Fix 8) will produce compiler errors at any hidden caller. The fail-count is the progress meter; each error names the next site. BUT all such sites must be inside the authorized scope (`src/comms/` + the one runtime.rs fd fix + the retired test) — the orchestrator's crawl found NO callers outside it. If the compiler names a site OUTSIDE scope, the crawl was wrong → STOP and report it; do not chase it.
