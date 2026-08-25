# DESIGN — HOME #5: EDN gets a home

> **Builder, 2026-08-25:** *"draw src/edn/ and release it"* — asked after *"what's our next home to
> move? we unexpectedly found stuff in string that got moved."*

## THE THESIS

EDN is the largest coherent family still lying loose at `src/` root, and it is the only one there
that is a **clean carve** rather than a megafile or a re-export shim.

```
src/edn_shim.rs             5016   ":wat::edn::* — render any wat value as EDN/JSON text"
src/wat_edn_bridge.rs        941   "plain-EDN serializer/deserializer for WatAST"
src/to_edn.rs                389   "the ONE serialization contract for every error/diagnostic type"
src/to_edn_derive_tests.rs   417   behavioural tests for the `#[to_edn(...)]` attribute DSL
src/runtime_error_edn.rs     231   "Errors-as-EDN extension"
                            ─────
                            6994   one domain · five loose root files · NO `src/edn/`
```

Every sibling domain already has one — `src/string/`, `src/rete/`, `src/value/`, `src/types/`,
`src/collection/`, `src/comms/`, `src/kernel/`. EDN does not.

## ⛔ IT CANNOT SKIP TO THE CRATE, AND THAT IS THE POINT OF AN INTERMEDIATE HOME

`crates/wat-edn` and `crates/wat-to-edn-derive` already exist, so the obvious move looks like
*"send these there."* Measured — they name root-crate types:

```
edn_shim.rs        -> value · types · runtime · ast · scope · stream · span · to_edn
wat_edn_bridge.rs  -> ast · scope · span · edn_shim
runtime_error_edn.rs -> runtime · value · span · to_edn
```

A leaf crate cannot name `Value`, `TypeEnv` or `WatAST`. **`src/edn/` is the step that has to come
first**, exactly as `src/string/` did before anything could be cut — the builder's own trajectory:
*"src/ to just hold mod.rs, then crates/wat-* and sane build times."*

## THE FORM — uniform submodule paths, ONE way to say each thing

```
src/edn/mod.rs        the home
src/edn/render.rs   <- edn_shim.rs            ★ "shim" is a name that says TEMPORARY. It renders.
src/edn/bridge.rs   <- wat_edn_bridge.rs      keeps the honest half of its old name
src/edn/contract.rs <- to_edn.rs              its own doc: "the ONE serialization contract"
src/edn/error.rs    <- runtime_error_edn.rs
src/edn/derive_tests.rs <- to_edn_derive_tests.rs   (stays `#[cfg(test)]`)
```

Call sites move uniformly:

```
crate::edn_shim::X          ->  crate::edn::render::X
crate::wat_edn_bridge::X    ->  crate::edn::bridge::X
crate::to_edn::X            ->  crate::edn::contract::X
crate::runtime_error_edn::X ->  crate::edn::error::X
wat::edn_shim::X            ->  wat::edn::render::X          (and so on, in tests/)
```

⛔ **NO re-exports from `mod.rs`.** `crate::edn::ToEdn` alongside `crate::edn::contract::ToEdn`
would be two ways to say one thing, and `ToEdn` is named 51 times — the shorter path is exactly the
tempting one. One way. The extra segment is the price of not minting a synonym.

## ⚠ ONE THING I CALLED A SMELL AND WAS WRONG ABOUT

`src/to_edn_derive_tests.rs` — 417 lines of tests in the source tree — looked like tests in the
wrong place. It is not. It is `#[cfg(test)]`-gated in `lib.rs:117`, and its own header states the
constraint: `#[derive(ToEdn)]` generates `impl crate::to_edn::ToEdn for <T>`, **which only resolves
inside the `wat` crate**, so an integration test cannot host the toy types. It moves with the family
and stays in-crate. Recorded because "tests in src/" is the kind of thing a later reader will try to
"fix".

## THE CASCADE — measured

```
crate::edn_shim              29 files, 112 occurrences
crate::to_edn                27 files, 163 occurrences
crate::wat_edn_bridge         6 files,  14
crate::runtime_error_edn      2 files,   3
                          ────────────────────
                      internal ~292

wat::edn_shim  30 · wat::to_edn  20 · wat::wat_edn_bridge  5 · wat::runtime_error_edn  2
                      external    57   across 37 files under tests/
                          ────────────────────
                      TOTAL      ~349 across ~72 files
```

Every one is compiler-named: `mod` is renamed, `rustc` reports each stale path. **The failures are
the worklist** (`docs/SUBSTRATE-AS-TEACHER.md`), the same shape as arc 300 stone D's 19 → 0.

⚠ `render` and `bridge` are **mutually recursive** (`edn_shim` → `wat_edn_bridge` → `edn_shim`).
Under one parent that is `super::bridge::` / `super::render::` and costs nothing — but it means they
cannot be split across a module boundary that forbids cycles, so they stay siblings.

## THE FOUR QUESTIONS

- **Obvious?** YES — five files whose every doc-line says EDN, in a tree where every other domain
  has a named directory.
- **Simple?** YES — a move and a path rename. No logic changes, no new types, no behaviour.
- **Honest?** YES — `edn_shim` has not been a shim for a long time; 5,016 lines under a name that
  says *temporary* is a claim the file stopped supporting.
- **Good UX?** YES — `crate::edn::render` says where to look; `crate::edn_shim` says a thing was
  patched once.

## ACCEPTANCE

1. **`src/` root loses five files** — 36 → 31 `.rs` files. Derived: 36 at HEAD.
2. **`src/edn/` holds six** (`mod.rs` + the five), and no file at `src/` root names EDN any more.
3. **Zero `crate::edn_shim` / `crate::to_edn` / `crate::wat_edn_bridge` / `crate::runtime_error_edn`
   paths remain**, internally or in `tests/`. Derived: ~349 at HEAD.
4. **No `pub use` re-export in `src/edn/mod.rs`** that creates a second path to any item. One way.
5. **`derive_tests` stays `#[cfg(test)]`** and still compiles in-crate.
6. **Zero behaviour change** — no `.wat` corpus edit, no golden recapture beyond a Rust `:line` pin,
   no test logic touched. Only paths move.
7. Floor green **accounted BY NAME** (baseline 5057/5057, 19 skipped); clippy 0 under `-D warnings`.

## OUT OF SCOPE — affirmatively cut

- **Moving anything into `crates/wat-edn`.** Blocked by root-crate types, above. This stone makes
  that question askable; it does not answer it.
- **Splitting `render.rs` (5,016 lines).** It is one concern at length until measured otherwise —
  `partire`'s question, not this stone's. The home comes first; the carve inside it is separable.
- **`src/check.rs` (22,418) / `types.rs` (7,228) / `freeze.rs` (2,622).** Each is a legitimate mod
  root whose directory holds only 2–3 small files — **a carve begun and abandoned**, which is a
  different defect from a missing home and wants its own stone.
- **The re-export shims** — `lexer.rs` (3 lines), `ast.rs` (3), `span.rs` (23), `parser.rs` (59) are
  `pub use wat_reader::…`, the trailing edge of a crate extraction that already happened. Third
  class, third stone.
