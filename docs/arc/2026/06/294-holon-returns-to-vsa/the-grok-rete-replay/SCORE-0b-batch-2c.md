# SCORE — 0b batch 2c: an oracle's provenance is CHECKED, not claimed

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched.
Binary: `./target/release/wat` (boots). Floor + clippy run.

```
gate  cargo nextest run --release --test cli -E 'test(every_recorded_migration_replays)'
      Summary [  31.042s] 18 tests run: 18 passed, 57 skipped
floor scripts/floor.sh               .floor/2026-09-13T01-09-42Z
      Summary [ 209.870s] 5391 tests run: 5391 passed (1 slow), 22 skipped
clippy cargo clippy --release --all-targets -- -D warnings   CLIPPY_EXIT=0
```

Row 0: `NO_CODEMOD_WAT` before/after strike runs. One wat at a time. TIME ONE FILE FIRST:
`defrule-then-to-vector` on `chain-fp.wat`'s parent form, 0.268 s, rc=0, after matched history.

## EXPECTATIONS

| # | result |
|---|---|
| C1 | `ls wat-scripts/fixes/replay/*/ORACLE \| wc -l` **= 90** |
| C2 | gate 18/18. Arms (i)(j)(k) each RED, reverted. Verbatim below. |
| C3 | of the 38: **21 history**, **17 spec**. Table below. |
| C4 | 12 net + 3 rete rules, each mapped to a fixture line with a near-miss. |
| C5 | `grep -c 'defn :fix::\(has-ns?\|type-shaped?\|head-keyword-str?\|type-shaped-keyword-str?\)'` **= 0** in both files |
| C6 | `rename-wat-record-to-core-record.wat` lines 6–14 equal `086321255`. SCOPE moved to line 15 so the landing slice is byte-equal. |
| C7 | **5391 passed, 0 failed**. Clippy **0**. Per-shard times below. |

## C2 — the gate FAILS, ×3 (reverted)

**(i)** `kill-make-deftest/ORACLE` rewritten to `history 0d51a1305 wat/core.wat` (that commit lacks the line):

```
kill-make-deftest: changed before.pre line not in history parent: (:wat::test::make-deftest :deftest)
```

**(j)** `bare-none-keyword-to-fqdn/ORACLE` rewritten to a spec quote that is not in the header:

```
bare-none-keyword-to-fqdn: spec quote not in header: this quote is not in the header at all
```

**(k)** `kill-make-deftest/ORACLE` removed:

```
kill-make-deftest: no ORACLE file
```

Each reverted. Coverage green after.

Mixed history+spec fixtures (slash form, 2a Some, field-bearing match, repair emit): a changed before line of a spec-only case is not in the history parent — that is why it is spec. The before-check therefore applies only when the ORACLE is history-only.

## C3 — the 38, history where it exists

### History (21)

| stem | commit | path |
|---|---|---|
| defrule-then-to-vector | `1a0d503aa` | `wat-scripts/fixes/rete-truth-maintenance-probes/chain-fp.wat` |
| drop-env-wat-dot-prefix | `fe071b6d7` | `tests/services/probe_arc209_c0b3bd_user_program_foundation.wat` |
| face-underscore-bound-send-prime | `53bdfb0a1` | `wat-scripts/probes/arc-170/probe-s1-impure-gate.wat` |
| first-of-drop-to-nth | `8c28ace25` | `wat/deporder.wat` |
| kill-make-deftest | `6d88073d8` | `crates/wat-holon-lru/wat-tests/holon/lru/HologramCache.wat` |
| mandate-invocation-ctx-param | `037ef43ef` | `tests/services/probe_arc278_dead_child_speaks.wat` |
| reclaim-hologram-find-name | `890b60a43` | `wat-tests/cache/HolographicLru.wat` |
| reclaim-stdio-prime-names | `eae450014` | `wat/kernel/services/stdio-primes.wat` |
| rename-call-ctx-to-invocation | `9aa3476fb` | `tests/services/probe_arc278_call_context.wat` |
| rename-diederror-to-loci-died-error | `d60b1887a` | `tests/kernel/wat_hermetic_round_trip.wat` |
| rename-kernel-to-spawn | `1c51a2fd3` | `wat/spawn.wat` (the `derive Thread' Spawned` form; `ServiceEvent<I,O>` is unreadable today) |
| rename-list-to-seq | `5d16c933a` | `wat-tests/core/list-fold-aliases.wat` + `seq-fold-aliases.wat` (same-commit rename) |
| rename-seq-fold-aliases-to-core-reduce | `a9878d460` | `wat-tests/core/core-reduce.wat` |
| rename-sourcefile-to-source-file | `aa48a9799` | `wat/lint.wat` (later apply; landing `79f89773d` changed 0 corpus `.wat`) |
| rename-wat-tests-std-to-wat-tests | `8b799c362` | `wat-tests/test.wat` |
| retarget-peer-purity-probes | `890b60a43` | `tests/comms/probe_arc293_W2c_compile_time_send.wat` |
| rete-oracle-sigil | `d0973fb14` | `tests/rete/probe_arc278_insert_all_differential.wat` |
| spawn-program-to-test-spawn-peer | `8b799c362` | `tests/kernel/wat_run_sandboxed.wat` |
| stdin-frame-vocabulary | `357a223ad` | `tests/services/probe_arc170_stdio_prime.wat` |
| struct-new-failure-to-message-only-failure | `4543ef7a6` | `wat/kernel/sandbox.wat` |
| wrap-connect-prime-in-connectoutcome | `1e7065a24` | `tests/comms/probe_arc272_autobind_listener.wat` |

Today's tool on each parent form produced the historical child (byte-match, then keep-forms restored around it).

