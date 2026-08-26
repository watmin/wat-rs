//! Bundled wat stdlib — baked into the binary via `include_str!`.
//!
//! Per FOUNDATION.md § "Where Each Lives" (line 2088), each
//! `wat/<namespace>/*.wat` file ships one stdlib form whose keyword
//! path matches the file path. The wat's startup pipeline registers
//! these forms BEFORE user entry forms reach macro expansion, so any
//! user program can reference `:wat::holon::Subtract`,
//! `:wat::holon::Amplify`, `:wat::stream::*`, etc. without an
//! explicit `load!`.
//!
//! Files live in the repo under `wat/holon/` (algebra idioms over
//! `:wat::holon::*` primitives), `wat/kernel/` (kernel services —
//! hermetic, sandbox, channel, stdio services), and `wat/` root
//! (stream, test harness, and other stdlib). All compiled into the
//! binary at build time. The runtime has no filesystem dependency for
//! the stdlib — every deployment of `wat` carries the same stdlib bits.

use crate::ast::WatAST;
use crate::parser::{parse_all_with_file, ParseError};
use crate::source::{installed_dep_sources, WatSource};
use crate::span::Span;

/// Every stdlib source baked into the binary. Order here determines
/// registration order during startup — later files may reference
/// earlier ones (defmacros are available as soon as they register).
pub(crate) fn stdlib_files() -> &'static [WatSource] {
    STDLIB_FILES
}

