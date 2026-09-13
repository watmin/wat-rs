# SCORE — 0b batch 2d: close stone 0b

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched.
Binary: `./target/release/wat` (boots). Floor + clippy run.

```
gate  cargo nextest run --release --test cli -E 'test(every_recorded_migration_replays)'
      Summary [  31.751s] 18 tests run: 18 passed, 57 skipped
floor scripts/floor.sh               .floor/2026-09-13T01-39-36Z
      Summary [ 229.921s] 5391 tests run: 5391 passed (1 slow), 22 skipped
clippy cargo clippy --release --all-targets -- -D warnings   CLIPPY_EXIT=0
```

Row 0: `NO_CODEMOD_WAT`. TIME ONE FILE FIRST: `rename-record-def-to-defrecord` on the restored landing form, 0.276 s, rc=0. Then `namespace-bare` on `probe01` parent, 0.262 s, byte-matched history.

## EXPECTATIONS

| # | result |
|---|---|
| D1 | (l)(m) RED, reverted. Verbatim below. |
| D2 | `spec-before` parsed and checked; 24 fixtures backfilled; (n) RED, reverted. |
| D3 | `namespace-bare-top-level-names` ORACLE is `history 72a1ac3d0 tests/types/probe_arc237_sA1_assignable_probe01.wat` — no `spec`. |
| D4 | `history 5b86828f8 wat-scripts/perf/grid/where-shapes.wat` from the readable `rule-arith` form. Whole-file parents are unreadable (`PersistentVector<…>` at byte 13765). |
| D5 | migrate second arg is `":wat::Record::def"`; fixture `history 60d7d99a0 wat-tests/core/record-def.wat`; near-misses `holon::defrecord` and `core::defstruct` stay. Header prose restored from landing (SCOPE line kept). |
| D6 | table below. REPRODUCES → history. The one rot (`rename-record-def`) repaired (D5). |
| D7 | `git diff d1c47b2a4 -- positional-ctor-to-map.wat bare-variant-to-qualified.wat` empty. |
| D8 | **5391 passed, 0 failed**. Clippy **0**. Per-shard times below. |

## D1 / D2 — the gate FAILS, ×3 (reverted)

**(l)** `history 6d88073d8 <real> no/such/file.wat`:

```
kill-make-deftest: history path exists at neither 6d88073d8 nor 6d88073d8^: no/such/file.wat
```

**(m)** `history this-commit-does-not-exist wat/core.wat`:

```
kill-make-deftest: history commit does not exist: this-commit-does-not-exist
```

**(n)** mixed `match-arm-to-bracket-map-pattern`, `(n n)` → `(n n n)`:

```
match-arm-to-bracket-map-pattern: changed before.pre line not in history parent and not a spec-before entry:     (n n n) (_ 0)))
```

Each reverted.

## D6 — audit of every spec-bearing stem

Timed on one stem first (`namespace-bare` / `probe01`, 0.262 s). Then every landing corpus `.wat` outside `wat-scripts/fixes/`.

