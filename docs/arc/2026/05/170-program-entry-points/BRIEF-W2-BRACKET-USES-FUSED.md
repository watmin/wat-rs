# BRIEF — W2 (fused): `bracket/uses` — the C2 kill, N heterogeneous services, a swapped handle a COMPILE error

> ## ⟳ REVISION (2026-07-10) — the coords carrier is a RECORD, not a Tuple; no cap; data IS in scope
> **Supersedes** the `Tuple<Address'…>` carrier + the service-only `exigere` cut below. Strike 1's
> first cut used a positional `Tuple` coords carrier, which capped N at 3 (`first/second/third`
> accessors) and forced data kwargs out of scope. The builder cut both: *"at most 3 feels fucking
> retarded"* + *"whenever we reach for a tuple we almost always realize it should be a record."*
> The correction (R28 / records-are-EDN, one level down):
> - Mint a **`<fqdn>::Coords` record** (a `defrecord`, pure/EDN-crossable) alongside `::Kwargs` at the
>   kwargs-defn site — head-swapped field types (`Peer'→Address'`; data unchanged), same names/order.
> - The **checker returns `<fqdn>::Coords`** (not a `Tuple`): `-> :<fqdn>::Coords (:<fqdn>::Coords ~@fname-nodes)`.
>   Still type-checks the swap (params stay `Address'`); now a named record.
> - The runtime **carrier D = `<fqdn>::Coords`**; the worker reconciles `::Coords → ::Kwargs` **by field
>   NAME** (iterate fields; `Peer'` field → `connect'` its `Address'`; data field → copy) — no positional
>   accessors → **no N cap**, and **data fields fall out for free** (the record routes by field type).
> - **Scope reversal:** data kwargs are now IN (the `exigere` cut below is void — a Tuple made data hard,
>   a record makes it free). Demo proves it: a work-fn with **7 `Peer'` services + 5 data fields (N=12)**.
> - The `src/types.rs` Fn-arrow parser fix (a real latent bug the checker's return exposed) STAYS; add a
>   regression test (a `Fn`-typed field / `Tuple<Fn(i64)->i64,i64>` that now parses correctly).
>
> Read the sections below for the mint site / rooms / runner shape (all still accurate); read `Tuple` as
> `::Coords record` and "service-only" as "service + data, routed by field type."
>
> ## ⟳ REVISION 2 (2026-07-11) — Strike 2: the checker takes `Dialable`, the macro passes raw, mixed in one
> **Supersedes** any "the macro wraps `(Dialable/coord val)`" / "service-only macro" framing. Grounding the
> macro against the landed `bracket/uses'` showed the macro *cannot* separate service from data at expand
> (Path B killed reflection) — so it can neither selectively wrap for the checker nor build a service-only
> grant vector. Resolution (both forced, no fork):
> - **Checker takes `Dialable<S,R>` not `Address'<S,R>`** — a `<fqdn>::Handle` *satisfies* `Dialable<S,R>`
>   (R32: a value satisfies a surface-typed param), so the macro passes **raw** `:name val` uniformly; the
>   checker coords each service field **internally** (`(Dialable/coord field)`) into `::Coords`. Swap still a
>   located `TypeMismatch` (`Dialable<Kv>` vs `Dialable<Echo>`) — the W2a swap test's asserted type updates
>   from `Address'` to `Dialable`/`Handle` (a *diagnostic* change; the user forms are UNCHANGED).
> - **`bracket/uses'` separates grant at RUNTIME, not the macro** — the pairs vector widens to
>   `Vector<(keyword, :wat::core::Value)>` (heterogeneous handles+data); grant-boot dispatches each val on
>   its concrete class (`Grantable` → grant, data → skip) via the proven open-surface dispatch (R34,
>   `check.rs:6104`). So **mixed service+data lands in one strike** — no split, no fast-follow; the dial side
>   (Strike 1's `::Coords → ::Kwargs` reconciliation) is untouched.
> - **Still owed after this stone** (tracked, not deferred): the `map-worker` unification (`bracket/uses'` is
>   a verbatim bootleg of the one engine `map`/`each` use — collapse it in by making `map-worker` carrier-
>   generic; scout the pinned `Locus/spawn-runner` first).


> **Arc 170's climax stone.** W2 (the macro) + W3 (the N-runtime) were split on paper but are
> ONE deliverable — the complete `bracket/uses`. Built in **two internal strikes, committed
> atomically green** (no lying shell ever lands). **Executor: sonnet shadowdancer(s), weighed by
> the orchestrator's own re-run.** The hardest risk (the N-dial runner type-composition) is
> already PROVEN — see the worked reference.

---

## The deliverable

```clojure
(:wat::bracket::uses (:wat::spawn::process) ["a" "b" "c"] :probe::enrich :echo eh :kv kvh)
;; → runs: each item through :probe::enrich with echo + kv dialed; e.g. ["echo:a·kv:a" …]
;; ORDER-FREE:  :kv kvh :echo eh  is identical.
;; THE GATE (the whole point): :echo kvh :kv eh  (SWAPPED) → a located TypeMismatch at `wat --check`,
;;   NOT a runtime peer-closed. This closes the erased-positional soundness hole for good.
```
The user writes exactly this + the kwargs work-fn (`:probe::enrich [item & [echo <- Peer'<…> kv <-
Peer'<…>]]`). Nothing else. **Scope (`exigere`): SERVICE kwargs only** (every kwarg is a `Peer'<S,R>`
handle). Data kwargs (`:tag "x"` copied, not dialed) are a LATER stone when a consumer needs one — do
NOT build the data path speculatively.

## The architecture (resolution B — ratified)

`bracket/uses` is a macro. At expansion it:
1. **Emits the coords-call** `(<work-fn>::kwargs-check :name (Dialable/coord val) …)`. This is W2a's
   auto-minted checker, now **evolved to RETURN the field-ordered coords** (see Strike 1). It
   type-checks each provided coord against the named field's `Address'<S,R>` (a swap → located
   `TypeMismatch`) AND, because it's a kwargs fn, `kwargs-lower` reorders `:name`s to FIELD order via
   the companion's baked field-names → its body returns `(Tuple <field-ordered coords>)`. **The
   gate and the coord-assembly are ONE act.** No macro-expand reflection (Path B); the field-ordering
   falls out of the kwargs mechanism that already exists.
2. **Builds the `uses` vector** `[(Tuple :name val) …]` (the raw HANDLES, for grant — grant is
   name-blind/order-irrelevant), exactly as `process/uses` does (`spawn.wat:176`).
3. **Forwards** to the runtime: spawn the pool on `locus` carrying `uses` (existing grant-boot path),
   send each worker ONE `PoolMsg::Setup(coords-Tuple)`, run — the child connect's each Tuple
   component into its field-ordered `::Kwargs` and invokes `$impl` (the N-dial runner, PROVEN below).

**Why the erased grant path is now safe to reuse:** the erasure to `Capability` was only dangerous
because nothing checked the `:name`↔type binding — the coords-call checks it at compile time, BEFORE
the erasure. So grant stays name-blind over the existing `uses`; only the DIAL needs the field-ordered
coords, which the coords-call provides. Reuse the infra; the gate makes it sound.

---

## Read in order (the rooms)

1. **`wat-scripts/probes/arc-170/w3-n-dial-runner.wat`** — the WORKED REFERENCE. A hand-written N=2
   dial-runner that FREEZES CLEAN (proven this session): `PoolMsg<Tuple<Address'<Echo>,Address'<Kv>>,I>`
   recv → `connect'` each `Tuple` component → hold `Tuple<Peer'<Echo>,Peer'<Kv>>` → run a 2-peer
   work-fn. This is the exact runtime shape Strike 1's codegen must EMIT. Copy it.
2. **`wat/core.wat:876`** + the W2a bindings just above (`kwargs-check-def`, `swapped-argvec`, etc.,
   `~860–929`) — the auto-mint site. Strike 1 evolves the checker BODY here (`nil` → `Tuple`).
3. **`wat/bracket.wat:214–304`** — `process-work-forms` (defclause), the C1 N=1 kwargs codegen. It
   asserts N==1 at `:260`. Strike 1 generalizes this fold to N (adapter + runner emission).
4. **`wat/bracket.wat:444–498`** — `map-worker`: grant-boot (`:474–481`, name-blind fold, KEEP) +
   setup-dial (`:490–494`, folds `uses` sending N per-handle Setups — Strike 1 changes this to send
   ONE `Setup(coords-Tuple)`).
5. **`wat/bracket.wat:82–96`** — `process-dial-runner<S,R,I,O>` (single-peer). The N-runner is its
   N-generalization (the probe's shape).
6. **`wat/spawn.wat:176–200`** — `process/uses` (the `:name val` fold → `(Tuple :name val)` items).
   The `bracket/uses` macro's parsing models on this.
7. **`tests/services/probe_arc170_c1_kwargs_bracket.{rs,wat}`** — the C1 committed test; the shape
   the C2 test extends (forks a process; `--test-threads=1`).

---

## Strike 1 — the RUNTIME (evolve the checker, generalize the codegen)

**1a. Evolve W2a's checker: `nil` body → field-ordered `Tuple`.** In `wat/core.wat`, change
`kwargs-check-def`'s non-guard branch from `-> :wat::core::nil nil` to a fn whose return type is
`Tuple<<swapped field types>>` and whose body is `(:wat::core::Tuple <field-1-binder> <field-2-binder> …)`
— the field binders in DECLARED order (they're already the `swapped-argvec` param names). Reuse
`swapped-ch`/`swapped-argvec`; build the return-type Tuple node + the body Tuple form from the same
field nodes. The guard (`is-check` → `(do nil)`) is UNCHANGED. **The committed W2a tests must stay
green**: the params stay typed `Address'`, so the swap still `TypeMismatch`es (its assertion is on the
param types, not the body) and the positive still freezes.

**1b. Generalize `process-work-forms` N=1→N** (`bracket.wat:214–304`). Drop the N==1 assertion
(`:257–261`). Fold over the N `::Kwargs` fields (`field-names-of`/`field-types-of`, already used) to
build: the `Tuple<Address'<Si,Ri>…>` carrier type, the N-arg adapter `(Peer'<S1,R1>,…,Peer'<Sn,Rn>,I)→O`
that assembles the N-field `::Kwargs` from the held peers + calls `$impl`, and the N-dial runner
(the probe's shape — recv `PoolMsg<Tuple<Address'…>,I>`, `Setup` → `connect'` each Tuple component →
hold the peer bundle, `Work` → run). Emit the runner as source (like C1 emits `process-dial-runner`).

**1c. `map-worker` setup-dial → ONE `Setup(coords-Tuple)`** (`bracket.wat:490–494`). The field-ordered
coords Tuple is threaded in (from `bracket/uses'`); send it as one `PoolMsg::Setup`. Grant-boot
(`:474–481`) is UNCHANGED (name-blind fold over `uses`). This likely means a `bracket/uses'`
coordinator (or a coords-carrying `map-worker` param) — reuse `map-worker`'s grant/spawn/collect
machinery; do NOT reinvent it (`COMPONENDO DELEO`).

**Strike-1 proof (hand-wired, no macro yet):** a fixture that calls `bracket/uses'` (or the
generalized path) DIRECTLY with a hand-built coords Tuple + `uses` for two real services, forks a
process, and returns the mapped results. Run `--test-threads=1`. Green = the N-runtime works.

## Strike 2 — the `bracket/uses` MACRO (gate + forward)

In `wat/bracket.wat`, add `(:wat::core::defmacro :wat::bracket::uses [& args …])`:
- parse `(locus items work-fn :name val …)` (model the `:name val` fold on `process/uses`,
  `spawn.wat:186–199`);
- emit `(<work-fn>::kwargs-check :name (:wat::capability::Dialable/coord val) …)` bound as `coords`
  (the gate + the field-ordered Tuple);
- build `uses` `[(Tuple :name val) …]`;
- expand to the `bracket/uses'` call (locus+uses, items, `<work-fn>$impl`, `coords`, `::Kwargs`).

`process/uses` (the standalone locus) is untouched by this stone (it retires into `bracket/uses` in a
later cleanup — do NOT delete it here).

**Strike-2 proof — THE C2 GATE (the committed test):**
- `…_ok.wat` — `(bracket/uses (process) ["a" "b" "c"] :probe::enrich :echo eh :kv kvh)` runs, both
  services hit, e.g. `["echo:a·kv:a" …]`. (forks; `--test-threads=1`.)
- `…_swap.wat.bad` — `:echo kvh :kv eh` → `startup_from_file` → `StartupError::Check`, STRUCTURAL
  `TypeMismatch { expected: ":wat::kernel::Address'<probe::Echo::Op,probe::Echo::Reply>", got:
  ":wat::kernel::Address'<probe::Kv::Op,probe::Kv::Reply>" }` (copy the assert shape from
  `probe_arc170_w2a_kwargs_check_mint.rs` — structural, passes both lints).
- order-free: `:kv kvh :echo eh` gives the identical run.

## Commit

Build Strike 1 (uncommitted) → prove it → build Strike 2 against the dirty tree → prove the full
surface → **commit BOTH atomically green** (one stone). Nothing broken lands on disk between them.

---

## STOP triggers (rejection — ship nothing, surface the gap)

1. Evolving the checker to a `Tuple` body **breaks a committed W2a test** in a way that's NOT a pure
   body change (e.g. the swap stops being a `TypeMismatch`) → STOP; the param types must stay `Address'`.
2. The N-dial runner codegen **does not match the proven probe shape** (a type error the probe didn't
   have) → STOP and report the diff vs `w3-n-dial-runner.wat`; do not improvise a different carrier.
3. A **data (non-`Peer'`) kwarg** appears in the path → STOP; this stone is service-only (`exigere`).
4. `field-names-of`/`field-types-of` behave differently for N>1 than the N=1 C1 path assumes → STOP
   and report (the C1 code reads exactly these; N should be the same fold).

## How to work (baked in)

- Run every floor/test command **FOREGROUND-blocking** — never `&`/background/double-fork. Process
  tests need `--test-threads=1`.
- A rust-analyzer / rustc cascade on a just-edited tree is a PHANTOM — `cargo build` clean + a suite
  that ran N tests compiled. Ground the real signature.
- Negative asserts STRUCTURAL (match the error enum; no `contains`/`starts_with`; no inlined wat form
  in a `.rs` string). Intentionally-invalid fixtures are `.wat.bad`.
- OMIT a `:user::main` where a fixture only needs to freeze; never fake a `(let [_ 0] nil)` body.
- Do NOT commit until the whole surface is green; the orchestrator weighs + commits.

## Expectations (scorecard — report the REAL result of each)

| what | command | expected |
|---|---|---|
| checker→Tuple, W2a still green | `cargo nextest run -p wat -E 'test(w2a_kwargs_check_mint)'` | PASS (swap still TypeMismatch; positive freezes) |
| Strike-1 runtime runs N=2 | the hand-wired `bracket/uses'` fixture, `--test-threads=1` | mapped results returned, no crash |
| C2 correct wiring runs | `cargo nextest run -p wat -E 'test(<c2 test>)' --test-threads=1` (ok) | `["echo:a·kv:a" …]` |
| C2 swap is a compile error | same (swap) | structural `TypeMismatch` on the two `Address'` |
| order-free | the `:kv … :echo …` variant | identical to `:echo … :kv …` |
| full floor | `cargo nextest run --release` (FOREGROUND) | prior floor + these; 0-new (modulo the 1 known `no_inlined_wat` tracker) |

Runtime band: this is a large 2-strike stone — ~40–70 min. Report the real Summary line + honest deltas.