/// Foundational → derived. A file precedes another only if it has no
/// eval-time dependency on it (defmacro refs are order-free — registered
/// in the pre-pass). Enforced by `:wat::deporder::verify-stdlib` (see
/// tests) — a violation is a red build.
const STDLIB_FILES: &[WatSource] = &[
    // wat/core.wat MUST be first: foundational aliases + defclauses that
    // other stdlib files eval-depend on. No eval-deps on any other file
    // (its only outward refs are :wat::core::defrecord [a defmacro = order-free]
    // and :wat::holon::HolonAST [a builtin]).
    WatSource {
        path: "wat/core.wat",
        source: include_str!("../wat/core.wat"),
    },
    // Arc 296 step 1b — the kernel diagnostics aggregates, declared in wat (wat is the
    // source of truth). Loads immediately after core.wat because `Failure`/`StopFailure`
    // reference the `:wat::core::Error` surface declared there.
    WatSource {
        path: "wat/kernel/diagnostics.wat",
        source: include_str!("../wat/kernel/diagnostics.wat"),
    },
    // Arc 278 stone S1 — :wat::sqlite::* — the RAW sqlite interop surface over the fresh
    // `:rust::sqlite` shim (src/rust_deps/sqlite.rs, core's FIRST default :rust:: shim).
    // Below the backend-agnostic :wat::query::Store contract (wat/query.wat, S2 satisfies it
    // with this). No eval-deps beyond wat/core.wat's builtins (defrecord/defenum/defclause/
    // typealias/Result/Option/Vector/keyword) — loads immediately after core.wat.
    WatSource {
        path: "wat/sqlite.wat",
        source: include_str!("../wat/sqlite.wat"),
    },
    // Arc 118.2a — the clojure-named lazy/eager HOF surface (map/filter/take/drop are Rust
    // intrinsics, unconditionally available; this file adds `filter` [wat-defined], `mapv`/
    // `filterv`/`into`/`doall`/`dorun`/`reduce`/`count`). Moved here (immediately after
    // core.wat, before holon/*/string.wat/etc.) — those files call `mapv` et al., and
    // `:wat::deporder::verify-stdlib` enforces that a referenced name's defining file loads
    // no later than the referencing file. No eval-deps beyond core.wat's own substrate
    // (defclause/defalias/defn, `:wat::stream::*` builtins).
    WatSource {
        path: "wat/seq.wat",
        source: include_str!("../wat/seq.wat"),
    },
    // Arc 255.1b-iv-c — closed-domain enum types for the metadata-of reflection
    // surface: Kind / DefinedIn / Layer. No eval-deps beyond :wat::core::defenum
    // (a builtin), so it may load immediately after core.wat.
    WatSource {
        path: "wat/runtime-meta.wat",
        source: include_str!("../wat/runtime-meta.wat"),
    },
    // Arc 278 — :wat::gen:: — FINITE GENERATORS, the core of generative testing.
    // A generator is an INDEXED SET (`{card, at : i64 -> T}`), not a seeded random
    // source, which collapses enumerate / sample / shrink into one operation and makes
    // a failing case a PERMANENT coordinate rather than a seed. PROMOTED from
    // wat-scripts/lib/gen.wat on the wat/grep.wat precedent — a move of proven code:
    // 24 laws all mutation-proven, three live rete defects found by its first consumer.
    // Cost is PER SHAPE and no single number is honest: ~2.4us/point for `ints`,
    // ~33us/point for `coords`, ~490us/point for the `such-that o bind o record` shape
    // the rete fuzzer actually uses (measured 2026-08-26). The former "~23us/point"
    // here was a `coords` figure with the qualifier dropped. No oracle ratio is quoted:
    // the $oracle has no perf requirement and gets passively faster as wat stops being
    // interpreted, so any ratio against it decays toward false on its own.
    // Loads after wat/seq.wat (uses `into`/`filter`/`foldl`/`mapv`) and needs nothing
    // further — no holon, no rete, no comms. Design: docs/arc/2026/06/278-rules-engine/GENERATIVE-TESTING.md.
    WatSource {
        path: "wat/gen.wat",
        source: include_str!("../wat/gen.wat"),
    },
    WatSource {
        path: "wat/holon/Amplify.wat",
        source: include_str!("../wat/holon/Amplify.wat"),
    },
    WatSource {
        path: "wat/holon/Subtract.wat",
        source: include_str!("../wat/holon/Subtract.wat"),
    },
    WatSource {
        path: "wat/holon/Log.wat",
        source: include_str!("../wat/holon/Log.wat"),
    },
    WatSource {
        path: "wat/holon/ReciprocalLog.wat",
        source: include_str!("../wat/holon/ReciprocalLog.wat"),
    },
    WatSource {
        path: "wat/holon/Circular.wat",
        source: include_str!("../wat/holon/Circular.wat"),
    },
    WatSource {
        path: "wat/holon/Reject.wat",
        source: include_str!("../wat/holon/Reject.wat"),
    },
    WatSource {
        path: "wat/holon/Project.wat",
        source: include_str!("../wat/holon/Project.wat"),
    },
    WatSource {
        path: "wat/holon/Sequential.wat",
        source: include_str!("../wat/holon/Sequential.wat"),
    },
    WatSource {
        path: "wat/holon/Ngram.wat",
        source: include_str!("../wat/holon/Ngram.wat"),
    },
    WatSource {
        path: "wat/holon/Bigram.wat",
        source: include_str!("../wat/holon/Bigram.wat"),
    },
    WatSource {
        path: "wat/holon/Trigram.wat",
        source: include_str!("../wat/holon/Trigram.wat"),
    },
    // Arc 234 Stone 234.2b — :wat::core::defrecord macro. Mints user-defined
    // record-types as dual-form holograms (Value::wat__holon__Record): struct_form
    // (Rust-fast) + holon_form (VSA-aligned), both addressable, both canonical.
    // Generates constructor + per-field accessors + predicate. Consumes the
    // `:wat::core::Record/field-at` substrate primitive (its Stone 234.2a sibling
    // `:wat::core::Record::of` was deleted at arc 296 G-1b — routes through
    // `aggregate-new` now, per arc 294.c.2a).
    // Arc 293.2-rename — :wat::holon::defrecord RECLAIMED (was retired at Stone 234.6;
    // now the canonical holonic record macro head, peer to :wat::core::defrecord).
    WatSource {
        path: "wat/Record.wat",
        source: include_str!("../wat/Record.wat"),
    },
    // Arc 258 A2 — :wat::program::Env as a flat typed record (seven kernel-stamped fields).
    // Arc 293 annihilation: no longer an extensible base; it is a plain defrecord. Loaded
    // AFTER Record.wat (uses :wat::core::defrecord) and :wat::time::Instant (builtin).
    WatSource {
        path: "wat/program.wat",
        source: include_str!("../wat/program.wat"),
    },
    // Arc 170 capability circuit, stone 2 — :wat::capability::Capability (renamed from
    // Grantable, stone A), the uniform capability methods-surface every <fqdn>::Handle
    // satisfies. Relocated here (from wat/service.wat's old position ~328) so it loads
    // BEFORE wat/spawn.wat and wat/bracket.wat, both of which name it (:uses on the
    // process-locus / the bracket's grant-boot/revoke-shutdown/coordinate-dial). Deps only
    // on core.wat builtins.
    WatSource {
        path: "wat/capability.wat",
        source: include_str!("../wat/capability.wat"),
    },
    // Arc 170 closure #6 — :wat::process::Bracket / :wat::process::Service, the closed set
    // of ps-visible spawned-process identities (the label `:wat::spawn::ProcessOpts/label`
    // rides). Loaded AFTER Record.wat (uses :wat::core::defrecord), BEFORE wat/spawn.wat's
    // two consumers (wat/bracket.wat, wat/service.wat) that construct these types.
    WatSource {
        path: "wat/process.wat",
        source: include_str!("../wat/process.wat"),
    },
    // Arc 259 (The Forced Hand) — the host opts for spawn-program (the Keymaker):
    // ThreadOpts / ProcessOpts / RemoteOpts + their constructors. Loaded AFTER
    // Record.wat (uses :wat::core::defrecord).
    WatSource {
        path: "wat/spawn.wat",
        source: include_str!("../wat/spawn.wat"),
    },
    // Arc 259 S3.2a — the brackets layer runner server-loop.  Loaded AFTER
    // spawn.wat which provides :wat::kernel::Peer, recv', send'.
    WatSource {
        path: "wat/bracket.wat",
        source: include_str!("../wat/bracket.wat"),
    },
    WatSource {
        path: "wat/holon.wat",
        source: include_str!("../wat/holon.wat"),
    },
    // Arc 278 Cache Stone 1/3/4 — :wat::cache:: — the bounded LRU primitive (over the fresh
    // `:rust::cache::Lru` shim, src/rust_deps/cache.rs, core's SECOND default :rust:: shim), the
    // HolographicLru similarity composite, and (Stone 4) the `hologram-svc` service. A cache is
    // table-stakes substrate, so the cache tooling comes home to core (the wat-lru /
    // wat-holon-lru crates are the study ORACLEs, retired in Stone 5).
    // Load position: RELOCATED here (was immediately after wat/core.wat — see git history) once
    // Stone 4's `hologram-svc::init` started calling `:wat::holon::filter-coincident` /
    // `filter-present` / `filter-accept-any` (wat/holon.wat, defined just above) to turn its pure
    // `HologramFilterKind` seed into the live `Fn(f64)->bool` `HolographicLru::new` needs. Those
    // are wat `defn`s, not builtins and not defmacros — an eval-time dependency that IS
    // order-sensitive, unlike `:wat::core::defrecord` (a defmacro, order-free via the pre-pass;
    // Stones 1-3 had no eval-dep beyond that). `:wat::deporder::verify-stdlib` caught the
    // violation (referencer wat/cache.wat, definer wat/holon.wat, symbols the three filter
    // factories) when cache.wat still sat at position 2, long before holon.wat's factories
    // existed. Sitting immediately after wat/holon.wat fixes it by construction: nothing earlier
    // in the stdlib references `:wat::cache::` at all (verified across every prior file), so this
    // is the earliest legal position, not merely a legal one.
    WatSource {
        path: "wat/cache.wat",
        source: include_str!("../wat/cache.wat"),
    },
    // Arc 076: wat/holon/Hologram.wat removed. Hologram/get / put /
    // make / len / capacity are all substrate primitives now; the
    // construction-time filter eliminates the wat-stdlib wrapper layer
    // and the coincident-get / present-get conveniences (Q1 = a).
    WatSource {
        path: "wat/kernel/channel.wat",
        source: include_str!("../wat/kernel/channel.wat"),
    },
    // Arc 170 Phase 3 — stdin.wat is now just the `readln` macro + `MAX-READLN-BYTES` cap. The
    // hand-rolled StdInService (Req/Rep + handle fn) is DELETED; stdin is the primed
    // `:wat::kernel::stdin-svc` defservice (wat/kernel/services/stdio.wat). The old
    // `wat/kernel/services/{stdout,stderr}.wat` (pure dead handle fns) are DELETED — their streams are
    // the primed `stdout-svc`/`stderr-svc` defservices too.
    WatSource {
        path: "wat/kernel/readln.wat",
        source: include_str!("../wat/kernel/readln.wat"),
    },
    // Arc 170 CULMINATION (arc 278 IPC de-prime) — wat/kernel/hermetic.wat
    // and wat/kernel/sandbox.wat ANNIHILATED. They defined the manual
    // sandbox-a-program family (run-sandboxed / run-sandboxed-ast /
    // run-sandboxed-hermetic-ast + drive-sandbox + drain-lines helpers),
    // fully subsumed by the primed peer wire (spawn-program' + send' +
    // recv' + LociDiedError). Their loader entries are removed with the files.
    // Arc 170 slice 1e — `:wat::kernel::ExitCode` retired (REALIZATIONS
    // pass 10 — `:wat::core::nil` IS the success exit code; `:user::main`
    // returns nil; substrate maps to libc::exit(0); panic-cascade maps
    // to libc::exit(N) via slice 1i's StdErrService epilogue). The
    // typealias and its loaded form deleted; `wat/kernel/exit-code.wat`
    // removed in this slice.
    // Arc 118 — wat/stream.wat ANNIHILATED (2026-06-27). The thread-per-pure-stage
    // `:wat::stream::*` HOFs were built wrong, successfully; the namespace is reclaimed
    // for the lazy single-pass Stream family (the foundation `74883c15`, renamed in).
    // Arc 170 slice 3 — wat/std/hermetic.wat retired. The
    // `:wat::kernel::run-sandboxed-hermetic-ast` verb it defined
    // is subsumed by the testing-lib three-layer API per
    // `docs/arc/2026/05/170-program-entry-points/TIERS.md` —
    // `:wat::test::run-hermetic` (Layer 1, in `wat/test.wat`) gives
    // the polished form; tests that need full surface drop to
    // `(:wat::kernel::spawn-process fn)` (Layer 3, substrate). User-
    // source callers of `run-sandboxed-hermetic-ast` are phase B
    // sweep territory.
    // Arc 170 slice 3 — wat/std/sandbox.wat retired. Its
    // `:wat::kernel::run-sandboxed` / `:wat::kernel::run-sandboxed-ast`
    // verbs were the legacy "spawn a fresh-world program from
    // forms or source" surface; built on `spawn-program` /
    // the retired spawn-program substrate (arc 214 1b-ii-ζ.1).
    // Per `docs/arc/2026/05/170-program-entry-points/TIERS.md`,
    // tier-2 spawning post-arc-170 is `(:wat::kernel::spawn-process
    // fn)` — a fn satisfies the `:user::process` contract;
    // closure extraction packages the parent's world for the
    // child; testing-lib's `:wat::test::run-hermetic` (Layer 1,
    // in `wat/test.wat`) is the polished form.
    //
    // The `forms`-input shape sandbox.wat exposed has no clean
    // migration on the new substrate (the closure-extraction
    // primitive consumes a fn, not raw forms). User-source
    // callers of `run-sandboxed-ast` / `run-sandboxed` are phase B
    // sweep territory — migrate to `:wat::test::run-hermetic`
    // (Layer 1) for the typical "run this body and check for
    // failure" case; drop to Layer 3
    // (`:wat::kernel::spawn-process fn` directly) for full
    // typed-channel I/O.
    WatSource {
        path: "wat/test.wat",
        source: include_str!("../wat/test.wat"),
    },
    // Arc 170 slice 1f-η — Console namespace retired. The
    // paired-channel mini-TCP Console driver (arc 089 slice 5,
    // flattened from the :wat::std::service::Console family in arc 109
    // slice K.console) was the pre-orchestrator stdio gateway. The
    // trio of ambient stdio services (StdIn/StdOut/StdErr — slices
    // 1f-β-i/ii/iii) + the runtime orchestrator (slice 1f-γ) +
    // ambient `:wat::kernel::println`/`eprintln`/`readln` (slice
    // 1f-α) now own that contract per TIERS.md doctrine. Console-
    // mediated stdio access fully retired; consumers call the
    // ambient operations directly.
    //
    // Arc 209 naming-conversion stone — wat-level string helpers (kebab->pascal + capitalize).
    // Loads after core.wat so the Rust string primitives (to-uppercase, split, subs, concat, join)
    // are registered before this file's defns are evaluated.
    WatSource {
        path: "wat/string.wat",
        source: include_str!("../wat/string.wat"),
    },
    // Arc 251 — fix-source: the wat-to-wat faithful-Clojure converter (the corpus migrator,
    // written IN wat). Loads after core.wat so its substrate verbs (keyword/to-symbol,
    // the read-string/with-children bridge) are in place. Lives under :wat::fix:: because
    // wat/ is the blessed stdlib dir — a vended tool, explicitly listed here.
    WatSource {
        path: "wat/fix.wat",
        source: include_str!("../wat/fix.wat"),
    },
    // Arc 283 — :wat::source::File — the source-unit type (path + text), shared substrate for every
    // source-processing tool. Lifted out of deporder; MUST load before deporder (which references it).
    WatSource {
        path: "wat/source.wat",
        source: include_str!("../wat/source.wat"),
    },
    // Arc 255 Stone iv-b2-a — :wat::intrinsic::Example record (fqdn/expr/expected/run/pure/det).
    // The typed element returned by the `:wat::intrinsic::examples` reflection seam; records (not
    // heterogeneous tuples) so `verify-examples` (iv-b2-b) can pass typed `expr`/`expected` to
    // `:wat::eval-ast!` without a down-cast. No eval-deps beyond Record.wat + builtins.
    // Loads after source.wat (same deps; ordered here for locality with other type-only files).
    WatSource {
        path: "wat/doctest.wat",
        source: include_str!("../wat/doctest.wat"),
    },
    // Arc 275 Stone 275.1 — :wat::deporder:: — the stdlib load-order analyzer.
    // A pure-wat tool: given an ordered list of SourceFile{path,source} pairs,
    // parses each file's top-level forms, builds a symbol→(file,kind) map,
    // classifies cross-file references (defmacro = order-free; defn/defenum/
    // defalias/def/defclause = eval-dep), and returns Violations
    // (files that eval-depend on later-loaded files). The surface:
    //   (:wat::deporder::verify-stdlib) — verifies the real baked order.
    // Loads after fix.wat (uses read-string + ast->children + ast-kind + ast-name).
    // Array position is NOT load-bearing for the tool's defmacros (registered
    // in the pre-expansion pass); the defns evaluate at load time in order.
    WatSource {
        path: "wat/deporder.wat",
        source: include_str!("../wat/deporder.wat"),
    },
    // Arc 277 Stone 277.1 — :wat::lint:: — the wat-lint framework.
    // A pure-wat linter: a rule is (form → Vector<Finding>); lint-source runs
    // form-level rules over every top-level form of every file; lint-stdlib is
    // the surface (form-level findings + deporder load-order as rule-zero).
    // The first rule is nested-if-=-ladder (detect the if/=/true chain disguising
    // a HashSet/contains? membership). STOP-1: auto-fix deferred to 277.1b.
    // Loads after deporder.wat (uses SourceFile + stdlib-sources + verify).
    WatSource {
        path: "wat/lint.wat",
        source: include_str!("../wat/lint.wat"),
    },
    // Arc 209 Stone C.1 — :wat::service::defservice (pure-wat defmacro).
    // C.1 emits the op enum from the defservice surface; C.2/C.3 extend.
    // Order is not load-bearing (register_stdlib_defmacros pre-expansion walk).
    WatSource {
        path: "wat/service.wat",
        source: include_str!("../wat/service.wat"),
    },
    // wat-level IO conveniences over the Rust IOWriter primitives — write-file (one-shot) +
    // with-open-file (managed scope). The `with-` naming law: `with-` = managed lifecycle.
    WatSource {
        path: "wat/io.wat",
        source: include_str!("../wat/io.wat"),
    },
    // Arc 170 stdio-as-defservice (PHASE 1) — the three primed stdio defservices
    // (:wat::kernel::{stdout,stderr,stdin}-svc + their StdOut/StdErr/StdIn surfaces).
    // COEXISTS with the hand-rolled path (the old Std*Service/handle fns above + spawn_service_peer
    // + the five eval_kernel_* verbs). Loads AFTER wat/service.wat (defservice) and wat/io.wat (the
    // from-fd raw-fd constructors' neighbourhood). The fd rides `:init` as a PURE i64 and the impure
    // IOWriter/IOReader is born inside init via IOWriter/from-fd (dup-then-own) — never an init
    // param (the Pure-`Admin` containment wall, arc 293.W). Freeze-bootstrapped on the real fds by
    // src/freeze.rs; nothing flipped in Phase 1.
    WatSource {
        path: "wat/kernel/services/stdio.wat",
        source: include_str!("../wat/kernel/services/stdio.wat"),
    },
    // Arc 278 stone 1a — :wat::rete:: — the rete engine data model.
    // Pure data records (Token/Element, Rule, AlphaNode/RootJoinNode/
    // HashJoinNode/ProductionNode/QueryNode), the Session
    // record, and a render-dag inspection fn. All on stone-0 persistent
    // collections (PersistentMap/PersistentVector). No compile, no fire.
    // Loads AFTER Record.wat (uses :wat::core::defrecord); PersistentMap/PersistentVector
    // are Rust intrinsics — always available.
    WatSource {
        path: "wat/rete.wat",
        source: include_str!("../wat/rete.wat"),
    },
    // Arc 278 — interpreted compile (rule-set → network). Dual of the native
    // compiler. Loads AFTER wat/rete.wat (records, Session).
    WatSource {
        path: "wat/rete/compile.wat",
        source: include_str!("../wat/rete/compile.wat"),
    },
    // Arc 278 Stone 8-i — acc::* fold library. Loads AFTER wat/rete.wat (Element).
    // Fire's accumulate-pass eval-depends on these names.
    WatSource {
        path: "wat/rete/acc.wat",
        source: include_str!("../wat/rete/acc.wat"),
    },
    // Arc 278 — interpreted fire oracle, split like rete.wat / kernel/fire/.
    // insert → pass (alpha/join/production) → accum-pass → stratify → fire (once/rules)
    // → explain. Dual of the native kernel. Loads AFTER compile and acc::*.
    WatSource {
        path: "wat/rete/oracle/insert.wat",
        source: include_str!("../wat/rete/oracle/insert.wat"),
    },
    WatSource {
        path: "wat/rete/oracle/pass.wat",
        source: include_str!("../wat/rete/oracle/pass.wat"),
    },
    WatSource {
        path: "wat/rete/oracle/accum-pass.wat",
        source: include_str!("../wat/rete/oracle/accum-pass.wat"),
    },
    WatSource {
        path: "wat/rete/oracle/stratify.wat",
        source: include_str!("../wat/rete/oracle/stratify.wat"),
    },
    WatSource {
        path: "wat/rete/oracle/fire.wat",
        source: include_str!("../wat/rete/oracle/fire.wat"),
    },
    WatSource {
        path: "wat/rete/oracle/explain.wat",
        source: include_str!("../wat/rete/oracle/explain.wat"),
    },
    // Arc 278 — query / cond / defrule / defquery. query-read reads Session.
    // defmacro refs are order-free.
    WatSource {
        path: "wat/rete/syntax.wat",
        source: include_str!("../wat/rete/syntax.wat"),
    },
    // Arc 278 — DESIGN-STONE-wat-grep-is-a-feature — :wat::grep:: — wat-grep's vocabulary:
    // Node/Named/Span (the fact base wat-grep inserts, per file), Match/Capture (what a rule
    // asserts), and the ONE query q-match. Also extent-of (the sole ast-span/ast-end-span
    // unwrap door) and facts-of (source -> Facts). References :wat::core:: builtins (defrecord,
    // ast-span/ast-kind/ast-name/read-string, HashMap/Option/PersistentVector) and
    // :wat::rete::defquery (wat/rete/syntax.wat) — loads after both.
    WatSource {
        path: "wat/grep.wat",
        source: include_str!("../wat/grep.wat"),
    },
    // Arc 278 stone S4 — :wat::query:: — the backend-agnostic storage CONTRACT (DynamoDB-shaped
    // narrow waist: (pk,sk,data) + named-GSI (ipk,isk), all keys EDN-form strings), on the
    // services-as-surfaces OPERATION MODEL (arc 293 Path B): `Store` is a `:nature
    // :wat::kernel::Peer` surface — a dialed `:satisfies Store` peer IS a Store, intrinsically
    // (no wrapper struct, no extend-type). Pure declarations: the Store methods-bearing surface,
    // the Reason open-record error surface + Transient/Constraint/Fatal/Fault records, the
    // per-op `Store::<Op>Request`/`Store::<Op>Response` (outcome enum) records, and the plain
    // records every satisfier speaks (StoredRow/IndexKey/Row/IndexRow/Page/IndexPage/
    // TableSchema/IndexSchema). NO backend, NO logic (a satisfier — `mem-store`/
    // `sqlite-store` — lives in its own sibling file).
    // Loads AFTER wat/core.wat (defrecord/defenum/defsurface + Result/Option/Vector/HashMap/
    // keyword primitives); placed beside wat/rete.wat — this is the query engine's vocabulary.
    // `:wat::query::` is net-new + unprimed (no battery collides); a baked core source may
    // define under `:wat::` (stdlib bypasses the reserved-prefix gate — RegistrationPrivilege::
    // Stdlib in src/types.rs).
    WatSource {
        path: "wat/query.wat",
        source: include_str!("../wat/query.wat"),
    },
    // wat/query/mem.wat — `:wat::query::mem-store`, the FIRST `:wat::query::Store` satisfier (a
    // `:wat::service::defservice :satisfies :wat::query::Store` holding a
    // PersistentVector<StoredRow>; a real in-memory backend AND the oracle sqlite will be
    // differential-tested against). A dialed peer IS the Store — no wrapper struct. Baked in
    // CORE. Both baked-context gaps that first blocked a stdlib defservice satisfying a stdlib
    // surface are FIXED:
    //   (1) expand_all is stdlib-privileged, so a defservice's expansion-born `…/start` companion
    //       registers instead of hitting ReservedPrefix (MacroRegistry::stdlib_privilege, set in
    //       src/freeze/env.rs; gated by macros/tests.rs::stdlib_privilege_bypasses_reserved_prefix).
    //   (2) a baked body-only extend-type inherits its impl signatures from the surface's
    //       SurfaceMember::Method (register_stdlib_runtime_defs, src/runtime.rs) instead of the nil
    //       placeholders parse_extend_type_form emits — otherwise every method's sig reads nil.
    // Loads after wat/query.wat (the Store contract) and wat/service.wat (defservice).
    WatSource {
        path: "wat/query/mem.wat",
        source: include_str!("../wat/query/mem.wat"),
    },
    // wat/query/sqlite-store.wat — arc 278 stone S4: `:wat::query::sqlite-store`, the sqlite
    // `:wat::query::Store` satisfier (`:satisfies :wat::query::Store`; SQL over S1's
    // `:wat::sqlite` verbs, DDB-faithful secondary-complete-tables GSIs, clear-then-insert
    // `put`). A dialed peer IS the Store — no wrapper struct. Differential-tested against
    // `:wat::query::mem-store` (this file's sibling above). Loads after wat/sqlite.wat (S1
    // verbs), wat/query.wat (the Store contract + records), and wat/query/mem.wat (no eval-dep on
    // mem.wat, but grouped beside it as the query engine's second Store satisfier).
    WatSource {
        path: "wat/query/sqlite-store.wat",
        source: include_str!("../wat/query/sqlite-store.wat"),
    },
    // wat/telemetry.wat — arc 278 stone ①: the `:wat::telemetry` DATA VOCABULARY, PLUS (stone
    // T1b.1) the `Journal` surface — the telemetry sink's S4c contract, write half
    // (`write-metrics`/`write-logs`). The 7 pure vocabulary declarations (Tags/Numeric/Unit/Level/
    // Scope/Metric/Log) the telemetry sink + producer services (later stones) build on. (`Log.message`
    // is opaque `:wat::core::String` EDN text — arc 278 Stone B; the `LogMessage` open surface retired.)
    // Metric/Log are `defrecord`s that SPLICE the Scope surface via `~@:wat::telemetry::Scope`
    // (arc-293 surface-splice) — spliced fields inline first, then own; the unified aggregate ctor
    // + register_aggregate_methods mint the spliced accessors. Namespace is `:wat::telemetry`
    // (Stone C1 annihilated the legacy wat-telemetry battery crate that squatted the bare name;
    // Stone C3 reclaimed it — the prime is gone); a baked core source may declare under `:wat::`
    // (stdlib bypasses the
    // reserved-prefix gate). Loads AFTER wat/core.wat (defrecord/defenum/defsurface/typealias +
    // splice) AND, since T1b.1, AFTER wat/query.wat: the `Journal` surface's
    // `Journal::Write{Metrics,Logs}Response` enums reuse `:wat::query::{Constraint,Transient,
    // Fatal}` (the store's error vocabulary, pass-through — not a parallel telemetry error
    // vocabulary) as payloads, on the `:nature :wat::kernel::Peer` shape `Store` has
    // (wat/query.wat:101). Placed last in the manifest (after query.wat/mem.wat/sqlite-store.wat)
    // — already satisfies this dependency; no reorder needed.
    WatSource {
        path: "wat/telemetry.wat",
        source: include_str!("../wat/telemetry.wat"),
    },
    // wat/telemetry/journal.wat — `:wat::telemetry::journal` (arc 278 T1b.2), the telemetry sink
    // service. `:satisfies :wat::telemetry::Journal`, HOLDS a `:wat::query::Store` peer (S4d
    // `:peers`), serializes Metric/Log -> StoredRow -> Store/put. Loads LAST — after telemetry.wat
    // (Journal/Metric/Log/PartitionKey/Kind), query.wat (Store), and service.wat (defservice).
    WatSource {
        path: "wat/telemetry/journal.wat",
        source: include_str!("../wat/telemetry/journal.wat"),
    },
    // wat/telemetry/span.wat — `:wat::telemetry::span` (arc 278 Span.2), the PRODUCER service.
    // `:satisfies :wat::telemetry::Span`, HOLDS a `:wat::telemetry::Journal` peer; accumulates
    // counters/durations and emits them as Metrics on `close`. Loads after journal.wat.
    WatSource {
        path: "wat/telemetry/span.wat",
        source: include_str!("../wat/telemetry/span.wat"),
    },
    // wat/repl.wat — `:repl::turn`, the read/eval/print loop, as a stdlib MODULE. It defines
    // no `:user::main`: the entry point is the CLI's `--repl` shim, so this file adds a
    // LIBRARY (any program may `(:repl::turn defs)` to embed a loop seeded with its own
    // definitions) rather than a program. Depends only on builtins — `eval-with-defs!`,
    // `read-frame`, `println`, `read-string` — so it loads last with no eval-deps to satisfy.
    WatSource {
        path: "wat/repl.wat",
        source: include_str!("../wat/repl.wat"),
    },
];

