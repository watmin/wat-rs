# Arc 170 — CLOSURE BACKLOG

Six tracked items. Opened 2026-07-28, at the builder's direction, after the
"stopping is a protocol" stone landed green (lock-step restored; the stop asks
each service, awaits `Status::Stopped`, and severs last).

**Board state (2026-07-28, grounded against the disk at `db7cad6a`): items 2, 3
and 5 are CLOSED.** Three remain: **#4** (mechanical), **#6** (needs the
witness-not-claim wall), and **#1** — `wat --repl`, the closure condition.

**Scope ruling (builder, 2026-07-28):** items 1–5 below **must be addressed**.
Three further known-owed items were surfaced at the same time and **deliberately
NOT tracked here**: the stale CLIFFNOTES "Currently" block, the untaken
perf/behaviour measurement of the exec'd path, and arc 170's INSCRIPTION itself.
Their absence from this file is a decision, not an oversight.

**What is already closed and must NOT be re-derived:**
- The **fork bug / execve crusade** — dead at 24v. Spawned runtimes genuinely
  exec; steps 2d/3/4/5 banked; the argv probe is green.
- The **stopping stone** — the stop is a protocol, not a sever. Floor 4105/4105/0
  by the orchestrator's own `--release` re-run, both sigterm acceptance tests
  passing.

All counts below were **measured 2026-07-28**. Counts drift; re-ground before
striking. A grep that cannot reach the thing is not evidence of absence —
check that the pattern COULD have matched.

---

## 1. `wat --repl` as a CLI mode — THE CLOSURE CONDITION

**Status:** not shipped.

Arc 170 closes on a REPL, by the builder's ruling — not on the fork bug. The
REPL **exists** (`wat-scripts/demos/repl/repl.wat`) and runs, but only as a
script. There is no `--repl` mode.

**Grounded:** `src/distribution/argv.rs:33` mentions `--repl` only in a *comment*,
describing the per-mode arity refactor that makes a new mode possible:

> *"different contracts; giving each mode its own means a new mode (`--repl`, …)"*

The arity gate that used to block it (`positional.len() != 1`, arc 115) is
already fixed — each `Mode` now owns its own arity (`Check{…}` exactly one,
`Run{…}` at least one). So the door is open; nothing walked through it.

**Why it is the closure condition, in the builder's words:** *"i think we just
ship `wat --repl` so it can access the privileged tooling? that … kinda proves
the demo isn't a demo."*

**Done looks like:** `--repl` is a `Mode` variant; `wat --repl` starts the REPL
with privileged tooling reachable; a gate proves it.

---

## 2. `readln` raises on a stop — needs an outcome-returning signature

