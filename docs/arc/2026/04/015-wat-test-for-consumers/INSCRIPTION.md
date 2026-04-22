# Arc 015 — wat test for consumers — INSCRIPTION

**Status:** shipped 2026-04-21. Four slices plus one cave-quest
(slice 3a).
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**Backlog:** [`BACKLOG.md`](./BACKLOG.md) — the living ledger.
**This file:** completion marker.

---

## Motivation

Arc 013 closed with consumers able to write `wat::main! { ...,
deps: [wat_lru] }` and run their programs end-to-end. Arc 014
shipped scalar conversions to make test assertions honest.
What remained: **consumers couldn't write `.wat` tests that
compose external wat crates.**

The gap was a symmetry miss. Arc 013 built
`Harness::from_source_with_deps` for the `:user::main` path;
arc 013 left the test path using only `wat test <dir>` CLI,
which deliberately doesn't link external crates. Deftests
using `:user::wat::std::lru::LocalCache::*` had no home.

Before holon-lab-trading could adopt wat as its substrate,
consumer wat tests had to work. Arc 015 closes the symmetry.

Builder direction: *"users of wat will be instructed to
create two small rust-typical directories and at least one
small per directory — src/main.rs and tests/tests.rs — and
that's all the rust they'll typically need."*

---

## What shipped

Four slices + one cave quest.

### Slice 1 — `wat::test_runner` library module

Commit `e6b179b`.

Ports test-discovery + freeze + run + cargo-test-style output
from `src/bin/wat.rs`'s private `run_tests_command` into a
library module at `src/test_runner.rs`. Public surface:

```rust
pub fn run_tests_from_dir(
    path: &Path,
    dep_sources: &[&'static [WatSource]],
    dep_registrars: &[DepRegistrar],
) -> TestSummary;

pub fn run_and_assert(
    path: &Path,
    dep_sources: &[&'static [WatSource]],
    dep_registrars: &[DepRegistrar],
);
```

CLI refactored to delegate: `src/bin/wat.rs`'s
`run_tests_command` shrank from ~140 lines to ~15, keeping
only argv parsing + exit-code mapping. Single source of truth.

### Slice 2 — `wat::test_suite!` proc-macro

Commit `51a0176`.

Mirror of `wat::main!` for tests. Named args:

```rust
wat::test_suite! {
    path: "wat-tests",
    deps: [wat_lru],
}
```

Expands to `#[test] fn wat_suite() {
::wat::test_runner::run_and_assert(Path::new(path),
&[deps::wat_sources()...], &[deps::register...]) }`.
Cargo discovers and runs it. On failure, `run_and_assert`'s
panic carries the full failure summary.

### Slice 3a — global install for dep_sources (cave quest)

Commit `211a0d2`.

Cut mid-slice-3 when deftest and `run-hermetic-ast` were
found to bypass dep_sources — their internal `startup_from_forms`
re-freezes without them. Symmetric with arc 013 slice 4a's
discovery of the rust_deps install gap, and a precedent for
future paused-slice splits. Second arc (after 014) cut from a
paused slice; the shape is now settled.

**Substrate fix:** `wat::source::install_dep_sources` —
process-global OnceLock, first-call-wins. Symmetric with
`wat::rust_deps::install`. Every freeze (main, test, sandbox
via `run-sandboxed-ast`, fork child via `run-hermetic-ast`)
transparently sees dep surface through
`stdlib::stdlib_forms()`, which concatenates baked +
installed.

**Retired:** `startup_from_source_with_deps` — no
back-compat shim. Callers install globally then call
`startup_from_source`. `Harness::from_source_with_deps{,_and_loader}`,
`compose_and_run`, `test_runner::run_tests_from_dir` each
install both halves of the contract at entry.

**Dep sources are stdlib-tier now.** Pre-slice-3a they went
through user-tier (reserved-prefix gate active). Post-slice-3a
they flow through the stdlib pipeline (gate bypassed, same as
baked stdlib). Community discipline via `:user::*` convention
+ duplicate-define collision detection carry the protection.
The gate still applies to the USER's own source — which is
what genuinely needed protecting. The slice-2-era test
`harness_dep_declaring_under_wat_namespace_is_rejected` retired.

