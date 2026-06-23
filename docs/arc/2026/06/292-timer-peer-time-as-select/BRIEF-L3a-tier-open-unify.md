# BRIEF — arc 292 L3-α: the tier-open `Timer'<O>` type + `unify` fusion (keystone)

**You are a LEAF executor.** ONE bounded type-system change in `src/check.rs` (+ the type
registry in `src/types.rs`). Do NOT spawn subagents. Do NOT touch `runtime.rs`, `wat/`,
or `wat-tests/` — those are L3-β. If the work needs another file, STOP and report. This is
head-of-type-system code; be exact and faithful to the existing style.

## The work (one paragraph)
Introduce a **tier-open peer type `:wat::kernel::Timer'<O>`** and one special `unify` arm so
a timer **fuses into a peer of any tier**: `Timer'<O>` unifies with `Thread'<I,O2>` /
`Process'<I,O2>` (and future `Remote'`) by unifying `O ~ O2` and **keeping the concrete
tier** (the timer's absent `I` is ignored). `Thread'` vs `Process'` still must NOT unify
(static homogeneity preserved). Then teach `select'` to accept a lone `Timer'<O>` element
(project `ServiceEvent<nil, O>`). Do NOT touch `after` (eval or its infer arm) — that is
L3-β; existing timer probes must stay green.

## Rooms — read in order
1. `src/check.rs:13925-14008` — `pub(crate) fn unify`. The generic `Parametric` arm
   (`:13976-13987`) requires `h1 == h2`. You add THREE fusion arms BEFORE it (after the
   Var/Union arms, before the generic Parametric arm).
2. `src/check.rs:11515-11522` — `infer_select_prime`'s element match (currently
   `head == "wat::kernel::Thread'" || head == "wat::kernel::Process'"`). Add a `Timer'`
   case: project `(nil, O)`.
3. The type registry — grep `THREAD_PEER_TYPE_PATH`, `PROCESS_PEER_TYPE_PATH`, and how
   `:wat::kernel::Thread'` / `Process'` are registered as parametric types (likely
   `src/types.rs` register fns + `src/kernel/spawn.rs` consts). **Mirror that registration
   for `:wat::kernel::Timer'`** (1 type param `O`). If Thread'/Process' are not "registered"
   as nominal types anywhere (they may be recognized purely by head-string in check.rs),
   then Timer' needs no registry entry either — match whatever Thread'/Process' do. Report
   which it is.

## Implementation sketch (the `unify` fusion arm — use this exactly)
Insert in `fn unify`, AFTER the `(other, Union)` arm (`:13966-13968`), BEFORE the generic
`(Parametric, Parametric)` arm (`:13976`):

