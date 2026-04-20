//! Bundled wat stdlib — baked into the binary via `include_str!`.
//!
//! Per FOUNDATION.md § "Where Each Lives" (line 2088), each
//! `wat/std/*.wat` file ships one stdlib form whose keyword path
//! matches the file path. The wat-vm's startup pipeline registers
//! these forms BEFORE user entry forms reach macro expansion, so any
//! user program can reference `:wat::std::Subtract`, `:wat::std::Amplify`,
//! etc. without an explicit `load!`.
//!
//! Files live in the repo under `wat/std/` and are compiled into the
//! binary at build time. The runtime has no filesystem dependency for
//! the stdlib — every deployment of `wat-vm` carries the same stdlib
//! bits.

use crate::ast::WatAST;
use crate::parser::parse_all;

/// One baked stdlib file: a logical path (for diagnostics) plus its
/// source contents.
pub struct StdlibFile {
    pub path: &'static str,
    pub source: &'static str,
}

/// Every stdlib source baked into the binary. Order here determines
/// registration order during startup — later files may reference
/// earlier ones (defmacros are available as soon as they register).
pub fn stdlib_files() -> &'static [StdlibFile] {
    STDLIB_FILES
}

const STDLIB_FILES: &[StdlibFile] = &[
    StdlibFile {
        path: "wat/std/Amplify.wat",
        source: include_str!("../wat/std/Amplify.wat"),
    },
    StdlibFile {
        path: "wat/std/Subtract.wat",
        source: include_str!("../wat/std/Subtract.wat"),
    },
    StdlibFile {
        path: "wat/std/Log.wat",
        source: include_str!("../wat/std/Log.wat"),
    },
    StdlibFile {
        path: "wat/std/Circular.wat",
        source: include_str!("../wat/std/Circular.wat"),
    },
    StdlibFile {
        path: "wat/std/Reject.wat",
        source: include_str!("../wat/std/Reject.wat"),
    },
    StdlibFile {
        path: "wat/std/Project.wat",
        source: include_str!("../wat/std/Project.wat"),
    },
    StdlibFile {
        path: "wat/std/Sequential.wat",
        source: include_str!("../wat/std/Sequential.wat"),
    },
    StdlibFile {
        path: "wat/std/Ngram.wat",
        source: include_str!("../wat/std/Ngram.wat"),
    },
    StdlibFile {
        path: "wat/std/Bigram.wat",
        source: include_str!("../wat/std/Bigram.wat"),
    },
    StdlibFile {
        path: "wat/std/Trigram.wat",
        source: include_str!("../wat/std/Trigram.wat"),
    },
    StdlibFile {
        path: "wat/std/program/Console.wat",
        source: include_str!("../wat/std/program/Console.wat"),
    },
];

/// Parse every stdlib source into a flat vec of forms in source order.
/// Called by [`crate::freeze::startup_from_source`] to register stdlib
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
