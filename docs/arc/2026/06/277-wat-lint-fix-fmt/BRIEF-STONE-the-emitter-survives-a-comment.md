# BRIEF — STONE: the emitter survives a comment

Make the emitter's column state survive emitting a comment, stop writing whitespace-only blank lines,
and add the two blank-line rules. Read `[[DESIGN-STONE-the-emitter-survives-a-comment]]` first — it
carries the exact-column measurement, the 869-line hazard behind rule 2, and the one defect that is
NOT ruled.

## READ IN ORDER

1. **`wat/fmt.wat`, `emit-node` + `flush-comments`** — where a comment is written. **This is the
   site.** The indent threaded into `emit-node` is correct going in; it is lost coming out of a
   comment.
2. **`wat/fmt.wat`, `write-nl` / `spaces` / `Acc`** — the emitter writes indent then a newline,
   which is defect D: a blank line made of spaces.
3. **`wat-scripts/fmt/rules/defn.wat`** — the ret-spec and body Breaks, for rule 2's position.
4. **`wat/spawn.wat:195-203`** — the real file. Its variants must stay at indent 2 across their
   trailing comments, **with the tag padding intact**. ⚠ **Row 2's gate, and a real file, not a
   fixture.**
5. **`wat/deporder.wat`** — 323 lines, 85 comments, already formatted idempotently. **Its comment
   count must not move.**

## SKETCH

```wat
;; the column state: emitting a comment must leave the emitter exactly where a
;; sibling break would have. A comment is a LINE, not a node — it does not consume
;; the indent the next node is owed.
;;
;; a blank line is "\n", never indent + "\n".
;;
;; rule 2  no Break emits a blank between ret-spec and body
;; rule 3  exactly one blank line after each top-level form — inserted if absent,
;;         NOT duplicated if present (see row 7)
```

## BLAST RADIUS

```
wat/fmt.wat    emit-node's comment path, the blank-line writer, the top-level separator
```

**No Rust. No new record unless rule 3 needs one. No rule file changes expected — this is an emitter
stone. If a rule change turns out to be needed, say which and why.**

## STOP TRIGGERS

- **STOP-1 — rule 2 forbids ONE position (ret-spec → body), not every blank line in a form.**
  **869 blank lines sit inside top-level forms** and most are deliberate paragraph breaks. A purge
  passes rows 4-5 and fails row 6. **If the position cannot be isolated, STOP and report.**
- **STOP-2 — do NOT decide where a TRAILING comment goes.** Defect E is the builder's policy call
  and is explicitly out. **Report what became of each trailing comment; do not choose.**
- **STOP-3 — if the required blank accumulates across passes, STOP.** Same class as
  `EmptyVecAfter`'s `[] []`.
- **STOP-4 — a comment must never be LOST.** `wat/io.wat` is 28 and `wat/deporder.wat` is 85; both
  counts are gates. A comment count that drops is a red, not a rounding.
- **STOP-5 — if any existing test goes red, STOP.** Capture the whole block; do not re-run.

## ⚠ TRAPS

- `\;` is a **char literal**, not a comment — the reader stone measured it.
- A file in `rules/` is **not loaded** until a driver `load-file!`s it.
- **Every fixture must `wat --check` clean BEFORE the floor.**

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's.
