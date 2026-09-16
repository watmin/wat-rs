# DESIGN-STONE — a `with-` form's release is safe only if a Drop owns it

> **Origin (2026-08-31).** Class B1 of `VIGILIA-2026-08-30-WORK-LIST.md`, found by `secare`,
> verified on the disk, and — until now — **never driven**. Driven here, at HEAD `85043bbab`,
> on BOTH unwind paths. It leaks on both.

## Why

`wat/rete/syntax.wat:307-310`:

```
(:wat::core::let [base   (:wat::core::match (:wat::rete::compile-all rules queries) …)
                  result (body-fn base)]
  (:wat::core::do
    (:wat::rete::release-session base)
    result))
```

The release sits in a `do` **after** the body. Any unwind skips it. The lease is the sole owner
count (`DESIGN-STONE-intern-eviction`: *"the lease **is** the owner count"*), so a miss pins the
whole `InternedNetwork` until thread end.

### The measurement — driven, both arms, at `85043bbab`

Two probes, one per unwind path, appended to `src/rete/kernel/tests/arm_lease.rs`. Both RED:

```
FAIL scoped_work_with_network_releases_the_lease_when_the_body_raises
  arm_lease.rs:414: assertion `left == right` failed: with-network must release the lease
  compile-all took when the body raises a wat ERROR; table grew 0 -> 1, so the InternedNetwork
  is pinned until thread end
    left: 1
   right: 0

FAIL scoped_work_with_network_releases_the_lease_when_the_body_panics
  arm_lease.rs:442: assertion `left == right` failed: with-network must release the lease
  compile-all took when the body PANICS; table grew 0 -> 1, so the InternedNetwork is pinned
  until thread end
    left: 1
   right: 0
```

**Leases are not observable from wat** (row 3 of the scoped-work suite says so), and on an unwind
`with-network` never hands the Session back — so there is no id to ask `rete_arm_leases` about.
The instrument is a new `#[cfg(test)] rete_arm_table_len()` and the measure is a **delta**.

## ⚠ TWO UNWIND PATHS, AND A FIRST DRAFT RODE ONLY ONE

`:wat::kernel::assertion-failed!` **PANICS the host** — `runtime.rs:15922` says so outright. A
first draft of this probe used it as "the raise", and the panic **blew past the probe's own
assertion**: the test failed with no assertion message at all, which reads exactly like a test
that ran. The wat-*error* path (`DivisionByZero`, an `Err` out of `eval_in_frozen`) is a different
mechanism and needed its own probe. A second draft asserted both in sequence; arm 1 failed and
**arm 2 never ran**. Hence two separate tests. *One drive cannot prove a two-arm gate.*

## ⛔ THE ROOT IS NOT THE `let`+`do` SHAPE — AND THIS INVERTS THE WORK LIST'S FIX SHAPE

`wat/io.wat:40`'s `with-open-file` has the **byte-identical shape** — `let` + `(do (close w)
result)` — and it does **not** leak. Its resource is a Rust value whose `Drop` closes the fd
(`io.rs:1188`: *"`Drop` closes via OwnedFd"*, verified, not taken from the comment). The shape was
never the defect. **The absence of an owner is.**

So the class, stated so it covers the next one:

> A `with-` form's `(do (release …) result)` is unwind-safe **only when the thing released is
> owned by a Rust value whose `Drop` performs the release.** `with-open-file` earns that.
> `with-network` does not: the lease lives in a side table keyed by id, and
> `DESIGN-STONE-intern-eviction` **forbids** an intern handle on the Session.

`with-network`'s doc claims parity with `with-open-file`. That claim is false in exactly the
load-bearing respect, and the cure is to **make it true** rather than to bolt on a second release.

## ★ THE ONE CONTRACT DECISION

**The lease is owned by a value whose `Drop` releases it, and `with-network` contains NO release
call at all.** After this strike there must be no path — normal return, wat error, or host panic —
on which the lease outlives the wat scope that took it. The `(do (release-session base) result)`
is **deleted**, not supplemented: two release sites is the bug wearing a fix.

Climb to the shape. A second release call in an error branch is the rung below, and it is what
allowed this — `with-network` already *had* a release call.

## The algorithm

1. **`ArmLease` guard** in `src/rete/kernel/arm.rs` — holds the `u64` id; `Drop` calls
   `rete_arm_release(id)`. **ADOPT semantics:** it takes no new lease; it assumes ownership of the
   one `compile-all` already took. State that in its doc, because it is the non-obvious half.
2. **`rete_arm_release` must use `ARM_TABLE.try_with(...)`, not `.with(...)`.** A guard still alive
   at thread exit drops during TLS teardown, and `.with()` **panics** if the table was destroyed
   first (destruction order is unspecified). `try_with`'s `Err` is "the table is already gone" —
   ignore it; the semantics are already "missing id is a no-op".
3. **A restricted primitive** `(:wat::rete::adopt-session-lease <session>) -> :rust::rete::ArmLease`
   minting the guard via `make_rust_opaque` (`rust_deps/marshal.rs:342` — dedicated `Value`
   variants are explicitly *discouraged* there). `#[restricted_to(…, ":wat::rete::")]` so only
   rete's own wat can call it. **Exactly one call site.**
4. **`wat/rete/syntax.wat`** — `with-network` binds the guard in the same `let` as `base` and drops
   the `do` entirely.

## Blast radius

`src/rete/kernel/arm.rs` (guard + `try_with` + the test-only `rete_arm_table_len`), `src/runtime.rs`
(dispatch), `src/check.rs` (TypeScheme), `src/rete/purity.rs` (op list — it already lists
`release-session`), `wat/rete/syntax.wat`, and `src/rete/kernel/tests/arm_lease.rs`. No Session
field. No new `Value` variant. No `unsafe`.

## Out of scope — AFFIRMATIVELY CUT

- **Session `Drop` releasing.** `DESIGN-STONE-intern-eviction` § THE ONE CONTRACT rejects it by
  name (*"Do not put an intern handle on the Session"*); a Value copy would then deprovision a
  live network. Read that stone before re-proposing it.
- **Moving `with-network` into Rust.** Considered and rejected on *Simple*: `foldl` shows a
  polymorphic higher-order primitive needs hand-written inference (`check.rs:2385`,
  `infer_foldl`), and writing bespoke type inference to fix a lifetime bug is the wrong ladder.
- **`wat/query.wat:411`'s explicit `release-session`.** A different door (`:stop` on a connection),
  not a scoped body. Untouched.
- **An `ensure`/`finally` form for wat.** wat has none, and inventing a general unwind form to fix
  one leak is the situation-that-needs-the-patch, not the patch.
