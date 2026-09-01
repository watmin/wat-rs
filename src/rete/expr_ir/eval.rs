use super::*;

type ExecArena = Vec<Option<Value>>;

/// Apply a compiled rete-defn to concrete args (user acc fold: the gathered PV).
pub(crate) fn exec_call(
    program: &Program,
    args: &[Value],
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    exec_program_on(program, args, None, sym, span)
}

// ── exec ─────────────────────────────────────────────────────────────────────

/// Run a lowered `where` fence against one row's bindings — the module's other public entry.
///
/// Seeds the frame from `program.reads`, runs [`exec`], and requires a `bool`: a fence that
/// evaluates to anything else is a type error at the fence, not a silent non-match.
pub(crate) fn exec_where<B: Bindings + ?Sized>(
    program: &Program,
    bindings: &B,
    sym: &SymbolTable,
    span: &Span,
) -> Result<bool, EvalBreak> {
    match exec_value(program, bindings, sym, span)? {
        Value::bool(b) => Ok(b),
        other => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: ":wat::rete::where".into(),
                expected: ":wat::core::bool (a where predicate must return bool)",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

/// Write one value into a frame slot, refusing an out-of-range slot rather than growing.
///
/// A slot index out of range means the `Program` and the frame disagree on `frame_len`, which is a
/// bug in lowering — so it raises rather than resizing, keeping the disagreement visible.
fn write_slot(
    frame: &mut [Option<Value>],
    slot: u16,
    v: Value,
    span: &Span,
) -> Result<(), EvalBreak> {
    match frame.get_mut(slot as usize) {
        Some(s) => {
            *s = Some(v);
            Ok(())
        }
        None => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::rete::exec_value".into(),
                reason: format!("slot {slot} is outside frame_len {}", frame.len()),
            },
        )
        .into()),
    }
}

/// Prologue (token bindings → slots) + eval. `where` requires bool;
/// `compiled_rhs` takes the `Value` as a fact field.
pub(crate) fn exec_value<B: Bindings + ?Sized>(
    program: &Program,
    bindings: &B,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    with_exec_frame(program.frame_len as usize, |frame| {
        for (k, slot) in program.reads.iter() {
            if let Some(v) = bindings.get(k) {
                write_slot(frame, *slot, v.clone(), span)?;
            }
        }
        exec(&program.root, frame, &program.names, sym, span)
    })
}

// rune:sequi(ambient-context) — one thread, nested frames bump a high-water
// arena so exec_where / CallUser / foldl do not allocate per token after warmup.
thread_local! {
    static EXEC_ARENA: RefCell<ExecArena> = const { RefCell::new(Vec::new()) };
    static EXEC_SP: Cell<usize> = const { Cell::new(0) };
}

/// Run `f` with a frame of `len` slots, carved off a thread-local ARENA rather than allocated.
///
/// `exec` runs once per row per fire, so a `Vec` per call is the hot-path cost this exists to
/// avoid. `EXEC_SP` is a stack pointer into the arena: a frame is the window `[sp, sp+len)`, the
/// window is zeroed on entry, and the pointer is restored on the way out. Nested calls therefore
/// stack rather than collide.
///
/// ⚠ **THE `Err` ARM IS THE PART THAT MATTERS.** A nested `exec_where` / `CallUser` / fold can be
/// entered while the OUTER frame is still borrowed — the `RefCell` is already held, so the arena
/// cannot hand out a second window. That case falls back to a plain heap `vec` for the inner
/// frame, leaving the arena with its outer owner. It is a correctness path, not an optimisation
/// gap: without it, re-entrancy would be a panic on a live borrow.
fn with_exec_frame<R>(len: usize, f: impl FnOnce(&mut [Option<Value>]) -> R) -> R {
    EXEC_ARENA.with(|arena| {
        match arena.try_borrow_mut() {
            Ok(mut g) => {
                let start = EXEC_SP.get();
                let end = start + len;
                if g.len() < end {
                    g.resize(end, None);
                }
                for slot in &mut g[start..end] {
                    *slot = None;
                }
                EXEC_SP.set(end);
                let out = f(&mut g[start..end]);
                EXEC_SP.set(start);
                out
            }
            // Nested exec_where / CallUser / fold while the outer frame is live.
            // Stack frame; the TLS arena stays with the outer caller.
            Err(_) => {
                let mut local = vec![None; len];
                f(&mut local)
            }
        }
    })
}

/// One operand of `and`/`or`, required to be a `bool`.
///
/// Both arms need the identical refusal and differ only in the op they name, so it lives here
/// rather than twice: the two were near-verbatim copies, and a copy is where a fix lands on one
/// side only.
fn expect_bool(v: Value, op: &'static str, span: &Span) -> Result<bool, EvalBreak> {
    match v {
        Value::bool(b) => Ok(b),
        other => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: ":wat::core::bool",
                got: Box::new(ValueSnapshot::of(&other)),
            },
        )
        .into()),
    }
}

