# Arc 170 — CLOSURE BACKLOG

Five tracked items. Opened 2026-07-28, at the builder's direction, after the
"stopping is a protocol" stone landed green (lock-step restored; the stop asks
each service, awaits `Status::Stopped`, and severs last).

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

**Status:** open. Its own stone.

`readln`'s callers still **raise** when a stop is requested mid-read. The message
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

**Status:** open. A wat-fix codemod.

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

**Done looks like:** intueri cast on the verb + variant names; the ruling applied
across `wat/kernel/services/stdio-primes.wat` and consumers.

---

## 5. `as_raw_fd_for_poll` on `WatWriter` has no poll caller

**Status:** open. Grounded as real, unfixed.

`RealStdin` now reports `Some(0)` so the stdin read joins the poll multiplex
(that landed with the stdin lock-step stone). The **writer** side declares a
pollable fd that nothing polls — a hook with no consumer.

**The question to settle before striking:** is this a *gap* (a writer that should
be in the multiplex and is not) or *dead surface* (a hook that should be
deleted)? Ground the writer's blocking behaviour under a stop before deciding —
do not assume either way. Ground liveness by the **writer**, never a doc comment.

**Done looks like:** either the writer joins the multiplex with a gate proving it,
or the hook is annihilated.

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

## Method, standing

- Weigh by your **OWN** `cargo nextest run --release` — read the Summary line by
  hand, ANSI-stripped; never a piped or wrapped exit code.
- `.wat` corpus migrations are **wat-fix codemods**, never hand-edits, never
  python/sed. Dry-run on a `/tmp` copy + `diff` + prove idempotency first.
- Names are **cast** (intueri), never narrated.
- The orchestrator designs, briefs, delegates, and weighs. One rider runs its own
  tests; a fleet does not.