### Slice 3 — wat-lru self-tests

Commit `211a0d2` (combined with 3a).

- `crates/wat-lru/wat-tests/LocalCache.wat` — four deftests
  for the LocalCache surface (put-then-get, miss-returns-none,
  overwrite, evict-at-capacity). Runs through
  `run-sandboxed-ast` — the cave-quest install lets the inner
  sandbox world see wat-lru's defines.
- `crates/wat-lru/wat-tests/CacheService.wat` — restored from
  pre-4b `wat-tests/std/service/Cache.wat` with paths
  repathed to `:user::wat::std::lru::CacheService::*`. Uses
  `run-hermetic-ast`; fork child COW-inherits the parent test
  binary's DEP_SOURCES OnceLock state, so wat-lru's surface
  reaches the child process.
- `crates/wat-lru/tests/wat_suite.rs` — one-line
  `wat::test_suite! { path: "wat-tests", deps: [wat_lru] }`.
  Syntactically identical to how a downstream consumer
  writes it. **wat-lru is its own first consumer.**

Also renamed files to PascalCase matching the type names:
- `crates/wat-lru/wat/lru.wat` → `LocalCache.wat`
- `crates/wat-lru/wat/service.wat` → `CacheService.wat`
- Test files match.

All five wat-lru `.wat` tests pass. The four Rust-level
Harness integration tests that existed in
`crates/wat-lru/tests/wat_lru_tests.rs` as pre-arc-015
scaffolding (best-available coverage before `wat::test_suite!`
existed) retired — the `.wat` tests now provide the same
coverage through the idiomatic shape that downstream consumers
use.

### Slice 4 — user-facing rename + CONVENTIONS.md + INSCRIPTION

Commit `6688801` (rename) + this commit (CONVENTIONS + INSCRIPTION).

**User-facing rename.** Community wat crate authors don't
think of themselves as shipping "stdlib" — they ship **source**
that composes. The old names leaked wat-rs internal framing:

| Old | New |
|---|---|
| `wat::stdlib::StdlibFile` | `wat::WatSource` (re-exported at root) |
| `wat::stdlib::install_dep_sources` | `wat::source::install_dep_sources` |
| `pub fn stdlib_sources()` (contract) | `pub fn wat_sources()` |

No back-compat shims — pre-publish rename. `src/source.rs` is
the new public module; `src/stdlib.rs` is pub(crate) now
(holds the baked `STDLIB_FILES` array + `stdlib_forms()`
internal composition).

**CONVENTIONS.md** gains three load-bearing additions:
- **Crate folder layouts** — two walkable templates
  (publishable wat crate, consumer binary). References
  `crates/wat-lru/` and `examples/with-lru/`.
- **Three varieties of wat crate** — wrapper / rust-surface /
  pure-wat. Each satisfies the same `wat_sources()` +
  `register()` contract with one half possibly trivial.
- **Install-once discipline** — OnceLock semantics for both
  halves of the contract. One test binary = one consistent
  dep set.

---

## Resolved design decisions

- **2026-04-21** — **No new build tool.** Cargo is the
  authority. `wat::test_suite!` emits a `#[test] fn` Cargo
  already knows how to find and run.
- **2026-04-21** — **Macro sugar + library substrate.**
  `wat::test_runner` is the callable library; `wat::test_suite!`
  is sugar. Mirrors `wat::compose_and_run` + `wat::main!`.
- **2026-04-21** — **Three varieties of wat crate, one
  contract.** Wrapper / rust-surface / pure-wat all satisfy
  `wat_sources()` + `register()`.
- **2026-04-21** — **wat-lru is its own first consumer.**
  `deps: [wat_lru]` in wat-lru's own `tests/wat_suite.rs` is
  syntactically identical to how a downstream consumer writes
  it. Strongest proof available.
- **2026-04-21 (3a)** — **Global install-once for dep_sources.**
  OnceLock, symmetric with `rust_deps::install`. Every freeze
  transparently inherits.
- **2026-04-21 (3a)** — **startup_from_source_with_deps
  retired.** No back-compat shim. Callers install then call
  `startup_from_source`.
