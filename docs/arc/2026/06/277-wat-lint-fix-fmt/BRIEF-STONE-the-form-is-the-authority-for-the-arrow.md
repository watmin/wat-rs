# BRIEF — STONE: the FORM is the authority for `->`

Replace the grammar-indexed `->` glue with a lexical one: find the `->` child in the form, glue the
child after it. Read `[[DESIGN-STONE-the-form-is-the-authority-for-the-arrow]]` first — it carries
the index-shift that causes the defect and the three measurements behind the change.

## READ IN ORDER

1. **`wat-scripts/fmt/rules/siblings.wat`** — R11's `not (Slot …)` condition. **This is the site.**
   It becomes a lexical test instead of a `Slot` join.
2. **`wat/fmt.wat`, the `Slot` builder and its map** — leave standing; after this stone it has no
   consumer, and that is a finding to REPORT, not a deletion to make.
3. **`wat-scripts/fmt/rules/defn.wat`** — `defn-ret-break` already finds `->` **by name** in the
   form (`Named ?an = "->"`). ★ **The pattern you need is already written there. Copy it.**
4. **`wat-scripts/fmt/fixtures/generic-fn.wat`** — the failing shape.
5. **`wat/core.wat:1349`** — the real generic `fn` in the stdlib this protects.

## SKETCH

```wat
;; siblings.wat — withhold a Break for the child immediately AFTER a `->` child.
;; Purely positional within the form; no grammar, no Slot, no index arithmetic.
:when [ … the child ?c at index ?ci …
        ;; there is a sibling `->` at index ?ci - 1
        (:wat::grep::Node  (?arrow <- :id) (?p <- :parent) (?ai <- :index))
        (:wat::grep::Named (?arrow <- :id) (?an <- :name))
        (:wat::rete::where (:wat::rete::string::= ?an "->"))
        (:wat::rete::where (:wat::rete::i64::= ?ai (:wat::rete::i64::- ?ci 1 :undefined 0)))
        … ]
;; expressed as a NEGATED join so R11 withholds, mirroring today's `not (Slot …)`.
```

## BLAST RADIUS

```
wat-scripts/fmt/rules/siblings.wat   the glue condition
```

**No Rust. No new record. `Slot` LEFT STANDING (report its consumer count; do not delete). `Break`,
`Claim`, the type-application rule and the three walls all unchanged.**

## STOP TRIGGERS

- **STOP-1 — do NOT delete `Slot`.** It becomes consumer-less and that is the builder's call. Report
  the count; leave the code.
- **STOP-2 — the ret-spec must be BOTH TOKENS ON THE SAME LINE.** Not "each on its own line". The
  previous stone passed a row worded that way and shipped the defect.
- **STOP-3 — do NOT touch `:-`.** The type-application rule already handles it lexically.
- **STOP-4 — if a `->` appears where gluing is wrong, STOP and report the shape.** `defclause`'s
  arrow is NESTED inside a vector (`:name [-> :T] …`) — a sibling-index test should not reach it,
  but confirm rather than assume.
- **STOP-5 — if any existing test goes red, STOP.** Capture the whole block; do not re-run.

## ⚠ THE TRAPS

- `filter` returns a **lazy stream**; `length` raises. `into` a Vector first.
- A file in `rules/` is **not loaded** until a driver `load-file!`s it.
- **Validate a probe FIRES before reading its silence as a result.**

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's.
