# BRIEF — a match arm's pattern is not a constructor call

A `:then` operand walker descends into `match` arm patterns as if they were value expressions, so a
bare enum-variant arm gets arity-checked as a constructor and a legal `match` is refused. Teach the
walker the one form that needs it, and turn the banked red-by-design repro into a regression gate.

## Read in order

1. `src/rete/validate/mod.rs:774-800` — `walk_nested_constructors`. Note `let WatAST::List(items,
   span) = operand else { return }` (it walks **Lists only**) and the `:wat::core::kwargs-construct`
   special case, which is the shape to copy for a second recognised head.
2. `src/rete/validate/mod.rs:885-910` — the enum-variant arity branch. `enum_variant_ctor(types,
   head)` resolves the arm's leading keyword and the arity check fires the variant's declared field
   count against `args.len()`. **This is where the false `RhsArityMismatch` comes from.**
3. `src/rete/validate/mod.rs` tail of the walker — the *"Not a recognized constructor head — recurse
   into every item anyway"* fallthrough. This is what carries the walker into arm patterns.
4. `docs/arc/2026/06/278-rules-engine/harness-experiri/experiri-then-match.wat` — the repro that
   refuses, and `experiri-when-match.wat` — the byte-identical expression in a `where` fence, which
   loads. Read both headers; the first states its own disposal condition.
5. `src/rete/vocabulary.rs:584-585` — `:wat::rete::core::match` is a `RETE_OPS` row with `core_name`
   `:wat::core::match`. **Measure which spelling(s) reach the walker in a `:then` operand.**
6. `src/rete/clause.rs:260` — where `:wat::rete::core::match` is handled elsewhere, for the shape of
   a match form's parts.

## The shape

A match form is `(HEAD scrutinee arm…)`; an arm is `(pattern body…)`. Recurse into the **scrutinee**
and into each arm's **body**. Never into an arm's **pattern** (`items[0]` of the arm) — it is a bare
variant keyword, a destructuring List, or a literal, and none of those is a constructor call in that
position. A body **must** still be walked: it can legitimately nest a constructor.

## The enumeration is already done — do not redo it, and do not widen

`let` and `fn` bind in **Vectors**, so the walker returns before reaching them. `cond` clauses are
Lists but their `items[0]` is a call form, so keyword extraction fails and they fall through
harmlessly. **`match` is the only form that needs teaching.** Adding arms for the others would be
dead branches no mutation can prove.

## The repro is self-disposing

`experiri-then-match.wat` carries `rune:lint(red-by-design)` and says: *"If this file ever loads, D5
is cured and the rune must go with it."* Your fix makes it load. Turn the pair into a **regression
gate** asserting both spellings compile and agree, and retire the rune with its reason.

## Blast radius

`src/rete/validate/mod.rs`, one gate file, and the harness rune. Nothing in the engine's fire path.

## STOP triggers — halt and report

1. **If the bare and wrapped spellings do not agree after the fix**, stop and report both diagnostics.
   Agreement between the two spellings IS the cure; a fix that makes one compile and not the other has
   moved the coincidence, not removed it.
2. **If you cannot drive a spelling you are about to add a branch for**, stop. An arm for an
   unreachable head is a dead branch and its mutation cannot go red.
3. **If curing this reveals that some *other* `:then` form was relying on the walker's over-reach**,
   stop and report which — that is a finding, not something to patch around.
4. **If the fix requires touching `src/rete/kernel/fire/`**, stop.

## Mutation proofs — run all three, report all three

1. **Revert the walker fix** → the new gate must go RED on the bare spelling.
2. **Skip the arm BODY as well as the pattern** (recurse into neither) → a constructor nested inside a
   match arm's body must go **undetected**, and a gate for that must go RED. This is what proves the
   fix did not simply stop walking match forms altogether.
3. **Feed the wrapped spelling `((:probe::E::A) true)`** → must compile, before and after. Proves the
   fix did not break the path that already worked.

## What to report

- Both repro files' behaviour before and after, verbatim.
- Which match spelling(s) you measured as reachable, and how.
- All three mutation results.
- The rune's disposition.
- Scoped nextest Summary lines including `binary_id(wat::lint)`.
- Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.
- **Anywhere this brief was thin, wrong, or pointed at the wrong line.** Six riders have run on this
  arc; every one found a real defect in the brief, including three false claims of mine. Be blunt.

Do not commit. Leave the work in the tree and report.
