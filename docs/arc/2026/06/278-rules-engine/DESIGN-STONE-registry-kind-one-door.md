# DESIGN STONE — `RegistryKind`: one door over the five registries

**Arc 278. Ruled 2026-08-11 by the builder: *"we need an enum for registries that we walk."***

## The defect

A name in this substrate can be registered in **five** places, each answering at a different
phase:

| registry | phase | on `SymbolTable` |
|---|---|---|
| `macro_registry` | **EXPAND** | `:45` |
| `types` (`TypeEnv`) | **CHECK** | `:62` |
| `unit_variants` | EVAL | `:42` |
| `runtime_def_values` | EVAL | `:108` |
| `functions` | EVAL | `:33` |

**There is no resolver across them.** A grep for one returns nothing. Every consumer that must
answer *"what is registered under this name?"* asks each registry by hand, from memory, forever.

### ⚠ THE FIRST CENSUS IN THIS DOC WAS WRONG — corrected 2026-08-11, kept visible

The original table listed "six files hand-consulting three or more registries"
(`runtime.rs`, `symbol_table.rs`, `freeze.rs`, `closure_extract.rs`, `freeze/env.rs`,
`check/env.rs`). It was produced by a **file-level co-occurrence grep**, which cannot tell a
name LOOKUP from a mention — so it counted setters, bulk iteration, and test lines as
consumers. **The builder ruled "migrate all six" off that table.** The number was wrong in both
directions: four of the six are not resolution consumers at all (an owner, a `set_macro_registry`
call, a test line, a bulk `for` over two registries), and it MISSED five files that are —
including the largest by a factor of three.

The corrected census, counting actual cross-registry READ sites:

```
27  src/check.rs              ← absent from the first table entirely
 8  src/runtime.rs
 2  src/closure_extract.rs    ← the proven-wrong one
 1  src/types.rs
 1  src/rete/matcher.rs
 1  src/rete/compiled_rhs.rs
 1  src/resolve/walk.rs
───
41  read sites across 7 files   (includes test code — not yet separated)
```

**41 is a SURFACE, not a worklist.** Many of those sites are legitimately single-registry: at
CHECK time, asking only "is this name a type?" is phase-correct, not a hole. Handing a rider
"migrate 41 sites" would repeat the original error at larger scale.

**PREREQUISITE, and it gates the brief:** a per-site disposition over all 41 — for each,
*phase-correct single-registry lookup* (leave) vs *resolution that should ask the door*
(migrate) — with the reason recorded per site. That inventory IS the migration worklist. No
migration may be briefed off a count.

`closure_extract` asks four of five. It never reads `macro_registry`. That is not a missing
*entry* — it is a missing **dependency kind**, and it shipped a forked child a record type with
no constructor (`probe-arc278-union-closure-boots-a-process-child.wat`, proven by run:
`2 unresolved references … :probe::ffx::Record — call head, not a registered function`).

This is the class task #75 already pulled once — *"ONE DOOR for a type head's FQDN — 17
hand-rolls collapsed"* — one level up. There it was 17 spellings of one derivation. Here it is
six hand-consultations with no derivation at all.

## The census — and it CLOSES the open fork

`tests/reflection/probe_arc278_registry_census.rs`, over a frozen `defservice` world:

```
names registered anywhere : 2489
names in >1 registry      : 207

  [Macro, Type]        182   a record's type + its kwargs constructor
  [Function, DefValue]   25   every defn — `defn` expands to (def :n (fn …)), core.wat:1175
```

**Nothing in three or more. Nothing where one name means two UNRELATED things.**

So a name maps to a **set of facets of one concept**, never a set of rivals. The fork that was
open — *does the resolver need a precedence ruling?* — is **closed: it does not.** The resolver
returns every facet; the caller picks the one its phase needs.

