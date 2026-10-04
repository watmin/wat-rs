# BRIEF — STONE 255.93: a name in Rust is a name

**Drawn 2026-10-04 against `main` @ `5a5744376`.** **Executor: grok via pulsare, working solo** (it runs the floor).
Commit locally on `main` (`git add -- <paths>`, never `-A`; `git status` clean before the floor you report);
**do not push**.

## Where this sits

The builder's ruling (255.90's brief): identity is `{namespace, name, scope}`; no string is an identity; `::` dies in
the internals too. 255.91 built `Name`; 255.92 keyed every registry by it, with a **text bridge** for callers that still
pass strings (`get(&str)`, the spelling index `src/name_map.rs:100`, the enter cache `:79`; `WEIGH-STONE-255.92-…`).
This is stone 3 of 5: **the `REG` and `DISPATCH` literals become names.** Stone 4 (`CMP`/`BUILD`) converts the last
text callers, and **its** gate deletes the bridge (corrected from 255.92's WEIGH, which said stone 3: the bridge dies
with its last text caller, and some are comparisons). Stone 5 deletes `flat`.

## The form (builder approved, 2026-10-04)

- **A registration** is written in symbol spelling and becomes a `Name` **at compile time**:
  `#[wat_intrinsic("wat.i64/to-string")]`. The attribute macro (`crates/wat-macros/src/wat_intrinsic.rs:74-80`, the
  `fqdn: LitStr`) splits at the first `/` and **refuses `::` with a compile error**: a wall at construction.
- **A name constant in Rust** is a compile-time pair, never parsed at run time:
  `const SOCKET_ADDRESS_WIRE: … = name!("wat.kernel", "SocketAddressWire")` (or the equivalent your design needs). It
  compares with a runtime `Name` by content. `Name`'s fields are `Arc<str>` today (`crates/wat-reader/src/identifier.rs:78`)
  and an `Arc` is not `const`; choose the representation (a `&'static str` case beside the shared one, compared by
  content) and say why.
- **A dispatch** matches the pair: `match (n.namespace(), n.name()) { ("wat.core", "defn") => … }`. The scrutinee is a
  `Name` (or an `Identifier`'s pair) at that point. **No arm parses text.**

## What is known (255.90's census, `name-census.tsv`; re-run the example for today's numbers)

- `REG` 1,954 (`src` 1,879): **620 registration attributes**, **1,259 declared names or registry inserts** (e.g.
  `const SOCKET_ADDRESS_WIRE_CLASS: &str = "wat::kernel::SocketAddressWire"` at `src/capability/registry.rs:115`; the
  pair table at `src/check.rs:1115`).
- `DISPATCH` 683 (`src` 663): `src/check.rs` 250, `src/runtime.rs` 98, `src/rete/reachability.rs` 80,
  `src/rete/expr_ir/eval.rs` 70, then `edn/render.rs`, `types.rs`, `rete/clause.rs`. Shapes seen:
  `match head.as_str() { ":wat::core::defn" => … }` (`src/check.rs`), `match key { "wat::type::HashSet" => … }`.

## The work

1. **The tool first, recorded:** a Rust-literal codemod (`syn` + span locations; it lives where `syn` may be linked without the shipped `wat` crate linking it: its own crate under `crates/`, or `examples/` as 255.91 placed the census, never `src/bin/`) that
   rewrites a name literal to its symbol spelling (`":wat::core::defn"` → `wat.core/defn`), and a dispatch arm's
   literal pattern to a pair pattern (`("wat.core", "defn")`). Its translation is `Name::from_keyword` (one
   definition). Dry-run on a copy and `diff`; idempotent; committed with a test that drives it over a fixture holding
   every shape. **The tool never edits a string inside embedded wat** (the census's `WAT` class, 5c-iv's) **or a
   message** (`MSG`).
2. **Registrations:** the 620 attributes through the codemod, and the macro refusing `::`.
3. **Constants and inserts:** the declared names as compile-time pairs; registry inserts take them.
4. **Dispatch:** each `match`/`if` chain on a name. Where the scrutinee is text today, carry the `Name` (or the
   `Identifier`) to it instead; that is real type work at each site, done by hand where the codemod cannot see the
   type, and each site named in the SCORE.
5. **The bridge, measured:** count the remaining callers of `get(&str)` and of the spelling index after this stone,
   by class (`CMP`, `BUILD`, other). That count is stone 4's size.

## Gates

| what | how | expected |
|---|---|---|
| the codemod | its fixture test; a second run | every shape rewritten; second run 0 changes |
| the wall | a `#[wat_intrinsic(":wat::x::y")]` in a compile-fail test (or a `trybuild`-style probe) | refused at compile time |
| census | `cargo run --release --example name_census` | `REG` and `DISPATCH` (outside `WAT`/`MSG`/doc comments) → 0, or each remaining one listed with why |
| release floor | `scripts/floor.sh`, foreground, nothing else running, `git status` clean | all passed. **Test-name set against `.floor/2026-10-04T07-52-21Z`: MISSING 0** |
| cost | the fuzz deftest alone, six runs on `5a5744376` and six on yours | your mean ≤ the before mean |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| ignores | the SEAM's ledger command | 18 |

## Reds and STOPs

- A red caused by this stone's own change is the work: captured **verbatim**, cured, a **new** floor. Any other red is a
  STOP. Never re-run unchanged code for a green.
- **STOP-1:** a literal that names two different things in two places (one spelling, two meanings), or a dispatch whose
  arms depend on spelling (`/` versus `::`) to mean different things. List it and STOP on it.
- **STOP-2:** a literal the census calls `REG`/`DISPATCH` that is not a name (a Rust path such as `std::…`, a file
  path, a label). Leave it, list it.
- **STOP-3:** after the codemod, more than **150** dispatch sites need hand type work. Report the table and STOP before
  the hand work, so it can be split.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you. No string compare stands in for identity; no name is assembled by `format!` except
`Name`'s `Display`. No time limit is raised. A test that pinned a spelling is rewritten to assert the fact, never
deleted quietly. Capture `rc=$?` on the next statement. Never wait with `pgrep -f`. Never write a number, file:line or
example you did not measure. If this brief contradicts the code, the code wins: say so. Write
`SCORE-STONE-255.93-a-name-in-rust-is-a-name.md` beside this brief, commit it, **do not push**.
