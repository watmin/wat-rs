# STONE O-iii — the macro generates both doors from one declaration

> Read `DESIGN-STONE-O-one-declaration-feeds-both-doors.md` first. It carries the two defects, the
> measured populations, and the one contract decision. This brief is the strike.

## The work

`#[wat_intrinsic]` learns a third sniff. Today it reads the handler's **argument shape**
(`sniff_args`: `&WatAST` ⇒ Exact, `&[WatAST]` ⇒ Variadic) and its **return shape** (`sniff_return`:
`Value` vs `TrackedValue`). You add the **kind** sniff on the same mechanism: a handler whose leading
params are `&Value` (or one `&[Value]`) is **ALGEBRA**, and the macro generates BOTH doors from it —
the `NativeHandler` AST shim it already generates, plus the `ValueHandler` the `apply` path needs,
**both guarded by one generated arity check**. A handler whose leading params are `&WatAST` (or one
`&[WatAST]`) is **BINDING**: unchanged in every respect, exactly as it compiles today.

Then you migrate ONE namespace to prove it: `src/intrinsic/vector.rs`, six verbs. Five of them
(`length`, `empty?`, `contains?`, `get`, `conj`) have no value door today and gain one. The sixth
(`concat`) is written today as **two** fns — a shell and a hand-written value twin, both calling the
same `persistentvector_concat_inner` — and becomes **one**.

## Row 0 — before anything else, name the handler the census cannot see

`wat-scripts/hunt/stone-o-shell-census.awk` classifies **380** handlers. The registry holds **381**
names:

```bash
find src -name '*.rs' -exec cat {} + | tr '\n' ' ' \
  | grep -oP '#\[wat_intrinsic\(\s*"\K[^"]+' | grep -v '<fqdn>' | sort -u | wc -l     # 381
find src -name '*.rs' -print0 | xargs -0 awk -f wat-scripts/hunt/stone-o-shell-census.awk | wc -l  # 380
```

**Find the one handler the awk does not reach and say what shape defeated it.** It is one command's
difference between two lists, and it decides whether the census's SHELL/BINDING split has an
unexamined edge. Report the name and the shape; do not adjust any count until you have it.

## Rooms — verified against `9b25f3bbf`

```
crates/wat-macros/src/wat_intrinsic.rs:90    enum SniffedArgs      — the shape you extend
crates/wat-macros/src/wat_intrinsic.rs:102   fn sniff_args         — where &WatAST / &[WatAST] are recognised
crates/wat-macros/src/wat_intrinsic.rs:168   enum SniffedReturn    — Stone G's precedent: a second axis, same file, same style
crates/wat-macros/src/wat_intrinsic.rs:181   fn sniff_return       — copy its compile_error! voice for the new rejections
crates/wat-macros/src/wat_intrinsic.rs:251   fn is_ref_watast      — the predicate to mirror for &Value
crates/wat-macros/src/wat_intrinsic.rs:259   fn is_ref_watast_slice—     "        "        "     for &[Value]
crates/wat-macros/src/wat_intrinsic.rs:371   value_handler_field   — the `value = <path>` slot; it STAYS (19 arithmetic pairs still use it)
crates/wat-macros/src/wat_intrinsic.rs:531   let shim_body         — where the AST door is built; the value door is built beside it
crates/wat-macros/src/wat_intrinsic.rs:545   the ArityMismatch     — the exact error shape BOTH doors must now raise
crates/wat-macros/src/wat_intrinsic.rs:579   value_handler: field  — what the generated adapter is submitted as

src/intrinsic/mod.rs:162   NativeHandler  — fn(&[WatAST], &Span, &Environment, &SymbolTable) -> Result<TrackedValue, EvalBreak>
src/intrinsic/mod.rs:198   ValueHandler   — fn(&[Value]) -> Result<Value, EvalBreak>   ← what you generate
src/intrinsic/mod.rs:200   IntrinsicSubmission — no new field is needed; `value_handler` is already the record

src/intrinsic/vector.rs                 the proof namespace — all six verbs
src/intrinsic/vector.rs:214             the hand-written value twin that disappears
src/collection/eval.rs:829              persistentvector_length_inner(v: &Value) — the algebra ALREADY has the right signature
```

