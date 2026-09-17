# SCORE — boot names where its time goes

**Struck 2026-09-16** against `DESIGN.md` / `EXPECTATIONS.md`, branch `sns-sqs`, from HEAD
`5479a7f2e`. Instrument: `src/freeze/census.rs`, env-gated on `WAT_BOOT_CENSUS`, **default off**.

⛔ **NOTHING WAS OPTIMISED.** Not a `clone`, not an allocation, not a lookup, not a timeout. The
whole diff is instrumentation, plus one form-aware census program under `wat-scripts/scratch-pad/`.
`3f159b0c5`'s closing line is the rule this obeyed: **a prescription is a claim.** Row 11 is
therefore a recommendation, explicitly not a decision.

---

## ⭐ THE HEADLINE, and two of the DESIGN's three predictions were wrong

| | prediction | measured |
|---|---|---|
| **which phase** | expansion dominates | ⭐ **YES — `stdlib-expand` 201 ms / 49.7 %.** Confirmed. |
| | *(not predicted at all)* | ⛔ **TYPE-CHECKING IS 130 ms / 32 %, and 129.7 ms of that is stdlib** — on a one-line user program. `check_program`'s `forms` argument is the user residue; four of its passes ignore it and sweep the whole symbol table. Nobody had named this. |
| **per-file curve** | lumpy, `defsurface`/`defservice`-bearing files far above average | **Lumpy: YES, brutally** — 6 of 55 files are 71 % of the attributed cost. **Surface-bearing: NO, BACKWARDS.** The 9 surface-*declaring* files run at **0.56× the per-line rate of the other 46**. Four of the top five declare no surface at all. |
| **phases sum to ~430 ms** | ± noise | **YES.** Accounted 404.94 ms; pipeline wall 412.72 ms; since process boot 412.87 ms. Unaccounted in-pipeline **7.38 ms = 1.8 %**. |

---

## Row 1 ⭑⭑ — the per-PHASE breakdown, from the phases the loader ACTUALLY has

The DESIGN's guess (`parse / expand / check / freeze`) is **not** the set. The real pipeline is
`freeze::startup_from_source` → `startup_from_forms` → `startup_from_forms_post_config` →
`freeze::env::build_env`, and `build_env` is where the seams live: a numbered sequence of ~26 named
calls (steps 3a–7.8) whose step numbers are already in its own comments. **38 leaves**, nothing
nested, so the sum is the accounted total and no reading is double-counted.

Median of **10 warm runs**, `WAT_BOOT_CENSUS=phases`, release binary,
`wat-scripts/probes/arc-170/probe-trivial.wat` (one `defn`, one `println`):

| phase | ms | min | max | % accounted |
|---|---:|---:|---:|---:|
| `1    entry-parse` | 0.02 | 0.02 | 0.02 | 0.00 |
| `2    config-pass` | 0.01 | 0.00 | 0.01 | 0.00 |
| `3    resolve-loads` | 0.00 | 0.00 | 0.00 | 0.00 |
| `3a   stdlib-parse` | **29.24** | 28.88 | 32.08 | **7.22** |
| `3b   rete-defn-scan` | 0.00 | 0.00 | 0.01 | 0.00 |
| `4    stdlib-defmacro-register` | 4.36 | 4.23 | 4.65 | 1.08 |
| `4    user-defmacro-register` | 0.00 | 0.00 | 0.00 | 0.00 |
| `4    kwargs-companions` | 0.77 | 0.76 | 0.79 | 0.19 |
| `4    acronym-preregister(macro)` | 0.00 | 0.00 | 0.01 | 0.00 |
| ⭐ `4    stdlib-expand` | **201.37** | 198.82 | 210.50 | **49.73** |
| `4    user-expand` | 0.02 | 0.02 | 0.02 | 0.00 |
| `4b   legacy-walkers(user)` | 0.01 | 0.01 | 0.01 | 0.00 |
| `5    typeenv-with-builtins` | 0.13 | 0.13 | 0.14 | 0.03 |
| `5    stdlib-types-register` | 2.70 | 2.63 | 2.90 | 0.67 |
| `5    user-types-register` | 0.00 | 0.00 | 0.00 | 0.00 |
| `5    aggregate-containment` | 0.23 | 0.22 | 0.27 | 0.06 |
| `6    stdlib-defines-register` | 11.68 | 11.33 | 11.91 | 2.88 |
| `6a   defclause-stub-preregister` | 0.98 | 0.93 | 1.15 | 0.24 |
| `6b   stdlib-runtime-def-filter` | 0.04 | 0.03 | 0.04 | 0.01 |
| `6    user-defines-register` | 0.00 | 0.00 | 0.00 | 0.00 |
| `6a-9 auto-method-codegen` | 4.01 | 3.96 | 4.18 | 0.99 |
| `6.8  restriction-entry-drain` | 0.01 | 0.01 | 0.01 | 0.00 |
| `6.96 acronym-preregister(runtime)` | 0.00 | 0.00 | 0.00 | 0.00 |
| `6.97 typeenv-clone-attach` | 0.47 | 0.45 | 0.56 | 0.12 |
| `7    normalize-symbol-refs` | 0.00 | 0.00 | 0.00 | 0.00 |
| `7    resolve-references` | 0.00 | 0.00 | 0.00 | 0.00 |
| `7.6  stdlib-runtime-defs-register` | 10.11 | 9.93 | 10.58 | 2.50 |
| `7.7  extend-type-preregister(user)` | 0.00 | 0.00 | 0.01 | 0.00 |
| `7.8  freeze-validator-drain` | 0.01 | 0.01 | 0.01 | 0.00 |
| `8a   check:env-from-symbols` | 3.35 | 3.15 | 3.66 | 0.83 |
| `8b   check:retired-syntax(ALL fns)` | 12.02 | 11.78 | 12.77 | 2.97 |
| `8c   check:restricted-call(ALL fns)` | 3.18 | 3.07 | 3.79 | 0.79 |
| `8d   check:def-position(ALL fns)` | 1.85 | 1.70 | 2.34 | 0.46 |
| `8e   check:form-loop(user residue)` | **0.02** | 0.02 | 0.03 | 0.00 |
| ⛔ `8f   check:body-infer(ALL fns)` | **109.28** | 107.34 | 123.33 | **26.99** |
| `8g   check:impls-completeness` | 0.01 | 0.01 | 0.02 | 0.00 |
| `9    frozen-world-freeze` | 4.83 | 4.74 | 6.58 | 1.19 |

