# SCORE — STONE 255.38: the owner's end is its own type

Struck against draw `ce4051d0e` (parent `3fdb5eb4b`). `Spawned` is a surface.
Thread and Process implement it. They do not derive `Peer`. An owner handle
and a counterparty end are two types. `recv` still returns `RecvOutcome`.

## The surface

`(:wat::core::defsurface :wat::spawn::Spawned :- [S R] :nature :wat::core::Struct)`.
`S` is what the owner sends, `R` what the owner receives. A surface with those
parameters and no feature that names them is refused (`UnconsumedTypeParam`,
measured on the empty struct in 255.37). `send` and `recv` stay intrinsics: a
counterparty can do those too, so they are not what an owner is.

`close` is the owner lifecycle. Its result is `CloseOutcome`, and
`wat/kernel/outcomes.wat` loads after `wat/spawn.wat` because `AcceptOutcome`
names `Peer`, which deporder attributes to `spawn.wat`. This file cannot write
`CloseOutcome`. The feature that consumes `S` and `R` is `owner-end?`,
returning bool. Thread and Process implement it with

```
(:wat::core::extend-type :- [S R]
  (:wat::kernel::Thread :- [S R])
  (:wat::spawn::Spawned :- [S R])
  (owner-end? [self] -> :wat::core::bool true))
```

and the same for Process. The two `derive … Spawned` markers and the two
`derive … Peer` edges are gone.

The family rule reads that edge. `family_extends` walks the generic
`extend-type` binder, arguments ignored. `project_peer_io`, `select`, and
`poll` accept a 2-argument head that satisfies `Spawned`, or the Peer head
through `is_peer_head`. `close` accepts only `Spawned`. `==` compares of
`Thread` / `Process` / `Peer` in `src/check.rs`: 16 before, 6 after. The rule
replaced 10. The heresy ledger is tightened 208 → 198: `infer_poll_prime`
3→1, `infer_select_prime` 4→1, `infer_close_prime` and `project_peer_io` left
the ledger. The one compare left in `select` and in `poll` was already there.

## The cascade

Each step is a blank-file `--check` after a rebuild. The first step, edges
gone and the rule in place, seams still `Peer`, was the four errors from
255.37.

| step | retyped | reds after |
|---|---|---|
| 1 | `Locus/spawn-runner` returns `(Spawned :- [PoolMsg Tuple])` | the two spawn-runner errors leave. `bracket.wat`'s mapv closure still declares `Peer` and now produces `Spawned`. `Launched` stays |
| 2 | that mapv return | `collect-loop`'s peer vector, and the revoke `foldl`, expect `Peer` |
| 3 | `collect-loop`'s vector | the `foldl` remains |
| 4 | the `foldl` parameter | only the two `Launched` constructors |
| 5 | `Launched.handle` is `(Spawned :- [Sh Lu])` | every defservice `Handle` field, one template, 54 errors at the generated constructors (`cache`, `stdio`, `query`, `telemetry`) |
| 6 | `service.wat` `handle-peer-ty` is `Spawned` | stdlib blank is rc=0 |
| 7 | `recv-all` and `recv-all-loop` take `(Spawned :- [I O])` | the three `t18` recv-all calls go green. `counter-proc::*` still expects `Peer` and is handed a `Process` |
| 8 | those four `counter-proc` parameters | that file is rc=0. Census then flips one scratch probe |
| 9 | `try-with-lineage`'s lineage parameter. The client parameter stays `Peer` | census rc unchanged |

Eleven judgements. No site had to accept either family in one parameter.
`try-with-lineage` takes both, as two parameters: the connected client is
`Peer`, the lineage handle is `Spawned`.

## Rows

Pre-stone words are the draw binary, before this stone's rebuild. `rc` is the
next statement.

| row | pre | post |
|---|---|---|
| `Thread` returned as `Peer` | rc=0 | rc=1, `ReturnTypeMismatch`: body is `Thread`, signature declares `Peer` |
| `Peer` passed where `(Spawned :- [i64 i64])` is expected | the surface did not exist yet; this probe was not on the draw binary | rc=1, `TypeMismatch`: `need-owner` expects `Spawned`, got `Peer` |
| send and recv on a `Thread` and on a `Process` | the only errors are the two `close` calls, `DefRestrictedCallerNotAllowed` from `:user::` | the same two errors. `close` from `:user::` stays kernel-restricted. The type rule accepts the owner: a blank stdlib check is rc=0 |

## Gates

The first floor, `.floor/2026-09-25T19-59-22Z`, was red. One failure.
`verify_stdlib_has_no_load_order_violations` panicked at
`tests/kernel/test_stdlib_load_order.rs:20`: `assert_eq` left 3, right 0.
The three violations were `spawn.wat` naming `CloseOutcome` in `outcomes.wat`.
That floor was not re-run. The feature no longer names `CloseOutcome`.
`verify-stdlib` then prints 0 violations. The floor below is a new run.

Floor `.floor/2026-09-25T20-09-56Z`:

```
Summary [ 347.892s] 6124 tests run: 6124 passed (10 slow), 22 skipped
```

`FLOOR_RC=0`. Clippy `cargo clippy --all-targets --workspace -- -D warnings`
is 0. Census `.census/2026-09-25T20-08-14Z.txt` against the pre-change
`.census/2026-09-25T19-46-04Z.txt`: `census-diff: no STOP-8`, files 2280,
nonzero 215. One file, `probe-arc278-reap-which-link.wat`, went 0 → 1 on the
census taken before its lineage parameter was retyped; the census above is
after that retype. Delta `.delta/2026-09-25T20-09-09Z`: NEW 2, RECOVERY 0.
The two NEW files are the standing pair. The ledger test passed at 198.
