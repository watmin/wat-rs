# EXPECTATIONS — #88, the rete `defn`

Written **before** the strike, per examinare, so the result cannot move the goalposts. Every row is
scored by the orchestrator's own re-run, never the rider's report.

## Baseline, measured this session

```
HEAD 79edf2c7   floor 4373 passed / 0 failed   clippy 0   tree clean
```

## The scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the form exists | `./target/release/wat --check <fixture with a rete-defn>` | exit 0 — where today it is 2 × `MalformedForm` on the signature |
| 2 | **the membrane bites** | the new `tests/rete/` gate | RED before the body is legal, GREEN after — proven **both** directions by mutation |
| 3 | **the error names the HELPER** | read the gate's diagnostic | the located error carries the helper's FQDN. Today's failure names the *rule* and no frame names the helper — that inversion IS the stone |
| 4 | the four axes are REUSED | `grep -c 'fn is_pure_expr\|fn is_total_expr' src/rete/purity.rs` | unchanged — one implementation each, called from a second phase (STOP-1) |
| 5 | `RETE_OPS` untouched | `git diff --stat src/rete/vocabulary.rs` | no change to the table (STOP-3) |
| 6 | the cascade is mechanical | `git diff` on the 35 `Function { … }` sites | every one a bare `rete: None`; no logic moved |
| 7 | the migration is a RE-HEADING | `git diff wat/ wat-tests/ wat-scripts/` | **heads only.** Any body change is STOP-4 and a finding |
| 8 | the codemod is idempotent | re-run it on the migrated corpus | 0 changes (doctrine: a recorded migration must be re-runnable) |
| 9 | the codemod was dry-run | the `/tmp` copy + `diff` exists before the corpus moved | the diff is exactly the intended structural change |
| 10 | the stdlib still loads | `:wat::deporder::verify-stdlib` | prints `[]` |
| 11 | **the floor** | `scripts/floor.sh` → read the **Summary line** | ≥ 4373 passed, **0 failed**. A red is a red |
| 12 | clippy | `cargo clippy --release --all-targets` | 0 warnings |
| 13 | the count is the CHECKER's | the rider reports the screaming-site count | whatever the checker says. 27 is the stone's estimate, not a target |

Rows **2, 3, 7, 11** are load-bearing. Row 3 is the one that would be easiest to ship broken while
everything else goes green — a membrane that admits correctly but still blames the rule has fixed
nothing a user would notice.

## Runtime prediction

**35–55 min** for a sonnet rider, Mode A. Basis: the form + the marker + the one branch is a small,
well-mapped edit; the 35-site cascade is compiler-named and fast; the codemod and its dry-run are the
long pole. The stash-dance adds two release builds (~40 s each).

Time-box at **2×** = 110 min (`ScheduleWakeup`). Overrun is itself data — most likely it means STOP-4
fired and the "already clean" premise has a hole.

## Trap doors, named in advance

1. **The chicken-and-egg build.** The checker change makes the old corpus illegal, so the binary that
   must run the codemod cannot be built from the new tree. `wat/fix.wat:23-53` is the supported path.
   A prior self abandoned the tool here and hand-edited; that is the failure to avoid.
2. **A blind prefix rename.** Only the rete callees move. A codemod keyed on `:wat::core::defn` alone
   would re-head the entire corpus. It must be driven by an explicit name list.
3. **The admitted-namespace interaction (STOP-2).** `":wat::rete::core::"` is already in
   `RETE_MODULES`. Believed benign; **must be confirmed by a run, not by reading.**
4. **`--check` is not a complete arbiter.** An unknown callee defers to a runtime `UnknownFunction`.
   For row 2 the arbiter is the test RUN.
5. **A green `cargo build --release` over a red corpus.** The bake does not run the whole sweep. Row
   11 is the arbiter; the build is not.
6. **The gate that cannot go red.** If row 2's mutation does not flip it, the gate proves nothing —
   and this arc has shipped eleven such gates before (`91bbb8cd`).

## What would make me call this a Mode B

Any of: the body check re-implements an axis (STOP-1); a site needs a body change and it is quietly
made (STOP-4); the codemod is not idempotent; the count comes from a grep rather than the checker; or
the floor is reported green from a piped exit code rather than the Summary line.