Grouped:

| | ms | % |
|---|---:|---:|
| **macro expansion** (4: stdlib-expand + user-expand + defmacro-reg + kwargs + acronyms) | **206.52** | **51.0** |
| **type-checking** (step 8, all seven leaves) | **129.72** | **32.0** |
| **registration** (5 + 6 + 6a–9 + 6.97 + 7.6) | 30.35 | 7.5 |
| **parsing** (1 + 3a) | 29.26 | 7.2 |
| **freeze** (9) | 4.83 | 1.2 |

### ⛔ Two seams that are missing, and both are findings

**(a) There is no per-file seam past `stdlib-parse`.** `stdlib_forms()` (`src/load/stdlib.rs`) is the
**last** place in the whole boot that knows which manifest entry a form came from: it parses the 55
files in a loop and returns ONE flat `Vec<WatAST>`. Every later pass — defmacro registration,
expansion, type registration, define registration — consumes that flat vector and has no file
boundary at all. Row 3's curve therefore is **not** a phase decomposition; it is reconstructed from
each top-level form's `Span::file`, which survives. Stated as a finding rather than papered over,
per the DESIGN's own instruction.

**(b) `check_program`'s name and its argument both hide what it does.** Its signature is
`check_program(forms, sym, types)` and the boot passes `bundle.residue` — the **user** residue. But
four of its passes never read `forms`: they iterate `sym.function_values()` /
`sym.functions_iter()`, which by step 8 holds every function the stdlib registered. That is why
step 8 is recorded as six leaves instead of one — wrapping the call would have preserved the
illusion that its cost is the user program's. It is not: **0.02 ms of the 129.72 ms is the user's.**

---

## Row 2 ⭑⭑ — the phases reconcile with the wall clock

Three totals, printed by the instrument itself, medians of the same 10 runs:

```
ACCOUNTED (sum of 38 leaves) .............. 404.94 ms     (min 397.09  max 424.36)
PIPELINE WALL (1st census call → report) .. 412.72 ms     (min 404.27  max 431.54)
  unaccounted, in-pipeline ................   7.38 ms  = 1.8 %   (min 7.02  max 9.05)
SINCE PROCESS BOOT ....................... 412.87 ms     (min 404.43  max 431.69)
  pre-pipeline (boot → step 1) ...........   0.16 ms  = 0.04 %
```

Externally, through `timeout` + `EPOCHREALTIME` on the same box: **0.447–0.469 s** wall for the
whole process. The ~35 ms between `SINCE PROCESS BOOT` (412.87 ms) and the wall is process exec +
`:user::main` + teardown, all *after* the report is printed; the ~4 ms `timeout` fork is inside it
(measured: `/bin/true` through the identical harness is 0.003–0.006 s).

**No large unexplained remainder. 1.8 % is in `build_env` between named calls** — the `Vec`
plumbing, the `EnvBundle` construction, the bits deliberately left unwrapped so they would show up
here rather than being smeared into a neighbouring phase.

