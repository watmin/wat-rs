# SCORE — 0b batch 1: the 17 chain codemods replay

Branch: `replay/grok-rete` @ `f0e3eb900` plus this strike. **Committed, not pushed.** Main untouched.
Binary: `./target/release/wat` (boots). Floor + clippy run.

```
B1.7 cargo nextest run --release --test cli -E 'test(every_recorded_migration_replays)'
     Summary [  30.771s] 9 tests run: 9 passed, 57 skipped
B1.8 scripts/floor.sh               .floor/2026-09-12T23-09-16Z
     Summary [ 229.066s] 5382 tests run: 5382 passed (2 slow), 22 skipped
     shard_3 under floor: 78.908s  (why the nextest override exists)
     cargo clippy --release --all-targets -- -D warnings   CLIPPY_EXIT=0
```

Row 0: no `wat` / cargo-wat / nextest process before or after the strike runs. `NO_CODEMOD_WAT`. One wat at a time.

## The strike

1. 16 fixtures under `wat-scripts/fixes/replay/<stem>/{before.pre,after.post}` (plus `stdin` on `one-param-spec`). The 17th chain stem, `variant-vector-to-tagged-map`, is **not fixtured** — see Finding 1.
2. `;; SCOPE:` on the 17 chain stems + the 11 from 0a + A + B (30). Tooling scopes are explicit globs, not `corpus`.
3. `FROZEN_LEDGER` shrunk by the 16 fixtured names. 67 left. `variant-vector-to-tagged-map` stays, with the identity reason.
4. Gate coverage now asserts: every fixtured or runed stem has exactly one non-empty `;; SCOPE:`; every glob/`corpus` hits ≥1 tracked file. IO panics (no `Result` flattening of git/fs).
5. `.config/nextest.toml`: override `test(every_recorded_migration_replays)` warn 60s / kill 120s (default + ci). First gate run without it: shard_3 TIMEOUT at 30.004s. Isolated after the override: 30.771s; under floor: 78.908s.

No `src/`. No repairs (batch 2). No push. No main.

## EXPECTATIONS

| # | result |
|---|---|
| B1.1 | **29 dirs, not 30.** 13 from 0a + 16 new. `variant-vector-to-tagged-map` has no fixture (Finding 1). The other 16 named in the brief are present. |
| B1.2 | **67 left, not 66.** None of the 16 fixtured names remain. `variant-vector-to-tagged-map` is the 17th, still ledgered. |
| B1.3 | **1 each** on the 28 chain + A + B (30 stems). `grep -c '^;; SCOPE: '` = 1. |
| B1.4 | tooling scopes are **not** `corpus`. Each glob hits ≥1 tracked file (git ls-files). Table below. |
| B1.5 | **16/16 fixtures cited.** `after.post` never produced by the tool. Independently, today's `./target/release/wat` on `before.pre` matched `after.post` 16/16 (verification, not generation). The 17th has no `after.post` (Finding 1). |
| B1.6 | documented rewrite cases mapped below; ≥1 near-miss per fixture. Gaps vs the header's *full* case list are named, not papered. |
| B1.7 | **9 passed**, 57 skipped. |
| B1.8 | **5382 passed, 0 failed**, 22 skipped. Clippy **0**. |
| B1.9 | ≥1 commit on `replay/grok-rete`; **not pushed**. |

## Finding 1 — `variant-vector-to-tagged-map` cannot be fixtured without STOP-1

Header (`wat-scripts/fixes/variant-vector-to-tagged-map.wat:14–16`):

> it currently identity-rewrites: there is no wat tagged-literal form for the old wire in the corpus (R21: do not invent a rewrite of a shape that is not a wat form).

`migrate` is `src`. Landing `498c09daa` changed 19 `.wat` files; those deltas are comments and EDN-in-strings. A form-tree walk cannot rewrite string contents. The gate forbids `before.pre == after.post` (vacuous). Inventing a rewrite of a shape that is not a wat form is STOP-1 (changes documented behaviour). The reader accepts the pre-image, so this is not `rune:replay(unreadable-preimage)`.

