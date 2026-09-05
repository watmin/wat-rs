# BRIEF — STONE: a type application is atomic

Make the emitter render a type application verbatim on one line and descend no further. Read
`[[DESIGN-STONE-a-type-application-is-atomic]]` first — it carries the measured predicate and the
three counterexamples that killed the obvious one.

## READ IN ORDER

1. **`wat/fmt.wat`, `emit-node`** — the leaf path already calls `ast->source`
   (`wat/fmt.wat:140`-ish). **A type application takes that same path.**
2. **`wat/fmt.wat`, the `Slot` builder** — last stone's registry walk, for the shape of "inspect a
   form's children by index".
3. **`wat/grep.wat:25-28`** — the note that `Named` is emitted for **Symbol/Keyword/StringLit**.
   ⚠ **This is why the string `":-"` looks like the symbol `:-`** and why the predicate needs the
   kind check as well as the arity check.
4. **`wat/core.wat:1349`** — the generic `fn` the naive predicate would have collapsed. **Row 3's
   fixture is this shape.**

## SKETCH

```wat
;; a type application, for LAYOUT purposes, is a leaf.
;;   list  AND  exactly 3 children
;;   child 1 kind is symbol|keyword  AND  its name is ":-"
;;   child 2 kind is vector
;; -> emit (ast->source node); do NOT recurse; assert no Break inside it.
```

## BLAST RADIUS

```
wat/fmt.wat    emit-node's descent decision (and a helper predicate)
```

**No rule file changes. No Rust. No new record. `Slot`, `Break`, `Claim` unchanged. The three walls
stay exactly as they are.**

## STOP TRIGGERS

- **STOP-1 — the predicate is ARITY-3 plus the kind checks. Do NOT weaken it to "child 1 is `:-`".**
  That matches 5,772 forms including a generic `fn` and two string literals; the DESIGN names all
  three. A wrong predicate collapses real code onto one line.
- **STOP-2 — if a NESTED type application still breaks, STOP.** Atomic means the whole subtree
  renders verbatim, not just the outer form.
- **STOP-3 — do NOT reintroduce a subtree claim.** This is a descent decision in the emitter, not
  ownership. `Claim`/`ClaimedUnder` are untouched.
- **STOP-4 — if any existing test goes red, STOP.** Capture the whole block; do not re-run.

## ⚠ THE TRAPS FROM THE LAST THREE STONES

- `filter` returns a **lazy stream**; `length` on a stream raises. `into` a Vector first.
- A file in `rules/` is **not loaded** until a driver `load-file!`s it.
- **Validate a probe FIRES before reading its silence as a result.**

## PRIOR COMPARABLE

`[[SCORE-STONE-the-default-learns-slots]]` — same arc, immediately prior, same file.

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's.
