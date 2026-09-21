# SCORE — STONE 251.8d-i-b: `<-` is NOT always an annotation arrow

Branch: `main`. **Committed, not pushed.** Drawn against `89dcbe584` (draw `f82885a70`).
Parent: `BRIEF-STONE-251.8d-i-b-the-arrow-is-not-always-an-annotation.md`.
Floor / workspace clippy / census: orchestrator. Crate clippy + lint + replay run here.
**No corpus `.wat` converted.** Fixture + tool only. 8d-ii / 8d-iii not started.

## Gate: the 179-file delta — ONE tree

Same spread as 255.1–255.5: every 12th of the 8d-i census `files.txt` (**179**).
Originals: **live tree**. Converted: `/tmp/8d-i-b/conv` (re-converted with the **new** tool —
tree-after-1 is the old conversion and was not reused). Timed one file first:
`tests/rete/probe_arc278_4b_cascade.wat` copy **0.50 s**, rc=0.

The brief's baseline **64** is the orchestrator's 255.5 WEIGH tree. This stone's
before is **this tree at 255.5 (66)**. Like-for-like.

| | this tree at 255.5 | this stone |
|---|---|---|
| originals clean | **161 / 179** | **161 / 179** |
| still clean after conversion | 95 | **95** |
| ⛔ **REGRESSIONS** | **66** | **66** |

**0 closed. 0 newly_broken. Net 0.**

### Classification (66) — same residue, different guts

| cause | n | vs this tree at 255.5 (66) |
|---|---|---|
| `UnresolvedReference` | **23** | 23 |
| `ReteCheckErrors` | **21** | 21 — **did not collapse** |
| `MalformedDecl` | 9 | |
| `CheckErrors` | 7 | |
| `MalformedForm` | 4 | |
| `UnknownNamedType` / OTHER | 2 | |

255.5 lumped MalformedDecl as 13 (9 + 4 MalformedForm). Same 66 files.

## ⭐ THE ARROW FIX HELD — measured on the copies

The brief's example, old conversion vs new, same file
(`probe_vig_retract_multiplicity.wat`):

```
OLD :when [(vrm/F (?k :- k)) (vrm/G (?k :- k))]   :then [(vrm/Seen :k ?k)]
NEW :when [(vrm/F (?k <- :k)) (vrm/G (?k <- :k))] :then [(vrm/Seen :k ?k)]
```

`:k` survived in `:then` both times (no arrow). It now survives in `:when` too.

Across the 179-file sample:

| | old conversion (tree-after-1) | this stone |
|---|---|---|
| `?x :- ` files / occ | **28 / 394** | **0 / 0** |
| `?x <- ` occ | 14 | **408** |

The destruction class is gone. One fix cured both symptoms: `annotation-arrow?`
declines, so `prev-arrow?` never sets, so the post-arrow type rule cannot fire
on the field name. No second patch for `:k` → `k`.

Idempotence: second pass on the 179 converted copies, **179 / 179 unchanged** (9.87 s).

## ⭐ NON-VACUITY BOTH WAYS

Replay fixture `wat-scripts/fixes/replay/to-faithful-clojure/` now carries:

- rete `(?k <- :k)` **identical** in before and after (the near-miss)
- `(:vrm::F (?k <- :k))` → `(vrm/F (?k <- :k))` — head converts, binding stays
- `` `[~x <- :wat::core::i64] `` → `` `[~x :- wat.type/i64] `` — unquoted name still converts
- existing `[x <- :wat::core::i64]` → `[x :- wat.type/i64]` still converts

Timed `~` control (`Record.wat`'s shape): `[& ~call-args-sym <- …]` →
`[& ~call-args-sym :- …]`. Return arrows still convert.

## ⭐ FINDING: ReteCheckErrors 21 did not collapse

The brief predicted they would. They did not. The remaining error on every one
of the 21 is `MalformedClause`, and the clauses are the **correctly converted**
forms:

```
(vrm/F (?k <- :k))
(?fact <- weather/ColdAndWindy)
(wat.grep/Node (?id <- :id) (?k <- :kind))
```

Rete's `:when` parser does not accept a **symbol-headed** fact pattern. The
head-keyword rule (`:vrm::F` → `vrm/F`) is independent of the arrow: it fires
because the fact type is a `::`-keyword list head, not because `prev-arrow?`
was set. Leaving the arrow alone cannot change that.

That is not an annotation question and this stone cannot express it. Loading
rete in the faithful-Clojure surface is 8d-ii / the checker, not a second
codemod patch for `:k`.

Whole-fact bindings `(?fact <- :weather::ColdAndWindy)` keep `<-` (rete-var
guard) and still convert the namespaced type via **head-keyword**
(`weather/ColdAndWindy`), not via the post-arrow type rule. Same residue.

## What landed — the precondition, not a rete special case

`arrow?` stays the leaf fact (this symbol is `<-` / `->`).
`rete-var?` is a symbol whose `ast-name` starts with `?`.
`annotation-arrow?` is `arrow?` AND NOT `prev-rete-var?`.

`fix-seq` / `fix-text-seq-edits` carry a second bit `prev-rete-var?` next to
`prev-arrow?`. Convert the arrow only when `annotation-arrow?`; recurse with
that as the next `prev-arrow?`.

Chosen over "parent is a list" because return arrows live in lists
(`(defn name [args] -> Ret body)`). Chosen over "inside `:when`" because
`?`-prefix is local and the walk already carries sibling state. The 42 `~`
unquoted annotation names do not start with `?` and still convert.

The false comment *"a bare `->` SYMBOL is always an annotation arrow"* is
gone from `wat/fix.wat`.

`wat/fix.wat` is `include_str!`'d — rebuilt before any conversion.

## What the rule cannot express

`annotation-arrow?` answers *"is this `<-`/`->` an annotation?"* It cannot:

- make rete accept a symbol-headed `:when` clause (`vrm/F` vs `:vrm::F`)
- stop `head-keyword?` converting a namespaced fact type that happens to sit
  after a rete `<-` (that is a call-head, not a post-arrow type)
- teach `to-faithful-clojure-rete.wat`, which still threads raw `arrow?`
  (production conversion is `fix-text`; the rete drive was not this door)

## Test count

Predicted **0**. `cargo nextest list --release -p wat`: **5319** (unchanged).

## Walls I ran

- crate clippy `-p wat --all-targets -D warnings` — **0**
- `--test cli every_recorded_migration` — 18 passed (replay + fixtured-or-runed)
- `--test resolve probe_arc251` — 102 passed (incl. `fix-seq` callers, fix-text, local arrow rules)
- `--test lint` — 356 passed
- 179-file convert 30.04 s rc=0; idempotence 179/179; delta as table above

Floor + workspace clippy + `census.sh --diff`: **not run** (orchestrator,
uncontended). Do not push. Do not start 8d-ii.
