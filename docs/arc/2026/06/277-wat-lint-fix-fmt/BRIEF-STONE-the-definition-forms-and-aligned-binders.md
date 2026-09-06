# BRIEF — STONE: the definition forms, and the aligned `<-`

Give `defrecord` / `defstruct` / `defenum` `defn`'s treatment, and finally align the `<-` column —
in the new field vectors AND in `defn`'s own arg-spec. Read
`[[DESIGN-STONE-the-definition-forms-and-aligned-binders]]` first — it carries both measured defects
and the stride problem.

## READ IN ORDER

1. **`wat-scripts/fmt/rules/defn.wat` + `defn-args.wat`** — the exemplar. `defn.wat` claims the form
   and breaks name/argspec/ret/body; `defn-args.wat` claims the arg-spec VECTOR and breaks its
   children. **The definition forms are this pair, twice more.**
2. **`wat/fmt.wat`, `AlignPairs` + its map + the emitter's padding pass** — the capability to extend.
   It pads child 0 so child 1 lines up: **stride 2**. A field is a **triple**.
3. **`wat/bracket.wat:32-34`** — the hand-aligned `<-` in the live stdlib. **The target shape.**
4. **`wat/spawn.wat:195-203`** — a real `defenum`: bare variants (`:Shutdown`) and field-carrying
   variants (`:Admin [msg <- :A]`) **mixed in one child list**. ⚠ **Row 4's fixture.**
5. **`wat/core.wat:2118-2121`** — a real `defrecord`, already in the ruled shape by hand.

## SKETCH

```wat
;; rules/defrecord.wat  (+ defstruct, defenum; + load-file! in every driver)
;;   head is :wat::core::defrecord | defstruct | defenum
;;   -> Claim the form; NO Break on child 1 (the name RIDES)
;;   -> Break the field vector "block"; Break ret/body as defn does where they exist
;;
;; rules/defrecord-fields.wat
;;   claims the field VECTOR; Breaks every 3rd child; asserts the stride-3 alignment fact

;; the alignment fact — widen AlignPairs, or a sibling:
;;   pad child i so child i+1 lines up, for i stepping by STRIDE
;;   2 = key/value (today)     3 = name/binder/type (fields, and defn's arg-spec)
```

## BLAST RADIUS

```
wat/fmt.wat                          the stride on the alignment fact + its emitter pass
wat-scripts/fmt/rules/defrecord*.wat NEW (and defstruct / defenum)
wat-scripts/fmt/rules/defn-args.wat  assert the stride-3 alignment — row 3
wat-scripts/fmt/run*.wat             the load-file! lines
wat-scripts/fmt/fixtures/            fixtures for rows 1-4
```

**No Rust. `Break`, `Claim`, `BlankBefore`, `TableRow`, `Width`, `AllAtoms` unchanged. Three walls
unchanged.**

## STOP TRIGGERS

- **STOP-1 — row 3 is not optional.** A `defrecord` rule alone satisfies rows 1-2. **`defn`'s
  arg-spec `<-` alignment is the point of this stone**; it has been unbuilt since the first style
  table.
- **STOP-2 — no rule names a column.** Alignment stays the emitter's computation; the wall is 0.
- **STOP-3 — `defenum` mixes bare and field-carrying variants** (`wat/spawn.wat:195`). A rule
  assuming a fixed child position breaks every enum in the corpus. If the shape cannot be handled
  cleanly, **STOP and report it rather than special-casing**.
- **STOP-4 — do NOT touch `:examples`.** The fat-arrow proposal is deferred by the builder.
- **STOP-5 — if any existing test goes red, STOP.** Capture the whole block; do not re-run.

## ⚠ TRAPS

- A file in `rules/` is **not loaded** until a driver `load-file!`s it.
- `:where` admits only **PURE** ops — `string::starts-with` and `rete::or` are refused.
- `filter` returns a **lazy stream**; `length` raises — `into` a Vector first.
- **Every fixture must `wat --check` clean BEFORE the floor.**

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's.
