# BRIEF — STONE 118.B4-ii · `(first (drop X n))` → `(nth X n)`, self-hosted

44 sites across 13 files spell positional lookup as `(first (drop X n))`. B4-i (`c90647d4`) widened
`nth` so it now covers every receiver that idiom reaches. B4-iii closes `first`-on-Stream, which makes
the idiom illegal. This strike migrates the corpus, in between, while both spellings still work.

**R21: this is a wat-fix codemod — wat rewriting wat. Not hand-edits, not sed, not python.**

## The worklist is a census, not a grep

`wat-scripts/scratch-pad/census-first-of-drop.wat` walks the real form tree (`read-string` →
`ast->children`, structural head match) and returned **44 hits, 13 files, 0 malformed** at
`c90647d4`. Re-run it yourself first — it is your worklist and your acceptance gate.

```
wat/service.wat                                          10
wat/lint.wat                                              6
wat/fix.wat                                               5   ← the codemod rewrites ITSELF
wat-scripts/probes/arc-170/probe-m1-argcount.wat          5
wat/bracket.wat                                           4
wat-scripts/probes/arc-170/probe-s3b-extract.wat          3
wat-scripts/probes/arc-170/probe-s3b-astsplice.wat        3
wat-scripts/probes/arc-170/probe-c1-plain-fnforms-shape.wat 3
wat/deporder.wat                                          1
wat-scripts/probes/arc-170/probe-m1-dump-forms.wat        1
wat-scripts/fixes/drop-deftest-prelude.wat                1
wat-scripts/scratch-pad/census-parametric-surface-bindings.wat 1
wat-scripts/scratch-pad/census-defclause-arm-overlap.wat  1
```

Index operands across all 44: `1`×16, `2`×16, `3`×8, `4`×3, and one non-literal (`idx`).

## Read in order

1. **`wat/fix.wat:1–52`** — the framework header. It contains the STASH-DANCE note. **You do not need
   the stash dance**: it applies only when a codemod ships alongside a `src/` change that makes the
   old form illegal. There is no Rust change in this strike. The header says so in its last
   paragraph — one `cargo build --release` to pick up your new verb, then run.
2. **`wat-scripts/fixes/strip-expect-ascription.wat`** (40 lines) — the shape to copy: a thin
   `:user::migrate` over a generic `:wat::fix::` helper, plus `apply-each` + `main` boilerplate.
3. **`wat/fix.wat:148–360`** — the `fix-text-*` machinery: `ast-span`/`ast-end-span` → flat offsets →
   right-to-left splices via `fix-text-apply`. This is how an edit stays span-faithful, so comments
   and whitespace inside the operands survive untouched.
4. **`wat/core.wat:1393–1430`** — `nth` as B4-i left it: four arms, and the header's total-CONTRACT /
   partial-FUNCTION argument.
5. **`src/stdlib.rs:35–40`** and the `path: "wat/` list — the load order. **Grounded, not assumed:**
   `wat/core.wat` is **#1**; every stdlib file on the worklist is #22 or later (`bracket` 22, `fix`
   29, `deporder` 32, `lint` 33, `service` 34). `nth` is defined before any of them load.

## The rewrite

```wat
(:wat::core::first (:wat::core::drop  X  n ))     →     (:wat::core::nth  X  n )
```

`X` and `n` carry across as their **original source text**, byte for byte. The edit is structural:
the outer head becomes `nth`, the inner `(`+`drop` head goes away, and one closing paren goes away.
Nothing inside `X` may move.

## The strike path, in this order

```
1.  add the generic helper to wat/fix.wat + write wat-scripts/fixes/first-of-drop-to-nth.wat
2.  cargo build --release                      # the binary can now SEE your new verb
3.  DRY RUN on a /tmp copy of all 13 files, then `diff` — verify before you touch the real tree
4.  apply:  printf '["wat/service.wat" …all 13…]\n' | ./target/release/wat ./wat-scripts/fixes/first-of-drop-to-nth.wat
5.  cargo build --release                      # 5 stdlib files changed; rebake them into the binary
6.  re-run the census                          # expect ZERO
7.  run the codemod a SECOND time              # expect no diff — idempotent
8.  floor + clippy
```

Step 5 is not optional and is easy to skip: `wat/*.wat` is frozen into the binary at build time, so
until you rebuild, the binary is still running the pre-codemod stdlib.

## ★ `wat/fix.wat` rewrites itself — that is the proving point, not a problem

Five of the 44 sites are in the codemod framework. `fix.wat`'s own header calls this out: *"when the
corpus drive runs, fix-source fixes ITSELF (homoiconic self-application)."* The file is already
loaded in memory when the rewrite runs, so rewriting it mid-run is safe; step 5 is what makes the
new text real.

## Blast radius

The 13 census files, `wat/fix.wat` (the new helper), and one new `wat-scripts/fixes/*.wat`. **No
`src/` edits. No test edits.** If a test needs changing, that is a finding, not a task — see STOP-3.

## STOP triggers — each is "ship nothing further, report the gap"

**STOP-1 — a site whose result feeds a nil test.** The rewrite is **not semantically neutral at the
edges**. Measured at HEAD: out of range, `(nth v 7)` raises `"nth: index out of range"` (rc=2) while
`(first (drop v 7))` returns **`nil` silently** (rc=0). Closing that is the point. But if any of the
44 sites has surrounding code that tests the result for nil, branches on it, or passes it somewhere
nil-tolerant, that site changes behaviour — STOP, name the file and line, and report the surrounding
form verbatim. Do not migrate it on the assumption it is fine.

**STOP-2 — the dry-run diff shows anything but the intended structural change.** Whitespace inside
`X`, a moved comment, a touched line that has no hit on it: STOP and show the diff hunk.

**STOP-3 — the floor goes red anywhere outside the census files.** Copy the failing test's whole
block from `.floor/latest/clean.log` verbatim, name the assertion, stop. Do not edit a test to make
it pass.

**STOP-4 — the codemod is not idempotent.** If run 7 produces any diff, STOP and show it. A codemod
that is not a fixed point cannot be trusted as the recorded migration.

## A judgement call to REPORT, not decide

`wat-scripts/fixes/drop-deftest-prelude.wat` is itself a **recorded migration** from an earlier arc,
and it holds one site. It is live, loader-gated, type-checked code, so the codemod will rewrite it.
Migrate it, and **say in your report that you did** — an earlier arc's recorded migration having its
text changed is the orchestrator's call to ratify, not yours to make silently.

## Verification

Run everything in the FOREGROUND and block on it — your turn ends when the numbers are in your hands.

```
systemd-run --user --scope -q -p MemoryMax=12G -p MemorySwapMax=0 timeout 1500 scripts/floor.sh
cargo clippy --release --all-targets -- -D warnings
grep -rn '^\s*#\[ignore' --include=*.rs tests/ src/ crates/ | wc -l     # expect 13
```

Read the floor's **Summary line** out of `.floor/latest/clean.log`, never a piped exit code. On any
red: do not re-run — the re-run destroys the evidence.

## Prior result to copy for shape

`wat-scripts/fixes/strip-expect-ascription.wat` (the thin-wrapper form) and
`wat-scripts/fixes/rename-locidiederror-shutdown-to-stopped.wat` (a 16-site migration recorded and
committed as its own artifact — task #25's shape).