/// Evaluate a lowered [`Expr`] against a frame — the interpreter every rete expression runs
/// through, in both positions (an inline constraint and a `where` fence) and on both engines.
///
/// ── THE FRAME CONTRACT ───────────────────────────────────────────────────────────────────────
///
/// `frame` is slot-indexed and `names` is its parallel debug spelling: `frame[i]` is the value
/// bound to slot `i`, `names[i]` the source name it came from. **`names` is never read on a
/// successful path** — it exists so an unbound slot can say *which* symbol was unbound instead of
/// `slot 7`. Keeping them separate is what lets the hot path carry `Option<Value>` and nothing else.
///
/// `span` is the WHOLE form's span, reused for every diagnostic raised in here: a lowered `Expr`
/// carries no span of its own (the same trade `exec_dim` makes in `where_tree.rs`). A reader
/// chasing an error to a character offset will land on the form, not the sub-expression.
///
/// ── WHAT DIVERTS BEFORE THE ARGUMENTS ARE EVALUATED ──────────────────────────────────────────
///
/// ⚠ `Expr::Call` matches four op names — `foldl`, `reduce`, `mapv`, `filterv` — and returns
/// early, BEFORE the loop that evaluates arguments into values. That is not an optimisation: those
/// four take a FUNCTION operand and apply it per element, so there is no single value to evaluate
/// it into. Every other op is strict, and the eager loop below is correct for exactly that reason.
/// Adding a fifth higher-order op means adding it to that match; missing it means its function
/// operand gets evaluated as a value and the failure will not look like a missing arm.
///
/// ── WHAT IT RAISES ───────────────────────────────────────────────────────────────────────────
///
/// Program errors only — an unbound slot, an unknown field, a non-bool where a bool was required,
/// whatever a primitive refuses. It is NOT a door where a session ceiling can surface: those are
/// bounds the caller can act on and they became matchable values at the verbs (arc 278, the
/// outcome wall). Nothing here should learn to raise one.
pub(crate) fn exec(
    e: &Expr,
    frame: &mut [Option<Value>],
    names: &[SlotName],
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    match e {
        Expr::Lit(v) => Ok(v.clone()),
        Expr::Slot(s) => frame
            .get(*s as usize)
            .and_then(|o| o.clone())
            .ok_or_else(|| {
                let name = names
                    .get(*s as usize)
                    .and_then(|n| n.as_ref().map(|a| a.to_string()))
                    .unwrap_or_else(|| format!("slot {s}"));
                RuntimeError::new(span.clone(), RuntimeErrorKind::UnboundSymbol(name)).into()
            }),
        Expr::Field { recv, idx } => {
            let v = exec(recv, frame, names, sym, span)?;
            match v {
                Value::Aggregate(a) => a.fields.get(*idx).cloned().ok_or_else(|| {
                    RuntimeError::new(
                        span.clone(),
                        RuntimeErrorKind::UnknownField {
                            record_class: a.class.to_string(),
                            field: format!("{idx}"),
                            available: (*a.names).clone(),
                        },
                    )
                    .into()
                }),
                other => Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: ":wat::rete::lower".into(),
                        expected: "record",
                        got: Box::new(ValueSnapshot::of(&other)),
                    },
                )
                .into()),
            }
        }
        Expr::Construct {
            class,
            names: field_names,
            fields,
        } => {
            let mut vs = Vec::with_capacity(fields.len());
            for f in fields.iter() {
                vs.push(exec(f, frame, names, sym, span)?);
            }
            Ok(Value::Aggregate(Arc::new(AggregateValue::record(
                class.clone(),
                Arc::clone(field_names),
                Arc::new(vs),
            ))))
        }
        Expr::Variant {
            type_path,
            variant_name,
            names: field_names,
            fields,
        } => {
            let mut vs = Vec::with_capacity(fields.len());
            for f in fields.iter() {
                vs.push(exec(f, frame, names, sym, span)?);
            }
            Ok(Value::Enum(Arc::new(EnumValue {
                type_path: type_path.clone(),
                variant_name: variant_name.clone(),
                names: Arc::clone(field_names),
                fields: vs,
            })))
        }
        Expr::If { cond, then_, else_ } => match exec(cond, frame, names, sym, span)? {
            Value::bool(true) => exec(then_, frame, names, sym, span),
            Value::bool(false) => exec(else_, frame, names, sym, span),
            other => Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::BadCondition {
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into()),
        },
        // THIS ARM CARRIED A DEAD ACCUMULATOR — ⚠ AND IT WAS NOT A DEFECT. Say that plainly,
        // because "removed a dead accumulator" reads like a bug was fixed and none was.
        //
        // It was `let mut acc = true; … acc = acc && b`. The `!b` case returns early, so `b` is
        // `true` every time that line runs and `Ok(bool(acc))` was `Ok(bool(true))`
        // unconditionally — which is the CORRECT answer at that point, since no operand was
        // false. Old and new agree on every input; it was driven both ways to check, and the two
        // gates that redden for a genuinely broken `and` (`reachability_shard_2_of_6` and
        // `spec_equals_native_on_every_where_family`) stayed green across the change.
        //
        // What it cost was comprehension: the code CLAIMED to fold the operands and did not, so a
        // reader had to derive that `acc` cannot be false before trusting it. That is worth
        // removing and is worth nothing more than that.
        //
        // ⛔ NO TEST WAS MISSING HERE, and the reason generalises: a dead-but-correct expression
        // is invisible to every behavioural check by construction, and clippy cannot see it either
        // (`acc` is written AND read; its constancy is semantic, not syntactic). The only tool that
        // finds this class is MUTATION APPLIED TO THE IMPLEMENTATION — replace `acc` with `true`,
        // and if nothing goes red, `acc` was dead. This codebase already runs that discipline on
        // its gates; turning it inward on values is a known, unbuilt blind spot.
        //
        // Empty `xs` still yields `true` here and `false` in `Or`, which is the vacuous reading
        // and is what the old code did.
        Expr::And(xs) => {
            for x in xs.iter() {
                if !expect_bool(exec(x, frame, names, sym, span)?, ":wat::rete::core::and", span)? {
                    return Ok(Value::bool(false));
                }
            }
            Ok(Value::bool(true))
        }
        Expr::Or(xs) => {
            for x in xs.iter() {
                if expect_bool(exec(x, frame, names, sym, span)?, ":wat::rete::core::or", span)? {
                    return Ok(Value::bool(true));
                }
            }
            Ok(Value::bool(false))
        }
        Expr::Let { binds, body } => {
            for (slot, e) in binds.iter() {
                let v = exec(e, frame, names, sym, span)?;
                write_slot(frame, *slot, v, span)?;
            }
            exec(body, frame, names, sym, span)
        }
        Expr::Match { scrutinee, arms } => {
            let v = exec(scrutinee, frame, names, sym, span)?;
            for (pat, body) in arms.iter() {
                if pat_matches(pat, &v, frame, sym, span)? {
                    return exec(body, frame, names, sym, span);
                }
            }
            Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::PatternMatchFailed {
                    value_type: v.type_name(),
                },
            )
            .into())
        }
        Expr::Call { op, args } => {
            match RETE_OPS[*op as usize].core_name {
                ":wat::core::foldl" => return exec_foldl(args, frame, names, sym, span),
                ":wat::core::reduce" => return exec_reduce(args, frame, names, sym, span),
                ":wat::core::mapv" => return exec_mapv(args, frame, names, sym, span),
                ":wat::core::filterv" => return exec_filterv(args, frame, names, sym, span),
                _ => {}
            }
            let mut vs = Vec::with_capacity(args.len());
            for a in args.iter() {
                vs.push(exec(a, frame, names, sym, span)?);
            }
            apply_op(*op, &vs, span, Some(sym))
        }
        Expr::CallFallback { op, args, fallback } => {
            let row = &RETE_OPS[*op as usize];
            let mut vs = Vec::with_capacity(args.len());
            for a in args.iter() {
                vs.push(exec(a, frame, names, sym, span)?);
            }
            // ONE classification — see `where_tree.rs`'s twin and
            // `classify_fallback_outcome`. Only the recursion is this site's own.
            match crate::runtime::classify_fallback_outcome(
                apply_op(*op, &vs, span, Some(sym)),
                &row.ret,
                row.core_name,
                row.rete_name,
                span,
            )? {
                crate::runtime::FallbackVerdict::Value(v) => Ok(v),
                crate::runtime::FallbackVerdict::UseFallback => {
                    exec(fallback, frame, names, sym, span)
                }
            }
        }
        Expr::CallUser { program, args } => {
            if args.is_empty() {
                // Literal fn value — foldl applies it via exec_foldl.
                return exec_program_on(program, &[], None, sym, span);
            }
            let mut vs = Vec::with_capacity(args.len());
            for a in args.iter() {
                vs.push(exec(a, frame, names, sym, span)?);
            }
            exec_program_on(program, &vs, None, sym, span)
        }
    }
}

/// The `op` an arity refusal from [`exec_program_on`] names. A `Program` carries no callee
/// name (see the struct in `expr_ir/mod.rs`), so this names the call FORM — the counts and the
/// call-site span carry the rest. Adding a name field to `Program` to make the message prettier
/// is a wider change than this contract needs.
const CALL_USER_OP: &str = ":wat::rete::call-user";

