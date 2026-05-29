# NOTE (arc 109 vocabulary) — type-declaration `def`-prefix renames

**Filed 2026-05-28. NOT a decision on individual names — a queue marker carrying the
four-questions verdict that ratifies the family direction. Per-name intueri casts owed
per entry below.**

## What the verdict ratified

A four-questions cast (run inline 2026-05-28 during arc 241 form-collapse dialogue;
cost-of-impl excluded per user direction) compared three candidate naming conventions for
top-level definers:

1. `def*` prefix uniformly applied to ALL top-level definers
2. Bare-noun for type-decls (current state — `struct`/`enum`/`newtype`/`typealias`/`typeunion`/`recordtype`)
3. Tail-`def` in noun namespace (arc 227's `:wat::Record::def` pattern generalized)

**Verdict: Candidate 1 — `def*` prefix uniformly applied. YES YES YES YES.**

Candidate 2 disqualified on Obvious + Simple + Good UX (the bare/prefix split is two
rules; LLM co-authors trained on Clojure encounter friction at the bare-noun departure;
no single grep finds the declaration family).

Candidate 3 disqualified on Obvious + Simple + Good UX (substrate namespace ceremony per
declarator; verb-tail position violates Clojure-aligned verb-leads expectation).

### Required ratification — locked alongside the verdict

The verdict requires explicit lock on:

> **"def"** in the declarator prefix means *"top-level definition"* (concept), **NOT**
> *"expansion through the `def` primitive"* (mechanism).

`defn` literally expands through `def + fn`; the new `def<noun>` declarators do NOT —
they register types in `TypeEnv` via `parse_type_decl`. The def-as-concept interpretation
is what makes Honest hold uniformly across the family. Per `feedback_inscription_immutable`,
arc 198's inscription (where `def-restricted`'s `def` meant the literal primitive) stays
as historical record; this is forward-correction via the renames, not an edit to past
artifacts.

## Filing arc

Per spawn-block winding: defstruct lands in arc 241 (the focused dialogue surfaced it).
The remainder of the rename family is queued here under arc 109's broader vocabulary
work (sits alongside #564 / #565 — namespace reorg + intrinsic/substrate vocab).

## Queue — per-name renames + intueri questions to cast

Each rename below is a queued candidate. Per protocol (`feedback_spells_cast_via_subagent`),
intueri casts on the specific name shape before each lands.

| Current bare-noun | Candidate `def*` form | Intueri question owed |
|---|---|---|
| `:wat::core::struct` | **`:wat::core::defstruct`** — **LOCKED 2026-05-28** via intueri pair-cast (YES YES YES YES, Level 0). Lands in arc 241 implementation; rename target settled. |
| `:wat::core::enum` | **`:wat::core::defenum`** — **LOCKED 2026-05-28** via intueri pair-cast (YES YES PARTIAL YES, Level 0; PARTIAL Honest = historical C-enum ghost resolved by Rust normalization). Implementation queues here in arc 109; rename target settled. |
| `:wat::core::newtype` | **OPEN**: `defnewtype` / `deftype-new` / `deftypenew` / something else | **User direction 2026-05-28:** *"is it... deftypenew (or deftype-new) ... ?"* — this rename has multiple honest candidate shapes; intueri must adjudicate among them |
| `:wat::core::typealias` | **OPEN**: `deftype-alias` / `deftypealias` / `defalias` | Hyphen-separator vs glued, and whether the `type-` infix earns its place |
| `:wat::core::typeunion` | **OPEN**: `deftype-union` / `deftypeunion` / `defunion` | Symmetric question with typealias; cast as a pair |
| `:wat::core::recordtype` | **OPEN** — RECONCILIATION with arc 227 | Arc 227 shipped `:wat::Record::def` (base) + `:wat::holon::Record::def` (holonic) as the tail-`def` pattern. Under the verdict that tail-`def` is disqualified, both forms rename to `def*`-prefix. Candidate shapes need intueri reconciliation: `defrecord` (base) + `def-holon-record`? `defrecord` (base) + `defholonrecord`? Something else that honors the base/holonic flavor split? |

## Reconciliation with arc 227 (records flavor split)

Arc 227's `:wat::Record::def` / `:wat::holon::Record::def` was a self-consistent solution
under the tail-`def` interpretation. The 2026-05-28 verdict disqualifies that
interpretation; therefore arc 227's record forms also queue here for rename.

The arc 227 substantive doctrine (base records vs holonic records as two distinct
`Value` variants; per `feedback_no_semantic_abuse_of_option` and `project_typed_entities_doctrine`)
is unaffected. Only the form-NAMES retire. The mechanism stays; per
`feedback_inscription_immutable`, arc 227's inscriptions stay as historical record; the
renames land forward.

## Out of scope for this NOTE

- Implementation order across the family — to be picked when the work opens, informed by
  whichever consumers surface friction first
- Substrate parser unification — arc 241's territory; the renames depend on it landing
- Form-collapse work (struct absorbing struct-restricted via metadata maps) — captured
  separately in `docs/arc/2026/05/241-function-signature-unification/FORM-COLLAPSE-NOTES.md`

## Cross-references

- `docs/arc/2026/05/241-function-signature-unification/FORM-COLLAPSE-NOTES.md` — the
  dialogue this NOTE was filed from; defstruct's form-shape lives there
- `docs/arc/2026/04/109-kill-std/NOTE-reconsider-atomize-materialize.md` — sibling NOTE
  convention; this file follows the same shape
- `docs/arc/2026/05/198-defn-restricted/INSCRIPTION.md` — origin of `def-restricted`
  meaning expansion-through-`def`; ratification above forward-corrects the
  def-as-concept reading without editing past artifacts
- `docs/arc/2026/05/227-defrecord-defservice/` — the records flavor-split arc; its forms
  queue here for rename
- `feedback_spells_cast_via_subagent` — intueri casts via subagent; per-name evaluations
  owed before any rename ships
- `feedback_wat_llm_first_design` — the doctrine the verdict honors (one canonical path;
  LLM-first audience)
- `feedback_simple_is_uniform_composition` — the doctrine that earned Simple YES for
  Candidate 1 (one rule, N applications)
- `feedback_inscription_immutable` — discipline for forward-correction without editing
  past artifacts

---

*The discipline produced the verdict. The renames execute it. The names themselves still
require per-name intueri casts before they lock; this NOTE captures the family direction
and the specific open questions for each.*
