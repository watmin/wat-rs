# BRIEF — STONE: three head spellings, one seam

Make the formatter's ruled shapes fire on **all three** head spellings, through **one** canonicalising
seam, canonicalising to the **clojure target** so that dropping the keyword flavors later is the
deletion of one arm. Read `[[DESIGN-STONE-three-spellings-one-seam]]` first — it carries the
three-way A/B, the census, and why the canonical form's direction is the whole decision.

## READ IN ORDER

1. **`wat-scripts/fmt/rules/defn.wat:32-41`** — `(:wat::grep::Named (?h <- :id) (?n <- :name))` then
   `(:wat::rete::string::= ?n ":wat::core::defn")`. **This is the shape, 33 times over 12 files.**
2. **`wat/grep.wat:188-195`** — the producer. `Named` is built from `(:wat::core::ast-name node)`,
   which returns the spelling **verbatim**. Whatever the canonical fact is, it is born here.
3. **`crates/wat-reader/src/identifier.rs`** — the ONE name parser, and `tests/lint/one_name_grammar.rs`
   which enforces it. ⚠ **It is FQDN-shaped** (`leaf`/`path` split on `::`) and **not exposed to wat**,
   so it does not already solve this — but a hand-rolled split that duplicates it is the defect that
   went red on this arc's `:then` stone.
4. **`wat-scripts/fixes/node-kind-string-to-enum.wat`** — the codemod shape for retargeting the 33
   comparisons. R21.

## SKETCH

```
;; ONE canonicaliser. Three arms in, ONE spelling out — and the spelling out is the
;; CLOJURE TARGET, so the end of the migration is a deletion rather than a rewrite:
;;
;;   ":wat::core::defn"   ─┐
;;   ":wat.core/defn"     ─┼─►   "wat.core/defn"
;;   "wat.core/defn"      ─┘
;;
;; and a rule then reads:
;;   (:wat::rete::where (:wat::rete::string::= ?n "wat.core/defn"))
;;
;; When the keyword flavors die, the first two arms are deleted, the function becomes
;; the identity, and NOT ONE RULE FILE CHANGES. That is the acceptance (row 6).
```

## BLAST RADIUS

```
the canonicaliser             ONE function, ONE home — say which and why in the SCORE
wat/grep.wat                  the canonical fact the rules match on
wat-scripts/fmt/rules/*.wat   12 files, 33 comparisons — BY CODEMOD (R21)
wat-scripts/fmt/fixtures/     the same form in all three spellings
wat-scripts/fixes/<name>.wat  NEW — the recorded migration
```

## STOP TRIGGERS

- **STOP-1 — CANONICALISE TO `wat.core/…`, NOT `:wat::core::…`.** Both directions make all three
  spellings format alike and pass rows 2-4. Only one makes the end of the clojure migration a
  deletion. **If the target direction cannot work, STOP and report why** — do not quietly flip it.
- **STOP-2 — ONE function, ONE home.** Not a helper per rule file, not three parallel comparisons
  per rule. Three parallel comparisons work today and cost a 12-file sweep tomorrow, which is the
  thing the builder asked to avoid.
- **STOP-3 — do NOT hand-roll a second name parser.** If the canonicaliser needs `identifier`'s
  door exposed to wat, **say so and STOP** — that is a real finding about the door, not a licence to
  duplicate it.
- **STOP-4 — 33 comparisons across 12 files is a CODEMOD.** Dry-run on `/tmp` copies, diff, apply,
  commit it. Second run reports 0 changes; a run against a pre-migration file reports 1.
- **STOP-5 — the five real corpus files must format BYTE-IDENTICALLY.** They are all FQDN;
  canonicalising must be a no-op for them. Capture before you start.
- **STOP-6 — every spelling fixture carries ≥2 `defn` args and ≥2 `let` binders.** A one-argument
  form renders identically under the ruled rule and the generic fallthrough and **cannot** prove
  anything. This nearly closed the stone as a non-issue.
- **STOP-7 — if any existing test goes red, STOP.** Capture the whole block verbatim; do not re-run.

## ⚠ TRAPS

- **Row 6 is a DEMONSTRATION, not a claim.** Delete the FQDN arm, show only FQDN fixtures move and
  no rule file is touched, restore it. A SCORE that asserts "dropping a flavor would be one edit"
  has not done row 6.
- **Row 3 is the anti-"looks right" check.** Strip the head token from all three outputs and diff
  them. Three outputs can each look reasonable and differ.
- **The head KIND is not in play.** Spelling #3 is a `symbol` where #1/#2 are `keyword`s, but **no
  fmt rule gates on a head's kind** — `defrecord.wat`'s two `NodeKind::Keyword` tests are on
  defenum VARIANT TAGS. Measured. Do not add a kind test.
- **`wat/*.wat` is FROZEN into the release binary** — rebuild between an edit and a measurement.
- Every fixture must `wat --check` clean before the floor.

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's. Run the cheap
targeted checks — the three-spelling fixtures, the five-file byte-diff, the codemod's two runs, and
row 6's deletion — and report the numbers.
