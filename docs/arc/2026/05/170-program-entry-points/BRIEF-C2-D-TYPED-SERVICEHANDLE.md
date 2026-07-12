# BRIEF — C2 via D: service handles stay TYPED through the check (`TypedCapability<S,R>`, bodiless edge)

> **The C2 kill, landed honest.** N heterogeneous services dialed by name; a swapped handle a **compile
> error**. Service handles stay *typed* through the kwargs-check as a combined `TypedCapability<S,R>` surface, so
> they are dialed (typed) AND granted (typed) without ever erasing to `Value` — no runtime-satisfaction question.
>
> **The one thing that walled the first attempt is solved.** Re-declaring `coord`/`grant`/`revoke` in a third
> extend-type collides on the flat `<Type>/<method>` registration key (`DuplicateDefine`). The fix, **proven this
> session**: the third auto-emit is a **BODILESS extend-type** — it registers the satisfaction *edge* (for
> assignability) WITHOUT re-declaring the methods; runtime dispatch serves `coord`/`grant`/`revoke` from the
> Handle's *existing* Dialable+Capability impls via the flat key. Honest names, no collision, **zero `src/`
> change**.
>
> **Executor: sonnet shadowdancer, weighed by the orchestrator's own re-run.** Build **FRESH on committed HEAD**
> (the prior phantom was reverted for a clean probe — see §Baseline). The prior attempt's report proved the whole
> D shape type-checks except the auto-emit; this brief carries that recipe + the bodiless correction.

## Ratified vocabulary (intueri-cast + ratified — use these EXACT names)

| thing | name | note |
|---|---|---|
| the combined surface | **`:wat::capability::TypedCapability<S,R>`** | `Capability` with a typed coordinate; a surface names a capacity, not a thing |
| its methods | **`coord`** (`-> Address'<S,R>`), **`grant`**, **`revoke`** | reused verbatim — served by the Handle's existing Dialable/Capability impls via flat dispatch |
| pure carrier: crosses to the child, dialed | **`<work-fn>::Coords`** (HEAD already mints it) | typed `Address'` |
| impure parent-local carrier: the typed handles, granted through | **`<work-fn>::GrantHandles`** | an is-peer-filtered `defstruct` of `TypedCapability<Si,Ri>` fields |
| generated fn: grant one worker's pid to every held handle | **`<fqdn>::grant-worker`** (+ `<fqdn>::revoke-worker`) | unrolled typed `TypedCapability/grant\|revoke` over the literal service-field list |

---

## The disconfirming probes — PROVEN GREEN this session (the foundation; copy the SHAPE + names verbatim)

Ran against committed HEAD (`./target/release/wat`):

1. **Positive** (`scratchpad/probe-v-bodiless.wat`) — **freezes clean.** A `TypedCapability<S,R>` surface
   (`coord`/`grant`/`revoke`), a `echo'` service, and a **bodiless** `(extend-type :probe::echo'::Handle
   :probe::TypedCapability<probe::Echo::Op,probe::Echo::Reply>)` — no method bodies. A fn holds the abstract
   `TypedCapability<Echo…>` type and calls **both** `grant` and `coord` through it; a raw `echo'::Handle` is
   assignable to the param. **The edge registers from the bodiless form; no `DuplicateDefine`.**
2. **Negative** (`scratchpad/probe-v-swap.wat`) — **1 located `TypeMismatch`:** a `kv'::Handle` (bodiless-edged to
   `TypedCapability<Kv…>`) into a `TypedCapability<Echo…>` param → `expects
   :probe::TypedCapability<probe::Echo::Op,probe::Echo::Reply>; got :probe::kv'::Handle`. Sound.
3. **Runtime** (`scratchpad/probe-v-run.wat`) — **executed, printed OK:** `TypedCapability/coord h` dispatches to
   the Handle's `coord` (registered by Dialable-extend) via the flat `<Type>/<method>` key — a surface-method call
   resolves by the *receiver's* type, regardless of which surface names it.

**This is the exact mechanism the auto-emit must reproduce** (a bodiless per-service extend-type). Promote #1/#2
to committed tests (§Expectations).