⚠ **One reconciliation caveat, stated because it moved a number.** `stdlib-parse` reads **29 ms
warm** but **33–48 ms on the first runs after a fresh `cargo build`**. The stdlib is
`include_str!`-baked, so a cold run page-faults it in out of the freshly-written binary. Every
number in this SCORE is **warm**; a cold first boot is ~15 ms worse and the whole of that difference
lands in `stdlib-parse`.

---

## Row 3 ⭑⭑ — the per-MANIFEST-ENTRY curve, all 55 (+ the user's entry = 56 rows)

55 entries confirmed against `STDLIB_FILES` in `src/load/stdlib.rs` (a bare `grep -c include_str!`
says 57 — two of those are doc comments). 22,979 lines, matching the prior FINDING.

Median of **10 warm `WAT_BOOT_CENSUS=files` runs**. `ms` is the sum of the five per-file-attributable
passes (parse · defmacro-reg · expand · type-reg · define-reg) = **248.80 ms**; see "coverage" below
for what that does *not* include.

| # | manifest entry | lines | top-level forms | ms | % attributed | µs/line | expand ms | µs/form | defsurface |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | `wat/query/sqlite-store.wat` | 503 | 21 | **35.50** | **14.3** | 70.6 | 33.45 | 1593 | 0 |
| 2 | `wat/query/mem.wat` | 729 | 43 | **31.52** | **12.7** | 43.2 | 28.91 | 672 | 0 |
| 3 | `wat/telemetry/span.wat` | 732 | 26 | **31.09** | **12.5** | 42.5 | 28.41 | 1093 | 0 |
| 4 | `wat/telemetry/journal.wat` | 458 | 7 | **30.98** | **12.5** | 67.6 | 28.94 | **4134** | 0 |
| 5 | `wat/kernel/services/stdio.wat` | 407 | 19 | **24.48** | **9.8** | 60.2 | 22.69 | 1194 | 3 |
| 6 | `wat/cache.wat` | 431 | 16 | 23.92 | 9.6 | 55.5 | 22.41 | 1400 | 1 |
| 7 | `wat/service.wat` | **4604** | 22 | 10.11 | 4.1 | **2.2** | 0.93 | 42 | 1 |
| 8 | `wat/rete/compile.wat` | 1158 | 26 | 7.64 | 3.1 | 6.6 | 5.85 | 225 | 0 |
| 9 | `wat/core.wat` | 2152 | 19 | 6.54 | 2.6 | 3.0 | 0.69 | 36 | 1 |
| 10 | `wat/fix.wat` | 1291 | 68 | 6.38 | 2.6 | 4.9 | 3.81 | 56 | 0 |
| 11 | `wat/bracket.wat` | 1066 | 17 | 4.26 | 1.7 | 4.0 | 2.43 | 143 | 0 |
| 12 | `wat/telemetry.wat` | 523 | 23 | 3.63 | 1.5 | 7.0 | 1.98 | 86 | 3 |

…and the tail falls off a cliff: the bottom 30 entries together are **2.2 %**. The eleven
`wat/holon/*.wat` idiom files are 0.01–0.05 ms each.

### ⭑ TOP 5 AND THEIR SHARE

**153.58 ms = 61.7 % of the attributed 248.80 ms**, from **2,829 of 22,979 lines (12.3 %)** and
**116 top-level forms.** Extend to six and it is **71.3 %**. The curve is not lumpy; it is a spike.

**`µs/form` is the column that says what kind of spike it is.** `wat/telemetry/journal.wat` is
**4,134 µs per top-level form** — seven forms, 28.9 ms of expansion. `wat/core.wat` is 36 µs/form
and `wat/service.wat` 42 µs/form. **A factor of ~100.** The expensive files are not big; they
contain a few enormous forms.

### ⚠ Coverage of this curve — what it does NOT cover

The five instrumented passes are 248.80 of the 404.94 ms accounted (**61.4 %**). The largest
un-attributed piece is **`8f check:body-infer` (109 ms)**, and it cannot be attributed by this
mechanism: it iterates `sym.functions_iter()`, i.e. registered `Function`s, not source forms, so
there is no form whose `Span::file` the guard could stamp. **Naming which files own the 109 ms is a
separate measurement this stone did not take.** Said plainly rather than implied.

---

## Row 4 ⭑⭑ — does expansion dominate? YES — but it is not alone, and the runner-up is new

⭐ **Expansion dominates: `stdlib-expand` = 201.37 ms = 49.7 % of accounted.** The DESIGN's
prediction, inferred from two marginal-cost readings, is **confirmed by profile**.