**Status: ✅ CLOSED** — `591adcdf` (the wall + the 77-file corpus migration),
`ac64e67e` (the must-use twin gate: a dropped `ReadlnOutcome` is a compile error
in **both** discard doors), `f7d390ce` (the three readers' arms refined).

`:wat::kernel::ReadlnOutcome<T>::{Datum[v], Eof, Stopped}` is registered
(`src/types.rs:1097`); `infer_kernel_readln_prime` returns it; the head joins
`MUST_USE_PARAMETRIC_HEADS` with a verb-aware remedy naming `Datum/Eof/Stopped`
(without it the fall-through taught `Sent/Closed/Lost` — the *send* wall's arms —
on a readln). The migration rode a recorded codemod,
`wat-scripts/fixes/readln-to-outcome.wat`, collapsed onto the **new generic**
`:wat::fix::wrap-calls-in-match` (see "What `fix.wat` gained", below).

**Ruled by the builder, and it shaped the migration:** *"my preference is we
shield ourselves with verbosity — all code paths are immediately obvious at the
call site."* So **no `expect` sugar** — every one of the 77 sites faces all three
variants. Reaching for a sugar to shrink the site count is a difficulty argument,
and difficulty is not a design axis.

*(Historical, the condition this closed:)* `readln`'s callers used to **raise**
when a stop was requested mid-read. The message
is honest; the shape is wrong. A stop is a matchable value, not a raise that
flees past the reader (R53 `VERBO MEO CAPTVS` — the recv' wall's whole point).

**Grounded:** 87 non-comment `readln` call sites across `wat/`, `wat-tests/`,
`tests/`, `wat-scripts/` (measured with comment lines stripped). Note the shape:
`readln` is a **defmacro** that expands into the kernel-restricted positional
prime `readln'` (`src/check.rs:2670-2674`) — so the migration touches the macro's
lowering, not only the call sites.

**Precedent to copy:** `read-frame` already does this correctly — raw text in,
EOF as a matchable value, `Stopped` as a named variant. The capability was
already banked behind `stdio-read`; a REPL was the first caller that needed it.

**Done looks like:** `readln` returns an outcome its callers must face; a stop is
a variant, not a raise; the corpus migrated by codemod.

---

## 3. `LociDiedError::Shutdown` → `Stopped`

**Status: ✅ CLOSED** — `ff775663` (landed the same day this file was opened, at
`03de6d44`; this entry was never updated). `:wat::kernel::LociDiedError` now
carries `Stopped` (`src/types.rs:1289`), and the boundary held: Rust keeps
`shutdown` uniformly.

The wat-visible layer says **stopped**, not **shutdown** — ruled by the arc-170
intueri cast (`BRIEF-stopped-not-shutdown-rename.md`, RULING A): nothing is
shutting down when this fires; a stop was *requested* and the program decides.
`(:wat::kernel::stopped?)` already owns the word. This variant is the last
wat-visible holdout still wearing the Rust vocabulary.

**Grounded — 16 sites across 8 files** (the 24y seam estimated ~8; it is 16):

```
src/comms/mod.rs
src/runtime.rs
src/kernel/spawn.rs
wat/kernel/services/stdio-primes.wat
tests/comms/probe_arc209_structured_peer_death.wat
tests/comms/wat_arc113_raise_round_trip.wat
tests/diagnostics/probe_runtime_error_produces_structured_edn.wat
tests/diagnostics/probe_plain_panic_produces_structured_edn.wat
```

**The boundary, already ruled:** the rename stops at the Rust/wat line. Rust keeps
`shutdown` uniformly (`RecvError::Shutdown`, `trigger_shutdown`,
`SHUTDOWN_BROADCAST_READ_FD`); the wat-visible variant becomes `Stopped`. That
boundary IS the audience boundary — see the WHY comment already written at
`src/io.rs:1026-1034`.

**Done looks like:** a recorded `wat-scripts/fixes/*.wat` codemod, dry-run +
diffed on a `/tmp` copy, idempotent, applied to every listed path.

---

## 4. `StdIn::ReadLineResponse` — `read-line` / `:Line` naming

**Status:** open. Names only; no behaviour change.

A **frame** can span several physical lines, so `read-line` and the `:Line`
variant mumble about what they carry. Flagged at the intueri rename and
explicitly held out of that brief's scope.

**⚠ THE PATH MOVED.** `stdio-primes.wat` → **`wat/kernel/services/stdio.wat`**
(`6e800f12` — the primes replaced the non-primes months ago; the filename was out
of phase). The stdin half also moved: `services/stdin.wat` →
**`wat/kernel/readln.wat`**, because that is what the file actually was — a
`readln` macro and a constant, not a service.

**Grounded scope: 59 sites / 5 files, all internal.** No user surface moves —
`readln`/`println`/`eprintln` stay (ruled by the builder: *"that's the user
interface for working with stdio — what we do under the hood i'm less concerned
about"*). `:wat::fix::rename-keyword-exact` already exists.

**The lie is in code, not just in a name:** `stdio.wat:114-121` calls
`IOReader/read-frame` and relabels the result `::Line`. A frame relabelled as a
line.

**⚠ HEED 24t — a rename touches FIVE surfaces and a codemod reaches one and a
half:** `.wat` keywords · keywords built inside STRING literals · the other
`.wat.*` extensions (a `-name '*.wat'` glob silently excluded 243 files last
time) · `src/**/*.rs` literals in BOTH the colon-prefixed and bare-parametric
spellings · `tests/**/*.rs` goldens. That is what took 2530 → 20 → 3 → 0.
**ENUMERATE EXTENSIONS; never one glob.**

**Done looks like:** intueri cast on the verb + variant names; the ruling applied
across `wat/kernel/services/stdio.wat` and consumers, all five surfaces reached.

---

## 5. `as_raw_fd_for_poll` on `WatWriter` has no poll caller

**Status: ✅ CLOSED** — `2bf589c8`. **The framing above was wrong in both
directions, and grounding settled it: it was neither a gap nor dead surface.**

`as_raw_fd_for_poll` has a **live consumer** — `src/freeze.rs:269-271` seeds the
three stdio services from it. It is a hook whose *poll* consumer was never built.
So the answer was: **build it, and the name becomes true.** `PipeWriter::write`
now polls `[fd, broadcast_fd]` before every attempt; a blocked write can be
stopped. Gate: `tests/channel/probe_arc170_writer_joins_lockstep.rs` — 3.006s
(hung to the timeout) → 0.005s.

**⚠ THE LESSON, and it cost a regression:** my brief's constraint said *"shutdown
wins ties,"* lifted from the **reader** without asking whether it transfers. It
does not. A READ racing a stop has nothing left to read; a **WRITE racing a stop
may be the dying declaration** (R51 — `eprintln` is "the last thing I'll say").
Preferring shutdown made a stopped process unable to say why it stopped, and
regressed `wat_cli::sigterm_reaches_a_program_blocked_on_stdin` 0 → 1. Proven
mine by a **stash differential** (passes without `src/io.rs`, fails with it) and
reproduced **isolated**, which killed the load-flake hypothesis before it could
comfort me. Corrected to the **opposite tie-break**: if writable now, write;
surface the stop only when the write *would* block.

**A rule copied from a sibling subsystem owes the same grounding as a rule
invented from scratch.**

**⛔ AND IT LEFT A DECISION OWED — see "THE ONE DECISION", below.** The `#5` strike
deliberately did NOT distinguish a stopped write from a disconnect;
`src/channel/transfer.rs:87-100` carries its own tracking comment saying so.

---

## 6. Spawned procs identify themselves in `ps`

**Status:** designed 2026-07-28, not built. Added at the builder's direction.

Today `ps` shows N identical `wat` processes with no way to tell a bracket worker
from a service. Each spawned runtime should carry its own identity on the proc
line, as the EDN it already is:

```
/usr/local/bin/wat #wat.brackets/Worker {:id 3}
/usr/local/bin/wat #my.app/CounterSvc {}
```

**FIXED AT BOOT — builder-ruled:** *"these would be fixed at boot — the procs are
purpose built."* Identity, not state. Set once, immutable for the process
lifetime. This is the cheap version: **no `setproctitle`-style argv-memory
rewriting**, which exists for daemons that fork *without* exec and is bounded by
the original argv+envp region. Every spawned runtime execve's (24v), so argv at
exec time IS the whole proc line — we already have total control of it.

**The shape is forced, not chosen.** The uniform EDN rule — record → `{field-map}`,
enum variant → `[field-vec]`, `nil` → the unit value only — means a no-field
record renders `{}` by construction. So a service is `#ns/Svc {}` and a bracket
worker is `#wat.brackets/Worker {:id N}`. Consequence worth keeping: `ps` output
is readable by `edn::read`, not only by eyes.

**Where it goes:** `src/process/exec_plan.rs:113` already builds argv parent-side
(`vec![exe.clone()]`). The no-allocation rule is scoped to `exec_in_child`, NOT
to `build()` — the module doc: *"Every byte the child needs is built here, in the
parent, before the clone."* So the label is legal exactly where it belongs.

**⚠ THE WALL THAT MUST BE WRITTEN — it is a LABEL, never a CLAIM.** `exec_plan.rs:29-35`
rejects a `--forms-server` flag precisely because it would be *"typeable at a
shell and visible in `ps`"* and *"a CLAIM where this is a WITNESS."* That
objection is to a flag that **routes**. This label only **describes**: fd 3
(`LIFELINE_FD`) remains the sole gate, and forging the label at a shell must
accomplish exactly nothing. Write that invariant into the code, and gate it — a
test that a child ignores the label entirely. Without the wall it rots into a
flag the first time parsing it looks convenient.

**⚠ GROUND BEFORE STRIKING:** does the label leak into `(:wat::runtime::argv)`?
`exec_plan.rs:104` states *"argv is `[exe]` and nothing more. A spawned runtime
takes no command line"*, and 24v measured `CHILD-ARGV-LEN 0` — suggesting the
child's wat-visible argv comes from the boot handshake, not OS argv. **Verify
this.** If the ambient reads OS argv instead, every spawned child suddenly sees a
substrate token in its `argv` — a user-visible semantic change, not cosmetic, and
a different decision.

**The real work is plumbing, not the exec.** The label's content lives with the
*spawner*, which knows what it is making; `ExecPlan::build()` takes nothing today.
Threading the identity down is the change.

**Done looks like:** `ps` distinguishes every spawned runtime; the label is
registered EDN; nothing reads it; a gate proves a child ignores it.

---

## ⛔ THE ONE DECISION — owed to the builder, deliberately not settled alone

**Can a stopped write be told apart from a disconnect, and where does it land?**

`src/channel/transfer.rs:93` is `Err(_) => SendOutcome::Disconnected` — a wildcard
that erases every distinct write failure into one variant. That is exactly the
class 24x named at `src/kernel/peer.rs:118`, two hundred lines from where the
reader now does it right.

Killing it needs a `SendOutcome::Shutdown`, which cascades into
`src/kernel/address.rs:139` — a match over `{Ok, Disconnected}` with no wildcard,
whose `ConnectFail` offers only:

- `Refused` — retryable, *"the server may come up"*
- `Rejected` — identity mismatch, not retryable
- `Failed` — an io error carrying its reason

**None of those honestly means "this process is stopping."** Extending it reaches
the wat-facing `ConnectOutcome` (the arc-278 `connect'` wall).

**The cost, so it is visible:** a stop reaching `address.rs` today reports
`Refused` — telling a dialer to **retry a process that is shutting down** — in
the very arm whose own comment gets the outcome-wall discipline right. Not a
regression (every write failure already landed there), but a live lie.

## What `wat/fix.wat` gained — so the next wall is cheap

**The wrap family**, lifted during #2 because four codemods had hand-copied the
same ~60 lines (two admit it in their headers). Now first-class:

`Edit` (the span-splice tuple, previously spelled out 45× in that file) ·
`kw-name` · `head-name` · `calls-to?` (**EXACT** — a prime is never read as a
non-prime) · `node-start-offset` / `node-end-offset` · `arm-head-name` (tagged
AND bare-keyword unit patterns) · `arm-heads-contain?` · `wrapped-in-match?` ·
**`wrap-calls-in-match`** as the entry point.

A new outcome wall's codemod is now a header plus one call. Proven
**byte-identical** to the hand-rolled codemod it replaced. ⚠ Adding a verb here
re-bakes the stdlib — `cargo build --release` before any codemod sees it. ⚠ A
typealias RHS needs its leading colon: `:(A,B,C)`, or `(A,B,C)` parses as
`A<B,C>`.

**Cleanup owed, any time:** retrofit `wrap-client-method-match-in-recvoutcome`,
`wrap-connect-prime-in-connectoutcome` and `read-string-to-outcome` onto the
generic. Prove each with the same byte-identical differential.

## Method, standing

- Weigh by your **OWN** `cargo nextest run --release` — read the Summary line by
  hand, ANSI-stripped; never a piped or wrapped exit code.
- `.wat` corpus migrations are **wat-fix codemods**, never hand-edits, never
  python/sed. Dry-run on a `/tmp` copy + `diff` + prove idempotency first.
- Names are **cast** (intueri), never narrated.
- The orchestrator designs, briefs, delegates, and weighs. One rider runs its own
  tests; a fleet does not.
