# DESIGN-STONE — top-level names must be namespaced, and NOTHING enforces it today

> **Status: RULED, NOT BUILT.** The builder, 2026-08-01: *"we must impose that all rules are
> namespaced... the only thing that's ever allowed to not be namespaced is arg and let."* And, on
> finding out no definer imposes it: *"i thought defn already imposed it......... this is a major
> flaw."*

## The flaw, confirmed on the local build

```clojure
(:wat::core::defn      :no-ns   [] -> :wat::core::i64 0)
(:wat::core::defrecord :AlsoBare [x <- :wat::core::i64])
```

`--check` clean. Runs. Prints `0`.

Confirmed against `./target/release/wat`, **not** the installed binary — the builder's own repro
carried the stale-install warning, and this project has been bitten before by reasoning about one
build while measuring another. The flaw is current, not an artifact.

**No definer enforces it.** Not `defn`, not `defrecord`, not `defrule`, not any of them. There is no
lint either — `grep` over `src/`, `tests/lint/` and `CONVENTIONS.md` finds no rule of this shape.
The convention is documented as a *namespace table*; nothing rejects a name that ignores it.

The dangerous part is not the gap. It is that **the builder believed it was enforced.** A rule
everyone treats as guaranteed, which silently is not, is worse than a known-absent rule: nobody
checks for violations of a wall they think is standing.

## How it surfaced, which is its own small lesson

The `rule-record → defrule` codemod (`5b86828f`) minted **89 bare rule names** — `:arith`,
`:accessor`, … — and every gate stayed green. The rider had *probed* that a bare symbol is legal wat
and reported that as the resolution; the orchestrator accepted it and **praised the migration**.

Two failures stacked:

1. *Permitted ≠ conformant.* "The checker allows it" was read as "this is correct."
2. **The gate that certified the migration compares derived fact SETS. It structurally cannot see a
   naming violation.** A green gate certified a non-conformant corpus — R59's class exactly: the
   check passed because nothing in it depended on the property in question.

## Why the SHARED GATE is the right rung, not `defrule`

`defrule` is one door. `defn`, `defrecord`, `defenum`, `defservice`, `defsurface` and `defclause` all
mint top-level names too, and patching each macro is the convention rung wearing a wall's clothes.

They already share one:

```rust
// src/resolve/registration.rs:76
pub fn gate(name: &str, privilege: Privilege, existing: Existing) -> Registration {
    match existing {
        Existing::Equivalent => Registration::NoOp,
        Existing::Divergent  => Registration::Duplicate,
        Existing::Absent => {
            if privilege == Privilege::User && is_reserved_prefix(name) { Registration::Reserved }
            else { Registration::Insert }
        }
    }
}
```

It already takes the name and already reasons about its **shape**. An `Unnamespaced` variant sits
directly beside `Reserved`.

**★ And the domain fits exactly.** The rule is *"only args and let-bindings may be bare."* Args and
let-bindings are **lexical** — they never reach a registration gate. Everything that *does* reach
`gate()` is a top-level registration. So *"everything the gate sees must be namespaced"* is a precise
restatement of the rule with **no exceptions to carve out** — which is what makes this a wall rather
than a heuristic. Heuristics with exception lists rot; this one has none.

## The measurement — and the condition is nearly ideal

Counted with a pattern **validated against a file whose answer was known first** (two earlier greps
gave 0 and 5435, both wrong — one filtered on a string containing the prefix it excluded, the other
omitted `:` from the character class so every namespaced name truncated and looked bare):

| scope | bare top-level names |
|---|---|
| `wat/` — the stdlib | **0** |
| everything else | **148**, of which **89** are the corpus, minted 2026-08-01 |

