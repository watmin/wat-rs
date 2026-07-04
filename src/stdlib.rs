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
    // Generates constructor + per-field accessors + predicate. Consumes Stone
    // 234.2a substrate primitives (:wat::core::Record::of + :wat::core::Record/field-at).
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
    // Arc 259 (The Forced Hand) — the host opts for spawn-program (the Keymaker):
    // ThreadOpts / ProcessOpts / RemoteOpts + their constructors. Loaded AFTER
    // Record.wat (uses :wat::core::defrecord).
    WatSource {
        path: "wat/spawn.wat",
        source: include_str!("../wat/spawn.wat"),
    },
    // Arc 259 S3.2a — the brackets layer runner server-loop.  Loaded AFTER
    // spawn.wat which provides :wat::kernel::Peer', recv', send'.
    WatSource {
        path: "wat/bracket.wat",
        source: include_str!("../wat/bracket.wat"),
    },
    WatSource {
        path: "wat/holon.wat",
        source: include_str!("../wat/holon.wat"),
    },
    // Arc 076: wat/holon/Hologram.wat removed. Hologram/get / put /
    // make / len / capacity are all substrate primitives now; the
    // construction-time filter eliminates the wat-stdlib wrapper layer
    // and the coincident-get / present-get conveniences (Q1 = a).
    WatSource {
        path: "wat/kernel/channel.wat",
        source: include_str!("../wat/kernel/channel.wat"),
    },
    // Arc 214 Stone 8.2 — `:wat::kernel::services::StdInService::*`
    // (the universe-resident shape: Req/Rep records + ONE pure handle fn;
    // the Rust loop lives in src/services/; loaded after channel.wat
    // which provides IOReader typealiases).
    WatSource {
        path: "wat/kernel/services/stdin.wat",
        source: include_str!("../wat/kernel/services/stdin.wat"),
    },
    // Arc 214 Stone 8.1 — `:wat::kernel::services::StdOutService::*`
    // (the universe-resident shape: Req/Rep records + ONE pure handle fn;
    // the Rust loop lives in src/services/).
    WatSource {
        path: "wat/kernel/services/stdout.wat",
        source: include_str!("../wat/kernel/services/stdout.wat"),
    },
    // Arc 214 Stone 8.1b — `:wat::kernel::services::StdErrService::*`.
    // (the universe-resident shape: Req/Rep records + ONE pure handle fn;
    // the Rust loop lives in src/services/, mirroring stdout).
    WatSource {
        path: "wat/kernel/services/stderr.wat",
        source: include_str!("../wat/kernel/services/stderr.wat"),
    },
    // Arc 170 slice 1f-δ — restore :wat::kernel::run-sandboxed-hermetic-ast
    // as wat-side wrapper around spawn-process (closes § Row K from
    // slice 1f-β-i V2 SCORE). Also defines drain-lines-acc,
    // drain-lines, and failure-from-process-died helpers.
    WatSource {
        path: "wat/kernel/hermetic.wat",
        source: include_str!("../wat/kernel/hermetic.wat"),
    },
    // Arc 170 slice 1f-δ′ — restore :wat::kernel::run-sandboxed-ast as
    // wat-side wrapper around spawn-process (closes the largest
    // baseline failure category; sibling of slice 1f-δ's hermetic
    // restore). Loaded AFTER hermetic.wat so drain-lines /
    // failure-from-process-died helpers are already registered.
    WatSource {
        path: "wat/kernel/sandbox.wat",
        source: include_str!("../wat/kernel/sandbox.wat"),
    },
    // Arc 170 Stone D1 — `:wat::kernel::run-threads` bracket macro
    // (single-factory form). Wat-level defmacro; depends on
    // `:wat::kernel::spawn-thread`, `:wat::kernel::ThreadPeer/new` +
    // accessors, `:wat::kernel::Thread/input` + `Thread/output`
    // accessors (Stone C1), `:wat::kernel::Thread/drain-and-join`
    // (Stone A). Loaded AFTER sandbox.wat so the kernel-namespace
    // file ordering matches the C-side dependency tree. D2 (multi-
    // factory) + D3 (panic cascade) extend this same file.
    WatSource {
        path: "wat/kernel/run_threads.wat",
        source: include_str!("../wat/kernel/run_threads.wat"),
    },
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
    // Arc 091 slice 1 — :wat::edn::Tagged + :wat::edn::NoTag newtypes
    // around HolonAST. Used by wat-sqlite's auto-dispatch (arc 085) to
    // pick :wat::edn::write vs :wat::edn::write-notag at TEXT-bind time.
    WatSource {
        path: "wat/edn.wat",
        source: include_str!("../wat/edn.wat"),
    },
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
    // defalias/def/defprotocol/defclause = eval-dep), and returns Violations
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
    // Arc 278 stone 1a — :wat::rete:: — the rete engine data model.
    // Pure data records (Token/Element/Activation, Rule, AlphaNode/RootJoinNode/
    // HashJoinNode/ProductionNode/QueryNode), the Node defenum sum, the Session
    // record, and a render-dag inspection fn. All on stone-0 persistent
    // collections (PersistentMap/PersistentVector). No compile, no fire.
    // Loads AFTER Record.wat (uses :wat::core::defrecord); PersistentMap/PersistentVector
    // are Rust intrinsics — always available.
    WatSource {
        path: "wat/rete.wat",
        source: include_str!("../wat/rete.wat"),
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
        let forms = parse_all_with_file(file.source, file.path).map_err(|e| StdlibError {
            span: crate::rust_caller_span!(),
            kind: StdlibErrorKind::ParseFailed {
                path: file.path,
                cause: e,
            },
        })?;
        all.extend(forms);
    }
    for file in installed_dep_sources().iter().flat_map(|slice| slice.iter()) {
        let forms = parse_all_with_file(file.source, file.path).map_err(|e| StdlibError {
            span: crate::rust_caller_span!(),
            kind: StdlibErrorKind::ParseFailed {
                path: file.path,
                cause: e,
            },
        })?;
        all.extend(forms);
    }
    Ok(all)
}

/// Loader-level failure when a stdlib file can't be parsed.
/// Pattern A (Stone 243.7e): span at the outer struct level; variant data
/// in [`StdlibErrorKind`].
pub struct StdlibError {
    pub span: Span,
    pub kind: StdlibErrorKind,
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

        let stdlib_err = StdlibError {
            span: crate::rust_caller_span!(),
            kind: StdlibErrorKind::ParseFailed {
                path: "stdlib-probe.wat",
                cause: parse_err,
            },
        };

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

        let stdlib_err = StdlibError {
            span: crate::rust_caller_span!(),
            kind: StdlibErrorKind::ParseFailed {
                path: "s3a-probe.wat",
                cause: parse_err,
            },
        };

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
        // :span must reference a real Rust file (wat-rs/src/…), not <runtime>.
        // rune:lint(loose-assert) — variable Rust source file path embedded in span (varies by build environment); path prefix presence is the contract
        assert!(
            s.contains("\"wat-rs/src/"),
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
        let stdlib_err = StdlibError {
            span: known_span,
            kind: StdlibErrorKind::ParseFailed {
                path: "s3a-span-probe.wat",
                cause: parse_err,
            },
        };

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
