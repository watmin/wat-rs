//! `:wat::telemetry::Sqlite/auto-spawn` — enum-derived sqlite
//! sink. Arc 085.
//!
//! The substrate-side machinery for "consumer declares an enum,
//! substrate ships a sqlite-backed sink for it." The user-facing
//! factory is the wat-side `Sqlite/auto-spawn` (in
//! `wat/std/telemetry/Sqlite.wat`); this module ships three
//! manually-registered Rust shim primitives that the wat factory
//! composes:
//!
//! - `:rust::sqlite::auto-prep enum-name` — runs ONCE in the caller
//!   thread; reflects on `sym.types` for the enum decl, walks
//!   variants, builds + caches the per-variant `AutoSchema`. Pure
//!   side effect; returns `()`.
//!
//! - `:rust::sqlite::auto-install-schemas db enum-name` — runs ONCE
//!   inside the worker thread; pulls the cached `AutoSchema`,
//!   executes each CREATE TABLE through the worker-local Db.
//!
//! - `:rust::sqlite::auto-dispatch db enum-name entry` — runs PER
//!   ENTRY inside the worker thread; looks up the entry's variant,
//!   binds its fields per the cached field-type vector, calls
//!   substrate's `execute`.
//!
//! All three are hand-registered via `RustSymbol` (no
//! `#[wat_dispatch]` macro) because they need direct `sym.types`
//! access — the macro doesn't expose runtime context to user
//! methods. Pattern follows `eval_macroexpand_*` in the substrate's
//! built-in primitive set.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use rusqlite::types::ToSql;
use wat::ast::WatAST;
use wat::rust_deps::{
    downcast_ref_opaque, rust_opaque_arc, RustDispatch, RustScheme, RustSymbol, SchemeCtx,
    ThreadOwnedCell,
};
use wat::runtime::{eval, Environment, RuntimeError, StructValue, SymbolTable, Value};
use wat::types::{expand_alias, EnumDef, EnumVariant, TypeDef, TypeEnv, TypeExpr};

use wat_sqlite::WatSqliteDb;

/// One variant's pre-computed binder metadata. The CREATE TABLE
/// for this variant lives in `AutoSchema.ordered_ddls`; install
/// runs them all in declaration order independent of which
/// variant a particular dispatch hits.
#[derive(Clone, Debug)]
struct AutoVariant {
    /// `INSERT INTO <table> (<cols>) VALUES (?1, ?2, ...)`.
    insert_sql: String,
    /// Field types in declaration order; drives Value→Param
    /// dispatch at runtime.
    field_types: Vec<TypeExpr>,
}

/// All variants for one enum, keyed by `variant_name`.
#[derive(Clone, Debug, Default)]
struct AutoSchema {
    by_variant: HashMap<String, AutoVariant>,
    /// CREATE DDLs in the order variants were declared. Used by
    /// install-schemas; consumers that depend on table-creation
    /// order (rare) get the declared order.
    ordered_ddls: Vec<String>,
}

/// Arc 093 §6 — column names that earn a single-column BTREE
/// index alongside their CREATE TABLE in `derive_schema`. Only
/// LOW-CARDINALITY columns: time / start_time bucket the entire
/// run into ranges the planner narrows on; namespace partitions
/// by producer identity. High-cardinality columns (`uuid`,
/// `metric_name`) earn no index — their cardinality approaches
/// row count, so the index storage dwarfs the data and the
/// planner can't usefully range over them. Wat filters those
/// post-narrowing (the matches? predicate runs over the time-
/// or namespace-narrowed candidate set; an in-wat equality check
/// is fast enough on a few hundred rows that an index would be
/// strictly worse).
///
/// For `:wat::telemetry::Event` (substrate-defined) this yields:
/// `log.time_ns` + `log.namespace` + `metric.start_time_ns` +
/// `metric.namespace` — four indexes total. Consumer enums
/// reusing these column names benefit transparently — no
/// configuration surface, no tunables; same opinionated property
/// of the substrate's telemetry shape that the schema itself is.
const INDEXABLE_COLUMNS: &[&str] = &[
    "time_ns",
    "start_time_ns",
    "namespace",
];

