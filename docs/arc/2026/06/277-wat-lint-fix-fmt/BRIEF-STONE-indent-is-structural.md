# BRIEF — STONE: indent is STRUCTURAL

Change `Break {id, indent}` to `Break {id, kind}` where `kind` is `:block` or `:align`, and move
every column computation into the emitter. Read `[[DESIGN-STONE-indent-is-structural]]` first — it
carries the measurement that forces this and the kind-per-rule table.

## READ IN ORDER

1. **`wat/fmt.wat:7-9`** — the `Break` record. Two fields; `indent` becomes `kind`.
2. **`wat/fmt.wat:38-41`** — `Acc {out, next-id, comments}`. **No column is tracked.** It needs one,
   or the emitter derives the current column from `out`'s tail. Either is fine; the number must come
   from what has been EMITTED.
3. **`wat/fmt.wat:98`** — `emit-node`. It already takes an `indent` parameter and threads it. Today
   that thread is discarded in favour of the Break's absolute number. Stop discarding it.
4. **`wat/fmt.wat:163`** — `emit`, the entry.
5. **`wat-scripts/fmt/rules/{defn,let,match,siblings}.wat`** — all four rules. Each currently ends
   `:then [(:wat::fmt::Break :id ?x :indent (:wat::rete::i64::+ ?pc 1 :undefined 2))]` and each
   carries a `(:wat::grep::Span (?p <- :id) (?pc <- :col))` pattern **purely to feed that
   arithmetic**. Both go. The DESIGN's table says which kind each Break becomes.

## SKETCH

```wat
(:wat::core::defrecord :wat::fmt::Break
  [id   <- :wat::core::i64
   kind <- :wat::core::keyword])   ;; :block | :align
```

```wat
;; in emit-node, when a child carries a Break:
;;   :block  ->  this form's indent + 2
;;   :align  ->  the column at which this container's opening delimiter was emitted, + 1
;; NOTHING here consults ast-span / extent-of for a column. Spans still locate COMMENTS; they
;; must not decide an indent.
```

```wat
;; a rule, after — note what is GONE, not just what changed
(:wat::rete::defrule :fmt::match-arm-per-line
  :when [ … (:wat::rete::where (:wat::rete::i64::> ?ai 1))]     ;; no Span, no ?pc
  :then [(:wat::fmt::Break :id ?arm :kind :block)])
```

## BLAST RADIUS

```
wat/fmt.wat                      Break record + emit-node's indent computation
wat-scripts/fmt/rules/*.wat      all four: drop the Span-for-column pattern, name a kind
```

**No Rust change. No new intrinsic. No registry row. `ast-span`/`extent-of` stay exactly as they are
for COMMENT placement — only indent stops reading them.**

## STOP TRIGGERS

- **STOP-1 — if a ruled shape cannot be expressed by `:block` or `:align`, STOP and name the shape.**
  Do not add a third kind and do not put a column back. A third kind may well be right, but it is a
  contract change and it is the builder's.
- **STOP-2 — if the emitter needs a source column for anything other than placing a COMMENT, STOP.**
  That is the defect this stone exists to make unrepresentable.
- **STOP-3 — if `fmt(fmt(x)) != fmt(x)` on any fixture, STOP** and report the input and both outputs
  verbatim. Idempotence is the point, not a nice-to-have.
- **STOP-4 — do NOT also change `Claim`/`ClaimedUnder`.** It is a live question
  (`[[REFUTE-claim-the-forms-you-position-not-the-subtree]]`) and with columns gone some collisions
  may cease to exist. **This stone first; that gets re-measured after.** Two fixes at once and
  neither is attributable.
- **STOP-5 — if any existing test goes red, STOP.** Capture the whole block verbatim; do not re-run.

## PRIOR COMPARABLE

`[[SCORE-REFUTE-a-claim-must-cover-a-subtree-not-a-form]]` — same arc, immediately prior, same files.

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's. Clippy or the
floor has caught a real red in **three** of the last four stones; run what proves your change and
leave those to me rather than reporting them green unrun.
