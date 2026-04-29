//! `:rust::telemetry::uuid::v4` — fresh canonical-hyphenated UUID
//! per call.
//!
//! Hand-rolled `RustSymbol` registration (no `#[wat_dispatch]`)
//! because v4 is a free function with no opaque-type receiver —
//! same shape as wat-sqlite's `:rust::sqlite::auto-prep` /
//! `auto-dispatch` shims. The macro path is for `impl` blocks.
//!
//! The minting itself goes through `wat_edn::new_uuid_v4()` (arc
//! 092). wat-edn owns the UUID concept — `Value::Uuid`, `#uuid`
//! literal handling, and now generation. wat-measure consumes that
//! single source of truth instead of taking a second uuid pin.

use wat::ast::WatAST;
use wat::rust_deps::{
    RustDepsBuilder, RustDispatch, RustScheme, RustSymbol, SchemeCtx,
};
use wat::runtime::{Environment, RuntimeError, SymbolTable, Value};
use wat::types::TypeExpr;

/// Register `:rust::telemetry::uuid::v4` into the deps builder.
/// Called by [`crate::register`].
pub(crate) fn register(builder: &mut RustDepsBuilder) {
    builder.register_symbol(RustSymbol {
        path: ":rust::telemetry::uuid::v4",
        dispatch: dispatch_uuid_v4 as RustDispatch,
        scheme: scheme_uuid_v4 as RustScheme,
    });
}

/// Type scheme: `(:fn() -> :String)`. Validates arity-0 at type-check
/// time so call-site mistakes surface before runtime.
fn scheme_uuid_v4(args: &[WatAST], ctx: &mut dyn SchemeCtx) -> Option<TypeExpr> {
    if !args.is_empty() {
        ctx.push_arity_mismatch(":rust::telemetry::uuid::v4", 0, args.len());
    }
    Some(TypeExpr::Path(":String".into()))
}

/// Dispatch: mint via `wat_edn::new_uuid_v4()`, render to canonical
/// 8-4-4-4-12 hyphenated hex, hand back as a `Value::String`.
fn dispatch_uuid_v4(
    args: &[WatAST],
    _env: &Environment,
    _sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":rust::telemetry::uuid::v4";
    if !args.is_empty() {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 0,
            got: args.len(),
        });
    }
    let id = wat_edn::new_uuid_v4().to_string();
    Ok(Value::String(std::sync::Arc::new(id)))
}
