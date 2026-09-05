# BRIEF — STONE: keyword arguments, one pair per line, values aligned

Add `AlignPairs`, a rule for keyword-argument runs, and emitter support for column alignment. Read
`[[DESIGN-STONE-keyword-args-and-aligned-values]]` first — it carries both approved shapes, why
alignment is a new capability, and why level 2 is excluded.

## READ IN ORDER

1. **`wat-tests/service-telemetry-bridge.wat:44-50`** — the approved `defservice` shape: a
   positional rides, kwargs one per line, **values aligned**.
2. **`wat/grep.wat:284-289`** — the approved record shape: no positional, values aligned.
   ⚠ **Its `:line`/`:col` inner arguments are ALSO padded — that is level 2 and is OUT of scope.**
3. **`wat/fmt.wat`, `emit-node`** — where a line's indent is computed. **Alignment is computed the
   same way and in the same place**: from what is being emitted, never from the source.
4. **`wat/fmt.wat`, the `BlankBefore` record and its map** — the shape to copy for `AlignPairs`
   (a separate fact, its own map, consulted by the emitter).
5. **`wat-scripts/fmt/rules/let-blank.wat`** — a rule asserting a non-`Break` fact. Copy its shape.
6. **`wat-scripts/fmt/rules/siblings.wat`** — R11, for how a rule reads child index and kind.

## SKETCH

```wat
(:wat::core::defrecord :wat::fmt::AlignPairs
  [form <- :wat::core::i64])       ;; this form's broken children align their SECOND token

;; wat-scripts/fmt/rules/kwargs.wat  (+ its load-file! line in every driver)
;;   a KEYWORD-ARG RUN = a maximal tail of children where the even-offset ones are keywords
;;   -> Break the FIRST key (kind "block"); Break each subsequent key; WITHHOLD from each value
;;   -> assert (AlignPairs :form ?p)
;;   positional children BEFORE the first keyword get no Break — the leading-atom rule rides them
```

```wat
;; emitter: when a form carries AlignPairs, pad each broken child's first token to the widest
;; first token among that form's broken children, so the SECOND tokens share a column.
;; The width comes from what has been EMITTED. No rule names a column.
```

## BLAST RADIUS

```
wat/fmt.wat                        AlignPairs record + map + emitter padding
wat-scripts/fmt/rules/kwargs.wat   NEW
wat-scripts/fmt/run*.wat           the load-file! line
wat-scripts/fmt/fixtures/          fixtures for rows 1-3, 6
```

**No Rust. No registry. `Break`, `Claim`, `BlankBefore` unchanged. The three walls unchanged.**

## STOP TRIGGERS

- **STOP-1 — NO rule may name a column.** `grep -c 'col' wat-scripts/fmt/rules/*.wat` must stay 0.
  A rule ASKS for alignment; the emitter computes the width. This is
  `[[DESIGN-STONE-indent-is-structural]]`'s wall and it holds.
- **STOP-2 — do NOT attempt level-2 (cross-sibling-call) alignment.** It is in `grep.wat:284-289`
  and it is excluded with a reason. Delivering half of it silently is worse than not starting.
- **STOP-3 — a positional before the first keyword must still RIDE the head line.** `defservice`'s
  shape is the gate; a rule that breaks the name onto its own line has failed row 2.
- **STOP-4 — if alignment drifts across passes, STOP.** Padding computed from already-padded output
  is the same class of defect as indent-from-source. Report both passes verbatim.
- **STOP-5 — if any existing test goes red, STOP.** Capture the whole block; do not re-run.

## ⚠ THE TRAPS

- A file in `rules/` is **not loaded** until a driver `load-file!`s it.
- `filter` returns a **lazy stream**; `length` raises — `into` a Vector first.
- **Validate a probe FIRES before reading its silence as a result.**

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's.
