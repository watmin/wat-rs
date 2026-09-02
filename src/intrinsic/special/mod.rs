//! Special-form registry entries — arc 255.SF.
//!
//! Special forms are dispatched by the runtime engine (not by a `NativeHandler`).
//! Each sub-module annotates a unit struct with `#[wat_special_form("<fqdn>")]`,
//! which submits a `SpecialFormSubmission` via `inventory` so `registry()` folds
//! them into the `IntrinsicRegistry` as `Kind::SpecialForm` entries. This makes
//! `render-doc` work for `if`, `let`, and future special forms without
//! re-routing the runtime dispatch path.

pub(crate) mod and_form;
pub(crate) mod binding;
pub(crate) mod control_flow;
pub(crate) mod defsurface;
pub(crate) mod fn_form;
pub(crate) mod match_form;
pub(crate) mod or_form;
