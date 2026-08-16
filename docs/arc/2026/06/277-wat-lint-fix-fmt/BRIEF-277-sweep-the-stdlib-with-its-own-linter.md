# BRIEF — 277 · point the linter at the stdlib; the sweep already exists

**You are a rider, not the orchestrator. Ending your turn ENDS you** — nothing will wake you. Run
every verification in the **FOREGROUND** and block on it: your turn ends when the numbers are in your
hands, not when a command is launched.

Work in `/home/watmin/work/holon/wat-rs/`. **Do not commit, push, stash, or revert** — leave the work
in the tree. Any path containing `.claude/worktrees/` is harness state; re-anchor if you see one.

## There is NO new codemod to write. That is the whole finding.

`wat-scripts/fixes/sweep-lint-fixes.wat` exists, is documented, is idempotent, and has **never been
run against the corpus**. `:wat::lint::lint-fix-file` (= `lint-file` → `apply-fixes`,
`wat/lint.wat:680`) exists. The linter is finished enough to fix its own source and nobody pointed it
at the source. Same class as `insert-all` (278/UNADOPTED.md): capability built, never adopted.

Measured by the orchestrator this session against a `/tmp` copy — **already dry-run, already diffed**:

```
(:wat::lint::lint-stdlib)  ->  136 findings, ALL rule "concat-abuse", ALL severity "warn"
                               across 10 files; 0 findings of any other rule, 0 non-warn
sweep on a /tmp copy       ->  7 files changed, 52 lines
                               bracket 9 · core 21 · query 4 · Record 4 · rete 4 · service 8 · span 2
                               UNCHANGED: lint.wat, test.wat, telemetry/journal.wat (fix = None,
                               report-only — compound concat is a judgment the sweep declines)
```

The rewrite, verbatim from that diff:

```diff
-  (:wat::core::string::concat "kwargs-lower: missing argument :" fkebab)
+  (:wat::core::string::interpolate "kwargs-lower: missing argument :{fkebab}" :fkebab fkebab)
```

## The work

Apply it to the real tree and prove it did no harm.

```bash
printf '%s' "$(git ls-files 'wat/*.wat' 'wat/**/*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/')" \
  | ./target/release/wat ./wat-scripts/fixes/sweep-lint-fixes.wat
```

Derive the path set from `git ls-files` — do NOT hand-type it (arc 283's lesson, and the script's own
header says so). Then:

1. **Read the whole diff.** 52 lines. Every hunk must be a `concat` → `interpolate`/`format` rewrite
   and nothing else. A hunk that touches anything other than a string-building call is STOP-2.
2. **`./target/release/wat --check`** on any corpus file — this loads the whole stdlib, so a broken
   rewrite anywhere surfaces immediately. Cheapest possible smoke test; run it before the floor.
3. **`scripts/floor.sh`** — read the Summary line.
4. **`cargo clippy --release --all-targets`**.
5. **Re-lint** and report the new count: the number must DROP, and the residue must be exactly the
   report-only findings.
6. **Re-run the sweep a second time** — it must report ZERO `[fixed]` lines. That is its idempotence
   claim, and it has never been tested against this corpus.

## Rooms

- `wat-scripts/fixes/sweep-lint-fixes.wat` — the tool. Read its header first; it documents the
  invocation and what it deliberately declines.
- `wat/lint.wat:648` `apply-fixes`, `:680` `lint-fix-file` — the applier.
- `wat/fix.wat:320` `fix-text-apply` — `Tuple(offset, old-len, new-text)`, right-to-left; this is why
  the rewrite is comment-faithful (it splices the ORIGINAL text and never reformats).

## What you report

- The `[fixed]` list, and the per-file changed-line counts.
- The full diff, or if it is long, every hunk that is NOT a plain concat→interpolate rewrite.
- Floor Summary line verbatim; clippy count.
- Lint count before (136) and after, with the residue's rules named.
- The second sweep's output (must be empty).

## STOP triggers — rejection criteria. Ship nothing and report.

- **STOP-1 — the floor goes red.** Do NOT re-run. `scripts/floor.sh` keeps the untruncated log at
  `.floor/latest/ARM.txt`; copy the failing test's WHOLE stdout+stderr block verbatim — never a
  summary, never a `| head` window — and name the exact assertion that fired. A red is a red.
- **STOP-2 — a diff hunk changes something that is not a string-building call.** Name the file and
  line and stop. This codemod rewrites `concat`; anything else is a bug in it.
- **STOP-3 — the second sweep reports any `[fixed]`.** That falsifies idempotence, which is the
  property that makes this safe. Report which file.
- **STOP-4 — `wat --check` fails after the sweep.** The stdlib is broken; capture the error verbatim.
  Do not attempt to hand-repair a `.wat` — that is what the codemod doctrine forbids.

## One judgment call that is NOT yours

`wat/lint.wat` is itself among the linted files. If the sweep changes the linter's own source, say so
prominently — the toolchain editing itself is correct in principle and worth a human eye in fact.
Report it; do not treat it as special otherwise.