The evaluator's existing order (`runtime.rs:4431-4454`: `unit_variants → runtime_def_values →
functions`) is consistent with this and is **not** a conflict resolution: where `[Function,
DefValue]` co-occur they are the same `defn`, so either answer is the same thing. It is a
lookup order, and the enum does not disturb it.

## Why macros belong in the enum — the non-obvious half

At **eval** time a name never resolves to a macro; expansion already happened. That is exactly
why `closure_extract` — which copied the eval-time chain — has no macro step, and why the
omission looked correct for two arcs.

But closure extraction is not resolving *for eval*. It is collecting *for shipping*, and **the
forms it ships still contain macro CALLS**. `(:probe::ffx::Record :tag …)` in a shipped child
program is a kwargs-constructor invocation that expands **in the child**. The child therefore
needs the macro. "We ship expanded forms" means expanded w.r.t. the OUTER macro (`defservice`);
inner constructor calls remain and expand on the far side.

**A consumer's correct registry set depends on its phase, and no consumer can be trusted to
re-derive that from memory.** That is the whole argument for the door.

## The four questions

| option | Obvious | Simple | Honest | UX |
|---|---|---|---|---|
| **(a)** teach `closure_extract` to also read `macro_registry` | YES | YES | **NO** | — |
| **(b)** `RegistryKind` enum + one resolver returning facets | YES | YES | YES | YES |
| **(c)** merge the five registries into one table | **NO** | **NO** | **NO** | — |
| **(d)** document the five, rely on reviewers | YES | YES | **NO** | — |

- **(a) fails Honest** — it fixes today's missing kind and leaves the *class*. The next registry
  is missed the same way, and the next child dies at startup to tell us.
- **(c) fails Obvious and Simple** — the registries are split by **phase**, and that split is
  load-bearing: a macro must resolve before the checker exists. Fusing them is the OOP move
  (weld distinct concerns because they share a key) that R28 `SOLVIMVS NE MENTIRETVR` exists to
  refuse. Their separation is correct decomplection; the missing thing is a query surface, not
  a merger.
- **(d) fails Honest** — it is the convention that already failed, written down.

**(b), 4/4.**

## The design

```rust
/// Every registry a name can be registered in. Exhaustive BY LAW: the
/// `_`-wildcard ban on enum scrutinees means adding a variant makes every
/// consumer's match RED until it decides what the new kind means.
pub enum RegistryKind { Macro, Type, Function, UnitVariant, DefValue }

/// The ONE DOOR. Returns every facet registered under `name`.
/// Empty ⇒ the name is unregistered (a keyword literal at value position).
pub fn registrations(&self, name: &str) -> RegistrationSet;
```

The load-bearing property is **not** the convenience. It is that a sixth registry becomes
**unabsorbable**: adding `RegistryKind::Whatever` turns every exhaustive match red, and the
compiler hands back the located worklist of every consumer that must now decide. That is R65
`SCVTVM IDEM INDEX` — the exhaustiveness we pay for daily *is* the ledger — and it is why the
enum, not a helper function, is the right shape.

### The ladder (`extirpare`)

- **convention** (today) — remember to ask all five. Already failed, once provably.
- **check** — a gate that fails if a registry exists no resolver consults.
- **unrepresentable** (this stone) — one door, one enum, and a new kind cannot be silently
  skipped because there is no wildcard to swallow it.

## Scope

**In:** the enum; the resolver on `SymbolTable`; migrating `closure_extract` to it (the one
proven-wrong consumer), which fixes the missing-constructor bug as a *consequence* rather than
as a patch.

**Out, affirmatively — not deferred:**
- Migrating the other five consumers. They are not known-wrong, and a migration is only honest
  once each one's *phase-correct registry set* is stated. Tracked as its own stone.
- Merging or reordering any registry. Ruled against above.
- The one-entry `child-entry` reshape and deleting `service-forms-def`. Downstream, and blocked
  on this — a clean entry closure still ships uncallable types until the door exists.

## Gates

1. A RED probe: closure extraction over a fn constructing a **macro-synthesized** record ships a
   program whose child can call the constructor. Fails today, named at
   `wat_edn_bridge.rs:442`.
2. The census test stays green and non-vacuous (it asserts >0 multi-registry names, so a
   refactor that flattened the registries would fail loudly rather than read clean).
3. Floor `4388/4388`, clippy 0, by the orchestrator's own `--release` re-run.

## The template question — SETTLED for this stone's class, bounded beyond it

`walk_defmacro_form` (`closure_extract.rs:1105`) returns `Ok(())` on the stated assumption that
a template *"is a macro template, not an executable expression … self-contained."* If a
template could reference a user type or fn, macro collection would need its own transitive walk.

**Measured** — the generated kwargs constructor, read off an actual emitted manifest:

```clojure
(defmacro :probe::ffx::Record [& call-args <- Vector<WatAST>] -> WatAST
  (let [_kc-type (keyword-node ":probe::ffx::Record")]     ; a STRING, not a symbol
    (quasiquote (kwargs-construct (unquote _kc-type) (unquote-splicing call-args)))))
```

The type name enters only as a **string literal**; `kwargs-construct` is a substrate builtin.
**No symbolic dependency.** The assumption HOLDS for generated constructors — the whole class
this stone must ship — so macro collection here is a flat lookup, not a transitive walk.

**Bounded, not generalised:** this says nothing about an arbitrary USER `defmacro`, whose
template may reference anything. Collecting user macros transitively is a separate question
with a separate consumer, and it is **out of this stone's scope** — named here so the next hand
does not read "macros are self-contained" as a general law. The census's `[Macro, Type]` cohort
is 182 names and every one of them is a generated constructor of this shape.
