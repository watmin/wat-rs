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
    // Arc 170 slice 1f-β-i — `:wat::kernel::services::StdInService::*`
    // (wat-side StdInService program; mirrors StdInServiceEvent from
    // src/thread_io.rs; loaded after channel.wat which provides
    // Sender / Receiver typealiases).
    WatSource {
        path: "wat/kernel/services/stdin.wat",
        source: include_str!("../wat/kernel/services/stdin.wat"),
    },
    // Arc 170 slice 1f-β-ii — `:wat::kernel::services::StdOutService::*`
    // (wat-side StdOutService program; mirrors StdOutServiceEvent
    // from src/thread_io.rs).
    WatSource {
        path: "wat/kernel/services/stdout.wat",
        source: include_str!("../wat/kernel/services/stdout.wat"),
    },
    // Arc 170 slice 1f-β-iii — `:wat::kernel::services::StdErrService::*`.
    // (wat-side StdErrService program; mirrors StdErrServiceEvent
    // from src/thread_io.rs; fd 2 carries only panic-cascade EDN per
    // TIERS.md doctrine).
    WatSource {
        path: "wat/kernel/services/stderr.wat",
        source: include_str!("../wat/kernel/services/stderr.wat"),
    },
    // Arc 170 slice 1f-δ — restore :wat::kernel::run-sandboxed-hermetic-ast
    // as wat-side wrapper around fork-program-ast (closes § Row K from
    // slice 1f-β-i V2 SCORE). The TIERS.md migration to spawn-process
    // remains a separate future arc. Also defines drain-lines-acc,
    // drain-lines, and failure-from-process-died helpers.
    WatSource {
        path: "wat/kernel/hermetic.wat",
        source: include_str!("../wat/kernel/hermetic.wat"),
    },
    // Arc 170 slice 1f-δ′ — restore :wat::kernel::run-sandboxed-ast as
    // wat-side wrapper around spawn-program-ast (closes the largest
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
    // `spawn-program-ast` which slice 4 destructively retires.
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
    // primitive names (length, etc.) to per-Type impls. Loads BEFORE
    // wat/runtime.wat so dispatches are visible to any reflection-driven
    // macro that might reference them.
    WatSource {
        path: "wat/core.wat",
        source: include_str!("../wat/core.wat"),
    },
    // Arc 143 slice 6 — :wat::runtime::* reflection-driven macros.
    // Depends on substrate primitives from slices 1+2+3 (lookup-define,
    // signature-of-defn, body-of, rename-callable-name, extract-arg-names,
    // and computed-unquote in defmacro bodies). Loads last so all
    // substrate dispatch is in place when this defmacro registers.
    WatSource {
        path: "wat/runtime.wat",
        source: include_str!("../wat/runtime.wat"),
    },
    // Arc 143 slice 7 — :wat::list::* list-operation aliases.
    // Applies :wat::runtime::define-alias to create :wat::list::reduce
    // as an alias for :wat::core::foldl. Must load AFTER wat/runtime.wat
    // so the define-alias macro is registered before this application form.
    WatSource {
        path: "wat/list.wat",
        source: include_str!("../wat/list.wat"),
    },
];

/// Parse every stdlib source into a flat vec of forms in source order.
/// Includes BOTH the baked stdlib (compile-time `include_str!`) AND
/// any dep sources a consumer crate installed via
/// [`install_dep_sources`]. Every freeze pass (main, test, sandbox,
/// fork) uses this function, so external wat crates' wat surface
/// is uniformly available to any wat code running in the process —
/// including code inside `:wat::kernel::run-sandboxed-ast` and
/// `:wat::kernel::fork-program-ast`.
///
/// Called by [`crate::freeze::startup_from_source`] and
/// [`crate::freeze::startup_from_forms`] to register stdlib
/// defmacros ahead of user code.
pub fn stdlib_forms() -> Result<Vec<WatAST>, StdlibError> {
    let mut all = Vec::new();
    for file in stdlib_files() {
        let forms = parse_all_with_file(file.source, file.path).map_err(|e| StdlibError::ParseFailed {
            path: file.path,
            source: format!("{}", e),
        })?;
        all.extend(forms);
    }
    for file in installed_dep_sources().iter().flat_map(|slice| slice.iter()) {
        let forms = parse_all_with_file(file.source, file.path).map_err(|e| StdlibError::ParseFailed {
            path: file.path,
            source: format!("{}", e),
        })?;
        all.extend(forms);
    }
    Ok(all)
}

/// Loader-level failure when a stdlib file can't be parsed.
#[derive(Debug)]
pub enum StdlibError {
    ParseFailed {
        path: &'static str,
        source: String,
    },
}

impl std::fmt::Display for StdlibError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StdlibError::ParseFailed { path, source } => {
                write!(f, "stdlib file {} failed to parse: {}", path, source)
            }
        }
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
