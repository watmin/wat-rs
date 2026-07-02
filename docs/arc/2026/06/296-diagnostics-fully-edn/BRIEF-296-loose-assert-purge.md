# BRIEF — 296 close-gate: purge the loose-assert heresy (the shared campaign rule)

> **Executor: one sonnet per CLUSTER, MAIN tree** (the `../holon-rs` path dep breaks worktree builds).
> Orchestrator weighs each cluster by re-running the lint (the count must drop) + auditing every rune. **Commit
> nothing.** Anchor `/home/watmin/work/holon/wat-rs`; `pwd` first; reject `.claude/worktrees/`. Do NOT spawn subagents.
> The meter is `tests/lint/no_loose_string_assert.rs` (`cargo nextest run --release -E 'test(tests_carry_no_loose_string_assert)'`).

## The work (one paragraph)
The loose-assert lint flags 784 sites where a test asserts with `contains`/`starts_with`/`ends_with` (including negated
`!contains`) instead of an exact match. Your cluster is a subset (a file list, below). For EACH flagged site, do ONE of
exactly two things — **TIGHTEN** or **RUNE** — and drive your cluster's sites to zero on the lint.

## The classification (per site — flat, no third option)
1. **TIGHTEN** — the asserted value is DETERMINISTIC (fully fixed by the test's own inputs — a serialized EDN, a
   rendered string, a fixed output). Replace the loose check with an exact `assert_eq!(value, "<the whole thing>")`.
   - **Capture, do NOT guess** the expected string: run the test (or drop a temporary `eprintln!("{value}")`, run,
     copy the exact bytes, remove the eprintln). NEVER hand-type the expected value.
   - **Collapse N field-checks into ONE golden.** Several `assert!(s.contains(:field …))` on ONE deterministic output →
     a single `assert_eq!(s, "<the exact full output>")` (stronger + fewer lines). (e.g. `discover.rs`'s 6 EDN-field
     `contains` → one `assert_eq!(env, "#wat.test/DiscoveryFailed {…}")`.)
   - Use raw strings `r#"…"#` for goldens with quotes (mirror `probe_arc298_3_runtime_derive_identical.rs`).
2. **RUNE** — the value is LEGITIMATELY loose. Add a per-site `// rune:lint(loose-assert) — <reason>` on (or
   immediately above) the assert. ONLY these earn a rune:
   - **Variable value:** the output embeds something that varies per run — a PID, an OS thread id, a socket/inode
     number, a temp path, a timestamp, a hash, an address, a duration. (Say WHICH in the reason.)
   - **Property over a variable set:** `assert!(coll.iter().all(|e| e.starts_with("wat/")))` — the set/order varies;
     the property is the contract.
   - **Targeted absence on a large/variable output:** `assert!(!big.contains("bad"))` where asserting the whole `big`
     exactly is infeasible (it varies) — the absence is the real contract.

## ⛔ THE ANTI-HERESY-RUNE RULE (non-negotiable)
A rune is NOT a way to dodge the tightening work. The reason must NAME the specific variability — the orchestrator
casts **excusare** on every rune and STRIKES a reason that does not earn its standing. These are HERESY, not rune-worthy:
- *"the output is complex / long"* → if it's deterministic, TIGHTEN it (length is not variability).
- *"checking one field is clearer"* → collapse to ONE golden; the whole output is the contract.
- *"assert_eq! is brittle"* → that is the POINT; a deterministic value's exactness is the guarantee.
If you cannot name what VARIES, it is deterministic → TIGHTEN. When in doubt, TIGHTEN.

## The flaky guard (capture discipline)
If ANY part of a value varies per run, it is RUNE, never TIGHTEN — a flaky `assert_eq!` is worse than a loose
`contains`. When you tighten, RUN THE TEST TWICE and confirm the captured value is byte-stable before committing to the
`assert_eq!`. If it differs between runs, that variability is your rune reason.

## Never the reverse
NEVER weaken (assert_eq! → contains), invert an assertion, `#[ignore]` a test, or delete a test to clear a site. If a
tightened test goes red, the CODE is wrong or the value varies (→ rune) — STOP and report, do not soften.

## STOP triggers (REJECTION criteria)
- **STOP-1:** a site the lint flags that is NOT actually an assertion (a lint false-positive — control flow / a string
  literal that merely contains `.contains(`) → report it with the line; do NOT contort the test. (The orchestrator
  refines the lint, not you.)
- **STOP-2:** a tightened value that will not stabilize across runs and is not cleanly rune-able → report it.

## Proof (per cluster)
- Your cluster's sites resolved (each TIGHTEN or RUNE). Re-run the lint — your cluster's file:line entries are GONE from
  the offender list (the count dropped by your cluster size).
- The tests you touched stay GREEN (run them; run any tightened test twice for stability).
- FULL gate for your files: `cargo nextest run --release` on the touched test binaries = 0 failed.

## Report back
Per cluster: the count (N tightened, M runed), the tightened list (paste 2-3 sample goldens you captured), the FULL rune
list with each reason (so the orchestrator can cast excusare), any STOP, any lint false-positive, any deviation.
