# BRIEF — STONE 255.91: the name is the pair

**Drawn 2026-10-04 against `main` @ `61d54d1bf`.** **Executor: grok via pulsare, working solo** (it runs the floor).
Commit locally on `main` (`git add -- <paths>`, never `-A`; `git status` clean before the floor you report);
**do not push**.

## Where this sits

The builder's ruling (2026-10-04, recorded in `BRIEF-STONE-255.90-…`): a symbol **is** `{:namespace <base symbol>
:name <base symbol> :scope <hygiene scopes>}`; no string is an identity; a string exists only from
`"{namespace}/{name}"`; `flat` is deleted; `::` is removed from the internals too. 255.89 (the corpus conversion) is
parked on `origin/cutover-5c-iii`; `main` is green ground (the last full floor, `.floor/2026-10-03T13-09-11Z` at
`30cd8b69c`: 6412 passed / 24 skipped; `main` since then is docs plus the census bin). 255.90 measured the distance
(`SCORE-` and `WEIGH-STONE-255.90-…`). This is the first of the `Name` stones:
**(1) the type and pair equality (this)**, (2) the registries keyed by `Name`, (3) the `REG`/`DISPATCH` literals by a
recorded Rust-literal codemod, (4) `CMP`/`BUILD`, (5) `flat` deleted, then 5c-iii re-run on top.

## The contract (pinned)

- **`Name { namespace: Arc<str>, name: Arc<str> }`** in `crates/wat-reader` (beside `Identifier`). `Eq`/`Hash` over
  the two fields' **contents** (a pointer-equality fast path is allowed). **No global interner:** `docs/ZERO-MUTEX.md`
  refuses a lock-guarded table. `Display` is the one stringification: `"{namespace}/{name}"`, or the bare `name` when the
  namespace is `$bound`.
- **`Identifier` = a `Name` + its scope set.** `PartialEq`/`Hash` (`identifier.rs:128`, `:135`) become
  `(namespace, name, scopes)`. `flat` stays **as a print cache only** this stone (stone 5 deletes it); nothing decides by
  it. `bare` keeps the first-slash split; `into_bound` keeps `{$bound, <whole spelling>}`.
- **The transition translation, in one place:** `Name::from_keyword(&str) -> Option<Name>` turns a keyword spelling of a
  name into **the pair its symbol image has**. Its definition is **the recorded converter's**: for any keyword `k` that
  `wat-scripts/fixes/to-faithful-clojure.wat` rewrites to the symbol `s`, `Name::from_keyword(k)` equals
  `Identifier::bare(s)`'s pair. A keyword that is not a name (a value keyword such as `:k`, `:else`) is `None`. This
  function is deleted at the wall, with keywords.

## The work

1. **The agreement, measured before the code is trusted:** `origin/cutover-5c-iii` holds the converted corpus
   (`d45406b02`) beside its pre-image (`9d85da5a5`). Zip, per file, every keyword token the converter rewrote with the
   symbol it became (255.84 §6 zipped call heads the same way), across **all** converted files, heads and non-head
   names. Report the count of pairs and of **disagreements** between `Name::from_keyword(k)` and `bare(s)`'s pair; every
   disagreement is listed. The target is 0. A disagreement where the converter's choice looks wrong is a STOP-2 row, not
   a reason to bend the function. Commit the zip as a test fixture (a sample of a few hundred pairs, every distinct
   shape: `::` path, `/` member join, `::` type twin, `wat.type/` members, a trailing marker), and the driven test that
   holds them.
2. **The type and equality** as pinned. Then the sites 255.90 §4 names: **`substitute`** (`src/runtime.rs:14217`) today
   matches a binder against its body reference because both share `flat`; under pair equality a slashed local (`foo/bar`
   bound, `{$bound, foo/bar}`; read in the body, `{foo, bar}`) no longer compares equal. The builder's slash rule
   (255.88) is *a body reference resolves locally first*: say how `substitute`, `env_key` (`src/scope/resolution.rs:81`)
   and local resolution find a binder for a slashed reference, and make that one function, not a re-split.
   `ast_same_identity` (`src/macros/registry.rs:192-202`): its keyword/symbol cross arm compares through
   `Name::from_keyword` and the identifier's `Name`, not through `canonical_identity` strings (this is the
   translation's first consumer).
3. **The census moves to `examples/name_census.rs`**, and `syn`/`proc-macro2` return to `[dev-dependencies]` with the
   comment they had (255.90's WEIGH, correction 1). It still runs (`cargo run --release --example name_census`).

## Gates

| what | how | expected |
|---|---|---|
| the agreement | item 1 over every converted file | disagreements 0, or each a STOP-2 row |
| the equality probe | a test: `bare("a.b/c")` == `Name::from_keyword(":a::b::c")`'s and `":a::b/c"`'s pair; a scoped copy is unequal; a binder and a slashed body reference resolve to each other through the item-2 function | green |
| census still runs | the example | rc 0, same section totals as `name-census.tsv` within what this stone moved |
| release floor | `scripts/floor.sh`, foreground, nothing else running, `git status` clean | all passed. **The test-name set against `.floor/2026-10-03T13-09-11Z`: MISSING 0** |
| cost | the fuzz deftest's time on your floor | against 71.2 s at `.floor/2026-10-03T13-09-11Z`; report it, do not raise any limit |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| ignores | the SEAM's ledger command | 18 |

## Reds and STOPs

- A red caused by this stone's own change is captured **verbatim**, cured, and followed by a **new** floor. Any other
  red is a STOP. Never re-run unchanged code for a green.
- **STOP-1:** pair equality changes what an existing program binds or resolves (a site that relied on two spellings of
  one name being equal, or two names with one spelling being unequal). Quote the program and STOP.
- **STOP-2:** the agreement finds a keyword whose converted symbol is a different name than the keyword meant. List it
  and STOP on it; finish the rest.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you. No string compare stands in for identity; no name is built by `format!` except
`Name`'s `Display`. No time limit is raised. Capture `rc=$?` on the next statement. Never wait with `pgrep -f`. Never
write a number, file:line or example you did not measure. If this brief contradicts the code, the code wins: say so.
Write `SCORE-STONE-255.91-the-name-is-the-pair.md` beside this brief, commit it, **do not push**.