/// Run a `Program` against `args`, optionally over a PARENT frame.
///
/// `parent` is what makes a nested program see its enclosing scope's slots — a user fold's body
/// running inside the condition that gathered for it. Passing `None` runs it in isolation. The
/// rune above the parameter records why it stays a raw slice rather than a named alias: the slot
/// LAYOUT is the contract here, and an alias would hide it.
fn exec_program_on(
    program: &Program,
    args: &[Value],
    // rune:perspicere(intentional-structure) — SlotFrame row; alias body would hide the slot layout
    parent: Option<&[Option<Value>]>,
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    // ── THE ARITY CONTRACT — one integer comparison, on the fire path ────────────────────────
    //
    // `args` and `program.params` meet HERE and nowhere else: this function is downstream of the
    // wire (`unpack_expr`'s `:user` arm), of the lowering (`lower_expr` builds `CallUser` from
    // `lower_args` and `lower_rete_defn` without comparing them), of `exec_call`, and of all four
    // HOF arms (`foldl` / `reduce` / `mapv` / `filterv`). A wall at the import door would be a
    // second COPY of an invariant the executor still would not hold. Put the check where the two
    // quantities meet and there is no other door left to assume.
    //
    // ⛔ There is no `else` for a surplus argument, and there must never be one again. The branch
    // this replaces wrote an argument with NO parameter into the slot whose number happened to
    // equal its ARGUMENT POSITION, and it was driven: with one param at slot 1 and args [10, 30]
    // the surplus overwrote the declared parameter and a live fence answered 0 hits instead of 1
    // — a silent wrong answer from wire input. Making that write "safe" (clamped, guarded,
    // skipped) would leave an argument with no parameter still MEANING something. It has no
    // meaning to be given; the call is refused.
    //
    // `Program` carries no callee identity — only `frame_len`, `root`, `reads`, `params`, `names`
    // (slot -> binder) and its body `span` — so `op` names the CALL FORM, which is the most this
    // struct can honestly say. The counts are the payload.
    if args.len() != program.params.len() {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: CALL_USER_OP.into(),
                expected: program.params.len(),
                got: args.len(),
            },
        )
        .into());
    }
    let max_param = program
        .params
        .iter()
        .copied()
        .max()
        .map(|s| s as usize + 1)
        .unwrap_or(0);
    let n = (program.frame_len as usize)
        .max(parent.map(|p| p.len()).unwrap_or(0))
        .max(max_param);
    with_exec_frame(n, |inner| {
        if let Some(p) = parent {
            for (i, v) in p.iter().enumerate() {
                inner[i] = v.clone();
            }
        }
        // TOTAL, and by construction rather than by discipline: the lengths are equal (checked
        // above), and `n` is at least `max_param` = max(params) + 1, so every `idx` indexes
        // `inner`. Neither a `get` nor a bounds guard can fire here.
        for (i, v) in args.iter().enumerate() {
            let idx = program.params[i] as usize;
            inner[idx] = Some(v.clone());
        }
        exec(&program.root, inner, &program.names, sym, span)
    })
}

/// `foldl` — the higher-order op that motivates the callee machinery.
///
/// Its function operand is applied PER ELEMENT, so it cannot be pre-evaluated into a value the way
/// `exec`'s strict arms evaluate theirs; see the divert documented on [`exec`].
fn exec_foldl(
    args: &[Expr],
    frame: &mut [Option<Value>],
    names: &[SlotName],
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    if args.len() != 3 {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::core::foldl".into(),
                expected: 3,
                got: args.len(),
            },
        )
        .into());
    }
    let program = compiled_fn_arg(&args[0], ":wat::core::foldl", span)?;
    let mut acc = exec(&args[1], frame, names, sym, span)?;
    let coll = exec(&args[2], frame, names, sym, span)?;
    let items = eager_items(&coll, ":wat::core::foldl", span)?;
    for x in items {
        acc = exec_program_on(&program, &[acc.clone(), x], Some(frame), sym, span)?;
    }
    Ok(acc)
}

/// The fn operand of a HOF, as a compiled program. Shared so `reduce` cannot drift from `foldl`.
fn compiled_fn_arg(arg: &Expr, op: &str, span: &Span) -> Result<Arc<Program>, EvalBreak> {
    match arg {
        Expr::CallUser { program, .. } => Ok(Arc::clone(program)),
        _ => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: op.into(),
                reason: "fn-arg must be a compiled fn".into(),
            },
        )
        .into()),
    }
}

/// The eager containers a compiled fence can walk. A `Stream` is deliberately absent: it is lazy,
/// and the compiled executor has no stream machinery — so it reports a type mismatch that NAMES
/// the containers it does accept rather than silently producing nothing.
fn eager_items(coll: &Value, op: &str, span: &Span) -> Result<Vec<Value>, EvalBreak> {
    match coll {
        Value::Vec(xs) => Ok(xs.iter().cloned().collect()),
        Value::wat__core__PersistentVector(pv) => Ok(pv.iter().cloned().collect()),
        Value::wat__core__List(xs) => Ok(xs.iter().cloned().collect()),
        other => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: op.into(),
                expected: "wat::core::Vector, wat::core::PersistentVector, or wat::core::List",
                got: Box::new(ValueSnapshot::of(other)),
            },
        )
        .into()),
    }
}

/// `(:wat::core::reduce f init coll)` / `(:wat::core::reduce f coll)`.
///
/// ⛔ **FOLDL IS REDUCE, and this is a MIRROR of `wat/seq.wat:317-329`, not a reimplementation.**
/// That `defclause` states both clauses outright: the 3-arity form is literally
/// `(:wat::core::foldl f init coll)`, and the 2-arity form seeds the fold with the first element
/// and RAISES BY NAME on an empty collection. Both are reproduced here and nowhere else, so the
/// compiled answer and the interpreted one cannot diverge.
///
/// Why a compiled arm exists at all: `reduce` is a wat-level `defclause`, so unlike its siblings
/// it has no Rust dispatch to re-enter, and a compiled `where` fence has no defclause machinery.
/// Found 2026-08-28 by the § 4.1 ledger — the row passed admission, totality, arity and type and
/// then raised `unbound symbol: acc`, because lowering treats all four HOFs alike while `exec`
/// routed only `foldl`.
///
/// ⚠ The 2-arity empty case RAISES, while `RETE_OPS` declares this row `total: true`. That
/// contradiction is inherited, not introduced — and it went unnoticed precisely because nothing
/// could execute the row to find it. Recorded in `RETE-OPEN-WORK` § 4.1; not silently papered over
/// here, because answering an empty reduce with some invented value would be the worse bug.
fn exec_reduce(
    args: &[Expr],
    frame: &mut [Option<Value>],
    names: &[SlotName],
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::reduce";
    if args.len() == 3 {
        return exec_foldl(args, frame, names, sym, span);
    }
    if args.len() != 2 {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::ArityMismatch { op: OP.into(), expected: 3, got: args.len() },
        )
        .into());
    }
    let program = compiled_fn_arg(&args[0], OP, span)?;
    let coll = exec(&args[1], frame, names, sym, span)?;
    let items = eager_items(&coll, OP, span)?;
    let mut it = items.into_iter();
    let Some(mut acc) = it.next() else {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: OP.into(),
                reason: "the 2-arity form needs at least one element to seed the fold; got an \
                         empty collection"
                    .into(),
            },
        )
        .into());
    };
    for x in it {
        acc = exec_program_on(&program, &[acc.clone(), x], Some(frame), sym, span)?;
    }
    Ok(acc)
}

/// `(:wat::core::mapv f coll)` — the EAGER map. Returns a `Vector`, matching `eval_mapv`
/// (`collection/transform.rs`), whose every exit is `Ok(Value::Vec(..))`.
///
/// ⛔ **THE RETE SURFACE TAKES `mapv`, NOT `map`, AND THAT IS THE WHOLE POINT.** `:wat::core::map`
/// returns a LAZY `Stream`; a compiled `where` fence has no stream machinery and nothing in a
/// fence can consume one, so the `map` row was unreachable in every position. Adding an eager arm
/// under the `map` name would have made `:wat::rete::core::map` mean something different from
/// `:wat::core::map` — silently — when the `Redispatch` contract is "the same routine as
/// `core_name`". wat already ships the eager materializer under its clojure name, so rete takes
/// that instead: no invented semantics and no divergence. See `wat/seq.wat`'s "the eager forms".
fn exec_mapv(
    args: &[Expr],
    frame: &mut [Option<Value>],
    names: &[SlotName],
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::mapv";
    if args.len() != 2 {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::ArityMismatch { op: OP.into(), expected: 2, got: args.len() },
        )
        .into());
    }
    let program = compiled_fn_arg(&args[0], OP, span)?;
    let coll = exec(&args[1], frame, names, sym, span)?;
    let items = eager_items(&coll, OP, span)?;
    let mut out = Vec::with_capacity(items.len());
    for x in items {
        out.push(exec_program_on(&program, &[x], Some(frame), sym, span)?);
    }
    Ok(Value::Vec(Arc::new(out)))
}

