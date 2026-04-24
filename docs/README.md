# wat — docs

The authoritative specification for the wat language does not live here.
It lives at:

**https://github.com/watmin/holon-lab-trading/tree/main/docs/proposals/2026/04/058-ast-algebra-surface**

That directory is the 058 proposal batch — FOUNDATION.md, the
FOUNDATION-CHANGELOG, thirty-six sub-proposals (058-001 through 058-036),
and two rounds of reviewer notes (Hickey, Beckman). Every design decision
that shaped `wat` is recorded there, with dates and reasoning. When this
crate's behavior and the proposal disagree, the proposal wins — and this
crate gets a slice to close the gap.

Start with:

1. `FOUNDATION.md` — the language specification proper. Algebra core (6
   forms), language core (define / lambda / let / if / cond / match), kernel
   substrate (queue / send / recv / stopped / spawn / select), startup
   pipeline (parse → freeze in 12 steps), constrained eval, `:user::main`
   contract.
2. `FOUNDATION-CHANGELOG.md` — the audit trail. Every correction to the
   spec has an entry with the date and the reasoning.
3. `058-030-types/PROPOSAL.md` — the type system.
4. `058-029-lambda/PROPOSAL.md` — typed anonymous functions.
5. `058-028-define/PROPOSAL.md` — named function registration.

This crate's `README.md` (one level up) documents what has landed and how
to run the binary. For the *why*, read the proposal.

## Also in this directory

**[`USER-GUIDE.md`](./USER-GUIDE.md)** — if you're BUILDING an
application on wat, start here. Crate setup, first program, mental
model, writing functions, structs, algebra forms, concurrency
primitives, pipelines, Rust interop via `#[wat_dispatch]`, caching
tiers, stdio discipline, error handling, common gotchas. Concrete
examples throughout. The guide is alive — it evolves as the trading
lab (first real wat application) gets rebuilt. Where the guide lies,
the rebuild teaches us, and the guide gets updated.

**[`CONVENTIONS.md`](./CONVENTIONS.md)** — naming rules for adding
new primitives. Privileged prefixes, namespace roles
(core/config/algebra/kernel/io/std/rust), case and suffix rules,
and the two lessons that gate new additions (absence is signal;
verbose is honest). Read before proposing a new `:wat::*` or
`:wat::std::*` primitive.

**[`ZERO-MUTEX.md`](./ZERO-MUTEX.md)** — the concurrency architecture,
stated plainly. wat runs dozens of threads, serializes writes to
stdout across every program that wants to print, owns LRU caches hit
concurrently from multiple clients, composes pipeline stages in real
parallel — and has **zero Mutex**. Not fewer. Not mostly. Zero.

The doc names the three tiers (immutable `Arc<T>`; `ThreadOwnedCell<T>`;
program-owned message-addressed via channels) that cover every situation
a Mutex would conventionally answer, walks through every "I need a
Mutex" scenario and shows which tier claims it, and names the honest
caveats (atomics, `OnceLock`, `Arc` are not the tiers but not violations
either). Read it before writing your first concurrent wat program.
Read it before reaching for a lock.

## Arc docs — dated slice design notes

Living planning and postmortem notes for individual slices of work,
organized as `arc/YYYY/MM/NNN-slug/`:

- **`arc/2026/04/001-caching-stack/`** — the L1/L2 caching design
  (LocalCache + Cache program) and the deadlock postmortem where
  `ThreadOwnedCell` clarified its ownership story.
- **`arc/2026/04/002-rust-interop-macro/`** — the `#[wat_dispatch]`
  proc-macro design, the `:rust::` namespace principle, and the
  progress log that tracked the macro arc through its e-block
  features (Vec, Tuple, Result, shared / owned_move scopes).
- **`arc/2026/04/003-tail-call-optimization/`** — the design for TCO
  in the evaluator. Trampoline in `apply_function`; tail-position
  threading; Scheme + Erlang references. Prerequisite for long-running
  driver loops (Console/loop, Cache/loop-step, future pipeline stages).
