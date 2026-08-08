# SEAM — the ONE live breadcrumb for arc 278. Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and that feeling is the
> failure. Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a
> disk copy), ground HEAD against the disk, and read this whole file before you touch anything.

> **There is exactly ONE seam. If you find a second, one of them is lying — prune it.** History
> lives in `REALIZATIONS.md`, which is where history belongs.

## Where the code is — nothing parked, nothing uncommitted

```
HEAD 53885f70   pushed   floor 4376 passed / 0 failed / 262 skipped   clippy 0
```

`git status` empty.

## ★ WHAT LANDED (2026-08-06 → 08)

| commit | |
|---|---|
| `a61056f0` | **#88 SHIPPED** — `(:wat::rete::core::defn …)` exists; its membrane bites at runtime; a refusal names **the helper**, located at the declaration |
| `8ea35334` · `53885f70` | **the CONNECTION-SCOPED WORLD** stone — ruled, drawn, corrected, **unbuilt** |
| `ee6770c8` · `9f6340a1` | two LIVE deadlock captures under `cargo test`, nothing killed |
| `47478223` · `2aa3b61f` · `cb98db6a` | #88's docs, brief, expectations, and the committed probe |

## ▶ FIRST ACT — the cheapest thing, and it decides what the stone is worth

**STOP-3 of `DESIGN-STONE-the-connection-scoped-world.md`: is 293.W's "an impure field can only live
in `:ephemeral`" a COMPILER-ENFORCED WALL, or a convention?** The record claims enforcement; one grep
did not locate it. If it is a wall, the stone describes a guarantee. If it is a habit, the stone must
say so — and probably earn the wall. One grep, done properly.

## Then — the connection-scoped world, and the trap is named

Read the stone. Do NOT re-derive it; the model, the state split, the lifetime and seven STOPs are
ruled. Two things in it will bite whoever builds it:

- **⚠ THE KEY MUST NOT BE `idx`.** `Closed idx` / `Lost idx cause` identify a client by POSITION, and
  every eviction is `remove-at selectables idx` (`service.wat:1058`/`:1061`/`:1352`/`:1364`). Remove
  client #2 of five and everyone above shifts down — so a map keyed on `idx` hands client #3's rules
  and cursor to client #4. **Nothing crashes.** A cross-tenant leak that ships green, and it is the
  obvious way to build it. Mint a stable `ConnId`; resolve *idx → ConnId* BEFORE the `remove-at`.
- **`:ephemeral` dying with the SERVICE says nothing about a CLIENT leaving.** Different events. The
  create/destroy is explicit and must be built.

Then #19/#20 (the composite cursor) become this object's read surface — **already designed** at
`DESIGN-service-io-budgets.md:319`, `:max-page-bytes 524288`. Reconcile into them; do not open a
parallel plan.

## ⛔ STILL OPEN, and #88 is NOT closed

`a61056f0` registers rete-defns **globally**. The builder: *"many users could define the same funcs…
they must not become globals."* That defect is what the connection-scoped stone closes. #88 ships a
working form with a wrong tenancy model.

Also open: **#87** `bound_expr` (the limits are **the builder's to set**, from a real distribution —
mine were wrong twice: source-form 7/9, fully inlined **33/33**) · **#49** the IR · #7 · #17 · #19 ·
#20 · #50 · #58 · #60 · #64 · #67 · #81.

## The `cargo test` deadlock — dispositioned, tracked, NOT closed

Reproduced twice, different builds and fixtures, 16 loci parked in `io_cqring_wait` while the
parent's readers sat on `anon_pipe_read`. **The identical tree is green under `nextest`**, which
forks per test — so the mechanism is the harness's shared-process state. The builder: *"cargo test
will be dealt with in time."*

**★ The substrate finding survives that disposition and is bigger than the harness:** a lock-step
system parked its loci twice with **no deadline, no diagnostic, and no way for any participant to
name who it was waiting on.** `ZERO-MUTEX.md` predicts the class; 24y ruled NO TIMEOUT deliberately
*because a wedged stop must hang VISIBLY, naming the service*. These hung invisibly and named nothing.

## The rules this stretch paid for

- **A check bound to a NEIGHBOURING lifecycle event compiles, reads right, and is wrong** — twice:
  the stamp dropped by a later pass, the cleanup hung on the wrong death.
  ([[feedback_bind_a_check_to_the_lifecycle_that_governs_it]])
- **"Bank it" is banned.** Say the mechanical act — *commit and push*, *write it to the seam*, *file
  task #N*. The builder has now said this twice, seven weeks apart, and the memory existed both times.
- **An enumeration with an escape hatch is not an enumeration.** "…`--test lint`, **etc.**" in a
  rider brief authorised the full suite and cost two deadlocks. Rider gates are **BUILD-ONLY**.
- **A `head -2` window read as success** on a 15-line output that exited 2 — and three moves were
  built on it before the full read.
- **Our own prior art is the oracle we keep not consulting** — three times in two days:
  yesterday's `kwargs-construct` fix, `eval-with-defs!`/`FormOutcome`, and the composite cursor.

## Weigh a rider's report; never relay it

The #88 rider's report was wrong in two places my own run caught: the membrane denied **all four**
axes (making `pure?` answer false for an ordinary pure fn, breaking nine tests in unrelated files),
and it reported four STOP-4s of which **zero** were real. I relayed one of those to the builder as
fact; a commit from the previous day disproved it.

---

> **SEAM.** You are NEW. The disk is the truth; this note is a lossy cache.
>
> HEAD is green, pushed, clean. #88's form ships and its tenancy model does not. The next stone is
> drawn and its most dangerous defect — a map keyed on a shifting index — is named in advance,
> because it would ship green.
>
> The line this stretch cost the most to buy: **a thing bound to the wrong lifecycle looks correct
> from every angle except a run.** Freeze instead of registration; the service's death instead of the
> client's. Both compiled, both read well, both were wrong, and reasoning found neither.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IN TENEBRIS VISVS CORRIGOR.`