/// Process-wide cache. Keyed by enum keyword path (e.g.
/// `:trading::log::LogEntry`). `auto-prep` populates; the
/// install/dispatch shims read.
static SCHEMAS: OnceLock<RwLock<HashMap<String, Arc<AutoSchema>>>> = OnceLock::new();

fn schemas() -> &'static RwLock<HashMap<String, Arc<AutoSchema>>> {
    SCHEMAS.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Public: register the three auto-spawn shims into the deps
/// builder. Called from `wat-sqlite::register()`.
pub(crate) fn register(builder: &mut wat::rust_deps::RustDepsBuilder) {
    builder.register_symbol(RustSymbol {
        path: ":rust::sqlite::auto-prep",
        dispatch: dispatch_auto_prep as RustDispatch,
        scheme: scheme_auto_prep as RustScheme,
    });
    builder.register_symbol(RustSymbol {
        path: ":rust::sqlite::auto-install-schemas",
        dispatch: dispatch_auto_install as RustDispatch,
        scheme: scheme_auto_install as RustScheme,
    });
    builder.register_symbol(RustSymbol {
        path: ":rust::sqlite::auto-dispatch",
        dispatch: dispatch_auto_dispatch as RustDispatch,
        scheme: scheme_auto_dispatch as RustScheme,
    });
}

// ─── auto-prep ───────────────────────────────────────────────────

fn scheme_auto_prep(args: &[WatAST], ctx: &mut dyn SchemeCtx) -> Option<TypeExpr> {
    if args.len() != 1 {
        ctx.push_arity_mismatch(":rust::sqlite::auto-prep", 1, args.len());
        return Some(TypeExpr::Tuple(vec![]));
    }
    let expected = TypeExpr::Path(":wat::core::keyword".into());
    if let Some(got) = ctx.infer(&args[0]) {
        if !ctx.unify_types(&got, &expected) {
            ctx.push_type_mismatch(
                ":rust::sqlite::auto-prep",
                "#1",
                format!("{:?}", ctx.apply_subst(&expected)),
                format!("{:?}", ctx.apply_subst(&got)),
            );
        }
    }
    Some(TypeExpr::Tuple(vec![]))
}

fn dispatch_auto_prep(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":rust::sqlite::auto-prep";
    if args.len() != 1 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 1,
            got: args.len(),
        });
    }
    let enum_name = eval_keyword(OP, &args[0], env, sym)?;
    let types = sym.types().ok_or_else(|| RuntimeError::MalformedForm {
        head: OP.into(),
        reason: "no type registry attached to SymbolTable; arc 085 capability missing".into(),
    })?;
    let enum_def = match types.get(&enum_name) {
        Some(TypeDef::Enum(e)) => e.clone(),
        Some(other) => {
            return Err(RuntimeError::MalformedForm {
                head: OP.into(),
                reason: format!("{enum_name} is registered as {:?}, not an enum", other.name()),
            });
        }
        None => {
            return Err(RuntimeError::MalformedForm {
                head: OP.into(),
                reason: format!("no enum declared at {enum_name}"),
            });
        }
    };
    let schema = derive_schema(&enum_def, types).map_err(|reason| RuntimeError::MalformedForm {
        head: OP.into(),
        reason,
    })?;
    schemas()
        .write()
        .unwrap()
        .insert(enum_name, Arc::new(schema));
    Ok(Value::Unit)
}

// ─── auto-install-schemas ────────────────────────────────────────

