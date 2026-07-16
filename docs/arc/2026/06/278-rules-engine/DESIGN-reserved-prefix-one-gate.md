# DESIGN — the reserved-prefix gate, consolidated to exactly ONE waist

> **STATUS: ratified design, unbuilt. A substrate-cleanup arc held in 278.** Telemetry (T1b) is PAUSED
> until this lands — the builder: *"time to pivot and make exactly one… I'm not going to tolerate this
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

**ONE gate function.** Home: `src/resolve/reserved.rs` (beside `is_reserved_prefix`).

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
