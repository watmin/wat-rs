# SCORE — 0b batch 2: the ledger is gone

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched.
Binary: `./target/release/wat` (boots). Floor + clippy run.

```
gate  cargo nextest run --release --test cli -E 'test(every_recorded_migration_replays)'
      Summary [  31.295s] 18 tests run: 18 passed, 57 skipped
floor scripts/floor.sh               .floor/2026-09-13T00-06-44Z
      Summary [ 211.259s] 5391 tests run: 5391 passed (1 slow), 22 skipped
clippy cargo clippy --release --all-targets -- -D warnings   CLIPPY_EXIT=0
```

Row 0: `NO_CODEMOD_WAT` before/after strike runs. One wat at a time.

Commits this batch (not pushed):
- `5102bd478` 2a: 7 coverage gaps; git rm variant-vector; tuple-parens SCOPE
- `997c010ef` 2b: 40 fixtures, 5 runes, isolate positional-ctor
- this SCORE's commit: remaining fixtures, repairs, ledger deleted, arms e–h

## EXPECTATIONS (addendum + B2)

| # | result |
|---|---|
| A1 | **7/7 gaps closed** in 2a. Lines cited below. |
| A2 | `variant-vector-to-tagged-map.wat` **gone**; no ledger row; stems **95** |
| A3 | `grep -c '^;; SCOPE: ' …/tuple-parens-to-binder.wat` **= 1**; globs `wat-scripts/` `wat-tests/` `tests/` hit |
| A4 | per-shard table below. Light shards max **9.342s**. Isolated `positional_ctor` **64.201s** (4s over the 60s target; kill is 120s; isolated ~30s). |
| B2.1 | `grep -c FROZEN_LEDGER tests/cli/every_recorded_migration_replays.rs` **= 0** |
| B2.2′ | fixtures **90** + runes **5** = **95**; no stem in both |
| B2.3 | 5 runes; each is the reader's lex-error `:message` from running that codemod on its angle-bracket preimage. Verbatim in the header. |
| B2.4 | `to-faithful-clojure-net` / `-rete` **repaired**; fixtures pass; no `:fix::` user fn in any `:wat::rete::where` |
| B2.5 | repair list below |
| B2.6 | `grep -L '^;; SCOPE: ' wat-scripts/fixes/*.wat` **empty** |
| B2.7 | (e)–(h) each RED, reverted. Verbatim below. |
| B2.8 | provenance/coverage/near-miss per fixture; named below for repairs and the 7 gaps. Remaining fixtures: history file or header spec as cited in 2a/2b and the fixture comments. |
| B2.9 | **5391 passed, 0 failed**. Clippy **0**. |

## A1 — the 7 gaps (closed in 2a)

| stem | missing case | oracle | near-miss |
|---|---|---|---|
| positional-ctor-to-map | `(:usr::Shape::Circle 2)` → `{:r 2}` | history `434d7fac6` `tests/types/probe_arc296_h2__variant.wat` | already-map `Circle {:r 1}` |
| match-arm-to-bracket-map-pattern | field-bearing + unit arms | **spec** (header) | `cond`; already-Vector arm |
| assertion-failed-to-kwargs | actual/expected 3-arg | history `de2d9092a` `tests/comms/probe_arc209_structured_peer_death.wat` | already-kwargs |
| one-param-spec | unmarked `Vector [i64]` + BARE HashMap | history `919a551ce` `arc109-tuple-bracket-reader.wat` body + `probe_arc216_stone3_hashmap_roundtrip.wat` | already-`:-` Vector |
| repoint-retired-heads-to-live-spellings | reduce-walk; tuple-get t 0 | history `74b8d4df8` bench + arc109-2iii | `tuple-get t 1`; `thread` |
| node-kind-string-to-enum | `string::not=` on Node :kind | history `445309d5d` `wat-scripts/fmt/rules/kwargs.wat` | Named `string::not=` |
| bare-symbol-shorthand-to-fqdn | `Some` | **spec** (header; landing rewrote 0 Some sites) | already-keyword `:wat::core::Some` |

## A2 — delete identity