/// `(:wat::core::filterv pred coll)` — the EAGER filter. Returns a `Vector`, matching
/// `wat/seq.wat`'s `defclause`, which is `(:wat::core::into [] (:wat::core::filter pred coll))`
/// for both of its clauses.
///
/// The predicate must answer `bool`. A non-bool is refused BY NAME rather than coerced: a filter
/// that silently treats a non-boolean as truthy would drop or keep rows for a reason no user
/// wrote, which is the silent-wrong-answer class this arc exists to remove.
fn exec_filterv(
    args: &[Expr],
    frame: &mut [Option<Value>],
    names: &[SlotName],
    sym: &SymbolTable,
    span: &Span,
) -> Result<Value, EvalBreak> {
    const OP: &str = ":wat::core::filterv";
    if args.len() != 2 {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::ArityMismatch { op: OP.into(), expected: 2, got: args.len() },
        )
        .into());
    }
    let program = compiled_fn_arg(&args[0], OP, span)?;
    let coll = exec(&args[1], frame, names, sym, span)?;
    let items = eager_items(&coll, OP, span)?;
    let mut out = Vec::with_capacity(items.len());
    for x in items {
        match exec_program_on(&program, std::slice::from_ref(&x), Some(frame), sym, span)? {
            Value::bool(true) => out.push(x),
            Value::bool(false) => {}
            other => {
                return Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::TypeMismatch {
                        op: OP.into(),
                        expected: "wat::core::bool",
                        got: Box::new(ValueSnapshot::of(&other)),
                    },
                )
                .into());
            }
        }
    }
    Ok(Value::Vec(Arc::new(out)))
}

/// `Ok(true)` the arm binds, `Ok(false)` it does not, `Err` the PATTERN itself is wrong.
///
/// The third case exists because of the hash-destructure arm and it is not a widening of the
/// match's partiality — it is refusing to be SILENT. Core raises `UnknownField` for a field the
/// class does not declare (verified: it raises even with a catch-all arm after it), so returning
/// "does not match" here would make the same expression answer differently in the two engines,
/// AND would turn a typo into a constraint that compiles, fires and matches nothing — fix-list
/// F's class exactly. Diverging from core silently is the more expensive of the two.
fn pat_matches(
    pat: &Pat,
    v: &Value,
    frame: &mut [Option<Value>],
    sym: &SymbolTable,
    span: &Span,
) -> Result<bool, EvalBreak> {
    Ok(match pat {
        Pat::Wild => true,
        Pat::Bind(s) => match frame.get_mut(*s as usize) {
            Some(slot) => {
                *slot = Some(v.clone());
                true
            }
            None => false,
        },
        Pat::Lit(lit) => v == lit,
        Pat::Variant { name, payload } => match v {
            Value::Option(opt) => match (name.as_str(), opt.as_ref()) {
                ("None", None) => payload.is_none(),
                ("Some", Some(inner)) => match payload {
                    Some(p) => pat_matches(p, inner, frame, sym, span)?,
                    None => true,
                },
                _ => false,
            },
            Value::Result(r) => match (name.as_str(), r.as_ref()) {
                ("Ok", Ok(inner)) => match payload {
                    Some(p) => pat_matches(p, inner, frame, sym, span)?,
                    None => true,
                },
                ("Err", Err(inner)) => match payload {
                    Some(p) => pat_matches(p, inner, frame, sym, span)?,
                    None => true,
                },
                _ => false,
            },
            Value::Enum(e) => {
                let composed = format!("{}::{}", e.type_path, e.variant_name);
                let last = wat_reader::identifier::leaf(name).trim_start_matches(':');
                if composed != *name && e.variant_name != *name && e.variant_name != last {
                    return Ok(false);
                }
                match payload {
                    None => e.fields.is_empty(),
                    Some(p) => match e.fields.first() { Some(f) => pat_matches(p, f, frame, sym, span)?, None => false },
                }
            }
            _ => false,
        },
        // The subject carries its OWN class, so the index is resolved here rather than at lower
        // time — see `Pat::Fields`. `field_index` is the SAME lookup the accessor path uses, so
        // there is one definition of "which slot is `:x`", not two.
        //
        // A field the class does not declare makes the ARM not match, exactly as a wrong literal
        // or a wrong variant would; it is not an error. That keeps a `match` total, which every
        // rete row must be — an arm that raised would put a partial op inside the jump table.
        // The fall-through when NO arm matches is `PatternMatchFailed`, already this fn's caller's
        // job and unchanged.
        Pat::Fields(binds) => {
            // A non-aggregate subject does NOT match — same as a wrong literal or wrong variant.
            // That half is a legitimate arm outcome and stays silent, exactly like core's, whose
            // receiver dispatch falls through for any other value type.
            let Value::Aggregate(a) = v else { return Ok(false) };
            for (field, slot) in binds.iter() {
                // An undeclared field RAISES, mirroring core's `UnknownField` — see this fn's doc.
                // The message carries the available fields for the same reason core's does: the
                // ruin must teach, and "no arm matched" would teach nothing about the typo.
                let Some(idx) = field_index(sym, &a.class, field) else {
                    let available = sym
                        .types()
                        .and_then(|t| t.get(&format!(":{}", a.class.trim_start_matches(':'))).cloned())
                        .and_then(|d| match d {
                            crate::types::TypeDef::Aggregate(ag) => {
                                Some(ag.field_names().collect::<Vec<_>>().join(", "))
                            }
                            _ => None,
                        })
                        .unwrap_or_else(|| "<class not registered>".to_string());
                    return Err(RuntimeError::new(
                        span.clone(),
                        RuntimeErrorKind::MalformedForm {
                            head: ":wat::rete::core::match".into(),
                            reason: format!(
                                "match hash-destructure binds `{}` from field `:{}`, which \
                                 `{}` does not declare; available: [{}]",
                                names_of_slot(*slot), field, a.class, available
                            ),
                        },
                    )
                    .into());
                };
                let Some(val) = a.fields.get(idx) else { return Ok(false) };
                match frame.get_mut(*slot as usize) {
                    Some(cell) => *cell = Some(val.clone()),
                    None => return Ok(false),
                }
            }
            true
        }
    })
}

/// A slot has no name at exec time — the frame is positional. The diagnostic above still wants to
/// say WHICH binder failed, so it names the slot rather than pretending to know the identifier.
fn names_of_slot(slot: u16) -> String {
    format!("slot {slot}")
}

