# DESIGN-STONE C0b.3b-c — the post-spawn hook (owner-side after-spawn effects, per-env record)

> The owner-side mirror of `init-fn`. Every hosting env supports a post-spawn hook that runs in
> the OWNER after the peer is spawned, before `spawn-program'` returns, purely for EFFECTS (grant
> a child's pid to a service via `allow'`, log, metrics). What the hook RECEIVES differs per env
> — a per-env launch-info record — but the PATTERN is universal. Because the host opts type is
> known at the call site, the hook fn and its record accessors type-check at PARSE time.
> Tracked #237. Co-design: `DESIGN-C0b.3b-provisioning-and-spawn-hooks.md`.

## Why

C0b.3b-b shipped the allow-set gate (a process service refuses any connector not in its
birth-seeded allow-set). The owner provisions OTHER pids via `allow'` — but to do that it needs
the spawned child's pid, owner-side, right after spawning it. The post-spawn hook is that seam:
the owner spawns a client, the hook fires with the child's pid, the owner `allow'`s it on the
service, then the client connects and is served. The hook is general (grant is one use; log /
metrics are others); the grant use is what makes it load-bearing for the provisioning protocol.

## The shape (builder-confirmed 2026-06-13)

A post-spawn hook is the pattern **all** hosting envs support; the **record it receives differs
per env** (payload, not pattern):

- **thread** env → an EMPTY launch record (`ThreadLaunch` — no fields yet; grows if a need appears).
- **process** env → a launch record carrying the child pid (`ProcessLaunch [pid <- i64]`).
- **remote** env → its own record, when remote arrives (organic; NOT built now).

The hook is `Fn(<EnvLaunch>) -> :wat::core::nil`, **required-with-default** on every opts record
(mirrors `init-fn`: the field always exists; `(thread)`/`(process)` default it to a no-op
accepting that env's record). NOT `Option` — that keeps it off the optional-is-a-smell trap;
"optional" is only the UX of not passing it. The single record arg grows evolutionarily — add a
field, never re-sign the hook.

**The payoff (builder's insight):** `spawn-program'` dispatches on the host opts TYPE
(`ThreadOpts | ProcessOpts`, the defclause in `spawn.wat:90`), and record field access is
statically typed (`spawn.wat:95` already does `(:wat::spawn::ThreadOpts/init-fn host)`). So the
hook fn — and every accessor in its body — type-checks at parse time against the per-env record:
a process hook reading `…/pid` checks; a thread hook reading `…/pid` off the empty `ThreadLaunch`
is a CHECK ERROR, not a runtime surprise. This costs zero special checker work — it falls out of
the record types + the `Fn(<EnvLaunch>)->nil` field types.

## Names (intueri cast, weighed; builder ratifies)

- `:wat::spawn::post-spawn-fn` — the hook field on both opts records (sibling: `init-fn`).
- `:wat::spawn::ThreadLaunch []` — the thread env's empty launch record.
- `:wat::spawn::ProcessLaunch [pid <- :wat::core::i64]` — the process env's launch record.
- `:wat::spawn::ProcessLaunch/pid` — the child pid accessor.
- `(:wat::spawn::thread/post-spawn f)` / `(:wat::spawn::process/post-spawn f)` — constructors that
  set the hook (mirror `thread/init`); `(thread)`/`(process)` default it to a no-op.

## The algorithm (mirrors init-fn: extract in wat, apply in the Rust primitive)

The init-fn precedent: the `spawn-program'` defclause EXTRACTS the fn from the opts record and
PASSES it to the Rust tier primitive, which APPLIES it (`spawn.wat:95` → `spawn_thread_peer`
applies `init_fn` child-side at `spawn.rs:425`). The post-spawn hook follows the same path,
owner-side:

1. **`spawn.wat`** — define the two records (`:wat::Record::def`); add `post-spawn-fn` to
   `ThreadOpts` + `ProcessOpts`; default it in `(thread)`/`(process)` to a no-op
   `(fn [_ <- :wat::spawn::ThreadLaunch] -> :wat::core::nil nil)` (resp. `ProcessLaunch`); add
   `thread/post-spawn` / `process/post-spawn` ctors; in the `spawn-program'` defclause, extract
   `post-spawn-fn` and pass it to the tier primitive.
2. **`spawn-thread'` / `spawn-process'`** (the S2c-i kernel primitives) gain a `post-spawn-fn`
   param.
3. **`spawn_thread_peer`** (Rust): after `std::thread::Builder::spawn` returns the handle
   (owner-side, ~`spawn.rs:408`+, in the PARENT flow before returning the peer), construct
   `(:wat::spawn::ThreadLaunch)` via format→`parse_one!`→`eval` (precedent `spawn.rs:448`) and
   `apply_function(post_spawn_fn, vec![launch], sym, span)`.
4. **`spawn_process_peer`** (Rust): in the PARENT branch where the pidfd is in hand
   (`spawn.rs:642–650`; `Pidfd::pid()` at `clone.rs:217`), construct
   `(:wat::spawn::ProcessLaunch {pid})` the same way and `apply_function` it. Owner-side, in the
   parent — the hook is the owner's closure, no crossing.

The hook runs OWNER-side in both tiers (the closure is the owner's; for process it runs in the
parent after fork). No new `Process'/pid` wat accessor — the pid stays in the Pidfd; the hook
receives it inside `ProcessLaunch`.

## The ONE contract decision (pinned)

The `post-spawn-fn` is a REQUIRED field on every host opts record, typed
`Fn(<that env's Launch record>) -> :wat::core::nil`, defaulted to a no-op by the bare ctor. It
runs OWNER-side, exactly once, after the peer is spawned and before `spawn-program'` returns,
for effects only (its `nil` return is discarded). Its argument type is the per-env Launch record,
checked at parse time. If the hook raises, the spawn raises (the owner's effect failing is the
owner's error — surfaced, not swallowed).

## Files touched

`wat/spawn.wat` (2 records, 2 opts fields, 2 ctors, defclause extract+pass), `src/kernel/spawn.rs`
(`spawn_thread_peer` + `spawn_process_peer` gain the param + apply owner-side; the
`spawn-thread'`/`spawn-process'` eval entries thread it through), `src/check.rs` (only if the
defclause/Record::def flow needs a touch — expected FREE; verify). The RED probe(s).

## Out of scope (rejected — NOT deferred)

- **Remote post-spawn** — no remote env exists; it gets its own Launch record organically when it
  arrives (the `:remote` forcing function stays unbuilt).
- **`ThreadLaunch` fields beyond empty** — added only when a thread-tier need appears (don't build
  the forcing function; the empty record is the honest "nothing to report yet").
- **A combined `init`+`post-spawn` ctor** — chain or add when a program needs both; not now.
- **#238 user.program injection parity** — a separate stone.

## The gate (probes — RED at HEAD on exactly the gap)

1. **`probe_arc209_c0b3bc_process_post_spawn`** (wat e2e, the disconfirming probe): the owner makes
   an owner-side channel (`peer-pair'` / make-channel), spawns a `(process/post-spawn f)` where `f`
   captures the sender and `(send' tx (:wat::spawn::ProcessLaunch/pid info))`; the owner `recv'`s
   the pid and asserts it is `> 0` and `≠` the owner's own pid (a real child pid, owner-side).
   - RED at HEAD: `process/post-spawn` is an unknown ctor → check/eval error.
   - GREEN after: the owner reads the spawned child's pid that the hook forwarded.
2. **`probe_arc209_c0b3bc_thread_post_spawn`** (wat e2e): a `(thread/post-spawn f)` where `f`
   receives the empty `ThreadLaunch` and sends a sentinel to an owner-side channel; the owner
   reads the sentinel (the hook fired owner-side on the thread tier with the empty record).
   RED at HEAD (`thread/post-spawn` unknown), GREEN after.
3. **`probe_arc209_c0b3bc_accessor_typecheck`** (check-error probe): a `process/post-spawn` hook
   whose body reads a NONEXISTENT field, OR a `thread/post-spawn` hook reading `…/pid` off the
   empty `ThreadLaunch` → a CHECK error (the parse-time payoff). At HEAD the ctor is unknown
   (different error); after #237 the check error names the missing field. (Assert the check-error
   substring so it's RED→GREEN on exactly the type-check, not just "errors both ways".)

Regression: all spawn/c0b probes green untouched; the bare `(thread)`/`(process)` ctors still
spawn (default no-op hook fires harmlessly). Nursery 895/4 (zero new) + full compile.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** the default no-op hook breaks a bare `(thread)`/`(process)` spawn (any existing
   spawn/c0b probe goes red) — STOP; the default ctor must remain a clean no-op.
2. **STOP-2:** the parse-time accessor type-check does NOT fire (a thread hook reading `…/pid`
   compiles) — STOP; the payoff is the whole point. Report the checker gap.
3. **STOP-3:** applying the hook owner-side in `spawn_process_peer` cannot see `sym`/an env to
   `apply_function` + build the record — STOP, report (the init-fn precedent has `sym`; expected
   available).

## Deadlock contract

The hook is a synchronous owner-side `apply_function` call between spawn and return — no new
blocking primitive, no lifecycle change. A hook that itself blocks is the owner's own logic
(vended primitives never deadlock; user logic may). [[feedback_vended_primitives_never_deadlock]]