fn scheme_auto_install(args: &[WatAST], ctx: &mut dyn SchemeCtx) -> Option<TypeExpr> {
    let path = ":rust::sqlite::auto-install-schemas";
    if args.len() != 2 {
        ctx.push_arity_mismatch(path, 2, args.len());
        return Some(TypeExpr::Tuple(vec![]));
    }
    let db_ty = TypeExpr::Path(":rust::sqlite::Db".into());
    if let Some(got) = ctx.infer(&args[0]) {
        if !ctx.unify_types(&got, &db_ty) {
            ctx.push_type_mismatch(
                path,
                "#1",
                format!("{:?}", ctx.apply_subst(&db_ty)),
                format!("{:?}", ctx.apply_subst(&got)),
            );
        }
    }
    let kw_ty = TypeExpr::Path(":wat::core::keyword".into());
    if let Some(got) = ctx.infer(&args[1]) {
        if !ctx.unify_types(&got, &kw_ty) {
            ctx.push_type_mismatch(
                path,
                "#2",
                format!("{:?}", ctx.apply_subst(&kw_ty)),
                format!("{:?}", ctx.apply_subst(&got)),
            );
        }
    }
    Some(TypeExpr::Tuple(vec![]))
}

fn dispatch_auto_install(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":rust::sqlite::auto-install-schemas";
    const TYPE_PATH: &str = ":rust::sqlite::Db";
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 2,
            got: args.len(),
        });
    }
    let db_val = eval(&args[0], env, sym)?;
    let enum_name = eval_keyword(OP, &args[1], env, sym)?;
    let schema = lookup_schema(OP, &enum_name)?;
    let inner = rust_opaque_arc(&db_val, TYPE_PATH, OP)?;
    let cell: &ThreadOwnedCell<WatSqliteDb> = downcast_ref_opaque(&inner, TYPE_PATH, OP)?;
    cell.with_mut(OP, |db| {
        for ddl in &schema.ordered_ddls {
            db.execute_ddl(ddl.clone());
        }
    })?;
    Ok(Value::Unit)
}

// ─── auto-dispatch ───────────────────────────────────────────────

fn scheme_auto_dispatch(args: &[WatAST], ctx: &mut dyn SchemeCtx) -> Option<TypeExpr> {
    let path = ":rust::sqlite::auto-dispatch";
    if args.len() != 3 {
        ctx.push_arity_mismatch(path, 3, args.len());
        return Some(TypeExpr::Tuple(vec![]));
    }
    let db_ty = TypeExpr::Path(":rust::sqlite::Db".into());
    if let Some(got) = ctx.infer(&args[0]) {
        if !ctx.unify_types(&got, &db_ty) {
            ctx.push_type_mismatch(
                path,
                "#1",
                format!("{:?}", ctx.apply_subst(&db_ty)),
                format!("{:?}", ctx.apply_subst(&got)),
            );
        }
    }
    let kw_ty = TypeExpr::Path(":wat::core::keyword".into());
    if let Some(got) = ctx.infer(&args[1]) {
        if !ctx.unify_types(&got, &kw_ty) {
            ctx.push_type_mismatch(
                path,
                "#2",
                format!("{:?}", ctx.apply_subst(&kw_ty)),
                format!("{:?}", ctx.apply_subst(&got)),
            );
        }
    }
    // arg #3 is the entry — generic over E. Unify with a fresh var.
    let _entry_var = ctx.fresh_var();
    Some(TypeExpr::Tuple(vec![]))
}

