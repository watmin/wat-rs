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
pub(crate) mod config_set_eval_redef;
pub(crate) mod config_set_redef;
pub(crate) mod control_flow;
pub(crate) mod def;
pub(crate) mod defalias;
pub(crate) mod defenum;
pub(crate) mod defmacro;
pub(crate) mod defsurface;
pub(crate) mod digest_load;
pub(crate) mod fn_form;
pub(crate) mod forms;
pub(crate) mod load_file;
pub(crate) mod macroexpand;
pub(crate) mod macroexpand_1;
pub(crate) mod match_form;
pub(crate) mod newtype;
pub(crate) mod or_form;
pub(crate) mod quasiquote;
pub(crate) mod quote;
pub(crate) mod rete_i64_gt_alias;
pub(crate) mod signed_load;
pub(crate) mod struct_to_form;
pub(crate) mod structtype;
pub(crate) mod typealias;
pub(crate) mod use_form;
