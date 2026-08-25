# DESIGN — STONE: an edit carries what it CLAIMS to replace

> **Builder ruling, 2026-08-25:** *"fix-text-apply too - carry the old text"*
>
> The top rung, named and measured in `278/DESIGN-STONE-wat-grep-never-lies.md` as out of that
> stone's scope. `Written` made the corruption *expressible-against*; this makes it *impossible*.

## THE THESIS

`wat/fix.wat:324`'s `fix-text-apply` takes `(offset, old-len, new-text)`. It knows **how many**
characters to overwrite and never learns **what it believes is there**. So it cannot tell a correct
edit from a catastrophic one, and it splices either.

```wat
new-src (:wat::string::concat
          (:wat::string::subs src 0 off)
          new-text
          (:wat::string::subs src (:wat::core::+ off old-len) (:wat::string::length src)))
```

An edit gains a third thing it must carry: **the text the rule claims is at that offset.** Apply
compares it against the source and raises on disagreement. Then no codemod can corrupt silently —
whether or not its author has ever heard of `Written`, reader macros, or this stone.

## ⛔ THE ONE PINNED CONTRACT — `old-text` is the rule's CLAIM, never a slice of the source

This is the whole stone, and it is the one way to get it wrong while every test still passes.

Work the char/of corruption through. At `tests/value/wat_arc220_char.wat:11` the reader synthesized a
`:wat::core::char/of` keyword node whose span covered `\a` — two columns:

```
off      = offset of the span start
old-len  = fix-text-span-len(start, end, lines)  =  2      ← CORRECT. It IS the span width.
new-text = ":wat::core::char"                              ← 16 chars
```

**`old-len` was never wrong.** The splice replaced exactly the two characters the span named. The bug
was that the rule *believed* it was replacing `:wat::core::char/of` and was actually replacing `\a`.

★ **So an `old-text` DERIVED FROM THE SPAN would have been `"\\a"`, matched itself, and the splice
would have proceeded.** The check would be vacuous — comparing a slice against the slice it came
from. `old-text` must come from the RULE'S BELIEF, and the disagreement between belief and source is
the entire signal.

⚠ Deriving `old-text` from the span is the easiest way to make all 31 files type-check and every test
go green while the wall guards nothing. It is STOP-1.

## ★ THE INFORMATION IS ALREADY THERE, CAPTURED AND DISCARDED

`wat-scripts/fixes/rename-core-string-to-string.wat` — the recorded migration this whole session has
used as the shape — asserts both captures and consumes only one:

```wat
:captures (:wat::rete::core::PersistentVector
            (:wat::grep::Capture :name "old" :value ?n)          ← the belief. NEVER READ.
            (:wat::grep::Capture :name "new" :value …))          ← read via :rn::second-capture
```

`edits-of` calls `second-capture` for the replacement and computes the length from the span. **The
rule writes down exactly what it thinks it is replacing and then throws it away** — the same shape as
`wat/grep.wat:218` binding a parse cause to `__cause`. Twice in one subsystem, the knowledge in hand
and dropped one expression from its use.

## THE FORM

```wat
;; the edit tuple: (offset, OLD-TEXT, new-text)   — old-len is derivable as (length old-text),
;; so it is not carried. Two copies of one fact is how they come to disagree.
(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
```

```wat
;; fix-text-apply — verify, then splice.
;;   if (subs src off (+ off (length old-text))) != old-text  ->  RAISE, naming
;;   the offset, what was claimed, and what is actually there.
```

And a sibling door beside `fix-text-span-len`, for the callers whose belief genuinely is the source
text (a deletion, a reformat):

```wat
(:wat::core::defn :wat::fix::fix-text-span-text
  [start-span <- … end-span <- … lines <- … src <- :wat::core::String] -> :wat::core::String)
```

⚠ **`fix-text-span-text` is a loaded gun** — a caller that uses it to fill `old-text` has made the
check vacuous for that edit. It exists for edits whose subject genuinely IS the span (deleting a
region, reflowing whitespace), never for a rename. STOP-1 covers the misuse.

## THE CASCADE — measured