#[derive(Clone, Copy, Debug)]
enum OpExec {
    Eq, NotEq, Gt, Lt, Ge, Le,
    I64Eq, I64NotEq, StrEq, StrNotEq,
    StrLen, StartsWith, EndsWith, Contains, Not,
    I64Add, I64Sub, I64Mul, I64Div, I64Rem, I64Mod, I64ToF64, I64ToStr,
    F64Gt, F64Lt, F64Ge, F64Le, F64Eq, F64NotEq, F64Add, F64Sub, F64Mul, F64Div, F64ToStr,
    BoolToStr, StrEmpty, StrConcat, StrTrim, StrLower, StrSubs,
    PvLen, PvContains, PvGet, VecGet, ListGet, First, PvNew, VecNew, ListNew,
    PmContainsKey, PmNew, Second, Third, TupleNew, KwToStr, KwFromStr,
    Cosine, Dot, Coincident, Presence,
    Unknown,
}

impl OpExec {
    /// Map a core comparison FQDN to its fast-path kind.
    ///
    /// The generic and `i64`-specialised spellings deliberately fold together for the ordering
    /// comparisons (`:wat::core::>` and `:wat::core::i64::>` are one `Gt`) but stay SEPARATE for
    /// equality (`I64Eq` is not `Eq`) — equality on `i64` can take a direct integer compare,
    /// while the generic form has to go through value equality.
    fn of(core: &str) -> Self {
        match core {
            ":wat::core::=" => Self::Eq,
            ":wat::core::not=" => Self::NotEq,
            ":wat::core::i64::>" | ":wat::core::>" => Self::Gt,
            ":wat::core::i64::<" | ":wat::core::<" => Self::Lt,
            ":wat::core::i64::>=" | ":wat::core::>=" => Self::Ge,
            ":wat::core::i64::<=" | ":wat::core::<=" => Self::Le,
            ":wat::core::i64::=" => Self::I64Eq,
            ":wat::core::i64::not=" => Self::I64NotEq,
            ":wat::core::string::=" => Self::StrEq,
            ":wat::core::string::not=" => Self::StrNotEq,
            ":wat::core::string::length" => Self::StrLen,
            ":wat::core::string::starts-with?" | ":wat::core::String/starts-with?" => Self::StartsWith,
            ":wat::core::string::ends-with?" | ":wat::core::String/ends-with?" => Self::EndsWith,
            ":wat::core::string::contains?" | ":wat::core::String/contains?" => Self::Contains,
            ":wat::core::not" => Self::Not,
            ":wat::core::i64::+" => Self::I64Add,
            ":wat::core::i64::-" => Self::I64Sub,
            ":wat::core::i64::*" => Self::I64Mul,
            ":wat::core::i64::/" | ":wat::core::i64::quot" => Self::I64Div,
            ":wat::core::i64::rem" => Self::I64Rem,
            ":wat::core::i64::mod" => Self::I64Mod,
            ":wat::core::i64::to-f64" => Self::I64ToF64,
            ":wat::core::i64::to-string" => Self::I64ToStr,
            ":wat::core::f64::>" => Self::F64Gt,
            ":wat::core::f64::<" => Self::F64Lt,
            ":wat::core::f64::>=" => Self::F64Ge,
            ":wat::core::f64::<=" => Self::F64Le,
            ":wat::core::f64::=" => Self::F64Eq,
            ":wat::core::f64::not=" => Self::F64NotEq,
            ":wat::core::f64::+" => Self::F64Add,
            ":wat::core::f64::-" => Self::F64Sub,
            ":wat::core::f64::*" => Self::F64Mul,
            ":wat::core::f64::/" => Self::F64Div,
            ":wat::core::f64::to-string" => Self::F64ToStr,
            ":wat::core::bool::to-string" => Self::BoolToStr,
            ":wat::core::keyword/to-string" => Self::KwToStr,
            ":wat::core::keyword/from-string" => Self::KwFromStr,
            ":wat::core::String/empty?" => Self::StrEmpty,
            ":wat::core::String/concat" => Self::StrConcat,
            ":wat::core::string::trim" => Self::StrTrim,
            ":wat::core::string::to-lowercase" => Self::StrLower,
            ":wat::core::string::subs" => Self::StrSubs,
            ":wat::core::PersistentVector/length" => Self::PvLen,
            ":wat::core::PersistentVector/contains?" => Self::PvContains,
            ":wat::core::PersistentMap/contains-key?" => Self::PmContainsKey,
            ":wat::core::PersistentVector/get" => Self::PvGet,
            ":wat::core::Vector/get" => Self::VecGet,
            ":wat::core::List/get" => Self::ListGet,
            ":wat::core::first" => Self::First,
            ":wat::core::second" => Self::Second,
            ":wat::core::third" => Self::Third,
            ":wat::core::PersistentVector" => Self::PvNew,
            ":wat::core::Vector" => Self::VecNew,
            ":wat::core::List" => Self::ListNew,
            ":wat::core::Tuple" => Self::TupleNew,
            ":wat::core::PersistentMap" => Self::PmNew,
            ":wat::holon::cosine" => Self::Cosine,
            ":wat::holon::dot" => Self::Dot,
            ":wat::holon::coincident?" => Self::Coincident,
            ":wat::holon::presence?" => Self::Presence,
            _ => Self::Unknown,
        }
    }
}

/// Index `RETE_OPS` once; fire matches `OpExec`, never the FQDN string.
/// `sym` is required for holon rows (encoding ctx). The where-tree dim
/// walker may pass `None` and treat a holon miss as over-approx.
pub(crate) fn apply_op(
    op: u16,
    args: &[Value],
    span: &Span,
    sym: Option<&SymbolTable>,
) -> Result<Value, EvalBreak> {
    // rune:sequi(ambient-context) — opcode table interned once; not fire-domain state.
    static KINDS: OnceLock<Vec<OpExec>> = OnceLock::new();
    let kinds = KINDS.get_or_init(|| {
        RETE_OPS.iter().map(|r| OpExec::of(r.core_name)).collect()
    });
    let Some(&kind) = kinds.get(op as usize) else {
        return Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::rete::apply_op".into(),
                reason: format!("op index {op} is outside RETE_OPS"),
            },
        )
        .into());
    };
    apply_core_kind(kind, args, span, sym)
}