/// Parse every stdlib source into a flat vec of forms in source order.
/// Includes BOTH the baked stdlib (compile-time `include_str!`) AND
/// any dep sources a consumer crate installed via
/// [`install_dep_sources`]. Every freeze pass (main, test, sandbox,
/// fork) uses this function, so external wat crates' wat surface
/// is uniformly available to any wat code running in the process —
/// including code inside `:wat::kernel::run-sandboxed-ast` and
/// `:wat::kernel::spawn-process` children.
///
/// Called by [`crate::freeze::startup_from_source`] and
/// [`crate::freeze::startup_from_forms`] to register stdlib
/// defmacros ahead of user code.
pub fn stdlib_forms() -> Result<Vec<WatAST>, StdlibError> {
    let mut all = Vec::new();
    for file in stdlib_files() {
        let forms = parse_all_with_file(file.source, file.path).map_err(|e| StdlibError::new(
            crate::rust_caller_span!(),
            StdlibErrorKind::ParseFailed {
                path: file.path,
                cause: e,
            },
        ))?;
        all.extend(forms);
    }
    for file in installed_dep_sources().iter().flat_map(|slice| slice.iter()) {
        let forms = parse_all_with_file(file.source, file.path).map_err(|e| StdlibError::new(
            crate::rust_caller_span!(),
            StdlibErrorKind::ParseFailed {
                path: file.path,
                cause: e,
            },
        ))?;
        all.extend(forms);
    }
    Ok(all)
}