- **`arc/2026/04/004-lazy-sequences-and-pipelines/`** — the CSP
  pipeline stdlib design + `:rust::std::iter::Iterator` surfacing.
  The Ruby Enumerator pattern mapped to Rust's two-level answer
  (Iterator for in-process lazy; channel `Receiver::into_iter` for
  cross-thread). Depends on 003.
- **`arc/2026/04/007-wat-tests-wat/`** — the self-hosted testing
  arc. ScopedLoader capability gate, `:wat::kernel::run-sandboxed`
  + its hermetic subprocess sibling, `:wat::test::*` stdlib with
  panic-and-catch assertions, AST-entry sandbox + `deftest`
  defmacro, `wat test <path>` CLI with random-order discovery,
  `wat::Harness` thin Rust embedding wrapper. Migrated every
  stdlib test from Rust to `wat-tests/` along the way. Shipped
  alongside the `wat-vm` → `wat` and `program` → `service`
  renames.
- **`arc/2026/04/008-wat-io-substrate/`** — `:u8` primitive +
  `:wat::io::IOReader` / `IOWriter` abstract types +
  StringIoReader / StringIoWriter for in-memory testing + byte-
  honest read/write primitives. UTF-8 lexer correctness fix
  caught mid-migration. Prerequisite for arc 007 slice 2 —
  without substitutable stdio, the sandbox couldn't construct
  `:user::main`'s arguments.
- **`arc/2026/04/009-names-are-values/`** — the fn-by-name lift.
  A registered define's keyword-path in value position now
  evaluates to a callable `Value::wat__core__lambda`, and the
  type checker infers a `:fn(...)->Ret` scheme for the same
  position. Generalizes `:wat::kernel::spawn`'s long-standing
  accept-by-name convention to every `:fn(...)`-typed parameter
  position. Forced by arc 006 slice 4's with-state ergonomics;
  benefits every higher-order combinator downstream.
- **`arc/2026/04/010-variadic-quote/`** — `:wat::core::forms`, the
  variadic sibling of `:wat::core::quote`. Takes N unevaluated
  forms; returns `:Vec<wat::WatAST>` with each form captured as
  data. Closes the per-form quote ceremony at every sandbox /
  eval-ast / programs-as-atoms callsite. Paired with stdlib sugar
  `:wat::test::program` (defmacro alias) + `:wat::test::run-ast`
  (thin `run-sandboxed-ast` wrapper). Kills the escaped-string
  nesting that nested sandbox tests used to carry. Sibling to
  arc 009 in spirit: names are values; forms are values.
- **`arc/2026/04/011-hermetic-ast/`** — the AST-entry hermetic
  sibling. `:wat::kernel::run-sandboxed-hermetic-ast` (primitive) +
  `:wat::test::run-hermetic-ast` (stdlib wrapper) +
  `wat_ast_to_source` / `wat_ast_program_to_source` (substrate
  serializer). Service tests (Console, Cache) no longer carry
  stringified inner programs — same AST shape as the in-process
  sandbox, just with subprocess isolation.
- **`arc/2026/04/012-fork-and-pipes/`** — **shipped.** Raw Unix
  `fork(2)` + `pipe(2)` + `waitpid(2)` as kernel primitives.
  `:wat::kernel::pipe` + PipeReader/PipeWriter (direct-syscall
  writes, no `std::io::stdout` Mutex coupling) + `fork-with-forms`
  returning a `ForkedChild` struct + `ChildHandle` opaque type +
  `wait-child` idempotent via OnceLock-cached exit. Hermetic
  moved from a Rust primitive to wat stdlib
  (`wat/std/hermetic.wat`) on top. Both hermetic Rust primitives
  + the arc 011 AST-to-source serializer retired; side quest
  retired `in_signal_subprocess`'s `Command::spawn` via
  `libc::fork`. Zero `Command::spawn` remain in `src/`. The
  fork substrate is the single source of subprocess truth for
  wat-rs. Unix-only by design.

