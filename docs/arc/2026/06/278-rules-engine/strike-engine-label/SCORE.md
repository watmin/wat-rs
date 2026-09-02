# SCORE — C7 + C2, weighed against the orchestrator's own re-run

> Re-run at `00ca6b0eb`. **STOP-3 fired, twice over, and the rule I adopted was wrong.**

| # | pre-value | after |
|---|---|---|
| 1 | I briefed **3** `(engine)` sites | ⛔ **FIVE** — see A |
| 2 | the false arm | ✅ claim **dropped outright**, nothing put in its place |
| 3 | the true ones | ✅ named as qualified paths |
| 4 | ★ resolution excludes the test tree | ✅ **re-driven by me** — see C |
| 5 | bare `(engine)` REDs | ✅ |
| 6 | non-vacuity | ✅ floor, driven |
| 7 | C2's second citation | ✅ stale — and **my re-derivation was off by one**: `:1383` is blank, `const RUNS` is `:1384` |
| 8 | radius | ⚠ **+2 files**, both earned — see D |
| 9 | lint 182/182 | ✅ **196/196** |
| 10 | floor 5296/5296 | ✅ `5310 tests run: 5310 passed`, zero FAIL |
| 11 | clippy rc=0 | ✅ rc=0 |

## ⛔ A — three sites became five, and one of them is a different spelling

`compiled_rhs.rs:760` carries `(engine)` **inside a `#[cfg(test)]` mod in a `src/` file** — a shape I
had no model for. And `accum_alpha_cost.rs:841` claims `(THE ENGINE)`, which my "one word, so it can
be exact" reasoning missed on case alone.

## ⛔⛔ B — MY RULE WOULD HAVE DELETED A TRUE CLAIM, AND THE GATE IT CONTRADICTS IS MINE

The `L` arm calls **no** function: it replicates `root_for`'s body inline because production is not
callable there. Under *"only if its body CALLS the production function"* it must lose its label — yet
its claim is **true** and is already pinned by `rete_header_claims_are_asserted.rs:157`, which
asserts `AlphaRoots`' type and `root_for`'s body **exactly**. That is strictly stronger than a name
resolving.

**I wrote that gate myself during the item-#3 work, then drew a rule that would have overridden it.**

The rider's taxonomy is the fix — three shapes where I assumed two — and the contract became *an
engine claim carries its evidence: the function it CALLS, or the gate that PINS its replication*.
Promoted to memory: **a rule can outlaw a truth.**

## ⛔ C — my exclusion boundary was the wrong KIND of boundary

I wrote `kernel/tests/`. **26 files under `src/rete/` carry `#[cfg(test)]`**, and site 4 proves
label-bearing test code lives in them — so my decoy mutation would have passed while the hole stayed
open. The boundary is *"not inside a `#[cfg(test)]` module"*, by brace tracking.

**Re-driven here:** identical `fn seen_insert`, identical label — **RED** inside `#[cfg(test)]`,
**GREEN** in production scope. Red and green differ by placement alone.

I also blurred two things the rider kept apart: the **label** may live in test code (four of five
do); only the **resolution target** must be production.

## D — the radius grew twice, and both were earned

`rete_citation_resolves.rs`, then `universe_control_name.rs` (new), because of a coupling neither
gate owns: **a `gated by` claim writes a test fn's name into a `src/` string literal**, which retires
it as a test-only control elsewhere. The first such claim consumed one of that gate's two controls
**on landing**, and a tree-wide search found no replacement of the same kind.

**Closed rather than filed** — a ratchet toward zero is what this session has spent itself fighting.
An **owned, uncitable** control now floors the gate (its body asserts its own name appears nowhere
under `src/`, and its failure message says *"take the name back out of `src/`, do not adjust this
test"*), the real cited control is kept beside it, and the coupling is warned **at the `gated by`
definition** — where the next author is working, not in the gate they will never open.

The rider drove the exact feared scenario end to end: a `gated by` claim naming the control itself
leaves the **engine** gate green (correctly — the grammar is satisfied) and reds **both** guards.

## Three findings I would not have had

- **The obvious home for the control was wrong.** `rete_citation_resolves` excludes its own file, so
  a control defined there would never resolve — failing closed, for a confusing reason.
- **The control is deliberately stricter than the gate it floors** (raw text, so a `src/` *comment*
  fires it too). Stated, not smuggled.
- **The engine gate can never catch this class**, and mutation 1 shows it staying green *correctly*.
  It is a cross-gate resource conflict, so the paragraph at `GATED` is the only thing reaching the
  author before the collision — "not belt-and-braces", and not to be trimmed as prose.

## And the answer on the false arm was better than what I would have accepted

`(superseded)` was rejected as **subtly false** — `seen_insert`'s own fallback arm is still an
`FxHashSet<Value>` insert. The claim is dropped outright; the row already names what it times, and
the sibling `(identity)` row and `S−I predicted cut` line carry the comparison.

## Arms

Eleven, all **proven** (driven, red→green): bare label; unresolvable name; gate-name-not-a-test;
★ the decoy plus its production-scope counter-check; resolves-but-never-called; label scanner
blinded; walk blinded; positive control; the feared-scenario end-to-end; control walk blinded;
control stops resolving. **None unreached.**
