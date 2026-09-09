# DESIGN — the checker refuses a hoistable `where`

**Builder's ruling, verbatim:** *"i do not know if the compiler should do magic things - if it can
detect this, it must disallow it."*

That overrules my proposal (automatic hoisting) and it is the better call for two reasons, one of
which I had already written down as a caveat against myself:

1. **Hoisting makes the source lie.** The author reads nine patterns and seven filters; the machine
   runs a different network. Every other rung in this codebase pulls that divergence *out*. I
   proposed adding one.
2. ⭑ **Refusal has no purity caveat.** My hoist needed a bound — `where_tree.rs` makes `exec_where`
   *"the sole authority on the verdict (and on raises for predicates we actually run)"*, so hoisting
   a predicate that can **raise** changes when and how often it raises. Under refusal nothing is
   transformed: the author writes the inline form, so semantics never move. **Strictly fewer edge
   cases, not merely better taste.**

## The rule

A **standalone `(:wat::rete::where E)`** whose free `?vars` are *all* bound by some **single**
condition in the same `:when` is a **compile error**. The diagnostic names the variable, the
condition that binds it, and the rewrite.

There is no escape hatch and none is needed: for that shape, alpha and beta placement are
**semantically identical** — measured, 35 matches vs 35, identical `:line`/`:col` sets — and alpha
is never slower. Nothing is being traded away.

★ **The form's meaning narrows from two things to one.** Today `where` means both *"a filter on one
fact"* and *"a predicate across facts."* After this it means only the second, which is the only one
it is needed for.

## What it costs, measured — the reason this is worth a checker change

Same rule, same 5 rules, equivalence proven first (a fast wrong rule is the trap here):

| target | trailing `:where` | inline |
|---|---|---|
| `wat/service.wat` @ 1g / 2g / 4g | **killed (137)** at every ceiling | **exit 0, 1–2 s** |
| 1809 paths, one shot @ 2g | **killed (137)**; also at 8g, 12g, 16g | **exit 0, 41 s** |

The mechanism, from the shipped rule: three `Node` patterns join on a shared `?svc` and nothing
else, so a form with C children yields ~C³ sibling triples — and `?hi = 0` and `?si = 1`, each of
which cuts a stream from C to **one**, sit eight lines below as TestNodes that run on the triples
*after* they exist. It grows until killed; wall time rises with the ceiling.

⛔ **The author cannot see this.** The written form is one flat `:when` vector of sixteen sibling
clauses. Nothing marks that vertical position is the difference between C and C³, and the one place
it is written down is a Rust doc comment (`src/rete/matcher.rs:304`) that no `.wat` codemod author
reads. **1 of 93 recorded fixes uses the fast form.** That is the evidence a convention cannot carry
this.

## Blast radius — a census, not a guess

Balanced-paren census (a regex got this wrong twice; `[^)]*` cannot span nested parens):

| scope | files | refusable (var-vs-literal) | must survive |
|---|---|---|---|
| `wat-scripts/fixes/` | 26 | **231** | 1 var-vs-var, 11 `or`/`and` |
| `wat-scripts/` other | 54 | **92** | 11 var-vs-var, 46 composite |
| `wat/` stdlib | 1 | **0** | 1 composite (`or`) |
| `src/`, `tests/` `.rs` | 18 | unmeasured — rules embedded in Rust strings | — |

⭑ **No BOOTSTRAP / STASH-DANCE.** The stdlib's single standalone `where` is an `or`, which stays
beta, so the checker refusal never locks the tool out of fixing itself. That was the expensive risk
and it is absent. (Verified: `wat/rete/compile.wat` is the only stdlib *user*; `wat/rete/oracle/
pass.wat` merely names the symbol, being part of the compiler.)

## The migration is already written and validated

The mechanical transform — move each single-condition literal comparison into the pattern binding
its var — is written, validated, and **already applied to this campaign's own codemod**.
`wat-scripts/fixes/topic-record-drop-nsubs.wat` carries the inline form as of this commit: 17
constraints moved, **identical 35-match set** against the pre-change file, and the corpus-wide
census now runs in **one shot at 2 g in 41 s** where the trailing form was killed at every ceiling.
The A/B probe was deleted rather than left to rot beside the real rule; the measurement lives in
`FINDING-the-finder-hoists-nothing.md` and the trailing form lives in git history at `4b6767e3d`.

It ships as a recorded codemod, which is the pleasing recursion: **a codemod that fixes codemods.**
The `.rs` sites are hand work — rules embedded in Rust string literals are not `.wat` the tool can
walk.

## Out of scope = rejected

- **Automatic hoisting.** Overruled above.
- **A warning instead of an error.** A warning is a convention with a louder voice; 26 of 93 files
  is what conventions produce here.
- **Removing `where`.** It is required for var-to-var and cross-condition predicates — 12 and 58
  live sites respectively.
- **Deriving a `ParentHead` fact** so the C³ join need not be written at all. That is the higher
  rung and a separate stone; it removes the *join*, whereas this removes the *misplaced filter*.

## Open, and owed to the builder

**STOP-1 candidate:** a var bound inside `:not` / `:exists` (which `matcher.rs` handles via
`alpha-match-under`) or by an accumulator may not be inlinable even though one condition binds it.
The predicate must be *"∃ a condition into which this constraint can legally move"*, not merely
*"∃ a condition that binds these vars"* — and if those two differ anywhere, that difference is the
stone's real content.

**Filing:** this is a rete/checker change, not SNS/SQS work. It sits here because it was found here.
Where it belongs — its own arc, or a stone elsewhere — is the builder's ruling; no arc number has
been minted.