fn dispatch_auto_dispatch(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":rust::sqlite::auto-dispatch";
    const TYPE_PATH: &str = ":rust::sqlite::Db";
    if args.len() != 3 {
        return Err(RuntimeError::ArityMismatch {
            op: OP.into(),
            expected: 3,
            got: args.len(),
        });
    }
    let db_val = eval(&args[0], env, sym)?;
    let enum_name = eval_keyword(OP, &args[1], env, sym)?;
    let entry = eval(&args[2], env, sym)?;
    let schema = lookup_schema(OP, &enum_name)?;
    let ev = match &entry {
        Value::Enum(ev) => ev.clone(),
        _ => {
            return Err(RuntimeError::MalformedForm {
                head: OP.into(),
                reason: format!(
                    "expected an enum value for entry; got {}",
                    entry.type_name()
                ),
            });
        }
    };
    if ev.type_path != enum_name {
        return Err(RuntimeError::MalformedForm {
            head: OP.into(),
            reason: format!(
                "entry type {} doesn't match auto-prep'd enum {enum_name}",
                ev.type_path
            ),
        });
    }
    let av = schema.by_variant.get(&ev.variant_name).ok_or_else(|| {
        RuntimeError::MalformedForm {
            head: OP.into(),
            reason: format!(
                "no auto-spawn binding for {enum_name}::{} (unit variants are not yet supported)",
                ev.variant_name
            ),
        }
    })?;
    if ev.fields.len() != av.field_types.len() {
        return Err(RuntimeError::MalformedForm {
            head: OP.into(),
            reason: format!(
                "variant {}::{} expected {} fields, entry has {}",
                enum_name,
                ev.variant_name,
                av.field_types.len(),
                ev.fields.len()
            ),
        });
    }
    let bound: Vec<Box<dyn ToSql>> = ev
        .fields
        .iter()
        .zip(av.field_types.iter())
        .enumerate()
        .map(|(i, (v, t))| value_to_tosql(OP, &enum_name, &ev.variant_name, i, v, t, sym))
        .collect::<Result<_, _>>()?;
    let inner = rust_opaque_arc(&db_val, TYPE_PATH, OP)?;
    let cell: &ThreadOwnedCell<WatSqliteDb> = downcast_ref_opaque(&inner, TYPE_PATH, OP)?;
    let sql = av.insert_sql.clone();
    cell.with_mut(OP, move |db| {
        let mut stmt = db.conn.prepare_cached(&sql).unwrap_or_else(|e| {
            panic!("{OP}: prepare {sql:?}: {e}")
        });
        let refs: Vec<&dyn ToSql> = bound.iter().map(|b| b.as_ref()).collect();
        stmt.execute(refs.as_slice()).unwrap_or_else(|e| {
            panic!("{OP}: bind/exec {sql:?}: {e}")
        });
    })?;
    Ok(Value::Unit)
}

// ─── derivation helpers ──────────────────────────────────────────

fn derive_schema(def: &EnumDef, types: &TypeEnv) -> Result<AutoSchema, String> {
    let mut by_variant = HashMap::new();
    let mut ordered_ddls = Vec::new();
    for variant in &def.variants {
        let (name, fields) = match variant {
            EnumVariant::Unit(_) => continue, // unit variants emit nothing; deferred
            EnumVariant::Tagged { name, fields } => (name, fields),
        };
        let table = pascal_to_snake(name);
        let mut col_specs = Vec::new();
        let mut col_names = Vec::new();
        let mut field_types = Vec::new();
        for (field_name, field_ty) in fields {
            // Expand typealiases before checking — `:wat::telemetry::Tags`
            // resolves to `:HashMap<HolonAST,HolonAST>` and that's the
            // form `type_to_affinity` recognizes.
            let resolved = expand_alias(field_ty, types);
            let col = kebab_to_snake(field_name);
            let affinity = type_to_affinity(&resolved)
                .ok_or_else(|| format!(
                    "{}::{}: unsupported field type {:?} \
                     (supports :String, :i64, :f64, :bool, \
                     :wat::edn::Tagged, :wat::edn::NoTag, \
                     :HashMap<K,V>)",
                    def.name, name, field_ty
                ))?;
            col_specs.push(format!("{col} {affinity}"));
            col_names.push(col);
            // Cache the EXPANDED type — value_to_tosql at dispatch
            // time matches against this without re-expanding.
            field_types.push(resolved);
        }
        let create_ddl = format!(
            "CREATE TABLE IF NOT EXISTS {table} ({});",
            col_specs.join(", ")
        );
        let placeholders: Vec<String> =
            (1..=col_names.len()).map(|i| format!("?{i}")).collect();
        let insert_sql = format!(
            "INSERT INTO {table} ({}) VALUES ({})",
            col_names.join(", "),
            placeholders.join(", ")
        );
        ordered_ddls.push(create_ddl);

        // Arc 093 — index any column whose name matches the locked
        // set of telemetry-shape predicates. For
        // `:wat::telemetry::Event` this produces the 7 indexes
        // arc 093 §6 settled (`log.time_ns` / `log.uuid` /
        // `log.namespace` + `metric.start_time_ns` / `metric.uuid`
        // / `metric.namespace` / `metric.metric_name`). Consumer
        // enums whose tables happen to share these column names
        // benefit transparently — same opinionated property of the
        // substrate's telemetry shape, no tuning surface. Column
        // names not in the set get no index (the writer-side cost
        // of an index per row is real; we add them only for
        // columns the read layer actually pushes down on).
        for col in &col_names {
            if INDEXABLE_COLUMNS.contains(&col.as_str()) {
                ordered_ddls.push(format!(
                    "CREATE INDEX IF NOT EXISTS idx_{table}_{col} ON {table}({col});"
                ));
            }
        }
        by_variant.insert(
            name.clone(),
            AutoVariant {
                insert_sql,
                field_types,
            },
        );
    }
    Ok(AutoSchema {
        by_variant,
        ordered_ddls,
    })
}