- **`arc/2026/04/013-external-wat-crates/`** — **shipped.**
  Externalized `wat-lru` into a sibling crate
  (`crates/wat-lru/`). LocalCache left the baked stdlib
  entirely; repathed from `:wat::std::LocalCache` to
  `:user::wat::std::lru::LocalCache` under the community-tier
  convention (later promoted to `:wat::lru::*` by arc 036). `wat::Harness::from_source_with_deps` accepts
  external wat sources; `wat::main!` proc-macro composes
  baked + dep + user source in one declaration.
  `examples/with-lru/` is the walkable reference. Six slices,
  17 commits across two repos, all landed 2026-04-21.
  Chapter 18's *"wat is the language, Rust is the substrate"*
  operational at the ecosystem tier — third parties can
  publish wat crates; consumers compose them at the `main.rs`
  level. wat-rs root has zero dependency on wat-lru — the
  transitive-composition proof holds.

- **`arc/2026/04/014-core-scalar-conversions/`** — **shipped.**
  Cave-quest arc cut mid arc-013 slice 4b when
  `:wat::core::i64::to-string` was needed for honest test
  assertions. Eight primitives at
  `:wat::core::<source>::to-<target>` — i64/f64/bool/string
  conversions with `:Option<T>` for fallible paths (NaN / ±∞ /
  out-of-range / unparseable → `:None`). No implicit coercion;
  every conversion is explicit at the call site. First arc cut
  from a paused slice; the shape is now precedent for future
  cave-quest splits.

- **`arc/2026/04/015-wat-test-for-consumers/`** — **shipped.**
  Closed the last consumer-shape gap: `.wat` tests that
  compose external wat crates, discovered + run by `cargo test`.
  `wat::test!` proc-macro (originally shipped as `wat::test_suite!`;
  renamed in arc 018) mirrors `wat::main!` for tests;
  `wat::test_runner` library is the callable substrate;
  `wat::source::install_dep_sources` is the global OnceLock
  (symmetric with `wat::rust_deps::install`) that lets every
  freeze — main, test, sandbox via `run-sandboxed-ast`, fork
  child via `run-hermetic-ast` — transparently see dep surface.
  `StdlibFile` → `WatSource`, `stdlib_sources()` →
  `wat_sources()` user-facing rename (no back-compat).
  Two Rust files per consumer app (`src/main.rs` + `tests/tests.rs`);
  three when shipping own `#[wat_dispatch]` shim. Five slices
  (1, 2, 3, 3a, 4); second arc cut from a paused slice —
  cave-quest discipline now standing practice.

- **`arc/2026/04/016-failure-location-and-frames/`** — **shipped.**
  Wat test failures now render Rust-styled with wat-source
  `file:line:col`. Every `WatAST` node carries a `Span { file,
  line, col }` from parse; a thread-local call stack populates
  at `apply_function` via RAII guard (tail calls replace the
  top frame in place — constant stack depth for recursion); a
  `std::panic::set_hook` writes `cargo test`-shaped output to
  stderr, gated on `RUST_BACKTRACE` for the `stack backtrace:`
  block. Runtime-initiated frames carry their Rust source
  location (`file!()` / `line!()` / `column!()`) — same
  convention Rust uses for stdlib frames in backtraces. No new
  env var; no new format. Closes arc 007's "Location + Frames
  population" follow-up six months after it opened. Four
  slices + one polish pass, all 2026-04-21.

- **`arc/2026/04/017-loader-option-for-consumer-macros/`** —
  **shipped.** `wat::main!` and `wat::test!` (then named
  `wat::test_suite!` — renamed in arc 018) each gain an
  optional `loader: "<path>"` argument that expands to a
  `ScopedLoader` rooted at `CARGO_MANIFEST_DIR/<path>` so
  `(:wat::load-file! "...")` works from
  multi-file consumer programs. Absent preserves the pre-017
  defaults (InMemoryLoader for main, FsLoader for tests).
  `test_runner` learned **library-vs-entry discipline** — a
  `.wat` in the test dir is an entry iff it has top-level
  `(:wat::config::set-*!)` forms; files without setters are
  libraries and are silently skipped at freeze time (they remain
  `(load!)`-able from entries). Recursive `(load!)` chains
  flatten into the entry's frozen world at arbitrary depth.
  Cave-quest from the trading lab's Phase 0: shipped in one
  session (three slices + a clippy sweep that brought the
  workspace back to zero warnings) to unblock the lab rewrite.
  2026-04-22.

