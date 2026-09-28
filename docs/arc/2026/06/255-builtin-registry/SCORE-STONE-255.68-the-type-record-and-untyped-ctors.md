# SCORE — STONE 255.68: the checker's type record on `main`, and every untyped constructor call

**Executor: a Sonnet subagent.** Commits `faff7e384` (part 1) and this SCORE (part 2). Floor run alone,
in the foreground, once, per the orchestrator's correction after an earlier STOP-2 turned out to be two
concurrent floor runs self-inflicted by this executor (`.floor/2026-09-28T18-30-40Z` and
`.floor/2026-09-28T18-30-51Z`, started 11s apart) — recorded here so the false red is not mistaken for a
real one later.

## Part 1 — the type record (landed, `faff7e384`)

Ported fresh from `the-little-wat`'s `bbfac2ee8` (`origin/the-little-wat`, read-only there, never merged /
checked out / cherry-picked) onto `main`. Same names (`crate::check::type_record`), same env var
(`WAT_CHECK_TYPES`), same output shape. **No STOP-1** — every symbol the port needed
(`check_function_body`, `check_form`, `infer`, `infer_defclause`, `extract_def_binding`,
`check_program`, `FrozenWorld::{program,symbols,types}`, `Subst`, `reduce`, `format_type`,
`TypeEnv::enclosing_enum`, `parametric_head_fqdn`, `Span{file,line,col}`) still had the exact
name/signature the commit used; `src/distribution/check_output.rs` was byte-identical to the branch's
pre-commit copy.

**What it does.** `pub(crate) fn infer` was split into a thin wrapper — notes `(span, type)` via
`type_record::note` when recording is on, then delegates to a renamed `infer_node` (the old dispatch
body, unchanged) — so every recursive call site, unchanged, still routes through the note-taking
wrapper. `open_root`/`close_root` bracket each inference root that owns a `Subst`
(`check_function_body`, `check_form`, each `infer_defclause` clause incl. both early `continue` exits,
`extract_def_binding`), so a note resolves through that root's FINAL substitution, never mid-inference.
Unset, `dump_types` is `false` and nothing beyond `startup_from_source`'s existing result runs — `--check`
is byte-for-byte what it was.

**Proved additive:**
- `tests/cli/type_record.rs::type_record_unset_output_is_unchanged` — `WAT_CHECK_TYPES` unset, rc 0,
  empty stdout AND stderr on a well-typed fixture.
- `tests/cli/type_record.rs::type_record_records_element_type_from_call_site` — `WAT_CHECK_TYPES=1` on
  `tests/cli/wat_cli__check_types.wat` (declares `:user::takes-vec` with a
  `(wat.type/PersistentVector :- [wat.type/i64])`-typed parameter; `:user::main` calls it with a bare
  `(:wat::core::PersistentVector)`). The recorded node at `11:23` is compared, structurally, against a
  captured `.edn` golden (`tests/cli/wat_cli__check_types__recorded_type.edn`,
  `wat::assert_edn_matches_file!`) — `:tag "TYPE"` (fully resolved, not left a unification variable),
  `:ty`/`:wide` both `(:wat::core::PersistentVector :- [:wat::core::i64])` (i64 came from the checker,
  from the declared param type at the call site — the argument's own text names no element type at
  all), `:check-errors 0`.

**F-196** (cannot type inside a function body spelled with namespaced symbols, pre-normalization):
carried over unchanged from the ported commit; not independently re-verified beyond the port being
byte-identical to the branch's own text on this point. Part 2's measurement below surfaced a
**second, distinct** category of "not checked" beyond F-196 — see § The 141 `not-checked` sites below.

**Gates (this run, alone, foreground, waited to completion):**
- `cargo build --release` — clean.
- `cargo clippy --release --all-targets -- -D warnings` — `rc 0`.
- `scripts/floor.sh`, single run: `.floor/2026-09-28T19-04-14Z/` —
  **`Summary [ 379.425s] 6215 tests run: 6215 passed (22 slow), 24 skipped`**, `exit=0`. (6213 baseline
  at `ed5d7749b` + this stone's 2 new tests = 6215; matches exactly.)

**Arms A and B, found and fixed before the green floor above** (an earlier run, concurrent with a second
floor by mistake, is the false-red the orchestrator corrected — its findings on these two arms were
real and are fixed here regardless of the concurrency defect):
- **Arm A** — `tests/cli/type_record.rs` tripped `no_inlined_wat_in_tests` (an `assert_eq!` literal
  `"(:wat::core::PersistentVector :- [:wat::core::i64])"` is wat-reader-shaped text inlined in `.rs`)
  and `no_loose_string_assert` (two `.starts_with`/`.ends_with` assertions). Cured per
  `docs/CONVENTIONS.md` § 'Test idioms' → 'The `.edn` golden': the recorded node's six fields plus the
  trailer's `check-errors` count are assembled into one small EDN map at runtime (braces via
  `char::push`, never a `"{...}"` string literal — that in turn tripped a THIRD lint,
  `no_inlined_edn`, cured the same way) and compared via `wat::assert_edn_matches_file!` against a
  captured golden. No `rune:lint` exemptions used anywhere in this file.