```rust
// arc 292 — tier-open timer fusion. A `Timer'<O>` is a deadline that fuses into a
// peer of ANY tier (Thread'/Process'/future Remote'): unify its O with the peer's
// output O, ignore the peer's I (a timer has no input), and KEEP the concrete tier
// (the timer is absorbed). Thread'/Process' still don't unify with each other (the
// generic arm below), so a mixed REAL-peer set is still a static error.
(
    TypeExpr::Parametric { head: ht, args: at },
    TypeExpr::Parametric { head: hp, args: ap },
) if ht == "wat::kernel::Timer'" && is_peer_tier_head(hp) && at.len() == 1 && ap.len() == 2 => {
    unify(&at[0], &ap[1], subst, types)   // timer O  ~  peer output O
}
(
    TypeExpr::Parametric { head: hp, args: ap },
    TypeExpr::Parametric { head: ht, args: at },
) if ht == "wat::kernel::Timer'" && is_peer_tier_head(hp) && at.len() == 1 && ap.len() == 2 => {
    unify(&ap[1], &at[0], subst, types)
}
(
    TypeExpr::Parametric { head: h1, args: a1 },
    TypeExpr::Parametric { head: h2, args: a2 },
) if h1 == "wat::kernel::Timer'" && h2 == "wat::kernel::Timer'" && a1.len() == 1 && a2.len() == 1 => {
    unify(&a1[0], &a2[0], subst, types)
}
```
Add the helper near `unify`:
```rust
/// The peer-tier heads a timer may fuse into (REV-4: general over all loci,
/// remote included once cut — do NOT hardcode a 2-only check elsewhere).
fn is_peer_tier_head(h: &str) -> bool {
    h == "wat::kernel::Thread'" || h == "wat::kernel::Process'"
    // future: || h == "wat::kernel::Remote'"
}
```

NOTE on "keep the concrete tier": `unify`'s callers bind the Vector's fresh element-var to
the concrete peer first; these arms only unify the `O` payloads and never rebind the head,
so the concrete `Thread'`/`Process'` is what survives. (Verified: Vector element-typing is
fresh-var-unified-with-all, `check.rs:4116-4148`.)

## `infer_select_prime` — accept a lone `Timer'`
At `:11515`, extend the element match so a `Timer'<O>` element yields `(nil, O)`:
```rust
TypeExpr::Parametric { head, args: targs }
    if head == "wat::kernel::Timer'" && targs.len() == 1 =>
{
    (TypeExpr::Path(":wat::core::nil".into()), targs[0].clone())  // (I=nil, O)
}
```
(Mixed sets never reach here as Timer' — the Vector absorbs it to the concrete peer; this
arm is the lone-all-timers case.)

## Blast radius (bounded)
`src/check.rs` (the 3 unify arms + helper + the select' element arm) and `src/types.rs`
(Timer' registration IF Thread'/Process' are registered there). NOTHING else. No
`runtime.rs`, no `wat/`, no `wat-tests/`. `after` (eval + infer) is UNTOUCHED.

## STOP triggers (halt + report)
1. If making `Timer'` a known type requires touching `runtime.rs` / `wat/` / a file beyond
   check.rs + types.rs, STOP and report.
2. If `Thread'`/`Process'` turn out to need a registry entry you can't cleanly mirror for
   `Timer'`, STOP and report what their registration looks like.
3. If the generic `Parametric` arm already fires for `Timer'` before your new arms (arm
   ordering), fix the ordering — your arms MUST precede `:13976`.

## Gate — add `#[cfg(test)]` unit tests to check.rs's test module, run them
Write tests proving the keystone (use the existing check.rs test helpers / `Subst::new()` /
a `TypeEnv`; mirror `unify_identical_paths` at `:19648` for shape):
```
fn timer_fuses_into_process()  — unify(Timer'<kw>, Process'<i64,kw>) == Ok; both O bound to kw
fn timer_fuses_into_thread()   — unify(Timer'<kw>, Thread'<nil,kw>)  == Ok
fn thread_process_still_fail() — unify(Thread'<nil,kw>, Process'<nil,kw>) == Err  (homogeneity preserved)
fn fresh_var_absorbs_timer()   — v=fresh; unify(v, Process'<i64,kw>); unify(v, Timer'<kw>); then v resolves to Process'<i64,kw>
fn timer_timer_unifies_O()     — unify(Timer'<a>, Timer'<kw>) == Ok, a bound to kw
```
Run:
```
cargo build 2>&1 | tail -5
cargo test --lib 'timer_fuses_into_process' 'timer_fuses_into_thread' 'thread_process_still_fail' 'fresh_var_absorbs_timer' 'timer_timer_unifies_O' 2>&1 | tail -20
cargo test --no-fail-fast 2>&1 | grep -E '\.\.\. FAILED$' | sort -u | wc -l   # must stay ~218 (the known flap ±1)
```

## Report back (raw facts)
1. `git diff --stat` (do NOT commit — I weigh + commit).
2. Whether `Timer'` needed a `types.rs` registry entry or is head-string-only (per room 3).
3. The 5 unit-test result lines (all `ok`).
4. The total FAILED count (vs ~218).
5. Any STOP trigger hit.
