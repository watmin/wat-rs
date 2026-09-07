# WEIGH — RELAND 5, weighed centrally and STRUCK

`67e6833cf` · floor **5226/5226, 0 failed** · clippy 0 · pushed · tree clean.
**108 → 38 → 16 → 7 → 0.** The match-arm migration is closed.

## The rows, each re-run or re-read here — not taken from the SCORE

| # | row | verdict |
|---|---|---|
| 1 | floor `0 failed` | **PASS.** 5226/5226. Read `^ +Summary` unpiped, exit 0 |
| 2 | cause 1 goldens | **PASS.** Confirmed from the diff BEFORE reading the SCORE: 864→893, 871→900, 881→910, 889→918. Four files, uniform +29, no other byte |
| 3 | cause 2, refusal intact | **PASS.** `"malformed match arm"` still fires; the 2+-field refuse is a NAMED boundary, not a softened one. STOP-3 held |
| 4 | cause 2, grid live | **PASS.** Both axes green in the floor |
| 5 | cause 3 bisect | **PASS on the commit, CORRECTED on the event** — see below |
| 6 | 4 probe rows | **PASS.** `#[ignore]` 0 |
| 7 | clippy | **PASS.** 0. Workspace is `deny(clippy::all)`, so 0 is enforced, not observed |

## ★ The one correction, and it is the interesting one

The SCORE says cause 3 "does not predate the campaign." **The commit is right; the event is not
a regression.**

```
State/seen      introduced 2b9017a54 (arc 278) — never since changed
the campaign    never touched that file (ab52b7188 / 037ef43ef)
```

A census of the corpus, 107 `::State/<member>` uses:

```
67  ::State/durable                  State's own field
11  echo · 6 store · 4 sink · …      all declared :ephemeral fields
 1  ::State/seen                     the ONLY site reaching a DURABLE-RECORD field through State
```

`defservice` mints State as the durable ref plus ephemerals (`wat/service.wat:566`); the canonical
read is `(:svc::Record/<f> (:svc::State/durable s))`, **35+ instances on disk**. `State/seen` was
never valid.

What `480f38d05` changed is `src/resolve/walk.rs`: `items.iter().skip(4)` → `skip(2)`, plus
Vector-arm awareness. **The old walker skipped the first arms and never resolved their bodies.**
The check got better and found a defect that had been sitting since arc 278. Fixed at one site.

★ **A bisect names where a symptom became VISIBLE; it cannot alone say the commit CAUSED it.**
STOP-4 was held exactly as written and the report was honest — **the brief was the defect.** It
asked *which commit*. It should have asked *what the commit did*.

## What I chased and dropped, so it is not re-raised

`lower_variant_map` **discards the key** on a single-field variant and binds positionally, which
reads as a compiler/interpreter divergence. It is not: `check.rs:6604` rejects
``map-pattern key `:k` is not a field of `T` `` and enforces arity, so a mis-keyed pattern never
reaches `lower()`. Safe by construction.

## Standing, named, not fixed

Five rustc `dead_code` warnings — `Coverage::Wildcard`, `pattern_coverage`, `ident_span`,
`try_match_pattern_ast`, `substitute_many`. Not clippy (which is 0 and denied). `ident_span` lives
in `src/match_arm.rs`, **minted today**, so "pre-existing" is true of this stone and unmeasured of
the campaign. Corpse-or-door is a judgement per site and gets a stone, not a reflex deletion.

## Two things the split left honest

- **The named-payload IR** — `Pat::Variant` binding BY NAME, rete asking `type-of` — is owed. The
  grid is 0-1 field, so it is live without it; 2+ refuses by name.
- **`MatchArm::Literal` was a restoration.** The retired grammar handed the whole pattern to
  `try_match_pattern`, which compared literals. The bracket flip replaced *hand it to a general
  matcher* with an **enumeration of heads**, and the literal head fell through the enumeration.
  That is the shape to watch wherever else this migration replaced a general reader with a taxonomy.
