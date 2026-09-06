# BRIEF — STONE: a trailing run of pairs

Replace the kwargs rule's whole-arg-list test with a TRAILING-PAIR-RUN test, and delete its
unconditional "positionals ride". Read `[[DESIGN-STONE-a-trailing-pair-run]]` first — it carries the
five shapes one rule must produce and the substrate's own predicate.

## READ IN ORDER

1. **`src/check.rs:13440-13447`** — the substrate's `is_kwargs`. **Copy its SHAPE, do not invent a
   second definition.** Its comment says it is the same test the eval arm and `build_insert_fact`
   use; a formatter with a third opinion would argue with the language.
2. **`wat-scripts/fmt/rules/kwargs.wat`** — the shipped rule. Its run detection and its
   positionals-ride clause are what change.
3. **`wat-scripts/fmt/rules/siblings.wat`** — R11's leading-atom rule. **Positionals go back to
   obeying it**; nothing special replaces the deleted clause.
4. **`wat-scripts/fmt/fixtures/compound-then-kw.wat`** — the reproducer, already on disk. Today it
   yields the 132-column line.
5. **`wat-scripts/fmt/fixtures/kwargs-pos.wat`** — `defservice`'s shape, which must NOT regress; its
   positional is an ATOM and rides by the ordinary rule.

## SKETCH

```wat
;; kwargs.wat — a MAXIMAL TRAILING run of (keyword, value) pairs.
;;   scan from the LAST child backwards while the pattern holds
;;   run length must be >= 2 and EVEN, and every even-offset element a keyword
;;   -> Break each KEY; withhold from each VALUE; assert AlignPairs on the form
;;   children BEFORE the run get NO special treatment — R11's leading-atom rule owns them
```

## BLAST RADIUS

```
wat-scripts/fmt/rules/kwargs.wat    the run test; DELETE the positionals-ride clause
wat-scripts/fmt/fixtures/           fixtures for rows 1, 2, 5
```

**No Rust. No new record. `AlignPairs`, `Break`, `Claim`, `BlankBefore` unchanged. The three walls
unchanged. `grep -c 'col' rules/*.wat` must stay 0.**

## STOP TRIGGERS

- **STOP-1 — do NOT invent a second definition of a kwarg call.** Use `check.rs:13444`'s shape.
- **STOP-2 — do NOT add any clause that makes a positional ride.** The ordinary leading-atom rule
  handles `defservice` because its positional is an atom. **A special case here WAS the bug.**
- **STOP-3 — no rule may name a column.** The previous wall stands at 0.
- **STOP-4 — if the 132-column doc-example line survives, that is a RED, not a partial win.**
  Report it and stop; it is the defect this stone exists to remove.
- **STOP-5 — if any existing test goes red, STOP.** Capture the whole block; do not re-run.

## ⚠ TRAPS

- A file in `rules/` is **not loaded** until a driver `load-file!`s it.
- `filter` returns a **lazy stream**; `length` raises — `into` a Vector first.
- **A fixture must TYPE-CHECK** — `./target/release/wat --check <f>` before trusting it. The last
  stone lost a floor run to a fixture whose declared return type was wrong.
- **Validate a probe FIRES before reading its silence as a result.**

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's.
