# DESIGN — C0b.3b provisioning + spawn hooks (co-designed 2026-06-13, builder + Inquisitor)

> Captures the decisions from the C0b.3b co-design so the three stones below don't re-derive
> them. The SO_PEERCRED security MODEL is locked in `DESIGN-STONE-C0b-SECURITY.md`; this note
> is the *mechanism + surface* design that implements it, plus two adjacent spawn-config
> features the discussion surfaced. **C0b.3b-a SHIPPED** (`e1227004`): `comms::process::peer_cred(fd)`
> reads the kernel-vouched `{pid,uid,gid}` — the primitive everything below consumes.

## The trust model (recap — why the grant is owner-side)

SO_PEERCRED's point: *the owner vouches*. A child cannot self-register (anyone can connect
and even truthfully report its pid; the service must only trust pids the **owner** authorized).
So: the **owner** holds the child's pid (from spawn) and conveys "authorize pid X" to the
service; the **service** enforces at accept (one `getsockopt(SO_PEERCRED)` → `pid ∈ allow-set`).
No tokens, no secrets, no `/proc`. thread = the handle IS the grant (no allow-set); process =
the pid allow-set; remote = mTLS cert (later).

## The three stones (ordered)

### 1. C0b.3b-b — the enforcement MECHANISM (the security gate)

The `SocketListener` (kernel/listener.rs) carries an **allow-set** of pids
(`Mutex<HashSet<i32>>` — `CommListener` is `Send+Sync`, so not `RefCell`). The accept path
(`SocketListener::accept` + `poll'`'s Listener arm) reads `peer_cred` on the accepted fd →
checks **`uid == mine` (coarse, always-on)** + **`pid ∈ allow-set` (precise)** → serve, or
**refuse-and-drop SILENTLY** (close the fd, re-poll — "it saw the door, got no answer";
`poll'` never surfaces an unauthorized connection). Verbs to mutate the set: `allow'`/`deny'`
(working names — **intueri names them**) take the listener + a pid.
- Decisions: uid==mine **always-on** (a wat service serving a different uid is not a default
  we want); refuse is **silent** (matches the closed-service model; revisit observability —
  a `:Refused{cred}` event — only if a need surfaces, don't build it speculatively).
- Gate: same-process probe — a service `allow'`s its own pid → a same-process connector is
  served; empty/other allow-set → refused. (Connector's `peer_cred.pid` is the test's own
  pid → no pid-exposure needed yet.) This ships the gate WITHOUT the provisioning protocol.

### 2. The post-spawn block — owner-side after-fork effects (a general spawn feature)

A general hook, NOT security-specific (the grant is just one use; others: log a line, clock
spawn latency, register with N services). **On the opts record, adjacent to `init-fn`** (the
host-param IS the spawn-program configuration — builder's call). Shape:
- `Fn(child-pid: i64) -> nil`, run in the **OWNER** (calling thread) **after the fork, before
  `spawn-program'` returns control** (synchronous), purely for effects.
- **Subsumes the pid-exposure prerequisite** — the child's pid arrives AS the argument; no
  separate "get-child-pid" verb.
- **Mirror of `init-fn` in purpose, OPPOSITE in side:** `init-fn` = child-side env producer
  (runs at peer-start in the child); post-spawn = owner-side effects (runs in the owner). The
  consequence: post-spawn runs in the owner's memory, so it's a closure with **no
  crossing problem** → **uniform on `ThreadOpts` AND `ProcessOpts`, zero tier-asymmetry**
  (unlike `init-fn`, where process has the closure-can't-cross gap). Easy parity.
- Sig unmoved: `(spawn-program' host prog)` stays 2-arg; the block lives on the opts record,
  defaulted to a no-op constructor (like `(thread)` defaults init-fn to the EmptyEnv thunk).
- Open detail: the thread tier's pid arg is degenerate (a spawned thread shares the process
  pid). Keep `i64` pid (thread = the shared process pid, block usually a no-op), OR pass the
  thread's **tid** (`wat.os-thread-id`, the meaningful per-thread identity) — decide at draw.
- Names (the block, the opts slot, the ctor) → **intueri**.

### 3. user.program injection parity — structured data into the runtime (the frustration fix)

`user.program` is the structured-data injection channel (the `-J`/system-properties analog,
but a real `:wat::Record`, not ENV's string→string); wat-cli is the thin kernel↔universe
bridge that should declare it. **The gap:** `freeze.rs:1095` hardcodes `(:wat::program::EmptyEnv)`
for the **root main** (`:process`) — so **wat-cli cannot inject `user.program` into the root
universe**, and process children can't either. Only thread children (via the `init-fn` closure)
can today.
- **The fix is DATA-shaped, NOT a closure** (which is why it doesn't mirror the thread init-fn):
  wat-cli has no closure to run, and a closure can't cross into a process child. So root +
  process inject a `:wat::Record` **value** (trivially shippable — Records are
  `EdnRepresentable` post-i-0). The thread `init-fn` (a *fn*, for live computation in shared
  memory) stays the special case.
- Surface: wat-cli gains a way to declare the root `user.program` Record (the `-J` analog);
  `ProcessOpts` gains a `user.program` Record value (shipped to the child) — distinct from the
  thread `init-fn` fn. Names → intueri.
- This is an env/spawn-config feature, broader than C0b.3b; tracked here because the
  discussion surfaced it.

> **⚠️ DESIGN EVOLVED (2026-06-13) — env-fn is a SOURCE STRING, not a literal Record value.**
> Decomposed into three stones: **3b-d** (foundation, SHIPPED `1ea575ce`) =
> `invoke_user_main_with_program(frozen, args, user_program: Value)`; **3b-e** (process,
> STRIKE) = `ProcessOpts` carries `env-fn <- :wat::core::String`, a wat source string the child
> evals in its own frozen world → dispatch (0-arg fn → apply / `:wat::Record` → use) → user.program;
> **3b-f** (wat-cli root flag) = GATED on arc-213 (CLI → `spawn-program'`; do NOT build on the
> `fork_program_from_source` grave). A source string (not a literal Record) unifies named call /
> bare anon fn / direct ctor expr, crosses the fork trivially, is CLI-friendly + testable.
>
> **wat-cli surface (3b-f, builder 2026-06-13):** the root env-fn source comes from EITHER
> **`--env "(form)"`** (flag) OR **`WAT_ENV="(form)"`** (environment variable) — both supported,
> identical semantics (a wat source string eval'd before `:user::main` → user.program). **If BOTH
> are passed, wat-cli PANICS** (ambiguous source; refuse rather than silently pick one). Exactly
> one source, or none (→ EmptyEnv default).

## Then → thread+process PARITY → Stone C (the defservice defmacro)

With 3b-b (the gate) + the post-spawn block (the grant hook) + the address/listener/peer
entities (C0b.2e, done), the provisioning protocol composes: owner spawns the client → the
post-spawn block grants its pid to the service (via `allow'`) → the client connects → the
service enforces. A grant→connect ordering (the client must not dial before the grant lands)
uses the existing READY-handshake pattern (c0b2d) — a GRANTED handshake. Stone C = the
defservice defmacro that generates this whole shape (the c0b1b + c0b3aii service-loop probes
are its target output).
