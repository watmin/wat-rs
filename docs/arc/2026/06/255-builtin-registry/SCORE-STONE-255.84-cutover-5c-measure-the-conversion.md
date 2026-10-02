# SCORE — STONE 255.84: cutover 5c, measured — what the head conversion does to today's tree

Branch `main` @ `01cbefa48`. The pin in the brief is `167256d8b`; `git diff --stat 167256d8b HEAD` is this stone's brief only. **No source file was converted in the main tree.** The only commit this score asks for is itself. **Not pushed.**

Every conversion ran on a copy. Corpus copies are under `/tmp/census-5c/tree`. The stdlib conversion is the shared clone `/tmp/wat-5c` checked out at `167256d8bce56df4f4a5e61c1470c812644b7a79`. The pristine tool is `/home/john/work/holon/wat-rs/target/release/wat` (mtime 2026-10-02 14:53). `git status --porcelain` in the main tree was empty after every run (the last reading is 0 lines).

Head count, used everywhere below: a `::`-keyword call head is the first token of a list, outside strings and `;;` comments, whose text starts with `:` and contains `::`. The scanner is `/tmp/census-5c/drive.py` (`call_heads`). "Heads changed" is that count before minus that count after.

## 1. Driver

| run | command | result |
|---|---|---|
| one file | copy of `tests/collection/probe_seq_container_parity.wat` to `/tmp/pilot-5c.wat`, then `./target/release/wat ./wat-scripts/fixes/to-faithful-clojure.wat` | RC=0, stderr `ELAPSED 0.41`, stdout one EDN-quoted path. Heads 19 → 0. `(:wat::core::defn :p::first-pv [] -> wat.type/i64` became `(wat.core/defn p/first-pv [] :- wat.type/i64`. |
| twenty files | first 20 tracked `.wat` outside `wat/`, copied under `/tmp/pilot-batch/`, one EDN vector | RC=0, stderr `ELAPSED 1.72`, 20 stdout lines |

**Driver chosen: resumable batches of 40.** A batch is one wat process over one EDN path vector. On a nonzero exit the batch's copies are restored from the repo and each file is retried alone; the failure is recorded and the scan continues; the process exits 0. A 20-file batch cost 1.72s against 0.41s for a process per file, and `apply-each` writes each file only after `fix-text` returns, then raises out of the process on the first failure, so a batch that dies has already written the earlier files and has not touched the later ones. Restoring the batch and isolating is what makes a failure a record instead of a stop.

The census log is `/tmp/census-5c/driver.log` (the second `TRACKED_OUTSIDE_WAT` line is the run that was kept). 56 batches, 1 isolate, sum of the batch `elapsed=` fields **666.25s**. Per-file rows: `/tmp/census-5c/results.tsv`.

## 2. Corpus census

Tracked `.wat` outside `wat/`: **2219** (`git ls-files '*.wat'`). All 2219 have a row.

| | files | heads before | heads after | bytes changed |
|---|---:|---:|---:|---:|
| OK | 2217 | 86587 | 0 | 2213 changed, 4 unchanged |
| FAIL | 2 | 3 and 3 | left byte-identical to the repo (`cmp`) | 0 |

Head-delta buckets on the OK files (before − after):

| delta | files |
|---|---:|
| 0 | 12 |
| 1–9 | 898 |
| 10–99 | 1088 |
| 100–999 | 217 |
| 1000+ | 2 |

The two largest are `tests/cli/mode_parity__deep_freeze_recursion.wat` (3005) and `wat-scripts/fmt/rules/kwargs.wat` (1082).

OK and unchanged (0 heads either side): `tests/cli/mode_parity__empty.wat`, `tests/resolve/probe_arc251_type_namespace_fix__c02-core-parametric.wat`, `tests/types/typed_if_match__if_wrong_arity_needle.wat`, `wat-scripts/scratch-pad/probe-qq-arm-shape-src.wat`.

OK, bytes changed, 0 call heads either side (arrows, names, or type forms, no `::` call head): `tests/resolve/probe_arc251_stone9__malformed_variant_sym.wat`, `tests/resolve/probe_arc251_stone9__missing_purity_sym.wat`, `tests/resolve/probe_arc255_82_bound_bare_head.wat`, `tests/resolve/probe_arc255_82_quasiquote_bare_head.wat`, `tests/resolve/probe_arc255_83_diff_expect_sym.wat`, `tests/resolve/probe_arc255_83_qq_pure_sym.wat`, `wat-scripts/fmt/fixtures/spelling-clojure.wat`, `wat-scripts/scratch-pad/probe-arc255-66-the-534.wat`.

### Failures