---

## Baseline — build FRESH on committed HEAD

The prior phantom (Strike 2's `bracket/uses` macro + a Dialable-checker rework, then the failed D attempt) was
**reverted** to get a clean probe substrate. So HEAD is the baseline. What HEAD **has** (committed): W2a's
auto-minted `<fqdn>::kwargs-check` (head-swapping `Peer'→Address'`), Strike 1's `<work-fn>::Coords` mint +
`:wat::bracket::uses'` N-dial runtime (proven via *direct* `uses'`), and the parametric surfaces. What HEAD does
**not** have (was in the reverted phantom — rebuild it): the `bracket/uses` *macro*, the checker typed to a
surface (it's `Address'` today), the grant layer. **Read `wat/core.wat` (the `defn` kwargs branch) and
`wat/bracket.wat` (`uses'`) live to ground exactly what's there.**

De-risking (from the reverted attempt's own report — plausible, re-verify): `::GrantHandles` minted as an
is-peer-filtered `defstruct` cleared 293.W the same way `::Kwargs` does; `grant-worker`/`revoke-worker` as
unrolled typed calls type-checked; `uses'` took `grant-handles <- :G` + `grant-fn`/`revoke-fn <- Fn(G,i64)->nil`
by value (keyword-auto-upgrades-to-Fn, same as `map-worker`'s `work-fn`). Only the auto-emit collided — which the
bodiless edge fixes.

---

## Design

**The algorithm.** Service fields flow through the kwargs-check typed `TypedCapability<S,R>` (head-swap the
checker's service params `Peer' → TypedCapability`). A raw `Handle` satisfies it via the **bodiless auto-emit**.
The check body (which splits service-vs-data by `is-peer` at defn-mint, where field types are literal) builds two
carriers: the pure `::Coords` (typed `Address'` via `coord`, crosses, dials — HEAD already builds it) and the
impure parent-local `::GrantHandles` (the typed handles). A generated `<fqdn>::grant-worker` grants each
`::GrantHandles` field per worker via `TypedCapability/grant`. Data fields copy as EDN (deferred; §Out of scope).

**The one contract decision (pinned):** the per-service satisfaction of `TypedCapability` is a **bodiless**
extend-type (edge only). `Dialable<S,R>` and `Capability` are kept and unchanged (they provide the actual
`coord`/`grant`/`revoke` impls the flat dispatch serves).

**Files:** `wat/capability.wat` (surface), `wat/service.wat` (bodiless auto-emit), `wat/core.wat` (kwargs-check →
`TypedCapability` + `::GrantHandles` + `grant-worker`), `wat/bracket.wat` (the `bracket/uses` macro; `uses'` grant
layer), `tests/services/` (promote the probes + author the mixed macro test).

**Out of scope = rejected (`exigere`):**
- **The check/runtime satisfaction parity divergence is NOT fixed here** — D removes every consumer of it; inscribe
  it as a tracked follow-up (structural collapse of runtime `value_matches_type_by_name`/`conforms_check` onto the
  one `is_subtype` authority). Do NOT touch `src/runtime.rs`/`src/check.rs`.
- **The data-copy kwarg path** — the shape admits it; build when a consumer passes a data kwarg.
- **The `map` arg-order fn-first flip** — separate cosmetic sweep.

## The rooms — read in order

1. **`scratchpad/probe-v-bodiless.wat` + `probe-v-swap.wat` + `probe-v-run.wat`** — the proven shape + names. The
   bodiless extend-type + the flat-key dispatch are here, freezing clean / rejecting the swap / dispatching at
   runtime. **The codegen must emit this shape.**
2. **`wat/capability.wat:44–46`** — `Dialable<S,R>`. **ADD** `:wat::capability::TypedCapability<S,R>` beside it
   (`coord`/`grant`/`revoke`), exactly the probe's surface renamed.
3. **`wat/service.wat`** (grep `grantable-extend` / the Dialable auto-emit) — the two per-service auto-emitted
   extend-types. **ADD a THIRD, BODILESS:** `(extend-type <fqdn>::Handle :wat::capability::TypedCapability<Op,Reply>)`
   — no method bodies. Mirror `dialable-extend`'s `Op`/`Reply` wiring; emit an extend-type form with an empty
   method list.
4. **`wat/core.wat`** (the `defn` kwargs branch — read live) — W2a mints `<fqdn>::kwargs-check` head-swapping
   `Peer'→Address'`; Strike 1 mints `<work-fn>::Coords`. **CHANGE the checker head-swap target `Address' →
   TypedCapability`** (so `bracket/uses` passes raw handles typed as `TypedCapability`, and the swap is caught by
   the parametric edge). **ADD** the `::GrantHandles` mint (is-peer-filtered `defstruct` of the service fields) +
   `<fqdn>::grant-worker`/`revoke-worker` (unrolled typed calls from the literal service-field list).
5. **`wat/bracket.wat`** (`uses'`, read live) — **BUILD the `bracket/uses` macro** (parse `(locus items work-fn
   :name val …)`, emit the checker call + forward to `uses'`), and **extend `uses'`** to take `::GrantHandles` +
   `grant-fn`/`revoke-fn` and grant/revoke per worker via `grant-worker`. Dial via `::Coords` (HEAD's path).

## STOP triggers (rejection — ship nothing, surface the gap)

1. **The bodiless auto-emit doesn't register the edge / doesn't dispatch** (a real service's Handle fails to
   satisfy `TypedCapability<Op,Reply>`, or `TypedCapability/grant` fails to resolve at runtime) → STOP, report —
   though the probe proves it works, so this would mean the auto-emitted form differs from the probe's.
2. **`::GrantHandles` (impure parent-local `defstruct`) is refused by 293.W** → STOP (the reverted attempt cleared
   this; if it re-walls, report the exact diagnostic).
3. **The typed `grant-worker` fold can't be generated** over the heterogeneous `::GrantHandles` fields → STOP.
4. **The swap stops being a compile error** through the full macro → STOP; soundness is the whole point.
5. **You edit `src/runtime.rs` or `src/check.rs`** → STOP; D is purely the typed wat layer. The bodiless edge is
   the reason no `src/` change is needed.

## How to work

- `cargo build --release`; `cargo nextest run --release` **FOREGROUND-blocking** (never `&`). A mid-edit
  rust-analyzer diagnostic is a PHANTOM; a clean build + a suite that ran N tests compiled.
- Negative-test asserts are **structural** (match the error enum; no `contains`). `.wat.bad` for the swap fixture.
- Do NOT commit — leave the tree for the orchestrator to weigh, then land C2 whole in one commit.

## Expectations (fixed before the strike)

| what | command | expected |
|---|---|---|
| bodiless positive freezes | `./target/release/wat --check scratchpad/probe-v-bodiless.wat` | clean |
| swap a compile error | `./target/release/wat --check scratchpad/probe-v-swap.wat` | 1 `TypeMismatch` |
| build | `cargo build --release` | clean |
| **the mixed positive RUNS green** (author it) | `cargo nextest run --release -p wat -E 'test(c2_mixed_macro)'` | both PASS — `mixed_via_macro_runs` → `["run-7 echo:a·kv:a" …]`, swap = compile error |
| W2a + C1 + parametric surfaces intact | `cargo nextest run --release -p wat -E 'test(/arc170_(w2a\|c1\|parametric_surface)/)'` | all PASS |
| full floor | `cargo nextest run --release` (FOREGROUND) | prior floor + these; **0-new** (modulo the known `no_inlined_wat` lint) |

**Runtime prediction:** 45–75 min (fresh build of the macro + grant layer + D, guided by the recipe; not a small
diff). **Trap-doors:** `::GrantHandles` 293.W (STOP-2); the `grant-worker` typed fold (STOP-3); the macro's
`Tuple`/carrier plumbing to `uses'`.

Report: the diff per file, the bodiless auto-emit form, the `::GrantHandles`/`grant-worker` shapes, the
`mixed_via_macro_runs` result, the full nextest Summary line, and any STOP.
