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

## ⛔ THE BLOCKER — fix this first or the wall ships already holed

From the 24w record: the **verb-side** rejection path currently only `eprintln!`s instead of
returning a located error, because `from_symbols` returns `CheckEnv`, not `Result`. Its own note:
*"It has never fired; a warning is not a wall."*

If `Unnamespaced` lands on that path as-is, it **warns and the bare name registers anyway**. Shipping
a decorative wall to close a gap caused by a wall nobody built would be the exact failure this stone
exists to end. **Fix the eprintln path before arming.**

## Order of work

1. **Rename the corpus's 89** (codemod; must derive the printed row label by stripping the namespace,
   so the untouched `.clj` oracle still matches — see the rename brief).
2. **Clear the remaining ~59** (scratch probes, old fixtures).
3. **Fix the verb-side rejection** so it is a located error, not an `eprintln!`.
4. **Arm the gate** — `Unnamespaced` beside `Reserved` — at zero offenders.

Steps 1–2 are safe ahead of the wall. Step 4 must not precede step 3.

## What is NOT grounded

`gate()` has been read. **Nothing else of the registration path has.** Threading a new `Registration`
variant through the error taxonomy and every caller is a real change of unknown size, and it will not
be estimated from one function body. **Grounding the callers is the first act of the stone.**

Also unruled, and the builder's call:

- **Does `Privilege::Stdlib` get held to it too?** It costs nothing today (stdlib is already clean),
  and exempting it would leave a hole for exactly the code most likely to be copied as an exemplar.
- **What counts as namespaced** — is one `::` enough, or is there a required root?
- **Generated names**: accessors (`Type/method`), enum variants, macro-minted companions. These pass
  through registration too; the stone must not reject the substrate's own emissions.
