# BRIEF — make `.wat.bad` mean something, and rename the 16 files that do not mean it

`.wat.bad` claims a file fails to start up. Nothing checks that claim, and 16 of 281 files carrying
it start up clean. Build the gate, then rename the 16 — their tests are correct, their filenames are
not.

## Read in order

1. `src/freeze.rs:938-947` — **why the first draft of this strike was wrong.** The `:user::main`
   check is guarded on a main being *declared*: `startup_bare()` (no main) passes cleanly. The
   binary requires a main because it EVALS one; `startup_from_file` does not.
2. `probe-c18.rs.txt` beside this brief — the orchestrator's classifier, carrying its own
   wrong-driver lesson. Run it first as `examples/zz_c18_measure.rs`, then delete the example.
3. `tests/types/probe_arc237_8a_no_implicit_coercion.rs:112-123` — kind 1: the test asserts
   **`is_ok()`** and says why (*"arc 300 C4 retired 237.8a's reject"*). The fixture is a POSITIVE
   fixture wearing `.wat.bad`. Five siblings share this (`8a`×3, `8b`×2, `8c`, `8d`).
4. `tests/function/probe_diagnostic_dynamic_keyword_invocation.rs:138-154` — kind 2: it starts the
   world up and then INVOKES, asserting the error comes *at eval*. The builder's pattern exactly.
   The file is a valid program; the extension is wrong.
5. `tests/lint/every_walking_gate_declares_non_vacuity.rs` — house style for a discovered gate that
   requires a declared guard or a rune. Copy its shape.
6. `tests/lint/diagnostic_output_is_deterministic.rs` — the only existing reader of this corpus, and
   it reads for byte-stability, not failure. **Do not duplicate its discovery; check whether it
   should share yours.**

## Driven by the orchestrator at HEAD `beb0c9554`

Through `startup_from_file`, all 281 `.wat.bad`:

```
failed for their own reason  263
MainSignature                  2   (both wat_arc170_slice_1e_user_main_nil_* — legitimate)
DID NOT FAIL AT ALL           16   ⛔
```

The 16, by path — `probe_arc241_stone10_remedy_c04`, `_c08`, `probe_arc237_8b_regression_cross_lt`,
`_cross_plus`, `probe_arc241_stone5_c05`, `probe_diagnostic_non_keyword`, `probe_diagnostic_non_vector`,
`probe_arc237_8a_no_implicit_coercion_arith_f64_i64`, `_arith_i64_f64`, `_cmp_i64_f64`,
`probe_arc237_8c_equality_grid_cross_numeric`, `probe_arc237_8d_equality_intrinsic_cross_numeric`,
`probe_diag_typealias_leniency_check`, `probe_arc234_stone4_hash_destructure`,
`probe_undefined_builtin_resolves_bogus`, `probe_undefined_builtin_resolves_wrong_leaf`.

## The two pieces

1. **The gate.** Discovered, never listed — walk every `.wat.bad` under `tests/`, `wat-scripts/`,
   `docs/`; call `startup_from_file`; FAIL any that returns `Ok`, naming the path. Population must be
   **281** or STOP-3. A fixture that legitimately starts up clean does not get a rune — it gets
   renamed; the extension is the claim.
2. **Rename the 16 to `.wat`**, updating every test reference. **Check each one first**: read the
   test that drives it and confirm which kind it is (asserts `is_ok()`, or invokes and asserts an
   eval error). If a fixture turns out to be neither — a file nothing drives, or one whose test
   asserts `is_err()` and somehow passes — **STOP-1**; that is a live hole, not a rename.

⚠ Two gates read every `.wat` under `wat-scripts/` recursively (parse+type-check, and rete-name
resolution). A rename from `.wat.bad` to `.wat` **puts a file into those gates' population** — if any
of the 16 lives under `wat-scripts/`, confirm it loads and resolves. (The orchestrator's measurement
says all 16 are under `tests/`; verify rather than inherit.)

## Blast radius

The 16 fixture files (renamed), their `.rs` referrers (path strings), and one new gate under
`tests/lint/`. **No `src/` change.**

## STOP triggers

1. **If any of the 16 is driven by a test asserting `is_err()`**, stop and report — that test should
   be failing and is not, which is a bigger finding than this strike.
2. **If any of the 16 is referenced by no test at all**, stop and report it as a dead fixture rather
   than renaming it — deletion is a different decision and it is the builder's.
3. **If the gate's population is not 281**, stop and report the delta before proceeding.
4. **If renaming requires a `src/` change**, stop and report.

## Mutation proofs — run all three, report all three

1. **Rename one passing fixture back to `.wat.bad`** → the gate REDs, naming it.
2. **Break one renamed fixture so it no longer starts up** → its own test REDs. Proves the rename
   kept the test load-bearing rather than merely quieting a name.
3. **Point the gate at a directory with no `.wat.bad` files** → it must FAIL as vacuous, not pass.
   A discovered gate that finds nothing and reports success is the defect this arc keeps finding.

Verify every restore by **hash** — `git checkout <sha> -- <path>` STAGES, so `git diff` shows nothing.

## What to report

- The gate's output, population count, runtime.
- All three mutation results.
- A line per renamed file: which kind it was, and the test line that proves it.
- Whether `diagnostic_output_is_deterministic.rs` should share the discovery, and why.
- Scoped nextest `Summary` lines including `binary_id(wat::lint)`.
- **Anywhere this brief was thin or wrong. Be blunt.** The first draft of this very strike was
  measured with the wrong driver and had to be rewritten; two strikes before it had their ★ be a
  false claim in a file the brief said to trust. Assume there is one here.

Do not commit.