- **`arc/2026/04/018-opinionated-defaults-and-test-rename/`** —
  **shipped.** The consumer story collapses to two one-line
  macros. `wat::main! { deps: [...] }` with `wat/main.wat` as
  the implicit entry and `"wat"` as the implicit loader root;
  `wat::test! { deps: [...] }` (renamed from `wat::test_suite!`,
  pre-publish clean rename) with `wat-tests/` as the implicit
  path and loader. Explicit `source:` / `path:` / `loader:`
  arguments always win. `examples/with-lru/` and
  `examples/with-loader/` + wat-lru's self-tests migrated to
  demonstrate the minimal form — each consumer is `src/main.rs`
  (one `wat::main!` invocation) + `tests/test.rs` (one
  `wat::test!`) + `wat/main.wat` + `wat-tests/**/*.wat`.
  Convention-over-configuration applied to the consumer surface.
  Three slices, same day as arc 017. 2026-04-22.

- **`arc/2026/04/019-f64-round-primitive/`** — **shipped.**
  `:wat::core::f64::round` added as a core conversion primitive
  under the existing scalar-conversions umbrella. Port
  prerequisite for the trading lab's indicator arithmetic.
  2026-04-22.

- **`arc/2026/04/020-assoc/`** — **shipped.** `:wat::core::assoc`
  added over HashMap; returns a new map with a key→value
  association. Forcing function from the archive's
  scaled-linear HashMap threading. 2026-04-22.

- **`arc/2026/04/021-core-std-audit/`** — **shipped.** Core vs
  stdlib naming-discipline sweep: what lives in `:wat::core::`
  (syntactic primitives + arithmetic + literals + types) vs
  `:wat::std::` (blueprint library over core). 2026-04-22.

- **`arc/2026/04/022-holon-namespace-move/`** — **shipped.** All
  algebra primitives moved from `:wat::algebra::` to
  `:wat::holon::`. Matches FOUNDATION's programs-are-holons
  framing — the algebra operates on holons, not on an abstract
  "algebra" layer. 2026-04-22.

- **`arc/2026/04/023-holon-coincident/`** — **shipped.**
  `:wat::holon::coincident?` — the dual of `presence?` VSA
  literature hadn't named. `(1 − cosine) < noise_floor` returns
  bool; two holons "the same point on the sphere to the algebra"
  when the predicate fires. Lab's test-fact-is-bind-of-atom-and-
  thermometer assertion collapses from inline cosine arithmetic
  to one call. Convergence-with-the-greats pattern at a VSA
  naming level. 2026-04-22.

- **`arc/2026/04/024-presence-coincident-sigma/`** — **shipped.**
  Two new config knobs: `presence-sigma` + `coincident-sigma`.
  Defaults are FUNCTIONS of dims (`presence_sigma(d) = floor(sqrt(d)/2) − 1`,
  `coincident_sigma = 1`), not hardcoded. Validity check at
  commit catches predicate collapse. The "opinionated defaults
  are functions, not numbers" principle captured in memory.
  2026-04-22.

- **`arc/2026/04/025-container-surface-unified/`** — **shipped.**
  `get` / `assoc` / `conj` / `contains?` each polymorphized over
  {HashMap, HashSet, Vec} with illegal cells forced by container
  semantics (assoc on set illegal — use conj; conj on map
  illegal — use assoc). `:wat::std::member?` retired in favor of
  unified `contains?`. Forcing function: Phase 3.4 rhythm's Vec
  indexing. 2026-04-22.

- **`arc/2026/04/026-eval-coincident/`** — **shipped.** Four-
  primitive family: `eval-coincident?`, `eval-edn-coincident?`,
  `eval-digest-coincident?`, `eval-signed-coincident?`. Takes
  two expressions, evaluates, atomizes, measures coincidence of
  the resulting atom vectors. Chapter 28's `(= (+ 2 2) (* 1 4))`
  retort operational. Distributed-by-construction substrate
  shipped as four library calls. 2026-04-22.

