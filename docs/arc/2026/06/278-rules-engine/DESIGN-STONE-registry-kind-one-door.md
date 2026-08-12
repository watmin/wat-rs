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

**41 is a SURFACE, not a worklist** — and it is not the worklist we will use.

### ★ THE WORKLIST IS THE COMPILER'S, NOT A CENSUS'S (builder's ruling)

> *"this kind of migration is our bread and butter — we need a rust compiler check to light
> ablaze all heretics who speak the wrong thing — they self identify the migration's targets."*

Two censuses in this doc were wrong. A third would also be wrong, because a grep cannot
distinguish a mention from a lookup, a test from production, or a phase-correct narrow read from
a hole. **Do not survey for the worklist. Impose the wall and read the screams** —
`[[feedback_impose_the_check_and_read_the_screams]]`, and R52 `QVOD LEX ACCENDIT`: the corrected
law lights every violator ablaze, and that fire IS the worklist.

**The mechanism is field visibility.** All five registries are `pub` today
(`symbol_table.rs:33/42/45/62/108`), so reaching directly into one is the path of least
resistance and the omission of the other four is invisible. Make them **private**, expose only:

```rust
pub fn registrations(&self, name: &str) -> RegistrationSet;   // the door — every facet
pub fn macro_only(&self, name: &str)   -> Option<&MacroDef>;  // narrow, PHASE-NAMED,
pub fn type_only(&self, name: &str)    -> Option<&TypeDef>;   //   deliberate, greppable
…
```

Then `cargo build` enumerates **every** site — across `src/`, `tests/`, `crates/`, `examples/`,
including the eleven outside-`src/` files and any this doc's greps never reached. Each screaming
site is fixed to either the door or a **named narrow accessor**, so a surviving single-registry
read becomes an explicit, auditable choice rather than the default that happens because someone
forgot the other four.

The census numbers above are kept as **context for why the door is needed** — never as the
migration's worklist. The compiler owns that.

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

**In — and it is the WHOLE corpus, by the builder's ruling** (*"a half migration is worse"*):

1. `RegistryKind` + `RegistrationSet` + `registrations()`, plus the phase-named narrow
   accessors.
2. **Make the five registry fields private.** This is the wall; everything below is its fire.
3. Fix every site the compiler names — door or named-narrow, per site.
4. `closure_extract` adopts the door, which fixes the missing-constructor bug as a
   *consequence*, not as a patch.

Order matters: the wall is imposed BEFORE the sites are enumerated, because the enumeration is
the wall's output. Writing the wall first and reading its screams is the strike; any list drawn
before it is a guess.

**Out, affirmatively — not deferred:**
- Merging or reordering any registry. Ruled against above.
- The one-entry `child-entry` reshape and deleting `service-forms-def`. Downstream, and blocked
  on this — a clean entry closure still ships uncallable types until the door exists.
- Collecting arbitrary USER macros transitively (see the template section) — a separate
  question with a separate consumer.

## Honest note on size

The corpus fire is **not measured** and this doc will not pretend it is. 41 read sites is a
grep's lower bound on `src/` only; the true count includes `tests/`, `crates/`, `examples/`, and
whatever the pattern could not reach — which is the entire reason the compiler is being made to
produce it. Expect the first build after privatisation to be large, and treat the fail-count as
the progress meter (`docs/SUBSTRATE-AS-TEACHER.md`), not a crisis.

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
