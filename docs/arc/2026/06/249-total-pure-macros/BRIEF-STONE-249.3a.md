# BRIEF — Stone 249.3a — eval-time quasiquote: purity fence + `~@`-splice + `List?` form predicate

**Arc:** 249 (total-pure-macros). **Design:** `DESIGN-STONE-249.3.md` §2.1 + §2.2 + §2.3.
**Probe substrate (mirror this):** `tests/probe_arc249_threading_in_wat.rs` (rows A, B, C, E) + `tests/probe_arc249_macro_engine.rs` (gates A–E, the engine contract).
**You write substrate Rust. Do NOT commit. Do NOT run git. Do NOT touch any wat/ file.**

---

## The goal — three changes that make the macro engine threading-ready

The 249.2b body-model made macro bodies run through `runtime::eval`; a program body's *inner* quasiquote is walked by eval-time `walk_quasiquote` (`src/runtime.rs`). Three things are wrong/missing for a wat macro to manipulate forms safely, and they ship together (same two files):

1. **PURITY FENCE (load-bearing).** A program-body quasiquote currently runs computed unquotes UNFENCED at expand time — `` `(:wat::core::not ~(:wat::kernel::stopped?)) `` *executes the kernel call while building*. Close it.
2. **`~@`-SPLICE.** Eval-time `walk_quasiquote` has no unquote-splicing support, so `~@step` silently leaves the literal symbol in the output. Add it.
3. **`List?` FORM PREDICATE.** A wat macro must branch on step shape (list step → inject acc; bare symbol → wrap). No predicate over form-values exists. Mint `:wat::core::List?`.

When all three land, the macro engine can run *safe, form-manipulating* programs — and threading (`->`/`->>`, stone 249.3b) becomes ~8 lines of wat.

---

## Change 1 — the purity fence (in `src/macros/eval.rs`)

`validate_pure_total` (eval.rs:92) is the macro engine's DEFAULT-DENY gate. Today its quasiquote handling blanket-returns `Ok(())` (eval.rs:99) — it skips the ENTIRE quasiquote. But a quasiquote is **data** (the literal template) PLUS **code** (the `~`/`~@` computed expressions). The validator must check the code and skip the data.

**Do:** when `validate_pure_total` reaches a `(:wat::core::quasiquote X)` head, instead of returning `Ok(())` immediately, **walk the template `X` tracking quasiquote depth** (exactly as `walk_quasiquote` in runtime.rs tracks it): nested `(:wat::core::quasiquote …)` bumps depth +1; an `(:wat::core::unquote E)` or `(:wat::core::unquote-splicing E)` that *fires at depth 1* means `E` is real code — recurse `validate_pure_total(E)`. Material below depth 1 (inside a nested quasiquote, above its own unquote) stays data — skip it. `(:wat::core::quote …)` stays fully skipped (pure data, never evaluated).

**Why this location, not a flag through walk_quasiquote:** `validate_pure_total` runs ONLY via `macro_eval` (eval.rs:73). Fixing it here fences the MACRO context exclusively. Runtime quasiquotes (e.g. `eval-ast!` payloads) reach `walk_quasiquote` through plain `eval`, never through `macro_eval`, so they keep FULL power, untouched. No context flag to plumb.

**Result:** an impure computed unquote in a program-body quasiquote → `MacroErrorKind::RefusedInMacro { head }` (the existing error, eval.rs) at validate time, BEFORE eval. The load-bearing invariant doc-comment at the top of eval.rs should gain a line noting validate_pure_total descends into unquote sub-expressions (the fence covers computed unquotes structurally).

---

## Change 2 — `~@`-splice at eval-time (in `src/runtime.rs`)

`~@X` parses to `(:wat::core::unquote-splicing X)` (`src/parser.rs:320`). Eval-time `walk_quasiquote` (`src/runtime.rs:10380`) handles `~` (unquote, 10400-10412) but its plain-list child-walk (10417-10419) does NOT handle splice.

**The working precedent to MIRROR:** expand-time `splice_argument` (`src/macros/expand.rs:1097`) and the list-child splice in `walk_template`/`walk_quasiquote`'s expand-time sibling (expand.rs ~893-919). Study how it flattens spliced forms into the parent list at the OUTER-list level.

**Do:** in `walk_quasiquote`'s `WatAST::List` arm, where children are currently mapped 1:1 (10417-10419), detect any child that is `(:wat::core::unquote-splicing E)` *at depth 1* and **flatten** its result into the parent's child-vector (1-to-N), rather than producing a single nested node. Evaluate `E` (via the same `eval_inner` the unquote arm uses — it is now fenced by Change 1) and splice by value:
- `Value::Vec(elems)` → `value_to_watast` (runtime.rs, used by the unquote arm at 10403) each element; splice all. *(Mirrors `splice_argument`'s computed-Vec case.)*
- `Value::wat__WatAST(list)` where the inner AST is a `WatAST::List` → splice the inner list's **children** (this is the threading case: `~@step` where `step` is a list-form value → splice `(f a b)`'s children).
- any other value shape → a located error (use an existing `RuntimeErrorKind` — e.g. `TypeMismatch` with op `",@"` / `:wat::core::unquote-splicing`, "requires a sequence (Vec value or list form); got <shape>"). Honest refusal, not a silent pass.

Depth handling mirrors the existing unquote arm: splice fires only at depth 1; below depth 1 the `(:wat::core::unquote-splicing …)` wrapper is preserved and depth-peeled like the nested-unquote case.

---

## Change 3 — the `:wat::core::List?` form-shape predicate (intueri-named)

Mint a pure-total unary predicate `:wat::core::List?` over a form-value: given a `Value::wat__WatAST`, return `:wat::core::bool` — `true` iff the wrapped node is `WatAST::List`. (Name is the intueri cast's verdict — grounded: it inspects a core-minted `wat__WatAST` value, so it lives in `:wat::core::`, NOT `:wat::holon::`.)

**Three edits:**
1. **Impl** — a small `eval_list_q`-style fn in `src/runtime.rs`: eval the single arg to a `Value`; `Ok(Value::bool(matches!(v, Value::wat__WatAST(ast) if matches!(&**ast, WatAST::List(..)))))`; arity-check (1 arg) with an existing `RuntimeErrorKind::ArityMismatch`.
2. **Dispatch** — add the `":wat::core::List?" =>` arm to `dispatch_keyword_head_value` (the value-position dispatcher; near the existing `record?`/`empty?` predicate arms), routing to the impl.
3. **Allow-list** — add `":wat::core::List?"` to `is_pure_total` in `src/macros/eval.rs`, **co-located in the form-ops cluster** (alongside `:wat::core::forms` / `:wat::core::struct->form` near eval.rs:352-356), NOT in the type-inspection block.

**MARK THE SOURCE (load-bearing — intueri Level-1 catch):** at BOTH the dispatch arm and the allow-list entry, add a one-line comment noting the deliberate divergence from `:wat::holon::is-List?`:
> `// core form-shape predicate over WatAST::List; distinct from :wat::holon::is-List? (a classifier over HolonAST). The name diverges on purpose — the form-vs-holon distinction is the reason this exists. Do not "harmonize" the two names.`

