# BRIEF — ③b-ii phase ②a: the codemod, and its dry run

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`; use `git -C /home/john/work/holon/wat-rs`.

Read `DESIGN-the-flip-asks-843-times.md` first. Phase ① is landed (`579eb8a95`).

## ⛔ THIS PHASE DOES NOT TOUCH THE REAL CORPUS

You write the migration and prove it on a **`/tmp` copy**. The orchestrator performs the real
landing, because the separator flip and the corpus rewrite must be one commit and the tree cannot
build between them. Your deliverable is the codemod plus a diff someone can read.

## The work

`docs/arc/2026/06/255-builtin-registry/dot-flip-phase1-pairs.txt` holds **382** confirmed renames,
one per line, space-separated, no quotes:

```
:arena::Method::GET :arena::Method.GET
```

Each was produced by ASKING `:wat::runtime::variant-parent-of` with the declaring file loaded — not
by matching a shape. A capitalised-leaf grep predicts 1583 and is wrong in both directions: 1203 of
its predictions are surface methods and `defrecord` names (`Cache::Op`, `Admin::AllowPeer`) that
must NOT move, and it misses `PeerKind::thread`/`::process` entirely. **Do not regenerate, filter,
extend or "sanity-check" this list against a pattern.** It is the census; a pattern disagreeing
with it is the pattern being wrong.

Write `wat-scripts/fixes/variant-separator-to-dot.wat`, modelled on
`wat-scripts/fixes/bare-variant-to-qualified.wat` (arc 296 N — the exact INVERSE migration, and the
proof that `rename-keyword-exact` is the right primitive here).

## The shape, and why it is this shape

```
read the pair list once
for each input path:
    text := read-file path
    hits := pairs whose OLD token appears in text        (:wat::string::contains)
    if hits is empty -> skip, do not rewrite, do not write
    else fold rename-keyword-exact over hits, then write-file
```

The prefilter is load-bearing for runtime, measured: **median 2 matching pairs per file, mean 3.5,
max 17**. Without it every file pays 382 parses (`rename-keyword-exact` calls `read-string`
internally); with it, ~3.

## Two properties to VERIFY rather than assume

**Idempotence.** A second run must be a no-op. After the rewrite the old token is gone, so it
matches nothing — the same argument `bare-variant-to-qualified.wat` documents. Prove it: run twice
on the `/tmp` copy and show the second run's diff is empty.

**No cascade.** No pair's NEW token may be another pair's OLD token, or order would matter. New
tokens carry `.` at the variant boundary and old tokens never do (③b-i forbids a dotted leaf in a
declared name). Prove it with a set intersection over the 382 pairs and report the result.

## The dry run

```
copy the whole corpus to a /tmp dir (wat/, wat-tests/, wat-scripts/ — 845 .wat files)
run the codemod over EVERY path in the copy
diff -ru the copy against the original; keep the diff
run it a SECOND time on the same copy; that diff must be empty
```

Report: files changed, total lines changed, the full list of distinct `::`→`.` transitions the diff
actually performed, and a sample of ~10 hunks spanning different shapes (a match arm, a
constructor, a type position, a lowercase-leaf variant, a macro-generated `Response` variant).

## STOP triggers — each REJECTS the stone; ship nothing further and report

**STOP-1.** The diff changes ANYTHING other than a `::` → `.` at a variant boundary. Not
whitespace, not formatting, not a comment's text, not a `defenum` declaration slot. `fix.wat`'s own
header records arc 296 N RELAND 8, where a whole-file keyword rename corrupted `Option`'s unit
variant by treating a declaration slot as a use site. Report the hunk verbatim.

**STOP-2.** A file fails to parse during the rewrite. `rename-keyword-exact` calls `read-string`;
every tracked file parses today (`every_tracked_wat_parses`), so a failure means the rewrite
produced something unparseable — report the file and the error.

**STOP-3.** The second run's diff is non-empty. Idempotence is a property of this migration, not a
hope; a non-empty second diff means a cascade or a boundary bug.

**STOP-4.** A token in the pair list appears in the corpus but is NOT rewritten, or a token outside
the pair list IS rewritten. Either direction refutes the census or the tool.

## Blast radius

`wat-scripts/fixes/variant-separator-to-dot.wat` (new) and a `/tmp` copy. **No file under `wat/`,
`wat-tests/`, `wat-scripts/` (except your new fix), and nothing under `src/`.** Do not touch
`compose_variant`/`decompose_variant` — the flip is the orchestrator's act.

## What to run

Your codemod against the `/tmp` copy, and `diff`. You may `cargo build --release` once if a build
is needed to pick up the new fix file. **Do not run `scripts/floor.sh`, do not run clippy.** Run
every command in the FOREGROUND and block on it. Do not commit. Do not contact any peer.