/// Loader-level failure when a stdlib file can't be parsed.
/// Pattern A (Stone 243.7e): span at the outer struct level; variant data
/// in [`StdlibErrorKind`].
pub struct StdlibError {
    span: Span,
    /// Boxed (arc 109 stone C, mirroring `RuntimeError`'s B2). Inline, this
    /// field made `StdlibError` 152 bytes; boxed, it is 56 (48 span + 8
    /// pointer), so its width no longer tracks `StdlibErrorKind`'s widest
    /// variant. Private — reached only through `new` / `kind` / `into_kind`
    /// — so the box is invisible to callers, the same contract as
    /// `RuntimeError` (`src/value/signal.rs`).
    kind: Box<StdlibErrorKind>,
}

impl StdlibError {
    /// The ONE door for construction.
    pub fn new(span: Span, kind: StdlibErrorKind) -> Self {
        Self { span, kind: Box::new(kind) }
    }
    /// The ONE door for reading the kind.
    pub fn kind(&self) -> &StdlibErrorKind {
        &self.kind
    }
    /// The ONE door for taking the kind by value.
    pub fn into_kind(self) -> StdlibErrorKind {
        *self.kind
    }
    /// Span stays inline — it is not what this stone boxes.
    pub fn span(&self) -> &Span {
        &self.span
    }
}

