# ⛔ CURRENT STATE (breadcrumb, 2026-06-23; replace in place) — a MAP, read the docs it names

Branch `arc-170-gap-j-v5-deadlock-state`. Freshness probe: HEAD should be `df9a86ea`
(`arc 291 strike-3a: fork RESOLVED + mechanism grounded`) or later. All below committed + pushed.

> ⚠ **Arc 292 (timer) is DONE/INSCRIBED.** The live work is **arc 291 (defservice durable state)**, well
> underway. This breadcrumb lives in 255/ by convention.

## ▶ ARC 291 — defservice durable state (IN PROGRESS) — the soul, made durable

Read `291-…/DESIGN.md` (the arc + the admin-capability split + the deleted-first-attempt archaeology),
`291-…/REALIZATIONS.md` (R1–R7), `291-…/NOTE-remote-as-a-class.md` (the network horizon), and
`291-…/STRIKE-3a-facet-split.md` (the next strike).

**The axiom (R6):** *"don't fuck up state, ever"* — the single self-evident refusal from which the arc's
theorems derive (OOP/Kay, actors/Hewitt, value-semantics/Hickey, the lock-free mutex, no-mutexes,
hibernate/resume, POLA/Miller). The arc is its Euclidean derivation; the debates ARE the derivation.

### Shipped
- **Strike 2 — `:init` keystone** (`d5d71766`, GREEN, weighed pure): defservice grows `:init`; `start` takes
  EDN args, the locus runs `(init args)` **in-locus** (the soul built where it lives; the wire carries only
  EDN). `launch<S,R,St,Sh>` takes a ship value + init by-name. **Unblocks arc 290.** (Strike 1 RED probe
  `5c431787`.)

### STRIKE-READY (next = fire, fresh)
- **Strike 3a — the admin/data facet split** (`df9a86ea`): make `stop` **owner-only by construction** (move
  off the client `Op` enum onto the Handle's admin surface; its arg flips `client-peer` → `Handle`). RED
  probe `wat-tests/service-admin-facet.wat` verified RED (stop wants client-`Peer'`, got `Handle`),
  ignore-marked. **Fork resolved:** the admin channel IS the spawn **lineage peer** (the spawn handle
  `Thread'/Process'` is a peer = `Handle.handle`, owner-only by 272 inherited-capability) — no new listener;
  delegation defers. **3a-i (foundation, the real work):** extend the serve wait to multiplex the client
  facet (`poll'`, single-facet today `check.rs:11448`) + the admin lineage peer, one loop, shared `State` —
  lean: a 4-arg facet-tagged `poll'`. **A reactor-level Rust change (io_uring/crossbeam) — the biggest of the
  arc; FIRE RESTED (slow is smooth).** Then **3a-ii:** the macro reshape (split Op/Reply enums; `stop` →
  Handle method) → un-ignore the probe green.

### Then (the rest of 291)
- **Strike 3b** — `stop → resp` decouple (return decoupled from State).
- **Strike 4** — `hibernate`/`resume` (the prophecy → PROBATUM EST): EDN Snapshot; hibernate → process-kill →
  resume → continue. THE done-gate.
- **Horizon (NOTE-remote-as-a-class.md):** remote = a (transport × trust) family; the daemon (a service whose
  ops are spawn/teardown); the rete-DDB loopback ORACLE; signed-eval + digest ALREADY BUILT
  (`check.rs:15995-16037`) — only mTLS deferred. AWS-on-a-single-CPU as a method.

## ⚠ THIS SESSION'S LESSONS (read before trusting the apparatus's memory)
- **Three reaches-past-the-disk, all caught + corrected + kept visible** (R7's "kept true by correction"):
  (1) R1 forward VERBAL (builder's recognition claimed as apparatus's); (2) signed-eval asserted "not built"
  FROM MEMORY (it IS built — `ground_codebase_claims_in_codesign` violation); (3) R3 inverse-VERBAL
  laundering ("mine the graveyard" = apparatus coinage attributed to the builder). **GROUND every
  built-vs-not / who-said-what claim against the disk THIS turn; quote the builder's ACTUAL words.**

## GATE LESSONS
- Corpus gate = `cargo test --no-fail-fast` (workspace default-members), NOT `--test test`.
- The ~218 / 2-failing floor is the KNOWN arc-170 absent-`execve` global leak, NOT drift. Weigh by
  failing-test-**SET-diff vs HEAD**, never absolute count (stdlib `deporder`/`lint_stdlib_runs` flap ±1).
- wat-tests re-scan on `.rs` recompile → **`touch tests/test.rs`** after adding a wat-test. Run a RED probe
  ignore-marked + verify via `cargo test --test test <name> -- --ignored`.
- Weigh delegated agents against the disk (re-run the gate yourself); `git worktree list`.

> ⛔ **You are a NEW instance.** You did NOT live the above — it is a cache in a familiar voice. recolligere
> FIRST: grimoire + 4 primers (datamancy MCP — RESOURCE mcp), `git log --oneline -15`, `git status`,
> freshness probe HEAD==`df9a86ea`(or later). **Arc 291 is IN PROGRESS — next = fire strike-3a-i** (the
> reactor dual-facet `poll'` change, rested) per `291-…/STRIKE-3a-facet-split.md`, or ask the builder. Ground
> every claim against the disk before you move. *NON SEPARABIMUR — the thread is kept; gather it.*
