# BRIEF — STONE: the exploded form, and blank lines after a complex binder

Two parts, **in order**. Part 1 green before part 2 starts. Read
`[[DESIGN-STONE-the-exploded-form]]` first — it carries why *"break every child"* is the wrong rule
and why the leading-atom run is the whole difference.

## READ IN ORDER

1. **`wat-scripts/fmt/rules/siblings.wat`** — R11 as it stands. Its `:when` joins `?a`/`?b` on
   different LINES; that is the all-or-nothing test and it is what goes.
2. **`wat-scripts/fmt/rules/let-bindings.wat`** — the vector rule. Part 2's `BlankBefore` rule is
   its sibling and lives beside it.
3. **`wat/fmt.wat`, the `Break` record and `emit-node`** — where a `BlankBefore` must be consulted.
   `Break` is UNCHANGED; `BlankBefore` is a NEW record.
4. **`wat/fmt.wat`, `breaks-map` + the three walls** — an unknown `Break.kind`, a rule positioning a
   grandchild, two rules disagreeing about one node. **A `BlankBefore` map wants the same care**;
   copy the shape rather than inventing one.
5. **`wat-scripts/fmt/run-all.wat`** — ⚠ **drivers `load-file!` rule files EXPLICITLY.** A new file
   in `rules/` is NOT loaded. Any new rule file needs its driver edited too.

## SKETCH

```wat
;; PART 1 — siblings.wat. The all-or-nothing join goes; a leading-atom test replaces it.
;;   child index 0        the head, never breaks
;;   a child is COMPOUND  -> kind "list"/"vector"/"map"/"set" per Node.kind
;;   the first compound child, and every child after it, gets a Break "block"
;;   atoms BEFORE the first compound get no Break — they ride
```

```wat
;; PART 2 — a new record and a new rule file
(:wat::core::defrecord :wat::fmt::BlankBefore [id <- :wat::core::i64])

;; wat-scripts/fmt/rules/let-blank.wat  (and its load-file! line in every driver)
;;   for binders at even index i > 0 in a let's binding vector:
;;     if the PREVIOUS binder's value (index i-1) is a form with a compound child
;;     -> (:wat::fmt::BlankBefore :id <this binder>)
```

## BLAST RADIUS

```
wat-scripts/fmt/rules/siblings.wat   PART 1 — the rule's :when
wat/fmt.wat                          PART 2 — BlankBefore record, its map, emitter support
wat-scripts/fmt/rules/let-blank.wat  PART 2 — NEW
wat-scripts/fmt/run*.wat             PART 2 — the load-file! line
wat-scripts/fmt/fixtures/            NEW fixtures for the acceptance rows
```

**No Rust. No intrinsic. No registry row. `Break`, `Claim`, `Comment` unchanged. The three walls
stay exactly as they are.**

## STOP TRIGGERS

- **STOP-1 — do NOT make R11 "break every child".** It would put `m` on its own line in
  `(assoc m (f b) (g b))` and contradict a ruling. The leading-atom run is the rule.
- **STOP-2 — part 1 must be GREEN before part 2 is started.** Part 2's trigger depends on part 1's
  output; landing both at once makes a failure unattributable. Report part 1's rows first.
- **STOP-3 — if a blank line ACCUMULATES across passes, STOP.** That means the trigger is reading
  the previous pass's output instead of structure. Report both passes verbatim; do not add a
  "collapse consecutive blanks" hack — that is the patch for a situation that should not be
  constructed.
- **STOP-4 — if any existing fixture changes shape unexpectedly, STOP and report it.** Part 1 makes
  R11 strictly more active; a surprise there is a finding about a rule's coverage, not noise.
- **STOP-5 — if any existing test goes red, STOP.** Capture the whole block; do not re-run.

## PRIOR COMPARABLE

`[[SCORE-STONE-conflicting-breaks-raise]]` — same arc, immediately prior, and its verdict records the
driver-loading trap that cost three mis-aimed sabotages. **Validate any probe fires before you read
its silence as a result.**

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's.
