# BRIEF — decomplect bracket: one pool, one orthogonal kwargs-provisioning layer

> **Bracket should do one thing — a scoped worker-pool.** It has become many: `map`-worker braids the *pool*
> (spawn/collect) with *capability* (grant/revoke) and *service-dial* (Setup), and `uses'` is a second coordinator
> that re-forks all three the typed way. This stone **decomplects** them: `map`-worker becomes the one pool;
> **provisioning** (the `:name val` kwargs — services grant+dialed, data copied) becomes one orthogonal layer that
> rides on **both `map` and `each`**; `bracket/uses`, `spawn::process/uses`, and the old Capability grant/dial
> **delete**.
>
> **The ratified surface** (co-designed with the builder): the `:name val` tail is optional and rides on the pool
> verb, not a separate verb.
> ```clojure
> (bracket/map  locus items work-fn :echo eh :kv kvh :tag "run-7")  ; pooled map + N typed kwargs (services + data)
> (bracket/each locus items work-fn :echo eh :kv kvh :tag "run-7")  ; pooled each + the SAME kwargs layer
> (bracket/map  locus items work-fn)                                ; plain pool, no kwargs
> ```
> The kwargs are the work-fn's typed injected context (a `Peer'` field → grant + dial; a data field → copy as EDN;
> a resource → forbidden, 293.W). This is exactly C2's mechanism (the committed `mixed_via_macro_runs`, 7 services
> + 5 data) — *moved off the `uses` verb onto `map`/`each` and shared*.
>
> **Executor: sonnet shadowdancer, weighed by the orchestrator's own re-run. HEAD `f4939611` is on DR — if this
> walls, we `git reset --hard f4939611` and reattempt. So attempt the restructure boldly, but STOP (don't
> improvise a fallback) on a real substrate wall.**

## The substrate is CONFIRMED viable (probes ran this session — copy the shape)

- **`spawn-runner<D>` goes D-generic**: making the `Locus/spawn-runner` surface return `Peer'<PoolMsg<D,I>,…>`
  (was bare `Address'`) + the thread/process impl annotations `PoolMsg<D,I>` **compiles** (stdlib froze).
- **`collect-loop<D,I,O>`** (bracket.wat:431) is **already D-generic** — the drain is done.
- **`PoolMsg<:nil,I>` and `PoolMsg<::Coords,I>` are both valid, and `D` binds from the expected-type CONTEXT**
  (`scratchpad/probe-carrier-unit.wat` froze clean: a `wire<D,I>` whose `D` is only in the return binds `D=nil`
  from a plain caller's return type, `D=::Coords` from a rich one). So a **plain pool binds `D=nil`** (a unit
  carrier) and a **kwargs pool binds `D=::Coords`** — no carrier argument needed.
- **The one thing that broke the naive attempt** was map-worker's body producing a *concrete* `Address'` (from
  `Capability/coordinate`) while its signature said `PoolMsg<D,I>`. **This decomplection removes exactly that** —
  the carrier is no longer welded into map-worker's body; it flows from the provisioning layer.

## Design

**map-worker becomes the one carrier-generic pool coordinator; `uses'` folds into it and deletes.** Its body does
only pool work (spawn-runner → collect-loop) plus a **per-worker provisioning layer** it does not itself define:

- **carrier `D`** — the Setup payload. Plain: `D = :wat::core::nil`. Kwargs: `D = <work-fn>::Coords`.
- **provisioning** — for each worker (after spawn, before its first Work item): grant the worker's pid, then send
  `PoolMsg::Setup carrier`; at shutdown: revoke. Plain: a **no-op** (no grant, no Setup). Kwargs: the typed
  `<fqdn>::grant-worker`/`revoke-worker` + `PoolMsg::Setup coords` (C2's mechanism, unchanged).

The cleanest shape is map-worker absorbing `uses'`'s existing provisioning params — `uses'<D,G,I,O>` already takes
`grant-handles <- :G`, `grant-fn`/`revoke-fn <- Fn(G,i64)->nil`, `coords <- :D` (bracket.wat:608). Generalize
map-worker to the same, and:
- **plain** callers pass `carrier = nil`, `grant-handles = nil`, `grant-fn`/`revoke-fn` = a no-op `(fn [_g _pid] nil)`;
- **kwargs** callers pass `carrier = coords`, `grant-handles`, `grant-fn = <fqdn>::grant-worker`, etc.

Then **`uses'` deletes** (map-worker *is* it, with plain as the trivial case). *(Hook-of-fns vs params: take
whichever type-checks cleanest — the contract is "map-worker does pool + a parameterized per-worker provisioning;
plain is the trivial provisioning; `uses'` is gone." If a new durable name for the provisioning layer emerges,
FLAG it — naming is intueri's, not yours to pick.)*

**`map`/`each` become macros** that parse the optional trailing `:name val` (reuse the `bracket/uses` macro's exact
logic, bracket.wat:682–699 — `checker-call → coords + grant-handles`):
- **no tail** → emit the plain map-worker call (`nil` carrier, no-op provisioning);
- **tail** → emit the `<fqdn>::kwargs-check` reconciliation → the typed map-worker call.
`each` = `map` with the result discarded (`each-worker` is already `(do (map-worker …) nil)`, :728), so the SAME
macro logic serves both — the kwargs layer rides each for free.

**Retire (delete):** `:wat::bracket::uses` (the verb) + `:wat::bracket::uses'` (folded into map-worker) +
`:wat::spawn::process/uses` + `process/uses-pairs` + map-worker's old `Capability` grant-boot/Setup-dial
(bracket.wat ~522–562, `Capability/coordinate`/`Capability/grant`/`Capability/revoke`) + `:wat::spawn::uses` /
`ProcessOpts.uses` if orphaned. `COMPONENDO DELEO` — the correct change subtracts.

**spawn-runner** goes D-generic: `Locus/spawn-runner<D,I,O,W>` returning `Peer'<PoolMsg<D,I>,…>` (spawn.wat:386);
the thread impl's `PoolMsg<Address',I>` annotations → `PoolMsg<D,I>` (bracket.wat:125,127); the process impl
(bracket.wat:211) carries no carrier annotation (`process-work-forms` generates it) — likely no change.

**Out of scope (`exigere`):** the check/runtime satisfaction parity divergence (inscribed, decoupled — no `src/`
change here); the `map` arg-order fn-first flip.

## Suggested internal sequencing (checkpoint each — the DR net lets you back out, but land whole)

1. **spawn-runner<D>** (surface + thread annotations) + **map-worker carrier-generic with parameterized
   provisioning** (absorb `uses'`'s params; plain = nil/no-op). **CHECKPOINT: the existing `map`/`each` tests +
   `uses'`-based tests still green** (route the current `map`/`each` `defn` wrappers through the new map-worker
   with trivial provisioning; keep `uses'` alive routing through map-worker until the surface lands). This is the
   load-bearing step — the pool/provisioning decomplection.
