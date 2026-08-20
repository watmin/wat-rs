# BRIEF — STONE 255.1c-io-fs (home #12, strike 3 of 3 — the close)

Carve the last six `:wat::io::` verbs out of `runtime.rs` into `src/intrinsic/io/fs.rs`. This closes
home #12: after it, `grep '":wat::io::[^"]*" *=>' src/runtime.rs` returns **0**.

Strikes 1 and 2 (`io/reader.rs` 10 rows `b9e28b946`, `io/writer.rs` 13 rows `85174fc3f`) shipped the
directory, the family claim and the doc pattern. **This strike copies them; it invents nothing.**

**Your role: you write the text. The orchestrator builds, floors, and clippies.** Do not run `cargo`
in any form. Run everything else in the foreground and block on it — ending your turn ends you, and
nothing will wake you. Do not commit, push, stash, or revert; leave the work in the tree.

## Read in order

1. **`src/intrinsic/io/writer.rs`** — the template, one strike old and the closest in shape.
2. **`src/intrinsic/io/mod.rs`** — add `mod fs;` and move `fs` to present in the decomposition list.
   This is the strike that lets the module doc say the family is fully carved; say it.
3. **`src/runtime.rs:6444–6473`** — your source. Six arms at `6453–6470`; see the boundary note below.
4. **`src/check.rs`** — the six schemes, per verb:
   `TempFile/new 15938 · TempFile/path 15947 · TempDir/new 15956 · TempDir/path 15965 ·
    read-file 15974 · list-dir 15983`
   ⚠ Those line numbers are the ones the ORCHESTRATOR measured; confirm each by matching the verb
   string, and report any that moved rather than trusting the number.
5. **The bodies, `src/io.rs`** — read each before assigning its `@Category`:
   `temp_file_new 1697 · temp_file_path 1710 · temp_dir_new 1726 · temp_dir_path 1739 ·
    read_file 1770 · list_dir 1801`
6. **`wat/runtime-meta.wat`** — the `Category` prose. The prose is the ruling, never the name.
   ⚠ `:Mutate` is NOT a variant; line 169 records it REFUSED.

## The work

Six rows in `src/intrinsic/io/fs.rs`, same shape as `writer.rs`: `///` doc block → `//` maintainer
comment naming the deciding `src/io.rs` line → `#[wat_intrinsic]` → thin fn with typed params.

**All six delegates take the SAME argument order** — `(args, list_span, env, sym)`. Unlike strikes 1
and 2 there is no split here; copy each call verbatim anyway.

## ⚠ The boundary — one line belongs to another family and must not move

```
6444–6452   the io intro comment + the arc-093 temp-wrapper comment
6453–6470   YOUR SIX
6472–6474   ":wat::stdlib::sources"  ← a DIFFERENT family. It stays. Never carved here.
```

Rewrite the `6444–6452` comment block. After this strike **no `:wat::io::` arm remains**, so the
block should stop describing an io region and simply carry what `:wat::stdlib::sources` needs, with a
pointer to `src/intrinsic/io/`. The arc-093 temp-wrapper prose describes verbs that are leaving —
carry whatever of it is still true into `fs.rs`'s module doc rather than deleting the knowledge.

## Categories — decided at the body

Six rows, and the design expects them to straddle: the two `*/new` verbs mint something whose `Drop`
unlinks a real file or directory; the two `*/path` verbs read a component off a handle the caller
holds; `read-file` and `list-dir` are one-shot filesystem reads that open nothing the caller keeps.
Read `src/io.rs` at the line given and quote the deciding line in the `//` comment beneath each block.

★ Both prior strikes returned a row that **would not classify** — `rewind` and `IOWriter/new` — with
both arguments written down rather than one tidied away. Those were their most valuable outputs. Do
the same if it happens; do not force a fit to make the column tidy. No answer here is pre-decided.

**You do not edit `src/check.rs`.** If a scheme disagrees with its body, that is a FINDING.

## Blast radius

`src/intrinsic/io/fs.rs` (new) · `src/intrinsic/io/mod.rs` (list + one `mod`) · `src/runtime.rs`
(one deletion + one comment). No new types. No `src/check.rs`. No `tests/`. No `.wat`.

## STOP triggers — each rejects; none is a fallback

1. A doc type would not match its `TypeScheme`. STOP; report both readings.
2. A body does not delegate to the `crate::io::` fn at the line given. STOP; the room map is wrong.
3. A row needs a `Category` variant not in `wat/runtime-meta.wat`. STOP; report the argument.
4. The deletion would take `":wat::stdlib::sources"` or anything that is not one of the six. STOP.

## Acceptance criteria

- `src/intrinsic/io/fs.rs` holds exactly six `#[wat_intrinsic]` rows.
- `grep -c '":wat::io::[^"]*" *=>' src/runtime.rs` → **0** — the family is fully carved.
- `grep -c '":wat::stdlib::sources"' src/runtime.rs` → **1**.
- Every `@arg`/`@ret` transcribes its `check.rs` scheme; both `*/new` verbs are nullary (zero `@arg`).
- Every `@Category` has a `//` comment quoting the `src/io.rs` line that decided it.
- `src/check.rs` and `tests/` show no diff.