- **`arc/2026/04/027-deftest-inherits-loader/`** — **shipped.**
  `wat::test!` and deftest's sandbox `scope :None` inherit the
  test binary's filesystem loader so `(load!)` inside a sandbox
  reaches the same roots the test harness reached. Preparation
  for arc 031's sibling scope-inheritance move on Config.
  2026-04-22.

- **`arc/2026/04/028-load-eval-rename/`** — **shipped.** Load
  family gains `:wat::load-string!` sibling to `:wat::load-file!`;
  the old single-form `:wat::load!` dispatch retires. Eval family
  gains explicit `:wat::eval-string!` / `:wat::eval-file!` split.
  Nine forms hoisted from `:wat::core::*` to `:wat::*` root to
  match their kernel-primitive status. 2026-04-22.

- **`arc/2026/04/029-nested-quasiquote/`** — **shipped.**
  `walk_template` gains quote-depth tracking so nested quasiquote
  `,,X` resolves at the correct pass. `expand_form` preserves
  `(:wat::core::quote X)` bodies the same way it already
  preserved quasiquote bodies — a macro-generating-macro's
  registered body stays un-expanded until the inner macro fires.
  Substrate enabler for the configured-deftest factory shape.
  2026-04-23.

- **`arc/2026/04/030-macroexpand/`** — **shipped.**
  `:wat::core::macroexpand` and `:wat::core::macroexpand-1`
  runtime primitives — the standard Lisp macro-debugging tool.
  Fixpoint + one-step variants. Diagnosed arc 029's make-deftest
  nested-quasiquote bug. Arg-order flip for the test macros
  landed in the closing commit (matches arc 024's
  capacity-mode-before-dims discipline). 2026-04-23.

- **`arc/2026/04/031-sandbox-inherits-config/`** — **shipped.**
  Sandbox freeze (`run-sandboxed-ast`, `run-sandboxed-hermetic-ast`,
  `fork-with-forms` child) inherits the caller's committed
  Config by default. All four `:wat::test::*` macros drop their
  `mode` + `dims` parameters — tests inherit from the test
  file's top-level preamble. Path B shipped: one declaration
  site for capacity-mode + dims per test file; every sandbox
  inherits. Same scope-inheritance move arc 027 made for the
  source loader, applied to a different environment field.
  2026-04-23.

- **`arc/2026/04/032-bundle-result-typealias/`** — **shipped.**
  `:wat::holon::BundleResult` registered as a baked built-in
  typealias for `:Result<wat::holon::HolonAST, wat::holon::CapacityExceeded>`
  — Bundle's canonical return shape. Non-parametric. Named by
  the gaze-ward discipline; the `Result` suffix speaks at first
  read, no grep needed. 28-site migration across wat-rs
  (`src/`, `tests/`, `wat/`, `wat-tests/`) + lab rhythm files.
  Small substrate arc; ergonomic payoff across every Bundle-
  threaded caller. 2026-04-23.

- **`arc/2026/04/034-reciprocal-log/`** — **shipped.**
  `:wat::holon::ReciprocalLog` stdlib macro — pure-wat sugar over
  `:wat::holon::Log` with reciprocal bounds `(1/n, n)`. Takes
  `(n value)`, expands to `(Log value (/ 1.0 n) n)`. First-
  principles bound-family for ratio-valued indicators: N=2 covers
  ±doubling, N=3 ±tripling, N=10 ±10x. Log-symmetry automatic
  via reciprocal construction. Named by `/gaze` after builder
  proposed `BoundedLog`; settled as `ReciprocalLog` (Level-2-safe
  — the name IS the structural definition). Zero substrate
  change; 4 new wat-level tests. 2026-04-23.

