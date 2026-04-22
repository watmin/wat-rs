//! Bundled wat stdlib — baked into the binary via `include_str!`.
//!
//! Per FOUNDATION.md § "Where Each Lives" (line 2088), each
//! `wat/std/*.wat` file ships one stdlib form whose keyword path
//! matches the file path. The wat's startup pipeline registers
//! these forms BEFORE user entry forms reach macro expansion, so any
//! user program can reference `:wat::std::Subtract`, `:wat::std::Amplify`,
//! etc. without an explicit `load!`.
//!
//! Files live in the repo under `wat/std/` and are compiled into the
//! binary at build time. The runtime has no filesystem dependency for
//! the stdlib — every deployment of `wat` carries the same stdlib
//! bits.

use crate::ast::WatAST;
use crate::parser::parse_all;
use crate::source::{installed_dep_sources, WatSource};

/// Every stdlib source baked into the binary. Order here determines
/// registration order during startup — later files may reference
/// earlier ones (defmacros are available as soon as they register).
pub(crate) fn stdlib_files() -> &'static [WatSource] {
    STDLIB_FILES
}

const STDLIB_FILES: &[WatSource] = &[
    WatSource {
        path: "wat/std/Amplify.wat",
        source: include_str!("../wat/std/Amplify.wat"),
    },
    WatSource {
        path: "wat/std/Subtract.wat",
        source: include_str!("../wat/std/Subtract.wat"),
    },
    WatSource {
        path: "wat/std/Log.wat",
        source: include_str!("../wat/std/Log.wat"),
    },
    WatSource {
        path: "wat/std/Circular.wat",
        source: include_str!("../wat/std/Circular.wat"),
    },
    WatSource {
        path: "wat/std/Reject.wat",
        source: include_str!("../wat/std/Reject.wat"),
    },
    WatSource {
        path: "wat/std/Project.wat",
        source: include_str!("../wat/std/Project.wat"),
    },
    WatSource {
        path: "wat/std/Sequential.wat",
        source: include_str!("../wat/std/Sequential.wat"),
    },
    WatSource {
        path: "wat/std/Ngram.wat",
        source: include_str!("../wat/std/Ngram.wat"),
    },
    WatSource {
        path: "wat/std/Bigram.wat",
        source: include_str!("../wat/std/Bigram.wat"),
    },
    WatSource {
        path: "wat/std/Trigram.wat",
        source: include_str!("../wat/std/Trigram.wat"),
    },
    WatSource {
        path: "wat/std/stream.wat",
        source: include_str!("../wat/std/stream.wat"),
    },
    WatSource {
        path: "wat/std/hermetic.wat",
        source: include_str!("../wat/std/hermetic.wat"),
    },
    WatSource {
        path: "wat/std/test.wat",
        source: include_str!("../wat/std/test.wat"),
    },
    WatSource {
        path: "wat/std/service/Console.wat",
        source: include_str!("../wat/std/service/Console.wat"),
    },
];

/// Parse every stdlib source into a flat vec of forms in source order.
/// Includes BOTH the baked stdlib (compile-time `include_str!`) AND
/// any dep sources a consumer crate installed via
/// [`install_dep_sources`]. Every freeze pass (main, test, sandbox,
/// fork) uses this function, so external wat crates' wat surface
/// is uniformly available to any wat code running in the process —
/// including code inside `:wat::kernel::run-sandboxed-ast` and
/// `:wat::kernel::fork-with-forms`.
///
/// Called by [`crate::freeze::startup_from_source`] and
/// [`crate::freeze::startup_from_forms`] to register stdlib
/// defmacros ahead of user code.
pub fn stdlib_forms() -> Result<Vec<WatAST>, StdlibError> {
    let mut all = Vec::new();
    for file in stdlib_files() {
        let forms = parse_all(file.source).map_err(|e| StdlibError::ParseFailed {
            path: file.path,
            source: format!("{}", e),
        })?;
        all.extend(forms);
    }
    for file in installed_dep_sources().iter().flat_map(|slice| slice.iter()) {
        let forms = parse_all(file.source).map_err(|e| StdlibError::ParseFailed {
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
