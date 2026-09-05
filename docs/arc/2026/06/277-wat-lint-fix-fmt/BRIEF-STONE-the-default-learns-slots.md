# BRIEF — STONE: the default learns slots from the registry

Give `wat/fmt.wat` a `Slot` fact built from the registry's `@syntax` grammars, and make R11 withhold
a `Break` for any glued child. Read `[[DESIGN-STONE-the-default-learns-slots]]` first — it pins where
the head spelling comes from and when the rule must REFUSE.

## READ IN ORDER

1. **`[[NOTE-the-registry-already-knows-the-slots]]`** — the four measurements. **The grammars are
   already proven parseable and the slot already proven locatable; do not re-derive that.**
2. **`wat-scripts/scratch-pad/277-locate-the-slot-in-a-grammar.wat`** — a working walk of a parsed
   grammar printing each child's index and kind. **Copy its shape.**
3. **`wat-scripts/scratch-pad/277-can-wat-read-its-own-grammar.wat`** — the all-36 read-string pass,
   including the `Row/syntax` filter and the `into`-a-Vector idiom `filter` needs (it returns a lazy
   stream; `length` on a stream raises).
4. **`wat/fmt.wat`** — `Break`, `Claim`, `breaks-map`, the three walls. `Slot` is a NEW record here.
   `breaks-map`'s conflict wall is the shape to copy for any new map.
5. **`wat-scripts/fmt/rules/siblings.wat`** — R11. It gains one condition: no `Break` for a glued
   child.
6. **`wat/grep.wat:33-97`** — the per-file fact records, for contrast. **`Slot` does NOT go here**;
   the DESIGN says why.

## SKETCH

```wat
(:wat::core::defrecord :wat::fmt::Slot
  [head  <- :wat::core::String     ;; SOURCE spelling, read from the grammar's child 0
   glued <- :wat::core::i64])      ;; the child index that must NOT start a line

;; building it, once per run:
;;   rows with a non-empty Row/syntax
;;   -> read-string the syntax           (proven: all 36 parse)
;;   -> children of the parsed form
;;   -> head  = ast-name of child 0      ← NOT Row/name; that is the DOT form and joins nothing
;;   -> find a child whose source is "->" at index i
;;   -> IF any child before i is variadic ("...", or ending "+" / "*")  -> emit NOTHING
;;      ELSE emit (Slot :head head :glued (+ i 1))
```

```wat
;; siblings.wat — R11 withholds a Break for a glued child
:when [ … the head's name ?hn … the child's index ?ci …
        (:wat::rete::not (:wat::fmt::Slot (?hn <- :head) (?ci <- :glued))) ]
```

## BLAST RADIUS

```
wat/fmt.wat                          Slot record, its builder, its map
wat-scripts/fmt/rules/siblings.wat   one added condition
wat-scripts/fmt/fixtures/            a fixture for the refusal, if one is needed
```

**No Rust. No intrinsic. No registry row. `wat/grep.wat` UNTOUCHED. `Break`/`Claim` unchanged. The
three walls stay exactly as they are.**

## STOP TRIGGERS

- **STOP-1 — do NOT join on `Row/name`.** It is the DOT form (`:wat.core/fn`); the corpus is the
  COLON form. **A join that silently matches nothing looks exactly like a form having no slots** —
  the worst failure available here. Take the head from the parsed grammar's child 0.
- **STOP-2 — if a grammar has a VARIADIC before the arrow, emit NO Slot for that head.** A wrong
  index mangles the ret-spec on every use of that form. Refuse rather than guess.
- **STOP-3 — do NOT put `Slot` in `wat/grep.wat`.** Its contract is per-file source facts; this is
  registry-derived and global.
- **STOP-4 — do NOT attempt type applications** (`(HashMap :- [T])`). They carry no `@syntax`, the
  DESIGN records why, and a lexical rule for them is the builder's next call — not this stone.
- **STOP-5 — if any existing test goes red, STOP.** Capture the whole block; do not re-run.

## ⚠ THE TRAP THAT COST THREE SABOTAGES LAST TIME

`collect-rules :fmt` gathers by NAMESPACE but only from files a driver has `load-file!`d. **A file
dropped in `rules/` is not loaded.** And **validate that any probe FIRES before reading its silence
as a result** — point it at something that must visibly change first.

## PRIOR COMPARABLE

`[[SCORE-STONE-the-exploded-form]]` — same arc, immediately prior, same files.

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's.