/// Apply one compiled core op to its ALREADY-EVALUATED arguments.
///
/// This is the leaf of the compiled-expression interpreter: `expr_ir` lowers a
/// `:where` form to `Op`s, the walker evaluates the operands, and every actual
/// computation lands here. The 53 arms are a flat dispatch table — one per
/// `OpExec` — and are deliberately uncommented: each matches on `(kind, args)`
/// and its body IS its specification. Reach for the arm, not for prose.
///
/// Two things a reader cannot recover from the arms themselves:
///
/// **The pattern is the arity-and-type check.** An arm matches only when the
/// operand shapes match too (`[Value::i64(a), Value::i64(b)]`), so a wrong
/// arity or a wrong operand type does not reach a body — it falls through to
/// the catch-all. There is no separate validation pass; this table is it.
///
/// **The catch-all raises with head `"compiled-exec"`, and that head matters.**
/// `exec_dim`'s `CallFallback` swallows a `MalformedForm` only when its head
/// equals the op's own `core_name` (see `where_tree::exec_dim`). `"compiled-exec"`
/// never equals one, so a dispatch failure PROPAGATES rather than being
/// silently replaced by a fallback value. Do not retag this error to an op name.
///
/// `sym` is `None` off the encoding path; the holon arms that need it raise
/// `NoEncodingCtx` rather than assuming a context they were not given.
fn apply_core_kind(
    kind: OpExec,
    args: &[Value],
    span: &Span,
    sym: Option<&SymbolTable>,
) -> Result<Value, EvalBreak> {
    match (kind, args) {
        (OpExec::Eq, [a, b]) => Ok(Value::bool(a == b)),
        (OpExec::NotEq, [a, b]) => Ok(Value::bool(a != b)),
        (OpExec::Gt, [a, b]) => {
            ord(a, b, span, |o| o.is_gt())
        }
        (OpExec::Lt, [a, b]) => {
            ord(a, b, span, |o| o.is_lt())
        }
        (OpExec::Ge, [a, b]) => {
            ord(a, b, span, |o| !o.is_lt())
        }
        (OpExec::Le, [a, b]) => {
            ord(a, b, span, |o| !o.is_gt())
        }
        (OpExec::I64Eq, [a, b]) => Ok(Value::bool(a == b)),
        (OpExec::I64NotEq, [a, b]) => Ok(Value::bool(a != b)),
        (OpExec::StrEq, [a, b]) => Ok(Value::bool(a == b)),
        (OpExec::StrNotEq, [a, b]) => Ok(Value::bool(a != b)),
        (OpExec::StrLen, [Value::String(s)]) => {
            Ok(Value::i64(s.chars().count() as i64))
        }
        (OpExec::StartsWith, [Value::String(s), Value::String(p)]) => {
            Ok(Value::bool(s.starts_with(p.as_str())))
        }
        (OpExec::EndsWith, [Value::String(s), Value::String(p)]) => {
            Ok(Value::bool(s.ends_with(p.as_str())))
        }
        (OpExec::Contains, [Value::String(s), Value::String(p)]) => {
            Ok(Value::bool(s.contains(p.as_str())))
        }
        (OpExec::F64Gt, [a, b]) => ord(a, b, span, |o| o.is_gt()),
        (OpExec::F64Lt, [a, b]) => ord(a, b, span, |o| o.is_lt()),
        (OpExec::Not, [Value::bool(b)]) => Ok(Value::bool(!*b)),
        (OpExec::I64Add, [Value::i64(a), Value::i64(b)]) => match a.checked_add(*b) {
            Some(n) => Ok(Value::i64(n)),
            None => Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::IntegerOverflow {
                    op: "+".into(),
                    a: *a,
                    b: *b,
                },
            )
            .into()),
        },
        (OpExec::I64Sub, [Value::i64(a), Value::i64(b)]) => match a.checked_sub(*b) {
            Some(n) => Ok(Value::i64(n)),
            None => Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::IntegerOverflow {
                    op: "-".into(),
                    a: *a,
                    b: *b,
                },
            )
            .into()),
        },
        (OpExec::I64Mul, [Value::i64(a), Value::i64(b)]) => match a.checked_mul(*b) {
            Some(n) => Ok(Value::i64(n)),
            None => Err(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::IntegerOverflow {
                    op: "*".into(),
                    a: *a,
                    b: *b,
                },
            )
            .into()),
        },
        (OpExec::I64Div, [Value::i64(a), Value::i64(b)]) => {
            if *b == 0 {
                return Err(
                    RuntimeError::new(span.clone(), RuntimeErrorKind::DivisionByZero).into(),
                );
            }
            match a.checked_div(*b) {
                Some(n) => Ok(Value::i64(n)),
                None => Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::IntegerOverflow {
                        op: "/".into(),
                        a: *a,
                        b: *b,
                    },
                )
                .into()),
            }
        }
        (OpExec::I64Rem, [Value::i64(a), Value::i64(b)]) => {
            if *b == 0 {
                return Err(
                    RuntimeError::new(span.clone(), RuntimeErrorKind::DivisionByZero).into(),
                );
            }
            match a.checked_rem(*b) {
                Some(n) => Ok(Value::i64(n)),
                None => Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::IntegerOverflow {
                        op: "rem".into(),
                        a: *a,
                        b: *b,
                    },
                )
                .into()),
            }
        }
        (OpExec::I64Mod, [Value::i64(a), Value::i64(b)]) => {
            if *b == 0 {
                return Err(
                    RuntimeError::new(span.clone(), RuntimeErrorKind::DivisionByZero).into(),
                );
            }
            let r = match a.checked_rem(*b) {
                Some(n) => n,
                None => {
                    return Err(RuntimeError::new(
                        span.clone(),
                        RuntimeErrorKind::IntegerOverflow {
                            op: "mod".into(),
                            a: *a,
                            b: *b,
                        },
                    )
                    .into())
                }
            };
            Ok(Value::i64(if r != 0 && (r < 0) != (*b < 0) {
                r + *b
            } else {
                r
            }))
        }
        (OpExec::I64ToF64, [Value::i64(n)]) => Ok(Value::f64(*n as f64)),
        (OpExec::I64ToStr, [Value::i64(n)]) => {
            Ok(Value::String(Arc::new(n.to_string())))
        }
        (OpExec::F64ToStr, [Value::f64(n)]) => {
            Ok(Value::String(Arc::new(n.to_string())))
        }
        (OpExec::BoolToStr, [Value::bool(b)]) => {
            Ok(Value::String(Arc::new(b.to_string())))
        }
        (OpExec::F64Add, [Value::f64(a), Value::f64(b)]) => Ok(Value::f64(*a + *b)),
        (OpExec::F64Sub, [Value::f64(a), Value::f64(b)]) => Ok(Value::f64(*a - *b)),
        (OpExec::F64Mul, [Value::f64(a), Value::f64(b)]) => Ok(Value::f64(*a * *b)),
        (OpExec::F64Div, [Value::f64(a), Value::f64(b)]) => Ok(Value::f64(*a / *b)),
        (OpExec::F64Ge, [a, b]) => ord(a, b, span, |o| !o.is_lt()),
        (OpExec::F64Le, [a, b]) => ord(a, b, span, |o| !o.is_gt()),
        (OpExec::F64Eq, [a, b]) => Ok(Value::bool(a == b)),
        (OpExec::F64NotEq, [a, b]) => Ok(Value::bool(a != b)),
        (OpExec::StrEmpty, [Value::String(s)]) => Ok(Value::bool(s.is_empty())),
        (OpExec::StrConcat, [Value::String(a), Value::String(b)]) => {
            Ok(Value::String(Arc::new(format!("{a}{b}"))))
        }
        (OpExec::StrTrim, [Value::String(s)]) => {
            Ok(Value::String(Arc::new(s.trim().to_string())))
        }
        (OpExec::StrLower, [Value::String(s)]) => {
            Ok(Value::String(Arc::new(s.to_lowercase())))
        }
        (OpExec::StrSubs, [Value::String(s), Value::i64(start), Value::i64(end)]) => {
            let char_len = s.chars().count() as i64;
            if *start < 0 || *end < 0 || *start > *end || *end > char_len {
                return Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::string::subs".into(),
                        reason: format!(
                            "index out of range: start={start}, end={end}, char-length={char_len}; \
                             require 0 <= start <= end <= char-length"
                        ),
                    },
                )
                .into());
            }
            let result: String = s
                .chars()
                .skip(*start as usize)
                .take((*end - *start) as usize)
                .collect();
            Ok(Value::String(Arc::new(result)))
        }
        (OpExec::PvLen, [Value::wat__core__PersistentVector(pv)]) => {
            Ok(Value::i64(pv.len() as i64))
        }
        (OpExec::PvContains, [Value::wat__core__PersistentVector(pv), x]) => {
            Ok(Value::bool(pv.iter().any(|y| y == x)))
        }
        // Delegates to the SAME inner the interpreter calls (`runtime.rs`'s
        // `eval_persistentmap_contains_key_q` routes here too), rather than re-deriving map
        // membership — the sibling `PvGet`/`VecGet` arms below establish that shape. Its two
        // exits are audited in `vocabulary.rs`'s row comment: an unhashable key answers `false`
        // (the predicate ruling, not a sentinel), a wrong receiver raises `TypeMismatch` and is
        // refused by the checker before runtime because the row DECLARES its receiver.
        (OpExec::PmContainsKey, [m, k]) => {
            crate::collection::eval::persistentmap_contains_key_q_inner(m, k)
        }
        (OpExec::PvGet, [pv, i]) => {
            crate::collection::eval::persistentvector_get_inner(pv, i)
        }
        (OpExec::VecGet, [v, i]) => crate::collection::eval::vector_get_inner(v, i),
        (OpExec::ListGet, [v, i]) => crate::collection::eval::list_get_inner(v, i),
        (OpExec::First, [v]) => first_of(v, span),
        // The keyword converters, both delegating to the interpreter's own value-level routines so
        // an `Alias`/`Fallback` row cannot mean something different here than in core.
        (OpExec::KwToStr, [v]) => crate::runtime::keyword_to_string_value(v).ok_or_else(|| {
            EvalBreak::from(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::core::keyword/to-string".into(),
                    expected: "keyword",
                    got: Box::new(ValueSnapshot::of(v)),
                },
            ))
        }),
        // PARTIAL by design: a leading ':' or an angle-type head has no keyword. The row is
        // `Fallback`, so `CallFallback` substitutes the caller's mandatory `:undefined` value on
        // this Err — which is how the row is `total: true` without inventing an answer here.
        (OpExec::KwFromStr, [v]) => crate::runtime::keyword_from_string_value(v).ok_or_else(|| {
            EvalBreak::from(RuntimeError::new(
                span.clone(),
                RuntimeErrorKind::MalformedForm {
                    head: ":wat::core::keyword/from-string".into(),
                    reason: "a keyword's text may not start with ':' or carry an angle-type head"
                        .into(),
                },
            ))
        }),
        // `second`/`third` call the interpreter's own `positional_at`, so every container it
        // supports — Tuple included — is supported here by construction rather than by a list
        // someone remembered to keep in step. Arity is enforced at CHECK time
        // (`third` on a 2-tuple is a TypeMismatch naming "expects tuple with >= 3 element(s)"),
        // which is what makes these rows honestly `total: true`.
        (OpExec::Second, [v]) => {
            crate::runtime::positional_at(v.clone(), 1, ":wat::core::second", span)
        }
        (OpExec::Third, [v]) => {
            crate::runtime::positional_at(v.clone(), 2, ":wat::core::third", span)
        }
        (OpExec::PvNew, args) => Ok(Value::wat__core__PersistentVector(
            args.iter().cloned().collect(),
        )),
        (OpExec::VecNew, args) => Ok(Value::Vec(Arc::new(args.to_vec()))),
        (OpExec::ListNew, args) => Ok(Value::wat__core__List(Arc::new(
            args.iter().cloned().collect(),
        ))),
        // Mirrors `eval_tuple_ctor` (`runtime.rs`), including its one rule: arity 1+, because the
        // 0-tuple is the Unit `:()` and not a Tuple. The three sibling constructors above have no
        // such floor, which is why this is spelled out rather than folded in with them.
        (OpExec::TupleNew, []) => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: ":wat::core::Tuple".into(),
                reason: "tuple must have at least one element; the 0-tuple is :() (Unit)".into(),
            },
        )
        .into()),
        (OpExec::TupleNew, args) => Ok(Value::Tuple(Arc::new(args.to_vec()))),
        // The three sibling constructors above just collect; a map cannot, and the rules it must
        // follow are NOT invented here — every one is read off `eval_persistentmap_ctor`
        // (`collection/eval.rs`), which is what the interpreter runs: even arity, alternating
        // key/value, each key `value_is_key_hashable`, built with `PMap::from_pairs`. The two
        // semantic primitives are called directly rather than re-derived, so the compiled answer
        // and the interpreted one cannot drift; only argument EVALUATION differs, and the compiled
        // path has already done that.
        //
        // Found 2026-08-28 by the § 4.1 reachability ledger: this row passed admission, totality,
        // arity and type and then raised `cannot dispatch kind Unknown arity 2` at RUNTIME, exactly
        // like `PersistentMap/contains-key?` before it.
        (OpExec::PmNew, args) => {
            if !args.len().is_multiple_of(2) {
                return Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::MalformedForm {
                        head: ":wat::core::PersistentMap".into(),
                        reason: format!(
                            "arity must be even (alternating key/value pairs); got {}",
                            args.len()
                        ),
                    },
                )
                .into());
            }
            let mut pairs: Vec<(Value, Value)> = Vec::with_capacity(args.len() / 2);
            for pair in args.chunks(2) {
                if !crate::runtime::value_is_key_hashable(&pair[0]) {
                    return Err(RuntimeError::new(
                        span.clone(),
                        RuntimeErrorKind::TypeMismatch {
                            op: ":wat::core::PersistentMap".into(),
                            expected: "hashable key (primitive, HolonAST, WatAST, (HashSet :- [T]), (Vector :- [T]), or (HashMap :- [K V]))",
                            got: Box::new(ValueSnapshot::of(&pair[0])),
                        },
                    )
                    .into());
                }
                pairs.push((pair[0].clone(), pair[1].clone()));
            }
            Ok(Value::wat__core__PersistentMap(crate::value::pmap::PMap::from_pairs(pairs)))
        }
        (OpExec::Cosine, [a, b]) => {
            let Some(sym) = sym else {
                return Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::NoEncodingCtx {
                        op: ":wat::holon::cosine".into(),
                    },
                )
                .into());
            };
            cosine_outcome_from_values(a.clone(), b.clone(), span, sym)
        }
        (OpExec::Dot, [a, b]) => {
            let Some(sym) = sym else {
                return Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::NoEncodingCtx {
                        op: ":wat::holon::dot".into(),
                    },
                )
                .into());
            };
            dot_outcome_from_values(a.clone(), b.clone(), span, sym)
        }
        (OpExec::Coincident, [a, b]) => {
            let Some(sym) = sym else {
                return Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::NoEncodingCtx {
                        op: ":wat::holon::coincident?".into(),
                    },
                )
                .into());
            };
            coincident_q_from_values(a.clone(), b.clone(), span, sym)
        }
        (OpExec::Presence, [a, b]) => {
            let Some(sym) = sym else {
                return Err(RuntimeError::new(
                    span.clone(),
                    RuntimeErrorKind::NoEncodingCtx {
                        op: ":wat::holon::presence?".into(),
                    },
                )
                .into());
            };
            presence_q_from_values(a.clone(), b.clone(), span, sym)
        }
        _ => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::MalformedForm {
                head: "compiled-exec".into(),
                reason: format!("compiled apply cannot dispatch kind {kind:?} arity {}", args.len()),
            },
        )
        .into()),
    }
}