- **2026-04-21 (3a)** — **Dep sources are stdlib-tier, not
  user-tier.** Reserved-prefix gate no longer applies; community
  discipline via `:user::*` convention + duplicate-define
  collision detection carry the protection.
- **2026-04-21 (slice 4)** — **StdlibFile → WatSource,
  stdlib_sources → wat_sources.** User-facing rename, no
  back-compat.

---

## What this arc does NOT ship

- A new build tool / parallel test orchestrator. Cargo is the
  authority.
- Per-test parallelism beyond Cargo's outer `#[test]`.
- New assertion primitives — arc 007's `deftest` + `assert-eq` /
  `assert-contains` / `assert-stdout-is` stay the vocabulary.
- A way to run wat tests across multiple dep sets in one
  process — OnceLock first-call-wins applies. Different dep
  sets = different `tests/*.rs` files = different test
  binaries.
- Retirement of the `wat test` CLI subcommand — keeps its
  niche for pure-baked-stdlib wat programs.

---

## Open items cleared

- **Sub-fog 3a (hermetic-ast fork inheritance)** — resolved.
  Fork child COW-inherits the parent's DEP_SOURCES OnceLock
  state; `stdlib_forms()` in the child sees installed deps.
  Verified by wat-lru's CacheService test passing.
- **Nested Cargo-wat test output shape** — cargo's `#[test] fn
  wat_suite` wraps the library's per-test lines; stdout shows
  both layers. Reads cleanly with `--nocapture`; silent on
  success.
- **Explicit type parameters in call heads** — observed during
  slice 3 debug: `(:user::wat::std::lru::CacheService<String,i64>
  16 1)` fails with UnknownFunction. The inference-style
  call `(:user::wat::std::lru::CacheService 16 1)` works.
  Not in arc 015's scope to fix; a future substrate
  consideration if the pattern becomes common.

---

## Why this matters

The consumer story is now complete:

```
my-app/
├── Cargo.toml           # [dependencies] wat + wat-lru + whatever
├── src/
│   ├── main.rs          # wat::main! { source: ..., deps: [wat_lru] }
│   └── program.wat
├── wat-tests/
│   └── *.wat            # deftests using :user::wat::std::lru::*
└── tests/
    └── tests.rs         # wat::test_suite! { path: "wat-tests", deps: [wat_lru] }
```

**Two Rust files.** Three if the consumer adds its own
`#[wat_dispatch]` shim. Cargo does everything else.

Arcs 013 + 014 + 015 together deliver Chapter 18's *"wat is
the language, Rust is the substrate"* at the ecosystem tier:
- 013: Third parties can publish wat crates; consumers compose
  them at the `main.rs` level.
- 014: Scalar conversions for honest test assertions.
- 015: Consumer wat tests discover + run + compose external
  crates; cargo test does the work.

**holon-lab-trading is next.** The lab will declare Cargo deps
on `wat` + `wat-lru` (+ whatever rusqlite / reqwest / aya wat
crates emerge for its substrate needs). Its `src/main.rs`
will be one `wat::main!`. Its `tests/tests.rs` will be one
`wat::test_suite!`. Its `.wat` programs and tests carry the
domain logic. The lab moves in.

The cave-quest discipline precedent set across arcs 013 →
014 → 015 is now standing practice: when a slice surfaces
real substrate debt that blocks honest completion, pause, name
the key, cut the quest arc (or slice), return. Three arcs in
two days, all closed, zero broken promises.

---

**Arc 015 — complete.** Five slices shipped (1, 2, 3, 3a, 4).
The commits:

**wat-rs repo:**
- `c3c49b8` — docs open (DESIGN + BACKLOG)
- `e6b179b` — slice 1 (test_runner library module)
- `51a0176` — slice 2 (wat::test_suite! proc-macro)
- `211a0d2` — slice 3a (global install) + slice 3 (wat-lru
  wat-tests + wat_suite.rs)
- `6688801` — slice 4 (StdlibFile → WatSource,
  stdlib_sources → wat_sources rename)
- `<this commit>` — CONVENTIONS.md templates + this INSCRIPTION

One binary = one dep set. Consumer writes two Rust files.
Everything else is wat. The door is open.

*PERSEVERARE.*
