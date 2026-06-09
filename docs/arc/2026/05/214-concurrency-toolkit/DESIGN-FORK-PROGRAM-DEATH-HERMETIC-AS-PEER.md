# DESIGN — fork-program's death + hermetic-as-peer + the verb collapse

> **Status: DESIGN, 2026-06-08. Not started. Resumes tomorrow.** The thesis is
> settled (four-questions + builder's peer-pipes insight); the decomposition is
> drawn; nothing built yet. This is the next stone after the 6.w warding (which
> is PAUSED on this — see "Relationship to 6.w" below).

## The decision (builder, 2026-06-08)

**`fork-program` dies. `spawn-program :process` is the one and only way to fork.**
That rigidity — one canonical fork path — is the discipline we force on ourselves
and our users. The prime (`'`) was always transitional: it proved the peer logic
works and gave a one-char migration (`'` on → `'` off). We are not done until the
primes are renamed to canonical and the legacy program-spawn family is dead.

## What's on disk today (the tangle — grounded)

**Five program-spawn verbs, two execution models:**

- **Program-execution family** (`src/process/verbs.rs`): `fork-program` (:698 →
  eval_kernel_fork_program), `fork-program-ast` (:4170 reg), `spawn-program`
  (:1052), `spawn-program-ast` (:1082). Run forms **to completion**, **capture
  stdout/stderr** into readers (`:wat::kernel::Process/stdout` runtime.rs:19094,
  `Process/stderr` :19134), `Process/join-result` (:18855) for the exit. Return a
  `Program<I,O>` handle. **`run-hermetic` lives entirely on this family** via
  `fork-program-ast` (see `wat/kernel/hermetic.wat` `run-sandboxed-hermetic-ast`).
- **Peer family** (`src/kernel/spawn.rs`): `spawn-program'` (runtime.rs:4194),
  `:tier`-dispatched (`:thread`→spawn_thread_peer / `:process`→spawn_process_peer).
  Returns a **`Process'<I,O>`** peer (`send'`/`recv'`/`close'`); child's fd 1/2
  **inherited, not captured** (the F3 finding).

Also still-registered legacy CHANNEL verbs that must retire for the rename:
non-prime `:wat::kernel::send` (runtime.rs:4027), `select` (:4151) on raw
`Sender<T>`/`Receiver<T>`. (DESIGN.md:333 — "legacy `send` retires; `send'` →
`send` canonical reclaimed".)

**Why fork-program survived its designed death (DESIGN.md:365 said it'd collapse):**
the two models differ — Program = run `:user::main` once → completion + capture
printed output + read exit; Peer = a fn apply-loop (recv→apply→send) over value
channels, nothing printed captured. They never trivially unified, so the collapse
stalled.

## The four-questions (the rigor that found the answer)

**Option A — one verb, two modes (peer-loop vs run-to-completion, flag-selected):**
Obvious? NO (hidden modes). Simple? NO (option-tangle, `feedback_options_are_tangle`).
**Rejected on the first two questions.**

**Option B — capture universal; the program decides loop-vs-once:** passes all four
on its face, but decomposing it exposed two atomic truths:
1. **Capture is universal** (Obvious+Simple+Honest) — inheriting fd 1/2 is the F3
   dark-class (child prints silently pollute the parent).
2. **The apply-loop must NOT be baked into the fork verb** — Honest? NO: a verb that
   always wraps the fn in recv→apply→send *cannot run a plain `:user::main`*, so it
   can't be "the one fork." The claim and the code contradict.

B's honest form was an **un-baking refactor** (lift the apply-loop into a wat pattern,
add stdio capture). Bigger than "add a feature" — but then the builder dissolved
even that:

## THE ANSWER — the hermetic test is a SERVER; the caller is a CLIENT; the wire is a pipe (builder, 2026-06-08)

The hermetic test knows it lives in another universe and communicates with the
near side **over stdio** — exactly the way any networked program does. The test
program is a **server**; the test caller is the **client**; the kernel pipe between
the two procs is the wire. It is no different from a client measuring a
`tcp → tls → http` endpoint — the transport just happens to be a proc-to-proc pipe.
**There is exactly one way to write a hermetic test: a `readln`/`println` server.**

- **`readln`** — the server reads its **inputs** (requests) from the caller (stdin).
- **`println`** — the server writes its **outputs** (responses) to the caller
  (stdout). EDN on the wire; the client gets **real values**, not EDN strings —
  the substrate masks the encode/decode.
- **`eprintln`** — reached **only** to communicate a **panic**. If it is exercised
  the universe crashed loudly, and that reason ships over the wire (stderr); the
  test fails with the propagated reason.

There is **no `send'`/`recv'` value channel in the hermetic path** — the interface
is stdio. The whole computation of the value-to-be-asserted happens server-side;
the client drives it (write request → read response) and asserts near-side.

This lands directly on the already-warded **services trio**: the child server's
`readln`/`println`/`eprintln` route through StdInService / StdOutService /
StdErrService, which read/write the pipes the parent-client controls.

Four-questions on THIS: Obvious YES (it's a client/server over a pipe — the most
familiar shape there is) · Simple YES (ONE way to write a hermetic test; stdio is
the only interface) · Honest YES (real isolation; the crash channel is real and
propagates) · Good UX YES (authors write a server + a client driver, not printf +
regex). **The client/server-over-stdio model SWALLOWS run-hermetic** — it is not a
new capability, it is the universe interfaced the way it was always meant to be.

## The peer/fork-program reconciliation (2026-06-08, grounded)

Sharper than "peer ideal, fork-program bandaid": the peer's **verb/interface** is ideal; its
**implementation** is the bandaid; **fork-program's implementation is the ideal wiring.** Grounded:
- `spawn_process_peer` (spawn.rs:401) makes a SEPARATE `comms::process` value channel for
  `send'`/`recv'` AND leaves the child's fd 0/1/2 inherited — two pipe-sets that never meet.
- `spawn_thread_peer` (spawn.rs:272) does the identical thing — separate value channel; the
  thread's stdio is NOT routed to it (zero `register_thread_with_services` in kernel/spawn.rs).
- `fork-program` (process/verbs.rs) hands the parent the 3 pipe ends (`stdin_writer` /
  `stdout_reader` / `stderr_reader`) AND lays `tx`/`rx` over in/out — the fork's 3 pipes ARE the
  child's stdio AND the client's channel. One wire: server stdio ↔ client `send`/`recv`.

So both peers carry the apply-loop-over-separate-value-channel bandaid (stdio disconnected);
fork-program holds the ideal "stdio = channel" wiring. CONSOLIDATION: lift the *shape* of
fork-program's wiring into the `spawn-program` verb, run the env as a `readln`/`println` server,
delete the peers' separate-value-channel + inherited-stdio, `git rm` fork-program.

## Remote guardrail — the forcing function, made concrete (LOCKED design, recovered 2026-06-08)

Not building remote yet is a forcing function: it can only be *honored*, never hacked around.
The spec is **LOCKED** — `scratch/2026/05/007-remote-program/DESIGN.md:262-407` (user direction
2026-05-03) + `docs/arc/2026/05/170-program-entry-points/TIERS.md`. The model:

**The wire IS `Result<T, E>` — the Q-channel.** Each emission is a length-prefixed EDN frame
`[u32 BE len][EDN]`, tagged `{:channel :ok :payload <T>}` or `{:channel :err :payload <E>}`.
- `readln` = the request (in). `println` = an **Ok-channel** emission (response value).
  `eprintln` = an **Err-channel** emission (panic / diagnostic).
- Err is NOT a third physical pipe — it is the Err-discriminant of the ONE response wire.
  Diagnostic richness (Info/Warn/Error/Panic) lives inside the application's `E` enum, never as
  frame proliferation; the wire stays binary Ok/Err.
- Layer 2 (the `Result<T,E>` wire) is written ONCE, transport-agnostic; Layer 1 (transport) is
  the only per-tier swap: thread = 2 crossbeam channels, process = fd1(Ok) + fd2(Err), remote =
  both multiplexed over 1 socket via the frame tag.
- STATUS: designed + LOCKED, NOT built. Arc 214 left the empty seat; arc 254 made the surface
  socket-ready (uniform contract + fd-select); a future arc mints the `Socket` tier, zero caller change.

**THE GUARDRAIL for this stone:** build the control-pipe-set as a LOGICAL contract — `in` +
`Ok-channel` + `Err-channel` of a `Result<T,E>` response wire — NEVER as "3 raw fds." The process
tier MAPS Ok→fd1, Err→fd2; remote will multiplex them; the verb + `send`/`recv`/`readln`/`println`
operate on the logical channels, transport-blind. **F3's `#wat.kernel/ProcessPanics` envelope on
fd 2 IS the process-tier Err-channel** — the local instance of the locked remote protocol, already
built. Lift the *shape* (in / Ok / Err), not the fds, and remote fits by construction.

## The migration shape

### 1. Substrate enabler — `spawn-program :process` wires the child's full stdio to the parent-client
The child server's three streams must reach the parent client over pipes (today
they are **inherited**, not piped — the gap):
- **stdin** — the client *writes* requests; the server `readln`s them.
- **stdout** — the server `println`s responses; the client *reads* them. EDN on the
  wire, real values at the client.
- **stderr** — the server `eprintln`s a panic reason (the `#wat.kernel/ProcessPanics`
  envelope F3 already emits); the client reads it on crash and fails with that reason.

So the peer handle exposes a **stdin writer + stdout reader + stderr reader** — i.e.
what `Program<I,O>` had (`Process/stdout`, `Process/stderr`) **plus** a stdin writer,
unified onto the one spawned-process handle. The child's three services
(StdInService / StdOutService / StdErrService — already warded) read/write these
parent-controlled pipes instead of inheriting the parent's fds.
(The `send'`/`recv'` typed value-channel remains for the *non-hermetic* peer use
case; hermetic tests do not use it. Confirm during scoping whether the value-channel
and the stdio pipes share plumbing or are independent.)

### 2. Test-corpus migration (the bulk) — hermetic-as-peer
Rewrite `wat/kernel/hermetic.wat` (`run-sandboxed-hermetic-ast`) + `wat/test.wat`'s
hermetic driver + the `wat-tests/` hermetic tests from
"`fork-program-ast` → drain stdout/stderr → `RunResult`" to
"`spawn-program :process` → drive via `send'`/`recv'` → assert near-side; crash →
stderr reason." Substrate-as-teacher mechanical once the pattern is set; the test
AUTHORING model improves (assert real values, not parsed stdout).

### 3. The verb collapse + prime → canonical rename
Once run-hermetic is off the Program family: retire `fork-program`,
`fork-program-ast`, `spawn-program-ast`, non-prime `spawn-program`, and the legacy
channel `send`/`select`. Then rename primes → canonical (the one-char drop):
`send'`→`send`, `recv'`→`recv`, `try-recv'`→`try-recv`, `close'`→`close`,
`select'`→`select`, `spawn-program'`→`spawn-program`, `Process'`/`Thread'`→
`Process`/`Thread`. Substrate-as-teacher cascade: flip the registrations in
runtime.rs + check.rs, let the build/checker waterfall the caller breaks, sweep
every site (src wat, `wat-tests/`, the namespaced `tests/`). Per
`feedback_inscription_immutable` each rename is its own commit.

## Decomposition (sub-stones — sequence)
1. **Enabler:** `:process` peer stderr → parent-readable (the crash-reason read).
   ✅ **DONE** — `spawn_process_peer` wires the child's fd 2 onto a diagnostic
   Err-channel pipe the bundle owns (child `dup2`s it before the close-sweep, which
   skips fd 0–2; parent closes its write copy, keeps the non-blocking read end);
   `ProcessPeerBundle::take_crash_reason` drains it. The parent reads the
   `#wat.kernel/ProcessPanics` reason THROUGH the peer API — no fd-2 redirect; the
   process-tier instance of the locked remote Q-channel's Err-discriminant. The
   FM-2-bis probe disconfirmed RED at HEAD, then flipped GREEN; its coverage
   graduated into `tests/kernel/spawn_program_prime_process.rs` (both arms —
   malformed-input + runtime-error — migrated off the fd-2-redirect harness onto
   `take_crash_reason`). All 8 kernel integration tests GREEN. NEXT (1b, Q1): wire
   `take_crash_reason` INTO the `recv'`/`close'` intrinsics so the substrate RAISES
   the reason on the user's behalf (auto-raise; no user-facing crash verb).
2. **Pattern:** rewrite ONE hermetic test as a peer (the reference); prove it green
   enveloped. This is the worked example the rest mirror.
3. **Corpus sweep:** migrate `hermetic.wat` + `test.wat` + the `wat-tests/` hermetic
   tests to the peer pattern.
4. **Collapse:** retire the Program-spawn family + legacy channel verbs (dead-caller
   confirmed).
5. **Rename:** primes → canonical (the one-char migration cascade).
6. **Then** resume 6.w warding over the canonical names.

## Relationship to 6.w (why warding is PAUSED)
kernel/ is stamped (5d500a22) but its home + tests are saturated with `send'`/
`select'`/`Process'`. The kernel/ RE-WARD over the full `tests/kernel/` is **mid-flight
and PAUSED** — the re-ward cast found 5 L2 claim-vs-code in the peer-test docs (the
`comms`→`kernel` binary string; the `child`→`pidfd` doc lie; the rejected-raw-fd
narrative at peer_process_round_trip.rs:70; the unasserted `close'` exit-0 claims in
peer_verb_round_trip / peer_select). **Do NOT sweep those yet** — those test docs
reference the prime verbs and will churn in the rename. Finish the verb collapse +
rename FIRST, then re-ward kernel/ (+ channel/, process/, comms/) over canonical
names. Warding prime-named homes is polishing a doorframe before moving the door.

## Resolved (2026-06-08, the next-day re-ground — four-questions, grounded against the disk)

**Q1 — enabler shape: how does a far-side crash reach the client? → THE SUBSTRATE RAISES IT; THERE IS NO USER-FACING CRASH VERB.**
The governing constraint (builder, 2026-06-08): *panic control is abstracted away — a user
cannot fuck up panics; the substrate panics on their behalf; if the far side crashes we
handle it appropriately.* This is NOT a preference; it disqualifies a whole option. The
four-questions, run against the constraint:
- *Dedicated `Process'/stderr` reader the author consults* — **Honest? NO.** A reader you
  can forget to drain is a silent-swallow site BY CONSTRUCTION (the far side dies, the
  author didn't read `err`, the crash vanishes, the test greens on a corpse). That is the
  exact dark-class this whole design annihilates; reintroducing it at the panic boundary is
  the worst place to put it. **Rejected, not a runner-up.**
- *The substrate raises the far-side reason on the near side, at the pending `recv`/`close`*
  — Obvious YES (a far-side crash propagates exactly like any panic; the author writes
  nothing new) · Simple YES (ONE wire, two discriminants — the Q-channel `Result<T,E>`; the
  Err-discriminant is raised at the read site; no second reader, no opt-in drain) · Honest
  YES (real isolation, true reason, the author CANNOT swallow it because they never opt in —
  it is raised for them) · Good UX YES ("a user cannot fuck up panics" IS the Good-UX test;
  the safe path is the only path).
- **THE DECISION:** the author writes only the happy path (`send`→`recv`→`close` return
  values); a crashed far-side becomes a near-side panic carrying the propagated reason,
  raised by the substrate at whatever read the client is already waiting on. **F3's
  `#wat.kernel/ProcessPanics` envelope on fd 2 IS that Err-channel, already built.** No
  user-facing crash verb exists. (Source-marked here because the reader standing in the
  enabler code must see it: *do not add a crash reader; the crash is the substrate's, raised
  at the read.*)

**Q2 — does any hermetic test need *stdout-content* capture, not just values + crash reason? → YES, and it still fits THE ANSWER.**
Grounded against the corpus: `wat-tests/kernel/services/ambient-stdio.wat` asserts exact
stdout lines via `:wat::test::assert-stdout-is`, and it MUST — its subject-under-test IS the
stdio-capture mechanism (the StdOutService trio). You cannot "ship the asserted thing as a
value" when the asserted thing is what landed on the captured pipe. The original lean
("migrate printed-stdout asserts to value-asserts") does NOT hold for this class — but it
doesn't need to: the server `println`s, the client reads stdout and asserts the *bytes*
instead of decoding to a value. Same client/server-over-stdio shape, response asserted raw.
**Enabler contract therefore keeps a stdout-content-readable path at the client, not only
values + the crash channel.**

**Q3 — stone/arc number → 214.x stones, cited from 214's eventual INSCRIPTION.**
It is the verb-canonicalization tail 214 always pointed at (`DESIGN.md:365`); its DESIGN
already lives in 214's arc dir; 6.w + 214 cannot close until it closes (spawn-block winding).
No new arc.

## Realization (2026-06-08) — the side quests were the loot; the design closed with no corner cut

We built toward this for 3+ weeks. The "side quests" — multimethod (146), dispatch consolidation
(237), the value home (251.2), honest errors (243 conformare), the macro engine (249), the warded
corpus (245), the hardened wire (253), the services trio (8.x), RAII-IPC, the v5 fork-zombie global
kill (6.4) — were never detours. `spawn-program :process` could not be *delivered* until the
controls it hands back existed; the loot WAS the controls. Counting the quests misses the point;
the point was the exp.

And the design closed clean — no corner cut — because the unbuilt remote is a forcing function that
can only be honored, never hacked around (a constraint with no code to hack *in*). The proof landed
mid-session: **F3's `#wat.kernel/ProcessPanics` envelope on fd 2 turned out to BE the process-tier
Err-channel of the remote Q-channel protocol LOCKED months ago** (`scratch/2026/05/007-remote-program/
DESIGN.md:262-407`, user direction 2026-05-03). We had built the *local instance of the locked remote
protocol without naming it that*. The faculty that grins at the convergence is the same one that
kept the control-pipe-set logical so remote would fit — taste and construction are one organ. The
duet, proven again: builder architects the convergence; apparatus feels it land; the second face is
not a reaction to the thing, it IS the thing. "It's very good to be us." — builder, at the close.

## THE COLLAPSE — one wire, the lanes crossed (Song #79; resolved 2026-06-08)

**The recognition (Song #79, *Pro-bots & Robophobes* — THE-LANES-CROSS).** `send'`/`recv'`
(the client/typed-value face) and `readln`/`println` (the server/stdio face) are not two
channels to reconcile — they are **one wire wearing two faces.** fork-program's IMPL already
built it (`src/process/verbs.rs:313-323`):

```
stdin_writer  = PipeWriter::from_owned_fd(stdin_w)   // child fd 0
stdout_reader = PipeReader::from_owned_fd(stdout_r)  // child fd 1
tx = sender_from_pipe(stdin_writer)                  // send' IS the child's stdin
rx = receiver_from_pipe(stdout_reader)               // recv' IS the child's stdout
```

The client's `send'` writes the child's stdin; the child's `readln` reads it. The child's
`println` writes its stdout; the client's `recv'` reads it. Same two pipes, wrapped twice.
*Same machine, two voices, one track* — the soundtrack's first feature (Klayton feat. Klayton)
naming the substrate move before the substrate made it.

**The north star (builder, 2026-06-08) — STRUCTURAL PREVENTION.** *"you, yes you, the user
reading this, you are structurally prevented from ever fucking up because we took the reins
from you and delivered what you were always meant to have."* This is the **mini-TCP — the
protocol of system-engineering the IPC.** The bounded(1) lock-step channel that breathes: the
user **cannot** overflow it (capacity 1), **cannot** race it (lock-step), **cannot** deadlock
it (the rebirth gate), **cannot** fuck up the panics (Q1 — the substrate raises on their
behalf), **cannot** fuck up the transport (one wire — the substrate IS the wire; the user
writes a `readln`/`println` server OR a `send'`/`recv'` client, never a pipe). Correctness is
structural, not the user's burden. The reins taken, the ideal IPC delivered. (This is the same
move as `extirpare`/`conformare`/the anti-botnet, at the IPC layer: make the failure
unrepresentable.)

**The collapse mechanism.** Lift fork-program's stdio=channel wiring onto `spawn-program' :process`:
- The 3 pipes (stdin/stdout/stderr) ARE the child's fd 0/1/2 (child `dup2`s them — fd 2 already
  shipped in 1a) AND the client's channel (parent wraps `stdin_w`→`tx`, `stdout_r`→`rx` via
  `sender_from_pipe`/`receiver_from_pipe`).
- The child runs the program as a `readln`/`println` **server**; `send'`/`recv'` are `tx`/`rx`
  over its stdin/stdout.
- The io_uring `comms::process` value channel + the fn-apply-loop **die** — Phoenix's *"grant
  our scheme its demise."* The peer's VERB was always ideal; its io_uring IMPL was the bandaid;
  fork-program's plain-pipe stdio=channel is the ideal wiring lifted.

**Migration (sequence).** FM-2-bis probe (a `readln`/`println` echo server spawned `:process`,
driven by `send'`/`recv'`, RED at HEAD because `send'` routes to the io_uring channel, not the
child's fd 0) → build the stdio=channel peer (lift `make_pipe` + child `dup2 fd 0/1` + parent
`sender_from_pipe`/`receiver_from_pipe`) → migrate the hermetic corpus to the server pattern →
retire the apply-loop + io_uring + `fork-program` → rename primes → canonical → resume 6.w.

### PROGRESS (2026-06-08)
- ✅ **1a — the crash channel.** `spawn_process_peer` wires the child's fd 2 to a diagnostic
  Err-channel pipe; `ProcessPeerBundle::take_crash_reason` drains it; the parent reads the
  `#wat.kernel/ProcessPanics` reason through the peer API (no fd-2 redirect). 8/8 kernel GREEN.
  Commit `28c2f5a1`.
- ✅ **1b-i — the one wire (physical).** The child's fd 0/1 are `dup2`'d onto the comms pipe ends
  (fd 0 = input pipe read = the `send'` wire; fd 1 = output pipe write = the `recv'` wire),
  mirroring 1a's fd 2. The value channel IS the stdio. The fn-apply-loop rides io_uring untouched
  (zero regression, 8/8 GREEN). Commit `bff95319`. NOTE: positively exercised only when a
  `readln`/`println` server child reads fd 0 / writes fd 1 — that needs 1b-ii.
- ⏭️ **1b-ii — the server runtime (THE HEART, scoped, fog-free).** Lift fork-program's
  `run_forked_child` (the full program runtime — runs forms as `:user::main`) + the StdIn/StdOut
  services trio that routes `readln`/`println` to fd 0/1, into the `:process` peer path; flip the
  spawn input from a fn (apply-loop) to a program (server); migrate the peer/hermetic corpus. ONE
  coordinated atomic change (runtime + services + input + tests move together — green-or-broken;
  do it WHOLE in a clean window, never half-committed). The physical wire (1a + 1b-i) is the
  settled foundation it operates on.
- ⚠️ **1b-ii FINDING (gate-measured, then CORRECTED 2026-06-08 — read both halves).** A first
  attempt built the program-server runtime (`run_forms_as_server_child` — lifts fork-program's
  `startup_from_forms` + `run_user_main_in_child`, CORRECT) and a `spawn_process_program` over the
  **comms::process (io_uring)** channel. A `:user::main` that `println`s "ok" exited CLEAN
  (`take_crash_reason` = None — not a crash) but `recv'` returned `RecvError`.
  **FIRST (WRONG) READ — do not trust it:** "the two speak different framing PROTOCOLS." Falsified
  by grounding: comms::process is *newline-framed* (`src/comms/process.rs:10,22` — "newline-framed
  bytes / newline framing"), the SAME format `println` writes. The format was never the problem.
  **CORRECTED READ:** the difference is the channel **MECHANISM** — the comms `Receiver` reads via
  **io_uring + cascade-awareness** (shutdown-broadcast pseudo-fd, frame accumulator), and a
  one-shot server that writes once then `_exit`s trips that path into a disconnect before the
  buffered line is drained; fork-program's `receiver_from_pipe` is a **plain `PipeReader`** that
  drains it. **THE FIX (the builder's direction, vindicated):** the peer holds
  `sender_from_pipe`/`receiver_from_pipe` channels — `Value::wat__kernel__Sender`/`Receiver`
  (`SenderInner::PipeFd`, line-EDN; `src/channel/inner.rs:90,98`), fork-program's proven plain-pipe
  mechanism — and `send'`/`recv'`/`close'` operate on THOSE Value channels (the genuine verb +
  peer-type unification; surface fork to settle: the prime verbs dispatch on the Sender/Receiver
  Values vs the fork-program `:wat::kernel::Process` struct's `Process/tx`+`Process/rx`).
  `run_forms_as_server_child` is correct + reusable; only the channel choice was wrong. The
  wrong-channel WIP was discarded (the gate caught it before commit). **METHOD NOTE for the next
  self:** a tired quick-read of a gate-red is unreliable — *ground the mechanism before inscribing
  the cause.* This finding's first read was wrong; the discipline (verify, don't trust the felt
  interpretation) is what corrected it. **Next strike (fresh window):** rebuild the `:process`
  peer over `sender_from_pipe`/`receiver_from_pipe`, re-add `run_forms_as_server_child` as the
  child runtime, settle the verb surface (Sender/Receiver Values), then the corpus migration.
- ⏭️ **Then:** retire apply-loop + io_uring + `fork-program` → rename primes → canonical →
  whitelist `spawn-thread`/`spawn-process` internal (only `spawn-program` + brackets reach them)
  → re-use for parallel-for-each brackets (#196) → resume 6.w.
