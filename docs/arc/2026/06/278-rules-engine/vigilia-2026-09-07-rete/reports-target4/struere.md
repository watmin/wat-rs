# STRUERE — Cast Report (TARGET 4 — the test corpus) — **FINDINGS**

> Returned verbatim, before synthesis. Orchestrator verification follows the ward's text.

Sweep complete. I read the 236 non-test helper functions I could positively identify across the target (860 total `fn` minus a test-attribute-lookback heuristic; measured trigger says 247 — my count is 236, an 11-function delta likely from doc-commented `#[test]` blocks my lookback window missed; noted, not material to the findings below), plus spot-read ~40 test bodies calling them, plus the two named production files (`fire/mod.rs`, `kernel/arm.rs`, `kernel/stratify.rs`) only to check whether a helper duplicates or contradicts production logic.

## Findings

### Finding 1 — sharpest: a typed struct exists in the same file tree, same day, same bug fix, and a sibling function didn't reach for it

- **File:** `src/rete/kernel/tests/node_share_cost.rs:829` — `fn node_share_filter_counts(n: i64, m: i64) -> (u64, u64, u64)`
- **Compare:** `src/rete/kernel/tests/where_tree_branch_differential.rs:78-86` — `struct Dispatch { evals: u64, reuse: u64, pass: u64 }` with each field doc'd to its census key.
- **Lens:** type-doesn't-enforce (struere's own worked example: "comment promises X, type doesn't enforce it")
- **Level:** 1 (lie) — two of the three positions (`evals`, `pass`... actually all three) are the identical `u64`, so a transposition compiles silently
- **Evidence it was avoidable, not just missed:** both files were touched by the **same commit** (`d878408f7`, 2026-09-05, *"the census union — three gates were asserting arithmetic identities"*), fixing the exact same `filter:test-evals`/`filter:test-reuse`/`filter:test-pass` semantics in both files. `where_tree_branch_differential.rs` (created the day before, `5f0b2f1b1`) already carried the named struct; `node_share_cost.rs`'s sibling function, edited in the very same commit for the very same bug, still returns a bare `(u64, u64, u64)`. The call site (`node_share_cost.rs:~870`) destructures as `let (evals, reuse, passes) = node_share_filter_counts(n, m);` — correct today only by convention.
- **Direction:** type-tightening — return `Dispatch` (or a shared alias) from `node_share_filter_counts`, or promote `Dispatch` out of `where_tree_branch_differential` into a spot both modules can `use`.

### Finding 2 — recurring class: a subprocess helper discards an already-typed `std::process::Output` for a positional tuple

- **Representative site:** `tests/rete/probe_arc278_enum_variant_typo.rs:19-30`:
  ```rust
  fn run(rel: &str) -> (bool, String, String) {
      ...
      let out = Command::new(bin)...output()...;
      (out.status.success(), String::from_utf8_lossy(&out.stdout).into_owned(), String::from_utf8_lossy(&out.stderr).into_owned())
  }
  ```
  `out: std::process::Output` already carries named fields (`status`, `stdout`, `stderr`); the function actively unpacks that typed value into an unlabeled 3-tuple with two fields of identical type (`String`, `String`).
