# SCORE — STONE 251.8d-ii (THIRD DRAW): the bootstrap — **STOP**

Branch `main`, drawn against `2dd489273`. **The conversion is NOT committed.** `wat/` restored,
`git status --porcelain` empty before the SCORE commit. **Not pushed.** 8d-iii not started.
Parent: `BRIEF-STONE-251.8d-ii-THIRD-the-bootstrap.md`.

## VERDICT — STOP. The converted stdlib loads and type-checks. It cannot RUN.

⭐ **The gate the brief said was still owed is the one that failed.** With the converted stdlib
compiled in, `(:wat::kernel::readln)` — and therefore **every codemod, every `wat-scripts` program,
every stdin-reading test** — dies before it reads a byte:

```
#wat.kernel/AssertionFailure {:thread "main" :message "read-frame: stdin request framing rejected (RequestMalformed) — unreachable for a kernel-built request" :location #wat.kernel/Location {:file "/tmp/8d3-readln.wat" :line 2 :col 22} :actual nil :expected nil :frames [#wat.kernel/Frame {:file "/tmp/8d3-readln.wat" :line 2 :symbol ":wat::kernel::stdio-read-frame"} #wat.kernel/Frame {:file "src/freeze.rs" :line 1535 :symbol ":user::main"}] :upstream-chain nil}
```

The cure is **two one-line `type_denotation` calls in `src/`** — measured, below. `src/` is out of
scope for this stone, so: STOP, with the cure named and proven.

## Step by step — commands, timings, exit codes

