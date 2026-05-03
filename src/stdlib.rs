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
//! `:wat::holon::*` primitives) and `wat/std/` (everything else —
//! stream, test harness, services), and are compiled into the binary
//! at build time. The runtime has no filesystem dependency for the
//! stdlib — every deployment of `wat` carries the same stdlib bits.

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
    WatSource {
        path: "wat/stream.wat",
        source: include_str!("../wat/stream.wat"),
    },
    WatSource {
        path: "wat/std/hermetic.wat",
        source: include_str!("../wat/std/hermetic.wat"),
    },
    // Arc 105c — wat/std/sandbox.wat is now bundled. With arc
    // 105a's spawn-program-returns-Result + arc 105b's
    // ThreadDiedError/message accessor, the wat-level
    // `:wat::kernel::run-sandboxed` / `-ast` defines replace the
    // substrate Rust impls (deleted in arc 105c). Vec<String>
    // exits the kernel boundary; it survives only inside this
    // wat-level test-convenience helper where collected output IS
    // the assertion target — the discipline arc 103 named.
    //
    // Loads BEFORE test.wat so test.wat's `run` / `run-ast`
    // wrappers resolve through these defines instead of substrate
    // dispatch.
    WatSource {
        path: "wat/std/sandbox.wat",
        source: include_str!("../wat/std/sandbox.wat"),
    },
    WatSource {
        path: "wat/test.wat",
        source: include_str!("../wat/test.wat"),
    },
    // Arc 096 — `:wat::console::*` (the paired-channel mini-TCP
    // driver, arc 089 slice 5; flattened from
    // `:wat::std::service::Console::*` in arc 109 slice K.console)
    // STAYS in the substrate as a generic service-pattern reference.
    // The telemetry-specific wrappers (Console-dispatcher,
    // ConsoleLogger) and Service<E,G> moved to the wat-telemetry
    // crate; consumers gain them via `deps: [wat_telemetry]`.
    WatSource {
        path: "wat/console.wat",
        source: include_str!("../wat/console.wat"),
    },
    // Arc 091 slice 1 — :wat::edn::Tagged + :wat::edn::NoTag newtypes
    // around HolonAST. Used by wat-sqlite's auto-dispatch (arc 085) to
    // pick :wat::edn::write vs :wat::edn::write-notag at TEXT-bind time.
    WatSource {
        path: "wat/edn.wat",
        source: include_str!("../wat/edn.wat"),
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