**The stdlib is already 100% conformant.** So the wall can be armed at **zero stdlib offenders** —
the same condition as the `push_back` lint (task #41), *"turned on at zero offenders."* The ~59
non-corpus violations are scattered scratch probes and a handful of old test fixtures.

## ~~⛔ THE BLOCKER~~ — RETRACTED 2026-08-01, it was mis-aimed

> **The blocker below was wrong. Read the retraction before acting on the order of work.**

The original text (kept, per *what is inscribed is inscribed*):

> From the 24w record: the **verb-side** rejection path currently only `eprintln!`s instead of
> returning a located error, because `from_symbols` returns `CheckEnv`, not `Result`. Its own note:
> *"It has never fired; a warning is not a wall."* If `Unnamespaced` lands on that path as-is, it
> **warns and the bare name registers anyway**. **Fix the eprintln path before arming.**

**What the disk actually says.** That `eprintln!` is at `src/check/env.rs:158`, inside a loop that
**replays an ALREADY-FROZEN symbol table**, called at `Privilege::Stdlib`, and its own comment states
the reason: *"reserved-prefix policing for user code happens at define-registration, upstream of the
freeze, so re-asserting it here would reject the stdlib's own `:wat::` functions."* **It is not a
user-facing door.** It is a replay, and it cannot be the site where a user's bare name slips through,
because the user's name was already gated upstream.

The user-facing doors emit **located, hard errors today**:

| door | `file:line` | on `Reserved` |
|---|---|---|
| all types (`defrecord`/`defenum`/`defstruct`/`defsurface`/`typealias`) | `src/types.rs:557` | `TypeError::new(span, ReservedPrefix { name })` |
| user `def` (fn-shape) | `src/runtime.rs:917` | `RuntimeError::new(form_span, ReservedPrefix(path))` |

`Unnamespaced` added beside `Reserved` lands on those same arms and inherits the same located
rejection. **Step 3 is a real OWED item on a different path; it does not gate step 4.**

*Kept visible because the record carried a false blocker for a day, and the correction only came from
reading the callers — the grounding this stone's own "what is NOT grounded" section demanded.*

## Order of work

1. ~~**Rename the corpus's 89**~~ — DONE (`b096e779`). *Measured correction: it was 99 occurrences /
   91 unique names, not 89.*
2. **Clear the remaining bare names** — **MEASURED 2026-08-01: 57 occurrences across 24 files**
   (`wat-scripts/scratch-pad` 25 · `tests/types` 17 · `wat-tests/counter-actor-proof-process.wat` 6 ·
   `wat-scripts/fixes` 5 · `tests/resolve` 4). **stdlib `wat/` = 0, corpus = 0.**
3. **Arm the gate** — `Unnamespaced` beside `Reserved`. **No longer blocked on the eprintln path.**
4. *(Separate, still owed)* make the `check/env.rs:158` replay rejection a located error rather than
   an `eprintln!` — 24w's OWED, on its own merits.

**★ The arming is itself the enumeration, twice over.** `Registration` is matched exhaustively at
~11 call sites (`types.rs:552`, `macros/registry.rs:63`, `check/env.rs:322`, and `runtime.rs`
`:916` `:949` `:2496` `:2682` `:2743` `:2856` `:3383` `:3457` `:6522`). Adding a variant makes
**rustc name every door that must decide** — no grep, no caller map. Then freezing the corpus makes
**the 24 wat files name themselves**. R52 `QVOD LEX ACCENDIT` at both layers.

Order 2-vs-3 is free: the wall fires only on a *first* registration (`Equivalent → NoOp` short-circuits
ahead of it), and neither stdlib nor the corpus holds an offender.

## Rulings (2026-08-01)

- **`Privilege::Stdlib` IS held to it.** It costs nothing (stdlib is already 0), and exempting the
  code most likely to be copied as an exemplar is the imitation-tell 24t named (*"a scaffold left
  standing becomes architecture, and the tell is IMITATION"*).
- **One `::` is enough.** No required root: `:usr::`, `:my::`, `:app::`, `:probe::` are all legitimate
  and in live use. The predicate is `name.contains("::")` — **NOT** "starts with `:` and contains
  `::`", because parametric heads drop the leading colon (`wat::kernel::Peer`, recorded in 24t).

## The one genuine unknown — STOP-1 of the strike

**Generated names**: struct/record accessors (`Type/method`), enum variants, macro-minted companions,
the surface-minted `<Surface>::<op>/Request|Response` aliases (`69d7dd5a`). These reach the gate at
`runtime.rs:2743` / `:2856` and elsewhere. They are *derived* from a parent name, so a namespaced
parent should yield a namespaced child — **but the exact registered string has not been read.** If any
of the substrate's own emissions registers bare, arming breaks the substrate before it touches a single
heretic. That is a STOP, not an exemption to carve.