| step | command | result |
|---|---|---|
| path list | `git ls-files \| grep -E '^wat/.*\.wat$'` | **64**. `git ls-files 'wat/**/*.wat'` → **31** (the glob gap, third confirmation) |
| baseline build | `cargo build --release` | already current (0.19 s) |
| **apply** | `cat /tmp/8d3-wat-paths.edn \| ./target/release/wat ./wat-scripts/fixes/to-faithful-clojure.wat` | **rc 0, 1329 s**, 64/64 logged |
| changed | `md5sum -c` against the pre-image | **64 / 64 changed**; `git status --porcelain` = exactly those 64, nothing else |
| cross-check | `diff -rq wat /tmp/8d-ii-dry/wat` (the REDRAWN dry-run tree) | **0 differences** — a third independent run, byte-identical |
| leftovers | `grep -rl '(:wat::' wat/` | 39 files; **10 non-comment lines, all inside string literals** (`wat/core.wat` `:examples`, `wat/lint.wat` message templates, `wat/fix.wat`'s own quoted forms, `wat/spawn.wat` `:env-fn "(:wat::program::EmptyEnv)"`) |
| build | `cargo build --release` | **exit 0, 23.05 s**, `Compiling wat` shown — the converted stdlib IS embedded |
| starts | `wat --check` of `(wat.core/defn user/main [] :- wat.core/i64 1)` | only `MainSignatureError`, as 255.8 reported. **It loads.** |
| ⭐ **self-application** | converted codemod over 4 real unconverted files | **rc 2 — died in `readln`, 0 bytes written** |
| ⭐ **idempotence** | second pass over the converted live `wat/` | **rc 0, 578 s, 0 / 64 changed** (ran under the two src probes — see below) |
| recovery | `git checkout -- wat/` + `cargo build --release` (23.9 s) | tool works; re-converts the northstar probe **byte-identical to the pre-conversion reference**; `readln` reads again |

`cargo build --release` was run after **every** `wat/` or `src/` edit, and every measurement below
names which binary produced it.

## ⭐ THE MECHANISM — measured, not argued

Instrumented the discarded `RequestMalformed` payload in `wat/kernel/services/stdio.wat` (temporary,
reverted), rebuilt, re-ran:

```
PROBE RM path=["max-buffer-bytes"] expected=:wat::core::i64 got=Integer
```

An `Integer` refused by an `i64` field — because the two sides are not the same string:

- original: `[max-buffer-bytes <- :wat::core::i64]`
- converted: `[max-buffer-bytes :- wat.type/i64]` → identity **`:wat::type::i64`**

`src/edn/render.rs::edn_to_typed_value_inner` matches **`p.as_str()`** against a hard table keyed on
`":wat::core::i64"`. `:wat::type::i64` misses every arm → `mismatch()` → the service rejects the
request. `format_type` denotes for **printing**, which is why the error reads as the nonsense
`expected :wat::core::i64 got Integer`.

Control, decisive: changing that one field back to `wat.core/i64` and rebuilding moved the failure to
the **next** service (`stdio-write-out`) with the same class. The converted stdlib carries **1515**
`wat.type/` occurrences.

**This is the recurring class named in the injected CLAUDE.md** — a string comparison with one side
normalized and the other not. 255.8 added `type_denotation` and wired it into `is_subtype`, the
equatable/orderable gates and `extract_lazyable_elem`. It is **not** wired into the two sites that
decide whether a program can run.

### The two missing sites, and what they buy (probes applied, measured, then reverted)

| # | site | probe |
|---|---|---|
| 1 | `src/edn/render.rs` `edn_to_typed_value_inner`, `TypeExpr::Path(p) => match p.as_str()` | `match crate::edn::render::type_denotation(p).as_str()` |
| 2 | `src/function/subsume.rs` `value_matches_type_by_name`, `if p.as_str() == val_type` | `\|\| type_denotation(p) == type_denotation(val_type)` |

Site 1 alone: `readln` works; the codemod then dies inside `wat/fix.wat:243` on
`:wat::core::-` with `clause 2 skipped (arg 0: expected :wat::core::i64, got :wat::core::i64)` —
self-contradictory for the same reason (declared `:wat::type::i64`, printed denoted). **Every
arithmetic defclause in a converted stdlib is undispatchable.**

Sites 1+2 together:

- ⭐ **SELF-APPLICATION PASSES.** The converted codemod converted all four unconverted files, rc 0,
  and each output is **byte-identical (`cmp`) to the output the pre-conversion codemod produced on
  the same inputs**: `tests/rete/probe_arc278_northstar_cold_and_windy.wat`,
  `tests/services/probe_arc272_rs1_state_must_be_record.wat`,
  `tests/services/probe_arc278_sift_rules_arena.wat`,
  `wat-scripts/scratch-pad/census-one-param-spec.wat`.
- ⭐ **IDEMPOTENCE PASSES.** Second pass over live converted `wat/`: rc 0, 578 s, **0/64 changed**.
  Corroborated on the 179-file converted sample: rc 0, 21.8 s, **0/179 changed**.
- Floor: **739 → 416** failures (below).

**Neither probe is committed.** `git checkout -- src/` restored both; the rebuilt tool reproduces the
pre-conversion behaviour exactly.

## Gates

| gate | result | evidence |
|---|---|---|
| ⭐ converted codemod CONVERTS | ❌ **FAIL as drawn** (`readln` dies, 0 bytes). ✅ only with the two src probes, byte-identical to reference | above |
| ⭐ idempotent, 0 changes | ✅ **0 / 64**, rc 0, 578 s (under the probes — it cannot be run without them) | `md5sum -c`, 0 FAILED |
| floor green | ❌ **RED — 739 failed** | below, captured |
| clippy 0 | **not run** — the tree is restored to `2dd489273`; there is no `src/` change to lint, and the two probes are reverted | — |
| census `no STOP-8` | ❌ **FAIL — 131 STOP-8** | below |
| delta ≤ 18 | ✅ **18** pre-conversion (baseline reproduced exactly), **16** against the converted stdlib | below |
| `wat/` converted, nothing else | ✅ then reverted (STOP) | `git status` = 64 paths, all `wat/` |

### Floor — ⛔ A RED IS A RED. Captured, not re-run.

**FLOOR-A**, the state this stone would have landed (converted `wat/`, no `src/` change),
`.floor/2026-09-22T03-36-00Z/`:

```
     Summary [ 430.539s] 5959 tests run: 5220 passed (11 slow), 739 failed, 22 skipped
```

exit=100. **739 unique failing tests.** ARM.txt is 4.8 MB; pasting 739 whole blocks is not possible
in this document, so: the whole capture is at `.floor/2026-09-22T03-36-00Z/ARM.txt` and
`clean.log`, the complete failing-test list is in ARM.txt's `FAILING TESTS` section, and the
distribution by binary/module is:

```
228 wat::kernel test           96 wat rete                58 wat::rete probe_arc278_export
 50 wat runtime                34 wat::cli every_recorded_migration_replays
 32 wat::lint rete_compile_gate  22 wat::rete probe_arc278_vsa_where_native_differential  …
```

Two whole blocks, verbatim, naming two distinct arms:

```
        FAIL [   0.004s] (   1/5959) wat::lint ast_kind_nodekind_sync::ast_kind_arms_match_nodekind_variants
  stdout ───

    running 1 test
    test ast_kind_nodekind_sync::ast_kind_arms_match_nodekind_variants ... FAILED

    failures:

    failures:
        ast_kind_nodekind_sync::ast_kind_arms_match_nodekind_variants

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 359 filtered out; finished in 0.00s
    
  stderr ───

    thread 'ast_kind_nodekind_sync::ast_kind_arms_match_nodekind_variants' (642091) panicked at /home/john/work/holon/wat-rs/tests/lint/ast_kind_nodekind_sync.rs:43:9:
    defenum :wat::grep::NodeKind not found in wat/grep.wat
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Arm: a lint that **greps `wat/*.wat` for the rust-scheme spelling**. Mechanism: the file now says
`(wat.core/defenum wat.grep/NodeKind …)`. This class (lints that read the stdlib as TEXT) is real
work for 8d-iii and is invisible to `--check`.

```
        FAIL [   0.838s] (2005/5959) wat runtime::tests::step_user_function_call
  stdout ───

    running 1 test
    test runtime::tests::step_user_function_call ... FAILED

    failures:

    failures:
        runtime::tests::step_user_function_call

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1323 filtered out; finished in 0.83s
    
  stderr ───

    thread 'runtime::tests::step_user_function_call' (668889) panicked at src/runtime.rs:14710:13:
    type-check errors in test wat:
    #wat.check/CheckErrors {:message "2 type-check errors" :location nil :causes [] :errors [#wat.check/MalformedForm {:message "malformed :wat::core::i64 form: Doctrine 1 (arc 242): ':wat::core::i64' is a TYPE keyword, not a value; use a value of this type in value position" :location #wat.core/Span {:file "src/runtime.rs:14693" :line 14 :col 55 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 14 :col 70}}} :causes [] :head ":wat::core::i64" :reason "Doctrine 1 (arc 242): ':wat::core::i64' is a TYPE keyword, not a value; use a value of this type in value position" :remedies []} #wat.check/MalformedForm {:message "malformed :wat::core::i64 form: Doctrine 1 (arc 242): ':wat::core::i64' is a TYPE keyword, not a value; use a value of this type in value position" :location #wat.core/Span {:file "src/runtime.rs:14693" :line 14 :col 75 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 14 :col 90}}} :causes [] :head ":wat::core::i64" :reason "Doctrine 1 (arc 242): ':wat::core::i64' is a TYPE keyword, not a value; use a value of this type in value position" :remedies []}]}
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Arm: `MalformedForm` — a **rust-embedded** test program whose `:wat::core::i64` annotation is now
read in value position against the converted stdlib. A third class.

**FLOOR-B — a DIFFERENT tree state, not a re-run of A.** Converted `wat/` **plus the two uncommitted
`src/` probes**, `.floor/2026-09-22T03-57-43Z/`:

```
     Summary [ 431.099s] 5959 tests run: 5543 passed (14 slow), 416 failed, 22 skipped
```

`comm` on the two sorted failure sets: B ⊂ A exactly — **416 of A's 739 remain, 0 new**. So the two
denotation sites account for **323** failing tests. The 416 remainder is dominated by `wat::rete`
(195) and rete type inference degrading under the converted stdlib, e.g.

```
    thread 'probe_arc278_export::import_refuses_a_driver_tower_past_the_depth_bound' (1075190) panicked at /home/john/work/holon/wat-rs/tests/rete/probe_arc278_export.rs:890:41:
    freeze: #wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/TypeMismatch {:message ":wat::rete::i64::+: parameter #1 expects :wat::core::i64; got :wat::core::keyword" :location #wat.core/Span {:file "tests/rete/probe_arc278_export.wat" :line 24 :col 52 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 24 :col 54}}} :causes [] :callee ":wat::rete::i64::+" :param "#1" :expected ":wat::core::i64" :got ":wat::core::keyword" :remedies []}]}
```

FLOOR-A's red stands, reported whole. FLOOR-B does not retract it; it sizes it.

### Census — 131 STOP-8

`scripts/replay/census.sh` with the converted stdlib: `.census/2026-09-22T03-34-24Z.txt`,
files=2202, 60 s. Diff against 255.8's accepted `.census/2026-09-21T23-47-29Z.txt`:

```
census-diff rc=8, STOP-8 = 131        (wat-scripts 86 · tests 41 · docs 3 · wat-tests 1)
```

Passing the 64 produced paths as the exemption list changes nothing (**131 either way**) — **no
`wat/` file regressed**: `wat/` standalone-check failures are **37/64 before and 37/64 after, the
same 37** (the check-a-stdlib-file-out-of-context artifact). Corpus totals: **212 → 343 failures,
0 recoveries.**

### Delta — 18 (baseline reproduced) / 16 (against the converted stdlib)

**The sample is now a committed file: `docs/arc/2026/06/251-types-as-forms/delta-sample-179.txt`**
(179 paths, sha256 `33ede76cbb9b8809ecebc7656db3d4733769df54c2641b7e6d798773f1cbc3e5`). It is the
exact list 255.1→255.8 used (`/tmp/255-1-sample179.txt`), all 179 paths still present in the tree.

⚠ **It is NOT reproducible by the recipe that names it.** "Every 12th of the 8d-i census" was
2145 files then; the census is **2152** today (`git ls-files | grep '\.wat$' | xargs grep -l '(:wat::'`),
so `awk 'NR%12==1'` now yields **180** paths and diverges from the committed list after entry 47
(132 lines differ). **That** is why the last two stones disagreed on the baseline. Committing the
list ends it.

| | 255.8 | this stone, pre-conversion binary | this stone, converted binary |
|---|---|---|---|
| originals clean | 161 / 179 | **160 / 179** | **148 / 179** |
| converted clean | 143 | **143** | **143** |
| **NEW (orig clean → conv broken)** | **18** | **18** | **16** |

Kinds of the 18, pre-conversion: MalformedDecl 8, UnresolvedReferences 4, ProgramBodyEvalFailed 3,
CheckErrors 1, DuplicateMacro 1, DuplicateType 1 — **exactly 255.8's breakdown.** The baseline is
reproduced.

The 160-vs-161 is explained, not waved: I check the originals from `/tmp` **copies** (255.8 checked
them in the live tree). `wat-scripts/fmt/run-all.wat` does `(:wat::load-file! "rules/defn.wat")`,
which a copy tree cannot satisfy. In-tree it is rc 0. It is not a new break under either convention.

⚠ **The post-conversion 16 is a number whose DENOMINATOR moved.** The reference (the stdlib) changed,
so 12 unconverted originals went clean → broken:

```
#wat.check/CheckErrors        tests/rete/probe_enum_name.wat
#wat.resolve/UnresolvedRefs   tests/services/probe_arc278_sift_logs.wat
#wat.check/CheckErrors        wat-scripts/fixes/rename-four-families-to-their-homes.wat
#wat.check/CheckErrors        wat-scripts/fixes/rename-string-verbs-to-their-home.wat
#wat.check/CheckErrors        wat-scripts/fmt/rules/defn-args.wat
#wat.check/CheckErrors        wat-scripts/grep/core-numerics-ops.wat
#wat.resolve/UnresolvedRefs   wat-scripts/probes/arc-278/s2s-process-probe.wat
#wat.check/CheckErrors        wat-scripts/scratch-pad/277-all-atom-pair-runs.wat
#wat.check/CheckErrors        wat-scripts/scratch-pad/277-is-colon-dash-always-a-type-app.wat
#wat.check/CheckErrors        wat-scripts/scratch-pad/probe-cond-in-where-baseline.wat
#wat.macro/DuplicateMacro     wat/holon/Ngram.wat          ← the transitional duplicate, mirrored
#wat.type/DuplicateType       wat/source.wat               ← same class
```

Most read like the rete surface losing an operand's type: *"the rete enum-equality surface admits
ENUM operands only — got `_` and `:wat::grep::NodeKind.Vector`"*. Same family as FLOOR-B's 195.

## ⭐ `wat/holon/Ngram.wat` — TRANSITIONAL. Measured in both directions.

| binary | converted `Ngram.wat` copy | UNCONVERTED copy (control) |
|---|---|---|
| pre-conversion stdlib | rc 1 — `DuplicateMacro :wat::holon::Ngram` | **rc 0** |
| converted stdlib | **rc 0** | rc 1 — `DuplicateMacro :wat::holon::Ngram` |

In-tree `wat/holon/Ngram.wat` under the converted stdlib: **rc 0**.

It is **symmetric**: the duplicate fires whenever the file's binder-marker spelling differs from the
embedded snapshot's, in either direction. `macro_structurally_equivalent`'s `Keyword`↔`Symbol` arm
requires `id.is_reference()`, and `<-` / `->` / `:-` are not references, so bodies differing only by
the marker compare unequal. So: **not a defect that survives a fully converted corpus** — but during
8d-iii the hazard **changes sides**: every still-unconverted corpus file that re-declares a stdlib
macro/type shows a spurious duplicate against the converted stdlib. `wat/source.wat`
(`DuplicateType`) is the second witness of exactly that.

## ⛔ SEVENTEENTH CORRECTION — and a finding the brief did not predict

**The brief's hazard was the wrong one.** It flagged `Ngram`/`DuplicateMacro` as the measured hazard
and the wrong-join as the out-of-scope caveat. Both are real and both are survivable. What actually
stops the stone is the **EDN coerce table and clause dispatch**, neither of which is mentioned. "It
loads" was one layer short **again** — and one layer further than the brief expected: `--check` is
green on the converted stdlib while **nothing that reads stdin or does arithmetic can run**.

**A separate, unrelated finding, out of scope, not fixed:** the codemod converts
`(:wat::load-file! "x.wat")` → `(wat/load-file! "x.wat")`, and
`src/load/loader.rs::match_load_form` matches **`WatAST::Keyword` heads only** (`_ => return
Ok(None)`). The converted form is therefore a **silent no-op**: measured, `wat --check` of
`(:wat::load-file! "nope.wat")` is rc 1 (`wat.load/Fetch … file not found`) and of
`(wat/load-file! "nope.wat")` is **rc 0**; at runtime the keyword form loads the file and the symbol
form does not. **26 tracked `.wat` files use the six load forms**; zero are in `wat/`, so this stone
is unaffected — but in 8d-iii the flip will **delete their loads and the corpus will get GREENER as
it does so**, because `--check` cannot see a load that no longer happens. `wat-scripts/fmt/run-all.wat`
already demonstrates it: converted copy rc 0, unconverted copy rc 1, same missing sibling.

## What this green CANNOT see

- **Nothing landed.** Every number here describes a tree state that was reverted. There is no
  committed conversion to defend.
- **`--check` is not "runs".** That is the whole finding. 100 % of `wat/` type-checked while `readln`
  and `:wat::core::-` were dead. Any future gate phrased as "the stdlib checks" proves nothing about
  the bootstrap; the only proof is **the converted codemod converting a real file**, which is why the
  brief was right to demand it.
- **Four files are not the corpus.** Self-application (under the probes) was proven byte-identical on
  4 real files, not on 2152.
- **The two probes are diagnosis, not a design.** They were applied to locate the wall and reverted.
  `type_denotation` in `edn_to_typed_value_inner` changes the coercion of every EDN-decoded service
  request in the system; nobody has weighed that. 416 tests are still red with them applied.
- **255.8's wrong-join hole is untouched** (`(wat.core.Option.expect …)` resolves). A converted
  stdlib can carry a wrong-join call head and still be green. Nothing here tests that.
- **No clippy row.** The tree is at `2dd489273` with no `src/` change; the number would describe a
  state nobody proposes to land.
- **Floor-A's 739 blocks are not all quoted.** 4.8 MB. Three arms are quoted whole; the rest live in
  `.floor/2026-09-22T03-36-00Z/ARM.txt`, which is on disk and untruncated.

## What the next stone needs (not started here)

1. `type_denotation` at `src/edn/render.rs::edn_to_typed_value_inner` and
   `src/function/subsume.rs::value_matches_type_by_name` — with a probe per site that goes red without
   it, and a weigh of what else those two comparisons gate.
2. The 416 remainder, led by rete inference losing an operand's type under a converted stdlib.
3. The `load-file!` head (keyword-only in `match_load_form`) — **before** 8d-iii converts the 26 files
   that use it, because the flip silently removes their loads and the census gets greener.
4. The lint family that reads `wat/*.wat` as TEXT and greps the rust-scheme spelling.

## Artifacts

`.floor/2026-09-22T03-36-00Z/` (A, 739 red) · `.floor/2026-09-22T03-57-43Z/` (B, 416 red) ·
`.census/2026-09-22T03-34-24Z.txt` · `/tmp/8d3-convert.log`, `/tmp/8d3-idem.log`,
`/tmp/8d3-delta/delta-pre.tsv`, `/tmp/8d3-delta/delta-post.tsv`, `/tmp/8d3-self/` ·
committed: `docs/arc/2026/06/251-types-as-forms/delta-sample-179.txt`.