- **Arm B** — `tests/lint/keyword_heresy_ledger.rs`'s frozen census keys per-function; the `infer` →
  `infer_node` split moved 2 pre-existing keyword-literal comparisons (`:wat::core::Option.None`,
  `:wat::core::nil` — unrelated to type_record, just code that happened to live in the function body
  that got renamed) from the `"infer"` row to a new `"infer_node"` row. Ledger total unchanged: **148**.

## Part 2 — every untyped collection constructor call (measurement only; no `src/` change)

### Scope

`wat/`, plus every tracked `*.wat` / `*.wat.bad`, excluding `wat-scripts/fixes/**` and `*.wat.golden`
(`git ls-files 'wat/**/*.wat' '*.wat' '*.wat.bad'` filtered) — **2545 files**.

### Method (re-derived from scratch — the WEIGH's own instruction: "do not trust these numbers")

**1. The untyped-call census is AST-exact, not regex.** A throwaway example binary (not committed —
Part 2 lands only this SCORE) walked each file's real parsed AST via `wat_reader::parser::parse_all_with_file`
and its own `WatAST::List`/`Vector`/`Map`/`Set` recursion — comments and strings are structurally
invisible to a parser, closing exactly the false-positive class the WEIGH named ("keyword-node",
`Record/same-data?`-shaped text counted as calls). A hit is a `List` node whose head is one of the five
constructor names — either spelling (`:wat::core::X` keyword, still what constructor CALLS use per the
residue table, or the post-255.67 `wat.type/X` faithful symbol) — where `items[1]` is **not** the `:-`
binder keyword. Output: `head\tfile\tline\tcol`, one per hit.

  **Result: 1804 untyped constructor calls** (vs. the WEIGH's regex-derived ~1800 — close, as expected,
  but not identical; re-derived, not reused). 9 files could not even be parsed (all `.wat.bad`, all
  intentional lex/parse-negative fixtures — their sites are uncountable, correctly excluded from the
  1804 and folded into "not checked" nowhere, since a census can't count inside a file it can't read).

  | head | count |
  |---|---|
  | PersistentVector | 1339 |
  | Tuple | 237 |
  | PersistentMap | 122 |
  | List | 96 |
  | Vector | 10 |
  | **total** | **1804** |

**2. The type record, run once per corpus file, unioned.** `WAT_CHECK_TYPES=1 wat --check <file>` for
each of the 2545 files (parallel, `-P 16`, ~109s wall for the whole corpus), filtered to lines whose
recorded type's head is one of the five constructor names **in either rendering** — the nominal
`(:wat::core::X :- […])` shape AND the native tuple shape `:(A,B,…)` (`TypeExpr::Tuple` renders via
`format_type`'s own dedicated arm, `check.rs` ~18321-18328, as the compact colon-paren form, never as
`(:wat::core::Tuple :- […])` — missing this on the first pass silently zeroed out every Tuple site;
caught by noticing ALL 237 Tuple and ALL 10 Vector sites read "not checked" and manually tracing one).
Every file is loaded standalone as its own `--check` entry (most also re-load the stdlib and whatever
else they depend on). Of the 2545, **1941 freeze successfully as their own entry, 604 do not** (602
ordinary freeze errors + 2 that panic at freeze-time by design — `tests/cli/wat_cli__freeze_time_panic.wat`
and `tests/kernel/probe_arc109_assertion_kwargs__positional.wat` deliberately test a freeze-time
`assertion-failed!`; both are `not-checked`-correct, never re-run, panic captured verbatim in stderr).
The same node can appear in many runs regardless; the runs are unioned and deduplicated
by `(file, line, col, tag, ty, wide)` — **12,718 unique constructor-headed type records** across the
corpus's own files (one further record, at `src/declare/register.rs`, is a Rust-side synthetic
registration span, not a corpus file — excluded).