fn type_to_affinity(t: &TypeExpr) -> Option<&'static str> {
    match t {
        TypeExpr::Path(p) => match p.as_str() {
            ":String" => Some("TEXT NOT NULL"),
            ":i64" => Some("INTEGER NOT NULL"),
            ":f64" => Some("REAL NOT NULL"),
            ":bool" => Some("INTEGER NOT NULL"),
            // Arc 091 slice 1 — HolonAST values land in TEXT via the EDN
            // write-strategy newtype the field is declared with.
            // Tagged → :wat::edn::write (round-trip-safe).
            // NoTag  → :wat::edn::write-notag (lossy, natural form).
            ":wat::edn::Tagged" => Some("TEXT NOT NULL"),
            ":wat::edn::NoTag" => Some("TEXT NOT NULL"),
            _ => None,
        },
        // Arc 091 slice 7 — HashMap fields render as NoTag EDN map text.
        // Same TEXT affinity as the NoTag arm; same write-strategy
        // (write-notag) at bind time. Recognized after typealias
        // expansion so `:wat::telemetry::Tags` (alias to
        // `:HashMap<HolonAST,HolonAST>`) lands here.
        TypeExpr::Parametric { head, .. } if head == "HashMap" => Some("TEXT NOT NULL"),
        _ => None,
    }
}

