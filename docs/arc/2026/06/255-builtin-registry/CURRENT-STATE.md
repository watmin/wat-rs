# ⛔ CURRENT STATE (breadcrumb, 2026-06-23; replace in place) — a MAP, read the docs it names

Branch `arc-170-gap-j-v5-deadlock-state`. Freshness probe: HEAD should be `0a3888ab`
(`arc 291 strike-3a-ii: RESOLVED — the symmetric lineage protocol`) or later. **NOTE: the working tree may be
DIRTY** — a 3a-ii-α shadowdancer was in flight at compaction (see the IN-FLIGHT note below; check `git status`
FIRST). All committed work below is pushed.

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

### Strike 3a — the admin/data facet split (make `stop` owner-only by construction)
RED probe `wat-tests/service-admin-facet.wat` (`df9a86ea`, verified RED, ignore-marked): `(<svc>/stop h)`
wants `client-Peer'`, got `Handle`. **Fork resolved:** the admin channel IS the spawn **lineage peer** —
and the KEY grounding win: `poll'` arg0 (the self-peer / owner-lineage channel) was ALREADY in the wait,
just discarding its messages.
- **3a-i — SHIPPED GREEN (`1c6d8690`):** `poll'` now inspects index-0's result — `Ok(msg) → ServiceEvent::Admin{msg}`,
  `Err(_) → Shutdown`. `ServiceEvent<I,O> → <I,O,A>` (admin msg = self-peer's receive type). Both tiers
  (thread: Value; process: decode the wire frame). **STOP lesson:** a new ENUM VARIANT makes every match
  non-exhaustive (a real cascade my brief missed — param-free ≠ variant-free); resolved via the existing
  `Connection`-stub precedent (Admin arm on all 6 match sites; 5 `select'` sites "can't happen", service.wat
  re-loops). Weighed: 281/2/56, SET-diff = the known floor. **Emission verified by reading; 3a-ii exercises it.**
- **3a-ii — RESOLVED design** (`0a3888ab`, in `STRIKE-3a-facet-split.md` §"3a-ii RESOLVED"): the **symmetric
  lineage protocol** — `Admin` DOWN (`Init[seed]`/`Stop`), `LineageUp` UP (`Started[addr]`/`Final[state]`),
  uniform across tiers. control DOWN the channel, result UP via `LineageUp::Final`. `Handle.handle` re-types
  `Spawned` → `Peer'<Admin,LineageUp>`. **α/β cut:** 3a-ii-α = protocols + startup handshake migration (pure,
  existing tests green) → 3a-ii-β = stop dispatch + the Handle method (un-ignores `service-admin-facet.wat`).
- **⚠ IN FLIGHT AT COMPACTION (2026-06-23):** a shadowdancer (`a53cf41d`) was firing **3a-ii-α** — its edits
  are **UNCOMMITTED in the tree** after HEAD `0a3888ab`. **ON WAKE, FIRST:** `git status` + `git diff --stat`.
  → If `wat/service.wat`+`wat/spawn.wat` show lineage-protocol edits (the agent landed): **WEIGH it** —
  `cargo test --test test counter_on` (service-locus-parity + service-init-parity MUST be green; it's a pure
  migration) + `cargo test --test test 2>&1|tail -3` (SET-diff = the 2-test floor) + read the diff; commit on
  green as `arc 291 strike-3a-ii-α`. → If it STOP'd or left a broken/partial tree (red, type errors): read the
  agent transcript (`/tmp/claude-1000/.../tasks/a53cf41dd83fe3237.output` via the task tools, NOT `cat`), weigh
  the finding, fix or `git checkout` + re-fire 3a-ii-α per the brief. → If the tree is CLEAN: the agent didn't
  land — re-fire 3a-ii-α. **Then 3a-ii-β.**
- **DOCTRINE (banked this session, `feedback_loaded_context_is_the_asset_keep_building`):** the loaded
  context IS the smooth — keep building, don't preemptively bank/fire-fresh; compact only when forced (~90%+).
  The debates/designs ARE installed programs (R6/R17). recolligere cheaply re-loads from this trail.

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
