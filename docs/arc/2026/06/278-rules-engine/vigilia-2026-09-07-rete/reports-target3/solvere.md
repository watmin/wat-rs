## SOLVERE — Cast Report (TARGET 3 — the grid's 20 shell scripts)

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

## SCOPE

Read in full, this session: **all 20** scripts — the 11 `gen-*.sh` generators plus `run-axis.sh`, `run-all.sh`, `check-grid-three-way.sh`, `check-grid-speed.sh`, `compare-grids.sh`, `check-where-shapes.sh`, `check-query-compat.sh`, `check-spec-native.sh`, `peragrare-census.sh`. **No file was only sampled.**

Greps run: `grep -ho 'source [^ ]*\|\. [a-z-]*\.sh' gen-*.sh` (**empty** — confirms no shared generator helper); harness-shape greps across `gen-*.sh`; `#grid/Result` extraction greps across `check-*.sh`/`compare-grids.sh`/`run-all.sh`; `find_java`/`JAVA_HOME` greps; `WAT_BIN=`/`GRID_DIR=`/`REPO_ROOT=` greps; `grep -rn "rune:solvere" *.sh` (**none**).

## Finding 1 — 11 generators share a byte-identical Clara harness, in two variants

**Concerns braided:** (a) *per-axis workload emission* (genuinely different per axis) and (b) *the Clara measurement-and-wire-protocol harness* (session build, JIT warmup, timing, `#grid/Result` EDN printing) — identical across axes, re-typed eleven times with zero shared source.

**Variant 1 (simple timing — 8 files, identical modulo axis name):** `gen-negation.sh:33-38`, `gen-neg-consumer.sh:46-51`, `gen-min-finding.sh:40-45`, `gen-asym-join.sh:41-46`, `gen-user-reduce.sh:50-55`, `gen-node-share.sh:38-43`, `gen-strat-neg.sh:53-58`, `gen-leading-exists.sh:51-56` — each carries the identical triple: `build (fn [] (apply insert (mk-session '<axis> :cache false) seeds))`, `(dotimes [_ 3] …)   ; JIT + compile warmup`, then a `let` timing block ending in the `#grid/Result` print.

**Variant 2 (extended timing — 3 files):** `gen-fanout.sh:34-48`, `gen-deep-cascade.sh:41-55`, `gen-accum.sh:75-89` — same warmup convention, plus a second byte-identical block adding `:insert-ns`/`:fire-ns`/`:query-ns`/`:protocol-ns`.

**Where each should live:** the per-axis workload stays per file (genuinely eleven different programs — not a finding). The harness belongs in one sourced `gen-lib.sh`.
**Judgment: incidental.** Nothing forces eleven copies — copy-paste-and-rename, not a language limit. It already carries drift risk: the wire format has grown fields over this arc, and `check-grid-three-way.sh`'s own header notes a generator emitting a subtly different shape would not fail loudly — it would make one axis silently compare a different question.

## Finding 2 — `#grid/Result` field extraction duplicated, **and the two copies have already diverged**

Sites: `run-axis.sh:277-280` — four inline `grep -oP` calls, each hand-writing the `(?:#wat\.core/PersistentVector\s+)?\K\[[^]]*\]` tag-stripping pattern. `check-grid-three-way.sh:233-241` — a parameterized `extract()` reimplementing the same decode, **plus a `(?<=[ {])` lookbehind guard `run-axis.sh`'s copy lacks**, added per the comment at `:229-232` to refuse an ambiguous line.
**Judgment: incidental, but live** — the two copies have already diverged in robustness, which is exactly the "drift is silent" mechanism; it has not produced a wrong answer only because current field names happen not to collide.

## Finding 3 — `#grid/Verdict` field extraction duplicated, in two different techniques

Sites: `compare-grids.sh:39-60` — awk `field()`/`sizeof()`/`axisof()` via `match`/`substr`. `check-grid-speed.sh:57-60` — four `sed -E` one-liners re-extracting the same fields from the same line shape.
**Judgment: incidental** — no field is unique to either consumer; the split into two techniques is an artifact of who wrote which script.

## Finding 4 — JDK/`java` discovery logic triplicated

Sites, all encoding "PATH → `JAVA_HOME` → `$HOME/opt/jdk-*`": `check-where-shapes.sh:68-84` (inline), `check-query-compat.sh:25-44` (`find_java()`), `check-grid-three-way.sh:98-112` (`find_java()`). The two function versions differ only in trivial formatting — **the signature of independent retyping**. The convention is documented twice in near-identical header lines (`check-query-compat.sh:12`, `check-grid-three-way.sh:57`).
**Judgment: incidental** — pure infra boilerplate, zero domain content.

## Finding 5 — `rewrite_to_spec()` duplicated verbatim, **including its comment**

Sites: `check-spec-native.sh:28-35` and `check-query-compat.sh:50-55` — both define `rewrite_to_spec()` running the identical `perl -pe 's/:wat::rete::fire-rules(?!\$oracle)(?!-)/…/g'`, with the identical two-line comment copied word for word. Only local-var naming differs.
**Judgment: incidental but higher-risk than Finding 4** — this encodes actual naming-convention *domain* logic (the oracle verb), and `run-axis.sh:179-180` documents that this exact class of rename **has already silently broken a sibling regex once**: *"The 2026-08-20 skip was a no-op because it still matched `fire-rules-spec` after the `$oracle` rename."*

## Finding 6 — `WAT_BIN`/`GRID_DIR`/`REPO_ROOT` discovery-and-validation repeated 5×

Sites, byte-identical apart from the echoed script name: `check-spec-native.sh:15-23`, `check-query-compat.sh:15-23`, `check-grid-three-way.sh:60-68`, `check-where-shapes.sh:55-66`, and (wrapped in heavier guards unique to it) `run-axis.sh:45-62`. The rationale comment is itself duplicated: full-length at `run-axis.sh:48-55`, short paraphrase at `check-where-shapes.sh:59-61`.
**Judgment: incidental** — of the 5, only `run-axis.sh` has a freshness wall; the other 4 read the identical binary for the identical kind of measurement with no such protection, **which is itself evidence the copies are not being kept in sync as the convention evolves**.

No `rune:solvere` markers exist anywhere in the 20 scripts — none of the six is pre-acknowledged.

**FINDINGS — 6 rows**, all `duplicated encoding`, all judged **incidental** (none load-bearing, none irreducible, none historical-shape). Findings 1, 4, 5 and 6 each name **every** copy. No edits; no grid script executed.
