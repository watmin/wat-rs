# SCORE — STONE 255.6: rete's clause grammar adopts the door

Branch: `main`. **Committed, not pushed.** Drawn against `e3c40e3da` (draw `d31639b85`).
Parent: `BRIEF-STONE-255.6-rete-adopts-the-door.md`.
Floor / workspace clippy / census: orchestrator. Crate clippy + lint suite run here.
**No corpus `.wat` converted.** New fixtures only. 8d-ii not started.

## Gate: the 179-file delta — ONE tree

Same spread as 255.1–255.5: every 12th of the 8d-i census `files.txt` (**179**).
Originals: **live tree**. Converted: `/tmp/255-6/conv` — **re-converted with the CURRENT
codemod** (8d-i-b moved corpus and tool). Timed one file first: pprintln copy **0.30 s**;
batch 179 **31.32 s**.

| | this tree at 255.5 / 8d-i-b | this stone |
|---|---|---|
| originals clean | **161 / 179** | **161 / 179** |
| still clean after conversion | 95 | **100** |
| ⛔ **REGRESSIONS** | **66** | **61** |

**5 closed. 0 newly_broken. Net −5.**

Closed: `probe_arc278_4b_cascade` · `probe_arc278_fence_binder_shadow` ·
`probe_arc278_rete_edn` · `wat-scripts/perf/grid/asym-join` ·
`wat-scripts/perf/grid/retract-multiplicity`.

### Classification (61)

| cause | n | vs this tree at 255.5 (66) |
|---|---|---|
| `UnresolvedReference` | **23** | same |
| `ReteCheckErrors` | **16** | 21 → 16 |
| `MalformedDecl` | 9 | |
| `CheckErrors` / OTHER | 13 | |

## What landed — one door, not a second Symbol arm

`canonical_identity` is now the rete clause-head / fact-bind-type / constraint-head
key. Keyword **or** Symbol → one dispatch.

- `form_match::identity_text` — the dual-read sibling of `keyword_payload`.
- `classify_rete_clause`: `?`-prefixed symbol stays bind/fact-bind/accumulate
  (shape, not spelling). Every other head goes through identity, then **one**
  `match` (combinator / constraint / predicate). Not a second Symbol arm.
- `classify_constraint_head`: identity first, then the existing vocabulary door.
- `alpha_pattern`: the actual fact-pattern door for `(vrm/F …)` / `(Type/Name …)`.
  `?`-headed FactBind via the classifier; otherwise identity_text, colon-stripped.
  Lookup already canonicalizes (`TypeEnv::get`).
- Field bind stays a **keyword** (`:k`). `(?k :- k)` is still Unrecognized.

### ⭐ THE TWELFTH CORRECTION — the brief asked `is_known_type`

`classify_rete_clause` is documented **Independent of TypeEnv**, by SHAPE alone.
`is_known_type` needs a registry. Code wins: after `canonical_identity`, a
namespaced type has `::` (`weather/ColdAndWindy` → `:weather::ColdAndWindy`);
a field `:k` does not. Same discriminator as before, spelling-independent.

## Non-vacuity both ways

- `(weather/Temperature (?loc :- :location) (wat.rete.i64/< ?c 20))` **freezes**.
- `(?fact :- weather/ColdAndWindy)` **freezes**.
- `(?k <- :k)` inside `(vrm/F …)` is still **`MalformedClause`** — the symbol
  fact-pattern head is now legal; the retired arrow is not. Fixture
  `tests/rete/probe_arc255_6_rete_adopts_the_door.wat.bad`.
- Live rust-scheme northstar still `--check` rc=0.

## Measured, then touched

**`expr_is_provably_boolean`:** converted `:where` interiors do **not** reach it
today (`check_fence_interior` does). After head unification the Predicate arm
**would** — a Keyword-only gate there is a silent `false` for
`wat.rete.core/and` / `wat.string/empty?`. Wired through the same door.

**`check_fence_interior`:** converted `:where` interiors **do** reach it
(`wat.rete.core/or`, `wat.rete.string/=`). Keyword-only **fail-opened** (skipped
let/match shadow checks and constraint typing). Same door; wired.

## Generators of heads

`tests/lint/rete_bind_generators.rs` still green. `wat/query.wat` mints
`(?fact :- ~tkw)` with `keyword-node` from a type **string** — a fact-bind, not
a fact-pattern head, and already `:-`. No constructor of `(Type/Name …)` clause
heads found. :then heads in the corpus are source literals; the codemod rewrites
them and the **then validator** still wants a Keyword (below).

## ⭐ FINDING: ReteCheckErrors 21 → 16, remainder is `:then`

The five closed files failed on **`:when`** (fact-pattern head / fact-bind type /
combinator). The remaining **16** fail on **`:then` insert heads**:

