# BRIEF — STONE: `pprintln` is the one printer

Give the runtime's EDN pretty-printer the two things it lacks — prose strings that keep their
newlines, and `:examples` dressed by the fmt rules — so a `#wat.doc/Row` prints correctly from a
value. Read `[[DESIGN-STONE-pprintln-is-the-one-printer]]` first; it carries the measurements that
make three of the four pieces free.

⚠ **This replaces `[[HALT-the-three-slot-emitters-is-withdrawn]]`.** That stone pointed at
`print.rs` because I believed wat could not print raw text. It can.

## READ IN ORDER

1. **`crates/wat-doc/src/print.rs:104` `push_edn_string`** — *"newlines left LITERAL, continuation
   lines indent to the content column."* **This is the prose rendering to move, not invent.** Read
   it; do not modify it.
2. **`src/intrinsic/kernel/stdio.rs:102` `pprintln`** and whatever EDN writer it calls. **The site.**
3. **`wat/doctest.wat:13-70`** — `Example` and `Row`, the shape a `:wat::doc::Row` record follows.
4. **`crates/wat-doc/src/lib.rs:228` `DocComment`** — the fields the record must mirror.
5. **`wat-scripts/fmt/rules/`** — what `:examples` gets dressed by.

## SKETCH

```
;; 1. the record — the container is then FREE (measured: a record prints #ns/Name { … })
(:wat::core::defrecord :wat::doc::Row
  [doc <- :wat::core::String  added <- :wat::core::String  args <- … examples <- … …])

;; 2. prose rendering, scoped BY THE VALUE — not a flag, not a second verb:
;;      "line one\nline two"   ->   "line one
;;                                   line two"
;;    legitimate only because edn::read round-trips it — verify that, do not assume it

;; 3. :examples through the rules — pprintln is IN the runtime, so it can reach them
```

## BLAST RADIUS

```
src/intrinsic/kernel/stdio.rs + the EDN writer   prose rendering, :examples routing
wat/                                             the :wat::doc::Row record
tests/                                           a BYTE golden + the negative control
```

**`crates/wat-doc/` is NOT touched.**

## STOP TRIGGERS

- **STOP-1 — do NOT build the container.** A record already prints `#ns/Name { … }` with the tag on
  the brace's line. If you are prepending a tag string, stop and re-read the DESIGN.
- **STOP-2 — the prose scope must come from the VALUE.** Not a caller flag, not a `pprintln-doc`
  sibling. There are already two printers; a third is the failure. **If it cannot be derived, STOP
  and report what is missing.**
- **STOP-3 — literal newlines are legitimate ONLY while `edn::read` round-trips them.** Verify it in
  a test (row 4). If a change breaks that, STOP.
- **STOP-4 — do NOT touch `crates/wat-doc/`.** `print.rs` serves the proc-macro path and cannot reach
  the formatter; it stays as it is.
- **STOP-5 — do NOT make the prose mode global.** Every multi-line string in the corpus changing
  shape is a far wider stone. Row 6 guards it.
- **STOP-6 — if any existing test goes red, STOP.** Capture the block verbatim; do not re-run.

## ⚠ TRAPS

- **`pprintln` of a VALUE is already unescaped.** The escaping everyone hit is `println`/`pprintln`
  of a **String** — correct EDN behaviour. Print values; do not reach for `io::write-file` or a shell
  decoder. **A false "wat has no raw stdout" is what produced the withdrawn stone.**
- **`FORMS=2` is not on this path.** That is about re-reading a row as source text; nothing here
  re-reads.
- **A structural golden cannot verify layout** — `assert_edn_matches_file!` passed for weeks with
  `:doc` in the wrong shape. Diff bytes.
- `:args` is already correct. A diff that moves it is a regression, not an improvement.

## THE FLOOR IS MINE

`scripts/floor.sh` and `cargo clippy --all-targets -D warnings` are the orchestrator's. Run the cheap
targeted checks — the printed row, the `edn::read` round trip, the byte diff — and report the numbers.

---

## ⚠ THE TREE CARRIES UNCOMMITTED WORK FROM THE WITHDRAWN STONE

The halt reached you mid-strike. Two files are modified and **not committed**:

```
crates/wat-doc/src/print.rs   the three slot emitters — CONTRADICTS this brief (STOP-4)
wat/fmt.wat                   tag-symbol? + emit-tops — stitches #tag onto the map's line
```

**`print.rs` is out of scope now** and should be reverted: this stone does not touch `wat-doc`.

**`wat/fmt.wat`'s tag-stitching is a judgement call and it is yours to make.** It solves a real
defect — `#wat.doc/Row` orphaned from its map when a row is read back as source — but **nothing on
this path re-reads a row**, so it is not needed here. Keep it as its own small stone if it stands on
its own merits, or revert it; **do not fold it into this one to avoid throwing it away.**

Either way, say which you did and why. Neither choice is wrong; carrying it silently is.
