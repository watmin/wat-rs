//! Stone 255.90 — the name census. A bin, not a test: it reports and does not gate.
//!
//! Parses every `.rs` file under `src/`, `crates/`, and `tests/` with `syn` and
//! classifies `::` string literals by the syntax around them. The heresy ledger
//! (`tests/lint/keyword_heresy_ledger.rs`) is the shape: a real parser, not a
//! line window.
//!
//! ```text
//! name_census [--root DIR] [--out FILE] [--proof FILE]
//! ```
//!
//! `--proof` parses one extra scratch file and checks that each planted class
//! landed on the class its marker names. The scratch file is not part of the tree.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use syn::spanned::Spanned;
use syn::{Expr, Item, Lit, Pat, Stmt, Type};

const TEXT_CONTAINERS: &[&str] = &["HashMap", "BTreeMap", "HashSet", "BTreeSet"];
const DOORS: &[&str] = &[
    "canonical_identity",
    "fact_class_key",
    "ns_to_wat_path",
    "canonical_type_key",
];
const SURFACE_METHODS: &[&str] = &["as_str", "leaf", "path", "receiver", "flat"];
const CMP_METHODS: &[&str] = &[
    "starts_with",
    "ends_with",
    "strip_prefix",
    "strip_suffix",
    "contains",
    "eq",
    "ne",
];
const PRINT_MACROS: &[&str] = &[
    "panic",
    "todo",
    "unimplemented",
    "compile_error",
    "println",
    "eprintln",
    "print",
    "eprint",
    "writeln",
    "write",
    "format",
    "format_args",
    "bail",
    "ensure",
    "anyhow",
    "tracing",
    "info",
    "warn",
    "error",
    "debug",
    "trace",
];

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Class {
    Reg,
    Dispatch,
    Cmp,
    Build,
    Msg,
    Wat,
    Other,
}