Disposition: **left on `FROZEN_LEDGER`** with reason `stone 0b: identity rewrite — header: no wat tagged-literal form; non-vacuous fixture would require STOP-1`. SCOPE declared anyway (`tests/value/probe_arc278_read_foreign.wat wat-tests/edn/roundtrip.wat`) so B1.3 holds. Batch 2 must decide: the ledger dies, and neither fixture nor rune fits.

Not STOP-1 this batch: we did not repair, did not invent, did not fixture.

## B1.4 — tooling SCOPE hits

| stem | SCOPE | git ls-files hits |
|---|---|---|
| edits-carry-the-old-text | `wat-scripts/fixes/*.wat wat/fix.wat wat/lint.wat wat-scripts/lib/*.wat` | 100 + 1 + 1 + 1 |
| break-kind-string-to-enum | `wat-scripts/fmt/rules/*.wat` | 14 |
| node-kind-string-to-enum | `wat-scripts/fmt/rules/*.wat wat-scripts/fmt/fixtures/*.wat wat-scripts/grep/*.wat tests/cli/wat_grep*.wat` | 14 + 50 + 7 + 9 |
| fmt-head-fqdn-to-clojure | `wat-scripts/fmt/rules/*.wat wat-scripts/grep/*.wat` | 14 + 7 |
| variant-vector-to-tagged-map | `tests/value/probe_arc278_read_foreign.wat wat-tests/edn/roundtrip.wat` | 1 + 1 |
| rename-sort-prime-to-native | three named paths | 1 each |
| bare-symbol-shorthand-to-fqdn | three named paths | 1 each |
| rename-slipped-core-heads-to-their-homes | four named paths | 1 each |
| repoint-retired-heads-to-live-spellings | three named paths | 1 each |
| A rewriting-rules-join-written | 11 named 0a grep-rules files | 1 each (already on 0a) |
| B restore-verbatim-literals-after-the-fold | 5 named files | 1 each (already on 0a) |
| rest of chain + 11 from 0a | `corpus` | tracked `*.wat` outside `wat-scripts/fixes/` |

## B1.5 — oracle provenance

`after.post` was never produced by running the tool. Independently, the converted/new codemod on `before.pre` matched `after.post` 16/16 (verification, not generation).

| stem | after.post from |
|---|---|
| edits-carry-the-old-text | **spec** (header RULE 1: edit-tuple type `i64 i64 String` → `i64 String String`). Landing `acc95652d` mixed judgement-call old-text rewrites; today's tool on `strip-insert-rhs-marker.wat` at that commit is a no-op (got==before). RULE 1 is the mechanical part the header names first. |
| one-param-spec | `919a551ce` `tests/types/ord_vec_i64_gt.wat`. stdin `["{FILE}"]\n["{FILE}"]\n` (two path vectors). First candidate `parametric_target_bad_new.wat` also rewrote the return type `(Vector i64)` → `(Vector :- [i64])`; history after only changed the body — discarded. |
| mandatory-typed-quasiquote-residual | **spec** (header: `` `(:wat::core::Vector ~ty) `` → `` `(:wat::core::Vector :- [~ty]) ``). Landing `284cd7c93` whole `one-param-spec.wat` was the wrong target (that file is the *predecessor* tool). |
| rename-sort-prime-to-native | `1df9e0571` `wat/core.wat` — the `sort` / `sort-by` defclauses only. Whole-file landing mixed hand comment edits. |
| bare-symbol-shorthand-to-fqdn | `e9ff4fee4` `tests/cli/wat_cli__programs_are_atoms.wat` (`(Ok _)` / `(Err _)`). |
| rename-slipped-core-heads-to-their-homes | `63e7a0c7f` `wat-scripts/scratch-pad/t-bare.wat` (`:wat::core::println`) + `wat-scripts/probes/arc-170/probe-edn.wat` (`:wat::core::edn::write`). Near-miss `(:wat::kernel::println "already home")` is **spec** (idempotent / already-home). |
| repoint-retired-heads-to-live-spellings | `74b8d4df8` `wat-scripts/probes/arc-170/probe-cap2-process-grantpath.wat` from first `defn` (header comment at that commit is a hand add). |
| bare-none-keyword-to-fqdn | **spec** (header: `:None` → `:wat::core::None`). Landing `d64411a0d` changed 0 `.wat` (CODEMOD drafted, HELD). No later corpus apply (`git log -S`). |
| break-kind-string-to-enum | `f3c870221` let-bindings `"align"` + one defn.wat `"block"` rule. |
| node-kind-string-to-enum | `445309d5d` pair-table ctor `"symbol"`/`"keyword"` + `atoms.wat` first defrule rete `string::=` list/vector/map/set. |
| fmt-head-fqdn-to-clojure | `0b5742cc7` `wat-scripts/fmt/rules/if.wat` (`":wat::core::if"` → `"wat.core/if"`). |
| match-arm-to-bracket-map-pattern | `480f38d05` `wat-scripts/fmt/fixtures/half-broken.wat` (`(n n)`/`(_ 0)` → `[n n]`/`[_ 0]`). Near-miss `cond` + already-Vector arm are **spec** (header: cond is not touched; a Vector arm is already migrated). |
| assertion-failed-to-kwargs | `de2d9092a` `probe_arc259_deftest_prime_failing.wat`. |
| positional-ctor-to-map | `434d7fac6` `tests/types/probe_arc296_h2__unit.wat` (`:usr::Shape::Dot` → `:usr::Shape::Dot {}`). |
| bare-variant-to-qualified | **spec** (header: the five keyword leaves). Landing `49f03f179` on `probe_arc241_stone17_defmacro_canonical_c01.wat` mixed H-2 kwargs `{:value ~x}`; today's tool emits qualify-only. |
| variant-separator-to-dot | `085f3ed3e` `pair-table.wat` (`NodeKind::Symbol` → `NodeKind.Symbol`). |
| variant-vector-to-tagged-map | **no fixture** — Finding 1. Header spec is identity. |