/// `first` — delegates to the interpreter's `positional_at` at index 0.
///
/// ⛔ **THIS USED TO BE A SECOND IMPLEMENTATION, AND THAT WAS THE BUG.** It matched
/// PersistentVector / Vec / List and rejected everything else, so a `Tuple` built inside a `where`
/// fence could never be read — while core's `first` has always projected a Tuple. Two routines for
/// one verb, silently disagreeing about which containers exist. Now there is one.
fn first_of(v: &Value, span: &Span) -> Result<Value, EvalBreak> {
    crate::runtime::positional_at(v.clone(), 0, ":wat::core::first", span)
}

/// The shared body of EIGHT comparison ops — `<`, `<=`, `>`, `>=` in both the generic and the
/// `f64` spellings — reduced to one `Ordering` and the caller's predicate over it.
///
/// One comparison site is the point: `i64::<` and `f64::<` cannot drift on what is comparable or
/// on how a mixed pair is refused, because there is only one place that decides.
fn ord(
    a: &Value,
    b: &Value,
    span: &Span,
    pred: impl Fn(std::cmp::Ordering) -> bool,
) -> Result<Value, EvalBreak> {
    match compare_values(a, b) {
        Some(o) => Ok(Value::bool(pred(o))),
        None => Err(RuntimeError::new(
            span.clone(),
            RuntimeErrorKind::TypeMismatch {
                op: "compiled-compare".into(),
                expected: "comparable pair",
                got: Box::new(ValueSnapshot::of(a)),
            },
        )
        .into()),
    }
}

