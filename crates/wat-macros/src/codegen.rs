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
    Receiver, ReturnType, Type,
};

use crate::{Scope, WatDispatchAttr};

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

    let (receiver, args) = split_receiver_and_args(method)?;

    // Total wat-side arity: receiver (if any) + non-receiver args.
    let arity = args.len() + if receiver.is_some() { 1 } else { 0 };

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

    // Non-receiver arg bindings. Indices shift by 1 if a receiver is present.
    let non_receiver_start = if receiver.is_some() { 1 } else { 0 };
    let arg_bindings: Vec<TokenStream> = args
        .iter()
        .enumerate()
        .map(|(i, (_pat, ty))| {
            let bind_ident = format_ident!("arg_{}", i);
            let idx = non_receiver_start + i;
            Ok(quote! {
                let #bind_ident: #ty = <#ty as ::wat::rust_deps::FromWat>::from_wat(
                    &::wat::runtime::eval(&args[#idx], env, sym)?,
                    #wat_path,
                )?;
            })
        })
        .collect::<syn::Result<_>>()?;

    let arg_idents: Vec<Ident> = (0..args.len()).map(|i| format_ident!("arg_{}", i)).collect();

    // Method invocation depends on receiver shape + scope.
    let invocation = match (&receiver, attr.scope) {
        (None, _) => {
            // Associated fn.
            quote! {
                let result = <#self_type>::#name(#(#arg_idents),*);
            }
        }
        (Some(ReceiverKind::RefMut), Scope::ThreadOwned) => {
            // &mut self under thread-owned scope. args[0] is the opaque
            // handle; downcast to &ThreadOwnedCell<Self>; call
            // with_mut so the inner &mut Self can receive the method.
            quote! {
                let self_val = ::wat::runtime::eval(&args[0], env, sym)?;
                let self_inner =
                    ::wat::rust_deps::rust_opaque_arc(&self_val, TYPE_PATH, #wat_path)?;
                let self_cell: &::wat::rust_deps::ThreadOwnedCell<#self_type> =
                    ::wat::rust_deps::downcast_ref_opaque(&self_inner, TYPE_PATH, #wat_path)?;
                let result = self_cell.with_mut(#wat_path, |__self_ref| {
                    __self_ref.#name(#(#arg_idents),*)
                })?;
            }
        }
        (Some(ReceiverKind::Ref), Scope::ThreadOwned) => {
            // &self under thread-owned scope: with_ref instead.
            quote! {
                let self_val = ::wat::runtime::eval(&args[0], env, sym)?;
                let self_inner =
                    ::wat::rust_deps::rust_opaque_arc(&self_val, TYPE_PATH, #wat_path)?;
                let self_cell: &::wat::rust_deps::ThreadOwnedCell<#self_type> =
                    ::wat::rust_deps::downcast_ref_opaque(&self_inner, TYPE_PATH, #wat_path)?;
                let result = self_cell.with_ref(#wat_path, |__self_ref| {
                    __self_ref.#name(#(#arg_idents),*)
                })?;
            }
        }
        (Some(ReceiverKind::Ref), Scope::Shared) => {
            // &self under shared scope. args[0] is the opaque handle;
            // downcast to &Self directly (no guard). Plain &Self call.
            quote! {
                let self_val = ::wat::runtime::eval(&args[0], env, sym)?;
                let self_inner =
                    ::wat::rust_deps::rust_opaque_arc(&self_val, TYPE_PATH, #wat_path)?;
                let self_ref: &#self_type =
                    ::wat::rust_deps::downcast_ref_opaque(&self_inner, TYPE_PATH, #wat_path)?;
                let result = self_ref.#name(#(#arg_idents),*);
            }
        }
        (Some(ReceiverKind::RefMut), Scope::Shared) => {
            return Err(Error::new(
                proc_macro2::Span::call_site(),
                "wat_dispatch: scope = \"shared\" cannot take `&mut self` methods \
                 (shared handles lack interior mutability). Use \
                 scope = \"thread_owned\" for mutable state, or refactor the method \
                 to take `&self` with internal synchronization already baked in.",
            ));
        }
        (Some(ReceiverKind::Owned), Scope::OwnedMove) => {
            // By-value self under owned_move scope. Extract the T out
            // of the OwnedMoveCell on the first use; subsequent
            // invocations on the same handle error cleanly.
            quote! {
                let self_val = ::wat::runtime::eval(&args[0], env, sym)?;
                let self_inner =
                    ::wat::rust_deps::rust_opaque_arc(&self_val, TYPE_PATH, #wat_path)?;
                let self_cell: &::wat::rust_deps::OwnedMoveCell<#self_type> =
                    ::wat::rust_deps::downcast_ref_opaque(&self_inner, TYPE_PATH, #wat_path)?;
                let __self_owned = self_cell.take(#wat_path)?;
                let result = __self_owned.#name(#(#arg_idents),*);
            }
        }
        (Some(ReceiverKind::Owned), _) => {
            return Err(Error::new(
                proc_macro2::Span::call_site(),
                "wat_dispatch: by-value `self` receivers require scope = \"owned_move\" \
                 (a consumed-after-use semantic). For shared or thread-owned mutable \
                 state, use &self or &mut self respectively.",
            ));
        }
        (Some(ReceiverKind::Ref), Scope::OwnedMove) |
        (Some(ReceiverKind::RefMut), Scope::OwnedMove) => {
            return Err(Error::new(
                proc_macro2::Span::call_site(),
                "wat_dispatch: scope = \"owned_move\" only accepts by-value `self` \
                 receivers (the handle is consumed on use). &self / &mut self methods \
                 don't fit the move semantics — use scope = \"shared\" or \
                 scope = \"thread_owned\" instead.",
            ));
        }
    };

    // Return marshaling.
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

/// Which receiver shape a method uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReceiverKind {
    /// `&self`
    Ref,
    /// `&mut self`
    RefMut,
    /// `self` (by value)
    Owned,
}

type NonReceiverArgs = Vec<(Pat, Type)>;

/// Emit the TypeExpr for `Self` based on `attr.type_params`. Empty
/// type_params → `Path(":rust::...")`. Non-empty → Parametric with a
/// fresh var per param (phantom generics like `<K,V>` on LruCache).
fn emit_self_type_expr(attr: &WatDispatchAttr) -> TokenStream {
    if attr.type_params.is_empty() {
        let full_path = attr.path.clone();
        return quote! {
            ::wat::types::TypeExpr::Path(#full_path.into())
        };
    }
    let head = attr.path.trim_start_matches(':').to_string();
    let n = attr.type_params.len();
    let fresh_vars: Vec<TokenStream> = (0..n).map(|_| quote! { ctx.fresh_var() }).collect();
    quote! {
        ::wat::types::TypeExpr::Parametric {
            head: #head.into(),
            args: vec![#(#fresh_vars),*],
        }
    }
}

/// Partition a method's inputs into (optional receiver, non-receiver args).
fn split_receiver_and_args(
    method: &ImplItemFn,
) -> syn::Result<(Option<ReceiverKind>, NonReceiverArgs)> {
    let mut receiver: Option<ReceiverKind> = None;
    let mut args = Vec::new();
    for input in method.sig.inputs.iter() {
        match input {
            FnArg::Receiver(r) => {
                receiver = Some(classify_receiver(r)?);
            }
            FnArg::Typed(pt) => {
                args.push(((*pt.pat).clone(), (*pt.ty).clone()));
            }
        }
    }
    Ok((receiver, args))
}

fn classify_receiver(r: &Receiver) -> syn::Result<ReceiverKind> {
    match (&r.reference, r.mutability.is_some()) {
        (Some(_), true) => Ok(ReceiverKind::RefMut),
        (Some(_), false) => Ok(ReceiverKind::Ref),
        (None, _) => Ok(ReceiverKind::Owned),
    }
}

/// Given the method's return type, emit code that turns a local `result`
/// binding into a wat Value.
///
///   Return = ()         → Ok(::wat::runtime::Value::Unit)
///   Return = Self       → Ok(make_rust_opaque(TYPE_PATH, <wrapped>))
///                         where <wrapped> depends on scope.
///   Return = anything   → Ok(<T as ToWat>::to_wat(result))
///
/// Under `scope = "thread_owned"`, a `Self` return is wrapped in a
/// `ThreadOwnedCell<Self>` before the opaque payload — matches the
/// hand-written lru shim's `LruCacheCell` shape.
fn emit_return_marshal(
    self_type: &Type,
    attr: &WatDispatchAttr,
    output: &ReturnType,
) -> syn::Result<TokenStream> {
    let wrap_self_return = |inner: TokenStream| -> TokenStream {
        match attr.scope {
            Scope::ThreadOwned => quote! {
                Ok(::wat::rust_deps::make_rust_opaque(
                    TYPE_PATH,
                    ::wat::rust_deps::ThreadOwnedCell::new(#inner),
                ))
            },
            Scope::Shared => quote! {
                Ok(::wat::rust_deps::make_rust_opaque(TYPE_PATH, #inner))
            },
            Scope::OwnedMove => quote! {
                Ok(::wat::rust_deps::make_rust_opaque(
                    TYPE_PATH,
                    ::wat::rust_deps::OwnedMoveCell::new(#inner),
                ))
            },
        }
    };

    match output {
        ReturnType::Default => Ok(quote! {
            let _ = result;
            Ok(::wat::runtime::Value::Unit)
        }),
        ReturnType::Type(_, ty) => {
            if type_is_self(ty) || types_equal(ty, self_type) {
                return Ok(wrap_self_return(quote! { result }));
            }
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

    let (receiver, args) = split_receiver_and_args(method)?;
    let receiver_arity = if receiver.is_some() { 1 } else { 0 };
    let arity = args.len() + receiver_arity;

    // Self-arg check, if method has a receiver. For no-generics impls
    // (193b scope), the self-type parses from a wat annotation as
    // `TypeExpr::Path(":rust::...")` — we emit that form so unification
    // matches. Generic self-types (Parametric with args) land with
    // generics support (later sub-slice).
    let self_expected_ts = emit_self_type_expr(attr);
    let self_arg_check = if receiver.is_some() {
        quote! {
            {
                let expected_ty = #self_expected_ts;
                if let Some(got_ty) = ctx.infer(&args[0]) {
                    if !ctx.unify_types(&got_ty, &expected_ty) {
                        ctx.push_type_mismatch(
                            #wat_path,
                            "self",
                            format!("{:?}", ctx.apply_subst(&expected_ty)),
                            format!("{:?}", ctx.apply_subst(&got_ty)),
                        );
                    }
                }
            }
        }
    } else {
        quote! {}
    };

    // Per-non-receiver-arg: emit TypeExpr + infer + unify.
    let arg_checks: Vec<TokenStream> = args
        .iter()
        .enumerate()
        .map(|(i, (_pat, ty))| {
            let idx = receiver_arity + i;
            let expected_ty_ts = rust_type_to_type_expr_tokens(ty, attr)?;
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
        ReturnType::Type(_, ty) => rust_type_to_type_expr_tokens(ty, attr)?,
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
            #self_arg_check
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
fn rust_type_to_type_expr_tokens(ty: &Type, attr: &WatDispatchAttr) -> syn::Result<TokenStream> {
    if type_is_self(ty) {
        return Ok(emit_self_type_expr(attr));
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
                            let inner_ts = rust_type_to_type_expr_tokens(inner, attr)?;
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
                "Vec" => {
                    // Vec<T> — recurse on T.
                    if let PathArguments::AngleBracketed(ab) = &last.arguments {
                        if let Some(GenericArgument::Type(inner)) = ab.args.first() {
                            let inner_ts = rust_type_to_type_expr_tokens(inner, attr)?;
                            return Ok(quote! {
                                ::wat::types::TypeExpr::Parametric {
                                    head: "Vec".into(),
                                    args: vec![#inner_ts],
                                }
                            });
                        }
                    }
                    return Err(Error::new_spanned(
                        ty,
                        "wat_dispatch: Vec<T> must have exactly one type argument",
                    ));
                }
                "Result" => {
                    // Result<T, E> — recurse on both.
                    if let PathArguments::AngleBracketed(ab) = &last.arguments {
                        let mut generics = ab.args.iter().filter_map(|a| match a {
                            GenericArgument::Type(t) => Some(t),
                            _ => None,
                        });
                        if let (Some(t), Some(e)) = (generics.next(), generics.next()) {
                            let t_ts = rust_type_to_type_expr_tokens(t, attr)?;
                            let e_ts = rust_type_to_type_expr_tokens(e, attr)?;
                            return Ok(quote! {
                                ::wat::types::TypeExpr::Parametric {
                                    head: "Result".into(),
                                    args: vec![#t_ts, #e_ts],
                                }
                            });
                        }
                    }
                    return Err(Error::new_spanned(
                        ty,
                        "wat_dispatch: Result<T,E> must have exactly two type arguments",
                    ));
                }
                _ => {}
            }
        }
    }

    // Tuples: (), (A,), (A, B), (A, B, C), ...
    //
    // The unit tuple () becomes TypeExpr::Tuple([]). Non-empty tuples
    // recurse on each element. Arity up to 6 is supported by the
    // marshaling-trait impls (see src/rust_deps/marshal.rs); we emit
    // the TypeExpr for any arity here — the checker doesn't care about
    // the trait-bound limit.
    if let Type::Tuple(tup) = ty {
        if tup.elems.is_empty() {
            return Ok(quote! { ::wat::types::TypeExpr::Tuple(vec![]) });
        }
        let inner: Vec<TokenStream> = tup
            .elems
            .iter()
            .map(|e| rust_type_to_type_expr_tokens(e, attr))
            .collect::<syn::Result<_>>()?;
        return Ok(quote! {
            ::wat::types::TypeExpr::Tuple(vec![#(#inner),*])
        });
    }

    Err(Error::new_spanned(
        ty,
        "wat_dispatch 193a: unsupported argument/return type (supported: i64, f64, bool, \
         String, (), Option<T>, Self, wat::runtime::Value)",
    ))
}