## B1.6 — documented case → fixture line

Line numbers in `wat-scripts/fixes/replay/<stem>/before.pre`.

| stem | case | line |
|---|---|---|
| edits-carry-the-old-text | RULE 1 edit-tuple type annot | 4 `:i64 :i64 :String` |
| one-param-spec | BARE keyword Vector | 4 `(:wat::core::Vector :wat::core::i64 5)` |
| mandatory-typed-quasiquote-residual | Vector ~ty | 5 `` `(:wat::core::Vector ~ty) `` |
| rename-sort-prime-to-native | sort' in sort 1-ary | 5 |
| rename-sort-prime-to-native | sort' in sort 2-ary | 13 |
| rename-sort-prime-to-native | sort' in sort-by 2-ary | 20 |
| rename-sort-prime-to-native | sort' in sort-by 3-ary | 29 |
| bare-symbol-shorthand-to-fqdn | Ok | 26 `((Ok _) nil)` |
| bare-symbol-shorthand-to-fqdn | Err | 27 `((Err _) nil)` |
| rename-slipped-core-heads-to-their-homes | core::println → kernel::println | 3 |
| rename-slipped-core-heads-to-their-homes | core::edn::write → edn::write | 5 |
| repoint-retired-heads-to-live-spellings | process/grants → process | 6 |
| bare-none-keyword-to-fqdn | :None → :wat::core::None | 4 |
| break-kind-string-to-enum | "align" → BreakKind::Align | 29 |
| break-kind-string-to-enum | "block" → BreakKind::Block | 42 |
| node-kind-string-to-enum | ctor :kind "symbol" | 3 |
| node-kind-string-to-enum | ctor :kind "keyword" | 4 |
| node-kind-string-to-enum | string::= "list" | 16 |
| node-kind-string-to-enum | string::= "vector" | 20 |
| node-kind-string-to-enum | string::= "map" | 24 |
| node-kind-string-to-enum | string::= "set" | 28 |
| fmt-head-fqdn-to-clojure | ":wat::core::if" | 14, 23 |
| match-arm-to-bracket-map-pattern | (n n) / (_ 0) delimiter flip | 10 |
| assertion-failed-to-kwargs | 3-arg two-Nones dropped | 5 |
| positional-ctor-to-map | unit ctor → `{}` | 6 `:usr::Shape::Dot` |
| bare-variant-to-qualified | :wat::core::Some | 3 |
| bare-variant-to-qualified | :wat::core::None | 5 |
| bare-variant-to-qualified | :wat::core::Ok | 7 |
| bare-variant-to-qualified | :wat::core::Err | 9 |
| bare-variant-to-qualified | :None | 11 |
| variant-separator-to-dot | NodeKind::Symbol | 3 |
| variant-separator-to-dot | NodeKind::Keyword | 4 |

### Near-misses (byte-identical)

| stem | near-miss |
|---|---|
| edits-carry-the-old-text | line 1–3: a Tuple that is NOT the edit-tuple type (`i64 String`) stays |
| one-param-spec | `>` comparison and the `defn` stay; unmarked-bracket / HashMap **not in this fixture** |
| mandatory-typed-quasiquote-residual | line 2–3: PersistentVector ~ty stays (not a mandatory-typed ctor) |
| rename-sort-prime-to-native | public `sort` / `sort-by` defclause names stay (header: they do not move); nth-spec comment block stays (comments are not nodes) |
| bare-symbol-shorthand-to-fqdn | `(:wat::kernel::println "wat-atoms")` stays; `Some` **not in landing corpus** (where-control.wat site is a comment; header lists it) |
| rename-slipped-core-heads-to-their-homes | line 1–2: already-home `(:wat::kernel::println "already home")` stays |
| repoint-retired-heads-to-live-spellings | line 7: `(:wat::spawn::thread)` stays. **reduce-walk → foldl-spec-walk** and **tuple-get t 0 → first t** are documented and in the landing commit, **not in this fixture** |
| bare-none-keyword-to-fqdn | line 1 comment `:None` stays; line 5 already-qualified `:wat::core::None` stays |
| break-kind-string-to-enum | `"vector"` Node.kind string compares stay (header: Node.kind string comparisons are untouched) |
| node-kind-string-to-enum | Named string `":wat::core::defenum"` stays (line 33). **string::not=** (header rewrite 2) **not in this fixture** |
| fmt-head-fqdn-to-clojure | comments naming `if` stay (comments are not string nodes) |
| match-arm-to-bracket-map-pattern | line 3 `cond` stays; line 7 already-Vector arm stays. Field-bearing variant arms **not in this fixture** |
| assertion-failed-to-kwargs | deftest name / comment stay. **actual/expected 3-arg** (header's second case) **not in this fixture** |
| positional-ctor-to-map | defenum `:Circle` / `:Dot []` stay. Field-bearing ctor `(:ns::E::V a b)` **not in this fixture** |
| bare-variant-to-qualified | line 13 already-qualified `:wat::core::Option::Some` stays |
| variant-separator-to-dot | `:wat::core::defn` / `NodeKind` type-path `::` (not a census pair) stays |

## Gate timeout (not a red)

First `cargo nextest run --release --test cli -E 'test(every_recorded_migration_replays)'` without the override: shard_3 **TIMEOUT at 30.004s**. Default profile `slow-timeout = { period = "15s", terminate-after = 2 }` kills at 30s. Shard 3 (sorted stem index `% 8 == 3`) holds `positional-ctor-to-map` (BRIEF trap: ~10 s × 2) plus `bare-variant-to-qualified`, `rename-keyword-to-its-home`, `rewriting-rules-join-written`. Did not re-run that tree as a floor; added the override (same envelope as `retirement_table_is_fully_reachable`); gate then 9/9 in 30.771s. Under the full floor, shard_3 was 78.908s — inside 120s kill, dead at 30s.

`.config/nextest.toml` is outside the brief's listed blast. Without it B1.7/B1.8 cannot pass. Documented here so the orchestrator can keep or drop it.

## STOPs

| STOP | hit? |
|---|---|
| STOP-1 repair needs src/ / changes documented behaviour / missing primitive | **no repair.** Finding 1 is the identity rewrite left ledgered, not a repair. |
| STOP-2 after.post cannot come from history or spec | no — every fixture cites history or spec above |
| STOP-3 input only exists as a file type the temp `<stem>.wat` cannot carry | no |
| STOP-4 SCOPE cannot be determined | no |
| STOP-5 floor red outside this stone's files | no — floor 5382/5382 |

## Not done (by brief)

- push
- batch 2 (other 66, repairs of `to-faithful-clojure-net`/`-rete`, exemptions, ledger deleted, arms e–h)
- `src/`
- a non-vacuous fixture for `variant-vector-to-tagged-map`
