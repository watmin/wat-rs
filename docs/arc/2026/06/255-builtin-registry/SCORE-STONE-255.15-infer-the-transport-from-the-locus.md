# SCORE — STONE 255.15: infer the transport from the locus

⚠ Written by the orchestrator from the executor's returned text; the harness refused the executor's own
write ("Subagents should return findings as text, not write report files").

**Commit `7cfa923a0`**, on `7d79d539c` (which touched one FINDING doc and nothing in `src/`). Pre-cure
binary built from `a692f1d7b`. **VERDICT: LANDED.**

## 1. Design, and why this door

- **`src/types.rs`** — new field `TypeEnv::parametric_extensions: HashMap<String, Vec<TypeExpr>>`: the
  target of each parametric `extend-type` edge **as the parsed type, not a string**. One door,
  `register_parametric_extension(child, target, span)`, writes both halves: it calls
  `register_subtype(child, &format_type(target))` and stores the parsed target (skipping a duplicate via
  `type_exprs_same`). The `extend-type` handling in `splice_type_decls` routes parametric protocol forms
  through it; keyword and plain-path protocols unchanged. `retract_for_door_replace` removes the entry
  beside `subtype_edges`. Helper `parametric_extensions_of(sub, surface, env)` uses `is_subtype`'s start
  key (`type_denotation(sub)`) and matches heads via `parametric_heads_unify`.
- **`src/check.rs`**, arc-170 Gap-1 arm of `assignable`: when the exact-string edge fails, take the
  structured targets the actual type declared at that surface, check arity, unify the target's arguments
  with the expected ones **on a cloned substitution**, **commit only if exactly one solution**, then apply
  the nature-floor check as the concrete arm does. Zero solutions → refused as before; **two or more →
  refused as ambiguous, never picks one.**