```
validate_then_form: fact_items[0] must be WatAST::Keyword
converted `(weather/ColdAndWindy :location ?loc)` → MalformedClause `weather/ColdAndWindy`
```

The clause grammar this stone owns is `:when`. `:then` is a third one-spelling
slot. Named, not forced — same posture as the three prior declines, now on the
next class.

### Also named, not forced

- **`wat/rete/compile.wat` `compile-condition`** still exact `ast-name`
  `":wat::rete::where"` etc. A second parser. Freeze (`--check`) does not run it;
  8d-ii load will. Not a `.wat` this stone converts.
- **`fn` param `<-`** — 8d-iii.

## Test count

Predicted **+7** (4 dual-spelling + 1 clojure constraint-head + 2 freeze probes).
`cargo nextest list --release -p wat`: **5331** (was 5324).

## Walls I ran

- crate clippy `-p wat --all-targets --release -D warnings` — **0**
- `one_param_spec` / `rete_bind_generators` / `one_variant_separator` — pass
- `constraint_head_tests` (incl. `every_constraint_head_is_a_real_rete_row`) — pass
- `probe_arc255_6_rete_adopts_the_door` (accept + retired-arrow refuse) — pass

Floor / workspace clippy / census: orchestrator. Do not push. Do not start 8d-ii.

---

# ORCHESTRATOR'S WEIGH — independent re-run, 2026-09-21. **ACCEPTED.**

| row | result |
|---|---|
| `scripts/floor.sh` | ✅ **5953/5953 passed**, exit 0 |
| clippy `-D warnings --all-targets --workspace` | ✅ **0** |
| `census.sh --diff` | ✅ `no STOP-8` |
| no corpus `.wat` converted | ✅ **0** |
| ⭐ **THE DELTA — my tree, freshly re-converted** | **66 → 61.** Classification matches the SCORE **row for row**: UR 23 · rete 16 · defsurface 8 · programbody 3 · unknown 1 |

⭐ **Two independent measurements, identical numbers AND identical classification.** That has not
happened before in this arc — earlier stones agreed on direction and net but not on every row.

## ⭐ NON-VACUITY BOTH WAYS — the widening did not leak

```
(vrm/F (?k <- :k))   retired arrow  → MalformedClause   ← STILL REFUSED
(vrm/F (?k :- :k))   current form   → accepted
```

**The symbol fact-pattern head is now legal; the retired arrow is not.** Verified on the binary,
which is exactly what a stone that loosens a parser owes.

## ⭐ THE TWELFTH CORRECTION — and the executor was right

**The brief said: *"Ask the registry — `is_known_type`. Replace the character test."*** ⛔ **Wrong.**
`classify_rete_clause` is documented *"Independent of TypeEnv, by SHAPE alone"*, and the orchestrator's
instruction would have introduced a registry dependency into a module that deliberately has none.

**What it did instead:** canonicalize **first**, then keep the `::` test — because after
`canonical_identity` a namespaced type *has* `::` (`weather/ColdAndWindy` → `:weather::ColdAndWindy`)
and a field `:k` does not. ⭐ **Same discriminator, now spelling-independent, module independence
preserved.** Better than the instruction, and it said so plainly.

## ⭐ "MEASURED, THEN TOUCHED" — and it found a FAIL-OPEN

The brief flagged `expr_is_provably_boolean` as *"same gate, different question — measure before
touching."* It measured, and found **more than asked**:

| site | measured | acted |
|---|---|---|
| `expr_is_provably_boolean` | converted `:where` interiors do **not** reach it today, but **would** after head unification — a Keyword-only gate there is a **silent `false`** | wired through the same door |
| ⭐ **`check_fence_interior`** | converted interiors **do** reach it, and Keyword-only **FAIL-OPENED** — silently skipping let/match shadow checks and constraint typing | wired through the same door |

⛔ **A fail-open is worse than a red**: the checks were being skipped, not failing. **That was not in
the brief. Measuring found it.**

## ⛔ THE FINDING — the remaining 16 are `:then`, not `:when`

```
validate_then_form: fact_items[0] must be WatAST::Keyword
converted `(weather/ColdAndWindy :location ?loc)` → MalformedClause
```

The 5 closed files failed on **`:when`**; the 16 remaining fail on **`:then` insert heads** — a
**fourth** keyword-only slot, the sibling of the three this stone routed. ⭐ **The brief said *"name
it and do not force it"* and it did.** That is the fourth consecutive stone where the finding is the
most valuable output.

It also checked the generator class unprompted: `wat/query.wat` mints a fact-**bind** (already `:-`),
and **no constructor of `(Type/Name …)` clause heads exists** — so `:then` is a source-literal
problem, not a synthesised one.

**VERDICT: ACCEPTED.** The stone met its own gate: `:when` is through the door, and the residue is
named rather than forced.
