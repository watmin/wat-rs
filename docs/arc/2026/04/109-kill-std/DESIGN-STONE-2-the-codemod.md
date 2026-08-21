# DESIGN — arc 109 step ②: the codemod. `Head<…>` → `(Head [types])`, 2,541 sites.

Step ① made the bracket ACCEPTED (`f454c465`, `df90b990`). This stone migrates the corpus to it.
Step ③ then makes the angle form illegal. **This stone changes no legality** — both spellings remain
valid throughout, which is what lets the stdlib keep loading while it moves.

⚠ **②a (the 244 bare heads) is NOT a prerequisite for this stone.** They are disjoint populations: a
bare `<- :wat::core::PersistentMap` has no angle brackets, so the codemod skips it entirely. ②a gates
**③**, where the vec becomes mandatory. Sequencing them the other way — as an earlier plan did — held
the mechanical 2,541 hostage to the judgment-bearing 244.

## The populations, and who can rewrite each

```
2,541  .wat   in 469 files   ← wat-fix codemod. THIS stone.
  692  .rs    in 133 files   ← Rust string literals; wat-fix cannot reach them. Separate strike.
  113  .edn   goldens        ← move only if RENDERING changes, which is ③'s ruling, not ②'s.
```

## ★★ THE MACHINE IS ALREADY BUILT — do not write a discriminator

`wat/fix.wat` already carries the exact predicate this migration needs, and its doc comment states the
rule I independently re-derived from the lexer:

```wat
;; type-shaped-keyword? — a keyword STRUCTURALLY a type: a parametric `Head<...>` or a
;; tuple/fn `(...)`. The discriminator requires a MATCHING close — a parametric has BOTH `<`
;; and `>` … so the comparison operators `:wat::core::<` / `:wat::core::<=` (which contain `<`
;; but no `>`) are NOT mistaken for types.
```
`wat/fix.wat:105-118`

And `fix-seq` (`:123`) is position-aware, carrying `prev-arrow?` so a keyword after `<-`/`->` is known
to be in type position. **The 9,912 `<-`/`->`/`<`/`<=`/`>`/`>=` sites that must survive are already
handled by machinery in the tree.**

## ⛔ THE ONE GAP — the converter emits the wrong form, twice over

`fix-seq` rewrites a type keyword by calling `:wat::core::keyword/to-type-form`. Measured, today:

```
:wat::core::Vector<wat::core::i64>              →  (wat.type/Vector wat.type/i64)
:wat::core::HashMap<wat::core::String,…i64>     →  (wat.type/HashMap wat.type/String wat.type/i64)
```

**Both halves are wrong for this stone:**
1. **Args are FLAT**, not bracketed. That is the superseded 2026-06-06 grammar; the builder ruled the
   bracketed form on 2026-07-24. Already filed as a blocker:
   `300/NOTE-the-type-converter-emits-the-superseded-form.md`.
2. **The head is already flipped** to `wat.type/`. Step ② keeps the rust-ish `:wat::core::` spelling —
   the Clojure flip is later and separate, and mixing them makes one migration two.

### The fix: parameterize the renderer, do not fork it

`type_expr_to_clojure_form` (`src/edn_shim.rs:1200`; the `TypeExpr::Parametric` arm at `:1249-1253`
splices flat). It needs **both**: bracketed args always, and a head-spelling mode.

```
mode COLON  → (:wat::core::Vector [:wat::core::i64])     step ② — this stone
mode CLOJURE→ (wat.type/Vector [wat.type/i64])           the later flip
```

★ Bracketing the args **closes 300's blocker as a side effect** — that note says the converter must be
fixed before 300.1 runs, and this is that fix. Do not write a second converter; a fork is how the two
spellings drift.

⚠ **Do NOT touch `fix-seq` itself.** It performs the FULL faithful-Clojure flip — arrows to `:-`,
heads to symbols — and that is 300's drive, not this stone. This stone needs a walk that rewrites
**only** type-shaped keywords and leaves arrows and heads alone. `wat-scripts/fixes/*.wat` holds 60+
recorded migrations; copy the shape of one that rewrites keywords in place.

## The order

```
②-i    parameterize the renderer + expose a COLON-mode wat verb.   src/ only. Floor.
②-ii   write wat-scripts/fixes/parametrics-take-a-type-vector.wat. DRY-RUN on a /tmp copy, diff it.
②-iii  apply to wat/ ONLY (the stdlib, ~470 sites). Floor. Commit.
②-iv   apply to tests/ + wat-scripts/ (~2,070). Floor. Commit.
②-v    the 692 .rs literals — separate strike, not this one.
```

★ `wat/` goes first and alone. It IS the stdlib: if the codemod is wrong, `wat/` failing to load is a
loud, immediate, small-blast-radius signal. Discovering it after 2,541 sites moved is not.

⚠ **R21 is non-negotiable**: `.wat` migrations are a wat-fix codemod, never hand-edits, never
python/sed. Dry-run on a `/tmp` copy and `diff` before applying. Idempotent — re-running must be a
zero-change no-op, and that is a test.

## What this stone does NOT do

- **No legality change.** `Head<…>` still checks after ② — that is ③.
- **No `.rs` literals** (692) and **no goldens** (113).
- **No bare heads** (244) — disjoint population, ②a's business, gates ③ not ②.
- **No `Fn(…)->T`** (175) — its own stone, after ③.
- **No head renaming.** `:wat::core::` stays. The `wat.type/` flip is later.

## The four questions

- **Obvious?** YES — one shape, `Head<…>` → `(Head [types])`, everywhere.
- **Simple?** YES — the discriminator exists, the walk framework exists, the only new code is a mode
  flag on a renderer that must change anyway.
- **Honest?** YES — and specifically because it does NOT ride the Clojure flip along with it. One
  structural change at a time, so a red floor names its own cause.
- **Good UX?** YES — after ②, the corpus reads in the destination grammar and ③ becomes a one-line
  legality flip rather than a migration.