- **`arc/2026/04/033-holons-typealias/`** — **shipped.**
  `:wat::holon::Holons` registered as a baked built-in typealias
  for `:Vec<wat::holon::HolonAST>` — the list-of-holons shape
  Bundle takes as input and every `encode-*-facts` vocab
  function returns. Named under `/gaze` after rejecting
  `:wat::holon::Facts` on Level-1 epistemic grounds: the type
  is content-agnostic (facts today, predictions tomorrow).
  Plural of the element type, structurally honest, Level-1-safe
  across every content context. 18-site wat-rs migration;
  lab migration follows in lab arc 004. 2026-04-23.
- **`arc/2026/04/037-dim-router/`** — **shipped.** The last
  required magic value retires. `dims` is no longer a single
  global; the ambient `DimRouter` decides vector dim per
  Atom/Bundle construction from the AST's surface shape.
  `EncoderRegistry` materializes per-d encoders lazily with a
  shared seed. Cosine / presence? / coincident? normalize UP
  via AST re-projection at `max(d_a, d_b)`. Every substrate
  default is a function, every user override replaces our
  function: three capability carriers on SymbolTable
  (`dim_router`, `presence_sigma_fn`, `coincident_sigma_fn`)
  with AST-accepting setters (`set-dim-router!`,
  `set-presence-sigma!`, `set-coincident-sigma!`) and
  freeze-time signature checks. `CapacityMode` reduced to two
  variants (`:error` / `:abort`); `:silent` and `:warn`
  retired. Scalar `set-dims!` / `set-noise-floor!` retired;
  scalar sigma setters retired in favor of function-of-d form.
  Compatibility shims keep `:wat::config::dims` and
  `:wat::config::noise-floor` accessors returning
  `DEFAULT_TIERS[0]` defaults until lab callers migrate.
  Seven slices (slice 2 retired mid-arc). 2026-04-24.