⛔ **AND SAY THIS LOUDLY TOO: type-checking is 32 %, and it was not on anyone's list.** Step 8 is
**129.72 ms**, of which **109.28 ms is `check_function_body` over every function in the symbol
table**. The prior FINDING's phrase "parses, macro-expands and type-checks" put type-checking third
by implication; it is second, **4.4× parsing**, and — unlike expansion — its cost is invisible at
the call site, because the call is `check_program(&bundle.residue, …)` and `residue` is one form.

Parsing is **7.2 %**. It is not the problem, warm or cold.

---

## Row 5 ⭑ — `defsurface`/`defservice`-bearing files vs the rest: **THE PREDICTION IS BACKWARDS**

First, the count — and **a token count would have got this wrong**, which is why it was not used.
`grep -o ':wat::core::defsurface\|:wat::core::defservice'` over the 55 entries returns **18
occurrences across 10 files**. The form-aware count (head position = child at index 0 of its form,
via `wat --grep`, program recorded at
`wat-scripts/scratch-pad/boot-census/surface-and-service-declarations.wat`) returns:

> **17 `defsurface` declarations across 9 files. ZERO `defservice`.**

17 matches the DESIGN's number exactly, but they are **forms, not files**, and none is a
`defservice`: `defservice` lives in the parked queue promotion, not in today's manifest. The
18th token occurrence is in `wat/sqlite.wat`, which declares nothing.

| | files | lines | ms | **µs/line** |
|---|---:|---:|---:|---:|
| the 9 surface-**declaring** files (17 forms) | 9 | 10,161 | 76.39 | **7.5** |
| the other 46 | 46 | 12,818 | 172.39 | **13.4** |
| all 55 | 55 | 22,979 | 248.78 | 10.8 |

⛔ **Ratio 0.56×. Surface-declaring files are BELOW the average per line, not "far above" it.**
`wat/service.wat` — 4,604 lines, the largest entry in the manifest and the home of the whole
`defservice` machinery — is **2.2 µs/line, among the cheapest files there is.** And four of the top
five most expensive entries declare no surface whatsoever.

**What the expensive files have in common instead is NOT established by this stone**, and it is
being left open on purpose. The correlate visible in the data is that all six of the top entries are
*satisfier*-shaped — `sqlite-store`, `mem`, `span`, `journal`, `stdio`, `cache` are implementations
of surfaces declared elsewhere — while the declarers are cheap. But "satisfying a surface is what
costs" is a **mechanism claim**, and the measurement that would support it (per-form expansion cost
inside one file, keyed by form head) was not taken. `3f159b0c5` was charged for exactly this move.
The measured fact is: **the cost tracks µs/form, and it is not where the declarations are.**

---

## Row 6 ⭑⭑ — off by default, proven

⛔ **My box does not sit in the DESIGN's 0.425–0.438 s band, and neither did HEAD.** Measured
through `timeout` + `EPOCHREALTIME` **before writing a line of code**, at HEAD `5479a7f2e`:
`0.484 / 0.457 / 0.447 / 0.447 / 0.441 / 0.452 / 0.448 / 0.439` s. So a bare "instrumented boot is
in the band" claim would have been unfalsifiable here. The falsifiable comparison is an A/B **on
this box**, so I built one: `git stash`ed the eight modified files, rebuilt a true HEAD binary,
restored, and verified all ten files came back byte-identical by `sha256sum -c`.