Both are one class. `fix-text` raises inside `:wat::keyword::to-type-form` and the file is left untouched. The full stderr is `/tmp/census-5c/fail/tests__function__fn_rename_bare_fn_type.wat.err` (849 bytes; the other file's stderr is the same sentence).

```
#wat.kernel/LociDiedError.RuntimeError {:message "#wat.runtime/MalformedForm {:message \"malformed :wat::keyword::to-type-form form: type-keyword parse failed: MalformedTypeExpr { raw: \\\":fn(wat::core::i64)->wat::core::i64\\\", reason: \\\"a keyword-bodied fn type is retired; write the bracket `[A :-> R]` (stone 251.4c). `:fn(A)->R` and `:wat::core::Fn(A)->R` no longer parse\\\" }\" :location #wat.core/Span {:file \"wat/fix.wat\" :line 362 :col 42 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 362 :col 76}}} :causes [] :head \":wat::keyword::to-type-form\" :reason \"type-keyword parse failed: MalformedTypeExpr { raw: \\\":fn(wat::core::i64)->wat::core::i64\\\", reason: \\\"a keyword-bodied fn type is retired; write the bracket `[A :-> R]` (stone 251.4c). `:fn(A)->R` and `:wat::core::Fn(A)->R` no longer parse\\\" }\"}"}]
```

| file | where the keyword sits |
|---|---|
| `tests/function/fn_rename_bare_fn_type.wat` | line 4, `[g <- :fn(wat::core::i64)->wat::core::i64]` |
| `tests/function/fn_rename_mixed_legacy.wat` | line 4, `((g :fn(wat::core::i64)->wat::core::i64)` |

The isolate was batch `i=160` (`rc=1`, 10 of 40 printed, then per-file). These two are the only FAIL rows.

### Against the 2026-09-19 classes

The 2026-09-19 census (`BRIEF-STONE-251.8d-the-corpus-flip.md`) on 2140 files with `::` call heads: A 0, B 99 (refusing to splice), C 20 (trailing-`::` marker handed to `keyword/to-symbol`). Today's census of all 2219 tracked `.wat` outside `wat/`:

| class | 2026-09-19 | this census |
|---|---:|---:|
| A — `ast-name requires a Symbol, Keyword, or StringLit` | 0 | 0 |
| B — `refusing to splice` | 99 | 0 |
| C — `not a convertible call-head/reference keyword` (bare data keyword or namespace-prefix marker) | 20 | 0 |
| retired keyword-bodied fn type (`:fn(A)->R` inside `to-type-form`) | — | 2 |
| other | 0 | 0 |

`wat/fix.wat` now has `marker-keyword?`, and the OK copies contain the rewritten markers (section 6). Class C did not fire on this corpus. Class B did not fire on this corpus. Both sentences fire on embedded Rust literals (section 5).

## 3. Stdlib bootstrap

| step | result |
|---|---|
| `git clone --shared /home/john/work/holon/wat-rs /tmp/wat-5c` then `git checkout 167256d8bce56df4f4a5e61c1470c812644b7a79` | `CHECKOUT_RC=0`. Detached HEAD. Log `/tmp/stdlib-5c.log`. |
| path list | `git ls-files \| awk '/^wat\/.*\.wat$/'` → **65** |
| convert with the pristine binary, codemod `wat-scripts/fixes/to-faithful-clojure.wat`, paths inside the clone | `BATCH rc=0 elapsed=1312.08 stdout_lines=65 stderr_bytes=0`. **CHANGED 65 OF 65** against the main-tree bytes of the same paths. `git diff --stat` in the clone: `65 files changed, 11653 insertions(+), 11653 deletions(-)`. |
| `cargo build --release` in the clone | RC=0, `Finished release profile in 52.41s`, log line `Compiling wat v0.1.0 (/tmp/wat-5c)`. `/tmp/wat-5c-build.log`. |
| `scripts/floor.sh` in the clone, foreground, nothing else running | **RED. exit=100. Not re-run.** |

Summary, verbatim, from `/tmp/wat-5c/.floor/2026-10-02T23-03-54Z/` (also the `[floor]` line in `/tmp/wat-5c-floor.wrap`, `RC=100`):

```
Summary [ 307.162s] 6389 tests run: 2391 passed (16 slow), 3998 failed, 24 skipped
```

The converted stdlib **loads**. 2391 tests passed. It does not run the floor. The dominant death is at startup, inside `validate_named_type_annotations` (`src/check.rs:15382`), and the name it reports is a `wat.type` member.

A walk of `panicked at` lines in `ARM.txt` counted **4004** panic sites (the floor's own failed-test count is 3998; the walk is a reading of the captured text, and it is what the buckets below add up from). `/tmp/floor-group.out`.

| mechanism | panic sites |
|---|---:|
| startup `UnknownNamedType`, message `not a member of wat.type` | 3873 |
| `assertion left == right failed` (262 distinct assertion texts, 315 hits of that sentence in the file) | 61 |
| mcp server closed stdout | 13 |
| `expected a type-check error` | 8 |
| `UnknownNamedType` with some other message | 3 |
| `LociDiedError :message must be a string; got None` | 2 |
| child pid line empty | 2 |
| one-off gates and rete probes | the rest, 1 each |

Mentions of the three paths inside `ARM.txt` (a test's output can mention a path more than once): `:wat::type::Error` 5225, `:wat::type::EvalError` 378, `:wat::type::Bogus` 8.

One whole ARM block of the dominant mechanism, from `ARM.txt`:

```
        FAIL [   1.135s] ( 252/6389) wat::lint probe_arc277_1c_concat_format_autofix::bare_symbol_concat_rewrites_to_format
  stdout ───

    running 1 test
    test probe_arc277_1c_concat_format_autofix::bare_symbol_concat_rewrites_to_format ... FAILED

    failures:

    failures:
        probe_arc277_1c_concat_format_autofix::bare_symbol_concat_rewrites_to_format

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 374 filtered out; finished in 1.10s

  stderr ───

    thread 'probe_arc277_1c_concat_format_autofix::bare_symbol_concat_rewrites_to_format' (121827) panicked at /tmp/wat-5c/tests/lint/probe_arc277_1c_concat_format_autofix.rs:19:41:
    startup: #wat.type/UnknownNamedType {:message "not a member of wat.type: :wat::type::Error" :location #wat.core/Span {:file "src/check.rs" :line 15382 :col 13 :end #wat.core/Option.None {}} :causes [] :path ":wat::type::Error"}
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Separate from that startup death, several gates read the stdlib **text** for a keyword declaration and went blind on the converted spelling. Each fired once:

| test | panic text |
|---|---|
| `ast_kind_nodekind_sync::ast_kind_arms_match_nodekind_variants` | `defenum :wat::grep::NodeKind not found in wat/grep.wat` |
| `gen_doc_surface_matches::every_exported_gen_verb_is_documented` | `parsed only 0 verbs from wat/gen.wat` |
| `gen_doc_surface_matches::every_gen_name_the_doc_writes_actually_exists` | `docs/GENERATIVE-TESTING.md names 9 :wat::gen:: symbol(s) that do not exist in wat/gen.wat` |
| `rete_header_claims_are_asserted::session_record_field_count_matches_its_doc` | `` `:wat::rete::Session` defrecord is gone from wat/rete.wat `` |
| `no_raw_network_keys_in_oracle::no_raw_network_keys_walk_outside_topological_node_ids` | `:wat::rete::topological-node-ids must exist and be the one keys-walk` |
| `rete_bind_generators::rete_bind_generators_emit_binder_not_left_arrow` | `found no symbol-node "?…" bindings` |
| `rete_names_in_wat_scripts_resolve::known_forms_are_real` | `known-forms self-check went blind: 177 attested name(s), 1 form(s) declared` |

The reds were not cured. `wat/` in the main tree was not restored because it was never written. The clone keeps the converted `wat/`.

### Why the 8d-ii draws stopped

Each of these stopped the conversion and kept a cure, or stopped before a floor. Read from their SCORE headings.

| draw | stop |
|---|---|
| `251.8d-ii` | Converted stdlib did not load. `wat-doc`'s `wat_enum_from!` required a keyword `defenum` head; after a dual-spelling read, startup still died: `MalformedDecl`, `defenum` name must be a keyword (`wat/core.wat`). |
| REDRAWN | 64/64 converted, `cargo build --release` exit 0, startup died `UnknownNamedType` on `:wat::telemetry::Journal::QueryMetricsRequest` (and a second process named `:wat::query::Store::EnsureSchemaRequest`). Floor not run. |
| THIRD | Converted stdlib loaded and type-checked. `(:wat::kernel::readln)` died: `read-frame: stdin request framing rejected`. |
| FOURTH | Cure landed (floor 5989/5989 on the unconverted tree). Converted floor 416/5989, the same 416 the third draw had measured. |
| FIFTH | Purity-head cure landed. Converted floor 416 → 283. Conversion not committed. |
| SIXTH | Acc-fold cure landed. Converted floor 283 → 161. Conversion not committed. |
| SEVENTH | `:then` cures landed. Converted floor 161 → 112. Conversion not committed. |
| EIGHTH | Accessor-join cure and gate landed. Conversion not committed. The converted spelling `wat.spawn.process/post-spawn` is the image of two keyword spellings. |

Today's floor is the same question with today's checker: the binary builds, tests run, and 3873 panic sites are `UnknownNamedType` / `not a member of wat.type`.

## 4. Corpus on the converted stdlib

`scripts/replay/delta.sh --binary /tmp/wat-5c/target/release/wat --out /tmp/delta-5c` in the clone. Log `/tmp/delta-5c.log`.

```
[delta] list     /tmp/wat-5c/docs/arc/2026/06/251-types-as-forms/delta-sample-179.txt
[delta] list-sha 33ede76cbb9b8809ecebc7656db3d4733769df54c2641b7e6d798773f1cbc3e5
[delta] binary   /tmp/wat-5c/target/release/wat
[delta] paths=179 missing=1
[delta] MISSING wat-scripts/scratch-pad/probe-arc278-57-persistentmap-contains-key.wat
[delta] ⛔ the committed list does not match the tree — fix the list or the tree, do not paper over it
RC=1
```

The path is line 151 of the committed list. The file is absent in the main tree and in the clone. `git log --diff-filter=D` names the deleting commit `581478c9c` (`255.82: a slash-less call head that names nothing is refused`). The script exits before the codemod and before either `--check`. **NEW and RECOVERY were not produced.** The list was not rebuilt.

## 5. Embedded wat

When each converter applies, from its header:

| converter | applies |
|---|---|
| `to-faithful-clojure.wat` | The corpus codemod. Drives `:wat::fix::fix-text`. Span-faithful, comment-faithful. A file it cannot convert is left byte-unchanged because `fix-text` raises before `write-file`. |
| `to-faithful-clojure-net.wat` | The same conversion as a forward-chaining rete network. Rules deduce; the drive actions. Gate is a round-trip parse of the output. A synthesized head (span length ≠ name length) never becomes `:fix::Genuine`, so it never becomes an edit. |
| `to-faithful-clojure-rete.wat` | Pure rete: rules deduce `:fix::HeadConv`, `:fix::ArrowConv`, `:fix::TypeConv`; the drive builds span edits and calls `fix-text-apply`. Header: byte-identical to fix-text on `wat/source.wat`. `source.wat` has no `if`, so `strip-if` is not exercised there. |

`git ls-files '*.rs'` → **1300**. Each run was `./target/release/wat-fix-rust <codemod> --dry-run --list /tmp/rs-list.txt` from the main tree. Dry-run writes nothing: porcelain stayed 0.

`wat-fix-rust` returns FAILURE on the first codemod-batch error and prints no final "N file(s) scanned" line (`src/bin/wat-fix-rust.rs`). The counts below are only the files it printed before that return. A file with zero edits is not printed, so these are not a scan of 1300.

| converter | printed files | edits found | refused splices printed | where it stopped | RC |
|---|---:|---:|---:|---|---:|
| `to-faithful-clojure.wat` | 4 | 93 | 0 | `crates/wat-edn/tests/spec_strict.rs`, batch of 3, exit Some(1) | 1 |
| `to-faithful-clojure-net.wat` | 4 | 93 | 0 | same file, same batch shape, exit Some(1) | 1 |
| `to-faithful-clojure-rete.wat` | 1 | 76 | 0 | `crates/wat-doc/src/lib.rs`, batch of 25, exit Some(2) | 1 |

The four printed files for fix-text and for `-net` are `benches/perf_arc278_fire_baseline.rs` (76), `crates/wat-doc/src/lib.rs` (11), `crates/wat-doc/src/print.rs` (3), `crates/wat-edn/tests/round_trip.rs` (3). `-rete` printed only the bench file, then died on `lib.rs`.

fix-text and `-net` die in class C. The keyword text in the message is `"::`"`. From `/tmp/wfr-to-faithful-clojure.log`:

```
malformed :wat::keyword::to-symbol form: not a convertible call-head/reference keyword (bare data keyword or namespace-prefix marker): "::`"
```

`-net` raises the same sentence at `to-faithful-clojure-net.wat:271`. `-rete` dies in class B, at `fix-text-apply` (`wat/fix.wat:507`), on `lib.rs`:

```
fix-text-apply: edit at offset 37 claims old-text ":wat::core::quasiquote" (22 char(s)) but the source there is "`wat.core/Bytes`); gra" — the rule's belief and the source disagree; refusing to splice.
```

Printed edits also show the extractor handing the converter a token with the Rust fence still on it: `:wat::core::Bytes` → `wat.core/Bytes` (the backtick is in the log's old and new text). `:wat::core::File::write` → `wat.core.File/write`. `:wat::rete::Session/production-memory` → `wat.rete.Session/production-memory`.

## 6. What the conversion produces that 5a/5b treat differently

The door was measured with the pristine binary on `/tmp/id-probe.wat` (RC=0, `/tmp/id-probe.out`). `canonical-identity` prints a quoted string:

| input | printed |
|---|---|
| `wat.core/if` | `":wat::core::if"` |
| `:wat::core::if` | `":wat::core::if"` |
| `wat.core.Error/message` | `":wat::core::Error::message"` |
| `:wat::core::Error/message` | `":wat::core::Error/message"` |
| `wat.core.Option/expect` | `":wat::core::Option::expect"` |
| `:wat::core::Option/expect` | `":wat::core::Option/expect"` |
| `my.kernel` | `"my.kernel"` |
| `:my::kernel::` | `":my::kernel::"` |
| `usr.my-sift'/start` | `":usr::my-sift'::start"` |
| `:usr::my-sift'/start` | `":usr::my-sift'/start"` |
| `wat.core//` | `":wat::core::/"` |
| `:wat::core::/` | `":wat::core::/"` |

A name whose converted spelling and original keyword spelling print the same identity is in the "agrees" column. A name whose converted spelling prints a different identity is a mismatch. The counts are the scanner `/tmp/m6-scan.py` (and the stdlib continuation in the same functions) over keyword tokens outside strings and comments. The converted corpus files were the census copies; the stdlib files were the clone. Call-head sequences were zipped per file. **Unaligned call-head sequences: 0** in the 2217 OK corpus files and in all 65 stdlib files.

### Slash-less converted call heads

**0** in the OK corpus. **0** in the converted stdlib. A converted call head that the zip paired with a `::` keyword always still contained `/`.

Trailing-`::` markers become slash-less **namespace symbols**, and in this corpus they were not call heads. The census copies contain 27 such tokens, 10 spellings: `:my::` → `my`, `:my::admin::` → `my.admin`, `:my::auditor::` → `my.auditor`, `:my::internal::` → `my.internal`, `:my::issuer::` → `my.issuer`, `:my::kernel::` → `my.kernel`, `:my::test::` → `my.test`, `:probe::` → `probe`, `:test::` → `test`, `:user::` → `user`. Confirmed on disk: `tests/kernel/wat_arc198_def_restricted.wat` has `{:restricted-to [my.kernel]}`. The stdlib copies contain 8 marker tokens, 3 spellings: `:wat::kernel::` → `wat.kernel`, `:wat::spawn::` → `wat.spawn`, `:wat::test::` → `wat.test`. The door leaves `my.kernel` as `my.kernel` and leaves `:my::kernel::` as itself.

### Names the door maps to a different identity

A keyword that already contains `/` (a `Type/method` leaf) converts to `ns.Type/method`, and the door then joins the method with `::`.

| population | keyword tokens | identity agrees | `/` in the keyword, door disagrees | unique disagreeing spellings | markers |
|---|---:|---:|---:|---:|---:|
| 2217 OK corpus files | 116286 | 111036 | 5223 | 989 | 27 |
| 65 stdlib files | 14184 | 13027 | 1149 | 331 | 8 |

Witness, in the converted copy `crates/wat-edn/demo/probe-oneshot.wat`: the call is `(wat.core.Error/message __cause)`. The door prints `":wat::core::Error::message"` for that spelling and `":wat::core::Error/message"` for the keyword. Same shape for `:wat::core::Option/expect` → `wat.core.Option/expect` → door `:wat::core::Option::expect`. A primed name does it too: `:usr::my-sift'/start` → `usr.my-sift'/start` → door `:usr::my-sift'::start`.

### `DuplicateMacro` on `wat/holon/Ngram.wat`

The defmacro line converts from `(:wat::core::defmacro :wat::holon::Ngram` to `(wat.core/defmacro wat.holon/Ngram` (clone line 28). Four `--check` runs, copies at `/tmp/ngram-unconverted.wat` and `/tmp/ngram-converted.wat`:

| binary | file | RC | first stderr line |
|---|---|---:|---|
| pristine `target/release/wat` | unconverted | 0 | (empty) |
| pristine | converted | 1 | `#wat.macro/DuplicateMacro {:message "duplicate macro registration: :wat::holon::Ngram" ... :name ":wat::holon::Ngram"}` |
| clone `target/release/wat` (converted stdlib) | unconverted | 1 | `#wat.macro/DuplicateMacro {:message "duplicate macro registration: :wat::holon::Ngram" ... :name ":wat::holon::Ngram"}` |
| clone | converted | 1 | `#wat.type/UnknownNamedType {:message "not a member of wat.type: :wat::type::Error" ... :path ":wat::type::Error"}` |

The duplicate is symmetric across which side is converted, and both times the registered name in the message is `:wat::holon::Ngram`. The converted file against the converted stdlib dies at the same startup `UnknownNamedType` as the floor, before a duplicate can be reported. Logs: `/tmp/ngram-pristine-unconv.err`, `/tmp/ngram-pristine-conv.err`, `/tmp/ngram-clone-unconv.err`, `/tmp/ngram-clone-conv.err`.

## What this stone did not do

The main tree's `.wat` and `.rs` are the tree this score was drawn on. The floor in the clone was not re-run. The two census failures were not cured. 5c's conversion was not started.

## Appendix — per-file census

Source: `/tmp/census-5c/results.tsv`, the kept run. Columns: path, status, class, heads before, heads after, bytes changed (1 or 0), first error.

```
path	status	class	heads_before	heads_after	changed_bytes	first_error
benches/perf_arc278_fire_baseline.wat	OK	-	3	0	1	
crates/wat-edn/demo/probe-oneshot.wat	OK	-	12	0	1	
crates/wat-edn/demo/repl-daemon.wat	OK	-	14	0	1	
crates/wat-edn/wat-edn-clj/wat/shared.wat	OK	-	8	0	1	
docs/arc/2026/06/255-builtin-registry/probes-255.61/convert-doc-fragments.wat	OK	-	19	0	1	
docs/arc/2026/06/277-wat-lint-fix-fmt/probes/277-width-fixpoint-probe.wat	OK	-	46	0	1	
docs/arc/2026/06/278-rules-engine/harness-experiri/experiri-acc-head.wat	OK	-	21	0	1	
docs/arc/2026/06/278-rules-engine/harness-experiri/experiri-acc-wrapped.wat	OK	-	23	0	1	
docs/arc/2026/06/278-rules-engine/harness-experiri/experiri-then-match.wat	OK	-	9	0	1	
docs/arc/2026/06/278-rules-engine/harness-experiri/experiri-when-match.wat	OK	-	10	0	1	
docs/arc/2026/06/278-rules-engine/probes/enum-holds-record.wat	OK	-	9	0	1	
docs/arc/2026/06/278-rules-engine/probes/red-acc-refire-native-vs-oracle.wat	OK	-	69	0	1	
docs/arc/2026/06/278-rules-engine/probes/red-owner-signals-child.wat	OK	-	8	0	1	
docs/arc/2026/06/278-rules-engine/probes/red-send-cause-is-not-matchable.wat	OK	-	27	0	1	
docs/arc/2026/06/278-rules-engine/probes/surface-field-dispatch.wat	OK	-	13	0	1	
docs/arc/2026/06/278-rules-engine/strike-explain-drops-a-constraint/d6-explain-drops-enum-constraint.wat	OK	-	28	0	1	
docs/arc/2026/06/278-rules-engine/vigilia-2026-09-05/probes/probe_vig_explain_order.wat	OK	-	70	0	1	
docs/arc/2026/06/278-rules-engine/vigilia-2026-09-05/probes/probe_vig_left_idx_latch.wat	OK	-	91	0	1	
docs/arc/2026/06/278-rules-engine/vigilia-2026-09-05/probes/probe_vig_phantom_p0_real.wat	OK	-	2	0	1	
docs/arc/2026/06/278-rules-engine/vigilia-2026-09-05/probes/probe_vig_phantom_p1_unforced.wat	OK	-	3	0	1	
docs/arc/2026/06/278-rules-engine/vigilia-2026-09-05/probes/probe_vig_phantom_p2_forced.wat	OK	-	2	0	1	
docs/arc/2026/06/278-rules-engine/vigilia-2026-09-05/probes/probe_vig_phantom_p3_kernel_forced.wat	OK	-	6	0	1	
docs/arc/2026/06/278-rules-engine/vigilia-2026-09-05/probes/probe_vig_phantom_p4_kernel_untaken.wat	OK	-	5	0	1	
docs/arc/2026/06/278-rules-engine/vigilia-2026-09-05/probes/probe_vig_phantom_p5_unreserved.wat	OK	-	2	0	1	
docs/arc/2026/06/278-rules-engine/vigilia-2026-09-05/probes/probe_vig_phantom_p6_kernel_unforced.wat	OK	-	3	0	1	
docs/arc/2026/06/278-rules-engine/vigilia-2026-09-05/probes/probe_vig_retract_multiplicity.wat	OK	-	50	0	1	
examples/console-demo/wat/main.wat	OK	-	13	0	1	
examples/with-loader/wat-tests/helpers.wat	OK	-	1	0	1	
examples/with-loader/wat-tests/test-loader.wat	OK	-	4	0	1	
examples/with-loader/wat/deeper.wat	OK	-	1	0	1	
examples/with-loader/wat/helper.wat	OK	-	3	0	1	
examples/with-loader/wat/main.wat	OK	-	4	0	1	
tests/cli/grep_smoke_target.wat	OK	-	16	0	1	
tests/cli/metadata_of_example_formats.wat	OK	-	39	0	1	
tests/cli/mode_parity__deep_freeze_recursion.wat	OK	-	3005	0	1	
tests/cli/mode_parity__empty.wat	OK	-	0	0	0	
tests/cli/mode_parity__good.wat	OK	-	2	0	1	
tests/cli/pprintln_doc_row.wat	OK	-	16	0	1	
tests/cli/pprintln_doc_row_note.wat	OK	-	4	0	1	
tests/cli/synthetic_battery__alpha.wat	OK	-	1	0	1	
tests/cli/synthetic_battery__beta.wat	OK	-	1	0	1	
tests/cli/wat_cli__argv_passthrough.wat	OK	-	3	0	1	
tests/cli/wat_cli__bad_capacity_mode.wat	OK	-	1	0	1	
tests/cli/wat_cli__check_bad.wat	OK	-	2	0	1	
tests/cli/wat_cli__check_good.wat	OK	-	2	0	1	
tests/cli/wat_cli__check_types.wat	OK	-	5	0	1	
tests/cli/wat_cli__echo_program.wat	OK	-	7	0	1	
tests/cli/wat_cli__freeze_time_panic.wat	OK	-	7	0	1	
tests/cli/wat_cli__multiple_println.wat	OK	-	4	0	1	
tests/cli/wat_cli__presence_proof.wat	OK	-	18	0	1	
tests/cli/wat_cli__programs_are_atoms.wat	OK	-	8	0	1	
tests/cli/wat_cli__sigterm_blocked_on_stdin.wat	OK	-	10	0	1	
tests/cli/wat_cli__sigterm_polling_loop.wat	OK	-	8	0	1	
tests/cli/wat_cli__wrong_arg_type_main.wat	OK	-	1	0	1	
tests/cli/wat_fix_apply__liar.wat	OK	-	4	0	1	
tests/cli/wat_fix_apply__truthful.wat	OK	-	4	0	1	
tests/cli/wat_grep__any_symbol_rule.wat	OK	-	11	0	1	
tests/cli/wat_grep__balanced.wat	OK	-	3	0	1	
tests/cli/wat_grep__count_rules.wat	OK	-	22	0	1	
tests/cli/wat_grep__g7_rule.wat	OK	-	21	0	1	
tests/cli/wat_grep__g7_target.wat	OK	-	1	0	1	
tests/cli/wat_grep__no_reader_macros.wat	OK	-	2	0	1	
tests/cli/wat_grep__no_reader_macros_but_string.wat	OK	-	1	0	1	
tests/cli/wat_grep__sample_source.wat	OK	-	2	0	1	
tests/cli/wat_grep__with_tilde.wat	OK	-	2	0	1	
tests/cli/wat_repl__bad_then_good.wat	OK	-	2	0	1	
tests/cli/wat_repl__declare_only.wat	OK	-	2	0	1	
tests/cli/wat_repl__persist.wat	OK	-	3	0	1	
tests/cli/wat_repl__toplevel_expr.wat	OK	-	4	0	1	
tests/collection/bundle_capacity_accessors.wat	OK	-	406	0	1	
tests/collection/bundle_capacity_bad_return_type.wat	OK	-	4	0	1	
tests/collection/bundle_capacity_over_error.wat	OK	-	319	0	1	
tests/collection/bundle_capacity_over_panic.wat	OK	-	503	0	1	
tests/collection/bundle_capacity_try_propagate.wat	OK	-	408	0	1	
tests/collection/bundle_capacity_under_error.wat	OK	-	7	0	1	
tests/collection/bundle_capacity_under_panic.wat	OK	-	8	0	1	
tests/collection/list.wat	OK	-	39	0	1	
tests/collection/probe_arc215_collection_literal_inference.wat	OK	-	44	0	1	
tests/collection/probe_arc215_stone2.wat	OK	-	30	0	1	
tests/collection/probe_arc216_stone1_hashset_roundtrip.wat	OK	-	68	0	1	
tests/collection/probe_arc216_stone2_vector_roundtrip.wat	OK	-	94	0	1	
tests/collection/probe_arc216_stone3_hashmap_roundtrip.wat	OK	-	131	0	1	
tests/collection/probe_arc216_stone4_predicate_composition.wat	OK	-	20	0	1	
tests/collection/probe_arc216_stone5b_hashset_native_storage.wat	OK	-	76	0	1	
tests/collection/probe_arc216_stone5c_hashmap_native_storage.wat	OK	-	105	0	1	
tests/collection/probe_arc216_stone7_tuple_roundtrip.wat	OK	-	27	0	1	
tests/collection/probe_arc257_native_map_set.wat	OK	-	9	0	1	
tests/collection/probe_arc278_0a_persistent_map.wat	OK	-	21	0	1	
tests/collection/probe_arc278_0b_persistent_vector.wat	OK	-	18	0	1	
tests/collection/probe_arc278_0c_persistent_parity.wat	OK	-	36	0	1	
tests/collection/probe_arc278_0d_transform_dispatch_parity.wat	OK	-	58	0	1	
tests/collection/probe_brace_map_literal.wat	OK	-	18	0	1	
tests/collection/probe_collection_transform_ops.wat	OK	-	62	0	1	
tests/collection/probe_diagnostic_bundle_result_compose.wat	OK	-	38	0	1	
tests/collection/probe_hashmap_ctor_vector_symmetric.wat	OK	-	20	0	1	
tests/collection/probe_map_container.wat	OK	-	66	0	1	
tests/collection/probe_map_container_bad_assoc.wat	OK	-	2	0	1	
tests/collection/probe_nth.wat	OK	-	4	0	1	
tests/collection/probe_nth_persistent_vector.wat	OK	-	5	0	1	
tests/collection/probe_seq_container_parity.wat	OK	-	19	0	1	
tests/collection/probe_seq_container_registry.wat	OK	-	65	0	1	
tests/collection/probe_verify_hashset_of_vector_gap.wat	OK	-	3	0	1	
tests/collection/sort.wat	OK	-	41	0	1	
tests/collection/vector_algebra.wat	OK	-	41	0	1	
tests/collection/vector_first_class.wat	OK	-	93	0	1	
tests/collection/wat_arc167_vector_ast.wat	OK	-	2	0	1	
tests/collection/wat_dispatch_e1_vec.wat	OK	-	11	0	1	
tests/collection/wat_dispatch_e2_tuple.wat	OK	-	8	0	1	
tests/comms/probe_arc209_bound_listener.wat	OK	-	51	0	1	
tests/comms/probe_arc209_c0b1_thread_connection.wat	OK	-	44	0	1	
tests/comms/probe_arc209_c0b1b_select_listener.wat	OK	-	71	0	1	
tests/comms/probe_arc209_structured_peer_death.wat	OK	-	23	0	1	
tests/comms/probe_arc214_stone46b_select_prime.wat	OK	-	22	0	1	
tests/comms/probe_arc258_recv_infers_from_consumer.wat	OK	-	30	0	1	
tests/comms/probe_arc259_s2a_thread_self_peer.wat	OK	-	22	0	1	
tests/comms/probe_arc272_6a_capability_handoff.wat	OK	-	54	0	1	
tests/comms/probe_arc272_6c2_record_ipc_derisk.wat	OK	-	22	0	1	
tests/comms/probe_arc272_autobind_listener.wat	OK	-	16	0	1	
tests/comms/probe_arc278_accept_outcome_wall.wat	OK	-	28	0	1	
tests/comms/probe_arc278_connect_outcome_wall.wat	OK	-	17	0	1	
tests/comms/probe_arc278_failure_carries_structured_error.wat	OK	-	16	0	1	
tests/comms/probe_arc278_loci_died_error_round_trip.wat	OK	-	13	0	1	
tests/comms/probe_arc293_W2a_struct_no_cross.wat	OK	-	48	0	1	
tests/comms/probe_arc293_W2c_compile_time_send.wat	OK	-	4	0	1	
tests/comms/probe_arc293_W2c_controls.wat	OK	-	12	0	1	
tests/comms/probe_arc293_W2d_peer_purity.wat	OK	-	4	0	1	
tests/comms/probe_arc293_W2d_positive.wat	OK	-	3	0	1	
tests/comms/probe_arc293_W2e_address_wire.wat	OK	-	10	0	1	
tests/comms/probe_arc293_W2f_process_dials_thread.wat	OK	-	27	0	1	
tests/comms/probe_arc294_holon_wire_is_plain_edn.wat	OK	-	18	0	1	
tests/comms/probe_ioreader_read_frame.wat	OK	-	4	0	1	
tests/comms/probe_readln_max_buffer_kwarg.wat	OK	-	14	0	1	
tests/comms/probe_select_flood_no_deadlock.wat	OK	-	11	0	1	
tests/comms/wat_arc113_cross_fork_cascade.wat	OK	-	16	0	1	
tests/comms/wat_arc113_raise_round_trip.wat	OK	-	24	0	1	
tests/comms/wat_pipe.wat	OK	-	41	0	1	
tests/diagnostics/probe_arc242_stone2_value_position_doctrine_c02.wat	OK	-	1	0	1	
tests/diagnostics/probe_arc242_stone2_value_position_doctrine_c04.wat	OK	-	1	0	1	
tests/diagnostics/probe_arc242_stone2_value_position_doctrine_c06.wat	OK	-	2	0	1	
tests/diagnostics/probe_arc255_epprintln.wat	OK	-	3	0	1	
tests/diagnostics/probe_arc255_pprintln.wat	OK	-	3	0	1	
tests/diagnostics/probe_arc278_eprintln_terminal.wat	OK	-	10	0	1	
tests/diagnostics/probe_arc296_error_surface.wat	OK	-	11	0	1	
tests/diagnostics/probe_arc296_here.wat	OK	-	6	0	1	
tests/diagnostics/probe_arc296_pure_surface_field.wat	OK	-	2	0	1	
tests/diagnostics/probe_arc296_raise_gate.wat	OK	-	18	0	1	
tests/diagnostics/probe_arc296_record_in_surface_vector.wat	OK	-	6	0	1	
tests/diagnostics/probe_arc296_s7_ensure_reason_enum.wat	OK	-	3	0	1	
tests/diagnostics/probe_diagnostic_c3_macro_emits_record_def.wat	OK	-	20	0	1	
tests/diagnostics/probe_diagnostic_value_snapshot_in_errors_p1.wat	OK	-	2	0	1	
tests/diagnostics/probe_diagnostic_value_snapshot_in_errors_p2.wat	OK	-	3	0	1	
tests/diagnostics/probe_diagnostic_value_snapshot_in_errors_p3.wat	OK	-	2	0	1	
tests/diagnostics/probe_diagnostic_value_snapshot_in_errors_p4.wat	OK	-	3	0	1	
tests/diagnostics/probe_diagnostic_value_snapshot_in_errors_p7.wat	OK	-	5	0	1	
tests/diagnostics/probe_diagnostic_value_snapshot_in_errors_p8.wat	OK	-	3	0	1	
tests/diagnostics/probe_edn_write_unencodable_is_a_diagnostic.wat	OK	-	5	0	1	
tests/diagnostics/probe_no_default_rust_panic_noise_on_stderr.wat	OK	-	10	0	1	
tests/diagnostics/probe_plain_panic_produces_structured_edn.wat	OK	-	18	0	1	
tests/diagnostics/probe_runtime_err_stderr_visibility.wat	OK	-	10	0	1	
tests/diagnostics/probe_runtime_error_produces_structured_edn.wat	OK	-	11	0	1	
tests/diagnostics/refusals_teach_the_dot_separator_s1_not_namespaced.wat	OK	-	3	0	1	
tests/diagnostics/refusals_teach_the_dot_separator_s2_option_some.wat	OK	-	3	0	1	
tests/diagnostics/refusals_teach_the_dot_separator_s3_variant_parent_of.wat	OK	-	4	0	1	
tests/diagnostics/refusals_teach_the_dot_separator_s5_nested_map.wat	OK	-	4	0	1	
tests/diagnostics/refusals_teach_the_dot_separator_s6_keyword_subpattern.wat	OK	-	4	0	1	
tests/diagnostics/refusals_teach_the_dot_separator_s7_list_ctor.wat	OK	-	5	0	1	
tests/function/defn.wat	OK	-	38	0	1	
tests/function/defn_bad_type.wat	OK	-	1	0	1	
tests/function/defn_redef.wat	OK	-	3	0	1	
tests/function/fn_rename.wat	OK	-	31	0	1	
tests/function/fn_rename_bare_fn_type.wat	FAIL	other	3	3	0	[#wat.kernel/LociDiedError.RuntimeError {:message "#wat.runtime/MalformedForm {:message \"malformed :wat::keyword::to-type-form form: type-keyword parse failed: MalformedTypeExpr { raw: \\\":fn(wat::core::i64)->wat::core::i64\\\", reason: \\\"a keyword-bodied fn type is retired; write the bracket `[A :-> R]` (stone 251.4c). `:fn(A)->R` and `:wat::core::Fn(A)->R` no longer parse\\\" }\" :location #wat.core/Span {:file \"wat/fix.wat\" :line 362 :col 42 :end #wat.core/Option.Some {:value #wat.core/
tests/function/fn_rename_legacy_lambda.wat	OK	-	2	0	1	
tests/function/fn_rename_mixed_legacy.wat	FAIL	other	3	3	0	[#wat.kernel/LociDiedError.RuntimeError {:message "#wat.runtime/MalformedForm {:message \"malformed :wat::keyword::to-type-form form: type-keyword parse failed: MalformedTypeExpr { raw: \\\":fn(wat::core::i64)->wat::core::i64\\\", reason: \\\"a keyword-bodied fn type is retired; write the bracket `[A :-> R]` (stone 251.4c). `:fn(A)->R` and `:wat::core::Fn(A)->R` no longer parse\\\" }\" :location #wat.core/Span {:file \"wat/fix.wat\" :line 367 :col 44 :end #wat.core/Option.Some {:value #wat.core/
tests/function/fn_rename_multi_lambda.wat	OK	-	3	0	1	
tests/function/fn_signature.wat	OK	-	20	0	1	
tests/function/fn_signature_body_mismatch.wat	OK	-	2	0	1	
tests/function/fn_signature_malformed_args.wat	OK	-	2	0	1	
tests/function/probe_arc237_7a_length_intrinsic.wat	OK	-	12	0	1	
tests/function/probe_arc237_7b_intrinsic_typing.wat	OK	-	13	0	1	
tests/function/probe_arc237_7c_assoc_base_record.wat	OK	-	5	0	1	
tests/function/probe_arc237_7c_assoc_holonic_record.wat	OK	-	5	0	1	
tests/function/probe_arc237_7c_assoc_polymorphic.wat	OK	-	4	0	1	
tests/function/probe_arc237_8b_defclause_arithmetic.wat	OK	-	43	0	1	
tests/function/probe_arc237_8b_regression_cross_lt.wat	OK	-	2	0	1	
tests/function/probe_arc237_8b_regression_cross_plus.wat	OK	-	2	0	1	
tests/function/probe_arc237_stone2_defclause_substrate.wat	OK	-	33	0	1	
tests/function/probe_arc237_stone3_guard_ensure.wat	OK	-	52	0	1	
tests/function/probe_arc241_stone2_fn_parser_migration.wat	OK	-	10	0	1	
tests/function/probe_arc241_stone3_defclause_parser_migration.wat	OK	-	10	0	1	
tests/function/probe_arc241_stone5_c05.wat	OK	-	6	0	1	
tests/function/probe_arc241_stone5_defclause_rest_dispatch.wat	OK	-	20	0	1	
tests/function/probe_arc247_hof_fn_first.wat	OK	-	18	0	1	
tests/function/probe_arc255_67_wat_type_container_defclause_dispatch.wat	OK	-	7	0	1	
tests/function/probe_arc259_s2cii0_record_dispatch.wat	OK	-	5	0	1	
tests/function/probe_check_scoped_param_resolution_handwritten.wat	OK	-	1	0	1	
tests/function/probe_check_scoped_param_resolution_macro.wat	OK	-	3	0	1	
tests/function/probe_closure_body_prelude_lift_t1.wat	OK	-	16	0	1	
tests/function/probe_closure_body_prelude_lift_t2.wat	OK	-	19	0	1	
tests/function/probe_closure_body_prelude_lift_t3.wat	OK	-	16	0	1	
tests/function/probe_closure_body_prelude_lift_t4.wat	OK	-	22	0	1	
tests/function/probe_closure_body_prelude_lift_t5.wat	OK	-	16	0	1	
tests/function/probe_diagnostic_dynamic_keyword_invocation.wat	OK	-	26	0	1	
tests/function/probe_diagnostic_non_keyword.wat	OK	-	2	0	1	
tests/function/probe_diagnostic_non_vector.wat	OK	-	3	0	1	
tests/function/probe_stone255_69_wat_type_scalar_dispatch.wat	OK	-	3	0	1	
tests/function/probe_stone255_70_constructor_brackets_through_the_door.wat	OK	-	5	0	1	
tests/function/probe_stone255_71_template_typed.wat	OK	-	3	0	1	
tests/function/recursive_patterns_nonexhaustive.wat	OK	-	6	0	1	
tests/function/recursive_patterns_t1.wat	OK	-	8	0	1	
tests/function/recursive_patterns_t10.wat	OK	-	9	0	1	
tests/function/recursive_patterns_t2.wat	OK	-	7	0	1	
tests/function/recursive_patterns_t3.wat	OK	-	7	0	1	
tests/function/recursive_patterns_t4.wat	OK	-	6	0	1	
tests/function/recursive_patterns_t5.wat	OK	-	7	0	1	
tests/function/recursive_patterns_t6.wat	OK	-	7	0	1	
tests/function/recursive_patterns_t7.wat	OK	-	6	0	1	
tests/function/recursive_patterns_t9.wat	OK	-	6	0	1	
tests/function/stone18a.wat	OK	-	6	0	1	
tests/function/stone18a_e01.wat	OK	-	2	0	1	
tests/function/stone18a_e02.wat	OK	-	2	0	1	
tests/function/stone18a_e03.wat	OK	-	2	0	1	
tests/function/stone18a_e04.wat	OK	-	2	0	1	
tests/function/stone18a_e05.wat	OK	-	2	0	1	
tests/function/stone18a_e06.wat	OK	-	2	0	1	
tests/function/tco.wat	OK	-	102	0	1	
tests/function/variadic_define.wat	OK	-	36	0	1	
tests/function/variadic_define_amp_no_binder.wat	OK	-	1	0	1	
tests/function/variadic_define_arity_err.wat	OK	-	6	0	1	
tests/function/variadic_define_double_amp.wat	OK	-	1	0	1	
tests/function/variadic_define_fixed_after_rest.wat	OK	-	1	0	1	
tests/function/variadic_define_non_vector_rest.wat	OK	-	1	0	1	
tests/function/variadic_define_strict_extra_args.wat	OK	-	4	0	1	
tests/function/variadic_define_type_err.wat	OK	-	6	0	1	
tests/function/wat_arc170_closure_extraction_t1.wat	OK	-	2	0	1	
tests/function/wat_arc170_closure_extraction_t10.wat	OK	-	3	0	1	
tests/function/wat_arc170_closure_extraction_t11.wat	OK	-	3	0	1	
tests/function/wat_arc170_closure_extraction_t12.wat	OK	-	4	0	1	
tests/function/wat_arc170_closure_extraction_t13.wat	OK	-	6	0	1	
tests/function/wat_arc170_closure_extraction_t14.wat	OK	-	8	0	1	
tests/function/wat_arc170_closure_extraction_t15b.wat	OK	-	7	0	1	
tests/function/wat_arc170_closure_extraction_t16.wat	OK	-	3	0	1	
tests/function/wat_arc170_closure_extraction_t17.wat	OK	-	3	0	1	
tests/function/wat_arc170_closure_extraction_t18.wat	OK	-	3	0	1	
tests/function/wat_arc170_closure_extraction_t19.wat	OK	-	5	0	1	
tests/function/wat_arc170_closure_extraction_t2.wat	OK	-	5	0	1	
tests/function/wat_arc170_closure_extraction_t20.wat	OK	-	5	0	1	
tests/function/wat_arc170_closure_extraction_t21.wat	OK	-	4	0	1	
tests/function/wat_arc170_closure_extraction_t22.wat	OK	-	3	0	1	
tests/function/wat_arc170_closure_extraction_t3.wat	OK	-	8	0	1	
tests/function/wat_arc170_closure_extraction_t4.wat	OK	-	3	0	1	
tests/function/wat_arc170_closure_extraction_t5.wat	OK	-	7	0	1	
tests/function/wat_arc170_closure_extraction_t6.wat	OK	-	8	0	1	
tests/function/wat_arc170_closure_extraction_t7.wat	OK	-	8	0	1	
tests/function/wat_spawn_fn.wat	OK	-	72	0	1	
tests/kernel/peer_select_prime_process.wat	OK	-	28	0	1	
tests/kernel/peer_verb_round_trip_process.wat	OK	-	21	0	1	
tests/kernel/probe_arc109_assertion_kwargs__control_no_assertion.wat	OK	-	2	0	1	
tests/kernel/probe_arc109_assertion_kwargs__kwargs_all_three.wat	OK	-	2	0	1	
tests/kernel/probe_arc109_assertion_kwargs__kwargs_message_only.wat	OK	-	2	0	1	
tests/kernel/probe_arc109_assertion_kwargs__positional.wat	OK	-	2	0	1	
tests/kernel/probe_arc214_beta_forms_server.wat	OK	-	21	0	1	
tests/kernel/probe_arc255_29_address_wire.wat	OK	-	13	0	1	
tests/kernel/probe_arc255_29_shared_in_pure_record.wat	OK	-	2	0	1	
tests/kernel/probe_arc255_29_thread_address_to_process.wat	OK	-	67	0	1	
tests/kernel/probe_arc255_30_bracket_kwargs_pool.wat	OK	-	28	0	1	
tests/kernel/probe_arc255_30_pure_on_thread_peer.wat	OK	-	23	0	1	
tests/kernel/probe_arc255_30_thread_defservice.wat	OK	-	35	0	1	
tests/kernel/probe_arc255_34_connect_says_what_happened.wat	OK	-	35	0	1	
tests/kernel/probe_arc255_35_accept_says_what_happened.wat	OK	-	15	0	1	
tests/kernel/probe_arc255_36_send_says_what_happened.wat	OK	-	44	0	1	
tests/kernel/probe_arc259_bracket_runner_large_stream.wat	OK	-	26	0	1	
tests/kernel/probe_arc259_bracket_runner_stream_of_messages.wat	OK	-	32	0	1	
tests/kernel/probe_arc259_brackets_each_50_items.wat	OK	-	6	0	1	
tests/kernel/probe_arc259_brackets_each_small.wat	OK	-	5	0	1	
tests/kernel/probe_arc259_brackets_map_doubles.wat	OK	-	9	0	1	
tests/kernel/probe_arc259_brackets_map_small.wat	OK	-	5	0	1	
tests/kernel/probe_arc259_brackets_worker_each_drains.wat	OK	-	9	0	1	
tests/kernel/probe_arc259_brackets_worker_map_doubles.wat	OK	-	12	0	1	
tests/kernel/probe_arc259_brackets_worker_map_id.wat	OK	-	8	0	1	
tests/kernel/probe_arc259_deftest_hermetic_prime_failing.wat	OK	-	2	0	1	
tests/kernel/probe_arc259_deftest_hermetic_prime_passing.wat	OK	-	3	0	1	
tests/kernel/probe_arc259_deftest_prime_failing.wat	OK	-	2	0	1	
tests/kernel/probe_arc259_deftest_prime_passing.wat	OK	-	3	0	1	
tests/kernel/probe_arc259_s2ci_spawn_thread_prime.wat	OK	-	22	0	1	
tests/kernel/probe_arc259_s2cii_b_defclause.wat	OK	-	22	0	1	
tests/kernel/probe_arc259_s2d_raii_hinge_blocked.wat	OK	-	15	0	1	
tests/kernel/probe_arc259_s2d_raii_hinge_used.wat	OK	-	22	0	1	
tests/kernel/probe_arc259_started_at_boot.wat	OK	-	16	0	1	
tests/kernel/probe_arc275_verify_stdlib.wat	OK	-	27	0	1	
tests/kernel/probe_arc278_call_site.wat	OK	-	15	0	1	
tests/kernel/probe_arc278_close_outcome_wall.wat	OK	-	12	0	1	
tests/kernel/probe_deftest_verdict_wall.wat	OK	-	8	0	1	
tests/kernel/probe_string_to_lowercase.wat	OK	-	2	0	1	
tests/kernel/probe_time_duration_readout_across_units.wat	OK	-	3	0	1	
tests/kernel/probe_time_duration_readout_instant_delta.wat	OK	-	5	0	1	
tests/kernel/probe_time_duration_readout_same_unit.wat	OK	-	3	0	1	
tests/kernel/probe_time_duration_readout_truncates.wat	OK	-	3	0	1	
tests/kernel/spawn_program_prime_process.wat	OK	-	1	0	1	
tests/kernel/spawn_program_prime_process_echo.wat	OK	-	8	0	1	
tests/kernel/spawn_program_prime_process_empty_env.wat	OK	-	1	0	1	
tests/kernel/test_stdlib_load_order.wat	OK	-	3	0	1	
tests/kernel/wat_arc198_def_restricted.wat	OK	-	3	0	1	
tests/kernel/wat_arc198_def_restricted_bad_exact_fqdn_denied.wat	OK	-	3	0	1	
tests/kernel/wat_arc198_def_restricted_bad_outside_namespace.wat	OK	-	3	0	1	
tests/kernel/wat_arc198_def_restricted_bad_value_position_alias.wat	OK	-	3	0	1	
tests/kernel/wat_arc198_def_restricted_ok_exact_fqdn_allowed.wat	OK	-	3	0	1	
tests/kernel/wat_arc198_def_restricted_ok_multi_prefix.wat	OK	-	5	0	1	
tests/kernel/wat_arc251_8d_restricted_to_neither_errors.wat	OK	-	3	0	1	
tests/kernel/wat_arc251_8d_restricted_to_symbol_exact_denied.wat	OK	-	4	0	1	
tests/kernel/wat_arc251_8d_restricted_to_symbol_exact_ok.wat	OK	-	3	0	1	
tests/kernel/wat_arc251_8d_restricted_to_symbol_prefix_ok.wat	OK	-	3	0	1	
tests/kernel/wat_arc255_11_restricted_symbol_mention_in_data_denied.wat	OK	-	3	0	1	
tests/kernel/wat_arc255_11_restricted_symbol_mention_permitted_ok.wat	OK	-	3	0	1	
tests/kernel/wat_dispatch_193a.wat	OK	-	9	0	1	
tests/kernel/wat_dispatch_193b.wat	OK	-	12	0	1	
tests/kernel/wat_dispatch_e3_result.wat	OK	-	13	0	1	
tests/kernel/wat_dispatch_e4_shared.wat	OK	-	11	0	1	
tests/kernel/wat_dispatch_e5_owned_move.wat	OK	-	10	0	1	
tests/kernel/wat_engram_library.wat	OK	-	34	0	1	
tests/kernel/wat_harness_deps_dep_a.wat	OK	-	1	0	1	
tests/kernel/wat_harness_deps_dep_b.wat	OK	-	1	0	1	
tests/kernel/wat_harness_deps_user_main.wat	OK	-	3	0	1	
tests/kernel/wat_harness_trivial_main.wat	OK	-	3	0	1	
tests/kernel/wat_hermetic_round_trip.wat	OK	-	26	0	1	
tests/kernel/wat_io.wat	OK	-	94	0	1	
tests/kernel/wat_math_sqrt.wat	OK	-	11	0	1	
tests/kernel/wat_online_subspace.wat	OK	-	28	0	1	
tests/kernel/wat_reckoner.wat	OK	-	31	0	1	
tests/kernel/wat_run_sandboxed.wat	OK	-	95	0	1	
tests/kernel/wat_run_sandboxed_ast.wat	OK	-	25	0	1	
tests/kernel/wat_simhash.wat	OK	-	42	0	1	
tests/kernel/wat_stat.wat	OK	-	25	0	1	
tests/kernel/wat_string_ops.wat	OK	-	34	0	1	
tests/kernel/wat_u8.wat	OK	-	13	0	1	
tests/lint/probe_arc277_1b_ladder_autofix.wat	OK	-	3	0	1	
tests/lint/probe_arc277_1c_concat_format_autofix.wat	OK	-	6	0	1	
tests/lint/probe_arc277_1d_concat_fix_position_gate.wat	OK	-	3	0	1	
tests/lint/probe_arc277_lint_concat_abuse.wat	OK	-	3	0	1	
tests/lint/probe_arc277_lint_if_ladder.wat	OK	-	3	0	1	
tests/lint/tracked_wat_dir_is_stdlib_sources.wat	OK	-	5	0	1	
tests/macros/make_deftest.wat	OK	-	8	0	1	
tests/macros/probe_arc209_c1_defmacro_ast_walk.wat	OK	-	12	0	1	
tests/macros/probe_arc209_macro_span_fidelity.wat	OK	-	9	0	1	
tests/macros/probe_arc241_stone17_defmacro_canonical_c01.wat	OK	-	2	0	1	
tests/macros/probe_arc241_stone17_defmacro_canonical_c03.wat	OK	-	3	0	1	
tests/macros/probe_arc249_4_rehome_in_wat_canon_comp.wat	OK	-	10	0	1	
tests/macros/probe_arc249_4_rehome_in_wat_kw_of.wat	OK	-	14	0	1	
tests/macros/probe_arc249_4_rehome_in_wat_kw_of_tmpl.wat	OK	-	12	0	1	
tests/macros/probe_arc249_4_rehome_in_wat_kw_to_str.wat	OK	-	4	0	1	
tests/macros/probe_arc249_4_rehome_in_wat_vec_first.wat	OK	-	6	0	1	
tests/macros/probe_arc249_macro_engine_prog_fold.wat	OK	-	7	0	1	
tests/macros/probe_arc249_macro_engine_prog_if.wat	OK	-	8	0	1	
tests/macros/probe_arc249_macro_engine_regression.wat	OK	-	6	0	1	
tests/macros/probe_arc249_threading_bare_sym.wat	OK	-	5	0	1	
tests/macros/probe_arc249_threading_in_wat_head_first.wat	OK	-	6	0	1	
tests/macros/probe_arc249_threading_in_wat_is_list_no.wat	OK	-	6	0	1	
tests/macros/probe_arc249_threading_in_wat_is_list_yes.wat	OK	-	7	0	1	
tests/macros/probe_arc249_threading_in_wat_thread_first.wat	OK	-	11	0	1	
tests/macros/probe_arc249_threading_in_wat_tl_pipeline.wat	OK	-	12	0	1	
tests/macros/probe_arc249_threading_in_wat_tl_single.wat	OK	-	9	0	1	
tests/macros/probe_arc249_threading_regression.wat	OK	-	5	0	1	
tests/macros/probe_arc249_threading_tf_first.wat	OK	-	4	0	1	
tests/macros/probe_arc249_threading_tl_last.wat	OK	-	4	0	1	
tests/macros/probe_arc249_threading_tl_pipeline.wat	OK	-	9	0	1	
tests/macros/probe_arc249_threading_tl_single.wat	OK	-	6	0	1	
tests/macros/probe_arc249_threading_witness_tf_empty.wat	OK	-	2	0	1	
tests/macros/probe_arc249_threading_witness_tl_empty.wat	OK	-	2	0	1	
tests/macros/probe_arc251_8d_hygiene_fn_type_is_not_a_binder.wat	OK	-	6	0	1	
tests/macros/probe_arc251_8d_macro_member_join.wat	OK	-	5	0	1	
tests/macros/probe_arc251_8d_macro_member_join_control.wat	OK	-	6	0	1	
tests/macros/probe_arc255_11_hygiene_symbol_control.wat	OK	-	9	0	1	
tests/macros/probe_arc255_12_macro_cross_spelling.wat	OK	-	3	0	1	
tests/macros/probe_arc255_mirror_wall_control.wat	OK	-	3	0	1	
tests/macros/probe_arc258_stone2_cond_macro.wat	OK	-	10	0	1	
tests/macros/probe_arc258_stone2b_macro_error_c01.wat	OK	-	4	0	1	
tests/macros/probe_arc258_stone2b_macro_error_c02.wat	OK	-	4	0	1	
tests/macros/probe_arc258_stone2b_macro_error_c03.wat	OK	-	4	0	1	
tests/macros/probe_arc260_1b_call_sugar.wat	OK	-	17	0	1	
tests/macros/probe_arc260_decl_kwargs_minted_record.wat	OK	-	6	0	1	
tests/macros/probe_arc260_keyword_args.wat	OK	-	4	0	1	
tests/macros/probe_arc265_acronym_registry.wat	OK	-	11	0	1	
tests/macros/probe_arc265_acronym_registry_svc.wat	OK	-	14	0	1	
tests/macros/probe_arc274_fresh_symbol_no_capture.wat	OK	-	8	0	1	
tests/macros/probe_arc278_macro_call_site.wat	OK	-	11	0	1	
tests/macros/probe_arc278_macro_generates_service.wat	OK	-	16	0	1	
tests/macros/probe_arc279_format.wat	OK	-	2	0	1	
tests/macros/probe_arc279_format_missing_kwarg.wat	OK	-	2	0	1	
tests/macros/probe_arc279_format_unused_kwarg.wat	OK	-	2	0	1	
tests/macros/probe_arc279b_format_escape.wat	OK	-	6	0	1	
tests/macros/probe_arc279b_subs_tuple_macro_eval.wat	OK	-	35	0	1	
tests/macros/probe_argspec_rest_param_hygiene.wat	OK	-	9	0	1	
tests/macros/probe_declaration_form_lift.wat	OK	-	98	0	1	
tests/macros/probe_diagnostic_macro_splice_from_let_splice_i64.wat	OK	-	10	0	1	
tests/macros/probe_diagnostic_macro_splice_from_let_splice_watast.wat	OK	-	12	0	1	
tests/macros/probe_do_splice_def_defn_expand.wat	OK	-	4	0	1	
tests/macros/probe_do_splice_def_two_defs.wat	OK	-	6	0	1	
tests/macros/probe_do_splice_def_via_macro.wat	OK	-	6	0	1	
tests/macros/probe_do_splice_define_two_vars.wat	OK	-	4	0	1	
tests/macros/probe_do_splice_define_via_macro.wat	OK	-	6	0	1	
tests/macros/probe_do_splice_enum_constructor.wat	OK	-	4	0	1	
tests/macros/probe_do_splice_enum_via_macro.wat	OK	-	6	0	1	
tests/macros/probe_do_splice_struct_accessor.wat	OK	-	4	0	1	
tests/macros/probe_do_splice_struct_via_macro.wat	OK	-	6	0	1	
tests/macros/probe_hash_scope_renumber_alias.wat	OK	-	3	0	1	
tests/macros/probe_hash_scope_renumber_direct.wat	OK	-	1	0	1	
tests/macros/probe_kwargs_emitted_by_macro.wat	OK	-	17	0	1	
tests/macros/probe_kwargs_slash_name.wat	OK	-	8	0	1	
tests/macros/probe_let_splice_def_defn_expand.wat	OK	-	4	0	1	
tests/macros/probe_let_splice_def_real_bindings.wat	OK	-	5	0	1	
tests/macros/probe_let_splice_def_two_defs.wat	OK	-	6	0	1	
tests/macros/probe_let_splice_define_two_vars.wat	OK	-	4	0	1	
tests/macros/probe_let_splice_define_via_macro.wat	OK	-	6	0	1	
tests/macros/probe_let_splice_enum_constructor.wat	OK	-	4	0	1	
tests/macros/probe_let_splice_enum_via_macro.wat	OK	-	6	0	1	
tests/macros/probe_let_splice_struct_accessor.wat	OK	-	4	0	1	
tests/macros/probe_let_splice_struct_via_macro.wat	OK	-	6	0	1	
tests/macros/probe_macro_hygiene_capture.wat	OK	-	19	0	1	
tests/macros/probe_macros_unbounded_depth.wat	OK	-	9	0	1	
tests/macros/probe_register_types_splice_aware_do_enum.wat	OK	-	3	0	1	
tests/macros/probe_register_types_splice_aware_do_newtype.wat	OK	-	3	0	1	
tests/macros/probe_register_types_splice_aware_do_struct.wat	OK	-	4	0	1	
tests/macros/probe_register_types_splice_aware_do_typealias.wat	OK	-	3	0	1	
tests/macros/probe_register_types_splice_aware_let_typealias.wat	OK	-	2	0	1	
tests/macros/probe_register_types_splice_aware_nested_do.wat	OK	-	4	0	1	
tests/macros/probe_register_types_splice_aware_typealias_usage.wat	OK	-	3	0	1	
tests/macros/probe_resolver_quote_awareness_forms_data.wat	OK	-	5	0	1	
tests/macros/probe_resolver_quote_awareness_quasiquote.wat	OK	-	6	0	1	
tests/macros/probe_resolver_quote_awareness_quote_data.wat	OK	-	4	0	1	
tests/macros/variadic_defmacro.wat	OK	-	13	0	1	
tests/macros/variadic_defmacro_bad_arity.wat	OK	-	6	0	1	
tests/macros/variadic_defmacro_bad_double_rest.wat	OK	-	2	0	1	
tests/macros/variadic_defmacro_bad_rest_no_binder.wat	OK	-	3	0	1	
tests/macros/vector_splice_symmetry.wat	OK	-	23	0	1	
tests/process/arc112_scheme_probe.wat	OK	-	8	0	1	
tests/process/doomed_child_boot_ack_does_not_hang.wat	OK	-	1	0	1	
tests/process/doomed_child_boot_ack_does_not_hang__empty_env.wat	OK	-	1	0	1	
tests/process/fixtures/probe-bracket-cause.wat	OK	-	6	0	1	
tests/process/fixtures/probe-cap2-peer-pid.wat	OK	-	43	0	1	
tests/process/fixtures/probe-child-inherits-defns.wat	OK	-	44	0	1	
tests/process/fixtures/probe-defclause-discriminate.wat	OK	-	27	0	1	
tests/process/fixtures/probe-fnforms-keyword-err.wat	OK	-	5	0	1	
tests/process/fixtures/probe-generic-shipped.wat	OK	-	40	0	1	
tests/process/fixtures/probe-m1-addr-roundtrip.wat	OK	-	44	0	1	
tests/process/fixtures/probe-s1-impure-gate.wat	OK	-	12	0	1	
tests/process/fixtures/probe-s3b-crux-fnforms-closure.wat	OK	-	50	0	1	
tests/process/probe_arc209_structured_peer_death_process.wat	OK	-	20	0	1	
tests/process/probe_arc259_spawn_host_opts.wat	OK	-	4	0	1	
tests/process/probe_arc259_thread_crash_reason.wat	OK	-	10	0	1	
tests/process/probe_arc272_rs2_process_stop_returns_final_state.wat	OK	-	44	0	1	
tests/process/probe_arc272_rs2_thread_stop_returns_final_state.wat	OK	-	44	0	1	
tests/process/probe_arc278_init_crash_reason.wat	OK	-	54	0	1	
tests/process/probe_arc278_recv_over_budget_reason.wat	OK	-	10	0	1	
tests/process/probe_run_hermetic_ast_stdout_capture.wat	OK	-	13	0	1	
tests/process/probe_run_hermetic_no_deadlock.wat	OK	-	18	0	1	
tests/process/signal_kill_produces_close_outcome_signaled.wat	OK	-	13	0	1	
tests/process/wat_arc170_closure6_label_wall_labeled.wat	OK	-	34	0	1	
tests/process/wat_arc170_closure6_label_wall_unlabeled.wat	OK	-	21	0	1	
tests/program/probe_arc170_edn_bridge_unspellable__lexemes.wat	OK	-	7	0	1	
tests/program/probe_arc170_repl_freeze_partition__session.wat	OK	-	5	0	1	
tests/program/probe_arc211_program_env_ambient.wat	OK	-	20	0	1	
tests/program/probe_arc213_program_edn_roundtrip.wat	OK	-	8	0	1	
tests/program/probe_arc258_program_env_record.wat	OK	-	7	0	1	
tests/program/probe_arc259_cpu_count.wat	OK	-	12	0	1	
tests/program/probe_arc259_env_identity.wat	OK	-	12	0	1	
tests/program/probe_arc259_env_peer_kind.wat	OK	-	12	0	1	
tests/program/probe_arc259_peer_env_install.wat	OK	-	34	0	1	
tests/program/probe_arc259_program_cpu_count.wat	OK	-	2	0	1	
tests/program/probe_arc259_program_init_fn.wat	OK	-	53	0	1	
tests/program/probe_arc259_user_program_slot.wat	OK	-	13	0	1	
tests/program/wat_arc170_program_contracts.wat	OK	-	2	0	1	
tests/program/wat_arc170_program_contracts_t11_legacy_main.wat	OK	-	1	0	1	
tests/program/wat_arc170_program_contracts_t17_run_hermetic.wat	OK	-	14	0	1	
tests/program/wat_arc170_program_contracts_t17b_run_hermetic_fail.wat	OK	-	13	0	1	
tests/program/wat_arc170_program_contracts_t18_echo_doubled.wat	OK	-	19	0	1	
tests/program/wat_arc170_program_contracts_t18b_recv_assert_fail.wat	OK	-	17	0	1	
tests/program/wat_arc170_program_contracts_t18c_recv_all_multi.wat	OK	-	22	0	1	
tests/program/wat_arc170_program_contracts_t1_legacy_3arg.wat	OK	-	1	0	1	
tests/program/wat_arc170_program_contracts_t2_let.wat	OK	-	3	0	1	
tests/program/wat_arc170_program_contracts_t3_argv.wat	OK	-	3	0	1	
tests/program/wat_arc170_program_contracts_t5_launch_lambda.wat	OK	-	21	0	1	
tests/program/wat_arc170_program_contracts_t6_launch_factory.wat	OK	-	20	0	1	
tests/program/wat_arc170_slice_1e_user_main_nil.wat	OK	-	6	0	1	
tests/program/wat_arc170_slice_1e_user_main_nil_argv.wat	OK	-	3	0	1	
tests/program/wat_arc170_slice_1e_user_main_nil_new_spelling.wat	OK	-	2	0	1	
tests/program/wat_arc170_slice_1f_gamma_orchestrator.wat	OK	-	3	0	1	
tests/program/wat_arc170_slice_1f_gamma_orchestrator_row_a.wat	OK	-	2	0	1	
tests/program/wat_arc170_slice_1f_gamma_orchestrator_row_e.wat	OK	-	6	0	1	
tests/program/wat_arc278_sigma_fn_purity_gate_coincident_valid.wat	OK	-	2	0	1	
tests/program/wat_arc278_sigma_fn_purity_gate_presence_valid.wat	OK	-	2	0	1	
tests/reflection/probe_arc241_stone6_def_metadata_map_c01.wat	OK	-	1	0	1	
tests/reflection/probe_arc241_stone6_def_metadata_map_c02.wat	OK	-	1	0	1	
tests/reflection/probe_arc241_stone6_def_metadata_map_c03.wat	OK	-	2	0	1	
tests/reflection/probe_arc241_stone6_def_metadata_map_c04.wat	OK	-	1	0	1	
tests/reflection/probe_arc241_stone6_def_metadata_map_c05.wat	OK	-	1	0	1	
tests/reflection/probe_arc241_stone7_metadata_of_reflection_c01.wat	OK	-	4	0	1	
tests/reflection/probe_arc241_stone7_metadata_of_reflection_c02.wat	OK	-	5	0	1	
tests/reflection/probe_arc241_stone7_metadata_of_reflection_c03.wat	OK	-	4	0	1	
tests/reflection/probe_arc241_stone7_metadata_of_reflection_c04.wat	OK	-	4	0	1	
tests/reflection/probe_arc241_stone7_metadata_of_reflection_c05.wat	OK	-	3	0	1	
tests/reflection/probe_arc255_ivb1_structured_doc.wat	OK	-	3	0	1	
tests/reflection/probe_arc255_ivb2a_examples_seam.wat	OK	-	2	0	1	
tests/reflection/probe_arc255_ivb2b_verify_examples.wat	OK	-	2	0	1	
tests/reflection/probe_arc255_ivc_metadata_plain_values.wat	OK	-	3	0	1	
tests/reflection/probe_arc255_reflection_parity.wat	OK	-	6	0	1	
tests/reflection/probe_arc255_reflection_parity_user_form.wat	OK	-	4	0	1	
tests/reflection/probe_arc255_spec_complete.wat	OK	-	15	0	1	
tests/reflection/probe_arc255_spec_complete_yields_witness.wat	OK	-	4	0	1	
tests/reflection/probe_arc296_is_type.wat	OK	-	10	0	1	
tests/reflection/probe_arc296_reflection_answers_for_every_type_kind.wat	OK	-	6	0	1	
tests/reflection/probe_arc296_type_of_six_kinds.wat	OK	-	80	0	1	
tests/reflection/probe_diagnostic_polymorphic_type_p1.wat	OK	-	2	0	1	
tests/reflection/probe_diagnostic_polymorphic_type_p2.wat	OK	-	2	0	1	
tests/reflection/probe_diagnostic_polymorphic_type_p3.wat	OK	-	2	0	1	
tests/reflection/probe_diagnostic_polymorphic_type_p4.wat	OK	-	2	0	1	
tests/reflection/probe_diagnostic_polymorphic_type_p5.wat	OK	-	2	0	1	
tests/reflection/probe_diagnostic_polymorphic_type_p6.wat	OK	-	2	0	1	
tests/reflection/probe_diagnostic_polymorphic_type_p7.wat	OK	-	4	0	1	
tests/reflection/probe_diagnostic_polymorphic_type_p8.wat	OK	-	4	0	1	
tests/reflection/probe_diagnostic_typed_entities_reflection_p1.wat	OK	-	5	0	1	
tests/reflection/probe_diagnostic_typed_entities_reflection_p2.wat	OK	-	6	0	1	
tests/reflection/probe_diagnostic_typed_entities_reflection_p3.wat	OK	-	7	0	1	
tests/reflection/probe_diagnostic_typed_entities_reflection_p4.wat	OK	-	6	0	1	
tests/reflection/probe_diagnostic_typed_entities_reflection_p5.wat	OK	-	9	0	1	
tests/reflection/probe_diagnostic_typed_entities_reflection_p6.wat	OK	-	7	0	1	
tests/reflection/probe_diagnostic_typed_entities_reflection_p7.wat	OK	-	6	0	1	
tests/reflection/probe_stone_metadata_of_whole_row.wat	OK	-	9	0	1	
tests/reflection/probe_subtype_marker.wat	OK	-	6	0	1	
tests/reflection/wat_arc144_uniform_reflection_canary.wat	OK	-	3	0	1	
tests/reflection/wat_arc144_uniform_reflection_defn_head.wat	OK	-	5	0	1	
tests/reflection/wat_arc144_uniform_reflection_empty.wat	OK	-	4	0	1	
tests/reflection/wat_arc144_uniform_reflection_length_shape.wat	OK	-	6	0	1	
tests/reflection/wat_arc144_uniform_reflection_macro.wat	OK	-	4	0	1	
tests/reflection/wat_arc144_uniform_reflection_primitive.wat	OK	-	6	0	1	
tests/reflection/wat_arc144_uniform_reflection_sig_body.wat	OK	-	8	0	1	
tests/reflection/wat_arc144_uniform_reflection_special_form.wat	OK	-	4	0	1	
tests/reflection/wat_arc144_uniform_reflection_type.wat	OK	-	5	0	1	
tests/reflection/wat_arc201_extract_arg_types_arity.wat	OK	-	10	0	1	
tests/reflection/wat_arc201_extract_arg_types_atoms_len.wat	OK	-	7	0	1	
tests/reflection/wat_arc201_extract_arg_types_atoms_types.wat	OK	-	6	0	1	
tests/reflection/wat_arc201_extract_arg_types_bundles.wat	OK	-	6	0	1	
tests/reflection/wat_arc201_extract_arg_types_err_non_bundle.wat	OK	-	4	0	1	
tests/reflection/wat_arc201_holon_ast_accessors_children_err_atom.wat	OK	-	5	0	1	
tests/reflection/wat_arc201_holon_ast_accessors_children_parametric.wat	OK	-	11	0	1	
tests/reflection/wat_arc201_holon_ast_accessors_children_sig.wat	OK	-	9	0	1	
tests/reflection/wat_arc201_holon_ast_accessors_first_compose.wat	OK	-	10	0	1	
tests/reflection/wat_arc201_holon_ast_accessors_first_err_empty.wat	OK	-	7	0	1	
tests/reflection/wat_arc201_holon_ast_accessors_first_err_leaf.wat	OK	-	5	0	1	
tests/reflection/wat_arc201_holon_ast_accessors_first_head.wat	OK	-	10	0	1	
tests/reflection/wat_arc201_signature_of_fn_anon_head.wat	OK	-	6	0	1	
tests/reflection/wat_arc201_signature_of_fn_compose_bundle.wat	OK	-	6	0	1	
tests/reflection/wat_arc201_signature_of_fn_compose_names.wat	OK	-	6	0	1	
tests/reflection/wat_arc201_signature_of_fn_err_non_fn.wat	OK	-	4	0	1	
tests/reflection/wat_arc201_signature_of_fn_monomorphic_args.wat	OK	-	5	0	1	
tests/reflection/wat_arc201_signature_of_fn_parametric_args.wat	OK	-	5	0	1	
tests/reflection/wat_arc201_signature_of_fn_ret_parametric.wat	OK	-	5	0	1	
tests/reflection/wat_arc201_signature_of_fn_ret_path.wat	OK	-	5	0	1	
tests/reflection/wat_arc201_structured_signature_types_alias.wat	OK	-	9	0	1	
tests/reflection/wat_arc201_structured_signature_types_atomic_plus.wat	OK	-	4	0	1	
tests/reflection/wat_arc201_structured_signature_types_foldl.wat	OK	-	4	0	1	
tests/reflection/wat_arc201_structured_signature_types_parametric_fn.wat	OK	-	8	0	1	
tests/reflection/wat_arc201_structured_signature_types_tuple.wat	OK	-	5	0	1	
tests/resolve/probe_arc251_ast_span.wat	OK	-	37	0	1	
tests/resolve/probe_arc251_decl_migrator.wat	OK	-	139	0	1	
tests/resolve/probe_arc251_fix_macro_param_types.wat	OK	-	2	0	1	
tests/resolve/probe_arc251_fix_source_head_rule.wat	OK	-	23	0	1	
tests/resolve/probe_arc251_fix_source_local_rules.wat	OK	-	39	0	1	
tests/resolve/probe_arc251_fix_text_comment_faithful.wat	OK	-	7	0	1	
tests/resolve/probe_arc251_implicit_generics.wat	OK	-	11	0	1	
tests/resolve/probe_arc251_io_string_primitives.wat	OK	-	8	0	1	
tests/resolve/probe_arc251_io_write_forms.wat	OK	-	11	0	1	
tests/resolve/probe_arc251_keyword_to_symbol.wat	OK	-	40	0	1	
tests/resolve/probe_arc251_keyword_to_type_form.wat	OK	-	37	0	1	
tests/resolve/probe_arc251_ordering_surface.wat	OK	-	17	0	1	
tests/resolve/probe_arc251_parametric_target.wat	OK	-	2	0	1	
tests/resolve/probe_arc251_parametric_target_bad_new.wat	OK	-	1	0	1	
tests/resolve/probe_arc251_parametric_target_bad_old.wat	OK	-	1	0	1	
tests/resolve/probe_arc251_read_file_ladder.wat	OK	-	7	0	1	
tests/resolve/probe_arc251_read_file_ladder__content.wat	OK	-	1	0	1	
tests/resolve/probe_arc251_stone0_symbol_head.wat	OK	-	3	0	1	
tests/resolve/probe_arc251_stone2_type_namespace.wat	OK	-	10	0	1	
tests/resolve/probe_arc251_stone3_parametric_form.wat	OK	-	8	0	1	
tests/resolve/probe_arc251_stone4_annotation_arrow.wat	OK	-	4	0	1	
tests/resolve/probe_arc251_stone4b_ann_form.wat	OK	-	4	0	1	
tests/resolve/probe_arc251_stone4c_fn_type_arrow.wat	OK	-	4	0	1	
tests/resolve/probe_arc251_stone5a_ast_bridge.wat	OK	-	18	0	1	
tests/resolve/probe_arc251_stone5a_read_string.wat	OK	-	10	0	1	
tests/resolve/probe_arc251_stone5a_recognition.wat	OK	-	26	0	1	
tests/resolve/probe_arc251_stone5a_with_children.wat	OK	-	33	0	1	
tests/resolve/probe_arc251_stone5a_write_forms.wat	OK	-	11	0	1	
tests/resolve/probe_arc251_stone9__declares_kw.wat	OK	-	3	0	1	
tests/resolve/probe_arc251_stone9__declares_sym.wat	OK	-	2	0	1	
tests/resolve/probe_arc251_stone9__malformed_variant_kw.wat	OK	-	1	0	1	
tests/resolve/probe_arc251_stone9__malformed_variant_sym.wat	OK	-	0	0	1	
tests/resolve/probe_arc251_stone9__missing_purity_kw.wat	OK	-	1	0	1	
tests/resolve/probe_arc251_stone9__missing_purity_sym.wat	OK	-	0	0	1	
tests/resolve/probe_arc251_type_namespace_fix.wat	OK	-	60	0	1	
tests/resolve/probe_arc251_type_namespace_fix__c02-core-parametric.wat	OK	-	0	0	0	
tests/resolve/probe_arc255_5_position_signal.wat	OK	-	6	0	1	
tests/resolve/probe_arc255_82_bound_bare_head.wat	OK	-	0	0	1	
tests/resolve/probe_arc255_82_quasiquote_bare_head.wat	OK	-	0	0	1	
tests/resolve/probe_arc255_83_diff_expect_kw.wat	OK	-	3	0	1	
tests/resolve/probe_arc255_83_diff_expect_sym.wat	OK	-	0	0	1	
tests/resolve/probe_arc255_83_qq_preds.wat	OK	-	41	0	1	
tests/resolve/probe_arc255_83_qq_pure_kw.wat	OK	-	4	0	1	
tests/resolve/probe_arc255_83_qq_pure_sym.wat	OK	-	0	0	1	
tests/resolve/probe_arc255_83_wat_identities.wat	OK	-	91	0	1	
tests/resolve/probe_arc255_8_namespace_is_also_a_type.wat	OK	-	2	0	1	
tests/resolve/probe_arc255_register_variant_is_its_own_door__defn_then_variant.wat	OK	-	2	0	1	
tests/resolve/probe_arc255_register_variant_is_its_own_door__dotted_defn.wat	OK	-	1	0	1	
tests/resolve/probe_arc255_register_variant_is_its_own_door__dotted_variant.wat	OK	-	1	0	1	
tests/resolve/probe_arc255_register_variant_is_its_own_door__variant_then_defn.wat	OK	-	2	0	1	
tests/resolve/probe_arc255_the_blanket_hides_a_phantom_head__bogus_rete_head.wat	OK	-	4	0	1	
tests/resolve/probe_arc255_the_blanket_hides_a_phantom_head__colon_spelling_is_refused.wat	OK	-	3	0	1	
tests/resolve/probe_arc255_the_blanket_hides_a_phantom_head__dot_spelling_is_the_constructor.wat	OK	-	3	0	1	
tests/resolve/probe_arc255_the_reserved_prefix_wall_is_not_the_blanket__control_user_prefix.wat	OK	-	8	0	1	
tests/resolve/probe_arc255_the_reserved_prefix_wall_is_not_the_blanket__defenum_wat.wat	OK	-	3	0	1	
tests/resolve/probe_arc255_the_reserved_prefix_wall_is_not_the_blanket__defmacro_wat.wat	OK	-	4	0	1	
tests/resolve/probe_arc255_the_reserved_prefix_wall_is_not_the_blanket__defn_rust.wat	OK	-	3	0	1	
tests/resolve/probe_arc255_the_reserved_prefix_wall_is_not_the_blanket__defn_wat.wat	OK	-	3	0	1	
tests/resolve/probe_arc255_the_reserved_prefix_wall_is_not_the_blanket__defstruct_wat.wat	OK	-	3	0	1	
tests/resolve/probe_arc255_the_reserved_prefix_wall_is_not_the_blanket__typealias_wat.wat	OK	-	3	0	1	
tests/resolve/probe_arc255_the_type_position_has_its_own_authority__bogus_in_arg.wat	OK	-	3	0	1	
tests/resolve/probe_arc255_the_type_position_has_its_own_authority__bogus_in_defenum_field.wat	OK	-	3	0	1	
tests/resolve/probe_arc255_the_type_position_has_its_own_authority__bogus_in_defstruct_field.wat	OK	-	3	0	1	
tests/resolve/probe_arc255_the_type_position_has_its_own_authority__bogus_in_head.wat	OK	-	3	0	1	
tests/resolve/probe_arc255_the_type_position_has_its_own_authority__bogus_in_return.wat	OK	-	3	0	1	
tests/resolve/probe_arc255_the_type_position_has_its_own_authority__bogus_keyword_spelling.wat	OK	-	3	0	1	
tests/resolve/probe_arc255_the_type_position_has_its_own_authority__control_bogus_head_carrying_a_binder.wat	OK	-	2	0	1	
tests/resolve/probe_arc255_the_type_position_has_its_own_authority__control_boundary_head_with_a_binder.wat	OK	-	4	0	1	
tests/resolve/probe_arc255_the_type_position_has_its_own_authority__control_empty_list.wat	OK	-	2	0	1	
tests/resolve/probe_arc255_the_type_position_has_its_own_authority__control_infer_marker.wat	OK	-	3	0	1	
tests/resolve/probe_arc255_the_type_position_has_its_own_authority__control_is_type_canonicalizes.wat	OK	-	10	0	1	
tests/resolve/probe_arc255_the_type_position_has_its_own_authority__control_the_four_names.wat	OK	-	4	0	1	
tests/resolve/probe_arc255_the_type_position_has_its_own_authority__control_values_after_the_type_vector.wat	OK	-	4	0	1	
tests/resolve/probe_arc258_stone3_fix_source.wat	OK	-	106	0	1	
tests/resolve/probe_arc269_rename_keyword_prefix.wat	OK	-	2	0	1	
tests/resolve/probe_arc281_ast_end_span.wat	OK	-	11	0	1	
tests/resolve/probe_arc283_source_file_lift.wat	OK	-	3	0	1	
tests/resolve/probe_arc284_interpolate.wat	OK	-	17	0	1	
tests/resolve/probe_stone_233_2_j_producer_migration.wat	OK	-	2	0	1	
tests/rete/datamancer.src.wat	OK	-	156	0	1	
tests/rete/probe_arc251_8d_make_rule_quote_boundary.wat	OK	-	292	0	1	
tests/rete/probe_arc251_8d_purity_head_identity.wat	OK	-	59	0	1	
tests/rete/probe_arc251_8d_then_item_identity.wat	OK	-	181	0	1	
tests/rete/probe_arc251_8d_unquote_marker_identity.wat	OK	-	43	0	1	
tests/rete/probe_arc255_6_rete_adopts_the_door.wat	OK	-	8	0	1	
tests/rete/probe_arc255_7_the_validator_adopts_the_door.wat	OK	-	6	0	1	
tests/rete/probe_arc278_1a_data_model.wat	OK	-	19	0	1	
tests/rete/probe_arc278_1b_compile.wat	OK	-	19	0	1	
tests/rete/probe_arc278_2a_alpha_match.wat	OK	-	23	0	1	
tests/rete/probe_arc278_2b_insert_alpha.wat	OK	-	71	0	1	
tests/rete/probe_arc278_4a_production_fire.wat	OK	-	108	0	1	
tests/rete/probe_arc278_4b_cascade.wat	OK	-	94	0	1	
tests/rete/probe_arc278_4c_retraction.wat	OK	-	131	0	1	
tests/rete/probe_arc278_55_slice_one_undefined_mandatory.wat	OK	-	2	0	1	
tests/rete/probe_arc278_55_slice_one_vocabulary.wat	OK	-	108	0	1	
tests/rete/probe_arc278_59_tco_and_or_ann_form.wat	OK	-	61	0	1	
tests/rete/probe_arc278_5a_defrule_query_plain.wat	OK	-	45	0	1	
tests/rete/probe_arc278_5a_defrule_query_with_rule.wat	OK	-	43	0	1	
tests/rete/probe_arc278_5b_collect_rules.wat	OK	-	38	0	1	
tests/rete/probe_arc278_6a_purity.wat	OK	-	95	0	1	
tests/rete/probe_arc278_6b_eval_test.wat	OK	-	44	0	1	
tests/rete/probe_arc278_6b_ii_a_where_oracle_cmp.wat	OK	-	44	0	1	
tests/rete/probe_arc278_6b_ii_a_where_oracle_impure.wat	OK	-	27	0	1	
tests/rete/probe_arc278_6b_ii_a_where_oracle_userfn.wat	OK	-	46	0	1	
tests/rete/probe_arc278_7a_negation_oracle.wat	OK	-	51	0	1	
tests/rete/probe_arc278_7b_negation_native_differential.wat	OK	-	73	0	1	
tests/rete/probe_arc278_7exists_native_differential.wat	OK	-	140	0	1	
tests/rete/probe_arc278_7strat_native_differential.wat	OK	-	110	0	1	
tests/rete/probe_arc278_8i_accumulator_folds.wat	OK	-	158	0	1	
tests/rete/probe_arc278_D10_then_field_types_notknowable.wat	OK	-	90	0	1	
tests/rete/probe_arc278_D10_then_field_types_ok.wat	OK	-	64	0	1	
tests/rete/probe_arc278_D11_nested_then_field_types_notknowable.wat	OK	-	102	0	1	
tests/rete/probe_arc278_D11_nested_then_field_types_ok.wat	OK	-	104	0	1	
tests/rete/probe_arc278_D6_constraint_omission_tagged.wat	OK	-	29	0	1	
tests/rete/probe_arc278_D6_constraint_omission_unit.wat	OK	-	28	0	1	
tests/rete/probe_arc278_P12_explain_walk.wat	OK	-	57	0	1	
tests/rete/probe_arc278_P12a_explain_substrate.wat	OK	-	76	0	1	
tests/rete/probe_arc278_P12c_explain_payload.wat	OK	-	68	0	1	
tests/rete/probe_arc278_P2_native_fire_once.wat	OK	-	140	0	1	
tests/rete/probe_arc278_P4a_native_fire_rules.wat	OK	-	162	0	1	
tests/rete/probe_arc278_P4c_native_retraction.wat	OK	-	173	0	1	
tests/rete/probe_arc278_P6_delta_asymmetric_join.wat	OK	-	20	0	1	
tests/rete/probe_arc278_accessor_purity.wat	OK	-	18	0	1	
tests/rete/probe_arc278_accumulate_from_types.wat	OK	-	25	0	1	
tests/rete/probe_arc278_alpha_is_fire_scoped.wat	OK	-	71	0	1	
tests/rete/probe_arc278_ast_to_source.wat	OK	-	20	0	1	
tests/rete/probe_arc278_cache_lru.wat	OK	-	76	0	1	
tests/rete/probe_arc278_compiled_where_ops.wat	OK	-	9	0	1	
tests/rete/probe_arc278_concurrent_retes.wat	OK	-	125	0	1	
tests/rete/probe_arc278_d7_parametric_erasure_differential.wat	OK	-	161	0	1	
tests/rete/probe_arc278_derived_exists_acc.wat	OK	-	101	0	1	
tests/rete/probe_arc278_enum_variant_typo.wat	OK	-	30	0	1	
tests/rete/probe_arc278_enum_variant_typo_bad.wat	OK	-	30	0	1	
tests/rete/probe_arc278_enum_variant_typo_keyword.wat	OK	-	38	0	1	
tests/rete/probe_arc278_enum_variant_typo_tagged.wat	OK	-	32	0	1	
tests/rete/probe_arc278_explain_order.wat	OK	-	167	0	1	
tests/rete/probe_arc278_export.wat	OK	-	319	0	1	
tests/rete/probe_arc278_expr_ir.wat	OK	-	41	0	1	
tests/rete/probe_arc278_fallback_generic_ret.wat	OK	-	40	0	1	
tests/rete/probe_arc278_fence_binder_shadow.wat	OK	-	20	0	1	
tests/rete/probe_arc278_fence_hof.wat	OK	-	27	0	1	
tests/rete/probe_arc278_fence_interior_types.wat	OK	-	18	0	1	
tests/rete/probe_arc278_field_span_bind.wat	OK	-	7	0	1	
tests/rete/probe_arc278_field_span_inline.wat	OK	-	8	0	1	
tests/rete/probe_arc278_field_span_kwargs.wat	OK	-	7	0	1	
tests/rete/probe_arc278_field_span_nested.wat	OK	-	9	0	1	
tests/rete/probe_arc278_field_span_ok.wat	OK	-	30	0	1	
tests/rete/probe_arc278_fixpoint_round_cap.wat	OK	-	28	0	1	
tests/rete/probe_arc278_fixpoint_round_cap_boundary_fail.wat	OK	-	48	0	1	
tests/rete/probe_arc278_fixpoint_round_cap_boundary_pass.wat	OK	-	43	0	1	
tests/rete/probe_arc278_fixpoint_round_cap_deep.wat	OK	-	42	0	1	
tests/rete/probe_arc278_fn_headed_producer.wat	OK	-	40	0	1	
tests/rete/probe_arc278_foreign_pred_purity.wat	OK	-	38	0	1	
tests/rete/probe_arc278_fuzzer_found_divergences.wat	OK	-	145	0	1	
tests/rete/probe_arc278_import_accounting.wat	OK	-	19	0	1	
tests/rete/probe_arc278_import_accounting_ceiling.wat	OK	-	21	0	1	
tests/rete/probe_arc278_import_accounting_default.wat	OK	-	33	0	1	
tests/rete/probe_arc278_import_fold_key.wat	OK	-	136	0	1	
tests/rete/probe_arc278_inline_constraint_cross_type.wat	OK	-	29	0	1	
tests/rete/probe_arc278_inline_constraint_per_type.wat	OK	-	29	0	1	
tests/rete/probe_arc278_inline_constraint_untyped_equality.wat	OK	-	29	0	1	
tests/rete/probe_arc278_inline_constraint_untyped_ordering.wat	OK	-	29	0	1	
tests/rete/probe_arc278_insert_all_differential.wat	OK	-	133	0	1	
tests/rete/probe_arc278_insert_reports_the_verb.wat	OK	-	14	0	1	
tests/rete/probe_arc278_join_carries_both_sides_into_the_rhs.wat	OK	-	53	0	1	
tests/rete/probe_arc278_leading_filter_multiplicity.wat	OK	-	104	0	1	
tests/rete/probe_arc278_left_idx_latch.wat	OK	-	91	0	1	
tests/rete/probe_arc278_match_arm_body_ok.wat	OK	-	45	0	1	
tests/rete/probe_arc278_match_arm_then_core_bare.wat	OK	-	40	0	1	
tests/rete/probe_arc278_match_arm_then_rete_bare.wat	OK	-	40	0	1	
tests/rete/probe_arc278_match_arm_then_wrapped.wat	OK	-	40	0	1	
tests/rete/probe_arc278_native_insert_differential.wat	OK	-	92	0	1	
tests/rete/probe_arc278_nested_wall_arity.wat	OK	-	9	0	1	
tests/rete/probe_arc278_nested_wall_missing_fields.wat	OK	-	9	0	1	
tests/rete/probe_arc278_nested_wall_ok.wat	OK	-	29	0	1	
tests/rete/probe_arc278_nested_wall_positional_retired.wat	OK	-	9	0	1	
tests/rete/probe_arc278_nested_wall_unknown_field.wat	OK	-	9	0	1	
tests/rete/probe_arc278_northstar_cold_and_windy.wat	OK	-	32	0	1	
tests/rete/probe_arc278_not_and_derived.wat	OK	-	74	0	1	
tests/rete/probe_arc278_open_surface_dispatch.wat	OK	-	31	0	1	
tests/rete/probe_arc278_oracle_accumulate_supersedes.wat	OK	-	149	0	1	
tests/rete/probe_arc278_query_harvest_protocol.wat	OK	-	40	0	1	
tests/rete/probe_arc278_query_type_safe.wat	OK	-	60	0	1	
tests/rete/probe_arc278_reduce_arity_totality_three.wat	OK	-	29	0	1	
tests/rete/probe_arc278_reduce_arity_totality_two.wat	OK	-	29	0	1	
tests/rete/probe_arc278_rete_defn_gap_control.wat	OK	-	5	0	1	
tests/rete/probe_arc278_rete_defn_recurse_dag.wat	OK	-	6	0	1	
tests/rete/probe_arc278_rete_edn.wat	OK	-	73	0	1	
tests/rete/probe_arc278_return_type_of.wat	OK	-	12	0	1	
tests/rete/probe_arc278_rhs_unbound_span.wat	OK	-	24	0	1	
tests/rete/probe_arc278_session_ceiling_second_session.wat	OK	-	55	0	1	
tests/rete/probe_arc278_session_memory_ceiling.wat	OK	-	44	0	1	
tests/rete/probe_arc278_session_memory_ceiling_fire_default.wat	OK	-	36	0	1	
tests/rete/probe_arc278_session_memory_ceiling_insert.wat	OK	-	32	0	1	
tests/rete/probe_arc278_sieve_pred.wat	OK	-	26	0	1	
tests/rete/probe_arc278_smem_roundtrip.wat	OK	-	256	0	1	
tests/rete/probe_arc278_sqlite_interop.wat	OK	-	33	0	1	
tests/rete/probe_arc278_sqlite_store_differential.wat	OK	-	148	0	1	
tests/rete/probe_arc278_stratified_query_replay.wat	OK	-	66	0	1	
tests/rete/probe_arc278_termination_fence_bad_down.wat	OK	-	18	0	1	
tests/rete/probe_arc278_termination_fence_bad_up.wat	OK	-	18	0	1	
tests/rete/probe_arc278_termination_fence_ok_down.wat	OK	-	18	0	1	
tests/rete/probe_arc278_termination_finite_domain.wat	OK	-	27	0	1	
tests/rete/probe_arc278_termination_finite_domain_too_large.wat	OK	-	46	0	1	
tests/rete/probe_arc278_termination_fn_head.wat	OK	-	30	0	1	
tests/rete/probe_arc278_termination_guarded_counter.wat	OK	-	30	0	1	
tests/rete/probe_arc278_then_is_an_expansion_boundary.wat	OK	-	82	0	1	
tests/rete/probe_arc278_then_kwargs_positional.wat	OK	-	44	0	1	
tests/rete/probe_arc278_then_operand_rendered_as_source.wat	OK	-	25	0	1	
tests/rete/probe_arc278_then_user_forms_expr.wat	OK	-	37	0	1	
tests/rete/probe_arc278_then_user_forms_impure.wat	OK	-	19	0	1	
tests/rete/probe_arc278_then_user_forms_notfact.wat	OK	-	15	0	1	
tests/rete/probe_arc278_then_user_forms_userfn.wat	OK	-	45	0	1	
tests/rete/probe_arc278_two_where_native_spec.wat	OK	-	41	0	1	
tests/rete/probe_arc278_vsa_where_native_differential.wat	OK	-	234	0	1	
tests/rete/probe_arc278_where_is_positionally_free.wat	OK	-	101	0	1	
tests/rete/probe_arc300_2_fix_defrule.wat	OK	-	314	0	1	
tests/rete/probe_construction_headline_green.wat	OK	-	49	0	1	
tests/rete/probe_construction_headline_red.wat	OK	-	20	0	1	
tests/rete/probe_constructor_meta_enum_variant_green.wat	OK	-	51	0	1	
tests/rete/probe_constructor_meta_kwargs_full_green.wat	OK	-	53	0	1	
tests/rete/probe_constructor_meta_surface_pure_green.wat	OK	-	27	0	1	
tests/rete/probe_constructor_meta_surface_total_aggregate.wat	OK	-	51	0	1	
tests/rete/probe_enum_name.wat	OK	-	33	0	1	
tests/rete/probe_fence_names_the_head_acc_core_op.wat	OK	-	19	0	1	
tests/rete/probe_fence_names_the_head_acc_impure.wat	OK	-	20	0	1	
tests/rete/probe_fence_names_the_head_acc_partial.wat	OK	-	18	0	1	
tests/rete/probe_fence_names_the_head_core_op.wat	OK	-	26	0	1	
tests/rete/probe_fence_names_the_head_nondet.wat	OK	-	27	0	1	
tests/rete/probe_fence_names_the_head_partial.wat	OK	-	26	0	1	
tests/rete/probe_fence_names_the_head_then_core_op.wat	OK	-	14	0	1	
tests/rete/probe_fence_names_the_head_then_partial.wat	OK	-	14	0	1	
tests/rete/probe_freeze_validator_lift_rete_namespace.wat	OK	-	5	0	1	
tests/rete/probe_then_cond_still_works.wat	OK	-	32	0	1	
tests/rete/probe_then_match_is_refused.wat	OK	-	16	0	1	
tests/rete/probe_then_operand_fits_the_field_same_enum.wat	OK	-	6	0	1	
tests/services/probe_arc170_c1_kwargs_bracket.wat	OK	-	107	0	1	
tests/services/probe_arc170_c2_d_bodiless_edge_ok.wat	OK	-	26	0	1	
tests/services/probe_arc170_c2_mixed_macro.wat	OK	-	173	0	1	
tests/services/probe_arc170_c2_strike1_mixed.wat	OK	-	177	0	1	
tests/services/probe_arc170_gapj_each_kwargs.wat	OK	-	59	0	1	
tests/services/probe_arc170_m1_teeth_admitted.wat	OK	-	64	0	1	
tests/services/probe_arc170_m1_teeth_revoked.wat	OK	-	103	0	1	
tests/services/probe_arc170_stdio_prime.wat	OK	-	76	0	1	
tests/services/probe_arc170_thread_kwargs_process_svc.wat	OK	-	27	0	1	
tests/services/probe_arc170_w2a_kwargs_check_mint_ok.wat	OK	-	36	0	1	
tests/services/probe_arc170_wrong_service_compile_error_ok.wat	OK	-	26	0	1	
tests/services/probe_arc209_c0b3a0_self_peer.wat	OK	-	25	0	1	
tests/services/probe_arc209_c0b3aii_process_service_loop.wat	OK	-	60	0	1	
tests/services/probe_arc209_c0b3bb_bounced.wat	OK	-	60	0	1	
tests/services/probe_arc209_c0b3bb_bounced_bounced.wat	OK	-	82	0	1	
tests/services/probe_arc209_c0b3bb_verbs.wat	OK	-	7	0	1	
tests/services/probe_arc209_c0b3bb_verbs_thread.wat	OK	-	6	0	1	
tests/services/probe_arc209_c0b3bc_post_spawn.wat	OK	-	33	0	1	
tests/services/probe_arc209_c0b3bc_post_spawn_bogus_accessor.wat	OK	-	21	0	1	
tests/services/probe_arc209_c0b3bc_post_spawn_thread.wat	OK	-	31	0	1	
tests/services/probe_arc209_c0b3bd_user_program_foundation.wat	OK	-	7	0	1	
tests/services/probe_arc209_c0b3be_process_env_fn.wat	OK	-	3	0	1	
tests/services/probe_arc209_c0b3be_process_env_fn_bare_fn.wat	OK	-	2	0	1	
tests/services/probe_arc209_c0b3be_process_env_fn_default.wat	OK	-	1	0	1	
tests/services/probe_arc209_c0b3be_process_env_fn_named_call.wat	OK	-	1	0	1	
tests/services/probe_arc209_c0b3be_process_env_fn_non_record.wat	OK	-	1	0	1	
tests/services/probe_arc209_c1_defservice_op_enum.wat	OK	-	25	0	1	
tests/services/probe_arc209_c2_defservice_dispatch.wat	OK	-	74	0	1	
tests/services/probe_arc209_c3_defservice_client_face.wat	OK	-	52	0	1	
tests/services/probe_arc209_locus_agnostic_start.wat	OK	-	52	0	1	
tests/services/probe_arc209_locus_protocol_foundation.wat	OK	-	20	0	1	
tests/services/probe_arc209_naming_conversion.wat	OK	-	19	0	1	
tests/services/probe_arc209_naming_conversion_svc.wat	OK	-	34	0	1	
tests/services/probe_arc209_spawned_marker.wat	OK	-	8	0	1	
tests/services/probe_arc255_14_namespace_join.wat	OK	-	11	0	1	
tests/services/probe_arc255_21_coord_names_its_transport.wat	OK	-	35	0	1	
tests/services/probe_arc255_24_defservice_declares_what_it_emits.wat	OK	-	101	0	1	
tests/services/probe_arc255_27_kwargs_keeps_its_transport.wat	OK	-	19	0	1	
tests/services/probe_arc255_32_lost_arm_expansion.wat	OK	-	24	0	1	
tests/services/probe_arc255_32_two_clients_see_the_crash.wat	OK	-	59	0	1	
tests/services/probe_arc255_43_journal_ensure_schema.wat	OK	-	27	0	1	
tests/services/probe_arc255_44_span_reports_the_sink.wat	OK	-	64	0	1	
tests/services/probe_arc272_6b_defservice_on_process.wat	OK	-	52	0	1	
tests/services/probe_arc272_6b_state_over_lineage.wat	OK	-	72	0	1	
tests/services/probe_arc272_rs1_state_must_be_record.wat	OK	-	38	0	1	
tests/services/probe_arc272_rs1_state_must_be_record_holon.wat	OK	-	34	0	1	
tests/services/probe_arc272_rs1_state_must_be_record_type_keyword.wat	OK	-	6	0	1	
tests/services/probe_arc272_rs1_state_must_be_record_unknown_option.wat	OK	-	8	0	1	
tests/services/probe_arc272_rs2_crash_surfaces_to_client.wat	OK	-	30	0	1	
tests/services/probe_arc278_arming_is_internal_only_control.wat	OK	-	14	0	1	
tests/services/probe_arc278_call_context.wat	OK	-	175	0	1	
tests/services/probe_arc278_client_validates_locally.wat	OK	-	56	0	1	
tests/services/probe_arc278_dead_child_speaks.wat	OK	-	37	0	1	
tests/services/probe_arc278_emitted_from.wat	OK	-	12	0	1	
tests/services/probe_arc278_journal_backend_differential.wat	OK	-	61	0	1	
tests/services/probe_arc278_journal_logs_on_process.wat	OK	-	45	0	1	
tests/services/probe_arc278_journal_query.wat	OK	-	48	0	1	
tests/services/probe_arc278_journal_query_logs.wat	OK	-	53	0	1	
tests/services/probe_arc278_journal_query_metrics_on_process.wat	OK	-	40	0	1	
tests/services/probe_arc278_journal_service_logs.wat	OK	-	53	0	1	
tests/services/probe_arc278_journal_service_on_process.wat	OK	-	53	0	1	
tests/services/probe_arc278_journal_service_sqlite_on_process.wat	OK	-	53	0	1	
tests/services/probe_arc278_journal_surface.wat	OK	-	39	0	1	
tests/services/probe_arc278_log_captures_call_line.wat	OK	-	51	0	1	
tests/services/probe_arc278_mem_store_on_process.wat	OK	-	30	0	1	
tests/services/probe_arc278_metric_edn_write.wat	OK	-	6	0	1	
tests/services/probe_arc278_peers_bijection_case1_old_missing.wat	OK	-	70	0	1	
tests/services/probe_arc278_peers_bijection_case2_old_extra.wat	OK	-	70	0	1	
tests/services/probe_arc278_peers_bijection_case3_form_ok.wat	OK	-	70	0	1	
tests/services/probe_arc278_peers_bijection_case4_form_missing.wat	OK	-	70	0	1	
tests/services/probe_arc278_peers_bijection_case5_form_extra.wat	OK	-	70	0	1	
tests/services/probe_arc278_per_op_enforcement_codegen.wat	OK	-	65	0	1	
tests/services/probe_arc278_per_op_request_too_large.wat	OK	-	64	0	1	
tests/services/probe_arc278_recv_outcome_wall.wat	OK	-	156	0	1	
tests/services/probe_arc278_rst_peer_notify_baseline.wat	OK	-	26	0	1	
tests/services/probe_arc278_s2s_peer_on_process.wat	OK	-	73	0	1	
tests/services/probe_arc278_s2s_peer_on_thread.wat	OK	-	70	0	1	
tests/services/probe_arc278_self_scheduling.wat	OK	-	88	0	1	
tests/services/probe_arc278_service_max_frame_bytes.wat	OK	-	111	0	1	
tests/services/probe_arc278_sift_arena.wat	OK	-	184	0	1	
tests/services/probe_arc278_sift_logs.wat	OK	-	185	0	1	
tests/services/probe_arc278_sift_rules.wat	OK	-	256	0	1	
tests/services/probe_arc278_sift_rules_arena.wat	OK	-	398	0	1	
tests/services/probe_arc278_span_macros.wat	OK	-	48	0	1	
tests/services/probe_arc278_span_nested.wat	OK	-	52	0	1	
tests/services/probe_arc278_span_service.wat	OK	-	60	0	1	
tests/services/probe_arc278_span_surface.wat	OK	-	43	0	1	
tests/services/probe_arc278_tagged_keys_store.wat	OK	-	90	0	1	
tests/services/probe_arc293w_peer_derives_threadselfpeer.wat	OK	-	7	0	1	
tests/services/probe_arc294_expand_order.wat	OK	-	64	0	1	
tests/services/probe_mapv_side_effect_once.wat	OK	-	84	0	1	
tests/services/probe_thread_kwargs_thread_svc.wat	OK	-	27	0	1	
tests/types/enums_mixed_unit_tagged.wat	OK	-	12	0	1	
tests/types/enums_tagged_variant.wat	OK	-	11	0	1	
tests/types/enums_unit_variant.wat	OK	-	8	0	1	
tests/types/enums_wildcard_arm.wat	OK	-	5	0	1	
tests/types/newtype_as_struct_field_roundtrip.wat	OK	-	9	0	1	
tests/types/newtype_construct_and_accessor_roundtrip.wat	OK	-	6	0	1	
tests/types/ord_algebra_vector_distinct_order.wat	OK	-	9	0	1	
tests/types/ord_algebra_vector_ge_self.wat	OK	-	5	0	1	
tests/types/ord_algebra_vector_le_self.wat	OK	-	5	0	1	
tests/types/ord_algebra_vector_lt_self_false.wat	OK	-	5	0	1	
tests/types/ord_bytes_ge_prefix_tie.wat	OK	-	2	0	1	
tests/types/ord_bytes_gt.wat	OK	-	2	0	1	
tests/types/ord_bytes_le.wat	OK	-	2	0	1	
tests/types/ord_bytes_lt.wat	OK	-	2	0	1	
tests/types/ord_duration_ge.wat	OK	-	4	0	1	
tests/types/ord_duration_gt.wat	OK	-	4	0	1	
tests/types/ord_duration_le.wat	OK	-	4	0	1	
tests/types/ord_duration_lt.wat	OK	-	4	0	1	
tests/types/ord_instant_ge.wat	OK	-	4	0	1	
tests/types/ord_instant_gt.wat	OK	-	4	0	1	
tests/types/ord_instant_le.wat	OK	-	4	0	1	
tests/types/ord_instant_lt.wat	OK	-	4	0	1	
tests/types/ord_option_none_lt_some.wat	OK	-	4	0	1	
tests/types/ord_option_recursion_deep.wat	OK	-	7	0	1	
tests/types/ord_option_recursion_shallow.wat	OK	-	4	0	1	
tests/types/ord_option_some_ge_payload.wat	OK	-	4	0	1	
tests/types/ord_option_some_gt_none.wat	OK	-	4	0	1	
tests/types/ord_option_some_le_same.wat	OK	-	4	0	1	
tests/types/ord_result_err_ge_smaller.wat	OK	-	8	0	1	
tests/types/ord_result_err_lt_ok.wat	OK	-	11	0	1	
tests/types/ord_result_ok_gt_err.wat	OK	-	11	0	1	
tests/types/ord_result_ok_le_same.wat	OK	-	8	0	1	
tests/types/ord_result_recursion_deep.wat	OK	-	8	0	1	
tests/types/ord_result_recursion_shallow.wat	OK	-	8	0	1	
tests/types/ord_tuple_ge.wat	OK	-	2	0	1	
tests/types/ord_tuple_gt.wat	OK	-	2	0	1	
tests/types/ord_tuple_le_equal.wat	OK	-	2	0	1	
tests/types/ord_tuple_lt.wat	OK	-	2	0	1	
tests/types/ord_tuple_recursion_deep.wat	OK	-	2	0	1	
tests/types/ord_tuple_recursion_shallow.wat	OK	-	2	0	1	
tests/types/ord_vec_i64_gt.wat	OK	-	2	0	1	
tests/types/ord_vec_i64_lt.wat	OK	-	2	0	1	
tests/types/ord_vec_recursion_deep.wat	OK	-	2	0	1	
tests/types/ord_vec_recursion_shallow.wat	OK	-	2	0	1	
tests/types/ord_vec_string_ge_equal.wat	OK	-	2	0	1	
tests/types/ord_vec_string_le.wat	OK	-	2	0	1	
tests/types/parametric_enum_walk_visitor.wat	OK	-	12	0	1	
tests/types/parametric_enum_walkstep_continue.wat	OK	-	6	0	1	
tests/types/parametric_enum_walkstep_skip.wat	OK	-	8	0	1	
tests/types/probe_255_1_wat_type_membership.wat	OK	-	1	0	1	
tests/types/probe_arc118_2_lazy_map.wat	OK	-	10	0	1	
tests/types/probe_arc118_2z_takewhile_lazy.wat	OK	-	10	0	1	
tests/types/probe_arc118_lazy_seq.wat	OK	-	17	0	1	
tests/types/probe_arc170_parametric_surface.wat	OK	-	10	0	1	
tests/types/probe_arc170_parametric_surface_param_ok.wat	OK	-	18	0	1	
tests/types/probe_arc170_w3_n_dial_runner.wat	OK	-	64	0	1	
tests/types/probe_arc214_lexer_primed_generic_head_control.wat	OK	-	1	0	1	
tests/types/probe_arc214_lexer_primed_generic_head_primed.wat	OK	-	2	0	1	
tests/types/probe_arc214_stone46i_typed_peer_probe1.wat	OK	-	2	0	1	
tests/types/probe_arc214_stone46i_typed_peer_probe2.wat	OK	-	15	0	1	
tests/types/probe_arc214_stone46i_typed_peer_probe3.wat	OK	-	12	0	1	
tests/types/probe_arc226_stone1_type_predicates.wat	OK	-	105	0	1	
tests/types/probe_arc227_stone2_defrecord.wat	OK	-	207	0	1	
tests/types/probe_arc232_generic_method_type_application.wat	OK	-	2	0	1	
tests/types/probe_arc234_7a_base_record_roundtrip.wat	OK	-	11	0	1	
tests/types/probe_arc234_7b_holon_record_roundtrip.wat	OK	-	17	0	1	
tests/types/probe_arc234_stone15_namespace_promotion.wat	OK	-	2	0	1	
tests/types/probe_arc234_stone2a_record_primitives.wat	OK	-	18	0	1	
tests/types/probe_arc234_stone2b_defrecord_macro.wat	OK	-	42	0	1	
tests/types/probe_arc234_stone2c_accessor_class_safety.wat	OK	-	22	0	1	
tests/types/probe_arc234_stone3a_record_read_verbs.wat	OK	-	34	0	1	
tests/types/probe_arc234_stone3b_record_assoc.wat	OK	-	34	0	1	
tests/types/probe_arc234_stone3c_fix_narrow_fallthrough.wat	OK	-	11	0	1	
tests/types/probe_arc234_stone3c_keyword_accessor.wat	OK	-	18	0	1	
tests/types/probe_arc234_stone5_holon_auto_dispatch.wat	OK	-	37	0	1	
tests/types/probe_arc237_8a_no_implicit_coercion.wat	OK	-	12	0	1	
tests/types/probe_arc237_8a_no_implicit_coercion_arith_f64_i64.wat	OK	-	2	0	1	
tests/types/probe_arc237_8a_no_implicit_coercion_arith_i64_f64.wat	OK	-	2	0	1	
tests/types/probe_arc237_8a_no_implicit_coercion_cmp_i64_f64.wat	OK	-	2	0	1	
tests/types/probe_arc237_8c_equality_grid.wat	OK	-	18	0	1	
tests/types/probe_arc237_8c_equality_grid_cross_numeric.wat	OK	-	2	0	1	
tests/types/probe_arc237_8d_equality_intrinsic.wat	OK	-	45	0	1	
tests/types/probe_arc237_8d_equality_intrinsic_cross_numeric.wat	OK	-	2	0	1	
tests/types/probe_arc237_derive_verb.wat	OK	-	11	0	1	
tests/types/probe_arc237_s0_records_gate_struct.wat	OK	-	3	0	1	
tests/types/probe_arc237_s0_records_gate_typeunion.wat	OK	-	3	0	1	
tests/types/probe_arc237_sA1_assignable_probe01.wat	OK	-	5	0	1	
tests/types/probe_arc237_sA1_assignable_probe02.wat	OK	-	5	0	1	
tests/types/probe_arc237_sA1_assignable_probe04.wat	OK	-	5	0	1	
tests/types/probe_arc237_sA_hierarchy_probe07.wat	OK	-	2	0	1	
tests/types/probe_arc237_sA_hierarchy_probe08.wat	OK	-	2	0	1	
tests/types/probe_arc237_sA_hierarchy_probe09.wat	OK	-	2	0	1	
tests/types/probe_arc237_sA_hierarchy_probe10.wat	OK	-	2	0	1	
tests/types/probe_arc237_sB1_recordtype.wat	OK	-	14	0	1	
tests/types/probe_arc237_sB2_defrecord_recordtype.wat	OK	-	15	0	1	
tests/types/probe_arc237_sC2ab_field_order.wat	OK	-	14	0	1	
tests/types/probe_arc237_sC2d_same_data.wat	OK	-	32	0	1	
tests/types/probe_arc237_sC3_macro_split.wat	OK	-	54	0	1	
tests/types/probe_arc237_stone1_typeunion_substrate_probe11.wat	OK	-	1	0	1	
tests/types/probe_arc237_stone1_typeunion_substrate_probe12.wat	OK	-	6	0	1	
tests/types/probe_arc237_stone1_typeunion_substrate_probe14.wat	OK	-	8	0	1	
tests/types/probe_arc237_stone5_conforms.wat	OK	-	44	0	1	
tests/types/probe_arc237_stone5fix_nominal.wat	OK	-	37	0	1	
tests/types/probe_arc237_stone6_is_predicate.wat	OK	-	31	0	1	
tests/types/probe_arc241_stone8_defstruct_c01.wat	OK	-	1	0	1	
tests/types/probe_arc241_stone8_defstruct_c02.wat	OK	-	1	0	1	
tests/types/probe_arc241_stone8_defstruct_c03.wat	OK	-	1	0	1	
tests/types/probe_arc241_stone8_defstruct_c04.wat	OK	-	1	0	1	
tests/types/probe_arc241_stone8_defstruct_c05.wat	OK	-	1	0	1	
tests/types/probe_arc241_stone9_defenum_c01.wat	OK	-	1	0	1	
tests/types/probe_arc241_stone9_defenum_c02.wat	OK	-	1	0	1	
tests/types/probe_arc241_stone9_defenum_c03.wat	OK	-	1	0	1	
tests/types/probe_arc241_stone9_defenum_c04.wat	OK	-	1	0	1	
tests/types/probe_arc241_stone9_defenum_c08.wat	OK	-	2	0	1	
tests/types/probe_arc251_enrol_the_variant_in_the_lattice__alarm_sibling_to_variant.wat	OK	-	10	0	1	
tests/types/probe_arc251_enrol_the_variant_in_the_lattice__alarm_variant_to_enum.wat	OK	-	9	0	1	
tests/types/probe_arc251_enrol_the_variant_in_the_lattice__enum_to_variant.wat	OK	-	9	0	1	
tests/types/probe_arc251_enrol_the_variant_in_the_lattice__fn_narrow_param_for_wide_slot.wat	OK	-	12	0	1	
tests/types/probe_arc251_enrol_the_variant_in_the_lattice__fn_wide_param_for_narrow_slot.wat	OK	-	12	0	1	
tests/types/probe_arc251_enrol_the_variant_in_the_lattice__variant_to_enum.wat	OK	-	7	0	1	
tests/types/probe_arc251_enrol_the_variant_in_the_lattice__variant_to_variant.wat	OK	-	7	0	1	
tests/types/probe_arc251_instantiate_the_type_argument__enum_does_not_narrow.wat	OK	-	9	0	1	
tests/types/probe_arc251_instantiate_the_type_argument__non_parametric_record_keys.wat	OK	-	5	0	1	
tests/types/probe_arc251_instantiate_the_type_argument__non_parametric_variant_keys.wat	OK	-	5	0	1	
tests/types/probe_arc251_instantiate_the_type_argument__parametric_enum_via_match.wat	OK	-	6	0	1	
tests/types/probe_arc251_instantiate_the_type_argument__parametric_record_accessor.wat	OK	-	6	0	1	
tests/types/probe_arc251_instantiate_the_type_argument__parametric_record_keys.wat	OK	-	6	0	1	
tests/types/probe_arc251_instantiate_the_type_argument__parametric_struct_keys.wat	OK	-	6	0	1	
tests/types/probe_arc251_instantiate_the_type_argument__parametric_variant_accessor.wat	OK	-	6	0	1	
tests/types/probe_arc251_instantiate_the_type_argument__parametric_variant_accessor_runs.wat	OK	-	6	0	1	
tests/types/probe_arc251_instantiate_the_type_argument__parametric_variant_keys.wat	OK	-	6	0	1	
tests/types/probe_arc251_instantiate_the_type_argument__variant_widens_to_enum.wat	OK	-	7	0	1	
tests/types/probe_arc251_one_door_for_the_higher_order_fn_arg__filter_enum_elems_narrow.wat	OK	-	7	0	1	
tests/types/probe_arc251_one_door_for_the_higher_order_fn_arg__filter_variant_elems.wat	OK	-	7	0	1	
tests/types/probe_arc251_one_door_for_the_higher_order_fn_arg__foldl_variant_elems.wat	OK	-	7	0	1	
tests/types/probe_arc251_one_door_for_the_higher_order_fn_arg__map_variant_elems.wat	OK	-	7	0	1	
tests/types/probe_arc251_one_door_for_the_higher_order_fn_arg__mapv_output_concrete.wat	OK	-	7	0	1	
tests/types/probe_arc251_one_door_for_the_higher_order_fn_arg__mapv_variant_elems.wat	OK	-	7	0	1	
tests/types/probe_arc251_parametric_vector_field_is_checked.wat	OK	-	7	0	1	
tests/types/probe_arc251_type_the_polymorphic_accessor__correct_read_runs.wat	OK	-	5	0	1	
tests/types/probe_arc251_type_the_polymorphic_accessor__hashmap_receiver.wat	OK	-	5	0	1	
tests/types/probe_arc251_type_the_polymorphic_accessor__named_record_accessor_lie.wat	OK	-	5	0	1	
tests/types/probe_arc251_type_the_polymorphic_accessor__named_variant_accessor_lie.wat	OK	-	6	0	1	
tests/types/probe_arc251_type_the_polymorphic_accessor__named_variant_accessor_truth.wat	OK	-	6	0	1	
tests/types/probe_arc251_type_the_polymorphic_accessor__record_mono_lie.wat	OK	-	4	0	1	
tests/types/probe_arc251_type_the_polymorphic_accessor__record_mono_truth.wat	OK	-	4	0	1	
tests/types/probe_arc251_type_the_polymorphic_accessor__record_param_lie.wat	OK	-	5	0	1	
tests/types/probe_arc251_type_the_polymorphic_accessor__record_param_truth.wat	OK	-	5	0	1	
tests/types/probe_arc251_type_the_polymorphic_accessor__unknown_field.wat	OK	-	4	0	1	
tests/types/probe_arc251_type_the_polymorphic_accessor__variant_keys_lie.wat	OK	-	6	0	1	
tests/types/probe_arc251_type_the_polymorphic_accessor__variant_param_lie.wat	OK	-	5	0	1	
tests/types/probe_arc251_type_the_polymorphic_accessor__variant_param_truth.wat	OK	-	5	0	1	
tests/types/probe_arc255_12_coerce_and_dispatch.wat	OK	-	20	0	1	
tests/types/probe_arc255_12_redeclare_cross_spelling.wat	OK	-	9	0	1	
tests/types/probe_arc255_15_infer_transport_concrete_ok.wat	OK	-	18	0	1	
tests/types/probe_arc255_15_infer_transport_ok.wat	OK	-	42	0	1	
tests/types/probe_arc255_16_one_surface_one_binding_different_types.wat	OK	-	30	0	1	
tests/types/probe_arc255_16_one_surface_one_binding_identical.wat	OK	-	18	0	1	
tests/types/probe_arc255_17_last_type_argument_twins.wat	OK	-	35	0	1	
tests/types/probe_arc255_18_locus_names_its_transport_generic.wat	OK	-	17	0	1	
tests/types/probe_arc255_18_locus_names_its_transport_generic_reads_its_count.wat	OK	-	9	0	1	
tests/types/probe_arc255_18_locus_names_its_transport_process_yields_wire.wat	OK	-	8	0	1	
tests/types/probe_arc255_19_locus_methods_on_the_waist_start_process_claimed_wire.wat	OK	-	10	0	1	
tests/types/probe_arc255_19_locus_methods_on_the_waist_with_label_process_claimed_wire.wat	OK	-	12	0	1	
tests/types/probe_arc255_20_a_name_nothing_declares_declared_surface_member.wat	OK	-	3	0	1	
tests/types/probe_arc255_20_a_name_nothing_declares_registry_row_checks.wat	OK	-	2	0	1	
tests/types/probe_arc255_22_an_edge_declares_its_type_parameters_hello_Elem.wat	OK	-	11	0	1	
tests/types/probe_arc255_22_an_edge_declares_its_type_parameters_hello_T.wat	OK	-	11	0	1	
tests/types/probe_arc255_22_an_edge_declares_its_type_parameters_hello_mixed.wat	OK	-	11	0	1	
tests/types/probe_arc255_22_an_edge_declares_its_type_parameters_seqable_Elem.wat	OK	-	69	0	1	
tests/types/probe_arc255_25_transport_family_address_shared_field.wat	OK	-	2	0	1	
tests/types/probe_arc255_25_transport_family_address_wire_field.wat	OK	-	2	0	1	
tests/types/probe_arc255_25_transport_family_markers_are_pure.wat	OK	-	1	0	1	
tests/types/probe_arc255_25a_a_variant_is_a_type_when_its_enum_is_struct.wat	OK	-	10	0	1	
tests/types/probe_arc255_25a_a_variant_is_a_type_when_its_enum_is_variant.wat	OK	-	10	0	1	
tests/types/probe_arc255_28_purity_sees_through_a_generic_bare_shared_field.wat	OK	-	2	0	1	
tests/types/probe_arc255_28_purity_sees_through_a_generic_bare_shared_peer.wat	OK	-	4	0	1	
tests/types/probe_arc255_28_purity_sees_through_a_generic_generic_shared_field.wat	OK	-	4	0	1	
tests/types/probe_arc255_28_purity_sees_through_a_generic_generic_shared_peer.wat	OK	-	6	0	1	
tests/types/probe_arc255_28_purity_sees_through_a_generic_generic_wire_field.wat	OK	-	4	0	1	
tests/types/probe_arc255_28_purity_sees_through_a_generic_generic_wire_peer.wat	OK	-	6	0	1	
tests/types/probe_arc255_28_purity_sees_through_a_generic_self_referential.wat	OK	-	5	0	1	
tests/types/probe_arc255_28_purity_sees_through_a_generic_self_referential_shared.wat	OK	-	5	0	1	
tests/types/probe_arc255_28_purity_sees_through_a_generic_unbound_var.wat	OK	-	6	0	1	
tests/types/probe_arc255_28_purity_sees_through_a_generic_unbound_var_instantiated_shared.wat	OK	-	6	0	1	
tests/types/probe_arc255_39_edge_consumes.wat	OK	-	5	0	1	
tests/types/probe_arc255_48_featureless_surface.wat	OK	-	13	0	1	
tests/types/probe_arc255_51_binder_extend_bounded.wat	OK	-	2	0	1	
tests/types/probe_arc255_51_bounded_param.wat	OK	-	15	0	1	
tests/types/probe_arc255_52_conditional_membership.wat	OK	-	22	0	1	
tests/types/probe_arc255_53_tuple_member.wat	OK	-	25	0	1	
tests/types/probe_arc255_54_class_world.wat	OK	-	10	0	1	
tests/types/probe_arc255_54_classes.wat	OK	-	13	0	1	
tests/types/probe_arc255_55_newtype.wat	OK	-	27	0	1	
tests/types/probe_arc255_56_operators.wat	OK	-	19	0	1	
tests/types/probe_arc255_66_position.wat	OK	-	43	0	1	
tests/types/probe_arc255_67_cutover_types.wat	OK	-	88	0	1	
tests/types/probe_arc255_74_key_must_be_data__acceptance.wat	OK	-	10	0	1	
tests/types/probe_arc255_74_key_must_be_data__assoc_fn_key_into_empty_map.wat	OK	-	6	0	1	
tests/types/probe_arc255_74_key_must_be_data__conj_fn_into_empty_set.wat	OK	-	6	0	1	
tests/types/probe_arc255_74_key_must_be_data__control.wat	OK	-	2	0	1	
tests/types/probe_arc255_74_key_must_be_data__generic_bounded.wat	OK	-	5	0	1	
tests/types/probe_arc255_74_key_must_be_data__generic_unbounded.wat	OK	-	5	0	1	
tests/types/probe_arc255_74_key_must_be_data__hashmap_call_arg_fn_key.wat	OK	-	7	0	1	
tests/types/probe_arc255_74_key_must_be_data__hashmap_fn_key.wat	OK	-	5	0	1	
tests/types/probe_arc255_74_key_must_be_data__map_literal_fn_key.wat	OK	-	5	0	1	
tests/types/probe_arc255_74_key_must_be_data__persistentmap_fn_key.wat	OK	-	5	0	1	
tests/types/probe_arc255_74_key_must_be_data__set_literal_fn.wat	OK	-	5	0	1	
tests/types/probe_arc255_77_framing_floor_pin.wat	OK	-	14	0	1	
tests/types/probe_arc255_77_uuid_home_positions.wat	OK	-	15	0	1	
tests/types/probe_arc255_81_retired_name_refuses.wat	OK	-	12	0	1	
tests/types/probe_arc256_generic_defclause_c01.wat	OK	-	1	0	1	
tests/types/probe_arc256_generic_defclause_c02.wat	OK	-	3	0	1	
tests/types/probe_arc256_generic_defclause_c04.wat	OK	-	5	0	1	
tests/types/probe_arc256_generic_defclause_c05.wat	OK	-	3	0	1	
tests/types/probe_arc258_stone1_if_inference_c01.wat	OK	-	2	0	1	
tests/types/probe_arc258_stone1_if_inference_c02.wat	OK	-	2	0	1	
tests/types/probe_arc271_multi_param_generic_method.wat	OK	-	6	0	1	
tests/types/probe_arc278_capacity_derive.wat	OK	-	16	0	1	
tests/types/probe_arc278_opaque_purity_wall_control.wat	OK	-	6	0	1	
tests/types/probe_arc278_telemetry_records.wat	OK	-	9	0	1	
tests/types/probe_arc278_value_universal_top_widen.wat	OK	-	5	0	1	
tests/types/probe_arc283_1_rename_typearg.wat	OK	-	2	0	1	
tests/types/probe_arc293_4a_method_members.wat	OK	-	9	0	1	
tests/types/probe_arc293_4a_surface_method_member.wat	OK	-	8	0	1	
tests/types/probe_arc293_4b_surface_dispatch.wat	OK	-	20	0	1	
tests/types/probe_arc293_4c_extend_type_adapter.wat	OK	-	6	0	1	
tests/types/probe_arc293_4d_field_member_accessor.wat	OK	-	7	0	1	
tests/types/probe_arc293_4e_pre_ii_generic_surface_method.wat	OK	-	6	0	1	
tests/types/probe_arc293_4e_pre_iii_extend_impl_inherits_types.wat	OK	-	10	0	1	
tests/types/probe_arc293_4e_pre_surface_method_parity.wat	OK	-	8	0	1	
tests/types/probe_arc293_W2b_enum_purity.wat	OK	-	2	0	1	
tests/types/probe_arc293_W_containment.wat	OK	-	2	0	1	
tests/types/probe_arc293_acceptance_demo.wat	OK	-	39	0	1	
tests/types/probe_arc293_ctor_parity_newtype.wat	OK	-	3	0	1	
tests/types/probe_arc293_ctor_parity_struct.wat	OK	-	4	0	1	
tests/types/probe_arc293_decl_a_aggregatetype.wat	OK	-	5	0	1	
tests/types/probe_arc293_decl_b1_ctor_codegen.wat	OK	-	12	0	1	
tests/types/probe_arc293_defrecord_rename_core.wat	OK	-	5	0	1	
tests/types/probe_arc293_defrecord_rename_holon.wat	OK	-	5	0	1	
tests/types/probe_arc293_features_clause.wat	OK	-	8	0	1	
tests/types/probe_arc293_holder_bound_accept.wat	OK	-	6	0	1	
tests/types/probe_arc293_holder_bound_reject.wat	OK	-	6	0	1	
tests/types/probe_arc293_holder_ladder.wat	OK	-	15	0	1	
tests/types/probe_arc293_holder_ladder_foreign.wat	OK	-	6	0	1	
tests/types/probe_arc293_holder_root_symbol.wat	OK	-	7	0	1	
tests/types/probe_arc293_holder_substitution_c1.wat	OK	-	5	0	1	
tests/types/probe_arc293_holder_substitution_c2.wat	OK	-	5	0	1	
tests/types/probe_arc293_holder_substitution_c3.wat	OK	-	5	0	1	
tests/types/probe_arc293_k2_surface_record_emission.wat	OK	-	7	0	1	
tests/types/probe_arc293_k3_to_record.wat	OK	-	10	0	1	
tests/types/probe_arc293_k4_extend_type_own_aggregate.wat	OK	-	15	0	1	
tests/types/probe_arc293_k5_extend_surface.wat	OK	-	16	0	1	
tests/types/probe_arc293_r23_construction_parity.wat	OK	-	8	0	1	
tests/types/probe_arc293_r2_aggregate_codegen_parity.wat	OK	-	16	0	1	
tests/types/probe_arc293_record_surface_core.wat	OK	-	6	0	1	
tests/types/probe_arc293_record_surface_holon.wat	OK	-	6	0	1	
tests/types/probe_arc293_self_explicit.wat	OK	-	7	0	1	
tests/types/probe_arc293_struct_to_form_roundtrip.wat	OK	-	7	0	1	
tests/types/probe_arc293_structtype_primitive.wat	OK	-	7	0	1	
tests/types/probe_arc293_structural_surface.wat	OK	-	6	0	1	
tests/types/probe_arc293_surface_splice.wat	OK	-	12	0	1	
tests/types/probe_arc294_9a_kwargs_ctor.wat	OK	-	7	0	1	
tests/types/probe_arc294a_edn_measures_directly_map.wat	OK	-	4	0	1	
tests/types/probe_arc294a_edn_measures_directly_vec.wat	OK	-	4	0	1	
tests/types/probe_arc294c2a_aggregate_new.wat	OK	-	20	0	1	
tests/types/probe_arc296_A1_one_rule_for_assignability__if_branches_subtype_related.wat	OK	-	9	0	1	
tests/types/probe_arc296_A1_one_rule_for_assignability__if_branches_unrelated.wat	OK	-	8	0	1	
tests/types/probe_arc296_A1_one_rule_for_assignability__if_still_solves_a_type_var.wat	OK	-	7	0	1	
tests/types/probe_arc296_A1_one_rule_for_assignability__parameter_subsumes.wat	OK	-	8	0	1	
tests/types/probe_arc296_A2_a_variant_is_a_type__builder_program_runs.wat	OK	-	9	0	1	
tests/types/probe_arc296_A2_a_variant_is_a_type__ctor_carries_the_variant.wat	OK	-	8	0	1	
tests/types/probe_arc296_A2_a_variant_is_a_type__enum_does_not_narrow.wat	OK	-	9	0	1	
tests/types/probe_arc296_A2_a_variant_is_a_type__field_accessor_on_a_variant_runs.wat	OK	-	5	0	1	
tests/types/probe_arc296_A2_a_variant_is_a_type__intrinsic_param_accepts_a_variant.wat	OK	-	4	0	1	
tests/types/probe_arc296_A2_a_variant_is_a_type__match_still_works.wat	OK	-	7	0	1	
tests/types/probe_arc296_A2_a_variant_is_a_type__nested_variant_literal.wat	OK	-	8	0	1	
tests/types/probe_arc296_A2_a_variant_is_a_type__non_enum_container_stays_invariant.wat	OK	-	9	0	1	
tests/types/probe_arc296_A2_a_variant_is_a_type__nonexistent_variant.wat	OK	-	6	0	1	
tests/types/probe_arc296_A2_a_variant_is_a_type__process_full_box.wat	OK	-	6	0	1	
tests/types/probe_arc296_A2_a_variant_is_a_type__sibling_variants_join.wat	OK	-	7	0	1	
tests/types/probe_arc296_A2_a_variant_is_a_type__sibling_variants_join_in_match.wat	OK	-	8	0	1	
tests/types/probe_arc296_A2_a_variant_is_a_type__stdlib_enum_variant.wat	OK	-	7	0	1	
tests/types/probe_arc296_A2_a_variant_is_a_type__user_defn_param_accepts_a_variant.wat	OK	-	6	0	1	
tests/types/probe_arc296_A2_a_variant_is_a_type__variant_widens_to_enum.wat	OK	-	7	0	1	
tests/types/probe_arc296_enum_map_ctor__control.wat	OK	-	2	0	1	
tests/types/probe_arc296_enum_map_ctor__option_map.wat	OK	-	8	0	1	
tests/types/probe_arc296_enum_map_ctor__positional.wat	OK	-	8	0	1	
tests/types/probe_arc296_enum_map_ctor__user_map.wat	OK	-	8	0	1	
tests/types/probe_arc296_enum_map_ctor__user_unit_map.wat	OK	-	8	0	1	
tests/types/probe_arc296_h2__record.wat	OK	-	4	0	1	
tests/types/probe_arc296_h2__unit.wat	OK	-	4	0	1	
tests/types/probe_arc296_h2__unit_record.wat	OK	-	4	0	1	
tests/types/probe_arc296_h2__variant.wat	OK	-	4	0	1	
tests/types/probe_arc296_keys_on_aggregates__binder_first_control.wat	OK	-	7	0	1	
tests/types/probe_arc296_keys_on_aggregates__defholon.wat	OK	-	7	0	1	
tests/types/probe_arc296_keys_on_aggregates__defrecord.wat	OK	-	7	0	1	
tests/types/probe_arc296_keys_on_aggregates__defstruct.wat	OK	-	7	0	1	
tests/types/probe_arc296_keys_on_aggregates__holon_defrecord.wat	OK	-	7	0	1	
tests/types/probe_arc296_nature_roots.wat	OK	-	2	0	1	
tests/types/probe_arc296_nature_roots__record_is_not_a_struct.wat	OK	-	5	0	1	
tests/types/probe_arc296_nature_roots__struct_is_not_a_record.wat	OK	-	5	0	1	
tests/types/probe_arc296_p1_annotation_names_a_type__bare_legacy_primitive.wat	OK	-	4	0	1	
tests/types/probe_arc296_p1_annotation_names_a_type__builtin_and_generic_instantiation.wat	OK	-	7	0	1	
tests/types/probe_arc296_p1_annotation_names_a_type__control_declared_type.wat	OK	-	4	0	1	
tests/types/probe_arc296_p1_annotation_names_a_type__derive_marker_bound.wat	OK	-	5	0	1	
tests/types/probe_arc296_p1_annotation_names_a_type__generic_type_param.wat	OK	-	4	0	1	
tests/types/probe_arc296_p1_annotation_names_a_type__phantom_bare_uppercase_is_a_var.wat	OK	-	3	0	1	
tests/types/probe_arc296_p1_annotation_names_a_type__phantom_param_and_return.wat	OK	-	3	0	1	
tests/types/probe_arc296_p1_annotation_names_a_type__phantom_record_field.wat	OK	-	3	0	1	
tests/types/probe_arc296_p1_annotation_names_a_type__rust_without_use.wat	OK	-	3	0	1	
tests/types/probe_arc296_p1_annotation_names_a_type__use_rust_annotation.wat	OK	-	4	0	1	
tests/types/probe_arc296_p1b_a_parametric_head_is_a_named_type__bare_head_phantom.wat	OK	-	4	0	1	
tests/types/probe_arc296_p1b_a_parametric_head_is_a_named_type__parametric_arg_phantom.wat	OK	-	5	0	1	
tests/types/probe_arc296_p1b_a_parametric_head_is_a_named_type__parametric_head_phantom.wat	OK	-	5	0	1	
tests/types/probe_arc296_p1b_a_parametric_head_is_a_named_type__parametric_head_real.wat	OK	-	5	0	1	
tests/types/probe_arc296_p2a_a_monomorphic_variant_is_a_type__enum_does_not_flow_to_variant_param.wat	OK	-	7	0	1	
tests/types/probe_arc296_p2a_a_monomorphic_variant_is_a_type__generic_variant_stays_refused.wat	OK	-	4	0	1	
tests/types/probe_arc296_p2a_a_monomorphic_variant_is_a_type__is_type_on_a_variant.wat	OK	-	4	0	1	
tests/types/probe_arc296_p2a_a_monomorphic_variant_is_a_type__variant_annotation.wat	OK	-	5	0	1	
tests/types/probe_arc296_p2a_a_monomorphic_variant_is_a_type__variant_flows_to_enum_param.wat	OK	-	6	0	1	
tests/types/probe_arc296_p2prereq_is_type_asks_the_same_union__handlist_control.wat	OK	-	3	0	1	
tests/types/probe_arc296_p2prereq_is_type_asks_the_same_union__no_use_is_not_a_type.wat	OK	-	3	0	1	
tests/types/probe_arc296_p2prereq_is_type_asks_the_same_union__phantom_rust_name.wat	OK	-	3	0	1	
tests/types/probe_arc296_p2prereq_is_type_asks_the_same_union__use_then_is_type.wat	OK	-	4	0	1	
tests/types/probe_arc296_p3_one_question_one_answer__is_type_on_a_derive_marker.wat	OK	-	3	0	1	
tests/types/probe_arc296_p3_one_question_one_answer__is_type_on_a_non_marker.wat	OK	-	3	0	1	
tests/types/probe_arc296_p3_one_question_one_answer__stdlib_annotation_still_loads.wat	OK	-	2	0	1	
tests/types/probe_arc296_p3_one_question_one_answer__user_annotation_with_user_use.wat	OK	-	5	0	1	
tests/types/probe_arc296_p3_one_question_one_answer__user_annotation_without_user_use.wat	OK	-	4	0	1	
tests/types/probe_diagnostic_defprotocol_dispatch_p1.wat	OK	-	18	0	1	
tests/types/probe_diagnostic_defprotocol_dispatch_p2.wat	OK	-	11	0	1	
tests/types/probe_diagnostic_defprotocol_dispatch_p3.wat	OK	-	10	0	1	
tests/types/probe_first_bare_accessors_first_empty.wat	OK	-	2	0	1	
tests/types/probe_first_bare_accessors_first_list.wat	OK	-	2	0	1	
tests/types/probe_first_bare_accessors_first_persistent_vector.wat	OK	-	2	0	1	
tests/types/probe_first_bare_accessors_first_tuple.wat	OK	-	2	0	1	
tests/types/probe_first_bare_accessors_first_vector.wat	OK	-	2	0	1	
tests/types/probe_first_bare_accessors_third_vector.wat	OK	-	2	0	1	
tests/types/probe_stone118_3b_seqable_parametric_satisfaction.wat	OK	-	38	0	1	
tests/types/probe_stone_118_11a_next.wat	OK	-	30	0	1	
tests/types/probe_stone_118_b2c_surface_arm_never_dispatches.wat	OK	-	36	0	1	
tests/types/probe_stone_118_b2c_unreachable_arm_refused.wat	OK	-	12	0	1	
tests/types/probe_stone_118_b2d_generic_satisfier.wat	OK	-	8	0	1	
tests/types/probe_stone_118_b2d_generic_satisfier_pos.wat	OK	-	14	0	1	
tests/types/probe_stone_binder_is_universal.wat	OK	-	18	0	1	
tests/types/probe_stone_guard_the_peel_point.wat	OK	-	11	0	1	
tests/types/struct_destructure_field_order.wat	OK	-	4	0	1	
tests/types/struct_destructure_hyphenated_field.wat	OK	-	4	0	1	
tests/types/struct_destructure_mixed_bindings.wat	OK	-	4	0	1	
tests/types/struct_destructure_multi_field.wat	OK	-	4	0	1	
tests/types/struct_destructure_multi_form_body.wat	OK	-	7	0	1	
tests/types/struct_destructure_nested_let.wat	OK	-	6	0	1	
tests/types/struct_destructure_single_field.wat	OK	-	4	0	1	
tests/types/struct_restricted_all_restricted.wat	OK	-	5	0	1	
tests/types/struct_restricted_ctor_only.wat	OK	-	5	0	1	
tests/types/struct_restricted_field_allowed.wat	OK	-	5	0	1	
tests/types/struct_restricted_public_accessor.wat	OK	-	5	0	1	
tests/types/struct_restricted_whitelist.wat	OK	-	7	0	1	
tests/types/struct_restricted_whitelisted_caller_both_routes.wat	OK	-	5	0	1	
tests/types/structs_builtin_capacity_exceeded.wat	OK	-	6	0	1	
tests/types/structs_ctor_accessor_roundtrip.wat	OK	-	7	0	1	
tests/types/structs_heterogeneous_fields.wat	OK	-	5	0	1	
tests/types/structs_survive_rebinding.wat	OK	-	7	0	1	
tests/types/structs_user_method_auto_accessors.wat	OK	-	9	0	1	
tests/types/tuple_in_return_position.wat	OK	-	3	0	1	
tests/types/tuple_pascal_canonical.wat	OK	-	1	0	1	
tests/types/tuple_type_name_fqdn_pascal.wat	OK	-	2	0	1	
tests/types/typealias_alias_chain.wat	OK	-	6	0	1	
tests/types/typealias_hashmap_args.wat	OK	-	6	0	1	
tests/types/typealias_hashmap_std_get.wat	OK	-	5	0	1	
tests/types/typealias_return_type.wat	OK	-	4	0	1	
tests/types/typealias_simple_alias.wat	OK	-	5	0	1	
tests/types/typed_if_match__if_wrong_arity_needle.wat	OK	-	0	0	0	
tests/types/typed_if_match_if_false.wat	OK	-	2	0	1	
tests/types/typed_if_match_if_inside_match.wat	OK	-	5	0	1	
tests/types/typed_if_match_if_result_in_let.wat	OK	-	3	0	1	
tests/types/typed_if_match_if_true.wat	OK	-	2	0	1	
tests/types/typed_if_match_match_none.wat	OK	-	3	0	1	
tests/types/typed_if_match_match_result_in_let.wat	OK	-	4	0	1	
tests/types/typed_if_match_match_some.wat	OK	-	3	0	1	
tests/types/typed_if_match_untyped_if_bare.wat	OK	-	2	0	1	
tests/types/typed_if_match_untyped_match_bare.wat	OK	-	3	0	1	
tests/types/uuid_edn_roundtrip_typed.wat	OK	-	9	0	1	
tests/types/uuid_edn_write_reader_literal.wat	OK	-	9	0	1	
tests/types/uuid_eq_uses_values_equal_arm.wat	OK	-	8	0	1	
tests/types/uuid_equality_v4_differ_v5_equal.wat	OK	-	16	0	1	
tests/types/uuid_from_string_braced.wat	OK	-	6	0	1	
tests/types/uuid_from_string_garbage.wat	OK	-	6	0	1	
tests/types/uuid_from_string_nil_str.wat	OK	-	6	0	1	
tests/types/uuid_from_string_upper.wat	OK	-	6	0	1	
tests/types/uuid_from_string_urn.wat	OK	-	6	0	1	
tests/types/uuid_from_string_valid.wat	OK	-	6	0	1	
tests/types/uuid_nil_is_zero.wat	OK	-	8	0	1	
tests/types/uuid_string_not_equal_typed.wat	OK	-	11	0	1	
tests/types/uuid_to_string_roundtrip.wat	OK	-	18	0	1	
tests/types/uuid_v4_returns_typed_uuid.wat	OK	-	9	0	1	
tests/types/uuid_v5_with_typed_namespace.wat	OK	-	16	0	1	
tests/value/probe_299_time_measure.wat	OK	-	11	0	1	
tests/value/probe_299_uuid_v4_measure.wat	OK	-	7	0	1	
tests/value/probe_arc238_eq_completeness.wat	OK	-	21	0	1	
tests/value/probe_arc242_stone1_lexeme_role.wat	OK	-	3	0	1	
tests/value/probe_arc278_read_foreign.wat	OK	-	24	0	1	
tests/value/probe_arc279_str_totality.wat	OK	-	20	0	1	
tests/value/probe_arc294_holon_bare_leaf_read.wat	OK	-	20	0	1	
tests/value/probe_eval_signature_returns_tracked_value.wat	OK	-	5	0	1	
tests/value/probe_int_modrem.wat	OK	-	30	0	1	
tests/value/probe_rational_C5_mixed_compare.wat	OK	-	52	0	1	
tests/value/probe_rational_C5c_nan_unordered.wat	OK	-	73	0	1	
tests/value/probe_stone_233_2_e_ast_derived_provenance.wat	OK	-	6	0	1	
tests/value/probe_stone_D_join_over_seqable.wat	OK	-	15	0	1	
tests/value/wat_arc220_char.wat	OK	-	35	0	1	
tests/value/wat_arc221_char_atomization.wat	OK	-	47	0	1	
tests/value/wat_arc221_keyword_nil_tag_atomization.wat	OK	-	90	0	1	
tests/value/wat_arc221b_keyword_dispatcher_completeness.wat	OK	-	59	0	1	
tests/value/wat_arc221b_macro_support_keyword_shape.wat	OK	-	21	0	1	
tests/value/wat_eval_result.wat	OK	-	38	0	1	
tests/value/wat_eval_result_wrong_arity.wat	OK	-	3	0	1	
tests/value/wat_names_are_values.wat	OK	-	28	0	1	
tests/wat_lang/probe_arc109_match_arm__declared_order.wat	OK	-	7	0	1	
tests/wat_lang/probe_arc109_match_arm__positional_control.wat	OK	-	9	0	1	
tests/wat_lang/probe_arc109_match_arm__reversed_keys.wat	OK	-	7	0	1	
tests/wat_lang/probe_arc109_match_arm__unit_empty_map.wat	OK	-	7	0	1	
tests/wat_lang/probe_arc234_stone4_hash_destructure.wat	OK	-	21	0	1	
tests/wat_lang/probe_arc234_stone4_hash_destructure_unknown_field.wat	OK	-	4	0	1	
tests/wat_lang/probe_arc234_stone4_match_hash_destructure.wat	OK	-	31	0	1	
tests/wat_lang/probe_arc241_stone11_define_hard_cut.wat	OK	-	1	0	1	
tests/wat_lang/probe_arc241_stone12_defalias.wat	OK	-	9	0	1	
tests/wat_lang/probe_arc241_stone14_restricted_absorbed.wat	OK	-	3	0	1	
tests/wat_lang/probe_arc257_keys_destructure.wat	OK	-	8	0	1	
tests/wat_lang/probe_assert_true_false.wat	OK	-	16	0	1	
tests/wat_lang/probe_def_not_special.wat	OK	-	4	0	1	
tests/wat_lang/probe_def_not_special_mixed_ok.wat	OK	-	34	0	1	
tests/wat_lang/probe_def_not_special_spawn_ok.wat	OK	-	15	0	1	
tests/wat_lang/probe_nil_return_value_position_bug.wat	OK	-	5	0	1	
tests/wat_lang/probe_undefined_builtin_resolves.wat	OK	-	2	0	1	
tests/wat_lang/wat_arc072_letstar_parametric.wat	OK	-	17	0	1	
tests/wat_lang/wat_arc098_form_matches_runtime.wat	OK	-	138	0	1	
tests/wat_lang/wat_arc098_form_matches_typecheck.wat	OK	-	11	0	1	
tests/wat_lang/wat_arc136_do_form.wat	OK	-	35	0	1	
tests/wat_lang/wat_arc143_define_alias.wat	OK	-	8	0	1	
tests/wat_lang/wat_arc143_lookup.wat	OK	-	43	0	1	
tests/wat_lang/wat_arc143_manipulation.wat	OK	-	62	0	1	
tests/wat_lang/wat_arc144_hardcoded_primitives.wat	OK	-	52	0	1	
tests/wat_lang/wat_arc144_lookup_form.wat	OK	-	40	0	1	
tests/wat_lang/wat_arc144_special_forms.wat	OK	-	56	0	1	
tests/wat_lang/wat_arc153_nil_rename.wat	OK	-	3	0	1	
tests/wat_lang/wat_arc154_kill_let_star.wat	OK	-	26	0	1	
tests/wat_lang/wat_arc157_def.wat	OK	-	17	0	1	
tests/wat_lang/wat_arc157_def_do_splice_ok.wat	OK	-	3	0	1	
tests/wat_lang/wat_arc157_def_let_do_ok.wat	OK	-	5	0	1	
tests/wat_lang/wat_arc157_def_redef_true_ok.wat	OK	-	4	0	1	
tests/wat_lang/wat_arc157_def_sequential_ok.wat	OK	-	3	0	1	
tests/wat_lang/wat_arc168_let_flat_shape.wat	OK	-	39	0	1	
tests/wat_lang/wat_core_cond.wat	OK	-	36	0	1	
tests/wat_lang/wat_core_forms.wat	OK	-	47	0	1	
tests/wat_lang/wat_core_try.wat	OK	-	59	0	1	
tests/wat_lang/wat_idempotent_redeclare.wat	OK	-	10	0	1	
tests/wat_lang/wat_not_eq.wat	OK	-	14	0	1	
wat-scripts/census-defservice-arm-arity.wat	OK	-	106	0	1	
wat-scripts/cosines.wat	OK	-	10	0	1	
wat-scripts/demos/holon-literal/cosine.wat	OK	-	3	0	1	
wat-scripts/demos/stdio-service/stdio-service.wat	OK	-	19	0	1	
wat-scripts/demos/stream-protocol/stream-protocol.wat	OK	-	25	0	1	
wat-scripts/fixes/address-transport-arity.wat	OK	-	204	0	1	
wat-scripts/fixes/angle-brackets-to-binder.wat	OK	-	196	0	1	
wat-scripts/fixes/assertion-failed-to-kwargs.wat	OK	-	154	0	1	
wat-scripts/fixes/bare-none-keyword-to-fqdn.wat	OK	-	19	0	1	
wat-scripts/fixes/bare-symbol-shorthand-to-fqdn.wat	OK	-	25	0	1	
wat-scripts/fixes/bare-variant-to-qualified.wat	OK	-	27	0	1	
wat-scripts/fixes/break-kind-string-to-enum.wat	OK	-	128	0	1	
wat-scripts/fixes/caller-to-emitted-from.wat	OK	-	116	0	1	
wat-scripts/fixes/connect-outcome-facts.wat	OK	-	197	0	1	
wat-scripts/fixes/declare-max-request-bytes.wat	OK	-	179	0	1	
wat-scripts/fixes/defrule-then-to-vector.wat	OK	-	134	0	1	
wat-scripts/fixes/deprime-telemetry-sqlite.wat	OK	-	27	0	1	
wat-scripts/fixes/drop-deftest-prelude.wat	OK	-	77	0	1	
wat-scripts/fixes/drop-env-wat-dot-prefix.wat	OK	-	41	0	1	
wat-scripts/fixes/drop-unconsumed-negation-bind.wat	OK	-	233	0	1	
wat-scripts/fixes/edits-carry-the-old-text.wat	OK	-	284	0	1	
wat-scripts/fixes/eprintln-recv-arm-to-assertion-failed.wat	OK	-	105	0	1	
wat-scripts/fixes/face-underscore-bound-send-prime.wat	OK	-	120	0	1	
wat-scripts/fixes/first-of-drop-to-nth.wat	OK	-	21	0	1	
wat-scripts/fixes/fix-macro-param-types.wat	OK	-	19	0	1	
wat-scripts/fixes/fmt-head-fqdn-to-clojure.wat	OK	-	83	0	1	
wat-scripts/fixes/fn-keyword-to-bracket.wat	OK	-	133	0	1	
wat-scripts/fixes/hoist-where-into-condition.wat	OK	-	368	0	1	
wat-scripts/fixes/inline-constraint-per-type-spelling.wat	OK	-	101	0	1	
wat-scripts/fixes/kill-make-deftest.wat	OK	-	71	0	1	
wat-scripts/fixes/locus-methods-on-the-waist.wat	OK	-	63	0	1	
wat-scripts/fixes/locus-names-its-transport.wat	OK	-	160	0	1	
wat-scripts/fixes/mandate-invocation-ctx-param.wat	OK	-	116	0	1	
wat-scripts/fixes/mandate-request-malformed.wat	OK	-	218	0	1	
wat-scripts/fixes/mandatory-typed-quasiquote-residual.wat	OK	-	192	0	1	
wat-scripts/fixes/match-arm-to-bracket-map-pattern.wat	OK	-	739	0	1	
wat-scripts/fixes/move-deftest-callers-to-prime.wat	OK	-	21	0	1	
wat-scripts/fixes/move-deftest-hermetic-callers-to-prime.wat	OK	-	21	0	1	
wat-scripts/fixes/namespace-bare-top-level-names.wat	OK	-	199	0	1	
wat-scripts/fixes/namespace-defrule-names.wat	OK	-	192	0	1	
wat-scripts/fixes/node-kind-string-to-enum.wat	OK	-	226	0	1	
wat-scripts/fixes/one-param-spec.wat	OK	-	436	0	1	
wat-scripts/fixes/parametrics-take-a-type-vector.wat	OK	-	84	0	1	
wat-scripts/fixes/positional-ctor-to-map.wat	OK	-	747	0	1	
wat-scripts/fixes/positional-to-kwargs.wat	OK	-	179	0	1	
wat-scripts/fixes/query-answers-are-maps.wat	OK	-	249	0	1	
wat-scripts/fixes/read-string-to-outcome.wat	OK	-	124	0	1	
wat-scripts/fixes/readln-to-outcome.wat	OK	-	22	0	1	
wat-scripts/fixes/reclaim-deftest-names.wat	OK	-	22	0	1	
wat-scripts/fixes/reclaim-hologram-find-name.wat	OK	-	21	0	1	
wat-scripts/fixes/reclaim-ipc-prime-names.wat	OK	-	25	0	1	
wat-scripts/fixes/reclaim-service-fixture-names.wat	OK	-	25	0	1	
wat-scripts/fixes/reclaim-stdio-prime-names.wat	OK	-	26	0	1	
wat-scripts/fixes/rehead-rete-callees.wat	OK	-	23	0	1	
wat-scripts/fixes/rename-call-ctx-to-invocation.wat	OK	-	23	0	1	
wat-scripts/fixes/rename-core-bigint-rational-to-their-homes.wat	OK	-	113	0	1	
wat-scripts/fixes/rename-core-maps-to-their-homes.wat	OK	-	97	0	1	
wat-scripts/fixes/rename-core-numerics-to-their-homes.wat	OK	-	97	0	1	
wat-scripts/fixes/rename-core-set-and-list-to-their-homes.wat	OK	-	110	0	1	
wat-scripts/fixes/rename-core-string-to-string.wat	OK	-	97	0	1	
wat-scripts/fixes/rename-core-vectors-to-their-homes.wat	OK	-	149	0	1	
wat-scripts/fixes/rename-diederror-to-loci-died-error.wat	OK	-	117	0	1	
wat-scripts/fixes/rename-four-families-to-their-homes.wat	OK	-	123	0	1	
wat-scripts/fixes/rename-kernel-to-spawn.wat	OK	-	23	0	1	
wat-scripts/fixes/rename-keyword-to-its-home.wat	OK	-	81	0	1	
wat-scripts/fixes/rename-list-to-seq.wat	OK	-	21	0	1	
wat-scripts/fixes/rename-locidiederror-shutdown-to-stopped.wat	OK	-	27	0	1	
wat-scripts/fixes/rename-math-stat-seq-to-their-homes.wat	OK	-	136	0	1	
wat-scripts/fixes/rename-record-def-to-defrecord.wat	OK	-	22	0	1	
wat-scripts/fixes/rename-rete-numerics-to-their-homes.wat	OK	-	97	0	1	
wat-scripts/fixes/rename-seq-fold-aliases-to-core-reduce.wat	OK	-	23	0	1	
wat-scripts/fixes/rename-slipped-core-heads-to-their-homes.wat	OK	-	22	0	1	
wat-scripts/fixes/rename-sort-prime-to-native.wat	OK	-	25	0	1	
wat-scripts/fixes/rename-sourcefile-to-source-file.wat	OK	-	21	0	1	
wat-scripts/fixes/rename-string-verbs-to-their-home.wat	OK	-	195	0	1	
wat-scripts/fixes/rename-wat-record-to-core-record.wat	OK	-	21	0	1	
wat-scripts/fixes/rename-wat-tests-std-to-wat-tests.wat	OK	-	21	0	1	
wat-scripts/fixes/repoint-retired-heads-to-live-spellings.wat	OK	-	137	0	1	
wat-scripts/fixes/response-record-to-enum.wat	OK	-	232	0	1	
wat-scripts/fixes/restore-verbatim-literals-after-the-fold.wat	OK	-	91	0	1	
wat-scripts/fixes/retarget-peer-purity-probes.wat	OK	-	19	0	1	
wat-scripts/fixes/rete-bind-arrow-to-binder.wat	OK	-	64	0	1	
wat-scripts/fixes/rete-oracle-sigil.wat	OK	-	31	0	1	
wat-scripts/fixes/rete-truth-maintenance-probes/chain-fp.wat	OK	-	53	0	1	
wat-scripts/fixes/rete-truth-maintenance-probes/chain-spec.wat	OK	-	56	0	1	
wat-scripts/fixes/rete-truth-maintenance-probes/chain.wat	OK	-	99	0	1	
wat-scripts/fixes/rete-where-per-type-spelling.wat	OK	-	98	0	1	
wat-scripts/fixes/rewriting-rules-join-written.wat	OK	-	197	0	1	
wat-scripts/fixes/rule-record-to-defrule.wat	OK	-	237	0	1	
wat-scripts/fixes/send-outcome-facts.wat	OK	-	236	0	1	
wat-scripts/fixes/service-locus-to-user-rendezvous.wat	OK	-	21	0	1	
wat-scripts/fixes/spawn-builder-namespace-join.wat	OK	-	27	0	1	
wat-scripts/fixes/spawn-program-to-test-spawn-peer.wat	OK	-	21	0	1	
wat-scripts/fixes/stdin-frame-vocabulary.wat	OK	-	27	0	1	
wat-scripts/fixes/strip-expect-ascription.wat	OK	-	21	0	1	
wat-scripts/fixes/strip-insert-rhs-marker.wat	OK	-	69	0	1	
wat-scripts/fixes/strip-match-ascription.wat	OK	-	21	0	1	
wat-scripts/fixes/strip-useless-mains.wat	OK	-	143	0	1	
wat-scripts/fixes/struct-new-failure-to-message-only-failure.wat	OK	-	85	0	1	
wat-scripts/fixes/sweep-lint-fixes.wat	OK	-	26	0	1	
wat-scripts/fixes/thread-self-peer-to-peer.wat	OK	-	26	0	1	
wat-scripts/fixes/timer-prime-to-peer-prime.wat	OK	-	81	0	1	
wat-scripts/fixes/to-faithful-clojure-net.wat	OK	-	221	0	1	
wat-scripts/fixes/to-faithful-clojure-rete.wat	OK	-	168	0	1	
wat-scripts/fixes/to-faithful-clojure.wat	OK	-	19	0	1	
wat-scripts/fixes/transport-markers-to-family.wat	OK	-	102	0	1	
wat-scripts/fixes/tuple-parens-to-binder.wat	OK	-	170	0	1	
wat-scripts/fixes/type-member-colon-to-slash.wat	OK	-	31	0	1	
wat-scripts/fixes/type-query-to-defquery.wat	OK	-	321	0	1	
wat-scripts/fixes/typed-constructors.wat	OK	-	195	0	1	
wat-scripts/fixes/types-to-wat-type.wat	OK	-	373	0	1	
wat-scripts/fixes/unignore-arc170-concurrency.wat	OK	-	68	0	1	
wat-scripts/fixes/unstamp-transport-wire.wat	OK	-	211	0	1	
wat-scripts/fixes/unwrap-recvoutcome-false-positive.wat	OK	-	142	0	1	
wat-scripts/fixes/uuid-type-goes-home.wat	OK	-	78	0	1	
wat-scripts/fixes/variant-separator-to-dot.wat	OK	-	140	0	1	
wat-scripts/fixes/wrap-client-method-match-in-recvoutcome.wat	OK	-	120	0	1	
wat-scripts/fixes/wrap-compile-in-compileoutcome.wat	OK	-	131	0	1	
wat-scripts/fixes/wrap-connect-prime-in-connectoutcome.wat	OK	-	134	0	1	
wat-scripts/fixes/wrap-fire-once-in-fireoutcome.wat	OK	-	129	0	1	
wat-scripts/fixes/wrap-fire-rules-explain-in-fireoutcome.wat	OK	-	129	0	1	
wat-scripts/fixes/wrap-fire-rules-in-fireoutcome.wat	OK	-	129	0	1	
wat-scripts/fixes/wrap-insert-in-insertoutcome.wat	OK	-	137	0	1	
wat-scripts/fixes/wrap-nested-forms-recv-call.wat	OK	-	96	0	1	
wat-scripts/fixes/wrap-nested-forms-recv-in-recvoutcome.wat	OK	-	119	0	1	
wat-scripts/fixes/wrap-nested-forms-sendoutcome-stopped.wat	OK	-	99	0	1	
wat-scripts/fixes/wrap-overlay-in-fireoutcome.wat	OK	-	163	0	1	
wat-scripts/fixes/wrap-session-facts-in-factbag.wat	OK	-	154	0	1	
wat-scripts/fmt/fixtures/all-four.wat	OK	-	6	0	1	
wat-scripts/fmt/fixtures/assoc-pair.wat	OK	-	2	0	1	
wat-scripts/fmt/fixtures/assoc-ride.wat	OK	-	4	0	1	
wat-scripts/fmt/fixtures/atom-map.wat	OK	-	1	0	1	
wat-scripts/fmt/fixtures/claim-demo.wat	OK	-	7	0	1	
wat-scripts/fmt/fixtures/comment-indent.wat	OK	-	3	0	1	
wat-scripts/fmt/fixtures/compound-map.wat	OK	-	2	0	1	
wat-scripts/fmt/fixtures/compound-positional.wat	OK	-	3	0	1	
wat-scripts/fmt/fixtures/compound-then-kw.wat	OK	-	4	0	1	
wat-scripts/fmt/fixtures/cond-align.wat	OK	-	4	0	1	
wat-scripts/fmt/fixtures/cond-overflow.wat	OK	-	4	0	1	
wat-scripts/fmt/fixtures/defenum-mixed.wat	OK	-	1	0	1	
wat-scripts/fmt/fixtures/defenum-spawn-shape.wat	OK	-	2	0	1	
wat-scripts/fmt/fixtures/defmacro-ret.wat	OK	-	1	0	1	
wat-scripts/fmt/fixtures/defn-empty.wat	OK	-	1	0	1	
wat-scripts/fmt/fixtures/defn-multi.wat	OK	-	2	0	1	
wat-scripts/fmt/fixtures/defn-uneven.wat	OK	-	1	0	1	
wat-scripts/fmt/fixtures/defrecord-fields.wat	OK	-	1	0	1	
wat-scripts/fmt/fixtures/defstruct-fields.wat	OK	-	1	0	1	
wat-scripts/fmt/fixtures/diff-heads.wat	OK	-	3	0	1	
wat-scripts/fmt/fixtures/doc-example.wat	OK	-	34	0	1	
wat-scripts/fmt/fixtures/foldl-bare.wat	OK	-	4	0	1	
wat-scripts/fmt/fixtures/generic-fn.wat	OK	-	2	0	1	
wat-scripts/fmt/fixtures/get-odd.wat	OK	-	3	0	1	
wat-scripts/fmt/fixtures/half-broken.wat	OK	-	2	0	1	
wat-scripts/fmt/fixtures/hashmap-pair.wat	OK	-	1	0	1	
wat-scripts/fmt/fixtures/hof-lambda.wat	OK	-	4	0	1	
wat-scripts/fmt/fixtures/if-ride.wat	OK	-	5	0	1	
wat-scripts/fmt/fixtures/key-order.wat	OK	-	3	0	1	
wat-scripts/fmt/fixtures/kw-table.wat	OK	-	7	0	1	
wat-scripts/fmt/fixtures/kwargs-none.wat	OK	-	2	0	1	
wat-scripts/fmt/fixtures/kwargs-one.wat	OK	-	2	0	1	
wat-scripts/fmt/fixtures/kwargs-pos.wat	OK	-	1	0	1	
wat-scripts/fmt/fixtures/let-complex.wat	OK	-	6	0	1	
wat-scripts/fmt/fixtures/let-form.wat	OK	-	5	0	1	
wat-scripts/fmt/fixtures/let-top.wat	OK	-	2	0	1	
wat-scripts/fmt/fixtures/let-two.wat	OK	-	5	0	1	
wat-scripts/fmt/fixtures/long-map.wat	OK	-	1	0	1	
wat-scripts/fmt/fixtures/pair-table.wat	OK	-	5	0	1	
wat-scripts/fmt/fixtures/pos-table.wat	OK	-	1	0	1	
wat-scripts/fmt/fixtures/rec-shape.wat	OK	-	1	0	1	
wat-scripts/fmt/fixtures/spelling-clojure.wat	OK	-	0	0	1	
wat-scripts/fmt/fixtures/spelling-fqdn.wat	OK	-	2	0	1	
wat-scripts/fmt/fixtures/table-overflow.wat	OK	-	6	0	1	
wat-scripts/fmt/fixtures/type-ctor.wat	OK	-	1	0	1	
wat-scripts/fmt/fixtures/type-nested.wat	OK	-	1	0	1	
wat-scripts/fmt/fixtures/uneven-args.wat	OK	-	1	0	1	
wat-scripts/fmt/fixtures/unreadable-compound.wat	OK	-	3	0	1	
wat-scripts/fmt/fixtures/unruled-inside-defn.wat	OK	-	5	0	1	
wat-scripts/fmt/fixtures/unruled-top.wat	OK	-	4	0	1	
wat-scripts/fmt/rules/atoms.wat	OK	-	37	0	1	
wat-scripts/fmt/rules/cond.wat	OK	-	26	0	1	
wat-scripts/fmt/rules/defn-args.wat	OK	-	46	0	1	
wat-scripts/fmt/rules/defn.wat	OK	-	59	0	1	
wat-scripts/fmt/rules/defrecord-fields.wat	OK	-	97	0	1	
wat-scripts/fmt/rules/defrecord.wat	OK	-	134	0	1	
wat-scripts/fmt/rules/if.wat	OK	-	15	0	1	
wat-scripts/fmt/rules/kwargs.wat	OK	-	1082	0	1	
wat-scripts/fmt/rules/let-bindings.wat	OK	-	35	0	1	
wat-scripts/fmt/rules/let-blank.wat	OK	-	112	0	1	
wat-scripts/fmt/rules/let.wat	OK	-	36	0	1	
wat-scripts/fmt/rules/match.wat	OK	-	15	0	1	
wat-scripts/fmt/rules/siblings.wat	OK	-	168	0	1	
wat-scripts/fmt/rules/table.wat	OK	-	44	0	1	
wat-scripts/fmt/run-all.wat	OK	-	35	0	1	
wat-scripts/fmt/run-examples.wat	OK	-	88	0	1	
wat-scripts/fmt/run-let.wat	OK	-	32	0	1	
wat-scripts/fmt/run-r11.wat	OK	-	31	0	1	
wat-scripts/fmt/run-r3.wat	OK	-	32	0	1	
wat-scripts/fmt/run-r4.wat	OK	-	34	0	1	
wat-scripts/fmt/run-tables.wat	OK	-	98	0	1	
wat-scripts/fmt/run-types.wat	OK	-	34	0	1	
wat-scripts/fmt/run-width.wat	OK	-	130	0	1	
wat-scripts/fmt/run.wat	OK	-	30	0	1	
wat-scripts/grep/bare-variant-constructors.wat	OK	-	42	0	1	
wat-scripts/grep/can-raise.wat	OK	-	52	0	1	
wat-scripts/grep/core-numerics-ops.wat	OK	-	28	0	1	
wat-scripts/grep/defined-twice.wat	OK	-	36	0	1	
wat-scripts/grep/head-position.wat	OK	-	20	0	1	
wat-scripts/grep/rete-numerics-ops.wat	OK	-	28	0	1	
wat-scripts/grep/unwrap-of-lookup.wat	OK	-	37	0	1	
wat-scripts/intrinsic-metadata.wat	OK	-	5	0	1	
wat-scripts/lib/wat-grep.wat	OK	-	44	0	1	
wat-scripts/perf/deep-cascade.wat	OK	-	93	0	1	
wat-scripts/perf/grid/accum-lead-derived.wat	OK	-	117	0	1	
wat-scripts/perf/grid/accum-lead-rule-cascade.wat	OK	-	118	0	1	
wat-scripts/perf/grid/accum-over-derived.wat	OK	-	124	0	1	
wat-scripts/perf/grid/accum.wat	OK	-	182	0	1	
wat-scripts/perf/grid/asym-join.wat	OK	-	99	0	1	
wat-scripts/perf/grid/deep-cascade.wat	OK	-	141	0	1	
wat-scripts/perf/grid/fanout.wat	OK	-	124	0	1	
wat-scripts/perf/grid/leading-exists.wat	OK	-	99	0	1	
wat-scripts/perf/grid/leading-neg-consumer.wat	OK	-	133	0	1	
wat-scripts/perf/grid/min-finding.wat	OK	-	99	0	1	
wat-scripts/perf/grid/neg-consumer.wat	OK	-	92	0	1	
wat-scripts/perf/grid/negation.wat	OK	-	81	0	1	
wat-scripts/perf/grid/node-share.wat	OK	-	93	0	1	
wat-scripts/perf/grid/parametric-erasure.wat	OK	-	138	0	1	
wat-scripts/perf/grid/retract-accum-derived.wat	OK	-	129	0	1	
wat-scripts/perf/grid/retract-lead-accum.wat	OK	-	121	0	1	
wat-scripts/perf/grid/retract-multiplicity.wat	OK	-	82	0	1	
wat-scripts/perf/grid/strat-neg.wat	OK	-	310	0	1	
wat-scripts/perf/grid/user-reduce.wat	OK	-	95	0	1	
wat-scripts/perf/grid/userfn-accum-derived.wat	OK	-	126	0	1	
wat-scripts/perf/grid/userfn-head.wat	OK	-	107	0	1	
wat-scripts/perf/grid/where-accum-from-left.wat	OK	-	111	0	1	
wat-scripts/perf/grid/where-accum-group.wat	OK	-	103	0	1	
wat-scripts/perf/grid/where-accum-lead-cascade.wat	OK	-	60	0	1	
wat-scripts/perf/grid/where-accum-lead.wat	OK	-	121	0	1	
wat-scripts/perf/grid/where-accum-where-chain.wat	OK	-	72	0	1	
wat-scripts/perf/grid/where-accum-where.wat	OK	-	121	0	1	
wat-scripts/perf/grid/where-boolean.wat	OK	-	227	0	1	
wat-scripts/perf/grid/where-collection.wat	OK	-	215	0	1	
wat-scripts/perf/grid/where-control.wat	OK	-	191	0	1	
wat-scripts/perf/grid/where-exists.wat	OK	-	351	0	1	
wat-scripts/perf/grid/where-fact-bind.wat	OK	-	111	0	1	
wat-scripts/perf/grid/where-inline-computed.wat	OK	-	186	0	1	
wat-scripts/perf/grid/where-inline-keyword.wat	OK	-	119	0	1	
wat-scripts/perf/grid/where-join-left.wat	OK	-	133	0	1	
wat-scripts/perf/grid/where-join-order.wat	OK	-	123	0	1	
wat-scripts/perf/grid/where-multivar.wat	OK	-	200	0	1	
wat-scripts/perf/grid/where-nested-combinators.wat	OK	-	83	0	1	
wat-scripts/perf/grid/where-nesting.wat	OK	-	240	0	1	
wat-scripts/perf/grid/where-not-and-bound.wat	OK	-	159	0	1	
wat-scripts/perf/grid/where-not-and-not.wat	OK	-	169	0	1	
wat-scripts/perf/grid/where-not-and.wat	OK	-	166	0	1	
wat-scripts/perf/grid/where-not-bound.wat	OK	-	49	0	1	
wat-scripts/perf/grid/where-not-derived-in-query.wat	OK	-	71	0	1	
wat-scripts/perf/grid/where-not-fact.wat	OK	-	95	0	1	
wat-scripts/perf/grid/where-not-not.wat	OK	-	126	0	1	
wat-scripts/perf/grid/where-not-or.wat	OK	-	166	0	1	
wat-scripts/perf/grid/where-not-where.wat	OK	-	88	0	1	
wat-scripts/perf/grid/where-not-windy.wat	OK	-	117	0	1	
wat-scripts/perf/grid/where-numeric.wat	OK	-	178	0	1	
wat-scripts/perf/grid/where-or-and.wat	OK	-	117	0	1	
wat-scripts/perf/grid/where-or-conditions.wat	OK	-	274	0	1	
wat-scripts/perf/grid/where-or-inline.wat	OK	-	130	0	1	
wat-scripts/perf/grid/where-query-compat.wat	OK	-	278	0	1	
wat-scripts/perf/grid/where-query-params.wat	OK	-	76	0	1	
wat-scripts/perf/grid/where-record.wat	OK	-	274	0	1	
wat-scripts/perf/grid/where-shapes.wat	OK	-	154	0	1	
wat-scripts/perf/grid/where-string.wat	OK	-	202	0	1	
wat-scripts/perf/grid/where-test-chain.wat	OK	-	88	0	1	
wat-scripts/perf/matrix/fanout-join.wat	OK	-	57	0	1	
wat-scripts/probes/arc-054/probe-054-fn-idempotency.wat	OK	-	6	0	1	
wat-scripts/probes/arc-170/probe-bracket-process-runner.wat	OK	-	44	0	1	
wat-scripts/probes/arc-170/probe-c1-ast-shape.wat	OK	-	77	0	1	
wat-scripts/probes/arc-170/probe-c1-clean-surface.wat	OK	-	28	0	1	
wat-scripts/probes/arc-170/probe-c1-fnforms-kwargs-impl.wat	OK	-	27	0	1	
wat-scripts/probes/arc-170/probe-c1-kwargs-impl-astname.wat	OK	-	46	0	1	
wat-scripts/probes/arc-170/probe-c1-plain-fnforms-shape.wat	OK	-	76	0	1	
wat-scripts/probes/arc-170/probe-c2-narrow-2param-plainrecord.wat	OK	-	14	0	1	
wat-scripts/probes/arc-170/probe-c2-narrow-multisurface.wat	OK	-	17	0	1	
wat-scripts/probes/arc-170/probe-c2-nonparam-baseline.wat	OK	-	12	0	1	
wat-scripts/probes/arc-170/probe-cap2-isolate.wat	OK	-	17	0	1	
wat-scripts/probes/arc-170/probe-cap2-process-grantpath.wat	OK	-	16	0	1	
wat-scripts/probes/arc-170/probe-cap2-spawnrunner-pid.wat	OK	-	20	0	1	
wat-scripts/probes/arc-170/probe-compound-upcast.wat	OK	-	17	0	1	
wat-scripts/probes/arc-170/probe-defclause-open-arg.wat	OK	-	21	0	1	
wat-scripts/probes/arc-170/probe-defclause-real-shape.wat	OK	-	22	0	1	
wat-scripts/probes/arc-170/probe-deporder.wat	OK	-	8	0	1	
wat-scripts/probes/arc-170/probe-edn.wat	OK	-	9	0	1	
wat-scripts/probes/arc-170/probe-fnforms-keyword.wat	OK	-	9	0	1	
wat-scripts/probes/arc-170/probe-fnforms-shape.wat	OK	-	17	0	1	
wat-scripts/probes/arc-170/probe-freeze-narrow.wat	OK	-	11	0	1	
wat-scripts/probes/arc-170/probe-kwargs-peer.wat	OK	-	31	0	1	
wat-scripts/probes/arc-170/probe-kwargs-struct.wat	OK	-	11	0	1	
wat-scripts/probes/arc-170/probe-locus1-generic-surface-method.wat	OK	-	12	0	1	
wat-scripts/probes/arc-170/probe-locus2-abstract-surface-dispatch.wat	OK	-	14	0	1	
wat-scripts/probes/arc-170/probe-m1-ann-erase.wat	OK	-	83	0	1	
wat-scripts/probes/arc-170/probe-m1-ann-erase2.wat	OK	-	83	0	1	
wat-scripts/probes/arc-170/probe-m1-apply-generic.wat	OK	-	9	0	1	
wat-scripts/probes/arc-170/probe-m1-argcount.wat	OK	-	66	0	1	
wat-scripts/probes/arc-170/probe-m1-arity.wat	OK	-	10	0	1	
wat-scripts/probes/arc-170/probe-m1-dial-runner.wat	OK	-	71	0	1	
wat-scripts/probes/arc-170/probe-m1-dump-forms.wat	OK	-	35	0	1	
wat-scripts/probes/arc-170/probe-m1-erase-only.wat	OK	-	21	0	1	
wat-scripts/probes/arc-170/probe-m1-fix-norevoke.wat	OK	-	97	0	1	
wat-scripts/probes/arc-170/probe-m1-grant-admits.wat	OK	-	60	0	1	
wat-scripts/probes/arc-170/probe-m1-phantom-d.wat	OK	-	36	0	1	
wat-scripts/probes/arc-170/probe-m1-service-pid.wat	OK	-	20	0	1	
wat-scripts/probes/arc-170/probe-m1-surface-ast.wat	OK	-	38	0	1	
wat-scripts/probes/arc-170/probe-m1-worker-setup.wat	OK	-	95	0	1	
wat-scripts/probes/arc-170/probe-mod-in-macro.wat	OK	-	14	0	1	
wat-scripts/probes/arc-170/probe-nested-vector-of-tuples.wat	OK	-	16	0	1	
wat-scripts/probes/arc-170/probe-process-only.wat	OK	-	10	0	1	
wat-scripts/probes/arc-170/probe-reason-downcast.wat	OK	-	19	0	1	
wat-scripts/probes/arc-170/probe-recordtype-fixed.wat	OK	-	10	0	1	
wat-scripts/probes/arc-170/probe-s1-fn-forms.wat	OK	-	47	0	1	
wat-scripts/probes/arc-170/probe-s1-named.wat	OK	-	47	0	1	
wat-scripts/probes/arc-170/probe-s2-runner-count.wat	OK	-	38	0	1	
wat-scripts/probes/arc-170/probe-s3-bracket-loci.wat	OK	-	16	0	1	
wat-scripts/probes/arc-170/probe-s3-process-runner.wat	OK	-	57	0	1	
wat-scripts/probes/arc-170/probe-s3a-select-peer.wat	OK	-	6	0	1	
wat-scripts/probes/arc-170/probe-s3b-astsplice.wat	OK	-	63	0	1	
wat-scripts/probes/arc-170/probe-s3b-extract.wat	OK	-	33	0	1	
wat-scripts/probes/arc-170/probe-s3c-rendezvous.wat	OK	-	56	0	1	
wat-scripts/probes/arc-170/probe-strikeB-fields.wat	OK	-	24	0	1	
wat-scripts/probes/arc-170/probe-surface-ships.wat	OK	-	15	0	1	
wat-scripts/probes/arc-170/probe-thread-only.wat	OK	-	10	0	1	
wat-scripts/probes/arc-170/probe-trivial.wat	OK	-	2	0	1	
wat-scripts/probes/arc-170/probe-type-splice.wat	OK	-	18	0	1	
wat-scripts/probes/arc-170/root-gapA.wat	OK	-	33	0	1	
wat-scripts/probes/arc-170/scout-kwargs-expand.wat	OK	-	15	0	1	
wat-scripts/probes/arc-278/s2s-init-input.wat	OK	-	37	0	1	
wat-scripts/probes/arc-278/s2s-midlife-vec-probe.wat	OK	-	103	0	1	
wat-scripts/probes/arc-278/s2s-parent-echo.wat	OK	-	35	0	1	
wat-scripts/probes/arc-278/s2s-process-probe.wat	OK	-	74	0	1	
wat-scripts/probes/arc-278/s2s-thread-probe.wat	OK	-	71	0	1	
wat-scripts/probes/arc-293/s3-probe-struct-satisfies-nature-struct.wat	OK	-	14	0	1	
wat-scripts/probes/arc-293/s4c-carrier-probe.wat	OK	-	12	0	1	
wat-scripts/probes/arc-293/s4c-messages-acceptance.wat	OK	-	56	0	1	
wat-scripts/probes/arc-293/s4c-thread.wat	OK	-	56	0	1	
wat-scripts/read-flat.wat	OK	-	6	0	1	
wat-scripts/scratch-pad/109-dormant-minter/probe-a-kwargs-only.wat	OK	-	6	0	1	
wat-scripts/scratch-pad/109-dormant-minter/probe-b-binder-only.wat	OK	-	6	0	1	
wat-scripts/scratch-pad/109-dormant-minter/probe-c-binder-and-kwargs.wat	OK	-	6	0	1	
wat-scripts/scratch-pad/109-dormant-minter/probe-d-binder-used-in-kwargs-field.wat	OK	-	6	0	1	
wat-scripts/scratch-pad/109-dormant-minter/probe-row3-lru-svc-roundtrip.wat	OK	-	42	0	1	
wat-scripts/scratch-pad/1a-epsilon-probe/probe-noeval-args.wat	OK	-	8	0	1	
wat-scripts/scratch-pad/1a-epsilon-probe/probe-nonleading-setredef.wat	OK	-	4	0	1	
wat-scripts/scratch-pad/251-8d-ii-vii-is-primitive-spelling-sensitive.wat	OK	-	58	0	1	
wat-scripts/scratch-pad/255-14-defservice-minted-names.wat	OK	-	35	0	1	
wat-scripts/scratch-pad/255-17a-child-main-transport.wat	OK	-	39	0	1	
wat-scripts/scratch-pad/255-17b-transport-family-shapes.wat	OK	-	13	0	1	
wat-scripts/scratch-pad/255-18-generic-locus-narrows-to-a-clause.wat	OK	-	20	0	1	
wat-scripts/scratch-pad/255-19-head-name-of-a-surface-call.wat	OK	-	14	0	1	
wat-scripts/scratch-pad/255-19-start-impl-locus-param.wat	OK	-	47	0	1	
wat-scripts/scratch-pad/255-1a-alpha-what-the-sketch-now-says.wat	OK	-	14	0	1	
wat-scripts/scratch-pad/255-1a-beta-ii-the-lift-still-discriminates.wat	OK	-	16	0	1	
wat-scripts/scratch-pad/255-1a-beta-ii-the-registry-answers-for-all-nine.wat	OK	-	26	0	1	
wat-scripts/scratch-pad/255-1a-beta-the-declare-regime-and-the-unevaluated-pole.wat	OK	-	10	0	1	
wat-scripts/scratch-pad/255-1a-gamma-i-example-verification.wat	OK	-	39	0	1	
wat-scripts/scratch-pad/255-1a-gamma-i-macroexpand-hygiene-determinism.wat	OK	-	21	0	1	
wat-scripts/scratch-pad/255-21-kwargs-check-expansion.wat	OK	-	8	0	1	
wat-scripts/scratch-pad/255-25-child-main-says-wire.wat	OK	-	56	0	1	
wat-scripts/scratch-pad/255-27/expand-handle.wat	OK	-	11	0	1	
wat-scripts/scratch-pad/255-27/survivor-address_short.wat	OK	-	3	0	1	
wat-scripts/scratch-pad/255-27/survivor-bare_locus.wat	OK	-	2	0	1	
wat-scripts/scratch-pad/255-27/survivor-bare_record.wat	OK	-	3	0	1	
wat-scripts/scratch-pad/255-27/survivor-launched_short.wat	OK	-	3	0	1	
wat-scripts/scratch-pad/255-2a-an-alias-contradicts-its-target.wat	OK	-	6	0	1	
wat-scripts/scratch-pad/255-46/hello-extend-type-today.wat	OK	-	14	0	1	
wat-scripts/scratch-pad/255-46/mixed-spawned-vector.wat	OK	-	5	0	1	
wat-scripts/scratch-pad/255-69-census-walk.wat	OK	-	88	0	1	
wat-scripts/scratch-pad/255-alias-signature-drift.wat	OK	-	39	0	1	
wat-scripts/scratch-pad/255-b0-name-and-totality.wat	OK	-	17	0	1	
wat-scripts/scratch-pad/255-b0-rows-without-handlers.wat	OK	-	20	0	1	
wat-scripts/scratch-pad/255-b0-what-actually-gates-the-rete-rows.wat	OK	-	29	0	1	
wat-scripts/scratch-pad/255-can-the-reader-parse-a-syntax-grammar.wat	OK	-	11	0	1	
wat-scripts/scratch-pad/255-can-the-wat-reader-hold-a-doc-map.wat	OK	-	19	0	1	
wat-scripts/scratch-pad/255-does-edn-round-trip-a-wat-keyword.wat	OK	-	12	0	1	
wat-scripts/scratch-pad/255-edn-doc-row-char-before-after.wat	OK	-	10	0	1	
wat-scripts/scratch-pad/255-edn-type-method-keyword-roundtrip.wat	OK	-	8	0	1	
wat-scripts/scratch-pad/255-eval-walk-composition-roundtrip.wat	OK	-	21	0	1	
wat-scripts/scratch-pad/255-home-10-math-stat-seq-examples.wat	OK	-	84	0	1	
wat-scripts/scratch-pad/255-home-12-ast-verify-examples.wat	OK	-	68	0	1	
wat-scripts/scratch-pad/255-home4-string-carve-metadata.wat	OK	-	59	0	1	
wat-scripts/scratch-pad/255-home4-verify-examples-failures.wat	OK	-	17	0	1	
wat-scripts/scratch-pad/255-how-does-write-pretty-handle-a-docstring.wat	OK	-	7	0	1	
wat-scripts/scratch-pad/255-is-Type-method-ambiguous-on-the-wire.wat	OK	-	11	0	1	
wat-scripts/scratch-pad/255-p3/dump-to-hex-metadata.wat	OK	-	6	0	1	
wat-scripts/scratch-pad/255-p6-a-forms-still-run.wat	OK	-	27	0	1	
wat-scripts/scratch-pad/255-p6-a-nothing-else-moved.wat	OK	-	17	0	1	
wat-scripts/scratch-pad/255-p6-a-show-source-if-let.wat	OK	-	7	0	1	
wat-scripts/scratch-pad/255-p6c-w1-config-arity-all-four.wat	OK	-	46	0	1	
wat-scripts/scratch-pad/255-p6c-w1-config-direct-calls.wat	OK	-	18	0	1	
wat-scripts/scratch-pad/255-p6c-w1-config-metadata.wat	OK	-	30	0	1	
wat-scripts/scratch-pad/255-p6c-w2-arity.wat	OK	-	57	0	1	
wat-scripts/scratch-pad/255-p6c-w2-direct-calls.wat	OK	-	40	0	1	
wat-scripts/scratch-pad/255-p6c-w2-metadata.wat	OK	-	37	0	1	
wat-scripts/scratch-pad/255-p6c-w4-arity.wat	OK	-	35	0	1	
wat-scripts/scratch-pad/255-p6c-w4-direct-calls.wat	OK	-	20	0	1	
wat-scripts/scratch-pad/255-p6c-w4-metadata.wat	OK	-	23	0	1	
wat-scripts/scratch-pad/255-p6c-w5a-arity-errors.wat	OK	-	101	0	1	
wat-scripts/scratch-pad/255-p6c-w5a-metadata.wat	OK	-	65	0	1	
wat-scripts/scratch-pad/255-p6c-w5b-arity-errors.wat	OK	-	68	0	1	
wat-scripts/scratch-pad/255-p6c-w5b-metadata.wat	OK	-	44	0	1	
wat-scripts/scratch-pad/255-p6c-w5c-arity-errors.wat	OK	-	46	0	1	
wat-scripts/scratch-pad/255-p6c-w5c-direct-calls.wat	OK	-	76	0	1	
wat-scripts/scratch-pad/255-p6c-w5c-metadata.wat	OK	-	30	0	1	
wat-scripts/scratch-pad/255-p6c-w6-arity-errors.wat	OK	-	79	0	1	
wat-scripts/scratch-pad/255-p6c-w6-direct-calls.wat	OK	-	35	0	1	
wat-scripts/scratch-pad/255-p6c-w6-metadata.wat	OK	-	51	0	1	
wat-scripts/scratch-pad/255-p6c1-two-verbs-homed.wat	OK	-	14	0	1	
wat-scripts/scratch-pad/255-probe-a-declaration-cannot-be-stored-unvalidated.wat	OK	-	18	0	1	
wat-scripts/scratch-pad/255-probe-a-resolved-name-agrees-with-a-head.wat	OK	-	21	0	1	
wat-scripts/scratch-pad/255-probe-an-example-is-a-form.wat	OK	-	19	0	1	
wat-scripts/scratch-pad/255-probe-are-the-orphans-live.wat	OK	-	10	0	1	
wat-scripts/scratch-pad/255-probe-can-a-user-make-sort-effectful.wat	OK	-	17	0	1	
wat-scripts/scratch-pad/255-probe-metadata-of-one-shape.wat	OK	-	65	0	1	
wat-scripts/scratch-pad/255-probe-sort-imposes-purity.wat	OK	-	24	0	1	
wat-scripts/scratch-pad/255-probe-the-accessor-classifies-pure.wat	OK	-	16	0	1	
wat-scripts/scratch-pad/255-probe-the-classifier-cannot-see-through-a-closure.wat	OK	-	19	0	1	
wat-scripts/scratch-pad/255-probe-the-classifier-follows-a-capture.wat	OK	-	19	0	1	
wat-scripts/scratch-pad/255-probe-the-collection-readers.wat	OK	-	70	0	1	
wat-scripts/scratch-pad/255-probe-the-option-result-siblings.wat	OK	-	53	0	1	
wat-scripts/scratch-pad/255-registry-census.wat	OK	-	53	0	1	
wat-scripts/scratch-pad/255-sf3-bucket2-baseline.wat	OK	-	20	0	1	
wat-scripts/scratch-pad/255-sf3-bucket3-unregistered.wat	OK	-	8	0	1	
wat-scripts/scratch-pad/255-stepvalue-watast-roundtrip-probe.wat	OK	-	51	0	1	
wat-scripts/scratch-pad/255-stone-a-i-both-i64-spellings.wat	OK	-	38	0	1	
wat-scripts/scratch-pad/255-stone-a-i-i64-overflow-under-new-spelling.wat	OK	-	5	0	1	
wat-scripts/scratch-pad/255-stone-a-ii-both-f64-spellings.wat	OK	-	43	0	1	
wat-scripts/scratch-pad/255-stone-d-both-bigint-rational-spellings.wat	OK	-	53	0	1	
wat-scripts/scratch-pad/255-stone-e-i-both-map-spellings.wat	OK	-	43	0	1	
wat-scripts/scratch-pad/255-stone-g-producer-provenance-restored.wat	OK	-	8	0	1	
wat-scripts/scratch-pad/255-stone-h-1a-holon-metadata-arity.wat	OK	-	107	0	1	
wat-scripts/scratch-pad/255-stone-h-1a-holon-success-calls.wat	OK	-	124	0	1	
wat-scripts/scratch-pad/255-stone-h-1a-holon-wrong-arity.wat	OK	-	352	0	1	
wat-scripts/scratch-pad/255-stone-h-1b-atom-metadata-arity.wat	OK	-	182	0	1	
wat-scripts/scratch-pad/255-stone-h-1b-atom-success-calls.wat	OK	-	640	0	1	
wat-scripts/scratch-pad/255-stone-h-1b-atom-wrong-arity.wat	OK	-	602	0	1	
wat-scripts/scratch-pad/255-stone-layer-2-vector-codec-roundtrip.wat	OK	-	48	0	1	
wat-scripts/scratch-pad/255-stone-o-apply-has-three-broken-doors.wat	OK	-	55	0	1	
wat-scripts/scratch-pad/255-stone-o-apply-lies-about-what-exists.wat	OK	-	41	0	1	
wat-scripts/scratch-pad/255-stone-o-i-arity-mismatch-not-unknown-function.wat	OK	-	10	0	1	
wat-scripts/scratch-pad/255-stone-o-i-vector-concat-value-door-panic.wat	OK	-	27	0	1	
wat-scripts/scratch-pad/255-stone-o-ii-apply-agrees-with-direct.wat	OK	-	58	0	1	
wat-scripts/scratch-pad/255-stone-o-ii-apply-special-forms-still-refused.wat	OK	-	22	0	1	
wat-scripts/scratch-pad/255-stone-o-iii-six-verbs-direct.wat	OK	-	89	0	1	
wat-scripts/scratch-pad/255-stone-o-iv-a-both-branches-proven-separately.wat	OK	-	25	0	1	
wat-scripts/scratch-pad/255-stone-o-iv-a-door2-kind-changed.wat	OK	-	9	0	1	
wat-scripts/scratch-pad/255-stone-o-iv-b-collections-sweep-apply.wat	OK	-	124	0	1	
wat-scripts/scratch-pad/255-stone-o-iv-b-collections-sweep-direct.wat	OK	-	259	0	1	
wat-scripts/scratch-pad/255-stone-o-iv-c-0-require-family-wrong-type.wat	OK	-	93	0	1	
wat-scripts/scratch-pad/255-stone-o-iv-c-1-holon-sweep-apply.wat	OK	-	151	0	1	
wat-scripts/scratch-pad/255-stone-o-iv-c-1-span-still-points-at-caller.wat	OK	-	16	0	1	
wat-scripts/scratch-pad/255-stone-o-iv-c-2-arg-span-still-points-at-argument.wat	OK	-	16	0	1	
wat-scripts/scratch-pad/255-stone-o-iv-c-2-atom-sweep-apply.wat	OK	-	163	0	1	
wat-scripts/scratch-pad/255-stone-o-iv-d-declare-acronyms-unevaluated-check.wat	OK	-	11	0	1	
wat-scripts/scratch-pad/255-stone-o-iv-d-direct-calls.wat	OK	-	69	0	1	
wat-scripts/scratch-pad/255-stone-o-iv-d-metadata.wat	OK	-	11	0	1	
wat-scripts/scratch-pad/255-stone-o-iv-d-remainder-apply.wat	OK	-	59	0	1	
wat-scripts/scratch-pad/255-stone-o-iv-d-unevaluated-args-check.wat	OK	-	6	0	1	
wat-scripts/scratch-pad/255-stone-o-the-value-door-panics-on-arity.wat	OK	-	27	0	1	
wat-scripts/scratch-pad/255-stone-p2-intrinsic-untouched.wat	OK	-	22	0	1	
wat-scripts/scratch-pad/255-stone-p2-render-doc-unchanged.wat	OK	-	6	0	1	
wat-scripts/scratch-pad/255-stone-p2-special-form-entry-stops-lying.wat	OK	-	21	0	1	
wat-scripts/scratch-pad/255-stone-p5-b-restricted-yields-render.wat	OK	-	7	0	1	
wat-scripts/scratch-pad/255-stone-p5-b-yields-render.wat	OK	-	8	0	1	
wat-scripts/scratch-pad/255-stone-p6c-w6-core-collection-readers-metadata-arity.wat	OK	-	45	0	1	
wat-scripts/scratch-pad/255-stone-p7-metadata-purity.wat	OK	-	11	0	1	
wat-scripts/scratch-pad/255-stone-p7-nullary-algebra-eleven.wat	OK	-	72	0	1	
wat-scripts/scratch-pad/255-stone-q-2-direct-door-unchanged.wat	OK	-	37	0	1	
wat-scripts/scratch-pad/255-stone-q-2-the-threaded-span-must-be-used.wat	OK	-	46	0	1	
wat-scripts/scratch-pad/255-stone-registry-can-be-enumerated-census.wat	OK	-	88	0	1	
wat-scripts/scratch-pad/255-stop2-reduce-registry-alias-probe.wat	OK	-	19	0	1	
wat-scripts/scratch-pad/255-struct-field-is-a-constant-projection.wat	OK	-	30	0	1	
wat-scripts/scratch-pad/255-tail-door-rete-form-spelling.wat	OK	-	15	0	1	
wat-scripts/scratch-pad/255-the-mirror-wall.wat	OK	-	4	0	1	
wat-scripts/scratch-pad/255-the-record-family-homed.wat	OK	-	113	0	1	
wat-scripts/scratch-pad/255-the-registry-answers-first-wave-2.wat	OK	-	80	0	1	
wat-scripts/scratch-pad/255-the-registry-answers-first-wave-3.wat	OK	-	104	0	1	
wat-scripts/scratch-pad/255-the-registry-answers-first.wat	OK	-	45	0	1	
wat-scripts/scratch-pad/277-all-atom-pair-runs.wat	OK	-	23	0	1	
wat-scripts/scratch-pad/277-can-a-rete-rhs-carry-an-enum.wat	OK	-	31	0	1	
wat-scripts/scratch-pad/277-can-a-rete-where-compare-an-enum.wat	OK	-	59	0	1	
wat-scripts/scratch-pad/277-can-wat-read-its-own-grammar.wat	OK	-	37	0	1	
wat-scripts/scratch-pad/277-comments-both-sides.wat	OK	-	32	0	1	
wat-scripts/scratch-pad/277-does-defn-have-a-row.wat	OK	-	32	0	1	
wat-scripts/scratch-pad/277-does-the-registry-know-slots.wat	OK	-	23	0	1	
wat-scripts/scratch-pad/277-file-ends-with.wat	OK	-	43	0	1	
wat-scripts/scratch-pad/277-file-width-census.wat	OK	-	57	0	1	
wat-scripts/scratch-pad/277-fmt-dump.wat	OK	-	22	0	1	
wat-scripts/scratch-pad/277-head-kind-census.wat	OK	-	14	0	1	
wat-scripts/scratch-pad/277-how-many-map-literals.wat	OK	-	14	0	1	
wat-scripts/scratch-pad/277-is-colon-dash-always-a-type-app.wat	OK	-	22	0	1	
wat-scripts/scratch-pad/277-layout-shape-probe.wat	OK	-	26	0	1	
wat-scripts/scratch-pad/277-lint-recount.wat	OK	-	20	0	1	
wat-scripts/scratch-pad/277-locate-the-slot-in-a-grammar.wat	OK	-	36	0	1	
wat-scripts/scratch-pad/277-registry-row-pretty.wat	OK	-	46	0	1	
wat-scripts/scratch-pad/277-the-node-kind-boundary.wat	OK	-	48	0	1	
wat-scripts/scratch-pad/277-what-does-the-reader-lose.wat	OK	-	26	0	1	
wat-scripts/scratch-pad/277-width-as-a-fold.wat	OK	-	67	0	1	
wat-scripts/scratch-pad/277-width-offenders.wat	OK	-	44	0	1	
wat-scripts/scratch-pad/282-row1-unquote-liar.wat	OK	-	29	0	1	
wat-scripts/scratch-pad/282-verify-same-length-mismatch.wat	OK	-	4	0	1	
wat-scripts/scratch-pad/8d-probe-reader-macro-nodes.wat	OK	-	24	0	1	
wat-scripts/scratch-pad/a5-termination-silence.wat	OK	-	9	0	1	
wat-scripts/scratch-pad/arc-defclause-meta-probe/probe2.wat	OK	-	3	0	1	
wat-scripts/scratch-pad/arc-defclause-meta-probe/probe5-stdlib-loads-clean.wat	OK	-	2	0	1	
wat-scripts/scratch-pad/arc109-2i-colon-mode-verbatim-probe.wat	OK	-	18	0	1	
wat-scripts/scratch-pad/arc109-2iii-fn-bracket-destinations.wat	OK	-	15	0	1	
wat-scripts/scratch-pad/arc109-blocker5-parametric-form-reference-by-head.wat	OK	-	9	0	1	
wat-scripts/scratch-pad/arc109-gamma-i-anon-fn-is-rigid.wat	OK	-	13	0	1	
wat-scripts/scratch-pad/arc109-reap-the-twelve-row1-operators.wat	OK	-	20	0	1	
wat-scripts/scratch-pad/arc109-row3-kwargs-ctor-runtime.wat	OK	-	29	0	1	
wat-scripts/scratch-pad/arc109-tuple-arm-faults.wat	OK	-	15	0	1	
wat-scripts/scratch-pad/arc109-tuple-bracket-reader.wat	OK	-	12	0	1	
wat-scripts/scratch-pad/arc109-type-equal-acceptance.wat	OK	-	38	0	1	
wat-scripts/scratch-pad/arc109-type-equal-row6-macro.wat	OK	-	23	0	1	
wat-scripts/scratch-pad/arc109-type-reference-not-expression.wat	OK	-	31	0	1	
wat-scripts/scratch-pad/arc255-67/probe-never.wat	OK	-	3	0	1	
wat-scripts/scratch-pad/arc255-67/probe-resolution.wat	OK	-	23	0	1	
wat-scripts/scratch-pad/arc255-67/probe-resolution2.wat	OK	-	4	0	1	
wat-scripts/scratch-pad/arc278-88/rete-defn-callable-from-ordinary-wat.wat	OK	-	5	0	1	
wat-scripts/scratch-pad/arc278-88/verify-stdlib.wat	OK	-	3	0	1	
wat-scripts/scratch-pad/arc278-acc-count-unused-bind/probe-acc-count-unused-bind.wat	OK	-	59	0	1	
wat-scripts/scratch-pad/arc278-diag-stopped-poll.wat	OK	-	22	0	1	
wat-scripts/scratch-pad/arc278-fence-binder-shadow/census-fence-binders.wat	OK	-	159	0	1	
wat-scripts/scratch-pad/arc278-fence-interior-types/probe-clause-level-where-fires.wat	OK	-	39	0	1	
wat-scripts/scratch-pad/arc278-l2-3-stratify-numbers.wat	OK	-	52	0	1	
wat-scripts/scratch-pad/arc278-produced-type-userfn-facts.wat	OK	-	82	0	1	
wat-scripts/scratch-pad/arc278-produced-type-userfn-head.wat	OK	-	23	0	1	
wat-scripts/scratch-pad/arc278-shutdown-cohort-probe-process.wat	OK	-	21	0	1	
wat-scripts/scratch-pad/arc278-shutdown-cohort-probe-thread.wat	OK	-	20	0	1	
wat-scripts/scratch-pad/bench-118B5-into-stream-vs-native-concat.wat	OK	-	52	0	1	
wat-scripts/scratch-pad/bench-118B6-native-foldl-vs-wat-spec.wat	OK	-	36	0	1	
wat-scripts/scratch-pad/bench-118B7-reduce-collapse.wat	OK	-	43	0	1	
wat-scripts/scratch-pad/bench-arc278-session-origin-insert-door.wat	OK	-	54	0	1	
wat-scripts/scratch-pad/bench-reduce-foldl-vs-seqable-walk.wat	OK	-	39	0	1	
wat-scripts/scratch-pad/bench-surface-dispatch-cost.wat	OK	-	47	0	1	
wat-scripts/scratch-pad/census-defclause-arm-overlap.wat	OK	-	103	0	1	
wat-scripts/scratch-pad/census-first-of-drop.wat	OK	-	68	0	1	
wat-scripts/scratch-pad/census-growing-collection-in-a-lazy-walk.wat	OK	-	104	0	1	
wat-scripts/scratch-pad/census-join-scope-where.wat	OK	-	334	0	1	
wat-scripts/scratch-pad/census-one-param-spec.wat	OK	-	413	0	1	
wat-scripts/scratch-pad/census-parametric-surface-bindings.wat	OK	-	58	0	1	
wat-scripts/scratch-pad/census-three-call-stream-walks.wat	OK	-	82	0	1	
wat-scripts/scratch-pad/circumspicere-recon/probe-metadata-of-i64-plus.wat	OK	-	6	0	1	
wat-scripts/scratch-pad/circumspicere-recon/probe-metadata-of-user-form.wat	OK	-	7	0	1	
wat-scripts/scratch-pad/circumspicere-recon/probe-show-source-special-form.wat	OK	-	4	0	1	
wat-scripts/scratch-pad/circumspicere-recon/probe-special-form-arity.wat	OK	-	9	0	1	
wat-scripts/scratch-pad/d10-then-rhs-is-not-type-checked.wat	OK	-	28	0	1	
wat-scripts/scratch-pad/d11-nested-then-rhs-is-not-type-checked.wat	OK	-	35	0	1	
wat-scripts/scratch-pad/d2-derived-fact-axis.wat	OK	-	144	0	1	
wat-scripts/scratch-pad/d7-pack-width-controls.wat	OK	-	57	0	1	
wat-scripts/scratch-pad/d7-seed-batch-cost.wat	OK	-	51	0	1	
wat-scripts/scratch-pad/d7-two-writers-one-alpha.wat	OK	-	53	0	1	
wat-scripts/scratch-pad/dot-flip-ask-residuals.wat	OK	-	23	0	1	
wat-scripts/scratch-pad/dot-flip-ask-the-substrate.wat	OK	-	17	0	1	
wat-scripts/scratch-pad/dot-flip-derive-renames.wat	OK	-	29	0	1	
wat-scripts/scratch-pad/dot-flip-tests-defenum-pairs.wat	OK	-	110	0	1	
wat-scripts/scratch-pad/experiri-then/then-calib-i64gt-fire.wat	OK	-	36	0	1	
wat-scripts/scratch-pad/experiri-then/then-calib-i64gt-refuse.wat	OK	-	36	0	1	
wat-scripts/scratch-pad/experiri-then/then-calib-stringeq-fire.wat	OK	-	36	0	1	
wat-scripts/scratch-pad/experiri-then/then-calib-stringeq-refuse.wat	OK	-	36	0	1	
wat-scripts/scratch-pad/experiri-then/then-pv-length.wat	OK	-	37	0	1	
wat-scripts/scratch-pad/j2-holon-rete-classify.wat	OK	-	76	0	1	
wat-scripts/scratch-pad/probe-118B-dorun-retention-slope.wat	OK	-	14	0	1	
wat-scripts/scratch-pad/probe-118B-match-no-tco-control.wat	OK	-	13	0	1	
wat-scripts/scratch-pad/probe-118B-match-tco-drain.wat	OK	-	14	0	1	
wat-scripts/scratch-pad/probe-118B-memo-state-detector.wat	OK	-	10	0	1	
wat-scripts/scratch-pad/probe-118B-six-walkers-baseline.wat	OK	-	82	0	1	
wat-scripts/scratch-pad/probe-118B1-seqable.wat	OK	-	46	0	1	
wat-scripts/scratch-pad/probe-118B2-laziness-all-verbs.wat	OK	-	46	0	1	
wat-scripts/scratch-pad/probe-118B2-one-clause-lazy-producer.wat	OK	-	65	0	1	
wat-scripts/scratch-pad/probe-118B2-rider-verification.wat	OK	-	78	0	1	
wat-scripts/scratch-pad/probe-118B2-stream-pvec-tco-live.wat	OK	-	8	0	1	
wat-scripts/scratch-pad/probe-118B4-forces-per-element-by-walk-shape.wat	OK	-	23	0	1	
wat-scripts/scratch-pad/probe-118B8-dorun-effect-count.wat	OK	-	8	0	1	
wat-scripts/scratch-pad/probe-118B8-dorun-retention.wat	OK	-	18	0	1	
wat-scripts/scratch-pad/probe-198-ctor-macro-span.wat	OK	-	43	0	1	
wat-scripts/scratch-pad/probe-251-8c-matrix.wat	OK	-	1	0	1	
wat-scripts/scratch-pad/probe-251-keyword-vs-colon-quoted-symbol.wat	OK	-	11	0	1	
wat-scripts/scratch-pad/probe-255-72-signature-of-defn.wat	OK	-	50	0	1	
wat-scripts/scratch-pad/probe-255-defn-metadata-map.wat	OK	-	23	0	1	
wat-scripts/scratch-pad/probe-279.3-join-renders-its-elements.wat	OK	-	11	0	1	
wat-scripts/scratch-pad/probe-2a2-enum-fields.wat	OK	-	21	0	1	
wat-scripts/scratch-pad/probe-300-C5c-nan-unordered-gate.wat	OK	-	38	0	1	
wat-scripts/scratch-pad/probe-accumulate-gather-cost.wat	OK	-	179	0	1	
wat-scripts/scratch-pad/probe-arc255-66-the-534.wat	OK	-	0	0	1	
wat-scripts/scratch-pad/probe-arc255-Eii-vectors-acceptance.wat	OK	-	66	0	1	
wat-scripts/scratch-pad/probe-arc255-Eii-vectors-new-spelling.wat	OK	-	13	0	1	
wat-scripts/scratch-pad/probe-arc255-p6c-w3-runtime-reflection.wat	OK	-	32	0	1	
wat-scripts/scratch-pad/probe-arc278-57-enum-equality.wat	OK	-	9	0	1	
wat-scripts/scratch-pad/probe-arc278-74-literal-rtl-ctor.wat	OK	-	17	0	1	
wat-scripts/scratch-pad/probe-arc278-child-entry-static-call.wat	OK	-	53	0	1	
wat-scripts/scratch-pad/probe-arc278-edn-validate.wat	OK	-	25	0	1	
wat-scripts/scratch-pad/probe-arc278-fnforms-reaches-program-types.wat	OK	-	40	0	1	
wat-scripts/scratch-pad/probe-arc278-fnforms-walks-a-matches-pattern.wat	OK	-	22	0	1	
wat-scripts/scratch-pad/probe-arc278-fnforms-walks-into-quoted-data.wat	OK	-	18	0	1	
wat-scripts/scratch-pad/probe-arc278-forms-block-is-inert.wat	OK	-	13	0	1	
wat-scripts/scratch-pad/probe-arc278-free-user-name-in-parent-defn.wat	OK	-	30	0	1	
wat-scripts/scratch-pad/probe-arc278-invocation-family.wat	OK	-	27	0	1	
wat-scripts/scratch-pad/probe-arc278-let-tail-service-reaped.wat	OK	-	33	0	1	
wat-scripts/scratch-pad/probe-arc278-lift-mention-ships-transitively.wat	OK	-	46	0	1	
wat-scripts/scratch-pad/probe-arc278-macro-mints-a-lifted-toplevel-defn.wat	OK	-	17	0	1	
wat-scripts/scratch-pad/probe-arc278-nullary-enum-process-repro.wat	OK	-	33	0	1	
wat-scripts/scratch-pad/probe-arc278-parametric-surface-messages.wat	OK	-	7	0	1	
wat-scripts/scratch-pad/probe-arc278-reap-serve-event.wat	OK	-	65	0	1	
wat-scripts/scratch-pad/probe-arc278-reap-which-link.wat	OK	-	108	0	1	
wat-scripts/scratch-pad/probe-arc278-rules-cross-the-wire.wat	OK	-	114	0	1	
wat-scripts/scratch-pad/probe-arc278-rules-ship-as-declared-payload.wat	OK	-	73	0	1	
wat-scripts/scratch-pad/probe-arc278-stop0-lost-carries-failure.wat	OK	-	15	0	1	
wat-scripts/scratch-pad/probe-arc278-surface-registers-service-reads.wat	OK	-	10	0	1	
wat-scripts/scratch-pad/probe-arc278-symbol-head-form.wat	OK	-	19	0	1	
wat-scripts/scratch-pad/probe-arc278-tco-drops-caller-env.wat	OK	-	81	0	1	
wat-scripts/scratch-pad/probe-arc278-union-closure-boots-a-process-child.wat	OK	-	161	0	1	
wat-scripts/scratch-pad/probe-arc278-watast-identity-arm.wat	OK	-	31	0	1	
wat-scripts/scratch-pad/probe-arc278-watast-on-the-wire-decomposed.wat	OK	-	112	0	1	
wat-scripts/scratch-pad/probe-arc278-where-body-dep-not-shipped.wat	OK	-	37	0	1	
wat-scripts/scratch-pad/probe-arc278-who-diagnoses-a-bad-rule.wat	OK	-	30	0	1	
wat-scripts/scratch-pad/probe-arc278-wire-dos-service-killed.wat	OK	-	56	0	1	
wat-scripts/scratch-pad/probe-arc278-wire-type-enforcement-detonate.wat	OK	-	46	0	1	
wat-scripts/scratch-pad/probe-arc278-wire-type-enforcement.wat	OK	-	59	0	1	
wat-scripts/scratch-pad/probe-arc296-builtin-redeclared-in-wat.wat	OK	-	1	0	1	
wat-scripts/scratch-pad/probe-ast-span-totality-under-reader-macros.wat	OK	-	52	0	1	
wat-scripts/scratch-pad/probe-bracket-label-real.wat	OK	-	7	0	1	
wat-scripts/scratch-pad/probe-brief-f64-surface-is-a-stub.wat	OK	-	26	0	1	
wat-scripts/scratch-pad/probe-brief-first-nth-to-string.wat	OK	-	24	0	1	
wat-scripts/scratch-pad/probe-brief-get-is-total-by-fallback.wat	OK	-	97	0	1	
wat-scripts/scratch-pad/probe-call-site-frame.wat	OK	-	10	0	1	
wat-scripts/scratch-pad/probe-call-site-kwargs.wat	OK	-	12	0	1	
wat-scripts/scratch-pad/probe-call-site-through-macro.wat	OK	-	14	0	1	
wat-scripts/scratch-pad/probe-clause-tco-deep-defclause.wat	OK	-	9	0	1	
wat-scripts/scratch-pad/probe-clause-tco-deep-defn.wat	OK	-	9	0	1	
wat-scripts/scratch-pad/probe-clause-tco-ensure-still-fires.wat	OK	-	10	0	1	
wat-scripts/scratch-pad/probe-clause-tco-guard-selects.wat	OK	-	9	0	1	
wat-scripts/scratch-pad/probe-compose-variant-corpus-loads.wat	OK	-	1	0	1	
wat-scripts/scratch-pad/probe-cond-in-a-then.wat	OK	-	30	0	1	
wat-scripts/scratch-pad/probe-cond-in-where-baseline.wat	OK	-	29	0	1	
wat-scripts/scratch-pad/probe-cond-rete-expands-rete-spelled.wat	OK	-	17	0	1	
wat-scripts/scratch-pad/probe-cond-rete-scorecard.wat	OK	-	65	0	1	
wat-scripts/scratch-pad/probe-cond-rete-where.wat	OK	-	32	0	1	
wat-scripts/scratch-pad/probe-dash-variant-and-roundtrip.wat	OK	-	7	0	1	
wat-scripts/scratch-pad/probe-defclause-refusal-restored.wat	OK	-	17	0	1	
wat-scripts/scratch-pad/probe-derive-chain-split.wat	OK	-	105	0	1	
wat-scripts/scratch-pad/probe-derive-decomposition.wat	OK	-	89	0	1	
wat-scripts/scratch-pad/probe-destructure-string-key.wat	OK	-	10	0	1	
wat-scripts/scratch-pad/probe-dotted-eval.wat	OK	-	16	0	1	
wat-scripts/scratch-pad/probe-durable-forms-vector.wat	OK	-	9	0	1	
wat-scripts/scratch-pad/probe-eq-generic-instantiation.wat	OK	-	2	0	1	
wat-scripts/scratch-pad/probe-execve-argv-cow-leak.wat	OK	-	25	0	1	
wat-scripts/scratch-pad/probe-export-roundtrip-holon-literal-and-hashdestructure.wat	OK	-	57	0	1	
wat-scripts/scratch-pad/probe-f64-fallback-rows.wat	OK	-	38	0	1	
wat-scripts/scratch-pad/probe-failure-record-ctor.wat	OK	-	6	0	1	
wat-scripts/scratch-pad/probe-fixpoint-finite-domain-bool.wat	OK	-	25	0	1	
wat-scripts/scratch-pad/probe-four-homes-census.wat	OK	-	64	0	1	
wat-scripts/scratch-pad/probe-grep-cli.wat	OK	-	21	0	1	
wat-scripts/scratch-pad/probe-grep-driver.wat	OK	-	24	0	1	
wat-scripts/scratch-pad/probe-hashset-and-linkedlist-homes.wat	OK	-	45	0	1	
wat-scripts/scratch-pad/probe-holon-classifier-shape-mismatch.wat	OK	-	9	0	1	
wat-scripts/scratch-pad/probe-holon-classifier-space.wat	OK	-	22	0	1	
wat-scripts/scratch-pad/probe-holon-is-list-and-tag.wat	OK	-	13	0	1	
wat-scripts/scratch-pad/probe-holon-is-map-the-vsa-way.wat	OK	-	30	0	1	
wat-scripts/scratch-pad/probe-holon-is-tag.wat	OK	-	16	0	1	
wat-scripts/scratch-pad/probe-holon-rete-cell-values.wat	OK	-	46	0	1	
wat-scripts/scratch-pad/probe-holon-shape-literals.wat	OK	-	27	0	1	
wat-scripts/scratch-pad/probe-holon-two-lift-paths.wat	OK	-	19	0	1	
wat-scripts/scratch-pad/probe-home-8-examples.wat	OK	-	212	0	1	
wat-scripts/scratch-pad/probe-home13-substrate-impl-apply-fallback.wat	OK	-	10	0	1	
wat-scripts/scratch-pad/probe-inline-constraint-bypasses-law-a.wat	OK	-	36	0	1	
wat-scripts/scratch-pad/probe-insert-all-cost.wat	OK	-	61	0	1	
wat-scripts/scratch-pad/probe-insert-cost-split.wat	OK	-	79	0	1	
wat-scripts/scratch-pad/probe-into-is-quadratic.wat	OK	-	38	0	1	
wat-scripts/scratch-pad/probe-json-natural-record.wat	OK	-	10	0	1	
wat-scripts/scratch-pad/probe-kwargs-stack-shape.wat	OK	-	15	0	1	
wat-scripts/scratch-pad/probe-label-closed-set.wat	OK	-	9	0	1	
wat-scripts/scratch-pad/probe-large-float-edn-round-trip.wat	OK	-	4	0	1	
wat-scripts/scratch-pad/probe-macro-emitted-call-site.wat	OK	-	11	0	1	
wat-scripts/scratch-pad/probe-match-hash-destructure-core-vs-rete.wat	OK	-	10	0	1	
wat-scripts/scratch-pad/probe-match-hash-destructure-in-rete.wat	OK	-	36	0	1	
wat-scripts/scratch-pad/probe-mcp-nested-json-walk.wat	OK	-	15	0	1	
wat-scripts/scratch-pad/probe-mcp-reply-emit.wat	OK	-	8	0	1	
wat-scripts/scratch-pad/probe-mcp-response-shape.wat	OK	-	14	0	1	
wat-scripts/scratch-pad/probe-mcp-stone1-read-json-gate.wat	OK	-	23	0	1	
wat-scripts/scratch-pad/probe-mcp-wire.wat	OK	-	10	0	1	
wat-scripts/scratch-pad/probe-node-share-dedup.wat	OK	-	62	0	1	
wat-scripts/scratch-pad/probe-node-share-phase-split.wat	OK	-	97	0	1	
wat-scripts/scratch-pad/probe-optA-retag.wat	OK	-	160	0	1	
wat-scripts/scratch-pad/probe-ordering-is-partial-via-sort.wat	OK	-	5	0	1	
wat-scripts/scratch-pad/probe-overlay-refire-cost.wat	OK	-	109	0	1	
wat-scripts/scratch-pad/probe-per-type-equality-restored.wat	OK	-	29	0	1	
wat-scripts/scratch-pad/probe-purity-core-accessor.wat	OK	-	28	0	1	
wat-scripts/scratch-pad/probe-purity-facets.wat	OK	-	30	0	1	
wat-scripts/scratch-pad/probe-pv-lazy-materialize-cost.wat	OK	-	56	0	1	
wat-scripts/scratch-pad/probe-qq-arm-shape-src.wat	OK	-	0	0	0	
wat-scripts/scratch-pad/probe-qq-arm-shape.wat	OK	-	38	0	1	
wat-scripts/scratch-pad/probe-reland10-fire-direct.wat	OK	-	75	0	1	
wat-scripts/scratch-pad/probe-reland10-journal-then-fire.wat	OK	-	106	0	1	
wat-scripts/scratch-pad/probe-reland10-session-in-struct.wat	OK	-	48	0	1	
wat-scripts/scratch-pad/probe-reland4-eval-with-defs-type-of.wat	OK	-	12	0	1	
wat-scripts/scratch-pad/probe-reland4-sift-type-of.wat	OK	-	22	0	1	
wat-scripts/scratch-pad/probe-reland8-sort-by-row-sk.wat	OK	-	4	0	1	
wat-scripts/scratch-pad/probe-reland8-type-of-campaign-enums.wat	OK	-	175	0	1	
wat-scripts/scratch-pad/probe-reland9-expand-callctx.wat	OK	-	18	0	1	
wat-scripts/scratch-pad/probe-reland9-expand-journal.wat	OK	-	18	0	1	
wat-scripts/scratch-pad/probe-reland9-expand-sift.wat	OK	-	18	0	1	
wat-scripts/scratch-pad/probe-reland9-sift-fire.wat	OK	-	35	0	1	
wat-scripts/scratch-pad/probe-repl-declaration-refusal.wat	OK	-	19	0	1	
wat-scripts/scratch-pad/probe-repl-eval-in-gap.wat	OK	-	25	0	1	
wat-scripts/scratch-pad/probe-rete-if-in-where.wat	OK	-	29	0	1	
wat-scripts/scratch-pad/probe-rhs-builds-core-span.wat	OK	-	30	0	1	
wat-scripts/scratch-pad/probe-room4-cek-stepper-qualified-scrutinee.wat	OK	-	12	0	1	
wat-scripts/scratch-pad/probe-room4-value-position-none.wat	OK	-	8	0	1	
wat-scripts/scratch-pad/probe-rule-defn-shape-b.wat	OK	-	72	0	1	
wat-scripts/scratch-pad/probe-rule-defn-shape.wat	OK	-	100	0	1	
wat-scripts/scratch-pad/probe-rule-lits.wat	OK	-	102	0	1	
wat-scripts/scratch-pad/probe-rules-cascade.wat	OK	-	64	0	1	
wat-scripts/scratch-pad/probe-rules-core.wat	OK	-	64	0	1	
wat-scripts/scratch-pad/probe-runtime-qualified-builtin-variant.wat	OK	-	21	0	1	
wat-scripts/scratch-pad/probe-s5-tail-position-is-load-bearing.wat	OK	-	21	0	1	
wat-scripts/scratch-pad/probe-seed-insert-vs-insert-all.wat	OK	-	56	0	1	
wat-scripts/scratch-pad/probe-selectables-homogeneity.wat	OK	-	156	0	1	
wat-scripts/scratch-pad/probe-self-scheduling-loop.wat	OK	-	30	0	1	
wat-scripts/scratch-pad/probe-send-outcome-wall.wat	OK	-	23	0	1	
wat-scripts/scratch-pad/probe-seqable-is-spellable-today.wat	OK	-	12	0	1	
wat-scripts/scratch-pad/probe-seqable-to-stream-native-check.wat	OK	-	87	0	1	
wat-scripts/scratch-pad/probe-sift-body-direct.wat	OK	-	113	0	1	
wat-scripts/scratch-pad/probe-sift-expand.wat	OK	-	21	0	1	
wat-scripts/scratch-pad/probe-sift-expand2.wat	OK	-	18	0	1	
wat-scripts/scratch-pad/probe-sift-predicate-crux.wat	OK	-	14	0	1	
wat-scripts/scratch-pad/probe-sift-rules-stop1-bare-defsurface.wat	OK	-	13	0	1	
wat-scripts/scratch-pad/probe-sift-rules-stop1-dump.wat	OK	-	30	0	1	
wat-scripts/scratch-pad/probe-slice-one-registry-seam.wat	OK	-	35	0	1	
wat-scripts/scratch-pad/probe-span-narrower-than-name.wat	OK	-	22	0	1	
wat-scripts/scratch-pad/probe-span-substitution-exemplar.wat	OK	-	6	0	1	
wat-scripts/scratch-pad/probe-splice-strings.wat	OK	-	14	0	1	
wat-scripts/scratch-pad/probe-stdio-concurrent-dial.wat	OK	-	119	0	1	
wat-scripts/scratch-pad/probe-stone-2a-bracket-mechanics.wat	OK	-	18	0	1	
wat-scripts/scratch-pad/probe-stone-e-iv-keyword-new-home.wat	OK	-	27	0	1	
wat-scripts/scratch-pad/probe-strat2-derived-join-base.wat	OK	-	73	0	1	
wat-scripts/scratch-pad/probe-string-home-completeness.wat	OK	-	14	0	1	
wat-scripts/scratch-pad/probe-string-subs-fallback-row.wat	OK	-	26	0	1	
wat-scripts/scratch-pad/probe-timer-as-peer.wat	OK	-	77	0	1	
wat-scripts/scratch-pad/probe-tuple-observability.wat	OK	-	22	0	1	
wat-scripts/scratch-pad/probe-unquote-is-gone-before-the-fence.wat	OK	-	10	0	1	
wat-scripts/scratch-pad/probe-vec-arg-defrecord.wat	OK	-	12	0	1	
wat-scripts/scratch-pad/probe-vector-fn-type-bracket.wat	OK	-	3	0	1	
wat-scripts/scratch-pad/probe-vsa-seam-rete-rows.wat	OK	-	31	0	1	
wat-scripts/scratch-pad/probe-where-before-fact-condition.wat	OK	-	32	0	1	
wat-scripts/scratch-pad/probe-where-census-walker.wat	OK	-	148	0	1	
wat-scripts/scratch-pad/probe-where-fallback-op-does-not-raise.wat	OK	-	33	0	1	
wat-scripts/scratch-pad/probe-written-admits-fqdn-keyword-tokens.wat	OK	-	41	0	1	
wat-scripts/scratch-pad/probe-zero-magnitude-reachable.wat	OK	-	23	0	1	
wat-scripts/scratch-pad/rules-corpus-01-node-facts.wat	OK	-	79	0	1	
wat-scripts/scratch-pad/rules-corpus-02-gates-and-unknowns.wat	OK	-	122	0	1	
wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat	OK	-	171	0	1	
wat-scripts/scratch-pad/scout-ab-name.wat	OK	-	24	0	1	
wat-scripts/scratch-pad/scout-cap-ednwire.wat	OK	-	11	0	1	
wat-scripts/scratch-pad/scout-cap-head.wat	OK	-	28	0	1	
wat-scripts/scratch-pad/scout-cap-organic.wat	OK	-	21	0	1	
wat-scripts/scratch-pad/scout-cap-pure.wat	OK	-	23	0	1	
wat-scripts/scratch-pad/scout-cap-pure2.wat	OK	-	24	0	1	
wat-scripts/scratch-pad/scout-cap-pure3.wat	OK	-	16	0	1	
wat-scripts/scratch-pad/scout-cap-roundtrip.wat	OK	-	21	0	1	
wat-scripts/scratch-pad/scout-eval-1-why.wat	OK	-	15	0	1	
wat-scripts/scratch-pad/scout-eval-2-form.wat	OK	-	8	0	1	
wat-scripts/scratch-pad/scout-eval-3-unwrap.wat	OK	-	22	0	1	
wat-scripts/scratch-pad/scout-eval-4-record.wat	OK	-	31	0	1	
wat-scripts/scratch-pad/scout-eval-5-accessor.wat	OK	-	22	0	1	
wat-scripts/scratch-pad/scout-eval-6-defn.wat	OK	-	13	0	1	
wat-scripts/scratch-pad/t-a.wat	OK	-	4	0	1	
wat-scripts/scratch-pad/t-b.wat	OK	-	5	0	1	
wat-scripts/scratch-pad/t-bare.wat	OK	-	1	0	1	
wat-scripts/scratch-pad/t-c.wat	OK	-	6	0	1	
wat-scripts/scratch-pad/t-main-panic.wat	OK	-	8	0	1	
wat-scripts/scratch-pad/t-nomain.wat	OK	-	1	0	1	
wat-scripts/scratch-pad/wat-grep-with-network-shape.wat	OK	-	92	0	1	
wat-scripts/scratch-pad/wb5-witness-thread-last-macroexpand.wat	OK	-	6	0	1	
wat-scripts/scratch-pad/wb5-witness-thread-macroexpand.wat	OK	-	14	0	1	
wat-tests/bracket.wat	OK	-	43	0	1	
wat-tests/cache/HolographicLru.wat	OK	-	93	0	1	
wat-tests/core/core-arithmetic.wat	OK	-	207	0	1	
wat-tests/core/core-collection-aliases.wat	OK	-	52	0	1	
wat-tests/core/core-equality.wat	OK	-	47	0	1	
wat-tests/core/core-foldl-spec.wat	OK	-	47	0	1	
wat-tests/core/core-into-persistentvector-from-vector.wat	OK	-	21	0	1	
wat-tests/core/core-nth-differential.wat	OK	-	87	0	1	
wat-tests/core/core-nth-macro-body.wat	OK	-	6	0	1	
wat-tests/core/core-nth.wat	OK	-	78	0	1	
wat-tests/core/core-reduce.wat	OK	-	15	0	1	
wat-tests/core/core-seq-walkers.wat	OK	-	233	0	1	
wat-tests/core/core-seqable.wat	OK	-	63	0	1	
wat-tests/core/core-stream-materializers-differential.wat	OK	-	85	0	1	
wat-tests/core/core-threading.wat	OK	-	68	0	1	
wat-tests/core/expect-no-ascription.wat	OK	-	8	0	1	
wat-tests/core/generic-tuple-infer.wat	OK	-	10	0	1	
wat-tests/core/generic-tuple-nongeneric-baseline.wat	OK	-	8	0	1	
wat-tests/core/generic-tuple-turbofish.wat	OK	-	10	0	1	
wat-tests/core/match-no-ascription.wat	OK	-	4	0	1	
wat-tests/core/option-expect.wat	OK	-	35	0	1	
wat-tests/core/readln-no-ascription.wat	OK	-	11	0	1	
wat-tests/core/record-def.wat	OK	-	76	0	1	
wat-tests/core/result-expect.wat	OK	-	29	0	1	
wat-tests/core/struct-to-form.wat	OK	-	27	0	1	
wat-tests/core/unknown-call-head-panics.wat	OK	-	4	0	1	
wat-tests/counter-actor-proof-process.wat	OK	-	117	0	1	
wat-tests/deporder.wat	OK	-	31	0	1	
wat-tests/edn/render.wat	OK	-	40	0	1	
wat-tests/edn/roundtrip.wat	OK	-	41	0	1	
wat-tests/fix-stdlib-source-path.wat	OK	-	9	0	1	
wat-tests/format.wat	OK	-	19	0	1	
wat-tests/gen-patterns.wat	OK	-	212	0	1	
wat-tests/gen.wat	OK	-	675	0	1	
wat-tests/holon/Amplify.wat	OK	-	16	0	1	
wat-tests/holon/Bigram.wat	OK	-	31	0	1	
wat-tests/holon/Circular.wat	OK	-	12	0	1	
wat-tests/holon/Filter.wat	OK	-	35	0	1	
wat-tests/holon/Hologram.wat	OK	-	153	0	1	
wat-tests/holon/Ngram.wat	OK	-	21	0	1	
wat-tests/holon/ReciprocalLog.wat	OK	-	24	0	1	
wat-tests/holon/Reject.wat	OK	-	24	0	1	
wat-tests/holon/Sequential.wat	OK	-	17	0	1	
wat-tests/holon/Subtract.wat	OK	-	15	0	1	
wat-tests/holon/Trigram.wat	OK	-	22	0	1	
wat-tests/holon/char-round-trip.wat	OK	-	16	0	1	
wat-tests/holon/coincident.wat	OK	-	42	0	1	
wat-tests/holon/eval-coincident.wat	OK	-	66	0	1	
wat-tests/holon/list-round-trip.wat	OK	-	34	0	1	
wat-tests/holon/term.wat	OK	-	121	0	1	
wat-tests/interpolate.wat	OK	-	8	0	1	
wat-tests/kernel/services/ambient-stdio.wat	OK	-	85	0	1	
wat-tests/lint.wat	OK	-	136	0	1	
wat-tests/process/child-is-fresh-universe.wat	OK	-	12	0	1	
wat-tests/process/signal-reset-sigusr1-is-a-transition.wat	OK	-	34	0	1	
wat-tests/process/signal-terminate-kills-the-child-and-the-read-sees-it.wat	OK	-	33	0	1	
wat-tests/process/signal-user1-delivers-child-observes.wat	OK	-	32	0	1	
wat-tests/process/signal-user2-and-hangup-independent.wat	OK	-	33	0	1	
wat-tests/reflect/reflection-surface.wat	OK	-	13	0	1	
wat-tests/reflect/special-form-doc-if-let.wat	OK	-	13	0	1	
wat-tests/rete/differential-fuzz-nesting.wat	OK	-	154	0	1	
wat-tests/rete/differential-fuzz-rules.wat	OK	-	204	0	1	
wat-tests/rete/differential-fuzz-scalars.wat	OK	-	453	0	1	
wat-tests/rete/differential-fuzz-tms.wat	OK	-	206	0	1	
wat-tests/rete/differential-fuzz.wat	OK	-	325	0	1	
wat-tests/run-thread.wat	OK	-	36	0	1	
wat-tests/service-admin-facet.wat	OK	-	65	0	1	
wat-tests/service-cache-hologram.wat	OK	-	94	0	1	
wat-tests/service-cache-lru.wat	OK	-	97	0	1	
wat-tests/service-hibernate-resume.wat	OK	-	98	0	1	
wat-tests/service-init-parity.wat	OK	-	62	0	1	
wat-tests/service-locus-parity.wat	OK	-	87	0	1	
wat-tests/service-multiparam-init.wat	OK	-	66	0	1	
wat-tests/service-parametric-bare-messages.wat	OK	-	50	0	1	
wat-tests/service-parametric-messages.wat	OK	-	85	0	1	
wat-tests/service-parametric-two-params.wat	OK	-	48	0	1	
wat-tests/service-parametric.wat	OK	-	42	0	1	
wat-tests/service-request-malformed.wat	OK	-	56	0	1	
wat-tests/service-signal-observer.wat	OK	-	90	0	1	
wat-tests/service-stop-resp.wat	OK	-	66	0	1	
wat-tests/service-telemetry-bridge.wat	OK	-	168	0	1	
wat-tests/spawn/multiline-roundtrip.wat	OK	-	14	0	1	
wat-tests/spawn/recv-budget-override.wat	OK	-	21	0	1	
wat-tests/test.wat	OK	-	170	0	1	
wat-tests/time.wat	OK	-	276	0	1	
wat-tests/timer-after-process.wat	OK	-	7	0	1	
wat-tests/timer-after.wat	OK	-	7	0	1	
wat-tests/timer-env-grab-parity.wat	OK	-	68	0	1	
wat-tests/timer-family.wat	OK	-	26	0	1	
wat-tests/timer-tier-open.wat	OK	-	7	0	1	
```
