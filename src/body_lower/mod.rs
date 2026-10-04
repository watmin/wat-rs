//! Lowered bodies — docs/arc/2026/10/lowered-bodies/BRIEF.md.
//!
//! Additive shim beside the interpreter: ordinary function bodies lower to a
//! form that reads locals by slot and calls builtins through the registry's
//! value doors, hooked into `apply_function` only.
//!
//! **Not named `src/lower/`, as the brief's own prose says.** `src/lower.rs`
//! already exists — a pre-existing, unrelated module (`WatAST` → `HolonAST`
//! algebra-core lowering) that is itself a load-bearing entry on
//! `tests/lint/holon_is_vsa_only.rs`'s `VSA_HOME_FILES` allowlist. Renaming
//! it out of the way to make room for this arc's `src/lower/` would require
//! editing that lint test too — a file outside this module's own footprint,
//! which STOP-3 says to name before making. The lower-merge-cost fix, named
//! here instead of made silently: this module is `body_lower`, a sibling
//! name with zero collision, zero edits anywhere else, same shape otherwise
//! — one new module, one `mod` line in `src/lib.rs`, one hook in
//! `apply_function`.
//!
//! ## L0 — the census
//!
//! `WAT_LOWER_CENSUS=1` turns on a per-function application counter and a
//! structural classifier (`would_lower`) that answers, for each user
//! function `apply_function` ever applies, whether its body consists ONLY
//! of the forms L1 will lower:
//!
//! - literals (including bare keyword values);
//! - parameters and `let`-bound locals, referenced by a bound [`Identifier`]
//!   (the same identity `Environment`/`EnvBuilder` already key bindings by —
//!   this walk does not invent a second notion of "the same name");
//! - `if`;
//! - `let`, with every binder a plain symbol (no destructure);
//! - calls to user functions registered in `SymbolTable::functions`, by
//!   name, in tail position or not;
//! - calls to intrinsics whose registry entry has a `value_handler` AND
//!   `@Purity Pure`.
//!
//! Anything else refuses. The FIRST refusing form's head (a call's keyword)
//! or node kind is recorded against that function, and the report at
//! process exit prints both a per-function table and a ranked table of
//! refusal reasons, weighted by how many times the refusing function was
//! actually applied — the number L2's coverage order reads from.
//!
//! No lowering happens yet in L0: `would_lower` only answers yes/no/why. L1
//! adds the actual slot-program + exec; this file's classifier is written
//! so L1 can reuse it unchanged as the "does this body qualify" gate.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::ast::WatAST;
use crate::scope::Identifier;
use crate::value::{Function, FunctionBody, SymbolTable};

// ─── The classifier — total over `WatAST`, refuses rather than guesses ────

/// Why a body (or a sub-form of it) refuses to lower. Carries enough to
/// name both the per-function "first form that refused" detail and the
/// coarser bucket the ranked report groups by.
#[derive(Clone, Debug)]
pub(crate) enum Refusal {
    /// A call whose head is a keyword that is neither a registered user
    /// function nor a Pure, value-door-bearing intrinsic. Carries the exact
    /// FQDN so L2's ranked table can read off `match`/`cond`/field-access
    /// by name, as the brief predicts.
    Call(String),
    /// A call whose head is a bound symbol (a fn VALUE held in a local/
    /// param, e.g. a stored closure) — dispatch is dynamic, not a static
    /// name in `SymbolTable::functions`.
    DynamicCall,
    /// A bare symbol reference that is not this function's own parameter
    /// or a `let`-bound local in its own body — most commonly a
    /// closure's free variable, reached through `closed_env` rather than
    /// this frame's slots.
    FreeVariable(String),
    /// A `let` binder that is not a plain symbol (tuple/struct/keys
    /// destructure).
    NonPlainBinder,
    /// Any other node shape refused outright: `Vector`/`Map`/`Set`/empty
    /// `List` in value position, a nested list or literal as a call head,
    /// or a malformed `if`/`let` arity (type-checked code should never
    /// reach the last case; kept as a defensive, honestly-labeled refusal
    /// rather than a panic).
    NodeKind(&'static str),
}

impl Refusal {
    /// Per-function detail: "the FIRST form that refuses (its head, or its
    /// node kind)," exactly as the brief asks for L0's report.
    fn detail(&self) -> String {
        match self {
            Refusal::Call(head) => head.clone(),
            Refusal::DynamicCall => "call via a bound symbol (dynamic head, not a named fn)".to_string(),
            Refusal::FreeVariable(name) => format!(
                "free variable `{name}` (closure capture — not this function's own param/let-bound local)"
            ),
            Refusal::NonPlainBinder => "let binder is not a plain symbol (destructure)".to_string(),
            Refusal::NodeKind(kind) => kind.to_string(),
        }
    }

