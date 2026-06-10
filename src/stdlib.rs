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
use crate::parser::parse_all_with_file;
use crate::source::{installed_dep_sources, WatSource};
use crate::span::Span;

/// Every stdlib source baked into the binary. Order here determines
/// registration order during startup — later files may reference
/// earlier ones (defmacros are available as soon as they register).
pub(crate) fn stdlib_files() -> &'static [WatSource] {
    STDLIB_FILES
}

const STDLIB_FILES: &[WatSource] = &[
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
    // Arc 234 Stone 234.2b — :wat::Record::def macro. Mints user-defined
    // record-types as dual-form holograms (Value::wat__holon__Record): struct_form
    // (Rust-fast) + holon_form (VSA-aligned), both addressable, both canonical.
    // Generates constructor + per-field accessors + predicate. Consumes Stone
    // 234.2a substrate primitives (:wat::Record::of + :wat::Record/field-at).
    // :wat::holon::defrecord RETIRED at Stone 234.6 (HARD CUT; see git history).
    WatSource {
        path: "wat/Record.wat",
        source: include_str!("../wat/Record.wat"),
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
    WatSource {
        path: "wat/stream.wat",
        source: include_str!("../wat/stream.wat"),
    },
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
    // Arc 146 slice 2 — :wat::core::* dispatches. Routes polymorphic
    // primitive names (length, etc.) to per-Type impls. Array position
    // is not load-bearing for visibility: register_stdlib_defmacros
    // (src/macros/parse.rs) walks the entire concatenated stdlib in a
    // single pre-expansion pass, so all defmacros are registered before
    // any expansion runs regardless of file order.
    WatSource {
        path: "wat/core.wat",
        source: include_str!("../wat/core.wat"),
    },
    // Arc 143 slice 7 — :wat::list::* list-operation aliases.
    // Stone 241.12 — uses :wat::core::defalias (native substrate form) to create
    // :wat::list::reduce and :wat::list::fold as aliases for :wat::core::foldl.
    // Loads after core.wat so all substrate dispatch is in place.
    WatSource {
        path: "wat/list.wat",
        source: include_str!("../wat/list.wat"),
    },
    // Arc 251 — fix-source: the wat-to-wat faithful-Clojure converter (the corpus migrator,
    // written IN wat). Loads after core.wat so its substrate verbs (keyword/to-symbol,
    // the read-string/with-children bridge) are in place. Lives under :wat::fix:: because
    // wat/ is the blessed stdlib dir — a vended tool, explicitly listed here.
    WatSource {
        path: "wat/fix.wat",
        source: include_str!("../wat/fix.wat"),
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
            span: Span::unknown(),
            kind: StdlibErrorKind::ParseFailed {
                path: file.path,
                source: format!("{}", e),
            },
        })?;
        all.extend(forms);
    }
    for file in installed_dep_sources().iter().flat_map(|slice| slice.iter()) {
        let forms = parse_all_with_file(file.source, file.path).map_err(|e| StdlibError {
            span: Span::unknown(),
            kind: StdlibErrorKind::ParseFailed {
                path: file.path,
                source: format!("{}", e),
            },
        })?;
        all.extend(forms);
    }
    Ok(all)
}

/// Loader-level failure when a stdlib file can't be parsed.
/// Pattern A (Stone 243.7e): span at the outer struct level; variant data
/// in [`StdlibErrorKind`].
#[derive(Debug)]
pub struct StdlibError {
    pub span: Span,
    pub kind: StdlibErrorKind,
}

/// Variant data for [`StdlibError`]. The span lives in the outer struct;
/// variants carry ONLY data unique to each failure kind.
#[derive(Debug)]
pub enum StdlibErrorKind {
    ParseFailed {
        path: &'static str,
        source: String,
    },
}

impl std::fmt::Display for StdlibErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StdlibErrorKind::ParseFailed { path, source } => {
                write!(f, "stdlib file {} failed to parse: {}", path, source)
            }
        }
    }
}

impl std::fmt::Display for StdlibError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Span::unknown() - baked stdlib has no wat-source span; elide.
        write!(f, "{}", self.kind)
    }
}

impl std::error::Error for StdlibError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_stdlib_file_parses() {
        let forms = stdlib_forms().expect("stdlib must parse");
        assert!(!forms.is_empty(), "stdlib should ship at least one form");
    }
}