impl Class {
    fn tag(self) -> &'static str {
        match self {
            Class::Reg => "REG",
            Class::Dispatch => "DISPATCH",
            Class::Cmp => "CMP",
            Class::Build => "BUILD",
            Class::Msg => "MSG",
            Class::Wat => "WAT",
            Class::Other => "OTHER",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Taint {
    Identifier,
    Other,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Consume {
    /// Compared, keyed, stored, or passed on as a name.
    Identity,
    /// Reaches output only: format, write, panic, push_str.
    Print,
}

#[derive(Clone)]
struct Site {
    pat: bool,
    cmp: bool,
    /// Inside `format!` / `concat!` and the literal is part of an assembled name.
    build: bool,
    msg: bool,
    reg_attr: bool,
    reg_decl: bool,
    arm_body: bool,
    doc: bool,
    consume: Consume,
}

impl Site {
    fn neutral() -> Self {
        Site {
            pat: false,
            cmp: false,
            build: false,
            msg: false,
            reg_attr: false,
            reg_decl: false,
            arm_body: false,
            doc: false,
            consume: Consume::Identity,
        }
    }
}

struct LitRow {
    tree: String,
    class: Class,
    file: String,
    line: usize,
    text: String,
    why: String,
}

struct MapRow {
    tree: String,
    file: String,
    line: usize,
    owner: String,
    container: String,
    key: String,
    verdict: String,
    evidence: String,
}

struct SurfaceRow {
    tree: String,
    kind: String,
    consume: Consume,
    file: String,
    line: usize,
    call: String,
}

struct EqRow {
    tree: String,
    file: String,
    line: usize,
    op: String,
}

struct DoorRow {
    tree: String,
    door: String,
    consume: Consume,
    file: String,
    line: usize,
}

struct InsertEv {
    file: String,
    owner: String,
    evidence: String,
    name: bool,
    other: bool,
}

struct Acc {
    lits: Vec<LitRow>,
    maps: Vec<MapRow>,
    surface: Vec<SurfaceRow>,
    eqs: Vec<EqRow>,
    doors: Vec<DoorRow>,
    id_maps: Vec<MapRow>,
    pending_maps: Vec<PendingMap>,
    inserts: Vec<InsertEv>,
    parse_failures: Vec<String>,
}

struct PendingMap {
    tree: String,
    file: String,
    line: usize,
    owner: String,
    container: String,
    key: String,
    ident_key: bool,
}

struct FnState {
    scopes: Vec<BTreeMap<String, Taint>>,
    free_leaf: bool,
    free_path: bool,
}

impl FnState {
    fn new(free_leaf: bool, free_path: bool) -> Self {
        FnState {
            scopes: vec![BTreeMap::new()],
            free_leaf,
            free_path,
        }
    }

    fn push(&mut self) {
        self.scopes.push(BTreeMap::new());
    }

    fn pop(&mut self) {
        self.scopes.pop();
    }

    fn bind(&mut self, name: String, taint: Taint) {
        if let Some(top) = self.scopes.last_mut() {
            top.insert(name, taint);
        }
    }

    fn get(&self, name: &str) -> Option<Taint> {
        for scope in self.scopes.iter().rev() {
            if let Some(t) = scope.get(name) {
                return Some(*t);
            }
        }
        None
    }
}

fn main() -> ExitCode {
    let mut root = PathBuf::from(".");
    let mut out = PathBuf::from(
        "docs/arc/2026/06/255-builtin-registry/name-census.tsv",
    );
    let mut proof: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--root" => root = PathBuf::from(args.next().unwrap_or_default()),
            "--out" => out = PathBuf::from(args.next().unwrap_or_default()),
            "--proof" => proof = Some(PathBuf::from(args.next().unwrap_or_default())),
            "--help" => {
                println!("name_census [--root DIR] [--out FILE] [--proof FILE]");
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("unknown argument {other}");
                return ExitCode::from(2);
            }
        }
    }

    let mut acc = Acc {
        lits: Vec::new(),
        maps: Vec::new(),
        surface: Vec::new(),
        eqs: Vec::new(),
        doors: Vec::new(),
        id_maps: Vec::new(),
        pending_maps: Vec::new(),
        inserts: Vec::new(),
        parse_failures: Vec::new(),
    };

    for tree in ["src", "crates", "tests"] {
        let dir = root.join(tree);
        let mut files = Vec::new();
        collect_rs(&dir, &mut files);
        files.sort();
        for path in files {
            if path.ends_with("src/bin/name_census.rs") {
                continue;
            }
            analyze_path(&path, &root, tree, &mut acc);
        }
    }
    settle_maps(&mut acc);

    let proof_ok = if let Some(path) = &proof {
        match analyze_proof(path, &root) {
            Ok(()) => true,
            Err(msg) => {
                eprintln!("{msg}");
                false
            }
        }
    } else {
        true
    };

    if let Some(parent) = out.parent() {
        if !parent.as_os_str().is_empty() {
            let _ = fs::create_dir_all(parent);
        }
    }
    if let Err(e) = write_tsv(&out, &acc) {
        eprintln!("write {}: {e}", out.display());
        return ExitCode::from(1);
    }
    print_summary(&acc);
    print_pair_probe();
    if !acc.parse_failures.is_empty() {
        eprintln!("{} parse failures", acc.parse_failures.len());
        for f in &acc.parse_failures {
            eprintln!("PARSE {f}");
        }
        return ExitCode::from(1);
    }
    if proof_ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() {
            let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name == "target" || name == ".git" {
                continue;
            }
            collect_rs(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

fn analyze_path(path: &Path, root: &Path, tree: &str, acc: &mut Acc) {
    let Ok(text) = fs::read_to_string(path) else {
        acc.parse_failures
            .push(format!("read {}", path.display()));
        return;
    };
    let file = match syn::parse_file(&text) {
        Ok(f) => f,
        Err(e) => {
            acc.parse_failures
                .push(format!("{}: {e}", path.display()));
            return;
        }
    };
    let rel = path.strip_prefix(root).unwrap_or(path);
    let rel = rel.display().to_string();
    let (free_leaf, free_path) = free_fns(&file, &rel);
    let mut st = FnState::new(free_leaf, free_path);
    let ctx = Ctx {
        tree: tree.to_string(),
        file: rel,
    };
    for item in &file.items {
        walk_item(item, &ctx, &mut st, Site::neutral(), acc);
    }
}

struct Ctx {
    tree: String,
    file: String,
}

fn free_fns(file: &syn::File, rel: &str) -> (bool, bool) {
    let mut leaf = rel.ends_with("crates/wat-reader/src/identifier.rs");
    let mut path = leaf;
    for item in &file.items {
        if let Item::Use(u) = item {
            note_use(&u.tree, &mut leaf, &mut path);
        }
    }
    (leaf, path)
}

fn note_use(tree: &syn::UseTree, leaf: &mut bool, path: &mut bool) {
    match tree {
        syn::UseTree::Path(p) => note_use(&p.tree, leaf, path),
        syn::UseTree::Name(n) => {
            if n.ident == "leaf" {
                *leaf = true;
            }
            if n.ident == "path" {
                *path = true;
            }
        }
        syn::UseTree::Rename(n) => {
            if n.rename == "leaf" {
                *leaf = true;
            }
            if n.rename == "path" {
                *path = true;
            }
        }
        syn::UseTree::Group(g) => {
            for t in &g.items {
                note_use(t, leaf, path);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

fn walk_item(item: &Item, ctx: &Ctx, st: &mut FnState, site: Site, acc: &mut Acc) {
    match item {
        Item::Fn(f) => {
            for a in &f.attrs {
                walk_attr(a, ctx, acc);
            }
            st.push();
            for arg in &f.sig.inputs {
                if let syn::FnArg::Typed(t) = arg {
                    bind_pat(&t.pat, type_taint(&t.ty), st);
                    walk_type(&t.ty, ctx, owner_of_pat(&t.pat), acc);
                }
            }
            walk_block(&f.block, ctx, st, site, acc);
            st.pop();
        }
        Item::Impl(imp) => {
            for it in &imp.items {
                if let syn::ImplItem::Fn(f) = it {
                    for a in &f.attrs {
                        walk_attr(a, ctx, acc);
                    }
                    st.push();
                    for arg in &f.sig.inputs {
                        if let syn::FnArg::Typed(t) = arg {
                            bind_pat(&t.pat, type_taint(&t.ty), st);
                            walk_type(&t.ty, ctx, owner_of_pat(&t.pat), acc);
                        }
                    }
                    walk_block(&f.block, ctx, st, site.clone(), acc);
                    st.pop();
                }
            }
        }
        Item::Const(c) => {
            for a in &c.attrs {
                walk_attr(a, ctx, acc);
            }
            let mut s = site;
            s.reg_decl = true;
            walk_expr(&c.expr, ctx, st, s, acc);
            walk_type(&c.ty, ctx, c.ident.to_string(), acc);
        }
        Item::Static(c) => {
            let mut s = site;
            s.reg_decl = true;
            walk_expr(&c.expr, ctx, st, s, acc);
            walk_type(&c.ty, ctx, c.ident.to_string(), acc);
        }
        Item::Struct(s) => {
            for f in &s.fields {
                let owner = f
                    .ident
                    .as_ref()
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| s.ident.to_string());
                walk_type(&f.ty, ctx, owner, acc);
                for a in &f.attrs {
                    walk_attr(a, ctx, acc);
                }
            }
            for a in &s.attrs {
                walk_attr(a, ctx, acc);
            }
        }
        Item::Enum(e) => {
            for v in &e.variants {
                for f in &v.fields {
                    walk_type(
                        &f.ty,
                        ctx,
                        f.ident
                            .as_ref()
                            .map(|i| i.to_string())
                            .unwrap_or_else(|| v.ident.to_string()),
                        acc,
                    );
                }
                for a in &v.attrs {
                    walk_attr(a, ctx, acc);
                }
            }
        }
        Item::Type(t) => {
            walk_type(&t.ty, ctx, t.ident.to_string(), acc);
        }
        Item::Mod(m) => {
            if let Some((_, items)) = &m.content {
                for it in items {
                    walk_item(it, ctx, st, site.clone(), acc);
                }
            }
        }
        Item::Trait(t) => {
            for it in &t.items {
                if let syn::TraitItem::Fn(f) = it {
                    if let Some(b) = &f.default {
                        st.push();
                        walk_block(b, ctx, st, site.clone(), acc);
                        st.pop();
                    }
                    for arg in &f.sig.inputs {
                        if let syn::FnArg::Typed(pt) = arg {
                            walk_type(&pt.ty, ctx, owner_of_pat(&pt.pat), acc);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn walk_block(block: &syn::Block, ctx: &Ctx, st: &mut FnState, site: Site, acc: &mut Acc) {
    st.push();
    for stmt in &block.stmts {
        walk_stmt(stmt, ctx, st, site.clone(), acc);
    }
    st.pop();
}

fn walk_stmt(stmt: &Stmt, ctx: &Ctx, st: &mut FnState, site: Site, acc: &mut Acc) {
    match stmt {
        Stmt::Local(l) => {
            if let Some(init) = &l.init {
                let t = walk_expr(&init.expr, ctx, st, site, acc);
                bind_pat(&l.pat, t, st);
            } else {
                bind_pat(&l.pat, Taint::Other, st);
            }
            if let Pat::Type(t) = &l.pat {
                bind_pat(&t.pat, type_taint(&t.ty), st);
                walk_type(&t.ty, ctx, owner_of_pat(&t.pat), acc);
            }
        }
        Stmt::Item(it) => walk_item(it, ctx, st, site, acc),
        Stmt::Expr(e, _) => {
            walk_expr(e, ctx, st, site, acc);
        }
        Stmt::Macro(m) => walk_mac(&m.mac, ctx, st, site, acc),
    }
}

fn walk_expr(expr: &Expr, ctx: &Ctx, st: &mut FnState, site: Site, acc: &mut Acc) -> Taint {
    match expr {
        Expr::Lit(l) => {
            if let Lit::Str(s) = &l.lit {
                note_lit(ctx, s.value(), s.span().start().line, &site, acc);
            }
            Taint::Other
        }
        Expr::Array(a) => {
            for e in &a.elems {
                walk_expr(e, ctx, st, site.clone(), acc);
            }
            Taint::Other
        }
        Expr::Assign(a) => {
            walk_expr(&a.left, ctx, st, site.clone(), acc);
            walk_expr(&a.right, ctx, st, site, acc)
        }
        Expr::Binary(b) => {
            let cmp = matches!(b.op, syn::BinOp::Eq(_) | syn::BinOp::Ne(_));
            let mut child = site.clone();
            if cmp {
                child.cmp = true;
                child.consume = Consume::Identity;
            }
            let lt = walk_expr(&b.left, ctx, st, child.clone(), acc);
            let rt = walk_expr(&b.right, ctx, st, child, acc);
            if cmp && (lt == Taint::Identifier || rt == Taint::Identifier) {
                acc.eqs.push(EqRow {
                    tree: ctx.tree.clone(),
                    file: ctx.file.clone(),
                    line: b.span().start().line,
                    op: if matches!(b.op, syn::BinOp::Eq(_)) {
                        "==".into()
                    } else {
                        "!=".into()
                    },
                });
            }
            Taint::Other
        }
        Expr::Block(b) => {
            walk_block(&b.block, ctx, st, site, acc);
            Taint::Other
        }
        Expr::Call(c) => walk_call(c, ctx, st, site, acc),
        Expr::Cast(c) => walk_expr(&c.expr, ctx, st, site, acc),
        Expr::Closure(c) => {
            st.push();
            for p in &c.inputs {
                bind_pat(p, Taint::Other, st);
            }
            let t = match &*c.body {
                Expr::Block(b) => {
                    walk_block(&b.block, ctx, st, site, acc);
                    Taint::Other
                }
                other => walk_expr(other, ctx, st, site, acc),
            };
            st.pop();
            t
        }
        Expr::Field(f) => {
            let t = walk_expr(&f.base, ctx, st, site.clone(), acc);
            if t == Taint::Identifier {
                if let syn::Member::Named(id) = &f.member {
                    if SURFACE_METHODS.contains(&id.to_string().as_str()) {
                        acc.surface.push(SurfaceRow {
                            tree: ctx.tree.clone(),
                            kind: format!("field.{}", id),
                            consume: site.consume,
                            file: ctx.file.clone(),
                            line: f.span().start().line,
                            call: id.to_string(),
                        });
                    }
                }
            }
            Taint::Other
        }
        Expr::ForLoop(f) => {
            walk_expr(&f.expr, ctx, st, site.clone(), acc);
            walk_block(&f.body, ctx, st, site, acc);
            Taint::Other
        }
        Expr::If(i) => {
            let mut cond = site.clone();
            cond.cmp = true;
            walk_expr(&i.cond, ctx, st, cond, acc);
            walk_block(&i.then_branch, ctx, st, site.clone(), acc);
            if let Some((_, e)) = &i.else_branch {
                walk_expr(e, ctx, st, site, acc);
            }
            Taint::Other
        }
        Expr::Index(i) => {
            walk_expr(&i.expr, ctx, st, site.clone(), acc);
            let mut key = site;
            key.consume = Consume::Identity;
            walk_expr(&i.index, ctx, st, key, acc);
            Taint::Other
        }
        Expr::Let(l) => {
            let t = walk_expr(&l.expr, ctx, st, site.clone(), acc);
            let mut ps = site;
            ps.pat = true;
            walk_pat_lits(&l.pat, ctx, &ps, acc);
            bind_pat(&l.pat, t, st);
            Taint::Other
        }
        Expr::Loop(l) => {
            walk_block(&l.body, ctx, st, site, acc);
            Taint::Other
        }
        Expr::Match(m) => {
            let scrut_t = walk_expr(&m.expr, ctx, st, site.clone(), acc);
            for arm in &m.arms {
                st.push();
                bind_pat_from(&arm.pat, scrut_t, st);
                let mut ps = site.clone();
                ps.pat = true;
                walk_pat_lits(&arm.pat, ctx, &ps, acc);
                if let Some((_, g)) = &arm.guard {
                    let mut gs = site.clone();
                    gs.cmp = true;
                    walk_expr(g, ctx, st, gs, acc);
                }
                let mut body = site.clone();
                body.arm_body = true;
                walk_expr(&arm.body, ctx, st, body, acc);
                st.pop();
            }
            Taint::Other
        }
        Expr::MethodCall(m) => walk_method(m, ctx, st, site, acc),
        Expr::Paren(p) => walk_expr(&p.expr, ctx, st, site, acc),
        Expr::Path(p) => path_taint(&p.path, st),
        Expr::Reference(r) => walk_expr(&r.expr, ctx, st, site, acc),
        Expr::Repeat(r) => walk_expr(&r.expr, ctx, st, site, acc),
        Expr::Return(r) => {
            if let Some(e) = &r.expr {
                walk_expr(e, ctx, st, site, acc);
            }
            Taint::Other
        }
        Expr::Struct(s) => {
            for f in &s.fields {
                walk_expr(&f.expr, ctx, st, site.clone(), acc);
            }
            if let Some(rest) = &s.rest {
                walk_expr(rest, ctx, st, site, acc);
            }
            Taint::Other
        }
        Expr::Try(t) => walk_expr(&t.expr, ctx, st, site, acc),
        Expr::Tuple(t) => {
            for e in &t.elems {
                walk_expr(e, ctx, st, site.clone(), acc);
            }
            Taint::Other
        }
        Expr::Unary(u) => walk_expr(&u.expr, ctx, st, site, acc),
        Expr::Unsafe(u) => {
            walk_block(&u.block, ctx, st, site, acc);
            Taint::Other
        }
        Expr::While(w) => {
            let mut cond = site.clone();
            cond.cmp = true;
            walk_expr(&w.cond, ctx, st, cond, acc);
            walk_block(&w.body, ctx, st, site, acc);
            Taint::Other
        }
        Expr::Macro(m) => {
            walk_mac(&m.mac, ctx, st, site, acc);
            Taint::Other
        }
        _ => Taint::Other,
    }
}

fn walk_call(
    c: &syn::ExprCall,
    ctx: &Ctx,
    st: &mut FnState,
    site: Site,
    acc: &mut Acc,
) -> Taint {
    let name = expr_callee(&c.func);
    if let Some(door) = name.as_deref().filter(|n| DOORS.contains(n)) {
        acc.doors.push(DoorRow {
            tree: ctx.tree.clone(),
            door: door.to_string(),
            consume: site.consume,
            file: ctx.file.clone(),
            line: c.span().start().line,
            });
    }
    if let Some(n) = name.as_deref() {
        if (n == "leaf" && st.free_leaf) || (n == "path" && st.free_path) {
            acc.surface.push(SurfaceRow {
                tree: ctx.tree.clone(),
                kind: format!("free.{n}"),
                consume: site.consume,
                file: ctx.file.clone(),
                line: c.span().start().line,
                call: n.to_string(),
            });
        }
    }
    let mut child = site.clone();
    if name.as_deref().is_some_and(|n| DOORS.contains(&n)) {
        child.consume = Consume::Identity;
    }
    walk_expr(&c.func, ctx, st, site.clone(), acc);
    for a in &c.args {
        walk_expr(a, ctx, st, child.clone(), acc);
    }
    if name.as_deref().is_some_and(|n| {
        n == "bare" || n == "into_bound" || n == "add_scope" || n == "Identifier"
    }) {
        Taint::Identifier
    } else {
        Taint::Other
    }
}

fn walk_method(
    m: &syn::ExprMethodCall,
    ctx: &Ctx,
    st: &mut FnState,
    site: Site,
    acc: &mut Acc,
) -> Taint {
    let method = m.method.to_string();
    let recv_t = walk_expr(&m.receiver, ctx, st, site.clone(), acc);
    if recv_t == Taint::Identifier && SURFACE_METHODS.contains(&method.as_str()) {
        acc.surface.push(SurfaceRow {
            tree: ctx.tree.clone(),
            kind: format!("method.{method}"),
            consume: site.consume,
            file: ctx.file.clone(),
            line: m.span().start().line,
            call: method.clone(),
        });
    }
    if method == "insert" || method == "entry" {
        if let Some(key) = m.args.first() {
            let (name, other, ev) = expr_key_evidence(key);
            if let Some(owner) = receiver_owner(&m.receiver) {
                acc.inserts.push(InsertEv {
                    file: ctx.file.clone(),
                    owner,
                    evidence: format!("{method} {ev}"),
                    name,
                    other,
                });
            }
        }
    }
    let mut child = site;
    if CMP_METHODS.contains(&method.as_str()) {
        child.cmp = true;
        child.consume = Consume::Identity;
    } else if method == "expect" || method == "unwrap_or_else" {
        child.msg = true;
        child.consume = Consume::Print;
    } else if method == "push_str" || method == "write_str" {
        child.consume = Consume::Print;
        child.msg = true;
    } else if method == "insert" || method == "entry" {
        child.reg_decl = true;
        child.consume = Consume::Identity;
    }
    for (i, a) in m.args.iter().enumerate() {
        let mut one = child.clone();
        if method == "expect" && i == 0 {
            one.msg = true;
        }
        if (method == "insert" || method == "entry") && i == 0 {
            one.reg_decl = true;
        }
        if method == "unwrap_or_else" {
            one.msg = false;
            one.reg_decl = false;
        }
        walk_expr(a, ctx, st, one, acc);
    }
    if recv_t == Taint::Identifier
        && matches!(method.as_str(), "clone" | "add_scope" | "into_bound" | "bare")
    {
        Taint::Identifier
    } else {
        Taint::Other
    }
}

fn walk_mac(mac: &syn::Macro, ctx: &Ctx, st: &mut FnState, site: Site, acc: &mut Acc) {
    let name = mac
        .path
        .segments
        .last()
        .map(|s| s.ident.to_string())
        .unwrap_or_default();
    let args = split_args(mac.tokens.clone());
    if name == "concat" {
        let mut b = site;
        b.build = true;
        for a in &args {
            walk_arg_tokens(a, ctx, st, b.clone(), acc);
        }
        return;
    }
    if matches!(name.as_str(), "format" | "format_args") {
        for (i, a) in args.iter().enumerate() {
            let mut b = site.clone();
            if i == 0 {
                b.build = true;
            } else {
                b.consume = Consume::Identity;
            }
            walk_arg_tokens(a, ctx, st, b, acc);
        }
        return;
    }
    if matches!(name.as_str(), "assert_eq" | "assert_ne") {
        let mut b = site.clone();
        b.cmp = true;
        b.consume = Consume::Identity;
        for a in args.iter().take(2) {
            walk_arg_tokens(a, ctx, st, b.clone(), acc);
        }
        let mut msg = site;
        msg.msg = true;
        msg.consume = Consume::Print;
        for a in args.iter().skip(2) {
            walk_arg_tokens(a, ctx, st, msg.clone(), acc);
        }
        return;
    }
    if PRINT_MACROS.contains(&name.as_str()) || name == "assert" || name == "debug_assert" {
        for (i, a) in args.iter().enumerate() {
            let mut b = site.clone();
            if (name == "assert" || name == "debug_assert") && i == 0 {
                b.cmp = true;
            } else if name == "write" || name == "writeln" {
                if i >= 2 {
                    b.msg = true;
                    b.consume = Consume::Print;
                }
                if i == 1 {
                    b.build = true;
                }
            } else {
                b.msg = true;
                b.consume = Consume::Print;
            }
            walk_arg_tokens(a, ctx, st, b, acc);
        }
        return;
    }
    for a in &args {
        walk_arg_tokens(a, ctx, st, site.clone(), acc);
    }
}

fn walk_arg_tokens(
    tokens: &[proc_macro2::TokenTree],
    ctx: &Ctx,
    st: &mut FnState,
    site: Site,
    acc: &mut Acc,
) {
    let stream = tokens.iter().cloned().collect::<proc_macro2::TokenStream>();
    if let Ok(expr) = syn::parse2::<Expr>(stream.clone()) {
        walk_expr(&expr, ctx, st, site, acc);
        return;
    }
    for tt in tokens {
        if let proc_macro2::TokenTree::Literal(lit) = tt {
            let raw = lit.to_string();
            if let Some(text) = unquote(&raw) {
                if text.contains("::") {
                    note_lit(ctx, text, lit.span().start().line, &site, acc);
                }
            }
        } else if let proc_macro2::TokenTree::Group(g) = tt {
            let inner: Vec<_> = g.stream().into_iter().collect();
            walk_arg_tokens(&inner, ctx, st, site.clone(), acc);
        }
    }
}

fn note_lit(ctx: &Ctx, text: String, line: usize, site: &Site, acc: &mut Acc) {
    if !text.contains("::") {
        return;
    }
    let (class, why) = classify(&text, site);
    acc.lits.push(LitRow {
        tree: ctx.tree.clone(),
        class,
        file: ctx.file.clone(),
        line,
        text,
        why: why.to_string(),
    });
}

fn classify(text: &str, site: &Site) -> (Class, &'static str) {
    if is_embedded_wat(text) {
        return (Class::Wat, "embedded wat source");
    }
    if site.pat {
        return (Class::Dispatch, "match or if-let pattern");
    }
    if site.cmp {
        return (Class::Cmp, "equality, prefix, contains, or membership");
    }
    if site.build && assembles_name(text) {
        return (Class::Build, "format or concat assembles a name");
    }
    if site.reg_attr {
        return (Class::Reg, "registration attribute");
    }
    if site.reg_decl && is_nameish(text) {
        return (Class::Reg, "declared name or registry insert");
    }
    if site.msg {
        return (Class::Msg, "diagnostic or panic text");
    }
    if site.doc {
        return (Class::Other, "doc comment");
    }
    if site.arm_body && !is_nameish(text) {
        return (Class::Msg, "match arm text is not compared");
    }
    if site.arm_body {
        return (Class::Other, "match arm yields a name");
    }
    if site.build {
        return (Class::Msg, "format text that does not assemble a name");
    }
    (Class::Other, "no registration, dispatch, compare, build, message, or wat form")
}

fn is_embedded_wat(text: &str) -> bool {
    let form = text.contains("(wat.")
        || text.contains("(wat/")
        || text.contains("wat.core/")
        || text.contains("(defn")
        || text.contains("(defrecord")
        || text.contains("(defenum")
        || text.contains("\n(def");
    form && (text.contains('\n') || text.contains("(wat."))
}

fn assembles_name(text: &str) -> bool {
    if text.contains('{') && text.contains("::") {
        return true;
    }
    is_nameish(text)
}

fn is_nameish(text: &str) -> bool {
    let t = text.trim();
    !t.is_empty()
        && !t.contains(' ')
        && !t.contains('\n')
        && t.contains("::")
        && t.chars().all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, ':' | '/' | '.' | '_' | '-' | '\'' | '<' | '>' | '{' | '}' | '$')
        })
}

fn walk_pat_lits(pat: &Pat, ctx: &Ctx, site: &Site, acc: &mut Acc) {
    match pat {
        Pat::Lit(l) => {
            if let Lit::Str(s) = &l.lit {
                note_lit(ctx, s.value(), s.span().start().line, site, acc);
            }
        }
        Pat::Or(o) => {
            for c in &o.cases {
                walk_pat_lits(c, ctx, site, acc);
            }
        }
        Pat::Paren(p) => walk_pat_lits(&p.pat, ctx, site, acc),
        Pat::Reference(p) => walk_pat_lits(&p.pat, ctx, site, acc),
        Pat::Slice(s) => {
            for e in &s.elems {
                walk_pat_lits(e, ctx, site, acc);
            }
        }
        Pat::Tuple(t) => {
            for e in &t.elems {
                walk_pat_lits(e, ctx, site, acc);
            }
        }
        Pat::TupleStruct(t) => {
            for e in &t.elems {
                walk_pat_lits(e, ctx, site, acc);
            }
        }
        _ => {}
    }
}

fn bind_pat(pat: &Pat, taint: Taint, st: &mut FnState) {
    match pat {
        Pat::Ident(i) => st.bind(i.ident.to_string(), taint),
        Pat::Type(t) => bind_pat(&t.pat, type_taint(&t.ty), st),
        Pat::Reference(r) => bind_pat(&r.pat, taint, st),
        Pat::Tuple(t) => {
            for e in &t.elems {
                bind_pat(e, Taint::Other, st);
            }
        }
        Pat::TupleStruct(t) => {
            let sym = path_last(&t.path) == "Symbol"
                && t.path.segments.iter().any(|s| s.ident == "WatAST");
            for (i, e) in t.elems.iter().enumerate() {
                let one = if sym && i == 0 {
                    Taint::Identifier
                } else {
                    Taint::Other
                };
                bind_pat(e, one, st);
            }
        }
        Pat::Struct(s) => {
            for f in &s.fields {
                bind_pat(&f.pat, Taint::Other, st);
            }
        }
        _ => {}
    }
}

fn bind_pat_from(pat: &Pat, scrut: Taint, st: &mut FnState) {
    let t = if scrut == Taint::Identifier {
        Taint::Identifier
    } else {
        Taint::Other
    };
    bind_pat(pat, t, st);
}

fn type_taint(ty: &Type) -> Taint {
    if type_last(ty).as_deref() == Some("Identifier") {
        Taint::Identifier
    } else {
        Taint::Other
    }
}

fn path_taint(path: &syn::Path, st: &FnState) -> Taint {
    if path.segments.len() == 1 {
        let name = path.segments[0].ident.to_string();
        return st.get(&name).unwrap_or(Taint::Other);
    }
    if path_last(path) == "Identifier" {
        return Taint::Identifier;
    }
    Taint::Other
}

fn walk_type(ty: &Type, ctx: &Ctx, owner: String, acc: &mut Acc) {
    match ty {
        Type::Path(p) => {
            if let Some(seg) = p.path.segments.last() {
                let container = seg.ident.to_string();
                if TEXT_CONTAINERS.contains(&container.as_str()) {
                    if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
                        if let Some(key_ty) = ab.args.iter().find_map(|a| match a {
                            syn::GenericArgument::Type(t) => Some(t),
                            _ => None,
                        }) {
                            let ident_key = type_last(key_ty).as_deref() == Some("Identifier");
                            if ident_key || is_text_key(key_ty) {
                                acc.pending_maps.push(PendingMap {
                                    tree: ctx.tree.clone(),
                                    file: ctx.file.clone(),
                                    line: ty.span().start().line,
                                    owner: owner.clone(),
                                    container,
                                    key: type_sketch(key_ty),
                                    ident_key,
                                });
                            }
                            for a in &ab.args {
                                if let syn::GenericArgument::Type(t) = a {
                                    walk_type(t, ctx, owner.clone(), acc);
                                }
                            }
                            return;
                        }
                    }
                }
                if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
                    for a in &ab.args {
                        if let syn::GenericArgument::Type(t) = a {
                            walk_type(t, ctx, owner.clone(), acc);
                        }
                    }
                }
            }
        }
        Type::Reference(r) => walk_type(&r.elem, ctx, owner, acc),
        Type::Tuple(t) => {
            for e in &t.elems {
                walk_type(e, ctx, owner.clone(), acc);
            }
        }
        Type::Slice(s) => walk_type(&s.elem, ctx, owner, acc),
        Type::Array(a) => walk_type(&a.elem, ctx, owner, acc),
        Type::Ptr(p) => walk_type(&p.elem, ctx, owner, acc),
        Type::Group(g) => walk_type(&g.elem, ctx, owner, acc),
        Type::Paren(p) => walk_type(&p.elem, ctx, owner, acc),
        Type::BareFn(f) => {
            for a in &f.inputs {
                walk_type(&a.ty, ctx, owner.clone(), acc);
            }
            if let syn::ReturnType::Type(_, t) = &f.output {
                walk_type(t, ctx, owner, acc);
            }
        }
        _ => {}
    }
}

fn is_text_key(ty: &Type) -> bool {
    match ty {
        Type::Reference(r) => is_text_key(&r.elem) || type_last(&r.elem).as_deref() == Some("str"),
        Type::Path(p) => {
            let n = p.path.segments.last().map(|s| s.ident.to_string());
            match n.as_deref() {
                Some("String") | Some("str") => true,
                Some("Arc") => p.path.segments.last().is_some_and(|s| {
                    if let syn::PathArguments::AngleBracketed(ab) = &s.arguments {
                        ab.args.iter().any(|a| {
                            matches!(a, syn::GenericArgument::Type(t) if type_last(t).as_deref() == Some("str") || type_last(t).as_deref() == Some("String"))
                        })
                    } else {
                        false
                    }
                }),
                _ => false,
            }
        }
        _ => false,
    }
}

fn type_last(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
        Type::Reference(r) => type_last(&r.elem),
        _ => None,
    }
}

fn type_sketch(ty: &Type) -> String {
    match ty {
        Type::Reference(r) => format!("&{}", type_sketch(&r.elem)),
        Type::Path(p) => {
            let n = p
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();
            if let Some(seg) = p.path.segments.last() {
                if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
                    let args: Vec<_> = ab
                        .args
                        .iter()
                        .filter_map(|a| match a {
                            syn::GenericArgument::Type(t) => Some(type_sketch(t)),
                            _ => None,
                        })
                        .collect();
                    if !args.is_empty() {
                        return format!("{n}<{}>", args.join(", "));
                    }
                }
            }
            n
        }
        _ => "type".into(),
    }
}

fn walk_attr(attr: &syn::Attribute, ctx: &Ctx, acc: &mut Acc) {
    let name = attr
        .path()
        .segments
        .last()
        .map(|s| s.ident.to_string())
        .unwrap_or_default();
    if name == "doc" {
        if let syn::Meta::NameValue(nv) = &attr.meta {
            if let Expr::Lit(l) = &nv.value {
                if let Lit::Str(s) = &l.lit {
                    let mut site = Site::neutral();
                    site.doc = true;
                    note_lit(ctx, s.value(), s.span().start().line, &site, acc);
                }
            }
        }
        return;
    }
    let reg = name.starts_with("wat_");
    let mut site = Site::neutral();
    site.reg_attr = reg;
    let tokens = match &attr.meta {
        syn::Meta::List(list) => list.tokens.clone(),
        syn::Meta::NameValue(nv) => {
            if let Expr::Lit(l) = &nv.value {
                if let Lit::Str(s) = &l.lit {
                    note_lit(ctx, s.value(), s.span().start().line, &site, acc);
                }
            }
            return;
        }
        syn::Meta::Path(_) => return,
    };
    let dummy = FnState::new(false, false);
    // Attributes are not inside a function body. A fresh state is enough:
    // the literals are what this walk is after.
    let mut st = dummy;
    let args = split_args(tokens);
    for a in &args {
        walk_arg_tokens(a, ctx, &mut st, site.clone(), acc);
    }
}

fn settle_maps(acc: &mut Acc) {
    let inserts = std::mem::take(&mut acc.inserts);
    let pending = std::mem::take(&mut acc.pending_maps);
    for m in pending {
        let hits: Vec<&InsertEv> = inserts
            .iter()
            .filter(|ins| ins.file == m.file && owner_matches(&ins.owner, &m.owner))
            .collect();
        let (verdict, evidence) = if hits.is_empty() {
            (
                "STOP",
                "no insert or entry in this file names this binding".to_string(),
            )
        } else {
            let name = hits.iter().any(|h| h.name);
            let other = hits.iter().any(|h| h.other);
            let ev = hits
                .iter()
                .map(|h| h.evidence.as_str())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
                .join("; ");
            if name && other {
                ("STOP", format!("name evidence and other-text evidence: {ev}"))
            } else if name {
                ("NAME", ev)
            } else if other {
                ("OTHER", ev)
            } else {
                ("STOP", format!("insert seen, key is neither a name nor other text: {ev}"))
            }
        };
        let row = MapRow {
            tree: m.tree,
            file: m.file,
            line: m.line,
            owner: m.owner,
            container: m.container,
            key: m.key,
            verdict: verdict.to_string(),
            evidence,
        };
        if m.ident_key {
            acc.id_maps.push(row);
        } else {
            acc.maps.push(row);
        }
    }
}

fn owner_matches(insert_owner: &str, map_owner: &str) -> bool {
    insert_owner == map_owner
        || insert_owner
            .strip_prefix("self.")
            .is_some_and(|rest| rest == map_owner)
        || insert_owner.ends_with(&format!(".{map_owner}"))
}

fn expr_key_evidence(expr: &Expr) -> (bool, bool, String) {
    let mut name = false;
    let mut other = false;
    let mut why = String::from("key");
    fn walk(e: &Expr, name: &mut bool, other: &mut bool, why: &mut String) {
        match e {
            Expr::Lit(l) => {
                if let Lit::Str(s) = &l.lit {
                    let v = s.value();
                    if v.contains("::") || v.starts_with(':') {
                        *name = true;
                        *why = ":: literal".to_string();
                    } else if v.contains('/') && (v.contains('.') || v.starts_with('/')) {
                        *other = true;
                        *why = "path literal".to_string();
                    }
                }
            }
            Expr::Call(c) => {
                if let Some(n) = expr_callee(&c.func) {
                    if DOORS.contains(&n.as_str())
                        || n == "as_str"
                        || n == "leaf"
                        || n == "path"
                        || n == "to_string"
                        || n == "bare"
                        || n == "into_bound"
                        || n == "add_scope"
                    {
                        *name = true;
                        *why = n;
                    }
                }
                for a in &c.args {
                    walk(a, name, other, why);
                }
            }
            Expr::MethodCall(m) => {
                let method = m.method.to_string();
                if SURFACE_METHODS.contains(&method.as_str()) || method == "to_string" {
                    *name = true;
                    *why = method;
                }
                walk(&m.receiver, name, other, why);
                for a in &m.args {
                    walk(a, name, other, why);
                }
            }
            Expr::Path(p) => {
                if let Some(id) = p.path.get_ident() {
                    let n = id.to_string();
                    *why = format!("var {n}");
                    if matches!(
                        n.as_str(),
                        "path"
                            | "file"
                            | "dir"
                            | "label"
                            | "msg"
                            | "message"
                            | "text"
                            | "filename"
                    ) {
                        *other = true;
                    }
                }
            }
            Expr::Reference(r) => walk(&r.expr, name, other, why),
            Expr::Paren(p) => walk(&p.expr, name, other, why),
            Expr::Field(f) => walk(&f.base, name, other, why),
            Expr::Index(i) => {
                walk(&i.expr, name, other, why);
                walk(&i.index, name, other, why);
            }
            Expr::Binary(b) => {
                walk(&b.left, name, other, why);
                walk(&b.right, name, other, why);
            }
            _ => {}
        }
    }
    walk(expr, &mut name, &mut other, &mut why);
    (name, other, why)
}

fn receiver_owner(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Path(p) => p.path.get_ident().map(|i| i.to_string()),
        Expr::Field(f) => {
            let base = receiver_owner(&f.base).unwrap_or_default();
            let member = match &f.member {
                syn::Member::Named(i) => i.to_string(),
                syn::Member::Unnamed(n) => n.index.to_string(),
            };
            if base.is_empty() {
                Some(member)
            } else {
                Some(format!("{base}.{member}"))
            }
        }
        Expr::Reference(r) => receiver_owner(&r.expr),
        Expr::Paren(p) => receiver_owner(&p.expr),
        _ => None,
    }
}

fn expr_callee(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
        Expr::Paren(p) => expr_callee(&p.expr),
        _ => None,
    }
}

fn path_last(path: &syn::Path) -> String {
    path.segments
        .last()
        .map(|s| s.ident.to_string())
        .unwrap_or_default()
}

fn owner_of_pat(pat: &Pat) -> String {
    match pat {
        Pat::Ident(i) => i.ident.to_string(),
        Pat::Type(t) => owner_of_pat(&t.pat),
        Pat::Reference(r) => owner_of_pat(&r.pat),
        _ => "pat".into(),
    }
}

fn split_args(tokens: proc_macro2::TokenStream) -> Vec<Vec<proc_macro2::TokenTree>> {
    let mut args = Vec::new();
    let mut cur = Vec::new();
    let mut depth = 0;
    for tt in tokens {
        match &tt {
            proc_macro2::TokenTree::Punct(p) if p.as_char() == ',' && depth == 0 => {
                args.push(std::mem::take(&mut cur));
            }
            proc_macro2::TokenTree::Group(g) => {
                depth += 1;
                cur.push(tt.clone());
                let _ = g;
                depth -= 1;
            }
            _ => cur.push(tt),
        }
    }
    if !cur.is_empty() {
        args.push(cur);
    }
    args
}

fn unquote(raw: &str) -> Option<String> {
    let t = raw.trim();
    if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') {
        return Some(t[1..t.len() - 1].to_string());
    }
    if let Some(rest) = t.strip_prefix("r") {
        if let Some(start) = rest.find('"') {
            let hashes = &rest[..start];
            if hashes.chars().all(|c| c == '#') {
                let body = &rest[start + 1..];
                let end = format!("\"{hashes}");
                if let Some(cut) = body.rfind(&end) {
                    return Some(body[..cut].to_string());
                }
            }
        }
    }
    None
}

fn write_tsv(path: &Path, acc: &Acc) -> std::io::Result<()> {
    let mut s = String::new();
    s.push_str("section\ttree\tclass\tfile\tline\tkey\tdetail\n");
    for r in &acc.lits {
        s.push_str(&format!(
            "LIT\t{}\t{}\t{}\t{}\t{}\t{}\n",
            r.tree,
            r.class.tag(),
            r.file,
            r.line,
            tsv(r.why.as_str()),
            tsv(&r.text)
        ));
    }
    for r in &acc.maps {
        s.push_str(&format!(
            "MAP\t{}\t{}\t{}\t{}\t{} {}\t{}\n",
            r.tree,
            r.verdict,
            r.file,
            r.line,
            r.container,
            tsv(&r.key),
            tsv(&format!("{} | {}", r.owner, r.evidence))
        ));
    }
    for r in &acc.surface {
        let class = match r.consume {
            Consume::Identity => "IDENTITY",
            Consume::Print => "PRINT",
        };
        s.push_str(&format!(
            "SURFACE\t{}\t{}\t{}\t{}\t{}\t{}\n",
            r.tree,
            class,
            r.file,
            r.line,
            r.kind,
            tsv(&r.call)
        ));
    }
    for r in &acc.eqs {
        s.push_str(&format!(
            "EQ\t{}\t==\t{}\t{}\t{}\tIdentifier\n",
            r.tree, r.file, r.line, r.op
        ));
    }
    for r in &acc.id_maps {
        s.push_str(&format!(
            "IDMAP\t{}\t{}\t{}\t{}\t{} {}\t{}\n",
            r.tree,
            r.verdict,
            r.file,
            r.line,
            r.container,
            tsv(&r.key),
            tsv(&format!("{} | {}", r.owner, r.evidence))
        ));
    }
    for r in &acc.doors {
        let class = match r.consume {
            Consume::Identity => "IDENTITY",
            Consume::Print => "PRINT",
        };
        s.push_str(&format!(
            "DOOR\t{}\t{}\t{}\t{}\t{}\tdoor\n",
            r.tree, class, r.file, r.line, r.door
        ));
    }
    fs::write(path, s)
}

fn tsv(s: &str) -> String {
    s.replace(['\t', '\n', '\r'], " ")
}

fn print_summary(acc: &Acc) {
    println!("PARSE_FAIL {}", acc.parse_failures.len());
    for tree in ["src", "crates", "tests"] {
        let rows: Vec<&LitRow> = acc.lits.iter().filter(|r| r.tree == tree).collect();
        println!("LITERALS {tree} {}", rows.len());
        for class in [
            Class::Reg,
            Class::Dispatch,
            Class::Cmp,
            Class::Build,
            Class::Msg,
            Class::Wat,
            Class::Other,
        ] {
            let n = rows.iter().filter(|r| r.class == class).count();
            println!("CLASS {tree} {} {n}", class.tag());
        }
        let mut files: BTreeMap<&str, usize> = BTreeMap::new();
        let mut prefix: BTreeMap<String, usize> = BTreeMap::new();
        for r in &rows {
            *files.entry(r.file.as_str()).or_default() += 1;
            *prefix.entry(prefix_of(&r.text)).or_default() += 1;
        }
        let mut frank: Vec<_> = files.into_iter().collect();
        frank.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        for (f, n) in frank.iter().take(20) {
            println!("TOPFILE {tree} {n} {f}");
        }
        let mut pr: Vec<_> = prefix.into_iter().collect();
        pr.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        for (p, n) in pr.iter().take(20) {
            println!("PREFIX {tree} {n} {p}");
        }
        let maps: Vec<&MapRow> = acc.maps.iter().filter(|m| m.tree == tree).collect();
        println!("MAPS {tree} {}", maps.len());
        for verdict in ["NAME", "OTHER", "STOP"] {
            let n = maps.iter().filter(|m| m.verdict == verdict).count();
            println!("MAPCLASS {tree} {verdict} {n}");
        }
        let surf: Vec<&SurfaceRow> = acc.surface.iter().filter(|s| s.tree == tree).collect();
        println!("SURFACE {tree} {}", surf.len());
        for consume in [Consume::Identity, Consume::Print] {
            let n = surf.iter().filter(|s| s.consume == consume).count();
            let tag = match consume {
                Consume::Identity => "IDENTITY",
                Consume::Print => "PRINT",
            };
            println!("SURFACECLASS {tree} {tag} {n}");
        }
        let mut sfiles: BTreeMap<&str, usize> = BTreeMap::new();
        for s in &surf {
            *sfiles.entry(s.file.as_str()).or_default() += 1;
        }
        let mut srank: Vec<_> = sfiles.into_iter().collect();
        srank.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        for (f, n) in srank.iter().take(20) {
            println!("SURFACETOP {tree} {n} {f}");
        }
        println!(
            "EQ {tree} {}",
            acc.eqs.iter().filter(|e| e.tree == tree).count()
        );
        println!(
            "IDMAP {tree} {}",
            acc.id_maps.iter().filter(|e| e.tree == tree).count()
        );
        let doors: Vec<&DoorRow> = acc.doors.iter().filter(|d| d.tree == tree).collect();
        println!("DOORS {tree} {}", doors.len());
        for door in DOORS {
            for consume in [Consume::Identity, Consume::Print] {
                let n = doors
                    .iter()
                    .filter(|d| d.door == *door && d.consume == consume)
                    .count();
                let tag = match consume {
                    Consume::Identity => "IDENTITY",
                    Consume::Print => "PRINT",
                };
                println!("DOOR {tree} {door} {tag} {n}");
            }
        }
    }
}

fn prefix_of(text: &str) -> String {
    let Some(idx) = text.find("::") else {
        return "none".into();
    };
    let bytes = text.as_bytes();
    let mut start = idx;
    while start > 0 {
        let c = bytes[start - 1] as char;
        if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '$' || c == ':' {
            start -= 1;
        } else {
            break;
        }
    }
    let mut seg = text[start..idx].trim_matches(':').to_string();
    if seg.is_empty() {
        seg = "empty".into();
    }
    seg
}

fn print_pair_probe() {
    let samples = [
        ":a::b/c",
        "a.b/c",
        "wat.core.Option/expect",
        ":wat::core::Option/expect",
        "wat.core//",
        "wat.core/x",
    ];
    println!("PAIR samples {}", samples.len());
    let mut ids = Vec::new();
    for s in samples {
        let id = wat_reader::Identifier::bare(s);
        println!(
            "PAIR spell {:?} ns {:?} name {:?} flat {:?} ref {}",
            s,
            id.namespace(),
            id.method(),
            id.as_str(),
            id.is_reference()
        );
        ids.push(id);
    }
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            let flat_eq = ids[i] == ids[j];
            let pair_eq = ids[i].namespace() == ids[j].namespace()
                && ids[i].method() == ids[j].method();
            if flat_eq != pair_eq {
                println!(
                    "PAIR DISAGREE {:?} {:?} flat_eq {flat_eq} pair_eq {pair_eq}",
                    samples[i], samples[j]
                );
            }
        }
    }
    let r = wat_reader::Identifier::bare("wat.core/x");
    let b = r.clone().into_bound();
    let flat_eq = r == b;
    let pair_eq = r.namespace() == b.namespace() && r.method() == b.method();
    println!(
        "PAIR into_bound flat_eq {flat_eq} pair_eq {pair_eq} bound_ns {:?} bound_name {:?} flat {:?}",
        b.namespace(),
        b.method(),
        b.as_str()
    );
}

fn analyze_proof(path: &Path, root: &Path) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|e| format!("proof read: {e}"))?;
    let file = syn::parse_file(&text).map_err(|e| format!("proof parse: {e}"))?;
    let mut acc = Acc {
        lits: Vec::new(),
        maps: Vec::new(),
        surface: Vec::new(),
        eqs: Vec::new(),
        doors: Vec::new(),
        id_maps: Vec::new(),
        pending_maps: Vec::new(),
        inserts: Vec::new(),
        parse_failures: Vec::new(),
    };
    let rel = path.strip_prefix(root).unwrap_or(path).display().to_string();
    let ctx = Ctx {
        tree: "proof".into(),
        file: rel,
    };
    let mut st = FnState::new(false, false);
    for item in &file.items {
        walk_item(item, &ctx, &mut st, Site::neutral(), &mut acc);
    }
    let expect = [
        (":wat::census::Reg", Class::Reg),
        (":wat::census::RegAttr", Class::Reg),
        (":wat::census::Dispatch", Class::Dispatch),
        (":wat::census::CmpEq", Class::Cmp),
        (":wat::census::CmpPre", Class::Cmp),
        (":wat::census::{}", Class::Build),
        ("missing :wat::census::Msg here", Class::Msg),
        (":wat::census::Wat", Class::Wat),
        ("std::census::Other", Class::Other),
    ];
    for (needle, class) in expect {
        let hits: Vec<_> = acc
            .lits
            .iter()
            .filter(|r| {
                if needle == ":wat::census::Wat" {
                    r.text.contains(needle)
                } else {
                    r.text == needle
                }
            })
            .collect();
        if hits.len() != 1 {
            return Err(format!(
                "proof {needle}: expected 1 literal, found {}",
                hits.len()
            ));
        }
        if hits[0].class != class {
            return Err(format!(
                "proof {needle}: class {} why {} line {}",
                hits[0].class.tag(),
                hits[0].why,
                hits[0].line
            ));
        }
        println!(
            "PROOF {} {} line {}",
            class.tag(),
            needle,
            hits[0].line
        );
    }
    println!("PROOF ok {}", expect.len());
    Ok(())
}