    /// Coarser grouping key for the ranked, weighted-by-application-count
    /// table. Call heads stay exact (so `match`/`cond`/field-access read
    /// off by name); the structural refusals collapse to one bucket each
    /// so the table ranks BY REASON, not by incidental variable name.
    fn rank_key(&self) -> String {
        match self {
            Refusal::Call(head) => head.clone(),
            Refusal::DynamicCall => "<dynamic call via bound symbol>".to_string(),
            Refusal::FreeVariable(_) => "<free variable (closure capture)>".to_string(),
            Refusal::NonPlainBinder => "<let destructure binder>".to_string(),
            Refusal::NodeKind(kind) => format!("<{kind} in value position>"),
        }
    }
}

const IF_HEAD: &str = ":wat::core::if";
const LET_HEAD: &str = ":wat::core::let";

/// Classify one AST node. `bound` is the set of identifiers resolvable by
/// slot in the CURRENT function frame (its params, its rest-param, and
/// every `let`-local in scope at this point) — never touched by a nested
/// function's own frame, since a nested `fn`/`defn` body is never walked
/// from here (a `:wat::core::fn` literal is itself an unsupported call
/// head, refused before its body is ever reached).
fn classify(ast: &WatAST, sym: &SymbolTable, bound: &mut Vec<Identifier>) -> Result<(), Refusal> {
    match ast {
        // Literals — trivially representable, no resolution needed.
        WatAST::IntLit(..)
        | WatAST::FloatLit(..)
        | WatAST::RationalLit(..)
        | WatAST::BigIntLit(..)
        | WatAST::CharLit(..)
        | WatAST::BoolLit(..)
        | WatAST::StringLit(..)
        | WatAST::NilLit(..) => Ok(()),

        // A bare keyword in VALUE position (not a call head — calls are
        // handled by `classify_call` below, which never recurses back into
        // `classify` on the head itself) is a keyword literal.
        WatAST::Keyword(..) => Ok(()),

        // A bare symbol reference: must resolve to THIS frame's own params
        // or let-locals. Same identity `Environment::lookup`/`EnvBuilder`
        // already use (`Identifier`'s own `PartialEq`, name + hygiene scope
        // set) — not a second notion of "the same name."
        WatAST::Symbol(id, _) => {
            if bound.iter().any(|b| b == id) {
                Ok(())
            } else {
                Err(Refusal::FreeVariable(id.as_str().to_string()))
            }
        }

        // Binding-syntax-only shapes reached here means they appeared in
        // VALUE position — illegal today per the AST's own doc, but this
        // walk refuses rather than assumes.
        WatAST::Vector(..) => Err(Refusal::NodeKind("Vector")),
        WatAST::Map(..) => Err(Refusal::NodeKind("Map")),
        WatAST::Set(..) => Err(Refusal::NodeKind("Set")),

        WatAST::List(items, _) if items.is_empty() => Err(Refusal::NodeKind("EmptyList")),
        WatAST::List(items, _) => classify_call(items, sym, bound),
    }
}

/// Classify a non-empty `List` — a call, or `if`/`let`.
fn classify_call(items: &[WatAST], sym: &SymbolTable, bound: &mut Vec<Identifier>) -> Result<(), Refusal> {
    match &items[0] {
        WatAST::Keyword(head, _) if head == IF_HEAD => classify_if(&items[1..], sym, bound),
        WatAST::Keyword(head, _) if head == LET_HEAD => classify_let(&items[1..], sym, bound),
        WatAST::Keyword(head, _) if sym.has_function(head) => classify_args(&items[1..], sym, bound),
        WatAST::Keyword(head, _) => match crate::intrinsic::registry().lookup_entry(head) {
            Some(entry)
                if entry.value_handler.is_some() && matches!(entry.purity, wat_doc::Purity::Pure) =>
            {
                classify_args(&items[1..], sym, bound)
            }
            _ => Err(Refusal::Call(head.clone())),
        },
        // A bound-symbol head is a dynamic call through a stored fn value
        // (e.g. a closure parameter) — not a static name in
        // `SymbolTable::functions`. Refused regardless of whether the
        // symbol itself is bound in this frame; the REASON is the dynamic
        // dispatch, not an unbound name.
        WatAST::Symbol(..) => Err(Refusal::DynamicCall),
        // Anything else as a call head — a literal, a nested list (an
        // inline `((fn ...) args)` call), a Vector/Map/Set — refuses by
        // node kind.
        WatAST::IntLit(..) => Err(Refusal::NodeKind("IntLit")),
        WatAST::FloatLit(..) => Err(Refusal::NodeKind("FloatLit")),
        WatAST::RationalLit(..) => Err(Refusal::NodeKind("RationalLit")),
        WatAST::BigIntLit(..) => Err(Refusal::NodeKind("BigIntLit")),
        WatAST::CharLit(..) => Err(Refusal::NodeKind("CharLit")),
        WatAST::BoolLit(..) => Err(Refusal::NodeKind("BoolLit")),
        WatAST::StringLit(..) => Err(Refusal::NodeKind("StringLit")),
        WatAST::NilLit(..) => Err(Refusal::NodeKind("NilLit")),
        WatAST::List(..) => Err(Refusal::NodeKind("List")),
        WatAST::Vector(..) => Err(Refusal::NodeKind("Vector")),
        WatAST::Map(..) => Err(Refusal::NodeKind("Map")),
        WatAST::Set(..) => Err(Refusal::NodeKind("Set")),
    }
}

fn classify_args(args: &[WatAST], sym: &SymbolTable, bound: &mut Vec<Identifier>) -> Result<(), Refusal> {
    for a in args {
        classify(a, sym, bound)?;
    }
    Ok(())
}

/// `(:wat::core::if cond then else)` — exactly 3 args; classify in order.
fn classify_if(args: &[WatAST], sym: &SymbolTable, bound: &mut Vec<Identifier>) -> Result<(), Refusal> {
    if args.len() != 3 {
        // Type-checked code never reaches this; an honest refusal beats a panic.
        return Err(Refusal::Call(IF_HEAD.to_string()));
    }
    classify(&args[0], sym, bound)?;
    classify(&args[1], sym, bound)?;
    classify(&args[2], sym, bound)
}

/// `(:wat::core::let [name expr ...] body ...)` — every binder a plain
/// symbol; each RHS sees prior bindings (sequential, matching
/// `eval_let`/`eval_let_tail`'s own order); the whole implicit-do body
/// classified under the fully extended set.
fn classify_let(args: &[WatAST], sym: &SymbolTable, bound: &mut Vec<Identifier>) -> Result<(), Refusal> {
    if args.is_empty() {
        return Err(Refusal::Call(LET_HEAD.to_string()));
    }
    let bindings_form = &args[0];
    let items = match bindings_form {
        WatAST::Vector(items, _) if items.len() % 2 == 0 => items,
        _ => return Err(Refusal::Call(LET_HEAD.to_string())),
    };
    let original_len = bound.len();
    let mut i = 0;
    while i < items.len() {
        let binder = &items[i];
        let rhs = &items[i + 1];
        // RHS classified under the bindings seen so far (sequential; does
        // not see its own binder).
        if let Err(e) = classify(rhs, sym, bound) {
            bound.truncate(original_len);
            return Err(e);
        }
        match binder {
            WatAST::Symbol(id, _) => bound.push(id.clone()),
            _ => {
                bound.truncate(original_len);
                return Err(Refusal::NonPlainBinder);
            }
        }
        i += 2;
    }
    let body = &args[1..];
    let result = classify_args(body, sym, bound);
    bound.truncate(original_len);
    result
}

/// Classify a whole function body: `None` means it would lower; `Some`
/// carries the first refusal.
fn classify_function(func: &Function, sym: &SymbolTable) -> Option<Refusal> {
    let body_ast: &WatAST = match &func.body {
        FunctionBody::Wat(ast) => ast,
        // Native builtins never reach `apply_function` (the runtime dispatch
        // match intercepts them first) — this arm should be unreachable in
        // practice; refused rather than assumed if it ever is reached.
        FunctionBody::Native => return Some(Refusal::NodeKind("NativeBody")),
    };
    let mut bound: Vec<Identifier> = func.params.clone();
    if let Some(rest) = &func.rest_param {
        bound.push(Identifier::bare(rest.clone()));
    }
    classify(body_ast, sym, &mut bound).err()
}

// ─── L0 — the census ───────────────────────────────────────────────────────

struct CensusEntry {
    display_name: String,
    applications: u64,
    refusal: Option<Refusal>,
}

struct CensusState {
    functions: HashMap<usize, CensusEntry>,
}

static ENABLED: OnceLock<bool> = OnceLock::new();
static CENSUS: OnceLock<Mutex<CensusState>> = OnceLock::new();
static ATEXIT_REGISTERED: OnceLock<()> = OnceLock::new();

fn census_enabled() -> bool {
    *ENABLED.get_or_init(|| {
        std::env::var("WAT_LOWER_CENSUS")
            .map(|v| v == "1")
            .unwrap_or(false)
    })
}

fn display_name_of(func: &Function) -> String {
    match &func.name {
        Some(name) => name.clone(),
        None => {
            let span = match &func.body {
                FunctionBody::Wat(ast) => format!("{}", ast.span()),
                FunctionBody::Native => "<native>".to_string(),
            };
            format!("{} @ {span}", crate::value::ANON_FN_SYMBOL)
        }
    }
}

/// The ONE hook `apply_function` calls, once per loop iteration (so a
/// self-tail loop's 1,000,000 hops count as 1,000,000 applications, not
/// one). A no-op, one `OnceLock` read, unless `WAT_LOWER_CENSUS=1`.
pub(crate) fn record_application(func: &Arc<Function>, sym: &SymbolTable) {
    if !census_enabled() {
        return;
    }
    ATEXIT_REGISTERED.get_or_init(|| {
        // SAFETY: `print_report_c` is a plain `extern "C" fn()` reading only
        // this module's own statics; registering it is the standard
        // `libc::atexit` contract. `libc` and the atexit mechanism are
        // already load-bearing elsewhere in this crate (process/clone.rs,
        // process/stdio.rs) — reused, not a new pattern.
        unsafe {
            libc::atexit(print_report_c);
        }
    });
    let ptr = Arc::as_ptr(func) as usize;
    let mutex = CENSUS.get_or_init(|| {
        Mutex::new(CensusState {
            functions: HashMap::new(),
        })
    });
    let mut state = match mutex.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    match state.functions.get_mut(&ptr) {
        Some(entry) => entry.applications += 1,
        None => {
            let refusal = classify_function(func, sym);
            let display_name = display_name_of(func);
            state.functions.insert(
                ptr,
                CensusEntry {
                    display_name,
                    applications: 1,
                    refusal,
                },
            );
        }
    }
}

extern "C" fn print_report_c() {
    print_report();
}

fn print_report() {
    let Some(mutex) = CENSUS.get() else {
        return;
    };
    let state = match mutex.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    let mut rows: Vec<&CensusEntry> = state.functions.values().collect();
    rows.sort_by(|a, b| b.applications.cmp(&a.applications));

    eprintln!("\n=== WAT_LOWER_CENSUS: per-function report ({} functions applied) ===", rows.len());
    eprintln!("{:>14}  {:<5}  function / first refusal", "applications", "lowers");
    for entry in &rows {
        match &entry.refusal {
            None => eprintln!("{:>14}  {:<5}  {}", entry.applications, "YES", entry.display_name),
            Some(r) => eprintln!(
                "{:>14}  {:<5}  {}  —  refuses: {}",
                entry.applications,
                "no",
                entry.display_name,
                r.detail()
            ),
        }
    }

    let mut by_reason: HashMap<String, (u64, u64)> = HashMap::new();
    for entry in &rows {
        if let Some(r) = &entry.refusal {
            let slot = by_reason.entry(r.rank_key()).or_insert((0, 0));
            slot.0 += entry.applications;
            slot.1 += 1;
        }
    }
    let mut reasons: Vec<(String, (u64, u64))> = by_reason.into_iter().collect();
    reasons.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

    eprintln!("\n=== WAT_LOWER_CENSUS: refusal reasons, ranked by application count ===");
    eprintln!("{:>14}  {:>10}  reason", "applications", "functions");
    for (reason, (apps, nfn)) in &reasons {
        eprintln!("{:>14}  {:>10}  {}", apps, nfn, reason);
    }
    let would_lower_apps: u64 = rows.iter().filter(|e| e.refusal.is_none()).map(|e| e.applications).sum();
    let total_apps: u64 = rows.iter().map(|e| e.applications).sum();
    eprintln!(
        "\n=== WAT_LOWER_CENSUS: {would_lower_apps} / {total_apps} applications land on a function whose body would lower ==="
    );
}
