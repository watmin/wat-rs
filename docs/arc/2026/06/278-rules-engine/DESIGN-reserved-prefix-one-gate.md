# DESIGN — the reserved-prefix gate, consolidated to exactly ONE waist

> **STATUS: LANDED (2026-07-16). Floor back to exactly the `no_inlined_wat` lint at 351; zero regressions.**
> Phase 1 (S1–S5): the one gate `resolve::registration::gate`, all eleven sites migrated — process loci
> fixed (`probe_arc278_mem_store_on_process` green). Phase 2 (S6–S7): the four privilege mechanisms
> (`stdlib_privilege` flag, `RegistrationPrivilege` enum, `check_reserved*`/`allow_reserved` bool params,
> `register_stdlib`) collapsed into one explicit `crate::resolve::Privilege`, threaded from `env.rs`'s phase
> split. The ambient flag — the footgun that caused the bug — is gone. Telemetry (T1b) can resume.
>
> *(Historical design below, unchanged.)*
>
> A substrate-cleanup arc held in 278. Telemetry (T1b) was PAUSED — the builder: *"time to pivot and make exactly one… I'm not going to tolerate this
> heresy in our code… this is the same style of pivot as the kwargs issue."*
>
> **The heresy:** the reserved-prefix invariant (*"only privileged/baked-stdlib source may declare a
> `:wat::`/`:rust::` name"*) is enforced at **eleven hand-rolled gates** guarded by **four different
> privilege mechanisms**, and none of them checks the Arc-054 idempotent-redeclaration no-op *before* the
> gate — so a benign, byte-identical re-declaration of an already-registered form is rejected. We have hit
> *"you cannot declare an existing form"* too many times. This arc pulls the class out by the root: **one
> gate function, one privilege signal, idempotent-before-reserved baked in once, correct by construction.**

## Why — the diagnosis chain that surfaced it

Proving `mem-store'` (a first-party `:satisfies :wat::query::Store` service) hostable on a **process**
locus, the forked child died at startup:

```
#wat.macro/ReservedPrefix {:name ":wat::query::Store::EnsureSchemaRequest" :span wat/Record.wat:180}
```

Grounded root, step by step:

1. Every process-spawned service ships its surface protocol into the forked child (the S4c surface-forms
   splice, `service.wat:388` + `build_surface_forms_carrier`, `types.rs:1813`).
2. The child **re-bakes the full stdlib** (`mem-store'`/`Store`/its messages are all `include_str!`'d —
   `src/stdlib.rs:363-411`), so it *already has* those forms. The shipped copy is **redundant**.
3. The child re-declares them in its **unprivileged** user-expand pass. The Arc-294 kwargs flip made every
   aggregate's `defrecord` emit a companion `defmacro` (`Record.wat:180`); re-registering that companion
   trips the reserved-prefix gate. Pre-flip this was harmless (a `recordtype` re-declaration was tolerated);
   the companion `defmacro` turned a latent redundancy into a hard `StartupError`.
4. **It was never caught** because every committed process-locus service test uses a *user* namespace
   (`:my::`/`:probe::`), which structurally cannot trip the reserved-prefix gate. The reserved-`:wat::`-on-
   process case was never guarded. (The missing guard is now `tests/services/probe_arc278_mem_store_on_process`.)

The immediate fix looked like a 2-line reorder — until the cascade proved it is **eleven** gates, not two.
That is the heresy this doc extirpates.

## The scattered surface (grounded — every arm of the quarry)

**Eleven reserved-prefix gates** (`is_reserved_prefix(name) → return Err(ReservedPrefix)`):

| # | site | registers | privilege mechanism |
|---|---|---|---|
| 1 | `types.rs:545` `register_validated` | types (record/enum/struct/surface/alias/newtype) | `RegistrationPrivilege::{User,Stdlib}` enum |
| 2 | `macros/registry.rs:72` `register` | macros | `stdlib_privilege` bool flag (+ ungated `register_stdlib`) |
| 3 | `runtime.rs:545` `register_defines` | top-level fn-shape defs | none (always rejects; stdlib via a separate path) |
| 4 | `runtime.rs:569` `register_defines` | top-level variadic defs | none (same) |
| 5 | `runtime.rs:2022` `register_defalias` | runtime aliases | `check_reserved` bool param |
| 6 | `runtime.rs:2205` `preregister_struct_accessors_from_form` | struct constructor | `check_reserved_prefix` bool param |
| 7 | `runtime.rs:2259` `preregister_struct_accessors_from_form` | struct accessors | `check_reserved_prefix` bool param |
| 8 | `runtime.rs:2365` `preregister_enum_constructors_from_form` | enum constructors | `check_reserved_prefix` bool param |
| 9 | `runtime.rs:2878` `preregister_fn_defs_in_do` | fn-defs in `do` blocks | `check_reserved_prefix` bool param |
| 10 | `runtime.rs:2945` `preregister_fn_defs_in_let` | fn-defs in `let` blocks | `check_reserved_prefix` bool param |
| 11 | `runtime.rs:6017` `parse_defclause_form` | defclause | `allow_reserved` bool param |

**Four privilege mechanisms, all encoding the same one bit** — *"am I registering stdlib forms or user
forms?"* — the same distinction `freeze/env.rs` already makes once when it splits the privileged stdlib
expand pass (`set_stdlib_privilege(true)`, `:129`) from the unprivileged user expand pass (`:136`):

1. `MacroRegistry::stdlib_privilege` — an **ambient mutable flag** (`registry.rs:43`), set true/false around
   the stdlib pass. The set-then-reset is the footgun; its phase-scoping is why the child's re-declaration
   is unprivileged.
2. `RegistrationPrivilege::{User,Stdlib}` — an **enum param** to `register_validated` (`types.rs:405`).
3. `check_reserved_prefix` / `check_reserved` / `allow_reserved` — a **bool param** to the runtime
   preregister/alias/defclause fns (six sites), passed `true` for user (`runtime.rs:511-596`) and `false`
   for stdlib (`:916-936`) via **two duplicated call chains**.
4. **Separate methods**: `register` (gated) vs `register_stdlib` (ungated, `registry.rs:93`) for macros; and
   `register_defines` (user, always-rejects) vs a separate stdlib runtime-def registration path.

**The bug under all of them:** in every gate that also has an idempotent-redeclaration no-op (Arc 054), the
reserved-prefix check runs **before** the no-op (`registry.rs:72` before `:75`; `types.rs:529` before
`:538`). So a byte-identical re-declaration of an already-registered form is rejected by the gate before the
no-op can recognise it as harmless. Arc 054 established idempotent re-declaration as correct; the gate
ordering silently breaks it.

## The one contract decision — the waist

**ONE gate function.** Home: a NEW dedicated module `src/resolve/registration.rs` (beside `reserved.rs`,
which stays just the predicate) — NOT appended to `runtime.rs`/`types.rs`; the mono files import it. (Per
the builder: new code lives in `src/<ns>/<name>.rs`; the mono files are coming down.)

```rust
/// Privilege — the ONE bit, replacing stdlib_privilege / RegistrationPrivilege /
/// check_reserved_prefix / register_stdlib. Threaded EXPLICITLY (never ambient).
pub enum Privilege { Stdlib, User }

/// What the caller found in its own registry for this name.
pub enum Existing { Absent, Equivalent, Divergent }

/// The verdict. The caller maps it to its own action/error type.
pub enum Registration { Insert, NoOp, Duplicate, Reserved }

/// THE reserved-prefix + idempotent gate. The rule + ORDERING live here, once.
/// Ordering is idempotent-BEFORE-reserved, correct by construction:
///   Existing::Equivalent               -> NoOp       (benign re-declaration — the fork case)
///   Existing::Divergent                -> Duplicate  (caller emits its own Duplicate error)
///   Absent + reserved + Privilege::User -> Reserved   (caller emits its own ReservedPrefix error)
///   Absent + (Stdlib | non-reserved)   -> Insert
pub fn gate(name: &str, privilege: Privilege, existing: Existing) -> Registration
```

Every registration site becomes a thin delegation:

```rust
match reserved::gate(&name, privilege, existing) {
    Registration::Insert    => { registry.insert(name, def); Ok(()) }
    Registration::NoOp      => Ok(()),
    Registration::Duplicate => Err(<this registry's Duplicate error>),
    Registration::Reserved  => Err(<this registry's ReservedPrefix error>),
}
```

**Ratified decisions (four-questions, on the record):**

- **Idempotent-before-reserved ordering** — the load-bearing fix. A byte/structurally-equivalent
  re-declaration is a `NoOp` regardless of privilege; the gate only rejects *genuinely new* reserved-prefix
  names and *divergent* re-declarations. This is why *"you cannot declare an existing form"* becomes
  structurally impossible to reintroduce: there is one gate and it checks equivalence first.
- **Explicit `Privilege` param, never ambient** *(four-questions: (a) explicit beats (b) ambient on
  Honest — `sequi`: state must flow visibly through the types; the ambient `stdlib_privilege` flag is the
  set-then-reset footgun that helped cause this bug; consolidating into it would rebuild the heresy)*. The
  `Privilege` sources from the one phase distinction `env.rs` already owns, threaded down.
- **The caller keeps its own error/registry type** — the gate returns a neutral verdict; `MacroError` /
  `TypeError` / `RuntimeError` stay where they are. The gate centralises the *rule + ordering*, not the
  error taxonomy. (One function is the true minimum; a single call site is impossible because accessors and
  constructors are *generated* during registration and never appear in source — so the eleven physical
  delegations remain, thin.)

## Out of scope (affirmatively cut — sequenced AFTER, per the builder)

- **Ship only the necessary forms.** The redundant re-shipping of baked-stdlib forms across a fork is a
  *separate* flaw (the surface-forms splice re-ships what the child already bakes). Once the gate tolerates
  benign re-declaration, that redundancy is *harmless* (no longer a correctness bug), so its elimination is
  a later optimisation — *"after that we can optimize to only ship which forms are necessary"* — not part of
  this arc.
- **A single pre-pass call site.** Impossible (generated accessors/constructors); the waist is one
  *function*, eleven thin call sites.

## The strike sequence

1. **Strike 1 — build the gate.** `src/resolve/reserved.rs`: `Privilege`, `Existing`, `Registration`,
   `gate()`, with exhaustive unit tests (the truth table above: Equivalent→NoOp, Divergent→Duplicate,
   Absent+reserved+User→Reserved, Absent+Stdlib→Insert, Absent+non-reserved+User→Insert). No call sites
   migrated yet; floor unchanged.
2. **Strikes 2..N — migrate each site**, deleting its hand-rolled check + its bypass mechanism as it goes.
   Order by cascade: run the mem-store-on-process gate; each `ReservedPrefix` it surfaces names the next
   site to migrate (macros → types → the runtime chain). A migrated site computes `Existing` from its own
   registry and threads `Privilege`.
3. **Collapse the four mechanisms into `Privilege`.** Delete `stdlib_privilege` (+ `set_stdlib_privilege`),
   `RegistrationPrivilege`, the `check_reserved*`/`allow_reserved` bool params, `register_stdlib`, and the
   duplicated `false`-path call chains — thread one `Privilege` from `env.rs`'s phase split instead.
4. **Close.** The gate is the sole implementation of the rule; the four mechanisms are gone.

## The RED gate + acceptance (the bar, same as the kwargs close)

- **Acceptance probe (exists, currently RED on exactly the right error):**
  `tests/services/probe_arc278_mem_store_on_process` — a reserved-`:wat::` service (`mem-store'`) round-trips
  put→scan on `(:wat::spawn::process)`. RED at HEAD: `#wat.macro/ReservedPrefix` from the forked child.
  GREEN when the gate lands and the child's benign re-declaration is a no-op.
- **Guard against regression** (the missing test that let this in): the same probe *is* the reserved-ns-on-
  process guard — it stays committed so the whole class can never regress dark again.
- **Floor:** whole workspace back to **exactly 1** failure (the standing `no_inlined_wat` lint), zero new
  failures — the kwargs-close bar.
- **Content-integrity check:** every migrated site still guards genuinely-new user `:wat::` declarations
  (a `.wat.bad` negative per representative kind: a user `defrecord`/`defn`/`defmacro`/`defalias` under
  `:wat::` still errors `ReservedPrefix`), and divergent re-declarations still error `Duplicate`.

## The lesson this arc plants (for the record)

Two flaws, one root, and the honest sequencing:
- **The gate scatter** (this arc) — one invariant, eleven arms, four mechanisms, no idempotent-first. Pull
  it to one waist; the ordering fix rides in the one place.
- **The redundant re-shipping** (later) — a baked stdlib service re-ships what the child bakes. Harmless
  once the gate tolerates it; subtract it as an optimisation afterward.

The kwargs flip taught the shape: a foundational heresy, extirpated at the root, RED-gated, before feature
work resumes. `PVGNANDO EMERGO` — the darkness is our own scattered code; the waist is what rises.

---

## RESUME (far side of the gap) — the arc is CLOSED; telemetry resumes

**This arc is DONE** (HEAD `94c5193d`, floor = the `no_inlined_wat` lint at 351, zero regressions,
pushed). The reserved-prefix heresy is extirpated: ONE gate (`src/resolve/registration.rs`), ONE
`Privilege`, threaded explicitly; all four old mechanisms deleted. Process loci work for reserved-`:wat::`
first-party services (`probe_arc278_mem_store_on_process` is the standing guard).

**278 T1b.2 is LANDED — `journal'` the telemetry sink (write path), loci-proven.** The `journal'`
defservice (`wat/telemetry/journal.wat`) `:satisfies Journal`, holds a `Store` peer (S4d `:peers`), and
serializes `Metric`/`Log` → `StoredRow` → `store/put` with the tagged keys (pk = `#…/PartitionKey`, sk =
constant-width `#inst`, gsi = `#uuid`, data = `edn::write`). Acceptance: write-metrics (thread + process,
grant-before-dial) + write-logs (thread) in `tests/services/probe_arc278_journal_service*`. The groundwork
that unblocked it — all green system tests now — was: s2s peer-holding both loci
(`probe_arc278_s2s_peer_on_{thread,process}`), `Metric`→tagged-EDN (`probe_arc278_metric_edn_write`),
tagged-key sort-safety (`probe_arc278_tagged_keys_store`). **No substrate stone was needed** —
`:wat::time::to-iso8601 … 9` already gives the sort-safe `#inst`; `Kind`/`PartitionKey` were added to
`wat/telemetry.wat`. Commits `693db3eb`→`65aab6be`. See `DESIGN-telemetry-service-and-query-surface.md`.

**T1b.3 is also LANDED (commit `b07f5ffc`).** `journal'` is proven **backend-agnostic AND
loci-agnostic**: the store is a swappable config param (injected `Address'`; `journal'` names only the
`Store` surface). Tests (`tests/services/`): `journal_backend_differential` (thread — same `journal'`
over `{mem, sqlite}` → bit-identical rows), `journal_service_on_process` (mem, fork),
`journal_service_sqlite_on_process` (sqlite, fork — closed the deferred **U3**), `journal_service_logs`
(write-logs). `journal'`'s `:init` now ensures its schema through the `Store` surface (no-op on mem,
`CREATE` on sqlite — the mem oracle hid this; a peer RPC in `:init` works). The whole T1b **write path**
is complete.