`git rm wat-scripts/fixes/variant-vector-to-tagged-map.wat`. Header: *"it currently identity-rewrites."* Builder ruling 2026-09-12. Arc 296 SCOREs that mention it left untouched.

## A4 — shard wall times under the floor (`.floor/2026-09-13T00-06-44Z`)

| test | s |
|---|---|
| shard_0 | 7.367 |
| shard_1 | 6.973 |
| shard_2 | 7.132 |
| shard_3 | 7.422 |
| shard_4 | 7.574 |
| shard_5 | 7.320 |
| shard_6 | 9.342 |
| shard_7 | 6.932 |
| shard_8 | 7.965 |
| shard_9 | 5.873 |
| shard_10 | 5.881 |
| shard_11 | 6.448 |
| shard_12 | 6.099 |
| shard_13 | 6.023 |
| shard_14 | 5.898 |
| shard_15 | 5.921 |
| positional_ctor (isolated) | **64.201** |
| coverage | 0.762 |

16 shards skip `positional-ctor-to-map`. Dedicated test runs it twice (~15s isolated × 2). Under floor it is the one SLOW (>60s warn); kill 120s. A4's 60s cap is missed by 4s on that one test.

## B2.3 — runes (unreadable-preimage)

All five: today's reader lex-refuses `<` in a name. Verbatim `:message` is on the header. Orchestrator reproduces by running the codemod on:

| stem | preimage | byte |
|---|---|---|
| angle-brackets-to-binder | `:wat::core::Vector<wat::core::i64>` | 48 |
| parametrics-take-a-type-vector | same | 48 |
| address-transport-arity | `:wat::kernel::Address<i64,i64>` | 51 |
| unstamp-transport-wire | `Address<i64,i64,Wire>` | 51 |
| timer-prime-to-peer-prime | `:wat::kernel::Timer'<i64>` | 50 |

`tuple-parens-to-binder` walks **raw text** and **is fixtured** (reader would refuse `:(A,B)`, the tool does not parse).

## B2.5 — repairs

| stem | symptom | root cause | change | fixture |
|---|---|---|---|---|
| mandate-request-malformed | second replay rc=2 lex error | emit string still used `Vector<String>` | emit `(:wat::core::Vector :- [:wat::core::String])` | `replay/mandate-request-malformed/` |
| to-faithful-clojure-net | rc=2 `where expr is not a rete primitive` / `not total` | `:fix::has-ns?`, `:fix::type-shaped?`, `:wat::core::+` inside `:wat::rete::where` | `string::contains?` / `core::or`/`and`; `i64::+` | `replay/to-faithful-clojure-net/` |
| to-faithful-clojure-rete | rc=2 `:fix::head-keyword-str?` not a rete primitive | user fns in `where` | same rete primitives | `replay/to-faithful-clojure-rete/` |
| rename-wat-record-to-core-record | identity no-op | migrate had been rewritten to `:wat::core::Record` → itself after the corpus moved | restore landing prefixes `:wat::Record` → `:wat::core::Record` | `replay/rename-wat-record-to-core-record/` |

## B2.7 — the gate FAILS, ×4 (reverted)

**(e)** stripped `;; SCOPE:` from `kill-make-deftest.wat`:

```
kill-make-deftest: expected exactly one `;; SCOPE:` line, found 0
```

**(f)** `;; SCOPE: this/path/does-not-exist.wat`:

```
kill-make-deftest: SCOPE glob `this/path/does-not-exist.wat` matches no tracked file
```

**(g)** empty rune reason on `angle-brackets-to-binder.wat`:

```
angle-brackets-to-binder: rune:replay(unreadable-preimage) has an empty reason
```

**(h)** added a rune to fixtured `kill-make-deftest.wat`:

```
kill-make-deftest: in more than one category (fixture=true rune=true)
```

Each reverted.

## Gate shape

- `FROZEN_LEDGER` deleted.
- Coverage: every stem is fixture XOR rune, plus exactly one non-empty SCOPE whose globs hit.
- 16 replay shards + isolated `positional_ctor` + coverage = 18 tests.

## STOPs

None. No `src/`.
