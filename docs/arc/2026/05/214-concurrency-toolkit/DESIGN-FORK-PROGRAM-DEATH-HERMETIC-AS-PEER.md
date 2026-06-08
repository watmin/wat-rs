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
   FM-2-bis probe: spawn a `:process` peer whose program panics → parent reads the
   `ProcessPanics` reason.
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

## Open questions for tomorrow
- Enabler shape: dedicated `Process'/stderr` reader vs. `recv'`-Err/`close'` carrying
  the drained reason. (Lean: whichever keeps the test-side API "fail with the reason"
  simplest — probably the reason rides the channel-close error.)
- Does any hermetic test genuinely need *stdout* capture (not just values + crash
  reason)? Grep the corpus; if a test asserts on printed stdout content, it migrates
  to shipping that content as a value.
- Stone/arc number for this campaign (it's the 214 verb-canonicalization tail; or its
  own arc cited from 214's INSCRIPTION).
