# BRIEF — STONE: the table rule claimed a definition, and the file lost its end

Stop `table.wat` claiming definition forms, give it the fit test `atoms.wat` already has, and stop
`emit` putting a blank line after the last top-level form. Read
`[[DESIGN-STONE-the-table-rule-claimed-a-definition]]` first — it carries the ablation, the
padding-vs-code accounting, and the measurement showing the guard alone is not enough.

## READ IN ORDER

1. **`wat-scripts/fmt/rules/table.wat`** — the whole file, 34 lines. `table-start` fires on any 2+
   adjacent `list` siblings sharing head and key-sequence. At a file's top level, consecutive `defn`s
   of the same arity satisfy that. **This is the site.** ⚠ `table-extend` keys off an existing
   `TableRow`, so a guard on `table-start` alone should be sufficient — **verify that, do not assume
   it.**
2. **`wat-scripts/fmt/rules/defrecord.wat:79-81`** — `(:wat::rete::where (:wat::rete::string::not= ?cn ":-"))`.
   **This is the guard idiom.** Copy its shape.
3. **`wat-scripts/fmt/rules/atoms.wat`** — the fit test that already exists: inline iff every value is
   an atom **and it fits**. `table.wat` needs the same clause.
4. **`wat/fmt.wat`, `:wat::fmt::emit`** — `acc2 (:wat::fmt::ensure-blank acc1)` after the fold. That
   is the terminal blank. `ensure-nl` is beside it.
5. **`wat/deporder.wat`** — the real file. Its 8 top-level `defn`s are the ones being claimed.
6. **`wat-scripts/fmt/fixtures/kw-table.wat`, `pair-table.wat`, `pos-table.wat`** — the tables that
   must STILL align. ⚠ **`pos-table.wat` is the positional (`#N`) key-sequence case — the same shape
   that catches the top-level `defn`s.** It is the fixture a too-wide guard kills, and it is the
   reason the guard must key on the HEAD and never on the key-sequence.

## SKETCH

```wat
;; table-start :when — ONE structural guard. A table is DATA; data is never top-level.
;; Measured against a five-head list and against removing the rule: this is strictly
;; better than the list (spawn 75 -> 9) and equals removal on spawn and deporder.
(:wat::rete::where (:wat::rete::i64::not= ?p 0))
;; NOT a head list. The corpus has twelve definition heads at column 0 and a list of
;; them is a list that gets patched again. The registry cannot help — defn/defrecord/
;; defstruct/deftest are wat-level macros with ZERO registry rows.

;; and the fit test: a group whose aligned width exceeds the budget does not align.
;; Falling back means each member formats BY ITS OWN RULES — never truncated, never re-broken.

;; emit: the last top-level form gets ONE newline, not a blank line.
```

## BLAST RADIUS

```
wat-scripts/fmt/rules/table.wat      the guards + the fit test
wat/fmt.wat                          one word in emit
wat-scripts/fmt/fixtures/            ONE new fixture — row 10's oversized non-definition group
```

**No Rust. No new record. No new rule file.** If a new fact turns out to be needed for the fit test,
say which and why before writing it.

## STOP TRIGGERS

- **STOP-1 — do NOT turn the guard back into a head list.** If `?p != 0` cannot express "not at the
  top level" in this rule, STOP and report which fact is missing. A list of definition heads is the
  rejected design, not the fallback.
- **STOP-2 — the fit test must FALL BACK, never truncate and never re-break.** A group that does not
  fit formats as if `table.wat` had not fired. If that cannot be expressed without a new fact, STOP
  and report.
- **STOP-3 — `defenum`'s tag padding is `AlignPairs`, NOT `table`.** If the guard changes
  `defenum-mixed` or `defenum-spawn-shape` output at all, the guard is too wide. STOP.
- **STOP-4 — the file must end `)` + ONE newline.** Not a blank line, and not a missing newline. Three
  outcomes; only one passes.
- **STOP-5 — do NOT touch `if`, `cond`, or defect E.** `if` and `cond` are ruled and are the NEXT
  stone. A trailing comment's home is still the builder's call.
- **STOP-6 — if any existing test goes red, STOP.** Capture the whole block verbatim; do not re-run.

## ⚠ TRAPS

- **`wat/*.wat` is FROZEN into the release binary.** Editing `wat/fmt.wat` and re-running proves
  nothing without `cargo build --release`. Rule files under `wat-scripts/` **are** read from disk.
- **Row 10 is the row that proves the fit test fires.** The definition-head guard alone already takes
  `deporder` to `over120=0`, so a no-op fit test passes rows 2-6. Build the oversized
  non-definition fixture first and confirm it FAILS before the fit test exists.
- **A file in `rules/` is not loaded until a driver `load-file!`s it.**
- Every fixture must `wat --check` clean before the floor.

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's. Run the
cheap targeted checks — `--check`, the census, the fixtures — and report the numbers.