**Three runs, env unset, final binary** (the row's literal ask):

```
0.456 s (rc=0)
0.455 s (rc=0)
0.447 s (rc=0)
```

…and three more, same invocation: `0.469 / 0.453 / 0.449 s`. **stderr is 0 bytes**; the string
`WAT_BOOT_CENSUS` appears nowhere in the default output.

**Interleaved A/B, 12 pairs, alternating binaries within the same seconds:**

```
             baseline   instrumented(off)    delta
  pair  1      0.469         0.460          -0.009
  pair  2      0.444         0.452          +0.009
  pair  3      0.444         0.451          +0.007
  pair  4      0.445         0.440          -0.005
  pair  5      0.455         0.440          -0.015
  pair  6      0.450         0.444          -0.007
  pair  7      0.448         0.443          -0.005
  pair  8      0.454         0.451          -0.003
  pair  9      0.439         0.448          +0.009
  pair 10      0.443         0.449          +0.006
  pair 11      0.446         0.456          +0.010
  pair 12      0.452         0.450          -0.002
```

**The delta straddles zero** (−15 ms … +10 ms, median −2.5 ms; baseline median 0.4475 s,
instrumented 0.4495 s). The sign flips run to run, so the off-path cost is below this box's
run-to-run noise. Mechanically that is what the code says it should be: `phase()` is `#[inline]` and
returns `f()` after one read of an already-initialised `OnceLock`; `file_guard()` returns `None` the
same way. No clock, no lock, no allocation.

`WAT_BOOT_CENSUS=garbage` → **one** stderr line naming the variable, the offending text and the
accepted spellings, and the census stays **off**. (The
`WAT_STARTUP_HANDSHAKE_DEADLINE_MS` contract: must not fail, must not read as deliberate.)

---

## Row 7 ⭑ — the instrument's own overhead, and which numbers it distorts

The census has **two levels** precisely so it can measure itself
(`[[feedback_the_measurement_contains_the_measurer]]`):

- `=phases` — 38 clock pairs per boot. Unmeasurable against 400 ms.
- `=files` — **3,115 `file_guard` open/close pairs** (the report prints this count next to its own
  output), each two `Instant::now()` reads + one `String` + one `HashMap` entry.

**Same-phase comparison, medians of 10 runs each mode** — the only way to bound it, since the wall
clock cannot:

| reading | `phases` | `files` | Δ |
|---|---:|---:|---:|
| `3a stdlib-parse` (55 guards) | 29.24 | 29.34 | **+0.10** |
| `4 stdlib-expand` (~2,900 guards) | 201.37 | 201.70 | **+0.33** |
| `4 stdlib-defmacro-register` | 4.36 | 4.46 | +0.10 |
| `5 stdlib-types-register` | 2.70 | 2.86 | +0.17 |
| `6 stdlib-defines-register` | 11.68 | 11.70 | +0.02 |
| `8f check:body-infer` (0 guards) | 109.28 | 108.72 | −0.57 |
| **ACCOUNTED** | 404.94 | 400.62 | −4.32 |

⇒ **≈ 0.11 µs per guard pair; ≈ 0.35 ms total for `files` mode = 0.08 % of boot.** The ACCOUNTED
and whole-boot medians actually came out *lower* in `files` mode by 4.3–4.7 ms — the noise floor is
**more than ten times the effect**, which is the honest statement of the bound.

**Which numbers it distorts:** the five instrumented passes are inflated by ~0.1–0.3 ms each in
`files` mode (≤0.2 % of each, and ≤6 % on `stdlib-types-register`, which is only 2.7 ms — the one
row where the distortion is worth remembering). `check:body-infer` carries **no** guards and is
undistorted. **Every headline number in this SCORE is taken from `=phases` mode**, which carries 38
clock pairs and no allocation, so the phase table above is effectively undistorted; only Row 3's
per-file table comes from `=files`.

One further honesty: the census's own **report printing** happens after the last phase and inside
the pipeline wall, so it is counted in neither `ACCOUNTED` nor the phase rows — it lands in
`SINCE PROCESS BOOT`. It is a few hundred µs of `format!`.

---

## Row 8 ⭑⭑ — Floor

### ⛔ THE FIRST FLOOR WENT RED — TWO FAILURES, BOTH CAUSED BY THIS STONE

Reported before the green, in the order `scripts/floor.sh` demands. **Not re-run on the red**; the
arm was captured whole, the cause of each was found, each was FIXED, and only then was a second
floor run. The first run's evidence is preserved at `.floor/2026-09-17T05-41-16Z/ARM.txt`.

**Summary line, verbatim:**

```
     Summary [ 561.890s] 5284 tests run: 5282 passed (9 slow), 2 failed, 22 skipped
```

```
        FAIL [   0.102s] (  77/5284) wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert
        FAIL [   2.031s] (3287/5284) wat::process probe_supervisor_select_lost::select_prime_yields_lost_when_process_child_crashes
```

- log: `.floor/2026-09-17T05-41-16Z/raw.log` · `clean.log` · **`ARM.txt` — yes, written**

**RED #1 — `tests_carry_no_loose_string_assert`.** The exact arm: `tests/lint/no_loose_string_assert.rs:112`,
the `assert!(violations.is_empty(), …)`, listing **18 sites, every one of them mine** —
`src/freeze/census.rs:717,718,719,720,721,742,743,744,746,760,761,762,777,778,780,783` and
`tests/diagnostics/boot_census.rs:89,108`. The wall was right and it fired on arrival: I had written
the census's report tests as `assert!(t.contains("83.10"))`-style samples, which pass on a report
that has reordered its columns, dropped a row, or grown garbage.

**Fixed by TIGHTENING, not exempting.** `report_text` is a pure function of its readings, so with
fixed inputs its output is deterministic to the byte — the exact form was available and is now used:
three whole-text goldens (`EXPECTED_PHASES_REPORT`, `EXPECTED_STRAY_PHASE_REPORT`,
`EXPECTED_FILES_REPORT`), captured from the renderer rather than guessed, compared with
`assert_eq!`. The unrecognised-mode warning is likewise pinned whole. Only the **two** integration
sites carry a per-site `// rune:lint(loose-assert)` rune, and they earn it under the wall's own
stated exemption: a real boot's report is a ~60-row table whose every millisecond varies per run, so
one is a targeted *presence* and the other a targeted *absence* over a large variable output — and
the wording both of them probe is pinned byte-for-byte by the goldens.

**RED #2 — `select_prime_yields_lost_when_process_child_crashes`.** The exact arm:
`tests/process/probe_supervisor_select_lost.rs:202`, the
`assert_edn_matches_file!(… "probe_supervisor_select_lost__process_panics.edn" …)`. One field
differed, and it names its own cause:

```
actual:    #wat.kernel/Frame {:file "src/freeze.rs" :line 1541 :symbol ":user::main"}
expected:  #wat.kernel/Frame {:file "src/freeze.rs" :line 1521 :symbol ":user::main"}
```

**The golden pins a Rust source LINE NUMBER**, and my census wrappers added a net **+20 lines** to
`src/freeze.rs` — all of them above that frame (`git diff -U0` hunks end at 1362). 1521 + 20 = 1541,
exactly the observed value. The frame genuinely moved; the golden was updated by one digit.

**Sibling check, because a one-sided fix is the habit:** `grep`ped every `.edn`, `.rs` and `.wat`
under `tests/` for a pinned `src/freeze.rs` line. **This is the only one in the tree** — the other
files that mention `src/freeze.rs` do so in prose. No class to gate, and the fragility is named
rather than silently absorbed: *any* edit above `src/freeze.rs:1541` reddens this golden, which is a
property of that test's design and is not this stone's to change.

### The floor after the fixes

**Summary line, verbatim:**

```
     Summary [ 558.656s] 5284 tests run: 5284 passed (9 slow), 22 skipped
```

**5284 / 5284 — the same 5284 tests as the red run, all passing.** Both repairs hold and nothing
else moved.

- log: `.floor/2026-09-17T06-03-42Z/raw.log` · `clean.log` (untruncated, ANSI-stripped, kept before
  reading) — `exit=0. Log kept … regardless — a green run is evidence too.`
- **`ARM.txt`: NOT written**, and that is the correct outcome — `scripts/floor.sh` writes one only
  on a red. The red run's `ARM.txt` is still on disk at `.floor/2026-09-17T05-41-16Z/ARM.txt`.

---

## Row 9 — clippy + `--no-run`

```
cargo clippy --release --workspace --all-targets
    Finished `release` profile [optimized] target(s) in 16.53s       # zero warnings, zero errors

cargo nextest run --release --no-run
    Finished `release` profile [optimized] target(s) in 1m 33s       # every target compiles
```

(The first clippy attempt was a genuine red — `clippy::type_complexity` on `snapshot()`'s 3-tuple
return. Fixed by giving it a named `Snapshot` struct, not by an `#[allow]`.)

---

## Row 10 ⛔ — nothing optimised, no timeout touched

`.config/nextest.toml` — **sha256 unchanged**, recorded before the work began and re-checked after:

```
706f59851bc01cca13ad37f2c0d07abe68a7133d362418ffeb77da21d22a5372  .config/nextest.toml
$ git diff --stat .config/nextest.toml
(empty)
```

The whole diff:

| file | what changed |
|---|---|
| `src/freeze/census.rs` | **new** — the instrument. Env gate, 38 phase names, per-file guards, the report, 9 unit tests. |
| `src/freeze.rs` | `pub mod census;` + 5 phase wrappers (steps 1, 2, 3, 9) + the `report()` call. |
| `src/freeze/env.rs` | 24 phase wrappers around the existing calls, steps 3a–7.8. Not one call reordered, added or removed. |
| `src/check.rs` | 7 phase wrappers partitioning `check_program`, plus comments naming which passes sweep the whole symbol table. |
| `src/load/stdlib.rs`, `src/macros/parse.rs`, `src/macros/expand.rs`, `src/types.rs`, `src/runtime.rs` | one `let _census = …file_guard(…);` at the top of each of the five loops that iterate top-level forms. |
| `tests/diagnostics/boot_census.rs` | **new** — the end-to-end proof that an armed census attributes real time to real manifest entries. |
| `wat-scripts/scratch-pad/boot-census/surface-and-service-declarations.wat` | **new** — the form-aware `defsurface`/`defservice` census for Row 5. |
| `tests/process/probe_supervisor_select_lost__process_panics.edn` | **one digit**, `:line 1521` → `1541` — the golden pins a `src/freeze.rs` source line and my +20 lines moved that frame. See Row 8, RED #2. |

⚠ The `.edn` golden is the ONE behavioural-looking line in the diff, and it is not behavioural: the
frame's `:file` and `:symbol` are unchanged and the runtime emits the same structure it always did.
It moved because the instrument's wrappers are above it in the same file.

No `clone` removed, no allocation avoided, no lookup cached, no batching proposed
(`3f159b0c5` refuted it by measurement; it is not revisited).

**And the diff is checkable as wrapping-only:** `git diff -w src/freeze/env.rs` shows every deleted
line returning as the same call inside a `census::phase(…)` — `let stdlib = stdlib_forms()?;` →
`let stdlib = census::phase(census::P_STDLIB_PARSE, stdlib_forms)?;`, and so on for all 24. Not one
call was reordered, added, or removed; the large raw line count is re-indentation.

### The instrument is itself under test — 10 tests

9 unit (`src/freeze/census.rs`): the default is `Off` and `phase` is pass-through · every accepted
spelling and every refusal · the warning names variable + input + outcome · **the report sums leaves
and reconciles against both anchors** (361/39/30 ms and an 83.10 % share, all asserted as printed
text) · **a phase recorded outside `PHASE_ORDER` makes the report shout `instrument bug`** rather
than silently falsify the total · the per-file table ranks by cost, splits by pass, and states its
own guard count · every pass label has a column · `PHASE_ORDER` has no duplicates · records
accumulate and count.

1 integration (`tests/diagnostics/boot_census.rs`): arms `Mode::Files`, runs a **real** freeze,
and asserts the census accounted >10 ms, attributed ≥50 distinct files, named six real phases and
three real manifest entries by path, printed its guard count and all four reconciliation lines, and
did **not** shout. It asserts on `force_mode`'s return so a shared-process runner is a RED, not a
silent skip.

---

## Row 11 ⭑ — what a fix should target. **A RECOMMENDATION, NOT A DECISION.**

The profile says the target is **not one thing but two, and they are the same shape**: 51 % of boot
is macro-expanding the stdlib and 32 % is type-checking it, so **83 % of a `wat` process's startup
is re-deriving, from identical inputs, two artefacts that are byte-identical in every process and
fully determined at build time** — while parsing, the thing the phrase "22,979 lines of stdlib"
invites you to blame, is 7 %. Anything that caches or precomputes the *expanded, checked*
environment attacks both halves at once; anything aimed at the parser buys 7 % at best. Two things
the curve says are **not** worth attacking: parsing, and the surface/service machinery — the 9
surface-declaring files run at 0.56× the average per line and `wat/service.wat` at 2.2 µs/line, so
the marginal-cost inference that promotion is expensive *because* it is service-bearing is not what
the profile shows; what the profile shows is 6 files, 12 % of the lines, 71 % of the attributed cost,
and a µs-per-**form** spread of ~100×, i.e. a few very large forms rather than a lot of code. Two
cheap follow-on measurements would sharpen this before anyone commits: (a) **which files own
`check:body-infer`'s 109 ms** — this stone could not attribute it, because that pass walks registered
`Function`s and not source forms, so there is no span to stamp; and (b) **per-form expansion cost
inside `telemetry/journal.wat`** (7 forms, 4,134 µs each), which would say whether the spike is one
macro's blow-up or a broad property of satisfier-shaped code. I am explicitly **not** naming a fix:
the builder's ruling is the builder's, and the prior arc was charged for describing a mechanism and
prescribing a remedy in the same breath. **A prescription is a claim.**

---

## What surprised me

1. **Type-checking at 32 %, hidden behind an argument name.** I read `check_program(&bundle.residue,
   …)`, saw that `residue` is the user's forms, and wrote in my own notes that step 8 could not be
   expensive on a one-line program. The census said 130 ms. The four whole-symbol-table sweeps
   inside it are the reason, and `8e check:form-loop(user residue)` = **0.02 ms** is the number that
   proves the argument was never the cost.
2. **The DESIGN's surface/service prediction is inverted.** I expected to confirm it. Surface
   *declarers* are the cheap files.
3. **`wat/service.wat` — the biggest file in the manifest, 4,604 lines — is 4 % of the cost.**
4. **`wat/telemetry/journal.wat`: 7 top-level forms, 29 ms of expansion.** 4.1 ms *per form*.
5. **The token count and the form count disagree** — 18 occurrences, 17 declarations, zero
   `defservice`. The `wat --grep` route was not ceremony; it changed the answer.
6. **`stdlib-parse` moves 29 → 48 ms on the first runs after a rebuild.** Page-faulting the
   `include_str!`ed stdlib in from a cold binary. Every prior hand-timing of "boot" in this campaign
   is warm-or-cold ambiguous by ~15 ms.

---

## ⭑ REGRADED BY THE ORCHESTRATOR ON HIS OWN RUNS, 2026-09-17

```
floor    Summary [ 546.721s] 5284 tests run: 5284 passed (9 slow), 22 skipped
         .floor/2026-09-17T06-17-23Z/ · exit=0 · NO ARM.txt · inside the 540.4–561.5 s band
clippy   0 · nextest --no-run clean · .config/nextest.toml sha256 UNCHANGED
```

| row | the orchestrator's own result |
|---|---|
| 1–2 | ✅ **census re-run independently**: `stdlib-expand` **199.50 ms (48.88 %)**, `stdlib-parse` 30.51 (7.48 %), `check:body-infer(ALL fns)` **111.10 ms (27.34 %)**, `stdlib-defines-register` 11.65, `stdlib-runtime-defs-register` 10.31. **ACCOUNTED 406.37 · PIPELINE WALL 413.26 · unaccounted 6.89 ms = 1.7 %.** Within a percent of the executor's medians. |
| 6 | ✅ **off by default, proven the hard way.** My first attempt reported "1329 bytes on stderr" — **that was my own broken test program** (`:user::main` must return `:wat::core::nil`; `nil` IS the success exit code, arc 170 1e), not the census. With a valid program: **0 bytes**, three runs at 0.444 / 0.457 / 0.466 s. ⚠ My own instrument was the defect, caught before it became a row. |
| 10 | ✅ **measure-only holds**: `git diff` is instrumentation plus one golden line number; **`.config/nextest.toml` untouched**. No optimisation anywhere. |
| — | ✅ the executor's **RED floor is on disk** at `.floor/2026-09-17T05-41-16Z/` with `ARM.txt`, `5282 passed, 2 failed`. Captured, fixed, re-weighed — not re-run away. |

## ⭐⭐ THE TWO RESULTS THAT REDIRECT THE ATTACK

### 1. Type-checking is 32 % of boot — and it is checking the STDLIB, hidden behind an argument name

`check_program(&bundle.residue, …)` reads as *"check the user residue"*. Four of its passes never touch
that argument; they sweep `sym.function_values()` — **everything**. Measured:
`check:form-loop(user residue)` = **0.02 ms**; the remaining **129.7 ms is stdlib**.

⛔ **The executor found it only by partitioning step 8 into SEVEN leaves instead of wrapping the call.**
Wrapping would have produced one honest-looking number and preserved the illusion. ★ **A phase boundary
drawn at the API's name measures the name, not the work.**

### 2. The `defsurface`/`defservice` prediction was BACKWARDS

The DESIGN predicted service-bearing files would dominate. Measured, with a **form-aware** count
(`wat --grep`, not text — text says 18 occurrences across 10 files; the form says **17 `defsurface`, 9
files, ZERO `defservice`**):

```
files DECLARING a surface   7.5 µs/line      the other 46 files  13.4 µs/line     ⇒ 0.56×
wat/service.wat             2.2 µs/line      (4,604 lines — the entire service macro)
four of the top five expensive files declare no surface at all
```

★ **What actually tracks cost is µs per FORM, not per line**: `telemetry/journal.wat` is 7 forms at
~4,134 µs each against `core.wat`'s 36 µs — **~100×**. Top 5 files = **61.7 % of attributed cost from
12.3 % of the lines**; the bottom 30 = 2.2 %.

⚠ **I predicted this wrong in the DESIGN, and the stone was built to let that happen** — predictions
written down first, so the measurement could contradict them. It did.

## The instrument is honest about itself

Overhead **≈0.11 µs per guard pair ≈ 0.35 ms total = 0.08 % of boot**, bounded by same-phase diff
because whole-boot medians differ by −4.7 ms — *noise more than 10× the effect*. Headline numbers come
from `phases` mode (38 clock pairs, no allocation); only the per-file table uses `files` mode.
⭐ And it reports that **no box in this campaign has been sitting in the DESIGN's quoted 0.425–0.438 s
band** — HEAD measured 0.439–0.484 before a line was written — so it built a true baseline via a
verified stash round-trip and ran **12 interleaved pairs** (deltas straddle zero, −15…+10 ms) rather
than compare against a number from a quieter moment. `stdlib-parse` also moves **29 → 48 ms cold vs
warm**, which means **every hand-timing in this campaign is warm/cold ambiguous by ~15 ms**.

## What it refuses to do, correctly

It names no fix. Row 11 is a recommendation and says so: **83 % of boot re-derives, from identical
inputs, the expanded and checked environment — both fully determined at build time — while parsing is
7 %.** Anything caching *that* attacks both halves; anything aimed at the parser buys 7 % at best. Two
things the curve says **not** to attack: parsing, and the surface/service machinery. Two cheap
follow-ons that would sharpen a fix before anyone commits to one:

1. **which files own `check:body-infer`'s 109 ms** — unattributable today because that pass walks
   registered `Function`s rather than source forms, so there is no span to stamp;
2. **per-form expansion cost inside `telemetry/journal.wat`**, the ~100× outlier.