**3. Join.** Each of the 1804 untyped-call sites is looked up by exact `(file, line, col)` against the
12,718 type records. A site with no match at that exact position is **not checked**; 0 sites in this
corpus recorded more than one distinct type at the same position (no `MULTI` case).

**4. Classification** (concrete / generic) reads the resolved type's own element argument(s) — inside
the `:- […]` bracket for a nominal head, or the comma-separated elements for a native tuple, recursing
into nested compounds. An argument is a **rigid type parameter** (→ `generic`) iff its own head-segment
carries no `::` at all (matches `check.rs`'s own representation: `Path(":T")`, bare, for a declared
type-parameter letter — every real type name in this corpus is fully-qualified, `wat::core::i64` /
`wat::WatAST` / `u::In`, always at least one `::`). Two regex-based first attempts at this over- and
under-counted (a hyphenated namespace segment like `wat-tests` broke a naive `[A-Za-z0-9_]+` char
class; the native tuple shape's un-colon-prefixed elements, e.g. `probe::CallCtx3` inside
`:(wat::core::i64,probe::CallCtx3,probe::CallCtx3)`, tokenized to a bare trailing segment and
false-flagged as generic) — both errors found by hand-inspecting every "generic" hit before trusting
the count; the final pass is a per-position recursive parser, not a regex.

### Result

