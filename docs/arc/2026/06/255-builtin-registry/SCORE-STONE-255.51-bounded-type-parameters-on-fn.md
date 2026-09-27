# SCORE — STONE 255.51: bounded type parameters on `fn`

Struck against draw `44166fd57` (brief drawn at `260c88151`). **STOP-1.** The checker is in the tree. One corpus file wrote a binder entry that was dropped until now. That file was not edited. The floor was not re-run.

## STOP-1

`.floor/2026-09-27T00-41-21Z`, exit 100. Summary, verbatim:

```
Summary [ 355.757s] 6155 tests run: 6154 passed (9 slow), 1 failed, 22 skipped
```

The failing block from `.floor/2026-09-27T00-41-21Z/ARM.txt`, verbatim:

```
        FAIL [ 355.750s] (6155/6155) wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime
  stdout ───

    running 1 test
    test wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime has been running for over 60 seconds
    test wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime ... FAILED

    failures:

    failures:
        wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime

    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 363 filtered out; finished in 355.74s

  stderr ───

    thread 'wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime' (323655) panicked at /home/john/work/holon/wat-rs/tests/lint/wat_scripts_fixes_load.rs:64:5:
    1 of 790 wat-scripts/ files do not load on the current runtime (rotted):
      wat-scripts/fmt/fixtures/generic-fn.wat
          #wat.check/CheckErrors {:message "1 type-check error" :location nil :causes [] :errors [#wat.check/MalformedForm {:message "malformed :wat::core::fn form: binder entry must be a bare name or [Name :< Type]; got :wat::core::i64" :location #wat.core/Span {:file "wat-scripts/fmt/fixtures/generic-fn.wat" :line 5 :col 25 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 5 :col 40}}} :causes [] :head ":wat::core::fn" :reason "binder entry must be a bare name or [Name :< Type]; got :wat::core::i64" :remedies []}]}
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

The arm is the panic at `tests/lint/wat_scripts_fixes_load.rs:64`. Kind `Check` / `MalformedForm`, head `:wat::core::fn`, reason `binder entry must be a bare name or [Name :< Type]; got :wat::core::i64`.

The site, `wat-scripts/fmt/fixtures/generic-fn.wat:5`:

```clojure
(:wat::core::fn :- [:wat::core::i64]
  [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
  ...)
```

The binder's only entry is the keyword `:wat::core::i64`. `peel_type_binder` used to keep bare symbols and drop every other entry (`src/function/metadata.rs` before this stone). The parameters are already annotated `:wat::core::i64`, so the dropped entry named nothing the signature used. It is a declaration that never meant what it said.

A scan of every git-tracked `*.wat` and `*.wat.bad` for a direct `:-` binder on `fn`, `defn`, or `extend-type` found this one checked site. Eight other hits are syntax-quote templates that splice names when the macro runs (`~@binder-names-ch` at `wat/core.wat:1397`, `~@peer-tp-syms` at `wat/core.wat:1282`, `:1332`, `:1336`, `~@fqdn-tp-syms` at `wat/service.wat:2188`, `~@handle-tp-syms` at `wat/service.wat:2937`, `:2955`, `:2967`). The stdlib loaded: the other 6154 floor tests start it. Those templates are not binder entries at load.

No second site. The fixture was left as written.

## What the checker does

One door, `parse_binder_entries` (`src/types.rs:5864`). An entry is a bare name, or `[Name :< Type]`. Anything else is an error that names the entry: `binder entry must be a bare name or [Name :< Type]; got {rendered}`. `peel_type_binder` (`src/function/metadata.rs:49`) returns that error. `extend-type` still refuses a bounded entry, and the refusal names the reason (`src/types.rs:5933`): `extend-type does not accept a bounded type parameter yet (conditional membership is a later stone); entry {shown}`.

The bound is `type_param_bounds: Vec<Option<TypeExpr>>`, parallel to `type_params`, on `Function` and on `TypeScheme`. `derive_scheme_from_function` carries it. `instantiate` and `instantiate_with_args` return an `Instance` whose `bounds` are the letter, the fresh variable, and the bound renamed with the same mapping. `TypeExpr` itself has no bound slot. STOP-2 did not fire.

`SurfaceMember::Method` stores the same vector so a method binder is not dropped. Method dispatch does not consult it. The method row this stone tests is the malformed-entry error, head `:wat::core::defsurface`.

After a call's arguments unify, `enforce_type_bounds` (`src/check.rs:17974`) asks `assignable(got, bound)` on a concrete instantiation. A miss is `CheckErrorKind::BoundNotSatisfied` (`src/check/error.rs:107`): `{function}: type parameter {param} is bounded by {bound}; got {got}`. A variable still open is a `PendingBound`. `flush_pending_bounds` (`src/check.rs:18008`) runs at the end of `check_function_body` (the function name is the defn path, `src/check.rs:1906`) and at the end of `check_form` (the label is `"<form>"`, `src/check.rs:1965`). Still a variable: `BoundUnresolved`. The generic function's own rigid `:T` is the parameter, not that error.

Inside the body, `assignable` (`src/check.rs:17308`) treats a path whose bare name is in the current bound stack as the bound: `assignable(bound, expected)`. The hook only returns true; `assignable(:T, :T)` still falls through. Named `defn` bodies keep rigid `Path(":T")` and push the scheme's bounds for the body. An anonymous `fn` freshens unbounded names and leaves bounded names as paths `:T` for the body. The `TypeExpr::Fn` it returns is `expose_bounds` (`src/function/infer.rs:255`), so a caller sees the bound where the letter stood.

## The rows

`cargo test --test types probe_arc255_51` — 10 passed, rc 0. Each refusal matches kind and names.

| row | result |
|---|---|
| `:u::pass` of `:u::In` | i64 1 |
| anonymous `fn` with `[T :< :u::Mark]` of `:u::In` | i64 1 |
| `:u::pass` of `:u::Out` | `BoundNotSatisfied` function `:u::pass` param `T` bound `:u::Mark` got `:u::Out` |
| unbounded `:- [T]` passed to `:u::takes-mark` | `TypeMismatch` callee `:u::takes-mark` param `#1` expected `:u::Mark` got `:T` |
| `(:u::pass (nth [] 0))` inside `:u::bad` | `BoundUnresolved` function `:u::bad` param `T` bound `:u::Mark` |
| `defn` binder `[T :- :u::Mark]` | Runtime `MalformedForm` head `:wat::core::defn`, reason names `[T :- :u::Mark]` |
| `defn` binder `(T <- :u::Mark)` | same head, reason names `(T <- :u::Mark)` |
| method binder `[T :- :u::Mark]` | Type `MalformedDecl` head `:wat::core::defsurface`, reason `method member \`go\`: … got [T :- :u::Mark]` |
| `extend-type :- [:u::Mark]` | Type `MalformedDecl` head `extend-type`, reason names `:u::Mark` |
| `extend-type :- [[T :< :u::Mark]]` | Type `MalformedDecl` head `extend-type`, the later-stone sentence naming `[T :< :u::Mark]` |

Count against 6145 at `352eada3d`: the floor ran 6155, which is 6145 plus these 10. 6154 passed. The one failure is the fixture above, not one of the rows.

Clippy was not run. Census and delta were not run. No pre-census was taken on the unmodified draw, so there is no pre/post pair for this stone.

## Later stones, unchanged

Multi-clause `defclause` whose clauses do not share a signature-letter set. Letters are the free-type-var rule (bare, no `::`, first letter upper). `sort` and `sort-by` are the known pair: clause 1 of each lacks `Cmp` (`wat/core.wat:1602` vs `:1609`; `:1625` vs `:1633`). Six others:

| form | where | letters by clause |
|---|---|---|
| `:wat::bracket::thread-enter` | `wat/bracket.wat:226`, `:229` | `O D I`, then those plus `W` |
| `:wat::bracket::process-work-forms` | `wat/bracket.wat:377`, `:516` | none, then `W` |
| `:wat::core::reductions` | `wat/seq.wat:655`, `:658` | `U T`, then `T` |
| `:wat::kernel::spawn-program` | `wat/spawn.wat:373`, `:383` | `S R`, then `I O` |
| `:wat::test::spawn-peer` | `wat/test.wat:354`, `:359` | `S R`, then `I O` |
| `:my::route` | `tests/types/probe_stone_118_b2c_unreachable_arm_refused.wat:32` | none, then `W` |

40 multi-clause forms in the git-tracked corpus, 48 single-clause. No multi-clause form other than `sort` / `sort-by` has a `<` in a clause; those two call `<` inside the comparator function, not as the clause body.

`derive` whose parent is a surface: 0. Nine `(:wat::core::derive` forms are tracked. Their parents are `:probe::Marker`, `:t::Marker`, `:usr::Parent`, `:probe::Th`, none of which is a `defsurface`. `wat/service.wat:1439` emits `` `(:wat::core::derive ~enum-name ~service-op-kw) ``; the parent is the service `Op` enum. `Spawned` is a surface and is joined by `extend-type`, not by `derive`.

`sort` / `sort-by` results, git-tracked `*.wat` and `*.wat.bad`, calls whose head is exactly that name, the definition form excluded. `sort`: 113 calls in 100 files, 4 under `wat/` (three are `wat/fix.wat:1287`, `:1350`, `:1478`). `sort-by`: 7 calls in 5 files, 3 under `wat/` (`wat/query/mem.wat:136`, `:163`, `wat/bracket.wat:769`). The ruling's earlier count was 75 / 72 / 5 and 4 / 3 / 3. This count is the remeasure.

Of those, 45 `sort` calls are the direct argument of `:wat::core::reverse`. 23 are the direct argument of a `vec->pvec` whose parameter is `(Vector :- [:wat::core::i64])` (the shape at `wat-scripts/perf/grid/negation.wat:84`, the call at `:94`). None is the direct argument of `nth`, `get`, or `conj`. One let-bound `sorted` is passed to `length` (`tests/collection/sort.wat:51`). One let-bound `sorted` is passed to `into` (`wat-scripts/scratch-pad/probe-derive-decomposition.wat:95`).

`reverse` does not accept a `Stream`. `StreamContainer::ordered` is false for `Stream` (`src/collection/seq_container.rs:300`). `infer_reverse` then expects `(Vector :- [T])`, `(PersistentVector :- [T])`, or `(List :- [T])` (`src/collection/infer.rs:1083`). The intrinsic's argument note says a `(Stream :- [T])` is refused (`src/intrinsic/collection.rs:348`).