### Spec (17) — why history failed

| stem | why no history form |
|---|---|
| fix-macro-param-types | landing rewrites `:AST<wat::core::nil>` → `:wat::WatAST`. The preimage has angle brackets; today's reader refuses it. |
| namespace-bare-top-level-names | historical `:pi` → `:t::pi` sites are no-op under today's tool (a different rewrite than namespacing a bare defn name). |
| query-answers-are-maps | mixed landing `d2d73dc39` with `type-query-to-defquery`; extracted `query-by-type-string` forms are the other tool. |
| reclaim-ipc-prime-names | landing `890b60a43` mixed three reclaim/retarget tools; `send'` sites either rc=2 (`Peer'<…>`) or today's after lines are not in that commit. |
| rename-locidiederror-shutdown-to-stopped | every parent form containing `LociDiedError::Shutdown` also carries `Name<T>` (unreadable). |
| rename-record-def-to-defrecord | today's migrate prefix is `:wat::core::Record::def` (the tool was itself migrated). Historical `:wat::Record::def` is a no-op. |
| rename-wat-record-to-core-record | landing `086321255` mixed this prefix-rename with `Record::def` → `defrecord`. No file that commit changed ONLY by `:wat::Record` → `:wat::core::Record`. |
| rule-record-to-defrule | rete probes are no-op (not the where-grid `Rule` record shape the tool walks). |
| service-locus-to-user-rendezvous | only hits are whole `wat/service.wat` / `wat/spawn.wat`, both angle-bracketed. |
| strip-match-ascription | landing added a NEW already-bare file; parent of that path is empty. Other `match → :T` sites sit in mixed-ascription files. |
| sweep-lint-fixes | landing `26d408f7f` changed 0 corpus `.wat`. No later apply of the 3-rung `if-=` ladder. |
| to-faithful-clojure | landing `384f5d6fc` added rust-scheme to `wat/core.wat`; it did not convert a form to clojure. |
| to-faithful-clojure-net | landing `1a5f9c0c7` changed 0 `.wat`. |
| to-faithful-clojure-rete | landing `83b291f9b` rewrote the *probe's rete rules*, not a corpus conversion. |
| type-query-to-defquery | an isolated `query-by-type-string` form rc=2 (`heretic query in a file with no compile`); snippet after is not in the full-file commit. |
| unwrap-recvoutcome-false-positive | `__recv` wrap is the AFTER of wrap-client; parent of `1212c9ae6` does not contain it. |
| wrap-client-method-match-in-recvoutcome | mixed 395-file landing; smallest changed forms are other tools (strip-arrow on `if`/`readln`). |

## C4 — to-faithful coverage

One fixture per stem. Near-misses are unchanged in before/after (oracle does not check them).

### `to-faithful-clojure-net` (12)

| rule | fires on | near-miss |
|---|---|---|
| g1-keyword | `:wat::core::defn`, `:wat::core::if`, `:keep` | already-clojure `(wat.core/identity 1)` (not a keyword) |
| g2-symbol | param `x` | a keyword is not a symbol |
| g3-genuine | those keywords (span-len = name-len) | comment `:wat::core::defn` (not a node) |
| g4-namespaced | `::` in defn/if/i64/nil/Tuple(i64) | `:keep` (no `::`) |
| g5-type-shaped | `:wat::core::Tuple(i64)` (`(`+`)` in the name) | namespaced non-paren head `defn` |
| g6-arrow | `<-` / `->` | symbol `x` is not an arrow |
| g7-post-arrow | `i64` after `<-`, `nil` after `->` | first child is not post-arrow |
| tc-from-shaped | Tuple(i64) | a non-shaped keyword |
| tc-from-postarrow | `i64`, `nil` | a genuine keyword that is not post-arrow |
| t1-head-conv | `defn`, `if` → `wat.core/defn`, `wat.core/if` | `:keep` (not namespaced); Tuple(i64) is TypeShaped so not HeadConv |
| t2-type-conv | `i64`/`nil`/`Tuple(i64)` → `wat.type/…` | a head keyword |
| t3-arrow-conv | `<-`/`->` → `:-` | a non-arrow symbol |

Angle-bracket type-shaped (`Head<…>`) is unreadable today; the paren tuple spelling exercises g5.

### `to-faithful-clojure-rete` (3)

| rule | fires on | near-miss |
|---|---|---|
| head-keyword->conv | `:wat::core::defn`, `:wat::core::if` | already-clojure; `:keep` (no `::`) |
| arrow->conv | `<-` / `->` | symbol `x` |
| type-keyword->conv | post-arrow `i64`/`nil`; type-shaped `Tuple(i64)` | a namespaced non-type head |

Same near-miss comment `:wat::core::defn`.

## C7 — shard wall times under the floor (`.floor/2026-09-13T01-09-42Z`)

| test | s |
|---|---|
| shard_0 | 7.565 |
| shard_1 | 6.993 |
| shard_2 | 7.571 |
| shard_3 | 6.951 |
| shard_4 | 7.151 |
| shard_5 | 7.069 |
| shard_6 | 9.089 |
| shard_7 | 6.994 |
| shard_8 | 8.098 |
| shard_9 | 5.788 |
| shard_10 | 6.157 |
| shard_11 | 6.128 |
| shard_12 | 5.811 |
| shard_13 | 6.076 |
| shard_14 | 5.985 |
| shard_15 | 6.205 |
| positional_ctor (isolated) | **64.105** |
| coverage | 1.498 |

Light shards max **9.089s**. Isolated positional is the one SLOW (>60s warn); kill 120s.

## Finding 3 / 4

Dead helpers deleted. Prose at lines 6–14 restored from `086321255`.

## STOPs

None. No `src/`. No push. Main untouched.
