# Arc 277.1b — the nested-if-=-ladder AUTO-FIX (the keystone pays off)

> **STATUS: SHIPPED (2026-06-17).** First end-to-end `lint → fix`. `lint-fix-file` rewrites a 3-deep
> ladder into `(:wat::core::contains? (:wat::core::HashSet :wat::type::Infer "a" "b" "c") x)` —
> SURGICALLY: the surrounding `defn`/param-vector/arrow/return-type stayed byte-identical (eyeballed on
> the orchestrator's own build), confirming the `ast-end-span` extent math. `FixEdit` record;
> `Finding.fix : Option<FixEdit>` (None for the other two rules); `fix-text-span-len` (offset-of(end) −
> offset-of(start)) in fix.wat; `apply-fixes` + `lint-fix-file` riding `fix-text-apply`; new-text built
> via `format` (dogfood). Weighed: autofix gate 1/1, ladder-report 1/1, concat 1/1, deftest 260/1
> (+Case 7), deporder 0 (lint→fix dep order-satisfied), lib 929/36. The keystone paid off. Opened +
> shipped 2026-06-17.

## Why now

277.1 shipped the `nested-if-=-ladder` rule REPORT-ONLY because computing the edit `old-len` for a whole
`(...)` form needs the form's END (STOP-1, the keystone gap). Arc 281 closed it: `ast-end-span` returns
the end `{:line,:col}`. `fix.wat` already has the rest — `fix-text-offset-of` (`{:line,:col}`→flat
offset), `fix-text-apply` (right-to-left splice), and the `reverse`-then-apply pattern. So the fix is
now expressible.

## The contract (pinned)

### 1. The fix shape — a typed `FixEdit` (no stringly-typed magic)
A new record in `wat/lint.wat` (beside `Finding`):
```clojure
(:wat::Record::def :wat::lint::FixEdit
  [start-line <- :wat::core::i64
   start-col  <- :wat::core::i64
   end-line   <- :wat::core::i64
   end-col    <- :wat::core::i64
   new-text   <- :wat::core::String])
```
It carries the node's EXTENT as positions (from `ast-span` + `ast-end-span`) + the replacement text.
**Position-based, not flat-offset-based** — the rule has the spans but NOT the source; the applier holds
the source and flattens. This keeps rules pure functions of the form.

### 2. `Finding.fix : String` → `:wat::core::Option<:wat::lint::FixEdit>`
`None` = no fix (report-only); `Some(fe)` = an auto-fix. Ripples to the THREE constructors:
- `violation->finding` (`lint.wat:320`) → `(:wat::core::None)` (load-order has no mechanical fix).
- `make-concat-finding` (277.1c) → `(:wat::core::None)` (the concat→format fix is the NEXT stone).
- `make-ladder-finding` → `(:wat::core::Some <fe>)` (the rewrite).

(The `Finding` doc comment's "fix is String, '' = no fix" note updates to the Option shape.)

### 3. `make-ladder-finding` computes the `FixEdit`
It already receives `form`, `var-name`, `lits` (Vector<String> of the literal texts, e.g. `"\"a\""`).
Add:
- `sp = (ast-span form)`, `ep = (ast-end-span form)` → the four positions.
- `new-text` via **format (dogfood it):**
  `(:wat::core::format "(:wat::core::contains? (:wat::core::HashSet :wat::type::Infer {lits}) {var})" :lits (:wat::core::string::join " " lits) :var var-name)`
  → e.g. `(:wat::core::contains? (:wat::core::HashSet :wat::type::Infer "a" "b" "c") x)`.
- `fix = (Some (FixEdit start-line start-col end-line end-col new-text))`.

### 4. `fix.wat` primitive — `fix-text-span-len` (the structural old-len)
```clojure
(:wat::core::defn :wat::fix::fix-text-span-len
  [start-span <- :wat::std::HashMap<...>  end-span <- :wat::std::HashMap<...>  lines <- :wat::core::Vector<wat::core::String>]
  -> :wat::core::i64
  ;; offset-of(end) - offset-of(start)
  (:wat::core::i64::- (:wat::fix::fix-text-offset-of end-span lines)
                      (:wat::fix::fix-text-offset-of start-span lines)))
```
(`fix-text-offset-of` takes a `{:line,:col}` map; build those from the FixEdit's fields, or pass the
maps. Match `fix-text-offset-of`'s existing arg shape — `lint.wat:235` shows `(ast-span form)` is the
map.)

### 5. The applier — `apply-fixes` + `lint-fix-file` (in `wat/lint.wat`, calling `fix.wat`)
```clojure
(:wat::lint::apply-fixes [sf <- SourceFile  findings <- Vector<Finding>] -> :wat::core::String)
```
- `src = (SourceFile/source sf)`, `lines = (string::split src "\n")`.
- For each finding whose `Finding/fix` is `Some(fe)`: build `Tuple(off, old-len, new-text)` where
  `off = (fix-text-offset-of {:line (FixEdit/start-line fe) :col (FixEdit/start-col fe)} lines)`,
  `old-len = (fix-text-span-len <start-map> <end-map> lines)`, `new-text = (FixEdit/new-text fe)`.
- Collect ascending, `reverse`, `(:wat::fix::fix-text-apply src rev-edits)`.
- `lint-fix-file [sf] -> String` = `(apply-fixes sf (lint-file sf))` — the convenience entry the probe
  calls.

## Proof

- **`tests/probe_arc277_1b_ladder_autofix.rs`** (un-ignore): a 3-deep ladder source → `lint-fix-file` →
  output contains `contains?` + `HashSet` and NO longer contains the nested `(:wat::core::if (:wat::core::= x`.
- **deftest** (`wat-tests/lint.wat`, Case 7): the same, asserting the rewrite, AND a no-fix file
  (a clean form) round-trips byte-identical through `lint-fix-file`.
- **Floors**: lib 929/36, deftest (+1 → 260/1), deporder 0 (the new lint→fix dep is order-satisfied:
  fix.wat loads before lint.wat, `stdlib.rs:263`<`288`).

## Out of scope (rejected, not deferred)

- **The concat→format auto-fix** — the SAME machinery (FixEdit + apply-fixes) but a different new-text;
  the NEXT stone. `make-concat-finding` stays `None` here.
- **A general lint→fix CLI / the whole-corpus sweep** — that's the SWEEP stone, after both auto-fixes.
- **Comment/format preservation beyond the spliced form** — `fix-text-apply` already preserves
  everything outside `[off, off+old-len)` byte-identically; nothing extra needed.

## Four questions

- **Obvious?** YES — the rule already SAYS "use contains? instead"; the fix writes exactly that text.
- **Simple?** YES — one record, one Finding-field type change, one offset-math primitive, one applier;
  all riding existing `fix-text-apply`/`offset-of`.
- **Honest?** YES — `Option<FixEdit>` makes "has a fix" structural (no `""`-sentinel magic); the rewrite
  is proven by applying it and reading the diff, not asserted.
- **Good UX?** YES — `lint-fix-file` turns a report into a corrected source; the fix is the cure the
  message names.

## Blast radius

`wat/lint.wat` (FixEdit record + Finding.fix type change + 3 constructor updates + apply-fixes +
lint-fix-file), `wat/fix.wat` (fix-text-span-len), `wat-tests/lint.wat` (Case 7), and the probe
(un-ignore). NO Rust changes. The deporder gate must stay 0 (fix.wat already precedes lint.wat).