/// Variant data for [`StdlibError`]. The span lives in the outer struct;
/// variants carry ONLY data unique to each failure kind.
///
/// Arc 296 Strike 3a: `#[derive(ToEdn)]` generates `impl crate::to_edn::ToEdn
/// for StdlibErrorKind`. The `cause` field uses `error_edn_of` (floor form:
/// `:message`/`:location`/`:causes`) matching the deleted hand-written serializer
/// which called `cause.error_edn()`. The `path` field (`&'static str`) serializes
/// via the blanket `impl<T: ToEdn+?Sized> ToEdn for &T` + `impl ToEdn for str`.
#[derive(Debug, wat_edn::ToEdn)]
#[to_edn(namespace = crate::error_ns::STDLIB)]
pub enum StdlibErrorKind {
    ParseFailed {
        path: &'static str,
        #[to_edn(via = crate::to_edn::error_edn_of)]
        cause: ParseError,
    },
}

impl std::fmt::Display for StdlibErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StdlibErrorKind::ParseFailed { path, cause } => {
                write!(f, "stdlib file {} failed to parse: {}", path, cause)
            }
        }
    }
}

impl std::fmt::Debug for StdlibError {
    // Stone B: Debug emits EDN, not Rust struct layout.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&crate::to_edn::to_wire_edn(self))
    }
}

