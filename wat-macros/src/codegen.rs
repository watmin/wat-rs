//! Codegen for `#[wat_dispatch]`. Produces a module containing per-method
//! dispatch/scheme fns plus a public `register()` fn that wires everything
//! into a `wat::rust_deps::RustDepsBuilder`.
//!
//! Sub-slice 193a scope:
//! - Associated fns only (no `self`, `&self`, `&mut self`).
//! - Primitive arg types that have wat-side `FromWat` impls: `i64`, `f64`,
//!   `bool`, `String`, `()`, `wat::runtime::Value` pass-through.
//! - Return types: primitives, `Option<T>` of a primitive, `Self` as
//!   opaque (wrapped via `make_rust_opaque`).
//!
//! Sub-slice 193b will add `self` marshaling. Sub-slice 194 adds scope
//! semantics. Each sub-slice keeps existing tests green.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Error, FnArg, GenericArgument, Ident, ImplItem, ImplItemFn, ItemImpl, Pat, PathArguments,
    ReturnType, Type,
};

use crate::WatDispatchAttr;

/// Top-level codegen entry. Emits:
///   <original impl block>
///   mod __wat_dispatch_<TypeIdent> { ... }
///
/// The module contains dispatch_<m> + scheme_<m> + a public register fn.
pub(crate) fn emit(attr: &WatDispatchAttr, item: &ItemImpl) -> syn::Result<TokenStream> {
    // Derive the TYPE identifier from the impl's self-type. We only
    // support simple path forms like `MyType` or `my::ns::MyType` for
    // now — generics on the self-type are silently stripped from the
    // module identifier (their presence in the impl itself is fine).
    let self_type = &*item.self_ty;
    let module_ident = module_name_from_self_type(self_type)?;

    let type_path_lit = &attr.path;

    // Gather methods. Skip consts / types / macro items inside the
    // impl — those don't participate in dispatch.
    let methods: Vec<&ImplItemFn> = item
        .items
        .iter()
        .filter_map(|i| match i {
            ImplItem::Fn(m) => Some(m),
            _ => None,
        })
        .collect();

    // Emit one dispatch + scheme per method.
    let dispatch_fns: Vec<TokenStream> = methods
        .iter()
        .map(|m| emit_dispatch_fn(self_type, attr, m))
        .collect::<syn::Result<_>>()?;
    let scheme_fns: Vec<TokenStream> = methods
        .iter()
        .map(|m| emit_scheme_fn(attr, m))
        .collect::<syn::Result<_>>()?;

    let register_body = emit_register_body(attr, &methods)?;

    let impl_ts = quote! { #item };
    let out = quote! {
        #impl_ts

        #[allow(non_snake_case)]
        mod #module_ident {
            use super::*;

            const TYPE_PATH: &'static str = #type_path_lit;

            #(#dispatch_fns)*
            #(#scheme_fns)*

            pub fn register(builder: &mut ::wat::rust_deps::RustDepsBuilder) {
                builder.register_type(::wat::rust_deps::RustTypeDecl {
                    path: TYPE_PATH,
                });
                #register_body
            }
        }
    };
    Ok(out)
}

/// Extract a usable identifier from the impl's self-type for module
/// naming. Strips generic parameters.
fn module_name_from_self_type(ty: &Type) -> syn::Result<Ident> {
    if let Type::Path(type_path) = ty {
        if let Some(last) = type_path.path.segments.last() {
            let base = last.ident.to_string();
            return Ok(format_ident!("__wat_dispatch_{}", base));
        }
    }
    Err(Error::new_spanned(
        ty,
        "wat_dispatch: impl self-type must be a simple path (e.g. `MyType` or `my::mod::MyType`)",
    ))
}