★ **The algebra you need already exists with the right signature.** Every `:wat::vector::` verb's
shell ends in a `crate::collection::eval::persistentvector_*_inner(&Value…)` call. The migration is
mostly deletion: the shell's body becomes the `_inner` call and its params become `&Value`.

## Implementation sketch

The new sniff, mirroring `sniff_args`:

```rust
enum IntrinsicKind {
    Binding,                      // leading &WatAST / &[WatAST] — today's shape, untouched
    Algebra(SniffedArgs),         // leading &Value  / &[Value]   — both doors generated
}
```

What ALGEBRA emits — one arity check, two shims, one user fn:

```rust
// the value door: what `apply` reaches through dispatch_substrate_impl
fn __wat_intrinsic_value_persistentvector_length(vals: &[Value]) -> Result<Value, EvalBreak> {
    if vals.len() != 1 { return Err(/* the SAME ArityMismatch shape as line 545 */); }
    persistentvector_length(&vals[0])
}
// the AST door: eval each arg, then reuse the value door — one implementation, not two
fn __wat_intrinsic_shim_persistentvector_length(
    args: &[WatAST], list_span: &Span, env: &Environment, sym: &SymbolTable,
) -> Result<TrackedValue, EvalBreak> {
    if args.len() != 1 { return Err(/* ArityMismatch, as today */); }
    let vals: Vec<Value> = args.iter()
        .map(|a| crate::runtime::eval_inner(a, env, sym).map(TrackedValue::value_owned))
        .collect::<Result<_, _>>()?;
    __wat_intrinsic_value_persistentvector_length(&vals).map(TrackedValue::from)
}
```

and submits `value_handler: Some(__wat_intrinsic_value_…)`.

A migrated verb, in full:

```rust
/// … the doc comment is unchanged: same @added/@arg/@ret/@example …
#[wat_intrinsic(":wat::vector::length")]
pub(crate) fn persistentvector_length(v: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::persistentvector_length_inner(v)
}
```

⚠ **`@arg` names are checked against the signature's param idents** (`wat_doc::check_args`,
`wat_intrinsic.rs:410`). If you rename a param while migrating, the doc's `@arg` name moves with it
or the macro rejects the file. Keep the existing names and this never bites.

⚠ **The `_span` params disappear.** Five of the six carry
`_span: &Span, // rune:lint(unused-span)`; an ALGEBRA fn takes no span at all, so both the param and
its rune go. That rune exists to justify an unused param; with no param it has nothing to justify.

## Blast radius

`crates/wat-macros/src/wat_intrinsic.rs` and `src/intrinsic/vector.rs`. Nothing else.
No new registry field, no `src/runtime.rs` edit, no change to `dispatch_substrate_impl`, no change to
any BINDING handler, and no change to the `value = <path>` attribute — it stays for the 19
arithmetic pairs that are out of this strike's scope.

## STOP triggers — each REJECTS. Ship nothing and report the gap.

1. **A `&Value`-leading fn that also takes `env`, `sym`, or a `&Span`.** That is a contradiction, not
   a shape to accommodate: algebra by definition needs none of them. Emit a `compile_error!` naming
   the seam, in `sniff_return`'s voice. If an existing `:wat::vector::` verb turns out to need one,
   STOP — it is BINDING and the design's population is wrong.
2. **A `&Value`-leading fn returning `Result<TrackedValue, _>`.** Ruled out by the design's first
   affirmative cut: `ValueHandler` returns a bare `Value`, so the stamp could not survive the value
   door. Emit a `compile_error!` saying a provenance-stamping handler is BINDING. Do not add a
   provenance-carrying value door.
3. **Mixed `&Value` and `&WatAST` params in one signature.** Reject at compile time, the way
   `sniff_args` already rejects mixing `&[WatAST]` with `&WatAST`.