- **`arc/2026/04/038-user-guide-recovery/`** — **shipped.**
  USER-GUIDE.md recovery + forward sync. Commit `5b5fad8` (arc 028
  doc sweep) introduced content that crashed input processing on
  read; this arc reverted the file to commit `467a3d4` (the prior
  known-good state from 2026-04-22) and folded arcs 023-037 + the
  wat-lru namespace promotion forward via eight slices of targeted
  per-section edits. No `sed`/`perl`/whole-file rewrite at any
  point. The arc names *wholesale mechanical doc sweeps over
  markdown-heavy targets* as a poison class (sibling to BOOK
  Chapter 32's perl-with-pipes class) and retires it as a pattern.
  Future doc syncs land per-arc as their substrate work ships.
  2026-04-24.
- **`arc/2026/04/039-readme-drift-sync/`** — **shipped.**
  `wat-rs/README.md` drift-only sync. Sibling arc to 038, smaller
  scope (one doc, no recovery phase — builder confirmed via
  GitHub browse the file was readable, just stale). Same eight
  slices' targeted-edit discipline against arcs 022-037 surface
  changes: namespace migration `:wat::algebra::*` →
  `:wat::holon::*` (12 sites), capacity-mode reduction from four
  to two, BundleResult typealias adoption, ReciprocalLog mention,
  workspace layout including `wat/holon/`, arc directory listing
  extended to 039. Test count line removed (specific 731 / 31
  numbers were pre-arc-029, unverifiable without running cargo
  test); replaced with "zero regressions across every shipped
  arc" + per-arc test-suite list. 2026-04-24.
- **`arc/2026/04/040-conventions-drift-sync/`** — **shipped.**
  `wat-rs/docs/CONVENTIONS.md` drift-only sync. Slimmer than 039
  (CONVENTIONS tracks substrate decisions more closely than the
  user-guide tracks shipped surface). Three implementation
  slices: §Sandbox Config inheritance (set-dims! drop per arc 037),
  `wat::test_suite!` → `wat::test!` rename across 5 edit points
  with `tests/wat_suite.rs` → `tests/test.rs` in templates (arc
  018), §Namespaces table descriptions refreshed (`:wat::config::*`
  multi-tier setters, `:wat::holon::*` extended with
  eval-coincident? + typealiases, `:wat::std::*` minus LocalCache,
  new `:wat::lru::*` row) plus a load-file! syntax fix (arc 028
  iface-keyword leftover). Pre-edit verified `wat::test!` still
  emits `fn wat_suite()` so internal function-name references
  stay. 2026-04-24.
- **`arc/2026/04/041-wat-tests-readme-drift-sync/`** —
  **shipped.** `wat-rs/wat-tests/README.md` drift-only sync.
  Smallest doc-drift arc so far (one implementation slice on a
  70-line file). Standard audit set returned **zero retired-form
  occurrences** — drift was by *omission*, not correction. Two
  additions: §Layout gained an external-crates paragraph (consumer
  wat crates including `crates/wat-lru/` ship their own
  `wat-tests/` trees and run via `cargo test -p <crate>` — arcs
  013 + 015 + 036); §In-process vs hermetic restructured macros-
  first / primitives-below (the user-facing `deftest` /
  `deftest-hermetic` / `make-deftest` family from arcs 029 + 031,
  with `:wat::test::run` / `run-hermetic-ast` restated as
  expansion targets). 2026-04-24.
- **`arc/2026/04/042-zero-mutex-drift-sync/`** — **shipped.**
  `wat-rs/docs/ZERO-MUTEX.md` drift-only sync. Three Edits total
  — the smallest implementation surface of any doc-audit arc so
  far. Sole drift was three `:wat::std::service::Cache<K,V>`
  references migrating to `:wat::lru::CacheService<K,V>` per arcs
  013 + 036. Architectural prose otherwise current — the
  three-tier framing (immutable / thread-owned / program-owned),
  HandlePool discipline, spawn/send/recv/select primitives, and
  the empirical claim are all unaffected by arcs 028-037. The
  concurrency story is substrate-shape-agnostic. Builder
  predicted minimal work; correct prediction enabled tight arc
  scoping. 2026-04-24.
- **`arc/2026/04/044-second-verification-pass/`** — **shipped.**
  Round 3 of the "is wat-rs honest?" iteration. Arc 043 said yes;
  builder asked again; surveyed surfaces 043 hadn't covered and
  found 7 drift sites (proc-macro source comments, example
  wat-tests, baked wat-stdlib comments). After Slice 1 fixed
  those, BACKLOG-mandated broadened sweep caught 6 more in
  README.md + docs/README.md (arcs 039 + 042 had touched those
  files but missed these). 13 total fixes across 2 slices.
  Names the iterative-verification observation explicitly:
  "audited" is a pass count, not a final state. Lists what's
  still uncovered (Rust tests' embedded wat strings, wat-tests
  source comments, wat-lru's own docs, Cargo.toml metadata) so
  any round-4 has a starting point. 2026-04-24.
- **`arc/2026/04/043-verification-gaps-closure/`** — **shipped.**
  Verification follow-on to arcs 038-042. Builder asked "is wat-rs
  honest again?" — honest answer was "substantially more, not
  provably." Four checks executed: `cargo test --release`
  produced concrete numbers (725 Rust + ~58 wat tests, zero
  failures, zero clippy); `src/dim_router.rs:188-205` verified
  USER-GUIDE's sigma formula needed a `max(1)` clamp; `src/*.rs`
  grep surfaced 7 stale `wat::test_suite!` doc-comment refs and
  one set-dims! example; cross-doc spot-check surfaced **three
  drift spots in USER-GUIDE §12 that arc 038's slice 5 missed**
  (BundleResult typealias propagation, capacity-mode list
  reduction from 4 → 2 per arc 037, sqrt-formula reframing).
  Plus removed the stale `wat_cache` reference from README's
  integration-suite enumeration (file does not exist; cache tests
  live in wat-lru). Plus pre-commit drift catch: invented
  `:wat::core::i64::sqrt-floor` primitive in a draft, verified
  via grep before push. Honest disclosure: arc 042's "current
  through arc 037" claim was too optimistic; the correction
  lives here, historical record (arc 042 INSCRIPTION) stays
  frozen. 2026-04-24.

These docs are living — revised as slices ship. Superseded content
stays in git history rather than being deleted.