/// Build the register-fn body: one `register_symbol` call per method.
fn emit_register_body(attr: &WatDispatchAttr, methods: &[&ImplItemFn]) -> syn::Result<TokenStream> {
    let calls: Vec<TokenStream> = methods
        .iter()
        .map(|m| {
            let name = &m.sig.ident;
            let dispatch_ident = format_ident!("dispatch_{}", name);
            let scheme_ident = format_ident!("scheme_{}", name);
            let path_lit = method_wat_path(attr, m);
            Ok(quote! {
                builder.register_symbol(::wat::rust_deps::RustSymbol {
                    path: #path_lit,
                    dispatch: #dispatch_ident,
                    scheme: #scheme_ident,
                });
            })
        })
        .collect::<syn::Result<_>>()?;
    Ok(quote! { #(#calls)* })
}

/// `":rust::lru::LruCache" + "::" + method_name`
fn method_wat_path(attr: &WatDispatchAttr, method: &ImplItemFn) -> String {
    format!("{}::{}", attr.path, method.sig.ident)
}

// ─── Dispatch fn ─────────────────────────────────────────────────────

fn emit_dispatch_fn(
    self_type: &Type,
    attr: &WatDispatchAttr,
    method: &ImplItemFn,
) -> syn::Result<TokenStream> {
    let name = &method.sig.ident;
    let dispatch_ident = format_ident!("dispatch_{}", name);
    let wat_path = method_wat_path(attr, method);

    // Collect non-receiver args (193a scope: no `self` yet).
    let args = collect_non_receiver_args(method)?;
    let arity = args.len();

    // Arity guard.
    let arity_guard = quote! {
        if args.len() != #arity {
            return Err(::wat::runtime::RuntimeError::ArityMismatch {
                op: #wat_path.into(),
                expected: #arity,
                got: args.len(),
            });
        }
    };

    // Per-arg marshaling.
    let arg_bindings: Vec<TokenStream> = args
        .iter()
        .enumerate()
        .map(|(i, (_pat, ty))| {
            let bind_ident = format_ident!("arg_{}", i);
            let idx = i;
            Ok(quote! {
                let #bind_ident: #ty = <#ty as ::wat::rust_deps::FromWat>::from_wat(
                    &::wat::runtime::eval(&args[#idx], env, sym)?,
                    #wat_path,
                )?;
            })
        })
        .collect::<syn::Result<_>>()?;

    // Invocation: Self::method_name(arg_0, arg_1, ...)
    let arg_idents: Vec<Ident> = (0..arity).map(|i| format_ident!("arg_{}", i)).collect();
    let invocation = quote! {
        let result = <#self_type>::#name(#(#arg_idents),*);
    };

    // Return marshaling. Inspect the method's return type. Self → opaque.
    let return_marshal = emit_return_marshal(self_type, attr, &method.sig.output)?;

    Ok(quote! {
        fn #dispatch_ident(
            args: &[::wat::ast::WatAST],
            env: &::wat::runtime::Environment,
            sym: &::wat::runtime::SymbolTable,
        ) -> ::std::result::Result<::wat::runtime::Value, ::wat::runtime::RuntimeError> {
            #arity_guard
            #(#arg_bindings)*
            #invocation
            #return_marshal
        }
    })
}

/// Skip the receiver if present. 193a: error if receiver present.
fn collect_non_receiver_args(method: &ImplItemFn) -> syn::Result<Vec<(Pat, Type)>> {
    let mut out = Vec::new();
    for input in method.sig.inputs.iter() {
        match input {
            FnArg::Receiver(r) => {
                return Err(Error::new_spanned(
                    r,
                    "wat_dispatch 193a: methods with `self` receivers are not yet supported \
                     (this sub-slice supports associated fns only; receiver marshaling \
                     lands in 193b)",
                ));
            }
            FnArg::Typed(pt) => {
                out.push(((*pt.pat).clone(), (*pt.ty).clone()));
            }
        }
    }
    Ok(out)
}

/// Given the method's return type, emit code that turns a local `result`
/// binding into a wat Value.
///
///   Return = ()         → Ok(::wat::runtime::Value::Unit)
///   Return = Self       → Ok(make_rust_opaque(TYPE_PATH, result))
///   Return = anything   → Ok(<T as ToWat>::to_wat(result))
fn emit_return_marshal(
    self_type: &Type,
    _attr: &WatDispatchAttr,
    output: &ReturnType,
) -> syn::Result<TokenStream> {
    match output {
        ReturnType::Default => Ok(quote! {
            let _ = result;
            Ok(::wat::runtime::Value::Unit)
        }),
        ReturnType::Type(_, ty) => {
            // `Self` becomes an opaque-wrapped return.
            if type_is_self(ty) {
                return Ok(quote! {
                    Ok(::wat::rust_deps::make_rust_opaque(TYPE_PATH, result))
                });
            }
            // If the return type is the concrete self-type (e.g.
            // explicitly writing `MathUtils` instead of `Self`), also
            // wrap as opaque.
            if types_equal(ty, self_type) {
                return Ok(quote! {
                    Ok(::wat::rust_deps::make_rust_opaque(TYPE_PATH, result))
                });
            }
            // Default path: call ToWat on the return.
            Ok(quote! {
                Ok(<#ty as ::wat::rust_deps::ToWat>::to_wat(result))
            })
        }
    }
}

fn type_is_self(ty: &Type) -> bool {
    if let Type::Path(p) = ty {
        if p.path.segments.len() == 1 && p.path.segments[0].ident == "Self" {
            return true;
        }
    }
    false
}

fn types_equal(a: &Type, b: &Type) -> bool {
    // Structural equality via token stringification. Good enough for
    // the simple cases we handle in 193a.
    quote! { #a }.to_string() == quote! { #b }.to_string()
}

// ─── Scheme fn ───────────────────────────────────────────────────────

fn emit_scheme_fn(attr: &WatDispatchAttr, method: &ImplItemFn) -> syn::Result<TokenStream> {
    let name = &method.sig.ident;
    let scheme_ident = format_ident!("scheme_{}", name);
    let wat_path = method_wat_path(attr, method);

    let args = collect_non_receiver_args(method)?;
    let arity = args.len();

    // Per-arg: emit the TypeExpr that represents the declared Rust
    // type, then infer + unify.
    let arg_checks: Vec<TokenStream> = args
        .iter()
        .enumerate()
        .map(|(i, (_pat, ty))| {
            let idx = i;
            let expected_ty_ts = rust_type_to_type_expr_tokens(ty, &attr.path)?;
            let param_name_ts = format!("#{}", i + 1);
            Ok(quote! {
                {
                    let expected_ty = #expected_ty_ts;
                    if let Some(got_ty) = ctx.infer(&args[#idx]) {
                        if !ctx.unify_types(&got_ty, &expected_ty) {
                            ctx.push_type_mismatch(
                                #wat_path,
                                #param_name_ts,
                                format!("{:?}", ctx.apply_subst(&expected_ty)),
                                format!("{:?}", ctx.apply_subst(&got_ty)),
                            );
                        }
                    }
                }
            })
        })
        .collect::<syn::Result<_>>()?;

    // Return-type expression.
    let return_ty_ts = match &method.sig.output {
        ReturnType::Default => quote! { ::wat::types::TypeExpr::Tuple(vec![]) },
        ReturnType::Type(_, ty) => rust_type_to_type_expr_tokens(ty, &attr.path)?,
    };

    let fallback_ty = quote! { ::wat::types::TypeExpr::Tuple(vec![]) };

    Ok(quote! {
        fn #scheme_ident(
            args: &[::wat::ast::WatAST],
            ctx: &mut dyn ::wat::rust_deps::SchemeCtx,
        ) -> ::std::option::Option<::wat::types::TypeExpr> {
            if args.len() != #arity {
                ctx.push_arity_mismatch(#wat_path, #arity, args.len());
                return Some(#fallback_ty);
            }
            #(#arg_checks)*
            Some(#return_ty_ts)
        }
    })
}

/// Map a Rust `syn::Type` to a TokenStream that constructs the
/// matching `wat::types::TypeExpr` at runtime. 193a coverage:
///   i64, f64, bool, String, ()  → TypeExpr::Path
///   Self, concrete self-type    → TypeExpr::Parametric { head: <attr.path stripped>, args: [] }
///   Option<T>                   → TypeExpr::Parametric { head: "Option", args: [<T>] }
///   wat::runtime::Value         → fresh var (checker treats as poly)
fn rust_type_to_type_expr_tokens(ty: &Type, attr_path: &str) -> syn::Result<TokenStream> {
    if type_is_self(ty) {
        // Strip the leading ':' from the attr path (TypeExpr::Parametric
        // stores the head WITHOUT the colon — see existing call sites
        // in runtime.rs which use strings like "rust::lru::LruCache").
        let head = attr_path.trim_start_matches(':').to_string();
        return Ok(quote! {
            ::wat::types::TypeExpr::Parametric {
                head: #head.into(),
                args: vec![],
            }
        });
    }

    if let Type::Path(p) = ty {
        if let Some(last) = p.path.segments.last() {
            let name = last.ident.to_string();
            // Primitives
            match name.as_str() {
                "i64" => {
                    return Ok(quote! { ::wat::types::TypeExpr::Path(":i64".into()) })
                }
                "f64" => {
                    return Ok(quote! { ::wat::types::TypeExpr::Path(":f64".into()) })
                }
                "bool" => {
                    return Ok(quote! { ::wat::types::TypeExpr::Path(":bool".into()) })
                }
                "String" => {
                    return Ok(quote! { ::wat::types::TypeExpr::Path(":String".into()) })
                }
                "Value" => {
                    // wat::runtime::Value — treat as a fresh var (the
                    // checker unifies with whatever the caller passes).
                    return Ok(quote! { ctx.fresh_var() });
                }
                "Option" => {
                    // Option<T> — recurse on T.
                    if let PathArguments::AngleBracketed(ab) = &last.arguments {
                        if let Some(GenericArgument::Type(inner)) = ab.args.first() {
                            let inner_ts = rust_type_to_type_expr_tokens(inner, attr_path)?;
                            return Ok(quote! {
                                ::wat::types::TypeExpr::Parametric {
                                    head: "Option".into(),
                                    args: vec![#inner_ts],
                                }
                            });
                        }
                    }
                    return Err(Error::new_spanned(
                        ty,
                        "wat_dispatch: Option<T> must have exactly one type argument",
                    ));
                }
                _ => {}
            }
        }
    }

    // () → empty tuple.
    if let Type::Tuple(tup) = ty {
        if tup.elems.is_empty() {
            return Ok(quote! { ::wat::types::TypeExpr::Tuple(vec![]) });
        }
    }

    Err(Error::new_spanned(
        ty,
        "wat_dispatch 193a: unsupported argument/return type (supported: i64, f64, bool, \
         String, (), Option<T>, Self, wat::runtime::Value)",
    ))
}

