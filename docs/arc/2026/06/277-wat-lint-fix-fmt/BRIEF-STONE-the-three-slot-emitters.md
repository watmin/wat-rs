# BRIEF — STONE: the three slot emitters

Make a tagged literal one form so the formatter stops orphaning `#wat.doc/Row` from its map, then
give `:args` and `:ret` real layout in `print.rs`. **`:examples` stays the rules' job.** Read
`[[DESIGN-STONE-the-three-slot-emitters]]` first — it carries the captured `print` output, the
dependency fact that rules out one design outright, and the blocker.

## READ IN ORDER

1. **`wat/fmt.wat` + the rule files** — whatever decides that `#wat.doc/Row {…}` is two forms.
   ⚠ **Reproduce it first:** `#wat.doc/Row {:a 1 :b 2}` → `FORMS=2` with a blank between. **This is
   the blocker and it comes before any Rust.**
2. **`crates/wat-doc/src/print.rs:165` `print_args`, `:188` `print_ret`, `:197` `print_examples`** —
   the three single-line emitters. **Only the first two change.**
3. **`crates/wat-doc/src/print.rs:104` `push_edn_string`** — `:doc`'s emitter. **Already correct;
   read it so you do not "improve" it.** Real newlines, continuations at the content column.
4. **`crates/wat-doc/Cargo.toml` and `crates/wat-macros/Cargo.toml`** — wat-doc depends only on
   wat-reader; wat-macros is a **proc-macro** crate that depends on wat-doc. **This is why `print`
   cannot call the formatter.**

## SKETCH

```
;; FIRST — the blocker. A tagged literal is ONE form:
#wat.doc/Row {:a 1 :b 2}     FORMS=1, tag on the brace's line

// THEN — print.rs, data slots only:
:args [[session :wat.rete/Session "the compiled session (…)"]
       [alpha_id :wat.core/i64 "the AlphaNode id for this condition"]
       …]
:ret  [:wat.rete/DerivationStep
       "the per-edge explain payload (…)"]        // only because 137 > budget

// :examples UNCHANGED in Rust. The post-pass dresses it:
//   format-source over the emitted row -> the ruled shape (proven post-seam)
```

## BLAST RADIUS

```
wat/fmt.wat or a rule file    a tagged literal is one form
crates/wat-doc/src/print.rs   print_args, print_ret          ← and NOTHING else
tests/                        a BYTE golden + the negative control
```

## STOP TRIGGERS

- **STOP-1 — do row 2 FIRST and report before touching `print.rs`.** If `#tag {…}` cannot become one
  form, the post-pass orphans the tag and this stone's output is worse than today's. Say so and stop.
- **STOP-2 — do NOT lay out `:examples` in Rust.** It holds wat forms; Rust laying them out is a
  second formatter beside `wat/fmt.wat`. If that feels necessary, **STOP and report** — it is the
  design's central cut.
- **STOP-3 — do NOT try to call the formatter from `print.rs`.** Measured impossible:
  `wat-doc` is a build-time dep of the proc-macro crate. Not awkward — unavailable.
- **STOP-4 — do NOT touch `push_edn_string` or `:doc`'s shape.** It is already right; a diff that
  moves `:doc` is a regression.
- **STOP-5 — the golden is diffed as BYTES.** `assert_edn_matches_file!` is structural and passed for
  weeks while `:doc` was in the wrong shape. If a byte compare cannot be written, STOP.
- **STOP-6 — if any existing test goes red, STOP.** Capture the block verbatim; do not re-run.

## ⚠ TRAPS

- **wat has NO raw stdout.** All six `kernel` stdio verbs EDN-encode; `pprintln` too. Capture with
  `io::write-file` or a `println!` inside a test, then `diff` the file. **Do not read output through
  a shell decoder** — one mishandling `\n` inside a string is what made this arc's reporting
  untrustworthy for an afternoon.
- **The `:examples` post-pass works only AFTER the three-spellings seam**, because the row's heads
  are dotted (`:wat.core/let`). A pre-seam measurement said the row could not be formatted; that
  result is stale and was re-measured.
- **`:ret` needs both cases** — one over budget that breaks, one under that does not.

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's. Run the cheap
targeted checks — the `FORMS=1` reproduction, the byte diff, the post-pass over the emitted row — and
report the numbers.
