# CERNERE — Cast Report (TARGET 3 — the grid) — **CLEAN**

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

## What I swept

**Inventory re-derivation** (all commands run from `/home/john/work/holon/wat-rs`):
- `find wat-scripts/perf/grid -type f | wc -l` → **148**. Matches the handed figure.
- `wc -l` over all 148 → **16616**. Matches.
- Extension breakdown (`find ... | sed 's/.*\.//' | sort | uniq -c`) → **54 wat, 43 clj, 29 txt, 20 sh, 2 md**. Matches exactly — no delta on the inventory line, unlike most prior casts here.
- `grep -ho '[a-z][a-z0-9.-]*/[a-zA-Z!?*<>=-]*' *.clj | sort -u | wc -l` → **70**, matches. `com.cerner/` in `.clj` only → **14 files**, matches. `clojure.string/` in `.clj` only → **6 files**, not 7 (small delta — `where-shapes.clj`/`where-string.clj` require `clojure.string` via `:as str`, which the literal-slash regex doesn't catch as a require line, so the true "clojure.string is in play" file count is actually 8, not 6 or 7; the handed "7" undercounts by a different amount than either of my raw regex passes — the point stands that the crude regex is not a list).

**Spec sources consulted, and their version:**
- `/home/john/work/holon/clara` — `git describe --tags --exact-match HEAD` → **`clara-rules-0.24.0`**, `git rev-list -n1 clara-rules-0.24.0` == HEAD (`8d0ce0bc59db09644e7cc07bc5e34c8b360e015b`), `git status --porcelain` clean. Confirmed **exact match** to the pin at `check-grid-three-way.sh:115` — no fallback to the `.m2` jar needed, though I cross-checked `~/.m2/repository/com/cerner/clara-rules/0.24.0/` exists too.
- `wat-rs/src/resolve/{walk.rs,quote.rs,reserved.rs,boundary.rs}`, `src/freeze.rs`, `src/check.rs`, `tests/lint/wat_scripts_fixes_load.rs`, `tests/lint/rete_names_in_wat_scripts_resolve.rs` — read as the wat-side spec/gate-mechanism sources.
- `wat-scripts/perf/grid/` itself, read against those two specs.

**Surface 1 (43 `.clj` + 11 `gen-*.sh` heredocs) method:** comment-stripped (`;.*$` and, for the `.sh` heredocs, both `#.*$` and `;;.*$`) extraction of every distinct call-head across the whole corpus (`grep -ohE '\([a-zA-Z][a-zA-Z0-9!?*<>=./+-]*'`), then traced every non-trivial head against `clara/src/main/clojure/clara/{rules.cljc,rules/{compiler.clj,dsl.clj,accumulators.cljc}}` and Java/Clojure core. Full line-by-line reads (not just head extraction) of `where-record.clj`, `userfn-head.clj`, `parametric-erasure.clj`, `where-fact-bind.clj`, `where-string.clj` as representative samples across the corpus's distinct patterns (record/enum dispatch, eval-based dynamic rule codegen, parametric erasure, fact-binding, string verbs).

**Surface 2 (54 `.wat`) method:** read both lint gates in full; traced `startup_from_source`'s pipeline (`src/freeze.rs:924`) and `resolve_references`/`check_form`/`is_resolvable_call_head` (`src/resolve/walk.rs`) and `check_quasiquote_template` (`src/resolve/quote.rs`) to determine the *actual mechanism* of the documented hole, rather than trusting the doctrine's prose; then grepped the 54-file corpus for the vulnerable shapes and manually cross-checked every quasiquote-embedded record/field reference against its file's own `defrecord`.

## Did not find (checked against the spec, exists)

- `acc/count`, `acc/max`, `acc/min`, `acc/sum`, `acc/accum`, `acc/exists` — all real, `accumulators.cljc:118,127,152,163,8,172`; every call site's arity (`acc/count` 0-arg, `acc/max`/`acc/min`/`acc/sum` 1-arg field, `acc/accum` 1-arg option-map with exactly `{:initial-value :reduce-fn :combine-fn :convert-return-fn}`) matches the real signatures.
- `mk-session`, `insert`, `insert!`, `insert-all`, `retract`, `fire-rules`, `query`, `defrule`, `defquery` — all real (`rules.cljc:11,23,29,47,62,78,311,374,396`); `:cache false` is a documented `mk-session` option (`rules.cljc:322`); every `insert`/`retract`/`insert!` call site's arity matches.
- `:and`/`:or`/`:not`/`:exists`/`:test` condition-type keywords — real (`dsl.clj:16`, `compiler.clj:569,658,691,918`); every file using `[:exists …]` requires `clara.rules.accumulators` (needed because the compiler expands `:exists` to a fully-qualified `clara.rules.accumulators/exists` call, `compiler.clj:691`) — confirmed present in all 4 files that use it (`where-exists.clj`, `where-query-compat.clj`, `gen-leading-exists.sh`, `gen-accum.sh`).
- `clojure.string/join` called fully-qualified without a local `:require` in 5 files (`accum-over-derived.clj`, `userfn-head.clj`, `accum-lead-rule-cascade.clj`, `parametric-erasure.clj`, `retract-multiplicity.clj`) — checked whether this is a phantom-by-omission: it isn't, because `clara.rules.compiler` and `clara.rules.dsl` both `:require [clojure.string :as string]` (`compiler.clj:8`, `dsl.clj:6`), so loading `clara.rules` transitively loads the namespace before these lines run. Real form, fragile idiom, not a phantom.
- `str/starts-with?`, `str/ends-with?`, `str/includes?`, `str/lower-case`, `str/trim`, `clojure.string/blank?` (mentioned only in a comment, never called) — all real `clojure.string` functions, correct arities.
- `Long/parseLong`, `System/nanoTime` — real Java static methods.
- `bit-test`, `the-ns`, `binding`, `cond->`, `juxt`, `mapcat` and the rest of the core-fn set extracted — all real.
- ~50 apparently-undefined local heads (`big?`, `combo?`, `edge?`, `feline?`, `heavy?`, `is-risky?`, `note-positive?`, `pent?`, `rep-pos?`, `install-link-rules!`, `install-step-rules!`, `after-one-retract`, etc.) — every one traced to a real `defn`/`let`-bound `fn` in the same file (an early automated pass mis-flagged these; direct `grep` confirmed each definition site).
- On Surface 2: every quasiquote-embedded record/field reference in the 14 files that build rules dynamically (`leading-exists.wat`, `accum-lead-rule-cascade.wat`, `neg-consumer.wat`, `accum-over-derived.wat`, `strat-neg.wat`, `deep-cascade.wat`, `min-finding.wat`, `negation.wat`, `node-share.wat`, plus the `where-*.wat` `conds`/`ins` helpers in `where-collection.wat`, `where-boolean.wat`, `where-control.wat`, `where-record.wat`, `where-string.wat`) — cross-checked field-for-field, positional-arg-order-for-order, against each file's own `defrecord` declarations. All correct: `:wsb::Req`'s 6 fields, `:mf::Busy`'s `(loc n)` order, `:cascade::Node`/`Tag`'s `(level id)` order, `:nsh::A/B/Out`, `:alrc::Link`, `:aod::Step`, `:strat::Item/S0..S9`, `:nc::Item/Bad/Tag/Ok/Final`, `:neg::Item/Bad/Ok`, `:lx::S1..S6`, `:wc::Item/Hit`, `:wsc::Req/Hit`, `:wr::Req/Hit`, `:wst::Req/Hit` — every one matches.
- No raw `def` (i.e. `(:wat::core::def …)`, as opposed to `defn`/`defrecord`/`defenum`/`defquery`/`defrule`) form exists anywhere in the 54-file corpus (`grep -ohE '[a-zA-Z:_-]*def[a-zA-Z?!-]*' *.wat` → only `defenum/defn/defrecord/defquery/defrule` as code heads; `defmacro` appears once, in prose only, `where-control.wat:31`).

## The gate-coverage answer for Surface 2

The documented hole is **real as a mechanism**, but its own framing in `wat-rs/CLAUDE.md` (a "`def` body nothing forces") is narrower than the actual code. Reading the resolver directly:

- `src/resolve/reserved.rs:14` — `RESERVED_PREFIXES` includes bare `":wat::"`, and `is_reserved_prefix` (`:42`) matches anything *under* it.
- `src/resolve/walk.rs:264-267` — `is_resolvable_call_head` returns `true` for **any** `:wat::*`-prefixed head, unconditionally, before ever checking whether the leaf name is a registered function. The comment there says this explicitly: leaf-level validation of a reserved-prefix name is "the type checker's concern," not the resolver's.
- `src/resolve/quote.rs:18-46` — `check_quasiquote_template` walks a quasiquote body but **skips every symbol that isn't inside an `unquote`/`unquote-splicing` escape** — i.e. every literal record name, field name, and rete predicate written inside a `(:wat::core::quasiquote …)` template is never visited by the resolver at all, regardless of whether the enclosing `defn` is a `def` or a `defn`, and regardless of whether anything ever calls it.
- `:wat::rete::where` bodies (the actual predicate expressions) are compiled by a separate, later RETE-specific pass (`compile-condition`, `wat/rete.wat:547+`, cited in-file at `node-share.wat:68` and `min-finding.wat:68`) — not by the general type checker — which is exactly why `rete_names_in_wat_scripts_resolve.rs` had to be purpose-built as a textual scanner for that one namespace.

Net effect for **this corpus specifically**: the vulnerable shape isn't "`def` vs `defn`" (this corpus uses zero raw `def`s, so that literal scenario is absent) — it's **quasiquote-templated record/field names outside the `:wat::rete::` prefix**, which 14 of the 54 files use for dynamic rule construction. Neither gate resolves them: gate 1's resolver explicitly treats quasiquote content as data; gate 2 only scans `:wat::rete::`-prefixed text. I traced every such reference in this corpus by hand against the declaring `defrecord`, and found all of them correctly spelled — so **the hole is real and currently unexploited here**, not a false alarm and not a live defect.

## Findings

None. Every Clara form (43 `.clj` + 11 `gen-*.sh` heredocs) traced to a real, correctly-aritied `clara-rules 0.24.0` / Clojure-core / Java form. Every wat form checked in the 54-file corpus traced to a real declaration in the file itself or the stdlib, including the quasiquote-embedded names the two lint gates cannot see.

Files read/grepped for this cast (all under `/home/john/work/holon/wat-rs/wat-scripts/perf/grid/` unless noted): all 43 `.clj`, all 11 `gen-*.sh`, all 54 `.wat` (via targeted grep/sampling), plus `/home/john/work/holon/wat-rs/{tests/lint/wat_scripts_fixes_load.rs,tests/lint/rete_names_in_wat_scripts_resolve.rs,src/freeze.rs,src/resolve/walk.rs,src/resolve/quote.rs,src/resolve/reserved.rs,src/check.rs}` and `/home/john/work/holon/clara/src/main/clojure/clara/{rules.cljc,rules/compiler.clj,rules/dsl.clj,rules/accumulators.cljc}`.

**CLEAN**