4. **Any migrated verb whose behaviour changes.** The direct call's output — value AND error text —
   must be byte-identical before and after for every one of the six. If any differs, STOP and report
   the difference; do not adjust the expectation.
5. **The generated arity error is not the same shape as the AST door's.** Same
   `RuntimeErrorKind::ArityMismatch`, same `op` string, same `expected`/`got`. A *different* error on
   the value door re-creates the split this stone exists to close. If you cannot reach line 545's
   exact shape from the value adapter, STOP and say why.
6. **Row 0 unanswered.** If you cannot name the 381st handler, report that as the finding and stop
   before touching the macro. A census with an unexplained edge is not a census.

## Acceptance — run each, report the actual output

```
 0. ★ THE 381ST HANDLER IS NAMED. The name, the file, and the signature shape that defeated the awk.

 1. ★ ONE DECLARATION, BOTH DOORS. For :wat::vector::length — which has NO value door today —
    prove the value door now exists and is served by the registry:
      ./target/release/wat wat-scripts/scratch-pad/255-stone-o-apply-lies-about-what-exists.wat
    The `:wat::vector::length` row must flip  APPLY=err:unknown function  ->  APPLY=ok:3.
    Every other row must be UNCHANGED — including the two that still say `unknown function`
    (`max-of`, `to-uppercase`, `sqrt`), which are O-iv's, not this strike's.

 2. ★ PROVE IT BY SABOTAGE, ON THE THING ITSELF. Make `persistentvector_length` return a wrong
    constant. Show BOTH doors return the sabotaged value — direct AND apply — then restore.
    Confirm the edit LANDED before reading the output: a no-op probe prints a meaningless green,
    and sabotaging anything OTHER than the fn under test answers a different question than the
    one you are asking.

 3. ★ THE MIGRATED VERB GUARDS ITS OWN ARITY. On today's tree
    `(apply :wat::vector::concat [one-pv])` panics at `src/intrinsic/vector.rs:214`.
    After the migration it must return an ArityMismatch whose text matches the direct call's.
    Show BOTH, side by side. ⚠ If Stone O-i (the central guard in `dispatch_substrate_impl`)
    has already landed, the panic is gone for every verb and this row proves LESS than it
    looks: say so, and prove the GENERATED check fires by removing the central guard for one
    run, or by pointing at the generated code. A row that passes for someone else's reason is
    not evidence.

 4. ★ TWO FNS BECAME ONE. `git diff --stat src/intrinsic/vector.rs`, plus the count of
    `expect("arity-checked")` in that file before and after (1 -> 0).

 5. BINDING IS UNTOUCHED. `cargo build --release --all-targets` is clean, and the count of
    `#[wat_intrinsic` sites is unchanged at 381 — this strike moves verbs between kinds, it does
    not add or remove any.

 6. THE SIX STILL ANSWER IDENTICALLY. For each of the six verbs, the direct call before and after,
    value and error text, byte-identical. Paste both columns.

 7. cargo nextest run --release -E 'binary_id(wat::wat_lang)' and any test naming vector/apply.
    Report the Summary line verbatim.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. Verify with `pwd` first. Any path containing
  `.claude/worktrees/` is harness state — never operate on it.
- Everything in the FOREGROUND. Your turn ends when the numbers are in your hands, not when a
  command is launched.
- You may run `cargo build`, `./target/release/wat --check`, `./target/release/wat <file>`, and a
  scoped `cargo nextest run --release -E '<filter>'`. **The orchestrator runs the full floor and
  clippy centrally** — leave those alone.
- You may not spawn sub-agents.
- Do not commit, push, stash, revert, or create a worktree. Leave the tree dirty; the orchestrator
  weighs and commits.
- If a number surprises you, report the surprise. A brief that turns out to be wrong is the most
  useful thing you can hand back — the last three stones in this arc were each corrected by a rider
  catching a defect in my own brief, and one of them refused an order that would have deleted live
  code.

## Report back with

Row-by-row: the command you ran, its actual output, PASS/FAIL. Then the honest deltas — what
surprised you, what the brief got wrong, what you had to decide that the brief did not settle.