- **Recurrence:** the identical function (byte-identical body) is defined independently in 9 files: `probe_arc278_D6_constraint_omission.rs:51`, `probe_arc278_D10_then_field_types.rs:52`, `probe_arc278_D11_nested_then_field_types.rs:57`, `probe_arc278_enum_variant_typo.rs:19`, `probe_arc278_field_span.rs:45`, `probe_arc278_fixpoint_round_cap.rs:45`, `probe_arc278_import_accounting.rs:46`, `probe_arc278_match_arm_is_not_a_call.rs:50`, `probe_arc278_nested_wall.rs:49`. Same shape with extra parameters: `wat_scripts_grid_axes_live.rs:274` (`run_sized_axis`), `:343` (`run_where_axis`), `:566` (`run_wat_path`), `wat_scripts_grid_port_check.rs:265` (`run_axis`); one narrower variant `probe_arc278_then_operand_rendered_as_source.rs:50` returns `(bool, String)`.
- **Lens:** type-doesn't-enforce. **Level:** 2 (mumble) — I checked every call site (grep above, ~55 destructuring sites) and every one is ordered `(ok, out, err)` / `(ok, stdout, stderr)` matching declaration order; no live swap exists today, so the risk is latent, not manifest.
- **This is distinct from 4S1** (which is a solvere finding about the missing shared *fire-driver* builder, scoped to `src/rete/`'s in-process format-string fire pattern). This is a different family — subprocess-spawn probes in `tests/rete/`, and the defect here is about the return *type*, not about the missing abstraction per se.
- **Direction:** type-tightening (a small `ProcOutput { ok: bool, stdout: String, stderr: String }`, or just keep `std::process::Output` and add `.stdout_lossy()`/`.stderr_lossy()` accessors) — would also fold the 9-way duplication, but that's secondary to struere's concern here.

## Checked and sound (verified negatives)

- **`src/rete/kernel/tests/where_tree_branch_differential.rs`** (the file with the most prior-art citations against it): I read `dispatch_of`, `derived_multiset`, `multiset_symdiff`, `show`, `reference_arm`, `fire_both_branches`, `differential`, and `Measured::exercises_the_branch_pair` in full (lines 88–270). All take values, return values, no hidden mutation, and the doc comments state exactly what a caller needs (including an explicit structural proof, not just an assertion, that the empty-tree swap took). No struere findings here beyond what's already rowed under 3R1/4R1.
- **`src/rete/kernel/tests/stratify_numbers.rs:82` `views_of`**: its doc says "Byte-copy of `fire_rules_on_session`'s view construction" — I checked whether this hides a duplication risk struere would flag as a lie. It does not: the file's module doc explicitly denies a parallel *extractor* ("Views are built the way fire/rules.rs builds them (the four extractors). No parallel view-builder") and the four extraction functions (`rule_produces`, `rule_negates`, `rule_consumes`, `rule_bag_consumes`) are imported directly from production `kernel::stratify`, not reimplemented — only the wiring loop is local, and that's disclosed. Honest, not a lie.
- **`poke`/`poke_named`/`field_of`/`import_one`/`poke_first_call_op`/`poke_first_prog_root`/`tamper_first_prog_root`** (`probe_arc278_export.rs`, `probe_arc278_import_fold_key.rs`, `probe_arc278_import_accounting.rs`): all take `Value` by value or `&mut Value` explicitly and return a new `Value` — values-not-places is honored throughout; the `&mut` sites name the mutation in their signature (struere's explicit non-flag case).
- **`collect_wat(dir, out: &mut Vec<PathBuf>)`** (`probe_arc278_compiled_where_ops.rs:237`): an out-parameter recursive walker — mutation is visible in the signature, matches the spell's stated non-flag for `&mut` params.
- **`classify`/`is_true`/`is_false`/`eval_i64`-style one-line boolean/i64 unwrap helpers** (5+ files: `accessor_purity.rs`, `fence_hof.rs`, `ast_to_source.rs`, `foreign_pred_purity.rs`, `sieve_pred.rs`, `probe_arc278_59...`, `probe_arc278_6a_purity.rs`, `probe_arc278_55...`): trivial, file-local, doc'd at each call site — no lens flags these; they're honest about what they do.
- **`AtomicU64`/`Arc<`/`thread::spawn` sites** (anchored against the measured triggers: `harvest_cost.rs`, `arm_lease.rs`, `probe_arc278_concurrent_retes.rs`, `accum_cost.rs`, `accum_alpha_cost.rs`, `alpha_discrimination.rs`): all are function-local counters/handles or ordinary `Arc<T>` cheap-clone value types, not shared mutable state smuggled past a signature. `probe_arc278_concurrent_retes.rs`'s `witnesses`/`expected` helpers are honest value-in/value-out functions; the concurrency correctness itself is asserted via returned `Vec<i64>` comparison, not via a global read.
- **`blamed_identity` / `prefix_holds` / `assert_recursive`** (`probe_arc278_rete_defn_recurse.rs`, `right_index_counter_invariant.rs`): heterogeneous-typed tuple returns (`(String, u32)`) or well-named `bool` returns whose doc comment states exactly the property tested — no conflation, no swap risk (different types close that door).

## Self-correction mid-cast

My awk-based fn/attribute-lookback extractor found 236 non-test helpers against the handed-down 247 — I did not spend budget forcing exact reconciliation since none of the findings above depend on the total count, but I flag the 11-function delta rather than silently adopting 247. I did not hit the `probe_arc278_export.rs` rune-between-`#[test]`-and-`fn` extractor bug two other wards reported — my lookback window tolerates a `//` line in between — but I can't rule out it swallowed a small number of other cases elsewhere, which is my best explanation for the delta.

FINDINGS

---

# ⚖ ORCHESTRATOR VERIFICATION — everything below is my own read of the disk, this session

## Finding 1 — ✅ VERIFIED, and it is STRONGER than the ward stated

Both citations are exact, symbol first and then address:

- `src/rete/kernel/tests/node_share_cost.rs:829` — `fn node_share_filter_counts(n: i64, m: i64) -> (u64, u64, u64)`. Call sites at `:876` and `:910`, both `let (evals, reuse, passes) = …`.
- `src/rete/kernel/tests/where_tree_branch_differential.rs:78-86` — `struct Dispatch { evals: u64, reuse: u64, pass: u64 }`, each field doc'd to its census key (`filter:test-evals`, `filter:test-reuse`, `filter:test-pass`). **The same three keys the tuple's own doc comment names.** Both functions extract them with a `get(k)` closure over census rows.

⭐ **The ward understated its own evidence on two points, and I am correcting UPWARD:**

1. **The commit did not "edit" the tuple function — it CREATED it.** `git show d878408f7 -- …/node_share_cost.rs` carries `+fn node_share_filter_counts(n: i64, m: i64) -> (u64, u64, u64)`. The function was **born** positional, in a commit whose own message reads: *"The rider also checked the one other reader (**where_tree_branch_differential**)."* The author states in the commit message that they had the file with the named struct open.
2. **The struct predates it by a day, not by the same day.** `5f0b2f1b1` (2026-09-04) created `where_tree_branch_differential.rs` **and** `Dispatch`; `d878408f7` is 2026-09-05.

So the accurate statement is sharper than "a sibling didn't reach for it": **the named type existed the day before, in the one file the author says they checked, and the new function was written positionally anyway.**

## Finding 2 — ✅ VERIFIED, with one count corrected — AND THE CHECK FOUND A SHARPER DEFECT THE WARD WALKED PAST

The type-shape claim holds exactly: `probe_arc278_enum_variant_typo.rs:19-31` unpacks a `std::process::Output` — which already carries named `status`/`stdout`/`stderr` — into an unlabeled `(bool, String, String)` with two same-typed `String`s.

**The swap-risk verdict is right, and I re-derived it rather than taking it.** All 55 destructuring sites, tallied by shape:

```
35  let (ok, out, err)          7  let (ok, stdout, stderr)     2  let (ok_ok, out_ok, err_ok)
 2  let (ok_d, out_d, err_d)    1 each: when_/then_/s_/ok_t/ok_pass/ok_fail/ok_deep/ok_big/n_/bare_
```

**Zero swaps.** Level 2 (latent, not manifest) is the correct call.

⛔ **BUT THE "BYTE-IDENTICAL IN 9 FILES" CLAIM IS FALSE, AND THE WAY IT IS FALSE IS THE FINDING.** I md5'd the 15-line block at each of the nine definitions. They fall into **two** groups, not one — 7 + 2 — and `diff` shows the two are not a cosmetic variant:

| group | files | how the subprocess is located |
|---|---|---|
| **7 copies** | `enum_variant_typo`, `D6`, `D10`, `D11`, `field_span`, `nested_wall`, `match_arm_is_not_a_call` | `.arg(rel)` **+ `.current_dir(manifest)`** — a RELATIVE path resolved against an explicitly-set cwd |
| **2 copies** | `import_accounting:46`, `fixpoint_round_cap:45` | `.arg(&path)` where `path = manifest.join(rel)` — an ABSOLUTE path, and **no `current_dir` call at all** |

**Nine helpers share a name, a signature and a calling convention, and two of them silently differ in working-directory semantics.** A caller reading `run("tests/rete/foo.wat")` in any of the nine cannot tell which behaviour they get, and whether that matters depends on how the spawned `wat` binary resolves anything relative to cwd.

⭐ **This is `4S2`'s comment made real in a second family** — *"two copies is how one of them silently stops subtracting"* — except here it is not the arithmetic that diverged, it is the process environment, and the divergence is already present. The ward reported the **return type** and asserted byte-identity of the body; the body is where the live divergence sits. **I found this only because I checked a claim the ward stated as settled.**

## Instrument note — the ward's 236-vs-247 delta, disclosed rather than absorbed

The ward reported 236 non-test helpers against my handed-down 247 and **flagged the gap rather than adopting my number** — which is the behaviour the brief asked for. Neither figure is load-bearing for either finding, and neither is re-derived here; both are recorded as a range, not a fact.