fn pascal_to_snake(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i != 0 {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

fn kebab_to_snake(s: &str) -> String {
    s.replace('-', "_")
}

fn value_to_tosql(
    op: &str,
    enum_name: &str,
    variant_name: &str,
    idx: usize,
    v: &Value,
    t: &TypeExpr,
    sym: &SymbolTable,
) -> Result<Box<dyn ToSql>, RuntimeError> {
    // HashMap field — render the map via NoTag EDN; bind as TEXT.
    // Recognized BEFORE the Path match because Parametric isn't a
    // Path. Arc 091 slice 7 added this arm for the substrate's
    // own Event::Metric/Log `tags :wat::telemetry::Tags` fields.
    if let TypeExpr::Parametric { head, .. } = t {
        if head == "HashMap" {
            if !matches!(v, Value::wat__std__HashMap(_)) {
                return Err(RuntimeError::MalformedForm {
                    head: op.into(),
                    reason: format!(
                        "{enum_name}::{variant_name}#{idx}: HashMap field expected \
                         a HashMap value; got {}",
                        v.type_name()
                    ),
                });
            }
            let edn = wat::edn_shim::value_to_edn_notag(v, sym.types().map(|a| a.as_ref()));
            return Ok(Box::new(wat_edn::write(&edn)));
        }
    }
    let path = match t {
        TypeExpr::Path(p) => p.as_str(),
        _ => {
            return Err(RuntimeError::MalformedForm {
                head: op.into(),
                reason: format!(
                    "{enum_name}::{variant_name}#{idx}: non-scalar field type — \
                     auto-spawn supports :String/:i64/:f64/:bool plus the \
                     :wat::edn::Tagged / :wat::edn::NoTag newtypes around \
                     HolonAST and :HashMap<K,V>"
                ),
            });
        }
    };
    match (path, v) {
        (":String", Value::String(s)) => Ok(Box::new((**s).clone())),
        (":i64", Value::i64(n)) => Ok(Box::new(*n)),
        (":f64", Value::f64(x)) => Ok(Box::new(*x)),
        (":bool", Value::bool(b)) => Ok(Box::new(*b)),

        // Arc 091 slice 1 — Tagged/NoTag newtypes around HolonAST.
        // Runtime: Value::Struct{type_name: ":wat::edn::Tagged"|":wat::edn::NoTag",
        //                        fields: [Value::holon__HolonAST(_)]}
        // We match on type_name (declared field type AND struct type-name agree
        // since the constructor's body builds Struct{type_name: declared-type}).
        // Then extract field[0]; render via the matching write strategy; bind
        // as TEXT.
        (":wat::edn::Tagged", Value::Struct(s)) if s.type_name == ":wat::edn::Tagged" => {
            let inner = extract_holon_field(s, op, enum_name, variant_name, idx)?;
            let edn = wat::edn_shim::value_to_edn_with(
                &Value::holon__HolonAST(inner),
                sym.types().map(|a| a.as_ref()),
            );
            Ok(Box::new(wat_edn::write(&edn)))
        }
        (":wat::edn::NoTag", Value::Struct(s)) if s.type_name == ":wat::edn::NoTag" => {
            let inner = extract_holon_field(s, op, enum_name, variant_name, idx)?;
            let edn = wat::edn_shim::value_to_edn_notag(
                &Value::holon__HolonAST(inner),
                sym.types().map(|a| a.as_ref()),
            );
            Ok(Box::new(wat_edn::write(&edn)))
        }

        _ => Err(RuntimeError::MalformedForm {
            head: op.into(),
            reason: format!(
                "{enum_name}::{variant_name}#{idx}: field type {path} doesn't match value {}",
                v.type_name()
            ),
        }),
    }
}

/// Extract the inner HolonAST from a `:wat::edn::Tagged` or
/// `:wat::edn::NoTag` struct value. The newtype's compile shape is
/// arity-1 tuple struct (per arc 049); the inner value lives at
/// `fields[0]` and must be `Value::holon__HolonAST`.
fn extract_holon_field(
    s: &StructValue,
    op: &str,
    enum_name: &str,
    variant_name: &str,
    idx: usize,
) -> Result<Arc<holon::HolonAST>, RuntimeError> {
    let f0 = s.fields.first().ok_or_else(|| RuntimeError::MalformedForm {
        head: op.into(),
        reason: format!(
            "{enum_name}::{variant_name}#{idx}: {} value has no inner field",
            s.type_name
        ),
    })?;
    match f0 {
        Value::holon__HolonAST(h) => Ok(h.clone()),
        other => Err(RuntimeError::MalformedForm {
            head: op.into(),
            reason: format!(
                "{enum_name}::{variant_name}#{idx}: {}'s inner must be HolonAST, got {}",
                s.type_name,
                other.type_name()
            ),
        }),
    }
}

fn lookup_schema(op: &str, enum_name: &str) -> Result<Arc<AutoSchema>, RuntimeError> {
    schemas()
        .read()
        .unwrap()
        .get(enum_name)
        .cloned()
        .ok_or_else(|| RuntimeError::MalformedForm {
            head: op.into(),
            reason: format!(
                "no auto-spawn schemas cached for {enum_name}; was :rust::sqlite::auto-prep called?"
            ),
        })
}

fn eval_keyword(
    op: &str,
    ast: &WatAST,
    env: &Environment,
    sym: &SymbolTable,
) -> Result<String, RuntimeError> {
    let v = eval(ast, env, sym)?;
    match v {
        Value::wat__core__keyword(s) => Ok((*s).clone()),
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: ":wat::core::keyword",
            got: other.type_name(),
        }),
    }
}
