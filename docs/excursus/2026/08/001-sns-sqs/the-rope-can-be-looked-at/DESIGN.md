# DESIGN — the rope can be looked at

Builder: *"draw #3"* — ranked third in
`the-crash-surface-is-enumerated/FINDING-the-crash-surface.md` §5: *"sanctioned wat-level `close'` —
unblocks Send Closed, TrySend Closed, CloseOutcome::Signaled, CloseOutcome::Failed. **Four cells.**"*

**Drawn 2026-09-13. NOT STRUCK.**

## ⛔⛔ FIRST — ITEM 3 AS RANKED DOES NOT EXIST. IT IS TWO ITEMS, AND ONLY ONE IS A STONE

My census said one door and four cells. The crawl found **two walls**, and they divide the cells:

```
WALL 1 (type)       :wat::kernel::close takes (Thread :- [I O]) | (Process :- [I O])
                    — a LINEAGE — never a dialed Peer. Probed: passing a Peer is a TypeMismatch.
WALL 2 (ruling)     close is :restricted-to [:wat::kernel::] — arc 259 S2d,
                    "THE USER NEVER HOLDS THE ROPE"; teardown is RAII Drop for ordinary code.
```

⭑ **Wall 2 is a RULING, already on disk, and a prior self already ran this entire crawl.**
`tests/process/signal_kill_produces_close_outcome_signaled.wat:11–24` records it in full: the arc-259
restriction, that `close` has **zero production wat call sites**, that the obvious test-only workaround
(a helper defined under `:wat::kernel::`) hits `ReservedPrefix` — *"verified empirically"* — and that
stdlib privilege is unavailable to `tests/`-loaded fixtures. **It even names the two sanctioned futures:**
*"a stdlib-privileged test helper, or a non-restricted **peek** verb"*, and says that fixture should then be
replaced by a pure-wat one.

### So the four cells split, and the split is the finding

| cells | behind what | disposition |
|---|---|---|
| `CloseOutcome::Signaled` · `CloseOutcome::Failed` | **observability** — nothing in wat can look at how a lineage ended | ⭑ **THIS STONE.** A non-consuming peek. Reverses no ruling. |
| `SendOutcome::Closed` · `TrySendOutcome::Closed` | **arc 259's ruling** — they require *use-after-close*, i.e. userland consuming a lineage and then touching it | ⛔ **RETIRE from the ranked list as UNREACHABLE BY DESIGN** |

★★★ **Those last two are not a gap — they are a wall working.** *"The user never holds the rope"* makes
use-after-close **unwritable in userland**, which is the top of the extirpare ladder. A census cell that is
empty because the mistake has no form is a success, not a debt. Building a way to reach them would mean
reversing arc 259 to improve a coverage number.

⭑ **And this refines the census's own taxonomy**, which is worth more than the two cells:
`UNREACHABLE` must split into **"no mechanism yet"** (a gap) and **"no mechanism by ruling"** (a wall
holding). The FINDING recorded both as one status, which is how a working wall ended up on a to-do list
ranked above real work.

## Why the peek, and why it is not a hole in the wall

The danger `close` carries is that it **tears down** — it consumes the peer, double-close raises, and
ordinary code is meant to let Drop do it. **An observation carries none of that.** Looking at how a lineage
ended cannot leak a resource, cannot double-free, and cannot be used to tear anything down.

> **The user never holds the rope. This lets them look at it.**

The Rust side already has exactly the non-blocking primitive: `try_wait()` (`src/process/clone.rs:171`)
returns `io::Result<Option<ExitStatus>>` — *"has it ended, and how"*, without consuming. `JoinHandle::
is_finished()` is its thread-tier sibling.

## The one contract decision

```
:wat::kernel::lineage-status  [peer <- ((Thread :- [I O]) | (Process :- [I O]))]
                              -> (:wat::core::Option :- [:wat::kernel::CloseOutcome])
```

> **`None` means still running. `Some(outcome)` means ended, and how.** It does **not** consume the peer,
> it is **not** caller-restricted, and it is **not** a teardown.

★ Reusing `CloseOutcome` inside an `Option` rather than minting a parallel enum: the three end-states are
identical (`Closed [exit]` / `Signaled [signal]` / `Failed [cause]`), and `CloseOutcome` is already
**Pure** and wire-crossable *because the peer is consumed elsewhere* — the registration comment says so at
`src/types.rs:2118–2121`. A second enum would be the same shape with a different name, and every arm would
have to be written twice.

⚠ **Naming is the builder's to overrule cheaply.** `lineage-status` describes what it reads. `peek` is the
word the prior self used; `ended?` would be a predicate and loses the outcome.

## The four questions

**Obvious?** YES — `None` = running, `Some` = ended thus; and it reads as an observation, not an action.
**Simple?** YES — one intrinsic over `try_wait`/`is_finished`, no new type, no restriction. **Honest?**
YES, and this is where the *other* half failed: widening `close`'s whitelist would have weakened a ruled
wall for a coverage number, and this changes no wall at all. **Good UX?** YES — it is the verb a caller
actually wants (*"did my child die, and how?"*) and the one `signal`'s users have had no answer from.

## Scope

**IN:** the `lineage-status` intrinsic (both tiers) · `CloseOutcome::Signaled` made reachable from wat by
`signal` + peek · `CloseOutcome::Failed` made reachable by a panicking child + peek · a probe that prints
both · the FINDING's item 3 rewritten to record the split and retire two cells.

**OUT = REJECTED:**
- ⛔ **Widening `close`'s `:restricted-to`.** Reverses arc 259 S2d. If the builder wants it, that is a
  doctrine ruling and its own stone — **not** a side effect of a coverage stone.
- ⛔ **`SendOutcome::Closed` / `TrySendOutcome::Closed`.** Retired as unreachable-by-design above. ⚠ Their
  ~217 and ~5 arms stay unexecuted, and that is now a **recorded, justified** fact rather than an open one.
- ⛔ **Replacing `tests/process/signal_kill_produces_close_outcome_signaled.rs`** with a pure-wat fixture.
  That header explicitly invites it *"if a future strike gives wat source a sanctioned path"* — which this
  is. **Tempting and rejected here:** this stone adds a verb; retargeting an arc-170 evidence fixture is a
  separate change to someone else's gate. Named so it is not lost.
- **A blocking `wait`.** `None`-means-running is the whole point; a blocking wait is `close` without the
  consume and reintroduces the rope.

## Trap-doors

1. ⛔ **Do not make it consume.** If the peer is unusable afterwards, this is `close` with a new name and it
   inherits arc 259's reasoning.
2. ⛔ **Do not restrict it.** A caller whitelist on an observation would recreate the wall this stone exists
   to route around honestly.
3. **Both tiers.** `try_wait` is process-only; a Thread needs `is_finished`. A verb that works on one tier
   and raises on the other is half a verb — and the `Peer`-vs-lineage type wall already caught me once here.
4. **`Signaled` means TERMINATED, never merely stopped** (`src/types.rs:2113`). A SIGSTOP'd child must not
   report `Signaled`; that is `Failed`'s *"stopped-not-terminated"* case.
5. **`CloseOutcome` is `Purity::Pure` and must stay so.** It is Pure *because* the peer is consumed
   elsewhere; an `Option<CloseOutcome>` returned from a peek still holds no live resource. Confirm, do not
   assume.
6. **This reaches `src/`** — the first stone this session to do so. Clippy and the floor both matter more
   than usual.
