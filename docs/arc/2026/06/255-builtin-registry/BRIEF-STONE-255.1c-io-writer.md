# BRIEF — STONE 255.1c-io-writer (home #12, strike 2 of 3)

Carve the thirteen `:wat::io::IOWriter/*` verbs out of `runtime.rs`'s literal dispatch into
`src/intrinsic/io/writer.rs`. Strike 1 (`io/reader.rs`, ten rows, `b9e28b946`) shipped the directory,
the family claim, and the doc pattern. **This strike copies that pattern; it invents nothing.**

**Your role: you write the text. The orchestrator builds, floors, and clippies.** Do not run `cargo`
in any form. Run everything else in the foreground and block on it — ending your turn ends you, and
nothing will wake you. Do not commit, push, stash, or revert; leave the work in the tree.

## Read in order

1. **`src/intrinsic/io/reader.rs`** — THE TEMPLATE, and it is one strike old. Read its module doc and
   two full rows: `///` doc block → `//` maintainer comment naming the deciding `src/io.rs` line →
   `#[wat_intrinsic]` → thin fn with typed params. Match it exactly.
2. **`src/intrinsic/io/mod.rs`** — add `mod writer;` and move `writer` from "not yet carved" to
   present in the decomposition list. Keep the family claim as written unless your body-reads refute it.
3. **`src/runtime.rs:6444–6511`** — your source arms, in **TWO runs** (see below).
4. **`src/check.rs`** — the thirteen schemes, per verb:
   `new 15818 · open-file 15827 · from-fd 15838 · to-bytes 15847 · to-string 15856 · write 15865 ·
    write-all 15874 · write-string 15883 · print 15892 · println 15901 · writeln 15910 ·
    flush 15919 · close 15932`
5. **The bodies, `src/io.rs`** — read each before assigning its `@Category`:
   `new 1173 · open_file 1198 · from_fd 1279 · to_bytes 1355 · to_string 1371 · write 1412 ·
    write_all 1427 · write_string 1446 · print 1466 · println 1483 · writeln 1501 · flush 1519 ·
    close 1547`
6. **`wat/runtime-meta.wat`** — the `Category` prose. The prose is the ruling, never the variant name.
   ⚠ `:Mutate` is NOT a variant; line 169 records it REFUSED. Do not reach for it.

## ⚠ The arms are in TWO runs, and what sits between them stays

```
6448–6474   nine arms   new · open-file · from-fd · to-bytes · to-string · write ·
                        write-all · write-string · print
6476–6498   ── STAYS ── TempFile/{new,path}, TempDir/{new,path}, read-file, list-dir
                        (strike 3's rows) and `:wat::stdlib::sources`, which is a
                        DIFFERENT family and never moves
6500–6511   four arms   println · writeln · flush · close
```

Delete the two runs; leave everything between them untouched. Then rewrite the `6444–6447` comment:
strike 1 already narrowed it to `IOWriter`, and after this strike no IOWriter arm remains there
either. Point it at `src/intrinsic/io/{reader,writer}.rs` and let it describe only what still sits
below it.

## ★★ THE RET TRAP — this family's return types are NOT uniform, and neighbours disagree

The reader rows were regular. These are not, and a reader who transcribes from the neighbouring row
instead of from `check.rs` will get several wrong:

```
write        → :wat::core::i64      write-all → :()          write-string → :wat::core::i64
print        → :()                  println   → :()          writeln      → :wat::core::i64
flush        → :()                  close     → :()
to-string    → Option<String>       ← NOT String
new          → params: vec![]       ← NULLARY. Zero @arg lines.
```

`writeln` returns `i64` while `println`, sitting directly beside it and doing the visibly same thing,
returns `:()`. **Transcribe every `@arg` and `@ret` from the `check.rs` line given above — never from
the row above it in your own file.** `doc_arg_ret_types_match_checker_scheme` compares by
`assert_eq!` at every floor and this stone will be perturbation-tested, so a copied line goes red.

**You do not edit `src/check.rs`.** If a scheme disagrees with its body, that is a FINDING.

## Categories — decided at the body, never from the name

Thirteen rows, and the design predicts they straddle: `new`/`open-file`/`from-fd` mint or claim,
`to-bytes`/`to-string` hand back a form of what was written, the write/print family pushes at the
world, `flush`/`close` administer. Read `src/io.rs` at the line given and quote the deciding line in
the `//` comment beneath each doc block.

★ Strike 1 returned `rewind` as a row that **would not classify**, with both arguments written down
rather than one tidied away — that was its most valuable output. Do the same. Two candidates to read
with particular care, and no answer is pre-decided:

- **`to-bytes` / `to-string`** — is handing back the accumulated buffer `:Transform` (the output is a
  form of the input) or `:Projection` (reading a component off a handle the caller holds)? Read
  `io.rs:1355` and `1371` and argue it.
- **`close`** — `:Resource` releases a handle. Check whether this body actually releases, or whether
  it flushes-and-marks. `kernel/resource.rs` distinguishes `signal` (administers, does not release)
  from `close'` (the only consumer); the same distinction may or may not apply here.

## Blast radius

`src/intrinsic/io/writer.rs` (new) · `src/intrinsic/io/mod.rs` (list + one `mod`) ·
`src/runtime.rs` (two deletions + one comment). No new types. No `src/check.rs`. No `tests/`. No `.wat`.

## STOP triggers — each rejects; none is a fallback

1. A doc type would not match its `TypeScheme`. STOP; report both readings. Do not adjust `check.rs`.
2. A body does not delegate to the `crate::io::` fn at the line given. STOP; the room map is wrong.
3. A row needs a `Category` variant that is not in `wat/runtime-meta.wat`. STOP; report the argument.
4. Either deletion would take a line that is not one of the thirteen arms — in particular anything in
   the `6476–6498` block. STOP; report what is actually there.

## Acceptance criteria

- `src/intrinsic/io/writer.rs` holds exactly thirteen `#[wat_intrinsic]` rows.
- `grep -c '":wat::io::IOWriter/[^"]*" *=>' src/runtime.rs` → **0**.
- `grep -c '":wat::io::Temp' src/runtime.rs` → **4**, and `":wat::stdlib::sources"` → **1**.
- Every `@arg`/`@ret` transcribes its `check.rs` scheme; `new` carries zero `@arg`.
- Every `@Category` has a `//` comment quoting the `src/io.rs` line that decided it.
- `src/check.rs` and `tests/` show no diff.
