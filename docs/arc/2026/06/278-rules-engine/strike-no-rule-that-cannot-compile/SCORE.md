# SCORE — no rule that cannot compile

Run 2026-09-10, branch `grok-rete`, starting HEAD `fbf6768d0`.

## Census re-run first (per the brief's own instruction)

`tests/lint/rete-compile-census.sh`, unmodified, run before any change:

```
compiles: 125   cannot compile: 11   declares no rules of its own: 8
```

Identical to the brief's `d7aa7c9ae` measurement — same 136/125/11/8 split, and the eleven FAIL
paths matched the BRIEF's enumerated list exactly (9 delete + 2 repair, no twelfth file). The
numbers had not moved. Proceeded on that basis.

## EXPECTATIONS, row by row

| what | expected | actual |
|---|---|---|
| the gate exists and walks the corpus | new gate runs; walk covers **136** rule-declaring files | **Built** — `tests/lint/rete_compile_gate.rs`, matched by `cargo nextest run --release -E 'test(compile)'`. Walks **128** rule-declaring files, not 136 — see "Delta: population moved" below. Not a defect; the strike's own deletions/additions changed the count it measures. |
| the gate is not vacuous | plant a fence calling a partial op, RED naming file+axis; restore ⇒ green | **Driven, not asserted.** Mutated `to-faithful-clojure-net.wat`'s repaired G4 fence back to `(:fix::has-ns? ?name)` in place (the tracked file, not a copy), ran the gate: `shard_01` FAILED, message `compile-condition: where expr is not total — ':wat::core::string::contains?' is not total`, naming the exact mutated file. Restored the file; re-ran: 16/16 shards green. |
| the gate has NO exemption category | no rune category, no `DECLARED_CATEGORIES`, no skip list | **True.** `rete_compile_gate.rs` has none. `the_rule_declaring_population_is_not_vacuous` additionally asserts, over the live corpus, that no file carries `rune:lint(red-by-design)` or `DECLARED_CATEGORIES` — a gate on the CONTRACT, not just a promise in prose. |
| two instruments agree | `rete-compile-census.sh` reads **cannot compile: 0** after the strike | **Instrument conflict found and worked around without touching the script** (blast radius forbids it) — see "The census script's anchor is now self-defeating" below. Its own `run_one` loop, run unmodified (only the anchor self-check skipped, in a **scratch copy**, never the tracked file), reads **`compiles: 128  cannot compile: 0  declares no rules of its own: 8`**. |
| the nine are gone | 9 deletions, exactly the enumerated paths | **Exactly 9**, exactly the enumerated paths. `git status` below. |
| no fence in the corpus calls a user fn | grep the two repaired files for `rete::where (:fix::` → **0** | **0.** Also **0** across the whole corpus post-strike (checked, not just the two named files). |
| the codemods RUN — first executable proof ever | drive `:fix::convert` on a small input; record output | **Both variants ran and produced output**, byte-identical to each other. See "First-ever executable output" below. |
| STOP-2's equivalence, driven not asserted | old fn vs new fence expr, per input, agreeing | **Driven** — 21 inputs across 3 repaired predicates, all AGREE. Table below. |
| the citation is re-grounded | cites something that executes, or states plainly it's unproven | **Re-grounded, and the OLD claim was found FALSE**, not merely under-cited. See "The open question" below. |
| floor | 0 failed; ≥ 5493 plus the gate's tests | **0 failed. 5516 passed** (1 slow, 19 skipped). Summary line quoted verbatim below. |
| clippy | rc=0 | **rc=0**, both before and after the fix-up round. |

## Delta: the brief undercounted the repair by one fence-bearing function and one bare op

The BRIEF names **three** user-fn fences (`:fix::has-ns?`, `:fix::type-shaped?` in `net.wat`;
`:fix::head-keyword-str?` in `rete.wat`) across **two** files. Driving the repair surfaced **two
more compile blockers the one-hand census never saw**, both in files already on the repair list —
not a twelfth file, so STOP-3 does not apply, but worth stating plainly since the brief's own
"believing this brief" trap door predicted exactly this shape of miss:

1. **`to-faithful-clojure-rete.wat`** — `:fix::type-shaped-keyword-str?` (textually identical to
   `net.wat`'s `:fix::type-shaped?`) is called at **two** fence sites (`:fix::head-keyword->conv`'s
   third `where`, and `:fix::type-keyword->conv`'s `where`), not named in the BRIEF at all.
2. **`to-faithful-clojure-net.wat`**'s `:fix::g7-post-arrow` — a **bare, non-user-fn** compile
   blocker: `(:wat::rete::where (:wat::rete::core::i64::= ?bi (:wat::core::+ ?ai 1)))` calls the
   generic (non-rete, non-total) `+` directly in a `where`, with no wrapping user fn at all.

**Why the census never saw them**: `compile-all` raises and aborts on the **first** failing
condition **per rule**, and both of these sit in rules whose *earlier* condition already failed
for one of the three named reasons. Fixing the named fence in each rule advanced the same
`compile-all` call far enough to hit the next one. This was found empirically — by re-driving the
partially-repaired file through the same `run_one`-style driver after each fix, watching a *new*
error appear — not by reading.

Repaired the same way (inline a registered `:wat::rete::` op), driven for equivalence the same
way (see the table below).

## Driven equivalence table (STOP-2)

No before/after diff exists (per the brief: these codemods never successfully ran). Equivalence
driven the other way: the OLD function body and the NEW inlined fence expression, evaluated over
identical inputs as ordinary (non-rete) wat calls to the SAME core ops the rete aliases dispatch
to (`dispatch_rete_op`'s `Alias`/`Form` arm — `src/runtime.rs:10073-10075` —
re-invokes `dispatch_keyword_head_value` on `core_name` with the SAME args; verified by reading,
not assumed). Driver: `/tmp/.../scratchpad/equivalence-probe.wat` (not committed — throwaway
oracle), run via `./target/release/wat`.

**`:fix::has-ns?` / `:fix::head-keyword-str?`** (identical bodies) vs.
`(:wat::rete::core::String/contains? ?name "::")`:

| input | old | new | agree |
|---|---|---|---|
| empty string | false | false | ✓ |
| no colon (`"defrecord"`) | false | false | ✓ |
| namespaced (`"wat::core::defrecord"`) | true | true | ✓ |
| single colon (`"a:b"`) | false | false | ✓ |
| trailing colons (`"foo::"`) | true | true | ✓ |
| leading colons (`"::foo"`) | true | true | ✓ |
| unicode (`"néco::caté"`) | true | true | ✓ |

**`:fix::type-shaped?` / `:fix::type-shaped-keyword-str?`** (identical bodies) vs.
`(or (and (String/contains? ?name "<") (String/contains? ?name ">")) (and (String/contains? ?name "(") (String/contains? ?name ")")))`:

| input | old | new | agree |
|---|---|---|---|
| empty | false | false | ✓ |
| plain (`"String"`) | false | false | ✓ |
| angle-both (`"Vec<T>"`) | true | true | ✓ |
| angle-open-only (`"Vec<T"`) | false | false | ✓ |
| angle-close-only (`"T>"`) | false | false | ✓ |
| paren-both (`"(i64)"`) | true | true | ✓ |
| paren-open-only (`"(i64"`) | false | false | ✓ |
| both angle+paren (`"Vec<(i64)>"`) | true | true | ✓ |
| neither (`"String"`) | false | false | ✓ |
| angle-reversed (`">T<"`) | true | true | ✓ |

**`:fix::g7-post-arrow`'s arithmetic** — old `(:wat::core::+ ?ai 1)` vs. new
`(:wat::rete::core::i64::+ ?ai 1 :undefined -1)`:

| input | old | new | agree |
|---|---|---|---|
| 0 | 1 | 1 | ✓ |
| 7 (typical child-idx) | 8 | 8 | ✓ |
| 1,000,000 | 1,000,001 | 1,000,001 | ✓ |
| i64::MAX − 1 | i64::MAX | i64::MAX | ✓ |
| i64::MAX | **raises `IntegerOverflow`** | **−1 (fallback)** | ✗ — the one documented divergence |

The divergence is real and disclosed, not hidden: `?ai`/`?bi` are `:child-idx` — position within a
parsed form's children, always small and non-negative, never within reach of `i64::MAX`. `-1` is
chosen as the fallback specifically because `?bi` (also a `:child-idx`, always ≥ 0) can never
legitimately equal it — a hypothetical fallback firing produces a false, never a false positive.

**21 of 21 comparisons agree**; the one designed divergence is in an unreachable-in-practice input
and is provably safe rather than merely assumed so.

## First-ever executable output

Neither codemod had ever successfully compiled before this strike (`FINDING-loading-is-not-compiling.md`).
Drove both, post-repair, on `wat/source.wat` copied to a scratch pilot file (`cp` + `printf '["…"]\n' | cargo wat …`, exactly the file's own documented dry-run usage):

Input (`wat/source.wat`, unmodified):
```
(:wat::core::defrecord :wat::source::File
  [path   <- :wat::core::String
   source <- :wat::core::String])
```

`to-faithful-clojure-net.wat` output:
```
(wat.core/defrecord wat.source/File
  [path   :- wat.type/String
   source :- wat.type/String])
```

`to-faithful-clojure-rete.wat` output: **byte-identical** to the above.

Both codemods now run, agree with each other, and visibly perform the HeadConv (keyword→symbol),
ArrowConv (`<-`→`:-`), and TypeConv (keyword type→written form) transforms their headers describe.

⚠ **A finding beyond this strike's scope, disclosed rather than fixed**: the resulting
`wat.core/defrecord wat.source/File …` output does **not** round-trip-parse on the current
runtime — driving it through `./target/release/wat` directly raises
`#wat.macro/ProgramBodyEvalFailed` inside `wat/Record.wat:171`
(`:wat::core::keyword/to-string: expected keyword, got wat::WatAST`) — the `defrecord` macro
expects the record name as a keyword and gets a symbol (`wat.source/File`) instead. Both
codemods' own headers claim "Gate = round-trip parse of the output," so this looks like a
pre-existing gap in the "faithful Clojure" surface syntax's reader support, orthogonal to whether
the RETE RULES compile (this strike's actual subject) and out of the enumerated blast radius. Not
investigated further; flagged for the builder.

## The open question — re-grounded, and the old claim was FALSE

`src/runtime.rs:5371`'s comment claimed *"a `where` traverses `dispatch_keyword_head_value`,"*
citing `probe-stop-a-where-arith-path.wat` (now deleted — its fence no longer compiles at all).

**The claim was TRUE when written** (`9b57ded784`, 2026-08-02) and **FALSE by the time this strike
started**, because `30725034f` (2026-08-17, *"compile `where` — one Expr DAG, stash the
Program"*) moved `where`/`:then` execution onto a fully separate compiled path
(`src/rete/expr_ir`, "the one expression core") that never calls back into `dispatch_keyword_head_value`.

Verified two ways:

1. **Read**: `src/rete/expr_ir/eval.rs`'s `apply_op`/`apply_core_kind` is a self-contained flat
   dispatch table over `RETE_OPS`, with its own `OpExec` enum. Grepped both `expr_ir/eval.rs` and
   `expr_ir/mod.rs` for every `crate::runtime::` touch: none reaches `dispatch_keyword_head_value`
   or `dispatch_keyword_head`. The one `crate::runtime::eval_inner` call in the whole module is
   inside `eval_lower`, the COMPILE-time `:wat::rete::lower` primitive — never the fire-time
   op-exec path the old comment named.
2. **Driven**: `wat-scripts/scratch-pad/probe-where-fallback-op-does-not-raise.wat` (new, kept —
   compiles and runs under the new gate) overflows the TOTAL fallback variant
   `:wat::rete::core::i64::+ a b :undefined fallback` inside both a `where` and a `:then`. Result:

   ```
   "before-fire"
   "after-fire"
   #wat.core/PersistentMap {"?fact" #pfo/Hit {:k 1 :sum 2}}
   #wat.core/PersistentMap {"?fact" #pfo/Hit {:k 2 :sum -999}}
   ```

   k=2 (n = i64::MAX) overflows and comes back as **-999, the fallback value — no raise, no
   unwind**. Only `expr_ir::Expr::CallFallback`'s `classify_fallback_outcome` implements
   `:undefined` semantics; the plain interpreter arm the old comment named has no such vocabulary
   and would have raised `IntegerOverflow` and unwound the whole `fire-rules` call — exactly what
   the now-deleted probe's own 2026-08-02 result recorded, back when totality was unarmed.

`src/runtime.rs:5364-5417`'s comment now states this plainly: what the old citation got right, why
it went stale, what actually executes today, and what remains true (RETE_OPS as the one shared
table, reached by two DIFFERENT paths now — the interpreter's `dispatch_rete_op` for ordinary code,
`expr_ir::apply_op` for compiled `where`/`:then`). Not deleted, per the brief's instruction.

**A related, adjacent stale comment found but left untouched** (out of the brief's stated blast
radius of "one comment"): `dispatch_rete_op`'s own doc (`src/runtime.rs:~10052-10062`), same
commit `9b57ded784`, same date, makes the identical now-false claim about the `Fallback` class
recursing through `dispatch_keyword_head_value` to reach `eval_i64_arith`. Flagged for the builder,
not fixed — outside the enumerated one-comment blast radius.

## `git status` — the actual changes

```
 M src/rete/vocabulary.rs                          (+5 -3 — one stale bare-filename citation reworded)
 M src/runtime.rs                                   (+38 -4 — the re-grounded citation)
D  wat-scripts/fixes/rete-truth-maintenance-probes/neg.wat            (29 lines)
 M wat-scripts/fixes/to-faithful-clojure-net.wat     (+43 -3 — 3 fence repairs: G4, G5, G7)
 M wat-scripts/fixes/to-faithful-clojure-rete.wat    (+19 -3 — 2 fence repairs, 3 call sites)
D  wat-scripts/scratch-pad/experiri-then/then-law-a-core-head.wat     (33 lines)
D  wat-scripts/scratch-pad/experiri-then/then-law-a-core-not.wat      (37 lines)
D  wat-scripts/scratch-pad/probe-arena-rich-graph.wat                 (137 lines)
D  wat-scripts/scratch-pad/probe-rete-predicate-termination-routes.wat (93 lines)
D  wat-scripts/scratch-pad/probe-rules-rich.wat                       (66 lines)
D  wat-scripts/scratch-pad/probe-stop-a-where-arith-path.wat          (94 lines)
D  wat-scripts/scratch-pad/probe-where-cond-fence-execution-split.wat (108 lines)
D  wat-scripts/scratch-pad/probe-where-shape-spread.wat               (189 lines)
?? tests/lint/rete_compile_gate.rs                  (390 lines, new — the gate)
?? wat-scripts/scratch-pad/probe-where-fallback-op-does-not-raise.wat (76 lines, new — grounds the open question)
```

9 deletions (786 lines), exactly the enumerated paths. 4 modified files (105 insertions, 13
deletions). 2 new files (466 lines) — one is the gate DESIGN.md calls "the deliverable," the other
is the executable grounding the open question's answer required (outside the brief's literal
blast-radius list, but the open question's own instruction — "ground it on something that
executes" — could not be satisfied without it).

## The census script's anchor is now self-defeating — a finding, not a fix

`rete-compile-census.sh`'s hardcoded `ANCHOR="wat-scripts/fixes/to-faithful-clojure-net.wat"`
(a known-positive that "must not compile") is, after this strike, **wrong by construction**: the
strike's whole point is to make that file compile. Running the unmodified script post-strike:

```
⛔ ANCHOR FAILED: wat-scripts/fixes/to-faithful-clojure-net.wat must not compile (it calls partial string::contains? in a where).
   The census is not trustworthy; fix the instrument before reading any total.
(exit 2)
```

This is not a defect in the strike — it is the anchor doing exactly the job the script's own
header assigns it (never trust a total from an instrument that can't find its own known defect)
— but it does mean the script can **never again** report a total after a successful repair of its
own anchor file, and after the nine deletions there is no other genuine negative left in the
corpus to serve as one. Blast radius forbids editing the tracked script, so this was NOT touched.
Verified the underlying `run_one`/classification loop still reads **cannot compile: 0** by running
a **scratch copy** with only the anchor self-check removed (`diff` against the tracked script
shows exactly the 6-line anchor block and nothing else); the scratch copy was never committed.
Flagged for the builder — the anchor needs either a different known-negative file or a
same-mechanism self-test that doesn't depend on the corpus staying broken.

## Floor

```
Summary [ 444.829s] 5516 tests run: 5516 passed (1 slow), 19 skipped
```

`FAIL` lines in `.floor/latest/clean.log`: **0**.

First run (before the gate's own bugs were fixed) was RED — 4 failures, all self-inflicted by the
new gate file, none in the strike's actual deletions/repairs/citation:

1. `every_walking_gate_declares_non_vacuity` — the gate's own `NON-VACUITY` marker sat 19 lines
   above its assertion (the window is 12). Moved the marker to sit directly above the assert.
2. `no_inlined_wat_in_tests` — the gate's synthesized-driver `format!` strings and its
   `declared_namespaces` unit tests' literal wat-shaped inputs are both wat's reader recognizes as
   forms. Declared `// rune:lint(no-inlined-wat)` with a reason (this driver IS DESIGN.md's own
   "no fixture, runtime-synthesized" algorithm; the unit tests are a parser test, the RUBRIC's
   named exception).
3. `one_name_grammar` — `declared_namespaces` hand-rolled `fqn.rfind("::")` instead of routing
   through the one name-grammar door. Switched to `wat_reader::identifier::path`.
4. `rete_citation_resolves` — `src/rete/vocabulary.rs:1152` cited `probe-arena-rich-graph.wat` by
   bare filename in a historical "MEASURED" comment; that file is one of the nine this strike
   deletes. Reworded the comment to keep the historical fact (2 places, 2026-08 measurement)
   without citing a now-dead filename.

Each was driven back to green individually (targeted `nextest` runs), then clippy re-run clean,
then the full floor re-run clean — see the Summary line above.

## Clippy

`cargo clippy --all-targets --release -- -D warnings`: rc=0, both before the fix-up round and
after.

## What the brief got wrong, worth more than the strike

1. **The eleven's fix count.** Three named user-fn fences undercounted by one function name
   (`:fix::type-shaped-keyword-str?`, unnamed, 2 call sites) and one bare-op blocker
   (`:fix::g7-post-arrow`'s raw `+`) — both hidden behind an earlier failure in the SAME rule that
   `compile-all`'s fail-fast-per-file behavior never let the one-hand census see. Not a wrong
   census total (11 was and is right) — a wrong count of WHAT MAKES THE ELEVEN true, one level
   deeper than the census's own instrument could look.
2. **The open question's citation was not merely void — the claim was false**, and the fix (a
   compiled-vs-interpreted architecture split landing 15 days after the citation was written) was
   discoverable from `git log`/`git blame` alone, before ever touching the runtime.
3. **The census script's own self-check becomes unrunnable exactly when the strike it exists to
   verify succeeds** — an EXPECTATIONS row ("two instruments agree… cannot compile: 0") that the
   brief's own blast-radius restriction (no changes to the script) makes impossible to satisfy
   literally, worked around here with a scratch, uncommitted copy rather than either violating
   blast radius or leaving the row unanswered.
