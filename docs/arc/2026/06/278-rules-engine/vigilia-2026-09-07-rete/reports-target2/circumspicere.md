## CIRCUMSPICERE — Cast Report (TARGET 2 — the fifteenth and last ward)

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

## SCOPE

Swept all 25 target files (23,886 lines). Read-only: `grep`/`Read` only, no edits, no cargo.

Skipped as already-covered (per the fourteen-ward table and the promoted classes): names, craft, dead code, braiding, exemptions, error-type shape, waste, deferral prose, phantom forms, registry substance, deep-type spelling, mutation coverage, phantom error heads, false caller counts, stale prose numbers. Did not re-litigate `reachability.rs`.

Commands run (representative): `grep -rn "rune:circumspicere"` over the 25 files; `grep -rl "rete/<file>" tests/lint/*.rs` per file, **then for every matched and unmatched gate, read its `collect_rs`/`root()` to find actual scan roots**; `grep -n "MAX_IMPORT_NODES\|MAX_IMPORT_DEPTH\|quadratic\|import"` across `export.rs` cross-checked against `wat/rete.wat`, `runtime.rs` dispatch, and callers; traced the mutual recursion `lower_expr → lower_list → lower_expr` in `expr_ir/mod.rs`, `exec` in `expr_ir/eval.rs`, `exec_dim` in `where_tree.rs`; grepped the whole `src/` tree for any `MAX_.*DEPTH`/`recursion_limit`/`stacker` guard (found only `export.rs`'s import wall and an unrelated display-depth cap in `value/observe.rs`); grepped `tests/rete/*` for depth-shaped tests.

**Measurement 1 (rune count) re-derived**: **0**, confirmed. No surface in this target has ever been declared an accepted-by-design edge.

**Measurement 2 (gate-naming) re-derived and resolved — NOT a gap.** The per-file counts match what was handed to me. But checking each gate's actual scan root shows **every one of the 25 files is swept regardless of being *named***: `no_ceiling_raise_in_rete.rs` and `no_mutex_in_rete.rs` recurse the whole `src/rete` tree (`collect_rs` has no file filter beyond `.rs` — `no_ceiling_raise_in_rete.rs:61-70`, `no_mutex_in_rete.rs:8-20`); and eleven more — `one_name_grammar`, `one_param_spec`, `no_loose_string_assert`, `no_angle_suffix_strip`, `no_angle_type_in_diagnostic`, `unused_span_justified`, `span_substitution_justified`, `retired_name_justified`, `universe_control_name`, `no_rc_use`, `no_rpds_rebuild_loop` — recurse `["src", "tests", "crates"]` or `src/` wholesale (e.g. `no_angle_suffix_strip.rs:105-107`). So the files with zero *name* hits are still walked by 13 broad gates apiece. **The measurement's own warning was correct to raise the question; the answer is "covered," not "gap."**

## FINDINGS

### 1. Unenforced invariant + claim-vs-code — unbounded recursion depth in "the one expression core," contradicting its own total-or-refuses promise

**Surround**: unenforced load-bearing invariant (facet 3) and claim vs code (facet 2), together.

**Coordinates** (two, per the ward's claim-vs-code format):
- **The claim** — `src/rete/expr_ir/mod.rs:14-19`: *"⛔ THE INVARIANT THAT MAKES THAT SPLIT WORTH IT: `lower` IS TOTAL OR IT REFUSES. A `Program` that exists is one `exec` can run — every name resolved, every arity checked, every head known. `exec` therefore raises only on VALUES … never on shape. A refusal that belongs at compile time and lands at fire time is a defect in this file, because it moves a diagnostic from the rule the author is writing to **the millionth row of someone's data**."*
- **The code that doesn't honour it** — `lower_expr`/`lower_list`/`lower_hof_callee` (`mod.rs:374,431,473`) mutually recurse over `WatAST` with **no depth counter anywhere** — `LowerCx` (`mod.rs:255-262`) carries `sym`, `slots`, `next`, `hof_fn_pos` and nothing else. `exec` (`eval.rs:201`) recurses over the resulting `Expr` tree the same way, on every row, every fire. `where_tree.rs`'s sibling `exec_dim` (`:560-576`) shares the shape.

**What the inward guard saw here**: nothing — `struere`/`probare`/`experiri` all cast on this file family and none reported it.

**Why this is real and not hypothetical**: this exact codebase already found and fixed the identical defect shape one file over, in the *same* subsystem's sibling recursive descent. `export.rs:350-370` (`MAX_IMPORT_DEPTH`) documents, with a specific driven measurement: *"the same 20,000-deep Export was ACCEPTED on a 256 MiB thread and killed a 2 MiB one with `fatal runtime error: stack overflow, aborting` — an abort, not a panic, so nothing catches it. Acceptance was a property of the importing THREAD."* Deep nesting there required no special construction. The parser that feeds `lower_expr` has **no depth guard either** (`grep -n "depth\|recursion\|MAX_" src/parser.rs` → nothing), so an equally ordinary deeply-nested wat expression in a rule's `:when`/`:where`/`:then` reaches `lower_expr` and `exec` the same unguarded way — and per the header's own words would land *"at the millionth row of someone's data,"* except as an **uncatchable process abort**, which is worse than either of the two outcomes the header claims are exhaustive. No test in `tests/rete/*` exercises single-expression nesting depth — the only "depth" tests found are forward-chain cascade rounds (max 20) and tail-call-position iteration depth, a different axis that does not touch structural AST nesting.

**Severity: L1** — a shipped claim (*"total or refuses," "never on shape"*) the code does not enforce, in the file that names itself the *sole* path every rete expression compiles and runs through.

**Closure**: thread a `depth: u32` through `LowerCx`, incremented at `lower_expr`/`lower_list`/`lower_hof_callee` entry, refusing past a measured threshold — the same method `MAX_IMPORT_DEPTH` already used. Because `lower` is the only door into a `Program` and `exec` can only run what `lower` produced, fixing the cap once in `lower` closes it for `exec` and `exec_dim` by the module's own already-stated construction.

### 2. Negative space — the one wat-facing spec of `Export` never mentions the resource walls `import` enforces

**Surround**: negative space (facet 4), adjacent to claim vs code.

**Coordinates**: `wat/rete.wat:329-352` (the `Export` record's doc block and `defrecord`) vs `src/rete/export.rs:308-370` (`MAX_IMPORT_NODES` / `MAX_IMPORT_DEPTH`, the two walls governing what `:wat::rete::import` — dispatched at `runtime.rs:5712`, callable from any wat program — will accept).

**What the inward guard saw here**: nothing — this is the one in-scope wat file that documents `Export`'s shape field by field, including behavioural notes (*"Import refuses a miss"* for `abi`, *"Import without deps refuses production fire"* for `deps`) — but says nothing about the two size/depth walls that also refuse. A wat author reading the one spec file that documents this record has no way to learn the 10,000-node / 300-deep ceilings exist, or that the build is quadratic beneath them, without reading Rust source comments.

**Note — the target-1 pointer's own shape, checked and found NOT recurring here**: I verified whether the quadratic-build/node-cap qualification "travels" the way the target-1 memory-ceiling qualification failed to. **It does**: the same relationship is stated consistently at three sites inside `export.rs` itself — the module header (`:89-104`), the constants' own doc comments (`:308-370`), and the runtime refusal message (`:2361-2369`) — and the node-count check runs *before* any node is unpacked (`:2361`, ahead of the unpack loop at `:2374`), so the refusal costs nothing. **This is the one case in this target where the discipline held**; the actual gap is the adjacent one above, where the qualification never reached the *spec* file at all.

**Severity: L2** — accurate where it speaks, silent on an operationally material default; not a correctness lie.

**Closure**: add a line to the `Export` doc block in `wat/rete.wat` naming the two walls so a wat-level reader meets the same artifact the Rust code enforces.

No `rune:circumspicere` exists anywhere in target 2 to adjudicate (measurement 1, confirmed zero).

**FINDINGS: 2 (1 L1, 1 L2).**
