# SCORE — STONE 255.4: ONE member join. `Type/member`, always.

Branch: `main`. **Committed, not pushed.** Drawn against `db40a06db` (draw `8470dfd25`).
Parent: `BRIEF-STONE-255.4-one-member-join.md`.
Floor / workspace clippy / `census.sh --diff`: orchestrator's row (not run).
Crate clippy (`-p wat` and `-p wat-macros`) + lint suite run here.
Position grammar **not started.** 8d-ii **not started.**

## Gate: the 179-file delta — ONE tree, before and after

Same spread as 255.1–255.3: every 12th path of the 8d-i census `files.txt` (2145/12 = **179**).
Originals: **live tree**. Converted copies: `/tmp/8d-i/census/tree-after-1`.
Timed one file first: `wat/cache.wat` **0.188 s**.

The brief's baseline **80** is the orchestrator's 255.3 WEIGH tree. This stone's before is
the **same 179 on this tree with the 255.3 binary** (84), so the comparison is like-for-like.

| | this tree before (255.3) | this stone |
|---|---|---|
| originals clean | **161 / 179** | **161 / 179** |
| of those, still clean after conversion | 77 | **80** |
| ⛔ **REGRESSIONS** | **84** | **81** |

**3 closed. 0 newly_broken. Net −3.**

The 2 `Bytes/to-hex` regressions **CLOSED** (orig and converted both rc=0):

- `tests/reflection/probe_arc255_ivc_metadata_plain_values.wat`
- `wat-scripts/scratch-pad/probe-slice-one-registry-seam.wat`

Third close: `wat-tests/cache/HolographicLru.wat` — converted `wat.cache.HolographicLru/put`
now reconstructs to the registered slash form.

### Classification (81)

| cause | n | vs this tree at 255.3 (84) |
|---|---|---|
| `UnresolvedReference` | **48** | 51 → 48 (the three closes were UR) |
| `ReteCheckErrors` | 16 | same |
| remaining (`MalformedDecl` / `CheckErrors` / OTHER / `UnknownNamedType`) | 17 | the three UR left; other-kind tagging on the rest is unchanged in substance |

## Derived set — not the orchestrator's grep

The brief carried three mutually inconsistent greps (83 / 11 / 63). The live
registries, asked:

**wat_intrinsic (5)** — the scratch-pad's Type::method five:
`:wat::core::Bytes/{to-hex,from-hex}`, `:wat::kernel::HandlePool/{new,pop,finish}`.
The call-head grep saw 1 `Bytes::to-hex` and 0 HandlePool. `from-hex` and the
HandlePool family are the tenth correction.

**wat_dispatch (one door)** — `crates/wat-macros/src/codegen.rs` `method_wat_path`
was `format!("{}::{}", path, method)`. Now `"/"`. That unifies
`:rust::cache::Lru/{new,put,get,len,is_empty}` and
`:rust::sqlite::{Connection,ReadConnection}/…` (and the test shims) at once.

**wat-side defn (8)** — `:wat::cache::{Lru,HolographicLru}/{new,put,get,len}` in
`wat/cache.wat`.

Do **not** rename `Record::def` — already retired to `defrecord` (293.2).

## What landed

- Registrations renamed `Type::member` → `Type/member`.
- A retirement-ledger row for each production old spelling (option::expect's shape).
  An old spelling teaches its replacement.
- Corpus moved by recorded codemod `wat-scripts/fixes/type-member-colon-to-slash.wat`
  (`rename-keyword-exact`, not prefix: `:Type::` is not a token boundary inside
  `:Type::member`). Dry-run on `/tmp` copies, diff, idempotent 0. Applied to the
  derived path list.
- Impasse at `types.rs` deleted. `reconstruct_call_path` still asks the registry;
  the second join is gone, so the answer is now sufficient.
- Wall: `types::stone_255_4_no_colon_joined_type_member_in_the_registry` (live
  registries) + `tests/lint/one_member_join.rs` (tracked `.wat` **call heads**).

### `.wat` files that moved (21)

Brief guessed an 11-file set. Derived: **21**. Difference: wat_dispatch test
fixtures, reflection probes (value-position `metadata-of` of Bytes), sqlite +
cache stdlib, Reject.wat helpers the call-head wall found, one golden string in
`wat-tests/reflect/reflection-surface.wat` (`"Bytes::to-hex"` → `"Bytes/to-hex"`
in render-doc output — a string, not a keyword, so the AST tool cannot see it).

Two grep-list files were **unchanged**: a `keyword-node` string
`":wat::core::Bytes::to-hex"` and the measurement probe's `:my::ns::Bytes::to-hex`.
Neither is a call head.

## What a door cannot express

- **Nested type names stay `::`.** `:wat::cache::Cache::GetRequest` is a type, not
  a method. Last-segment PascalCase vs lowercase is the discriminator the wall
  uses; identity reconstruction stays `::`.
- **Comments.** `rename-keyword-exact` is comment-faithful. `wat/cache.wat` prose
  still says `Lru::new`. The tool cannot rewrite `;;` text.
- **String literals.** `keyword-node ":wat::core::Bytes::to-hex"` is data about a
  spelling, not a call.
- **deftest names** (`:wat-tests::cache::HolographicLru::test-…`) are identity,
  not members — they sit in name position, which is 255.5.
- **Position grammar** (`:wat::WatAST`, declaration names) — out of scope, as drawn.

## Test count

Predicted **+2** (registry wall + call-head wall). `cargo nextest list --release -p wat`: **5317** (was 5315).

## Walls I ran

- `cargo clippy --release --all-targets -p wat --offline -- -D warnings` — **0**
- `cargo clippy --release --all-targets -p wat-macros --offline -- -D warnings` — **0**
- `one_variant_separator` / `one_param_spec` / `no_loose_string_assert` / `one_member_join` — pass
- `stone_255_3_registry_decides_the_join` / `stone_255_4_no_colon_joined_type_member_in_the_registry` — pass

Floor + workspace clippy + census: **not run**. Do not push. Do not start 8d-ii.
Do not start the position grammar.
