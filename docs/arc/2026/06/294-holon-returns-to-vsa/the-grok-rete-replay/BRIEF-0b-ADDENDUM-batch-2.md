# ADDENDUM to BRIEF 0b — batch 2 begins here

Batch 1 is VERIFIED and pushed (`8b2312dfb`). The orchestrator re-ran:
- provenance: 12/12 history-cited fixtures match every `.wat` their commit changed, with 0 misses;
- SCOPE: 30/30 stems have exactly one line, and the tooling globs hit;
- the gate: 9/9, 29.3 s run alone;
- the floor: 5382/5382; clippy 0.

`.config/nextest.toml`'s override STAYS. The orchestrator keeps it.

Do these in this order, before the rest of batch 2 as briefed in
`BRIEF-0b-every-recorded-migration-replays.md`. Commit on green as you go. **Do not push.** Yield
once, at the end of batch 2.

## 1. Close batch 1's coverage gaps FIRST — the replay leans hardest on exactly these

The brief required every documented rewrite case to be exercised. `SCORE-0b-batch-1.md` names these
as "not in this fixture". Add each one to the stem's existing fixture: the same oracle rule (history
or spec, never the tool), plus a near-miss.

| stem | missing case (the codemod's own header) |
|---|---|
| `positional-ctor-to-map` | a field-bearing ctor `(:ns::E::V a b)` → `(:ns::E::V {:f1 a :f2 b})`. This is its MAIN case; the fixture has only the unit ctor. |
| `match-arm-to-bracket-map-pattern` | a field-bearing variant arm `((:Enum::Variant a b) body)` → `[:Enum::Variant {:a a :b b} body]`, and a unit arm `(:Enum::Unit body)` → `[:Enum::Unit {} body]` |
| `assertion-failed-to-kwargs` | the 3-arg form with a real actual/expected → `:message … :actual … :expected …` |
| `one-param-spec` | the unmarked bracket `(:wat::core::Vector [:wat::core::i64] …)`, and the BARE-keyword `HashMap` `:k :v` |
| `repoint-retired-heads-to-live-spellings` | the reduce-walk → foldl-spec-walk repoint, and `tuple-get t 0` → `first t` (both are in its landing commit) |
| `node-kind-string-to-enum` | header rewrite 2: `string::not=` on a `Node :kind` binder |
| `bare-symbol-shorthand-to-fqdn` | `Some` (from spec if no corpus history, and say so) |

## 2. RULED by the builder, 2026-09-12: delete `variant-vector-to-tagged-map`

- Its `migrate` returns `src`, and its own header says *"it currently identity-rewrites."* A
  migration that cannot change a byte is not a migration.
- `git rm wat-scripts/fixes/variant-vector-to-tagged-map.wat`, and remove its `FROZEN_LEDGER` entry.
- The stem count becomes **95**.
- Arc 296's SCOREs that mention it are inscribed history. Leave them untouched.
- The commit message cites the header line and this ruling.

## 3. `tuple-parens-to-binder.wat:27` — a comment collides with the SCOPE grammar

`;; SCOPE: only `:(` keyword bodies (colon DIRECTLY followed by an open …` is prose, but the grammar
reads it as 14 globs, all matching nothing. Reword that comment so it no longer begins `;; SCOPE: `.
It is a comment-only edit. The file must then carry exactly ONE real `;; SCOPE:` declaration.
Measured: it is the only codemod with this collision.

## 4. Shard sizing — no shard near its kill under floor load

Shard 3 (the one holding `positional-ctor-to-map`) took 29.3 s run alone and 78.9 s under the floor,
against a 120 s kill. Batch 2 adds 66 fixtures.
- Size the shards so that **no shard exceeds 60 s under the full floor**: more shards, or the heavy
  stem isolated.
- Report the per-shard wall time from the final floor run in the SCORE.

## Added EXPECTATIONS rows (batch 2)

| # | what | expected |
|---|---|---|
| A1 | the 7 gaps closed | each case above exercised by a named fixture line, cited to history or spec, with a near-miss |
| A2 | the ruling applied | the file is gone; its ledger entry is gone; the gate's stems == **95** |
| A3 | the SCOPE collision fixed | `grep -c '^;; SCOPE: ' wat-scripts/fixes/tuple-parens-to-binder.wat` == 1, and every entry on it hits a tracked file |
| A4 | shard headroom | the per-shard table from the final floor; max ≤ 60 s |
| B2.2′ | replaces B2.2 | fixtures + runes == **95**; no stem in both |