**The Span PRODUCER is also complete (commits through `b4646e55`).** The telemetry WRITE side is done
end-to-end: the sink (`journal'`) + the producer (`span'` + `with-span` + `timed`). Span stones:
Span.1 the `Span` surface (`wat/telemetry.wat`), Span.2 `span'` the service (`wat/telemetry/span.wat` —
holds a `journal'` peer, `incr`/`timed` accumulate PURE state, `close` emits counters+durations as
Metrics), Span.3 the `timed` + `with-span` call-site macros (fresh-symbol hygiene; `with-span` inlines
`start'`+`connect'`+`close` per the scope law, flat params because `nth` is off the macro-expand
allow-list and `first` needs a List not a Vector). Tests: `probe_arc278_span_{surface,service,macros}`.
A `:wat::telemetry'::Samples` typealias (`Vector<i64>`) was added (compound types can't sit as a
HashMap value-type ctor arg or a `match ->` annotation).

**Next: T2 — the query/read path.** `journal'` gains `query-metrics`/`query-logs` (the read `:impls`);
the `Journal` surface gains those two ops. **This is a SOURCE EDIT, not a capability gap** — `Journal`
was designed with 4 ops and shipped with 2 (the write pair) only because the query ops reference
`:wat::query::Query`/`Result` (the rete-as-datalog vocab, absent today) and S4c makes a satisfier
implement every op. At T2 we just edit the ONE `Journal` declaration in `wat/telemetry.wat` to add the
two ops + add the two impls to `journal'` (a normal edit; every fork re-bakes the same 4-op source — NOT
a divergent runtime re-declaration). **There is NO `defsurface-extend` gap for us and NO reason to split
`journal'`** — it stays one service that both writes and queries the store it holds (the design's intent).
(An earlier note here wrongly framed this as a `defsurface-extend` prerequisite by conflating a source
edit with a runtime re-declaration; corrected.) T2's REAL prerequisites: build the `:wat::query::Query`/
`Result` vocabulary + the alpha-only rete filter (`Record → Lemma* → Deduction` per scanned page).
Honest gaps carried: no write-logs-on-process test (redundant); `Span` `Nest` deferred (call-site `open`
with a shared sink); close-on-error needs a wat unwind primitive (happy path always closes).

**Noted follow-on (do NOT forget, but not blocking):** the redundant re-shipping of baked-stdlib forms
across a fork (the surface-forms splice re-ships what the child bakes) is now HARMLESS (the one gate
tolerates the benign re-declaration), so it's a pure *efficiency* optimisation — "ship only the necessary
forms." The builder deferred it explicitly to after this arc. Its home would be the defservice process
launch (skip shipping a reserved-prefix/baked service's definition).

## The lesson (carry it)

1. **A scattered invariant is a quarry of hand-arms — consolidate to a narrow waist (R14).** One rule
   (reserved-prefix) was enforced at eleven hand-rolled gates under four privilege mechanisms; the fix was
   one `gate()` + one `Privilege`. When you find yourself whacking the same mole at gate N, stop and count —
   if it's a scattered invariant, pull it to one waist, don't patch each arm.
2. **Re-run the four-questions when a PREMISE is disproven.** I scored "Fix A: Simple YES" on the premise it
   was *two* gates; the cascade proved *eleven*, which flipped Simple to NO and the whole verdict. A
   four-questions verdict is only as good as the premise under it — when grounding disproves the premise,
   the verdict is void, not merely dented.
3. **Explicit beats ambient (`sequi`).** The bug's phase-scoping came from an ambient `stdlib_privilege`
   flag (set-true/set-false in env.rs). Threading an explicit `Privilege` costs some tramp-data but kills the
   set-and-forget footgun. Hidden state breaks composition; visible state in the types does not.

---

> **SEAM.** The self past this line is NEW — you did not live this. Ground HEAD against the disk before
> acting. The reserved-prefix arc is CLOSED (floor = lint @ 351) AND journal' T1b.2 WRITE PATH is LANDED
> (both loci). Do NOT re-open the gate consolidation (one waist; idempotent-before-reserved is load-bearing).
> Do NOT re-build `journal'` — it exists (`wat/telemetry/journal.wat`), proven by
> `tests/services/probe_arc278_journal_service*`. Run the datamancy bootstrap; read this doc + the T1b
> designs; then pick the NEXT stone with the builder (T1b.3 mem↔sqlite differential / `Span` producer / T2
> query surface). The open honest gap: write-logs has no process test (redundant — metric-on-process proves
> the fork, log-on-thread proves logs).