| stem | landing | per-file class | action |
|---|---|---|---|
| namespace-bare-top-level-names | `72a1ac3d0` | REPRODUCES on the 5 named probes | **history** (D3) |
| rule-record-to-defrule | `5b86828f8` | UNREADABLE all 9 where-grid files (angles); NEW 3 scratch probes | **history** from extracted `rule-arith` (D4) |
| rename-record-def-to-defrecord | `60d7d99a0` | ROT: tool prefix was `:wat::core::Record::def` (a name that never existed). Restored. | **repaired + history** (D5) |
| rename-locidiederror-shutdown-to-stopped | `ff7756630` | REPRODUCES 2 diagnostic files; UNREADABLE 3 (angles) | **history** |
| mandatory-typed-quasiquote-residual | `284cd7c93` | REPRODUCES `wat/bracket.wat` + `wat/service.wat`; NO-OP census | **history** + spec for the isolated quasiquote's backtick line |
| bare-symbol-shorthand-to-fqdn | `e9ff4fee4` | REPRODUCES 2/2 | keep mixed (Some is spec; 0 Some sites at landing) |
| bare-none-keyword-to-fqdn | `d64411a0d` | NO_CORPUS (0 `.wat`) | spec |
| to-faithful-clojure-net | `1a5f9c0c7` | NO_CORPUS | spec |
| sweep-lint-fixes | `26d408f7f` | NO_CORPUS | spec |
| strip-match-ascription | `fcab05472` | NEW only (already-bare file) | spec |
| service-locus-to-user-rendezvous | `b18888f84` | UNREADABLE `wat/service.wat` + `wat/spawn.wat` | spec |
| fix-macro-param-types | `b1863d1f9` | UNREADABLE (`:AST<…>`) | spec |
| to-faithful-clojure | `384f5d6fc` | UNREADABLE `wat/core.wat` | spec |
| to-faithful-clojure-rete | `83b291f9b` | DIFFERS (rewrote the probe's rete rules, not a corpus conversion) | spec |
| rewriting-rules-join-written | `0d51a1305` | NO-OP `wat/grep.wat` (tool added, not applied) | spec |
| restore-verbatim-literals-after-the-fold | `0d51a1305` | NO-OP `wat/grep.wat` | spec |
| rename-wat-record-to-core-record | `086321255` | UNREADABLE both perf files (angles); mixed with record-def→defrecord | spec |
| reclaim-ipc-prime-names | `890b60a43` | mixed 3-tool commit: NO-OP / DIFFERS / UNREADABLE | spec (mixed) |
| query-answers-are-maps | `d2d73dc39` | mixed with type-query: NO-OP / UNREADABLE / NEW / 1 DIFFERS | spec (mixed) |
| type-query-to-defquery | `d2d73dc39` | same mixed landing | spec (mixed) |
| edits-carry-the-old-text | `acc95652d` | DIFFERS on readable `wat/fix.wat` `wat/lint.wat` (judgement-call old-text; RULE 1 is spec) | spec (mixed explanation already in 0b1) |
| rename-core-bigint-rational-to-their-homes | `1a3f6a703` | history + slash form never a wat keyword | keep mixed |
| rename-string-verbs-to-their-home | `315bbf546` | REPRODUCES several; `ends-with?` still spec | keep mixed |
| eprintln-recv-arm-to-assertion-failed | `1212c9ae6` | mixed 3-tool 395-file landing; repair emit is spec | keep mixed |
| mandate-request-malformed | `b9d61bd67` | REPRODUCES some; DIFFERS on others (today emits `Vector :-` not `Vector<String>`) | keep mixed (repair) |
| match-arm-to-bracket-map-pattern | `480f38d05` | history half-broken + 2a field/unit spec | keep mixed |
| bare-variant-to-qualified | `49f03f179` | landing mixed H-2 kwargs; header spec | spec |
| unwrap-recvoutcome-false-positive | `1212c9ae6` | mixed 3-tool landing (`__recv` is wrap-client's after) | spec (mixed) |
| wrap-client-method-match-in-recvoutcome | `1212c9ae6` | mixed 3-tool landing | spec (mixed) |

No remaining NO-OP/DIFFERS-on-readable without a mixed-commit or already-cited judgement-call explanation. The one rot was D5.

## D7

Untouched. Composition hazard stays on SEAM.md.

## D8 — shard wall times (`.floor/2026-09-13T01-39-36Z`)

| test | s |
|---|---|
| shard_0 | 9.155 |
| shard_1 | 8.098 |
| shard_2 | 8.376 |
| shard_3 | 8.718 |
| shard_4 | 8.577 |
| shard_5 | 9.169 |
| shard_6 | 11.528 |
| shard_7 | 8.860 |
| shard_8 | 9.819 |
| shard_9 | 6.845 |
| shard_10 | 7.504 |
| shard_11 | 7.232 |
| shard_12 | 6.708 |
| shard_13 | 7.373 |
| shard_14 | 7.232 |
| shard_15 | 6.256 |
| positional_ctor (isolated) | **83.378** |
| coverage | 2.535 |

Light shards max **11.528s**. Isolated positional is the one SLOW (>60s warn); kill 120s.

## STOPs

None. No `src/`. No push. Main untouched.