- **Why this door:** the edge store keeps only the rendered string; an expectation holding a unification
  variable renders `(Loc :- [_])`, which can never equal it. The alternatives were re-parsing the stored
  string on every call (reader on a hot path, depends on round-tripping) or keeping the parsed type the
  declaration already produced (`parse_type_node`'s result, previously discarded after `format_type`).
  Chose the latter; **the string edge is now rendered in the same call from the same parsed type, so the
  two cannot be written apart.** No second string-keyed lookup (STOP-2 not hit); the substitution was
  already `&mut` in `assignable` (STOP-1 not hit).
- **Direct edges only — a deliberate departure from `is_subtype`'s transitive walk.** Measured: with
  `(derive :Th2 :Th)` the concrete arm accepts `Th2` via the chain and the program dies at run time with
  `UnknownFunction … :probe::Th2/transport`, even pre-cure. A first draft that walked the chain for parity
  (`wat-post1`) accepted `derive-var.wat` at rc=0 — it would have spread that hole to every type-variable
  call site. The final version refuses it; a committed fixture pins that.

## 2. Where the brief was wrong (the code wins)

1. **`src/types.rs:2151` is not the edge store** — it is prose for `:wat::eval::FormOutcome`. The edge is
   registered in `splice_type_decls`' `extend-type` handling (~`types.rs:4460-4495` pre-edit), stored by
   `register_subtype` into `subtype_edges` (~`:1303`).
2. **The "Gap 2 precedent" at `check.rs:5449` does not bind from the implementor** — it binds the surface's
   parameters from the *receiver type's own* arguments. For a concrete receiver the implementor's binding
   was already substituted into its `<Type>/<method>` scheme at `extend-type`. An analogue, not a
   precedent. The closest arm is Stone 118.3-B's parametric-vs-parametric (*"unify on the args, soundness
   in the guards"*).
3. `assignable :17297`, the arm near `:17372`, and `unify :16713` were correct.

## 3. Expectations — `wat-pre` (`a692f1d7b`) vs `wat-final` (`7cfa923a0`)

Fixtures `tests/types/probe_arc255_15_infer_transport*`, driven by `probe_arc255_15_infer_transport.rs`
(9 tests, 9/9). Every error assertion is structural on `CheckErrorKind` fields and asserts `errs.len() == 1`.

| row | pre-cure | final |
|---|---|---|
| ⭐ the gap closes (`…_ok.wat`) | rc=1, 5 errors, `expects (:probe::Loc :- [_]); got :probe::Th` | **rc=0** |
| `T` flows to the result (run) | rc=3 | **rc=0, stdout `"1"` `"w"` `"1"`** (Th→Shared, Pr→Wire, a two-param call sharing one `T`) |
| wrong transport | rc=1 — the gap (proves nothing) | **rc=1 `ReturnTypeMismatch`**: body produces `Shared`, signature declares `Wire` |
| conflicting bindings | rc=1, 2 errors (the gap) | **rc=1, 1 error**: param `#2` expects `(Loc :- [:probe::Shared])`; got `Pr` |
| conflicting, reversed (scratch) | rc=1 | rc=1, `#2 expects (Loc :- [:probe::Wire]); got Th` |
| non-implementor | rc=1 | rc=1 `TypeMismatch … got :probe::NotLoc` |
| nested mismatch (added) | rc=1 | rc=1 `TypeMismatch` |
| derive chain (added) | rc=1 | rc=1 `TypeMismatch … got :probe::Th2` (the transitive draft gave rc=0) |
| concrete A/B unchanged | rc=0 / rc=1 | rc=0 / rc=1 — **`cmp` of full stdout+stderr: byte-identical** |
| a pinned `T` still wins (scratch) | rc=1 | rc=1 `#2 expects (Loc :- [:probe::Wire]); got Th` |

## 4. Ambiguity — reachable, refused, and the refusal is load-bearing

One type can extend the same parametric surface twice: with two bodies it is refused at registration
(`DuplicateDefine`), but ⛔ **if the second is bodiless it is accepted** (bodiless `extend-type` is arc 170
C2-D). Given `Both` where `(Loc :- [T])` is expected, `…_ambiguous.wat.bad` → **rc=1** — refused, not
picked. **Mutation:** changing `solutions.len() == 1` to "take the first" and rebuilding (`wat-mut-ambig`)
gave rc=0; source restored. Control: `Both` with only the `Shared` edge → rc=0 after, rc=1 before.
⚠ The diagnostic is the generic `TypeMismatch`, not "ambiguous" (naming it would thread a new error kind
out of a `bool`-returning function).

## 5. ⛔ Two pre-existing holes — not widened by this stone

1. **Double instantiation + a bodiless edge → a runtime type lie through the CONCRETE arm.**
   `ambig-lie-run.wat`: `Both` extends `(Loc :- [Shared])` with a body and `(Loc :- [Wire])` bodiless;
   `sc [loc <- (Loc :- [Wire])] -> Wire` given `Both` → `--check` **rc=0 on both binaries**; run **rc=2**
   *"expected receiver of class :probe::Wire, got class :probe::Shared"*. Dispatch keys on the flat
   `<Type>/<method>`, so one body serves both instantiations. Likely root cure: refuse a second
   instantiation of one surface on one type at registration.
2. **A `derive` chain satisfies a surface statically but has no methods at run time.** `(derive :Th2 :Th)`
   to a `(Loc :- [Shared])` parameter → `--check` rc=0 pre-cure; run rc=1 `UnknownFunction …
   :probe::Th2/transport`. The inference reads direct edges only and does not inherit it.

## 6. Gates

- ⛔ **Floor run 1 RED — 6022/1**, `.floor/2026-09-24T01-46-00Z/`:
  `Summary [ 311.140s] 6023 tests run: 6022 passed (9 slow), 1 failed, 22 skipped`. The failure was the
  executor's own: `wat::lint no_inlined_wat_in_tests::tests_carry_no_inlined_wat`, assertion
  `tests/lint/no_inlined_wat_in_tests.rs:440`, offender `tests/types/probe_arc255_15_infer_transport.rs`
  (two golden `"(:probe::Loc :- [:probe::Shared])"` comparison strings). Fixed with a
  `// rune:lint(no-inlined-wat)` marker and reason — the disposition `probe_arc170_parametric_surface_param.rs`
  uses. Lint 10/10 after. Not re-run to clear.
- **Floor run 2 GREEN — 6023/6023**, `.floor/2026-09-24T01-52-43Z/`. `harvest_wrap_split`: **PASS**
  (0.152s), named as the brief requires. The 4 `keyword_heresy_ledger` tests pass.
- **Clippy** rc=0, **not cached** (touched `src/check.rs`; 6.29s).
- **Census** — PREV from the pre-cure binary, CURR from the final; 2221 files each; `no STOP-8`; the full
  diff is **exactly one line**: `…_infer_transport_ok.wat 1 → 0` (rc-0 count 2007 → 2008). **No other
  tracked `.wat` changed verdict under this permissive cure.** Control: the diff reversed gives
  `STOP-8 … rc 0 -> 1`, exit 8.
- **Delta** — ORIG-CLEAN 160/179, CONV-CLEAN 157/179, **NEW 3, RECOVERY 0**, exit 0 (baseline).
- **Ledger** — `LEDGER_TOTAL = 220`, unmoved.
- **Diff** — `src/check.rs` +40, `src/types.rs` ~+75, one `.rs` test, 10 fixtures. No stdlib `.wat`, no
  service-macro change.

## 7. What this green cannot see

A generic implementor (`extend-type :Box (Loc :- [T])` with `Box :- [T]`) — unprobed; a bare `Box` Path
could bind the caller's variable to the parameter name. Both axes in one call — unmeasured. **The process
locus** — every run was in-process; a forked child is believed to rebuild the store from the kept
`extend-type` forms, but no process locus was driven. Partially-successful multi-argument unify followed by
a later failure — covered by construction (cloned substitution), no fixture forces it. The ambiguity
diagnostic does not name ambiguity. The concrete arm still decides by exact string first, so §5's holes stand.
