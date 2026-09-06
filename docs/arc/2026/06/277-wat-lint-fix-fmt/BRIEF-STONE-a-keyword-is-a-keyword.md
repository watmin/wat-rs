# BRIEF — STONE: a keyword is a keyword, not a tagged value

Split a slash-in-name keyword on its `/` instead of its last `::`, so it renders as the keyword it
is. Read `[[DESIGN-STONE-a-keyword-is-a-keyword]]` first — the bug is the split, and the shape it
should produce already renders elsewhere in the same output.

## READ IN ORDER

1. **`src/edn/render.rs:4483` `keyword_from_wat_path`** — **the site.** It uses `path`/`leaf` (split
   on the last `::`), which leaves the `/` inside the name, which `try_ns` then correctly refuses.
2. **`crates/wat-reader/src/identifier.rs:250/260` `receiver`/`method`** — the right pair, already on
   the ONE door. ⚠ `tests/lint/one_name_grammar.rs` makes a hand-rolled split a RED; this arc has
   been caught by it once already.
3. **`src/edn/bridge.rs:100` `verbatim_keyword`** — the fallback. **Read its doc before touching it:**
   it replaced a bare String, *"a SILENT TYPE CHANGE … with no diagnostic."* **It stays.**
4. **`tests/cli/pprintln_doc_row__step_payload.edn`** — the golden carrying seven of these. Row 4.

## SKETCH

```rust
// keyword_from_wat_path — pick the split by what the name CONTAINS:
//
//   "wat::rete::Explained/support"   has a '/'  ->  receiver / method
//        ns   = "wat.rete.Explained"   name = "support"     ->  :wat.rete.Explained/support
//
//   "wat::core::first"               no '/'     ->  path / leaf      (UNCHANGED)
//        ns   = "wat.core"             name = "first"       ->  :wat.core/first
//
// try_ns already accepts a dotted multi-segment ns — `:wat.rete.i64/<` renders today.
// Anything try_ns still refuses keeps going to verbatim_keyword.
```

## BLAST RADIUS

```
src/edn/render.rs   keyword_from_wat_path — the split only
tests/              the round-trip test, the negative control, and the goldens that move
```

**`verbatim_keyword` is NOT modified. `try_ns` is NOT modified.**

## STOP TRIGGERS

- **STOP-1 — row 3 or nothing.** If `:wat.rete.Explained/support` does not DECODE back to
  `:wat::rete::Explained/support`, stop. A pretty lossy rendering is worse than the honest tagged
  carrier we have.
- **STOP-2 — do NOT delete `verbatim_keyword` or its arm.** Shrink what reaches it. A keyword EDN
  genuinely cannot spell must still be carried, not mangled.
- **STOP-3 — do NOT hand-roll the split.** `receiver`/`method` are on the door and the lint enforces
  it.
- **STOP-4 — ENUMERATE every golden whose bytes move**, and attribute each to the `X/y` shape. An
  unattributed diff is a finding. **Do not bulk-regenerate goldens.**
- **STOP-5 — if any existing test goes red for a reason you cannot attribute to `X/y`, STOP.**
  Capture the block verbatim; do not re-run.

## ⚠ TRAPS

- **The site's own comment explains the symptom as the cause.** It says the carrier avoids *"a
  two-slash keyword the reader cannot parse"* — that two-slash form is what the wrong split creates.
  Do not preserve the behaviour because the comment justifies it.
- **`try_ns` is right to refuse a slash in a name.** The fix is upstream of it.
- **This moves EDN bytes corpus-wide.** 89 `@example` lines carry the shape and the doctest gate runs
  them. Row 9.

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's. Run the cheap
targeted checks — the render, the decode, the doc row's count of `#wat.ast/Keyword`, and the golden
enumeration — and report the numbers.

---

## ⚠ THIS LANDS ON TOP OF THE UNCOMMITTED `pprintln` STONE

`[[SCORE-STONE-pprintln-is-the-one-printer]]`'s work is in the tree, verified and **not committed**
— because its golden `tests/cli/pprintln_doc_row__step_payload.edn` carries **seven**
`#wat.ast/Keyword` entries, and this stone removes them. Committing it first would bake in the shape
we are here to delete.

**So both land together.** Do not revert the pprintln work; build on it.

- **Row 4's before/after is exact:** that golden goes from **7** occurrences to **0**, and
  `(:wat.rete.Explained/support ex)` appears as a call.
- **Regenerate that one golden deliberately**, and it is the clearest instance of STOP-4: name it, and
  attribute its diff to the `X/y` shape.
- Everything the pprintln SCORE established stands — the container, `:doc` as prose, the scoping by
  record class, `print.rs` untouched. **This stone does not revisit any of it.**
