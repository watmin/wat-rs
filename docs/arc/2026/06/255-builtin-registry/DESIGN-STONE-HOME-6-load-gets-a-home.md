# DESIGN — HOME #6: load gets a home

> **Builder, 2026-08-25:** *"load and resolve are distinct - draw src/load/ and release it"* —
> ruling the naming question I put to them: `src/resolve/` already exists and is heavily warded, and
> "load" and "resolve" are adjacent words. They are distinct domains: `resolve/` is name-and-scope
> resolution; `load/` is **getting source text into the runtime**.

## THE FAMILY

```
src/load.rs      1894   "Recursive `load!` resolution with `:wat::verify::*` interface keywords"
src/stdlib.rs     884   "Bundled wat stdlib — baked into the binary via include_str!"
src/source.rs      76   "the user-facing surface for wat-source"
src/sandbox.rs     13   ⛔ ZERO items — see below
                 ─────
                 2867   4 files · no `src/load/`
```

Chosen over two measured alternatives, on **cohesion**, not on names:

```
LOADING   load<->config, source<->stdlib   TWO MUTUAL PAIRS
TESTING   test_runner->panic_hook->assertion, harness isolated   a one-way chain
HOLON     hologram->vm_registry only; sigma and lower isolated   a single edge
```

## ⛔ `src/sandbox.rs` HAS NO CODE IN IT

`grep -cvE '^\s*(//|$)' src/sandbox.rs` → **0**. Thirteen lines, every one a `//!` comment. Its own
header:

> *"`resolve_sandbox_loader` (and the corresponding `src/spawn.rs` callers) were retired in arc 298.
> **The module remains as a namespace anchor.**"*

A namespace anchor anchoring nothing. Its **only** reference in the whole tree is the
`pub mod sandbox;` in `lib.rs` that keeps it alive.

★ **It is DELETED, not moved.** Moving an empty anchor into a new home is minting exactly the thing
the four-homes stone was named for — a home nothing has earned. Its header is real history and goes
into `src/load/mod.rs`'s own doc, where a reader looking for the sandbox loader will find it.

## ⚠ `config.rs` IS DELIBERATELY LEFT OUT, AND THE REASON IS MEASURED

`config.rs` is the family's largest neighbour and its most-referenced (**212 in `src/`, 8 outside,
220 total** — more than the rest of the family combined). It is also the one member that is not one
thing:

```
collect_entry_file · collect_entry_file_with_inherit     LOADING
Config · CapacityMode · ConfigError · ConfigErrorKind    runtime configuration
DEFAULT_CAPACITY_MODE · DEFAULT_DIM_COUNT                ← a VSA DIMENSION COUNT
```

`DEFAULT_DIM_COUNT` has nothing to do with getting source into the runtime. Folding `config` in
would drag 220 references for a file that is half out of domain, and would bake a braid into a new
home on its first day. **It is `solvere`'s question — is `config.rs` two modules wearing one name? —
and it is asked separately, on its own evidence.**

## THE FORM — thin `mod.rs` plus named siblings, per the house

Measured across the existing homes: `edn/` 49, `value/` 51, `kernel/` 66, `rete/` 83,
`collection/` 130. (`resolve/` at 503 is the outlier, not the rule.) And `src/value/value.rs`
already establishes that a sibling named for what it holds is fine even when it echoes its home.

```
src/load/mod.rs                  the home (thin)
src/load/loader.rs  <- load.rs   the load! forms, the LoadSpec family, SourceLoader/FsLoader/InMemoryLoader, the Load*Errors
src/load/stdlib.rs  <- stdlib.rs
src/load/source.rs  <- source.rs
src/sandbox.rs                   DELETED
```

```
crate::load::X    ->  crate::load::loader::X
crate::stdlib::X  ->  crate::load::stdlib::X
crate::source::X  ->  crate::load::source::X
wat::load::X      ->  wat::load::loader::X        (and so on)
```

⛔ **NO re-exports in `src/load/mod.rs`**, same ruling as HOME #5 and for the same reason:
`wat::load::InMemoryLoader` is the shortest path and therefore the tempting one, and it would mint a
second way to say one item. One way.

## ⚠ FILES MOVE WHOLE — no splitting, and that is a consistency decision

`load.rs` carries `LoadFetchError` / `LoadError` / `LoadErrorKind` with their `ToEdn` impls, and
**every sibling home carves an `error.rs`** (`check/`, `types/`, `resolve/`, `edn/`). So the obvious
next move is a `src/load/error.rs`.

It is **out of scope here**, because HOME #5 cut splitting `render.rs`'s 5,016 lines on exactly this
ground — the home comes first, the carve inside it is separable, and a stone that does both is two
stones wearing one name. Named here so it is a work item rather than a mental note.

## THE CASCADE — measured

```
              src/   non-src   TOTAL
load            42        15      57
stdlib          17         1      18
source           9         4      13
sandbox          0         0       0   (deleted)
                              ──────
                                  88
```

Small — a fifth of HOME #5's 349. Compiler-named throughout.

⚠ **`tests/` is a separate compilation unit**, the trap HOME #5 named and did not spring: 20 of the
88 live outside `src/`. `cargo build --release` alone cannot see them; `--all-targets` can.

## THE FOUR QUESTIONS

- **Obvious?** YES — four files about getting source into the runtime, in a tree where every other
  domain has a named directory.
- **Simple?** YES — a move, a path rename, and one deletion. No logic changes.
- **Honest?** YES — and the deletion is the honest half: a module kept alive by nothing but its own
  `pub mod` line is a claim that something is there.
- **Good UX?** YES — `crate::load::stdlib` says where the bundled sources live; `crate::stdlib` says
  a file happened to be called that.

## ACCEPTANCE

1. **`src/` root 31 → 27 `.rs` files** (three moved, one deleted). Derived: 31 at HEAD.
2. **`src/load/` holds four** (`mod.rs`, `loader.rs`, `stdlib.rs`, `source.rs`).
3. **`src/sandbox.rs` is gone and `lib.rs` no longer declares it.** Zero references remain — there
   were zero to begin with, outside that one `pub mod`.
4. **Zero `crate::load::` / `crate::stdlib::` / `crate::source::` / `wat::…` paths of the old shape**,
   internally or in `tests/`. Derived: 88 at HEAD.
5. **No `pub use` in `src/load/mod.rs`** that creates a second path to any item.
6. **Zero behaviour change** — no `.wat` corpus edit, no test logic touched, only paths.
7. Floor green **accounted BY NAME** (baseline 5057/5057, 19 skipped); clippy 0.

## OUT OF SCOPE — affirmatively cut

- **`config.rs`** — measured above; `solvere`'s question, its own stone.
- **A `src/load/error.rs` carve** — every sibling home has one and this will want one; the home
  comes first.
- **`src/check.rs` (22,418) / `types.rs` (7,228) / `freeze.rs` (2,622)** — legitimate mod roots whose
  directories hold 2–3 small files. **A carve begun and abandoned**, a different defect.
- **The re-export shims** — `lexer.rs` (3 lines), `ast.rs` (3), `span.rs` (23), `parser.rs` (59) are
  `pub use wat_reader::…`, the trailing edge of a finished crate extraction. Third class.