```
341  occurrences of the edit-tuple TYPE annotation
 31  files:  wat/fix.wat (52)  ·  wat/lint.wat  ·  28 × wat-scripts/fixes/  ·  1 × wat-scripts/lib/
 43  call sites of fix-text-apply
```

And the middle value's binding, classified — **the transformation is a derivation, not a judgement
per site**, because *the length expression names its own subject and the subject is the old text*:

```
 29  (:wat::string::length X)        ->  old-text is X          mechanical, and NON-VACUOUS (X is the belief)
  7  (:wat::fix::fix-text-span-len …) ->  ⚠ the length came FROM THE SPAN. Supplying span text here
                                          makes the check vacuous — these need the rule's claim,
                                          which for the rules-based codemods is the "old" capture.
```

The 7 are the interesting ones and they are where the stone is won or lost.

## THE ITERATION LOOP

`wat/fix.wat` and `wat/lint.wat` are stdlib — `include_str!` at Rust-compile time, so a change costs
one ~19s rebuild. The other 29 live under `wat-scripts/` and **`wat --check` them in 0.145s each**
(measured). So: change the two stdlib files, rebuild once, then let the type-checker name the
remaining 29 one at a time. The wat type checker is this stone's rustc.

## THE FOUR QUESTIONS

- **Obvious?** YES — an edit that says *"replace the 19 characters I think are here"* is obviously
  safer than one that says *"replace 19 characters."*
- **Simple?** YES — one field changes type. `old-len` stops being carried because it is derivable;
  two copies of one fact is how they disagree.
- **Honest?** YES — this is the stone. An applier that cannot tell a correct edit from a catastrophic
  one, and splices either, is lying about what it does.
- **Good UX?** YES — a codemod author gets a located raise instead of a corrupted corpus, and gets it
  on the dry run.

## ACCEPTANCE

1. **★ THE CONTROL, and it is row one: a rule that claims to replace name N at a span holding
   different text RAISES instead of splicing.** Build it deliberately — match a `~` node by `Span`
   (not `Written`), claim `:wat::core::unquote` as `old-text`, and confirm `fix-text-apply` refuses,
   naming the offset, the claim, and what is actually there. **Before this stone that edit silently
   replaces `~` with a 19-character name.** Without this row the stone is unfalsifiable.
2. **The vacuity control:** an edit whose `old-text` was sliced from the source at its own offset must
   be recognised as proving nothing. Assert the raise fires on a genuine disagreement, never merely
   that some check exists.
3. **All 31 files `--check` clean**, and the loader gate `every_wat_scripts_file_loads` is green.
4. **A recorded migration re-runs to zero changes.** `rename-core-string-to-string.wat` over the
   corpus: 0 edits, byte-identical tree — it is idempotent as a query and the migration already ran.
5. **`fix-text-span-len` has zero callers, or it stays with its callers named.** If the migration
   leaves it dead, it goes — a helper kept for nobody is a graveyard.
6. Floor green **accounted BY NAME** (baseline 5053/5053, 19 skipped); clippy 0.

## MIGRATION ORDER — and why this one is a STASH-DANCE

The codemod that migrates the codemods is itself a codemod that calls `fix-text-apply`. So:

1. Write the migration against the **OLD** API (it still exists; it works).
2. Dry-run on a `/tmp` copy, diff byte-level.
3. Apply to the 31 files — **including itself**, which is fine: a recorded migration only has to
   type-check afterwards, not run again.
4. **Then** edit `wat/fix.wat` (the definition + `fix-text-span-text`) and `wat/lint.wat`.
5. Rebuild. `--check` sweep. Floor.

Between 3 and 5 the tree does not type-check. That is expected and it is why this lands as ONE atomic
commit — `wat/fix.wat`'s own header documents the dance.

## OUT OF SCOPE — affirmatively cut

- **Retro-fitting `Written` into the 68 recorded codemods.** Their migrations have run; they are
  history. This stone changes their edit shape because the loader gate requires it, and nothing more.
- **The `Span`/`Extent` field-lockstep pin** (`grep.wat:47`). Still open, still not this.
- **The four-homes rename.** Blocked behind this and re-counted after, since its 239 was measured on
  the old, silently-dropping instrument.