This prevents a future reader from collapsing the two predicates (they answer about different value types via different mechanisms — `WatAST` enum-variant vs `HolonAST` classifier-string).

---

## Verification (the scorecard — verify every row yourself, do not self-report green)

Run all from `/home/watmin/work/holon/wat-rs/`:

1. **Purity fence works** — `cargo test --release --test probe_arc249_threading_in_wat -- --ignored` : row **E** (`diag_program_body_quasiquote_impure_unquote_fenced`) now PASSES (startup refused). Then un-`#[ignore]` row E and confirm it passes under a plain `cargo test --release --test probe_arc249_threading_in_wat`.
2. **Splice works (thread-last)** — rows **A** (`diag_thread_last_single_step`) and **B** (`diag_thread_last_pipeline`) now PASS. Un-`#[ignore]` A and B; confirm green plain.
2b. **`List?` predicate works** — row **C** (`diag_is_list_over_form`) now PASSES (`List?` true for a list form, false for an int form). Un-`#[ignore]` C; confirm green plain. (Row **D** is a diagnostic — leave as-is.)
3. **Engine contract intact** — `cargo test --release --test probe_arc249_macro_engine` : gates A–E all still green (no regression to the bare-quasiquote path or the pure program-body paths). This is the FM-9 baseline — adjacent gates must not rot.
4. **Library green** — `cargo test --release --lib -p wat` : the pre-existing pass count holds (was 898/0/1 at the 249.2b engine; confirm no drop).
5. **Clippy clean** — `cargo clippy --release -p wat` : zero NEW warnings from your changes (the macros home + runtime touched lines).

Report each row with the actual command output. If any pre-existing test goes red, STOP and report it as a finding (it is data about the fence's reach) — do not work around it.

## Notes
- Bash + cargo work in this workspace; use them freely.
- Keep changes minimal and located: `src/macros/eval.rs` (the fence + the `List?` allow-list entry) + `src/runtime.rs` (the splice + the `List?` impl & dispatch arm). No new files, no wat/ edits, no new error variants (reuse `RefusedInMacro` + existing `RuntimeErrorKind`s).
- Mirror the EXISTING expand-time splice semantics (`splice_argument`) — you are porting proven logic to the eval-time walker, not inventing a new splice model.
- The `List?` source-divergence comment (Change 3) is load-bearing — do not omit it.