impl std::fmt::Display for StdlibError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&crate::to_edn::to_wire_edn(self))
    }
}

impl std::error::Error for StdlibError {}

// ─── Arc 296 — structured EDN ────────────────────────────────────────────────

impl crate::to_edn::WatError for StdlibError {
    /// Concise single-line headline: the span-free kind Display's first line
    /// (the baked stdlib has no wat-source span, so `:location` is nil).
    fn message(&self) -> String {
        crate::to_edn::first_line(self.kind.to_string())
    }
    fn location(&self) -> wat_edn::OwnedValue {
        crate::to_edn::location_from_span(&self.span)
    }
    fn causes(&self) -> wat_edn::OwnedValue {
        wat_edn::OwnedValue::Vector(vec![])
    }
    fn variant(&self) -> wat_edn::OwnedValue {
        use crate::to_edn::ToEdn;
        crate::to_edn::strip_span_from_tagged(self.to_edn())
    }
}

impl crate::to_edn::ToEdn for StdlibError {
    /// Pattern A: derive on StdlibErrorKind generates the variant body;
    /// `:span` appended via `span.to_edn()` (Stone B).
    fn to_edn(&self) -> wat_edn::OwnedValue {
        use crate::to_edn::edn_kw;
        use wat_edn::OwnedValue;
        let kind_val = self.kind.to_edn();
        match kind_val {
            OwnedValue::Tagged(tag, body) => {
                let mut fields = match *body {
                    OwnedValue::Map(f) => f,
                    other => vec![(edn_kw("body"), other)],
                };
                fields.push((edn_kw("span"), self.span.to_edn()));
                OwnedValue::Tagged(tag, Box::new(OwnedValue::Map(fields)))
            }
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_stdlib_file_parses() {
        let forms = stdlib_forms().expect("stdlib must parse");
        assert!(!forms.is_empty(), "stdlib should ship at least one form");
    }

    /// RED gate — arc 109 (kill-std), BRIEF-typeerror-loaderror-one-door (scope extended
    /// by the coordinator after STOP-2 named `StdlibError` as `StartupError`'s real
    /// remaining driver): `StdlibError` must stay narrow enough that clippy's
    /// `result_large_err` (threshold `>= 128`) never fires on `Result<_, StdlibError>`.
    ///
    /// `StdlibError` is `pub(crate)`, so this gate lives in-crate (mirrors the
    /// `s6_parse_failed_edn_carries_typed_cause_not_source_string` precedent above,
    /// which is in-crate for the identical reason) rather than in
    /// `tests/value/probe_runtime_error_width.rs`, which houses this campaign's other
    /// width walls but cannot name a `pub(crate)` type.
    ///
    /// MEASURED at HEAD (before this stone's StdlibError box): `size_of::<StdlibError>()
    /// == 152` (48 span + 104 `StdlibErrorKind`, `ParseFailed { path: &'static str, cause:
    /// ParseError }`). Expected after boxing the private `kind`: 56.
    ///
    /// The ceiling is 120, not 128 — see `runtime_error_stays_narrow` in
    /// `tests/value/probe_runtime_error_width.rs` for the grounding (clippy fires at
    /// `>= 128`; `RuntimeError` sitting at exactly 128 still threw all 482 of its warnings).
    #[test]
    fn stdlib_error_stays_narrow() {
        assert!(
            std::mem::size_of::<StdlibError>() <= 120,
            "StdlibError is {} bytes (ceiling 120; clippy::result_large_err fires at >= 128). \
             This stone boxes the private `kind` field to bring this to 56 — a red here means \
             either that box was removed or a fat field landed in the outer struct",
            std::mem::size_of::<StdlibError>()
        );
    }

    // ─── Arc 296 S6 probe — ParseFailed EDN carries :cause, not :source ──────
    //
    // StdlibError is pub(crate); this test lives in-crate where it's
    // constructible. Mirrors the approach of probe_arc296_typed_causes.rs for
    // S1/S2 (those are in the integration test since RuntimeError is pub).
    #[test]
    fn s6_parse_failed_edn_carries_typed_cause_not_source_string() {
        use crate::to_edn::ToEdn;

        let bad_source = "(unclosed";
        let parse_err = crate::parser::parse_all_with_file(bad_source, "stdlib-probe.wat")
            .expect_err("probe source must fail to parse");

        let stdlib_err = StdlibError::new(
            crate::rust_caller_span!(),
            StdlibErrorKind::ParseFailed {
                path: "stdlib-probe.wat",
                cause: parse_err,
            },
        );

        let edn = stdlib_err.to_edn();
        let s = wat_edn::write(&edn);

        eprintln!("=== S6 StdlibError::ParseFailed edn: {}", s);

        // Must be tagged.
        assert!(s.starts_with('#'), "must be tagged EDN; got: {}", s);

        // Must carry :cause — the nested ParseError floor form.
        // rune:lint(loose-assert) — output embeds dynamic ParseError location (file/line/col from parse call-site) making full assert_eq! infeasible; field presence is the contract
        assert!(
            s.contains(":cause"),
            "ParseFailed EDN must carry :cause (typed ParseError); got: {}",
            s
        );

        // Must NOT carry :source — the old prose field.
        // rune:lint(loose-assert) — targeted absence on variable output (nested ParseError embeds dynamic parse call-site location)
        assert!(
            !s.contains(":source"),
            "ParseFailed EDN must NOT carry old :source String; got: {}",
            s
        );

        // :cause must embed a nested #wat.parse/… tagged form.
        // rune:lint(loose-assert) — output embeds dynamic ParseError location (file/line/col from parse call-site) making full assert_eq! infeasible; tag prefix presence is the contract
        assert!(
            s.contains("#wat.parse/"),
            "ParseFailed :cause must be a nested #wat.parse/... tagged EDN; got: {}",
            s
        );

        // Must round-trip through EDN parse.
        wat_edn::parse_owned(&s).expect("must be valid EDN");
    }

    // ─── Arc 296 Strike 3a probe — StdlibError::ParseFailed derive set-equality ─
    //
    // The derive must produce the SAME key set as the deleted hand-written
    // serializer: `:path`, `:cause` (floor EDN from `error_edn_of`), and
    // optionally `:span` (elided when unknown, present when known).
    //
    // Full string-equality is not possible here because the nested ParseError's
    // `error_edn()` output is dynamic (file/line/col vary by parse call-site).
    // Instead we assert: tagged, correct variant name, correct key set, correct
    // span behaviour (present / elided), and valid EDN.

    /// S3a — `ParseFailed` with `rust_caller_span!()` MUST carry `:span` (arc 298.2:
    /// every span is a real location; the elide-when-unknown discipline is retired).
    #[test]
    fn s3a_parse_failed_edn_rust_caller_span_carries_span_key() {
        use crate::to_edn::ToEdn;

        let bad_source = "(unclosed";
        let parse_err = crate::parser::parse_all_with_file(bad_source, "s3a-probe.wat")
            .expect_err("probe source must fail to parse");

        let stdlib_err = StdlibError::new(
            crate::rust_caller_span!(),
            StdlibErrorKind::ParseFailed {
                path: "s3a-probe.wat",
                cause: parse_err,
            },
        );

        let edn = stdlib_err.to_edn();
        let s = wat_edn::write(&edn);

        eprintln!("=== s3a rust_caller_span StdlibError::ParseFailed edn: {}", s);

        // Must be the correct tagged form.
        // rune:lint(loose-assert) — EDN embeds rust_caller_span!() (variable file/line/col) and nested ParseError location; full assert_eq! infeasible; tag prefix is the contract
        assert!(
            s.starts_with("#wat.stdlib/ParseFailed"),
            "must be #wat.stdlib/ParseFailed; got: {}",
            s
        );
        // Must carry :path.
        // rune:lint(loose-assert) — output is variable (rust_caller_span!() + nested ParseError location); key presence is the contract
        assert!(s.contains(":path"), "must carry :path; got: {}", s);
        // Must carry :cause (floor form from error_edn_of).
        // rune:lint(loose-assert) — output is variable (rust_caller_span!() + nested ParseError location); key presence is the contract
        assert!(s.contains(":cause"), "must carry :cause; got: {}", s);
        // Arc 298.2: rust_caller_span!() IS a real location — :span MUST be emitted.
        // rune:lint(loose-assert) — output is variable (rust_caller_span!() + nested ParseError location); :span key presence is the contract
        assert!(s.contains(":span"), "rust_caller_span!() must emit :span; got: {}", s);
        // :span must reference a real Rust file (a `.rs` path), not <runtime>. The suffix is
        // the honest test; the old `wat-rs/` prefix was glue, not provenance.
        // rune:lint(loose-assert) — variable Rust source file path embedded in span (varies by build environment); path prefix presence is the contract
        assert!(
            s.contains(".rs\""),
            ":span file must be a real Rust path; got: {}",
            s
        );
        // Must NOT carry :source (old prose field).
        // rune:lint(loose-assert) — targeted absence on variable output (rust_caller_span!() + nested ParseError location); absence of old field is the contract
        assert!(!s.contains(":source"), "must NOT carry old :source; got: {}", s);
        // :cause must be a floor-form tagged value.
        // rune:lint(loose-assert) — output is variable (rust_caller_span!() + nested ParseError location); tag namespace prefix is the contract
        assert!(s.contains("#wat.parse/"), ":cause must embed #wat.parse/...; got: {}", s);
        // Must be valid EDN.
        wat_edn::parse_owned(&s).expect("must be valid EDN");
    }

    /// S3a — `ParseFailed` with a known span MUST carry `:span`.
    #[test]
    fn s3a_parse_failed_edn_known_span_carries_span_key() {
        use crate::to_edn::ToEdn;
        use std::sync::Arc;

        let bad_source = "(unclosed";
        let parse_err = crate::parser::parse_all_with_file(bad_source, "s3a-span-probe.wat")
            .expect_err("probe source must fail to parse");

        let known_span = crate::span::Span::new(Arc::new("s3a-span-probe.wat".to_string()), 1, 0);
        let stdlib_err = StdlibError::new(
            known_span,
            StdlibErrorKind::ParseFailed {
                path: "s3a-span-probe.wat",
                cause: parse_err,
            },
        );

        let edn = stdlib_err.to_edn();
        let s = wat_edn::write(&edn);

        eprintln!("=== s3a known-span StdlibError::ParseFailed edn: {}", s);

        // Must be the correct tagged form.
        // rune:lint(loose-assert) — EDN embeds nested ParseError location (dynamic file/line/col); full assert_eq! infeasible; tag prefix is the contract
        assert!(
            s.starts_with("#wat.stdlib/ParseFailed"),
            "must be #wat.stdlib/ParseFailed; got: {}",
            s
        );
        // Must carry :path.
        // rune:lint(loose-assert) — output is variable (nested ParseError location); key presence is the contract
        assert!(s.contains(":path"), "must carry :path; got: {}", s);
        // Must carry :cause.
        // rune:lint(loose-assert) — output is variable (nested ParseError location); key presence is the contract
        assert!(s.contains(":cause"), "must carry :cause; got: {}", s);
        // :span must reference the correct file with deterministic line=1 col=0.
        // Stone B: :span is now a #wat.core/Span tagged record, not a bare map.
        // rune:lint(loose-assert) — output is variable (nested ParseError location); known span tag + file is the deterministic contract proven here
        assert!(
            s.contains(r#":span #wat.core/Span {:file "s3a-span-probe.wat" :line 1 :col 0"#),
            ":span must embed the known file with line 1 col 0; got: {}",
            s
        );
        // :cause must be a floor-form tagged value.
        // rune:lint(loose-assert) — output is variable (nested ParseError location); tag namespace prefix is the contract
        assert!(s.contains("#wat.parse/"), ":cause must embed #wat.parse/...; got: {}", s);
        // Must be valid EDN.
        wat_edn::parse_owned(&s).expect("must be valid EDN");
    }
}