2. **`map`/`each` → macros** with the optional `:name val` tail (the plain path + the kwargs path).
3. **Retire** `uses'`/`bracket::uses`/`process/uses` + the old Capability path; **migrate** `probe_arc170_c2_mixed_macro`
   to `(bracket/map (process) items :enrich :name val …)`; migrate/delete `probe-c1-clean-surface.wat`.

## The rooms — read in order (live)

1. **`wat/bracket.wat:493` (`map-worker`), `:608` (`uses'`)** — the two coordinators to merge into one (map-worker
   absorbs `uses'`; `uses'` deletes). `:509` reads `(:wat::spawn::uses locus)` (the old path); `:522–562` is the
   Capability grant-boot/Setup-dial to delete; `:626–647` is `uses'`'s typed grant/Setup to fold in.
2. **`wat/bracket.wat:709` (`map`), `:723` (`each-worker`), `:734` (`each`)** — the thin wrappers → macros. `each`
   is already `map-worker` + discard, so one macro logic serves both.
3. **`wat/bracket.wat:676` (`bracket/uses` macro)** — the parse to reuse for the `:name val` tail (then delete the verb).
4. **`wat/bracket.wat:431` (`collect-loop<D,I,O>`)** — already D-generic; the drain, unchanged.
5. **`wat/spawn.wat:386` (the `spawn-runner` surface method)** + **`:176` (`process/uses` macro, to retire)**.
6. **`scratchpad/probe-carrier-unit.wat`** — the proven `D=nil`/`D=::Coords`-bind-from-context shape.

## STOP triggers (rejection — ship nothing, surface the located diagnostic)

1. **A plain pool won't compose through the D-generic map-worker** — `D=nil` can't bind, or the plain runner
   chokes on `PoolMsg::Setup nil` — STOP (the probe says `PoolMsg<nil,I>` is valid + `D` binds from context, so
   this would mean the *runtime* Setup path differs; report whether plain must send NO Setup vs a `nil` one).
2. **map-worker can't carry both the `nil` and `::Coords` provisioning** at one definition (the earlier break, if
   it recurs after removing the `Capability/coordinate` weld) — STOP, report the exact `ReturnTypeMismatch`.
3. **You reach into `src/`** — STOP; this is the typed wat layer only (the parity divergence is out of scope).
4. **A retirement isn't clean** — `process/uses`/`uses'`/`bracket::uses` has a consumer you can't migrate — STOP,
   name it (do NOT leave a dead half or a shim).

## How to work

- `cargo build --release`; `cargo nextest run --release` **FOREGROUND-blocking** (never `&`). A mid-edit
  rust-analyzer diagnostic is a PHANTOM; a clean build + a suite that ran N tests compiled.
- Weigh the **full floor** — `map`/`each` are widely used (259 S3), so a regression there is the risk; the floor is
  the gate. Negative asserts STRUCTURAL; `.wat.bad` for the swap.
- Do NOT commit — leave the tree for the orchestrator's own re-run.

## Expectations (fixed before the strike)

| what | command | expected |
|---|---|---|
| build | `cargo build --release` | clean |
| **plain `map`/`each` still green** (the decomplection preserved the pool) | `cargo nextest run --release -p wat -E 'test(bracket)'` | all PASS |
| **`map` + tail runs green** (services + data, the C2 case moved onto `map`) | the migrated `c2_mixed_macro` (author `(bracket/map …)` form) | `["run-7 echo:a·kv:a…" …]`; swap = compile error |
| **`each` + tail runs green** (side-effect pool + kwargs) | a new `each`-with-kwargs test (author it) | side effects fire for all items + services; returns nil |
| `bracket/uses` / `process/uses` / `uses'` GONE | `grep -rn "bracket::uses\b\|process/uses\b\|bracket::uses'" wat/` | only the retired-comment, no live defs |
| full floor | `cargo nextest run --release` (FOREGROUND) | prior floor + these; **0-new** (modulo the known `no_inlined_wat` lint) |

**Runtime prediction:** 60–90 min (a real refactor of core bracket machinery). **Trap-doors:** the plain-pool Setup
handling (STOP-1); map-worker carrying both provisionings (STOP-2); the macro tail-parse for `each`. The DR net
(`f4939611`) is your safety — attempt the restructure fully; STOP only on a genuine substrate wall.

Report: the diff per file, the unified map-worker shape + how plain vs kwargs provisioning is passed, the
`map`/`each` macro forms, the retired defs, the `map`+tail / `each`+tail results, the full nextest Summary, any STOP.
