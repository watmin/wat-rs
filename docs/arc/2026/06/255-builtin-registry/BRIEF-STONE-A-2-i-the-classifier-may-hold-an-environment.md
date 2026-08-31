# BRIEF — STONE A-2-i: the classifier may hold an environment

Give `src/rete/purity.rs`'s axis classifier an optional environment so it can resolve a head that
names a **local binding holding a closure** — today such a head default-denies, which is why
`sort-by`'s comparator `(fn [a b] (< (keyfn a) (keyfn b)))` classifies impure. Nothing consumes the
new capability in this stone. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-A-2-i-the-classifier-may-hold-an-environment.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, and no notification is coming.
Make text edits and report; your turn ends when your report is written. The orchestrator builds,
floors and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or
`scripts/floor.sh`. You may run `target/release/wat --check <file>` and `target/release/wat <file>`
against the EXISTING binary for a fast read, remembering it will not contain your Rust changes.
**You may not spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd`
first. Do not commit, push, stash, revert, or `git checkout --` anything.

## Read in order

1. The DESIGN above — the pinned contract decision, the hazard, and what is affirmatively cut.
2. `src/rete/purity.rs`, `fn head_ok` — the resolution ladder: constructor → field accessor →
   `sym.has_function` → rete namespace → `intrinsic_meta` → default-deny. Your insertion point is
   **immediately before that final default-deny**.
3. `src/rete/purity.rs`, `fn classify_expr`'s general `WatAST::List(items, _)` arm — the head is read
   as `Some(WatAST::Symbol(id, _)) => id.as_str()`. That is how `keyfn` arrives.
4. `src/rete/purity.rs`, `fn classify_fn` — the existing recursion guard: `seen: HashSet<String>`
   keyed on FQDN via `sym.get(fqdn)`. Note it cannot key an anonymous closure.
5. `src/value/environment.rs` — `Function.closed_env: Option<Environment>` and
   `Environment::lookup(&self, name: &str, head_span: &Span) -> Option<TrackedValue>`.
6. `src/value/value.rs:67` (`wat__core__fn(Arc<Function>)`) and `:684` (`Arc::ptr_eq` — the
   codebase's existing fn-identity idiom, which is your cycle guard).
7. `src/intrinsic/rete.rs:111` — `eval_rete_pure_intrinsic(expr, env, sym)` already receives an
   `env`; today it does not pass it down. That is what makes the capability observable from wat.

## The work

### 1 — thread the environment

Add `env: Option<&Environment>` to `classify_expr`, `head_ok`, and `classify_fn`. Every existing
call site inside `src/rete/purity.rs` (there are 19 for `classify_expr`) passes **`None`**, except
where you are deliberately propagating it down a walk. `find_axis_violation` keeps its current
signature and passes `None`; add an env-carrying sibling beside it.

**`None` must reproduce today's behaviour exactly.** That is the contract decision and it is what
makes the floor a control for this stone.

### 2 — resolve the capture

In `head_ok`, immediately before the final default-deny: if `env` is `Some` and
`Environment::lookup(head, at)` yields a `Value::wat__core__fn(f)`, classify `f`'s body against the
same axis — carrying **`f.closed_env`**, that function's own captured environment, not the caller's.
A `FunctionBody::Native` resolves the way `classify_fn`'s native arm already does (consult
`intrinsic_meta`; default-deny an unproven native).

If the lookup yields nothing, or yields a non-fn value, fall through to today's default-deny
unchanged.

### 3 — the cycle guard

An anonymous closure has `name: None` and is absent from `sym`, so the FQDN-keyed `seen` cannot hold
it. Guard the closure walk on the `Arc<Function>` pointer address, in a set kept separate from the
FQDN `seen` — mirroring `src/value/value.rs:684`'s existing `Arc::ptr_eq` fn-identity idiom. A
back-edge returns `Ok(())`, exactly as `classify_fn`'s FQDN back-edge does.

### 4 — let the wat predicates see it

Have `:wat::rete::pure?` / `deterministic?` / `total?` pass their own `env` down
(`src/intrinsic/rete.rs`). This is what makes the capability observable and is the stone's proof
surface.

### 5 — the probe

Write `wat-scripts/scratch-pad/255-probe-the-classifier-follows-a-capture.wat` with **both** rows,
because both are load-bearing:

- a **pure** `keyfn` bound in an enclosing `let`, asked through `pure?` about the `sort-by`-shaped
  comparator → must print `true` (it prints `false` today);
- an **effectful** `keyfn` bound the same way → must print `false`.

Row 2 proves the capability was added *without widening*. A classifier that started answering `true`
for everything passes row 1 and is the failure this stone must not ship.

## Blast radius

`src/rete/purity.rs` · `src/intrinsic/rete.rs` · new
`wat-scripts/scratch-pad/255-probe-the-classifier-follows-a-capture.wat`. No new types, no changes
to any verb's registration, no changes to `src/freeze.rs`, no changes to any `sort` site.

## STOP triggers — each rejects; ship nothing further on that point and report

**STOP-1 — a depth bound is not the guard.** If pointer identity turns out not to be reachable,
STOP and report. Do NOT substitute a recursion-depth limit: it returns "not proven" for "did not
look far enough", and this classifier's `false` must mean *proven not*.

**STOP-2 — the negative control must not move.**
`wat-scripts/scratch-pad/255-probe-the-classifier-cannot-see-through-a-closure.wat` asks about a
comparator whose `keyfn` is bound nowhere. It must still print `true` / `false` / `false`. If your
change makes its middle row `true`, the classifier is resolving something it cannot see — STOP and
report.

**STOP-3 — `None` is not a behaviour change.** If making an existing call site pass `None` requires
changing what it returns, the change is not additive. STOP and report which site and why.

**STOP-4 — do not widen to make a row pass.** If row 1 of the new probe will not go `true` without
admitting a head you cannot actually resolve, STOP and report the shape you hit.

## Report

Give the per-file diff summary, the exact output of both probe rows and of the negative control, and
how you guarded the cycle. Then the part the orchestrator cannot reconstruct: what surprised you —
a call site where `None` was not obviously right, a resolution path the design did not predict, or a
place where the threading read wrong.