| class | count | goes to |
|---|---|---|
| **concrete** | 1627 | the codemod writes `(wat.type/X :- [that-exact-type] …)` |
| **generic** | 3 | the codemod writes `:- [T]` (the enclosing definition's own type-parameter letter) |
| **unresolved** | 33 | still a variable at the checker's own final substitution; a person decides |
| **not-checked** | 141 | the file doesn't check, or the node is data (quote/forms/macro template — see below), not code |
| **total** | **1804** | |

Per head × class:

| head | concrete | generic | unresolved | not-checked | total |
|---|---:|---:|---:|---:|---:|
| PersistentVector | 1294 | 3 | 6 | 36 | 1339 |
| Tuple | 164 | 0 | 0 | 73 | 237 |
| PersistentMap | 95 | 0 | 17 | 10 | 122 |
| List | 74 | 0 | 10 | 12 | 96 |
| Vector | 0 | 0 | 0 | 10 | 10 |

### The 3 `generic` sites (full list)

- PersistentVector `wat/gen.wat:952:25` → `(:wat::core::PersistentVector :- [(:wat::gen::Gen :- [:B])])`
- PersistentVector `wat/gen.wat:983:31` → `(:wat::core::PersistentVector :- [:T])`
- PersistentVector `wat/seq.wat:133:36` → `(:wat::core::PersistentVector :- [:T])`

### The 33 `unresolved` sites (full list — checker's element var never got pinned by anything in scope)

- List `tests/collection/list.wat:11:29`
- List `tests/collection/list.wat:17:29`
- List `tests/collection/list.wat:20:29`
- List `tests/collection/probe_collection_transform_ops.wat:10:23`
- List `tests/collection/probe_collection_transform_ops.wat:18:23`
- PersistentMap `tests/rete/probe_arc278_1a_data_model.wat:11:10`
- PersistentMap `tests/rete/probe_arc278_1a_data_model.wat:21:10`
- PersistentVector `tests/rete/probe_arc278_export.wat:40:3`
- PersistentMap `wat-scripts/scratch-pad/255-stone-e-i-both-map-spellings.wat:17:49`
- PersistentVector `wat-scripts/scratch-pad/255-stone-o-iii-six-verbs-direct.wat:21:112`
- PersistentMap `wat-scripts/scratch-pad/255-stone-o-iv-b-collections-sweep-direct.wat:36:134`
- List `wat-scripts/scratch-pad/255-stone-o-iv-b-collections-sweep-direct.wat:75:10`
- List `wat-scripts/scratch-pad/255-stone-o-iv-d-direct-calls.wat:14:25`
- PersistentVector `wat-scripts/scratch-pad/probe-arc255-Eii-vectors-acceptance.wat:12:14`
- List `wat-scripts/scratch-pad/probe-hashset-and-linkedlist-homes.wat:22:54`
- List `wat-scripts/scratch-pad/probe-hashset-and-linkedlist-homes.wat:24:54`
- List `wat-tests/holon/list-round-trip.wat:21:9`
- PersistentMap `wat/rete/compile.wat:1177:44`
- PersistentMap `wat/rete/compile.wat:1184:30`
- PersistentMap `wat/rete/oracle/accum-pass.wat:194:7`
- PersistentMap `wat/rete/oracle/explain.wat:48:5`
- PersistentMap `wat/rete/oracle/explain.wat:75:33`
- PersistentMap `wat/rete/oracle/fire.wat:122:5`
- PersistentVector `wat/rete/oracle/fire.wat:210:5`
- PersistentVector `wat/rete/oracle/fire.wat:252:5`
- PersistentVector `wat/rete/oracle/fire.wat:481:33`
- PersistentMap `wat/rete/oracle/fire.wat:489:47`
- PersistentMap `wat/rete/oracle/fire.wat:490:46`
- PersistentMap `wat/rete/oracle/fire.wat:494:47`
- PersistentMap `wat/rete/oracle/fire.wat:505:21`
- PersistentMap `wat/rete/oracle/fire.wat:506:20`
- PersistentMap `wat/rete/oracle/pass.wat:652:21`
- PersistentMap `wat/rete/oracle/pass.wat:781:31`

### The 141 `not-checked` sites (full list)

Two mechanisms, cleanly separated by whether the FILE ever produced any type record at all
(cross-referenced against the 1941 files that froze successfully as their own `--check` entry,
unioned with every file's appearance anywhere across all 1941 runs — a stdlib file like
`wat/core.wat` freezes fine as a DEPENDENCY of nearly everything, so "this file never froze"
and "this file froze but never recorded here" are genuinely different facts, checked separately
against `records2.tsv`, not assumed):

**20 files (26 sites) genuinely never freeze, anywhere, as themselves or as a dependency** — 11 are
`.wat.bad` (intentionally invalid; correctly uncountable — a census cannot see inside a file it
cannot parse/freeze), the other 9 are `tests/**` fixtures that do not freeze standalone via
`wat --check <file>` (a pre-existing property of those files, not something this stone touched —
confirmed distinct from the 604-file freeze-failure set already measured for the whole corpus in
§ Method):

  - Tuple `tests/collection/probe_arc216_stone7_tuple_roundtrip_p7.wat.bad:6:8`
  - PersistentVector `tests/collection/probe_arc278_0d_transform_dispatch_parity.wat.bad:10:5`
  - Tuple `tests/collection/wat_dispatch_e2_tuple.wat:14:5`
  - Tuple `tests/collection/wat_dispatch_e2_tuple.wat:7:33`
  - Vector `tests/function/probe_arc237_7b_conj_wrong_elem.wat.bad:6:41`
  - Vector `tests/function/probe_arc237_7b_contains_wrong_elem.wat.bad:6:26`
  - Tuple `tests/function/recursive_patterns_nonexhaustive.wat:6:50`
  - List `tests/lint/probe_c19_nested_type_var_render.wat.bad:41:25`
  - Tuple `tests/lint/probe_c19_nested_type_var_render.wat.bad:41:3`
  - Tuple `tests/program/probe_arc170_edn_bridge_unspellable__lexemes.wat:16:84`
  - Vector `tests/resolve/probe_arc255_the_type_position_has_its_own_authority__bogus_in_return.wat:2:72`
  - PersistentVector `tests/rete/probe_arc278_enum_variant_typo_bad.wat:34:86`
  - PersistentVector `tests/rete/probe_arc278_enum_variant_typo_tagged.wat:33:85`
  - PersistentVector `tests/rete/probe_arc278_inline_constraint_cross_type.wat:24:64`
  - PersistentVector `tests/rete/probe_arc278_inline_constraint_untyped_equality.wat:25:64`
  - PersistentVector `tests/rete/probe_arc278_inline_constraint_untyped_ordering.wat:25:64`
  - PersistentVector `tests/rete/probe_arc278_query_type_safe_typo.wat.bad:16:109`
  - PersistentVector `tests/rete/probe_arc278_query_type_safe_typo.wat.bad:16:67`
  - PersistentVector `tests/rete/probe_arc278_query_type_safe_typo.wat.bad:17:33`
  - Tuple `tests/services/probe_arc170_wrong_service_colocation.wat.bad:46:11`
  - Tuple `tests/services/probe_arc170_wrong_service_colocation.wat.bad:47:11`
  - Vector `tests/services/probe_arc278_struct_new_nature_wall.wat.bad:10:7`
  - Tuple `tests/types/probe_arc255_53_tuple_member_nested.wat.bad:8:13`
  - Tuple `tests/types/probe_arc255_53_tuple_member_nested.wat.bad:8:46`
  - Tuple `tests/types/probe_arc255_53_tuple_member_out.wat.bad:8:13`
  - Vector `tests/types/probe_stone_118_b1a_neg.wat.bad:22:51`

**32 files (115 sites) freeze successfully — sometimes as their own entry, always as a dependency
of something else in the corpus — but the specific untyped-constructor node is never independently
walked by `infer`.** Every sampled case (a majority hand-traced, several confirmed exactly) is one
of two mechanisms, both structurally "this text is DATA, not code the checker walks":

  1. **Inside `(:wat::core::quote …)`** or a **syntax-quote macro template** (`` `(...) `` inside a
     `defmacro` body) — e.g. `wat/core.wat:1283:32`, `:1852:30`, `:1853:32` (all 34 of
     `core.wat`'s not-checked sites are inside backtick-quoted macro templates for the
     `defservice`-family macros). A macro template is data describing the form to *emit*; it is
     never itself walked by `infer`. `wat-scripts/scratch-pad/255-p6c-w6-arity-errors.wat`'s 4
     sites are all inside `(:wat::eval-ast! (:wat::core::quote (:wat::core::Vector 1 2) …))` —
     literally testing what happens when a MALFORMED call is evaluated as quoted-then-interpreted
     data, so the checker correctly never sees it as code;
     `wat-scripts/scratch-pad/probe-slice-one-registry-seam.wat:159:70` sits inside
     `(:wat::rete::pure? (:wat::core::quote (:wat::core::Bytes/to-hex (:wat::core::Vector 255 0
     16))))`.
  2. **Inside `(:wat::core::forms …)`** — a nested/embedded child-process program literal (the
     `nested_program_starts` family the floor's own test names refer to). Confirmed directly:
     `tests/kernel/probe_arc255_29_thread_address_to_process.wat:36:42`'s `(:wat::core::Tuple
     (:user::outcome-name …) addr)` sits inside a `(:wat::core::forms (:wat::core::defn
     :user::main …))` literal at lines 20-43 that is spawned as a **separate child program**, not
     type-checked as part of the parent's own body — confirmed by the type record for this file
     recording every node from line 5 through line 20, then nothing again until line 44, exactly
     bracketing the `forms` literal.

  - Tuple `tests/kernel/probe_arc255_29_thread_address_to_process.wat:36:42`
  - Tuple `tests/macros/probe_arc279b_subs_tuple_macro_eval.wat:30:20`
  - Tuple `tests/macros/probe_arc279b_subs_tuple_macro_eval.wat:31:20`
  - Tuple `tests/macros/probe_arc279b_subs_tuple_macro_eval.wat:32:14`
  - Tuple `wat-scripts/probes/arc-170/probe-compound-upcast.wat:40:31`
  - Tuple `wat-scripts/probes/arc-170/probe-generic-shipped.wat:32:24`
  - Tuple `wat-scripts/probes/arc-170/probe-m1-dial-runner.wat:56:103`
  - Tuple `wat-scripts/probes/arc-170/probe-m1-dial-runner.wat:67:103`
  - Tuple `wat-scripts/probes/arc-170/probe-m1-phantom-d.wat:38:79`
  - Tuple `wat-scripts/probes/arc-170/probe-nested-vector-of-tuples.wat:26:28`
  - Tuple `wat-scripts/probes/arc-170/probe-s3-process-runner.wat:53:24`
  - Tuple `wat-scripts/probes/arc-170/probe-s3b-astsplice.wat:75:28`
  - Tuple `wat-scripts/probes/arc-170/probe-s3c-rendezvous.wat:65:24`
  - Tuple `wat-scripts/scratch-pad/255-home-10-math-stat-seq-examples.wat:52:101`
  - Tuple `wat-scripts/scratch-pad/255-home-10-math-stat-seq-examples.wat:52:125`
  - Tuple `wat-scripts/scratch-pad/255-home-10-math-stat-seq-examples.wat:52:149`
  - List `wat-scripts/scratch-pad/255-home-10-math-stat-seq-examples.wat:62:47`
  - List `wat-scripts/scratch-pad/255-home-10-math-stat-seq-examples.wat:62:72`
  - Tuple `wat-scripts/scratch-pad/255-home-10-math-stat-seq-examples.wat:63:101`
  - Tuple `wat-scripts/scratch-pad/255-home-10-math-stat-seq-examples.wat:63:125`
  - Tuple `wat-scripts/scratch-pad/255-home-10-math-stat-seq-examples.wat:63:149`
  - List `wat-scripts/scratch-pad/255-home-10-math-stat-seq-examples.wat:65:50`
  - List `wat-scripts/scratch-pad/255-home-10-math-stat-seq-examples.wat:68:53`
  - Vector `wat-scripts/scratch-pad/255-p6c-w6-arity-errors.wat:12:80`
  - Vector `wat-scripts/scratch-pad/255-p6c-w6-arity-errors.wat:18:77`
  - Vector `wat-scripts/scratch-pad/255-p6c-w6-arity-errors.wat:21:78`
  - Vector `wat-scripts/scratch-pad/255-p6c-w6-arity-errors.wat:27:81`
  - PersistentVector `wat-scripts/scratch-pad/255-stone-o-apply-lies-about-what-exists.wat:50:53`
  - PersistentVector `wat-scripts/scratch-pad/255-stone-o-apply-lies-about-what-exists.wat:51:139`
  - PersistentVector `wat-scripts/scratch-pad/255-stone-o-i-vector-concat-value-door-panic.wat:29:38`
  - PersistentVector `wat-scripts/scratch-pad/255-stone-o-i-vector-concat-value-door-panic.wat:35:38`
  - PersistentVector `wat-scripts/scratch-pad/255-stone-o-i-vector-concat-value-door-panic.wat:35:75`
  - PersistentVector `wat-scripts/scratch-pad/255-stone-o-i-vector-concat-value-door-panic.wat:45:20`
  - PersistentVector `wat-scripts/scratch-pad/255-stone-o-iii-six-verbs-direct.wat:34:156`
  - PersistentVector `wat-scripts/scratch-pad/255-stone-o-iii-six-verbs-direct.wat:34:189`
  - PersistentVector `wat-scripts/scratch-pad/255-stone-o-iii-six-verbs-direct.wat:35:156`
  - PersistentVector `wat-scripts/scratch-pad/255-stone-o-iii-six-verbs-direct.wat:36:154`
  - PersistentVector `wat-scripts/scratch-pad/255-stone-o-iii-six-verbs-direct.wat:37:153`
  - PersistentVector `wat-scripts/scratch-pad/255-stone-o-iii-six-verbs-direct.wat:38:162`
  - PersistentVector `wat-scripts/scratch-pad/255-stone-o-iii-six-verbs-direct.wat:39:156`
  - PersistentVector `wat-scripts/scratch-pad/255-stone-o-iii-six-verbs-direct.wat:39:189`
  - List `wat-scripts/scratch-pad/255-stone-o-iv-b-collections-sweep-apply.wat:128:16`
  - List `wat-scripts/scratch-pad/255-stone-o-iv-b-collections-sweep-apply.wat:132:16`
  - List `wat-scripts/scratch-pad/255-stone-o-iv-b-collections-sweep-apply.wat:135:14`
  - List `wat-scripts/scratch-pad/255-stone-o-iv-b-collections-sweep-apply.wat:139:14`
  - List `wat-scripts/scratch-pad/255-stone-o-iv-b-collections-sweep-apply.wat:143:14`
  - PersistentMap `wat-scripts/scratch-pad/255-stone-o-iv-b-collections-sweep-apply.wat:28:34`
  - PersistentMap `wat-scripts/scratch-pad/255-stone-o-iv-b-collections-sweep-apply.wat:32:16`
  - PersistentMap `wat-scripts/scratch-pad/255-stone-o-iv-b-collections-sweep-apply.wat:35:32`
  - PersistentMap `wat-scripts/scratch-pad/255-stone-o-iv-b-collections-sweep-apply.wat:39:32`
  - PersistentMap `wat-scripts/scratch-pad/255-stone-o-iv-b-collections-sweep-apply.wat:43:14`
  - PersistentMap `wat-scripts/scratch-pad/255-stone-o-iv-b-collections-sweep-apply.wat:47:32`
  - PersistentMap `wat-scripts/scratch-pad/255-stone-o-iv-b-collections-sweep-apply.wat:52:34`
  - PersistentMap `wat-scripts/scratch-pad/255-stone-o-iv-b-collections-sweep-apply.wat:56:34`
  - Tuple `wat-scripts/scratch-pad/arc109-tuple-bracket-reader.wat:56:27`
  - Tuple `wat-scripts/scratch-pad/arc109-tuple-bracket-reader.wat:57:33`
  - PersistentVector `wat-scripts/scratch-pad/probe-arc278-rules-cross-the-wire.wat:95:15`
  - PersistentVector `wat-scripts/scratch-pad/probe-arc278-rules-ship-as-declared-payload.wat:94:15`
  - Vector `wat-scripts/scratch-pad/probe-slice-one-registry-seam.wat:159:70`
  - List `wat-tests/core/core-nth-differential.wat:102:38`
  - PersistentVector `wat-tests/core/core-nth-differential.wat:82:38`
  - PersistentVector `wat-tests/core/core-nth.wat:79:33`
  - List `wat-tests/core/core-nth.wat:99:33`
  - PersistentVector `wat-tests/gen.wat:837:3`
  - Tuple `wat/bracket.wat:235:12`
  - Tuple `wat/bracket.wat:491:26`
  - Tuple `wat/bracket.wat:616:56`
  - Tuple `wat/bracket.wat:747:98`
  - Tuple `wat/core.wat:1283:32`
  - Tuple `wat/core.wat:1852:30`
  - Tuple `wat/core.wat:1853:32`
  - Tuple `wat/core.wat:1854:32`
  - Tuple `wat/core.wat:1869:50`
  - Tuple `wat/core.wat:1870:34`
  - Tuple `wat/core.wat:1871:36`
  - Tuple `wat/core.wat:1872:36`
  - Tuple `wat/core.wat:1881:32`
  - Tuple `wat/core.wat:1882:34`
  - Tuple `wat/core.wat:1883:34`
  - Tuple `wat/core.wat:1891:32`
  - Tuple `wat/core.wat:1892:34`
  - Tuple `wat/core.wat:1893:34`
  - Tuple `wat/core.wat:1897:34`
  - Tuple `wat/core.wat:1898:36`
  - Tuple `wat/core.wat:1899:36`
  - Tuple `wat/core.wat:1900:34`
  - Tuple `wat/core.wat:1901:36`
  - Tuple `wat/core.wat:1902:36`
  - Tuple `wat/core.wat:1908:28`
  - Tuple `wat/core.wat:1909:30`
  - Tuple `wat/core.wat:1910:30`
  - Tuple `wat/core.wat:1911:55`
  - Tuple `wat/core.wat:1919:30`
  - Tuple `wat/core.wat:1920:32`
  - Tuple `wat/core.wat:1921:32`
  - Tuple `wat/core.wat:1922:20`
  - Tuple `wat/core.wat:1923:22`
  - Tuple `wat/core.wat:1924:22`
  - Tuple `wat/core.wat:1961:47`
  - Tuple `wat/core.wat:1981:27`
  - Tuple `wat/core.wat:2017:29`
  - Tuple `wat/core.wat:2020:21`
  - PersistentVector `wat/gen.wat:707:29`
  - PersistentVector `wat/query.wat:305:33`
  - PersistentMap `wat/query.wat:313:59`
  - PersistentVector `wat/query.wat:324:11`
  - PersistentVector `wat/query.wat:416:34`
  - PersistentVector `wat/query.wat:417:34`
  - PersistentVector `wat/query.wat:493:29`
  - PersistentVector `wat/rete/oracle/accum-pass.wat:135:34`
  - PersistentVector `wat/rete/oracle/insert.wat:93:29`
  - Tuple `wat/rete/oracle/pass.wat:281:36`
  - PersistentMap `wat/rete/syntax.wat:89:37`
  - Tuple `wat/service.wat:1633:46`
  - Tuple `wat/service.wat:1888:74`

**This is a THIRD, distinct instrument caveat beyond F-196**, worth carrying forward to the codemod
stone: a node inside `quote`, a macro's syntax-quote template, or a `:wat::core::forms`
nested-program literal will **never** appear in the type record, under any entry point, because it
is genuinely never walked by `infer` — not a bug in this port, a structural fact about what
"type-checked" means for data describing code. The codemod cannot rely on type_record for these;
it must fall back to the "read-only" prompt (a person decides), same as `unresolved`.

## The two scalar constructors — `u8` / `char`, registration and dispatch today (unchanged; measured only)

**Registered as ordinary Rust intrinsics, not through the container-constructor machinery:**
- `#[wat_intrinsic(":wat::core::u8")]` — `src/numeric/convert.rs::eval_u8_cast` (i64 → u8, range-checked
  0..=255, `MalformedForm` on overflow). EDN-tagged doc row present (`@ExpandTime Legal`, `@Purity Pure`).
- `#[wat_intrinsic(":wat::core::char")]` — `src/intrinsic/char.rs::eval_char_of` (String → char, BMP-only,
  refuses non-length-1 or supplementary-plane input).

**Checker registration:** `src/check.rs::register_builtins` registers both as plain `TypeScheme`s —
`u8: [i64] -> u8` (~line 18458), `char: [String] -> char` (~line 19860) — resolved through the SAME
generic callee-lookup path as any ordinary function (`:wat::i64::+`, etc.). **Not** routed through the
five-head "container constructor" redispatch arm in `infer_list` (`check.rs:2712-2719`,
`":wat::core::PersistentVector" | ":wat::core::Vector" | ":wat::core::List" | ":wat::core::PersistentMap"
| ":wat::core::Tuple"`) — u8/char are absent from that arm.

**Dispatch today:** ordinary intrinsic-registry call dispatch (`runtime.rs`'s generic descend-then-fire
path), identical in kind to any other builtin function call — not a special constructor form.

**Measured, empirically, right now (not changed):** `(wat.type/u8 65)` and `(wat.type/char "a")`
**type-check successfully** (`wat --check`, rc 0) but **fail at runtime**:
```
$ wat --check probe.wat   # (:wat::kernel::println (wat.type/u8 65))
rc=0
$ wat probe.wat
#wat.runtime/UnknownFunction {:message "unknown function: :wat::type::u8" ...}
rc=1
```
identically for `char` (`unknown function: :wat::type::char`). By contrast, the five collection heads
resolve `wat.type/X` end-to-end ALREADY — `(wat.type/PersistentVector 1 2 3)` both checks AND runs
today, printing a length-3 vector. The asymmetry: `src/resolve/normalize.rs::resolve_namespaced_symbol`'s
"Arc 255 Stone ⑤-E" branch normalizes a `wat.type/X` call head to the literal keyword `:wat::type::X`
(not denoted to `:wat::core::X`) whenever `TypeEnv::is_known_type` says X is a known type — plausibly true
for all seven names here (five collections + u8 + char; `wat.type/u8`/`wat.type/char` are ordinary,
working TYPE annotations throughout the corpus, e.g. every `<- wat.type/i64`/`wat.type/u8` parameter
signature, so both must already answer `is_known_type` true — not independently unit-tested against
`TypeEnv` directly within this stone's budget). The checker's
own type-level machinery (`edn::render::type_denotation`, used pervasively through `types.rs`'s
subsumption/comparison code) evidently treats a call whose head denotes to *any* known type as
"constructs that type" generically enough to pass `--check` for all seven uniformly. The RUNTIME's
function-name lookup has no such generic fallback; the five collection heads must reach their existing
intrinsics through some alias/redispatch this stone did not fully trace to its exact call site (candidate:
whatever currently makes `:wat::type::PersistentVector`-style heads resolve at runtime — not located
within this stone's budget); u8/char have no such alias registered.

**What making them genuine `wat.type/u8` / `wat.type/char` constructors would touch** (not done — brief
says do not change them):
1. Whichever runtime-side alias/redispatch already lets the five collection heads' `wat.type/` spelling
   reach their existing dispatch — extended to also map `:wat::type::u8` → `:wat::core::u8` and
   `:wat::type::char` → `:wat::core::char` before intrinsic lookup (the single highest-leverage touch
   point, once located precisely).
2. `src/check.rs::register_builtins`'s two scheme registrations, if the canonical registration key moves
   to the `wat.type/` spelling rather than relying on the denotation door.
3. The two intrinsics' own registered head strings / EDN doc rows, if the naming convention shifts.
4. Nothing about `eval_u8_cast`'s or `eval_char_of`'s actual cast/extraction logic — only the name(s)
   under which they are reachable.

## Files

- `src/check.rs`, `src/check/type_record.rs`, `src/distribution/mod.rs`, `src/distribution/check_output.rs`
  — part 1, committed `faff7e384`.
- `tests/lint/keyword_heresy_ledger.rs` — Arm B fix, committed `faff7e384`.
- `tests/cli/type_record.rs`, `tests/cli/wat_cli__check_types.wat`,
  `tests/cli/wat_cli__check_types__recorded_type.edn` — new tests + fixtures, committed `faff7e384`.
- This SCORE — part 2, committed separately, no `src/` change.
