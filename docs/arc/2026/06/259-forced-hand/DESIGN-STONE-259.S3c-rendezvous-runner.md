# 259 S3c — bake the process runner; the rendezvous coordinate (2026-07-08)

> **Supersedes the S3b process-arm shape.** The agent's S3b process arm SHIPPED a runner
> define (`:bracket::__pool-runner`) + the work-fn (`:bracket::__pool-work`) as `user.program`.
> That forced the whole reserved-name problem: to make the shipped runner name un-squattable we
> tried `:wat::bracket::…`, which the child rejects (`ReservedPrefix` — a shipped user-program
> form can't define a reserved name). S3c dissolves it (`COMPONENDO DELEO`): **don't ship a
> reserved define at all — bake the runner.**

## The one contract

**`:user::` is the RENDEZVOUS NAMESPACE** — the known-location coordinates where a program exposes
what a substrate consumer looks up. Not private/internal space; a rendezvous space.
- `:user::main` — wat-program's coordinate (the kernel-required entry, `[] -> :nil`).
- `:user::bracket::work-fn` — wat-bracket's coordinate (the work function the pool runner applies).

**The reserved thing is BAKED, never shipped:**
- `:wat::bracket::process-runner<I,O>` — a **baked, reserved** stdlib fn in `bracket.wat`.
  Generic; takes `[self <- Peer'<(i64,I),(i64,O)>, work-fn <- Fn(I)->O]`; index-wraps
  (`recv (idx,I) → send (idx, work-fn item)`); tail-recurses. Established in the child's
  phase-one stdlib load — **privileged, reserved, zero user input.** A user can never allocate it
  (`:wat::` is undefinable anywhere) and it isn't shipped, so nothing can collide with it.

**We ship only the user's code, into the clean room** (`spawn-program'` process arm, `user.program`):
- `(fn-forms work-fn :user::bracket::work-fn)` — the user's work-fn + its deps (their own
  `com.foo.bar` names, untouched), the reified value bound at the rendezvous coordinate.
- a generated `:user::main []` that **passes** the work-fn value to the baked runner:
  `(:wat::bracket::process-runner (:wat::program::self-peer <sp-out> <sp-in>) :user::bracket::work-fn)`.
  The runner is baked, so `:user::main` **passes** the value — it cannot look the coordinate up
  from stdlib (that would be a stdlib→user.program forward reference the resolver rejects).

**The derived-types AST-splice STAYS** (Blocker-A resolution, unchanged in spirit): the concrete
`(i64,arg)`/`(i64,ret)` tuple types for `self-peer` are still derived off the `fn-forms` output
(`ast-name` on the reified define's arg/return), now spliced into **`:user::main`'s `self-peer`
call** instead of a shipped runner-def. The generic baked runner needs no concrete types (they
monomorphize at the call). Never hardcode `i64` on the payload — the index dim is `i64`, the
payload dims are derived.

## Why no mistake is possible (the threat model, closed)

- **No reserved name is shipped** → the `ReservedPrefix` problem is gone; the baked runner is
  untouchable. A user can never allocate a reserved name (they can't define `:wat::` anywhere).
- **The rendezvous coordinate is non-reserved** (`:user::bracket::work-fn`). A clash needs a user
  work-fn dep named *exactly* that. Closed three ways: (1) we hold the `fn-forms` output parent-
  side and pick a bind-name not present in it (or gensym) — impossible by construction; (2)
  default no-redef → a clash is a loud located `DefRedefForbidden`, never silent corruption; (3)
  convention — users write their own `com.foo.bar` names, not `:user::*`.

## No underscores / no "internal" markers

The names are declared coordinates, not hidden internals — the name says what it is. Drop every
`__`: `:user::bracket::work-fn` (not `__pool-work`), `:wat::bracket::process-runner` (not
`__process-runner`). Reservedness lives in the `:wat::` prefix, not a decoration.

## Declare the convention

`bracket.wat`'s header declares: **bracket installs `:user::bracket::work-fn` into the program it
builds** — parallel to how `:user::main` is the program's kernel-required coordinate.

## Proven

`scratchpad/probe-s3c-rendezvous.wat` → `"6 10"` — the runner-takes-work-fn-value + rendezvous-
coordinate composition, with the runner still shipped (baking it is pure relocation into stdlib).

## Gate

- `scratchpad/probe-s3-bracket-loci.wat` → `[2 4 6 8 10] [2 4 6 8 10]` (thread pool AND process pool)
- `scratchpad/probe-s3c-rendezvous.wat` still `"6 10"`
- every arc259 bracket test green; whole floor 0-new (modulo the known `no_inlined_wat` lint)
- `verify_stdlib_has_no_load_order_violations` (deporder) green