/// Rule-compile refuse: `(:wat::rete::lower <quoted-expr>) -> nil` or raise.
pub(crate) fn eval_lower(
    args: &[WatAST],
    list_span: &Span,
    env: &crate::runtime::Environment,
    sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
    if args.len() != 1 {
        return Err(RuntimeError::new(
            list_span.clone(),
            RuntimeErrorKind::ArityMismatch {
                op: ":wat::rete::lower".into(),
                expected: 1,
                got: args.len(),
            },
        )
        .into());
    }
    let v = crate::runtime::eval_inner(&args[0], env, sym)?.value_owned();
    let ast = match v {
        Value::wat__WatAST(a) => (*a).clone(),
        other => {
            return Err(RuntimeError::new(
                args[0].span().clone(),
                RuntimeErrorKind::TypeMismatch {
                    op: ":wat::rete::lower".into(),
                    expected: ":wat::WatAST",
                    got: Box::new(ValueSnapshot::of(&other)),
                },
            )
            .into());
        }
    };
    lower(&ast, sym).map_err(LowerError::into_eval)?;
    Ok(Value::Unit)
}

#[cfg(test)]
mod rete_ops_native_coverage {
    use super::*;

    /// BRIEF-native-where-vsa-ops: the four holon rows native-lower to
    /// Call / CallFallback and must have an `OpExec` arm.
    ///
    /// ⚠ **THIS DOC USED TO SAY "`PersistentMap/contains-key?` is still Unknown — do not widen
    /// this gate into that hole", AND THAT SENTENCE WAS THE WHOLE DEFECT.** The row was fully
    /// reasoned into `RETE_OPS` (its two exits audited in the table's own row comment) and then
    /// never wired here, so it passed admission, totality, arity and type — and raised
    /// `#wat.runtime/MalformedForm "compiled apply cannot dispatch kind Unknown arity 2"` at
    /// RUNTIME, inside a `where` fence, for any user who wrote it. A comment instructing a gate
    /// not to look is not a scope note; it is an unowned deferral with no re-read (FM 23), and
    /// nothing would ever have surfaced it.
    ///
    /// Found 2026-08-28 by the § 4.1 reachability ledger (`rete/reachability.rs`), which drives
    /// each row rather than reading about it. Four more of the same shape were found the same day:
    /// the `PersistentMap` CONSTRUCTOR (fixed, `PmNew`), `reduce` (fixed, `exec_reduce` — a mirror
    /// of `wat/seq.wat:317-329`, where 3-arity reduce IS `foldl`), and `map`/`filter`/`Tuple`,
    /// which need a ruling rather than an arm.
    ///
    /// ⛔ **THIS GATE IS NOW THE NARROW ONE, AND THAT IS FINE — DO NOT WIDEN IT HERE.** Not for the
    /// old reason (a hole nobody wanted to look at) but because the general question is answered
    /// STRICTLY BETTER next door: `reachability.rs` DRIVES every row and requires a verdict, where
    /// this can only ask whether an `OpExec` arm exists. Arm-existence is the wrong question — it
    /// is neither necessary (`foldl` maps to `Unknown` and reaches the executor by its own route)
    /// nor sufficient (an arm can exist and the row still be unwritable in every position). Keep
    /// this one as the cheap holon-specific check it has always been; the wall is the ledger.
    #[test]
    fn holon_rete_ops_have_opexec() {
        let mut missing = Vec::new();
        for row in RETE_OPS {
            if row.rete_name.starts_with(":wat::rete::holon::")
                && matches!(OpExec::of(row.core_name), OpExec::Unknown)
            {
                missing.push(row.rete_name);
            }
        }
        assert!(
            missing.is_empty(),
            "native apply_op has no OpExec for holon row {missing:?}"
        );
    }
}


/// DISCONFIRMING PROBE for fix-list entry **F** — can a lowered `Expr::Call` be evaluated against
/// an ALPHA slot frame?
///
/// Entry F is: an inline constraint whose operand is a nested call is accepted everywhere, runs,
/// and matches nothing — silently. Three places conspire, and the fix hinges on ONE assumption:
/// that `compiled_cond`'s slot frame and this module's `exec` are the same thing. If they are, the
/// fix is to finish flip 3 (lower the operand through the core); if they are not, the whole
/// approach dies here and a different one is needed.
///
/// `compiled_cond::SlotFrame` is `Vec<Option<Value>>`; `exec` takes `&mut [Option<Value>]`. This
/// probe asserts they compose in fact and not merely in type: lower a nested call whose operand is
/// a `?var`, put a value in that slot by hand the way an `Op::Bind` prologue would, and demand the
/// arithmetic.
///
/// ⚠ What it does NOT settle, deliberately: `exec` requires a `&SymbolTable` and NO alpha executor
/// signature carries one — the per-fact hot path is sym-free on purpose. That is the real obstacle
/// and it is a separate decision (thread it, or refuse the sym-needing ops at compile time). This
/// probe uses a bare world's symbols to isolate the frame question from the sym question.
#[cfg(test)]
mod entry_f_frame_composition {
    use super::*;

    #[test]
    fn a_lowered_call_evaluates_against_a_bare_alpha_style_slot_frame() {
        let world = crate::freeze::startup_bare().expect("bare world");
        let sym = world.symbols();

        // `(:wat::rete::core::i64::+ ?x 2 :undefined 0)` — the exact shape an inline constraint
        // operand takes: a Fallback row with the mandatory `:undefined` marker pair.
        let src = "(:wat::rete::core::i64::+ ?x 2 :undefined 0)";
        let forms = crate::parser::parse_all_with_file(src, "<entry-f-probe>")
            .expect("the probe expression must parse");
        let expr_ast = forms.first().expect("one form");

        let program = lower(expr_ast, sym).expect("a nested rete call must lower through the core");

        // The alpha prologue's job, done by hand: `?x`'s slot holds 10, exactly as `Op::Bind`
        // would have written a field value into `scratch`.
        let slot = program
            .reads
            .iter()
            .find_map(|(name, s)| match name {
                Value::String(n) if n.as_str() == "?x" => Some(*s),
                _ => None,
            })
            .expect("the lowered program must read `?x` from a slot");
        let mut frame: Vec<Option<Value>> = vec![None; program.frame_len as usize];
        frame[slot as usize] = Some(Value::i64(10));

        let got = exec(&program.root, &mut frame, &program.names, sym, &expr_ast.span().clone())
            .expect("exec must evaluate the call against the frame");

        assert_eq!(
            got,
            Value::i64(12),
            "10 + 2 = 12. If this fails, `compiled_cond`'s slot frame and this module's `exec` do \
             NOT compose, and entry F's fix cannot be 'finish flip 3' — it needs a different shape"
        );
    }
}
