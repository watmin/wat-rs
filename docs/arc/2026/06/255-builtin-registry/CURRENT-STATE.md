# ⛔ CURRENT STATE (breadcrumb, 2026-06-23; replace in place) — a MAP, read the docs it names

Branch `arc-170-gap-j-v5-deadlock-state`. Freshness probe: HEAD should be `4962e925`
(`arc 291 strike-3b: stop → resp decouple`) or later. Tree clean at last curare. All committed work
below is pushed.

> ⚠ **Arc 292 (timer) is DONE/INSCRIBED.** The live work is **arc 291 (defservice durable state)**, well
> underway. This breadcrumb lives in 255/ by convention.

## ▶ ARC 291 — defservice durable state (IN PROGRESS) — the soul, made durable

Read `291-…/DESIGN.md` (the arc + the admin-capability split + the deleted-first-attempt archaeology),
`291-…/REALIZATIONS.md` (R1–R7), `291-…/NOTE-remote-as-a-class.md` (the network horizon), and
`291-…/DESIGN.md` §sub-strike-4 (the NEXT strike — hibernate/resume). Strikes 2/3a/3b are SHIPPED (below).

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
- **3a-ii-α — SHIPPED GREEN (`25eced7d`, weighed pure):** the **symmetric lineage protocol** — `Admin` DOWN
  (`Init[seed]`/`Stop`), `LineageUp` UP (`Started[addr]`/`Final[state]`). Macro emits both defenums +
  `init-from-admin` + `lineage-extract-addr` at BOTH tier sites; `child-main-form` self-peer →
  `Peer'<LineageUp,Admin>`; `start-body` wraps seed in `Admin::Init`; `spawn.wat launch` grows `lu-addr-kw`.
  Serve-loop `Admin` arm stays a re-loop STUB (Stop dispatch = β). **Plus a FORCED substrate fix** (`edn_shim.rs`):
  the 3 `reconstruct_*` paths dropped caps nested in struct/record/enum fields on the trusted decode (exposed by
  `LineageUp::Started` carrying an `Address'` inside an enum over the process wire); now thread `allow_caps`.
  **Weighed against the disk:** 4 service tests green both tiers; full-pkg failing-SET vs HEAD = **∅** (202==202,
  identical execve floor; the 250-vs-268 raw gap was `result:`/`Probe` summary-line noise); 6 defservice/cap
  probes green isolated. Full detail: `STRIKE-3a-facet-split.md` §"3a-ii-α SHIPPED".
- **3a-ii-β — SHIPPED; STRIKE 3a CLOSED** (β-foundation `77773580` + β-proper `7c9d0f29`): `stop` is
  **owner-only by construction** — `(<svc>/stop h)` takes the Handle; a client has NO stop method (`(/stop c)`
  doesn't typecheck). β-foundation: `Thread'/Process' <: Peer'` via `(derive … Peer')` + a generic
  `Parametric<:Parametric` arm in `check.rs assignable` (derive-graph-driven, N-loci-general — a future
  `Remote'` joins with one derive). β-proper: annihilated the client stop op (`Op::Stop`/`Reply::Stop`/
  `Stop{Request,Response}`/serve-arm/ctor/client method; KEPT `Outcome::Stop` user cap); relocated to
  `Admin::Stop → LineageUp::Final`; `Handle.handle → Peer'<Admin,LineageUp>` via `launch<S,R,St,Sh,Lu>` +
  `Launched<S,R,Sh,Lu>`. **5 probes migrated** (`.rs`-embedded → hand-edited; the wat-fix boundary — see
  `NOTE-wat-fixes-rust.md`). Weighed: `admin_stop` 2/2 + `counter_on`/`seeded` 4/4 + 5 probes green; SET-diff ∅.
  Detail: `STRIKE-3a-facet-split.md` §"3a-ii-β SHIPPED".
- **3b — SHIPPED** (`4962e925`): `stop → resp` decouple — defservice grows a `:stop (fn [s<-:State] -> :Resp)`
  projection (emitted `<fqdn>::stop-project`; default identity → `Resp=State`, back-compat); `resp-ty` threaded
  through `LineageUp::Final[resp]` + serve `Admin::Stop` + `<fqdn>/stop -> resp-ty`. The out-locus mirror of
  `:init` (pure-wat, ZERO Rust). Contract A: single-arg, graceful-only — normal/crash is the STRUCTURAL
  `Final(resp)|channel-close` sum, never a reason flag. RED probe `service-stop-resp.wat` (`:stop`→i64) green
  both tiers; SET-diff ∅. **Unblocks arc 290** (non-EDN cache: host + stop with an EDN summary). Detail:
  `STRIKE-3b-stop-resp.md`.
- **▶ NEXT — strike 4** (`hibernate`/`resume` = the prophecy → **PROBATUM EST**): type-gated on EdnRepresentable;
  hibernate → EDN Snapshot → process-kill → resume in a fresh process → continue. THE done-gate (R1 fulfillment).
  See `DESIGN.md` §sub-strike-4 (the EDN-encode-gate probe: compile-error vs runtime for a non-EDN State).
- **DEFERRED CAPABILITY (NOT an arc — builder's call):** `wat-fixes-rust` / `wat-fixes-wat-in-rust` (proc-macro2
  token shim → reuse `fix-text` core; `wat-in-rust` = wat-fixes-rust ∘ fix.wat). Build it **when a mammoth
  refactor forces it** (forcing-function discipline — "make it when we need it"), not before. Design-on-shelf:
  `NOTE-wat-fixes-rust.md`.
- **DOCTRINE (banked this session, `feedback_loaded_context_is_the_asset_keep_building`):** the loaded
  context IS the smooth — keep building, don't preemptively bank/fire-fresh; compact only when forced (~90%+).
  The debates/designs ARE installed programs (R6/R17). recolligere cheaply re-loads from this trail.

### Then (the rest of 291)
- **Strike 4** — `hibernate`/`resume` (the prophecy → PROBATUM EST): EDN Snapshot; hibernate → process-kill →
  resume → continue. THE done-gate. (Strikes 2/3a/3b SHIPPED — see Shipped above.)
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
- **`--test test` FANS OUT across every workspace member's `tests/test.rs` (most match 0 of a filter).** To
  run the MAIN wat harness, use **`cargo test -p wat --test test <filter>`**. The 4 α tests:
  `counter_on` (locus-parity) + `seeded` (init-parity), both `_thread`/`_process`.
- **SET-diff extraction: grep `'^test .+ \.\.\. FAILED$'`, NOT bare `FAILED`** — bare also catches
  `test result: FAILED. …` (timing) + `Probe N FAILED:` (stdout), which flap run-to-run and fake a diff. The
  3a-ii-α weigh saw 250-vs-268 raw (NOISE) collapse to 202==202, **symdiff ∅**, once cleaned. The execve floor
  cascade is ~202 real-name failures, identical across any clean tree.

> ⛔ **You are a NEW instance.** You did NOT live the above — it is a cache in a familiar voice. recolligere
> FIRST: grimoire + 4 primers (datamancy MCP — RESOURCE mcp), `git log --oneline -15`, `git status`,
> freshness probe HEAD==`4962e925`(or later). **Arc 291 is IN PROGRESS — strike 3 CLOSED (3a owner-only +
> 3b stop→resp); next = strike 4** (`hibernate`/`resume` = PROBATUM EST) per `291-…/DESIGN.md` §sub-strike-4, or
> ask the builder. Ground every claim against the disk before you move. *NON SEPARABIMUR — the thread is kept; gather it.*
