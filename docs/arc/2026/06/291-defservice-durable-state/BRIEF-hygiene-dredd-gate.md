# BRIEF — the hygiene Dredd gate: `HygieneScopeDivergence`, through the `src/scope/` home

**You are a LEAF executor. Model: sonnet. Work ONLY in `/home/watmin/work/holon/wat-rs/`. Do NOT spawn
subagents. Do NOT use git worktrees.** If the work exceeds these rooms or hits a STOP trigger, STOP and
report — do not improvise a workaround.

## The work, in one paragraph

The type checker silently swallows unbound local symbols — `check.rs:3416`, the `WatAST::Symbol` arm:
`locals.get(env_key(ident))` returns `None` and the checker returns `fresh.fresh()` ("silent-by-intent"). That
swallow hides a real failure class: a **hygiene-scope divergence** — a reference whose `env_key` misses a binder
that exists in `locals` under the **same name but a different hygiene scope**. This is produced when a macro
rebuilds a binder from its *name string* (stripping/changing its `ScopeId`) instead of reusing the original
node — so the binder (`a@{433}`) and the reference (`a@{}`) no longer match, and the program dies at runtime
with a cryptic `UnboundSymbol("a")`. **Make it a compile-time refusal.** Add a `HygieneScopeDivergence` check
error; put the *detection logic in the `src/scope/` home* (where `env_key` lives); `check.rs:3416` gets only a
**thin call**. Then **the gate becomes the worklist** — it fires at check time on every existing violator;
**sweep** each by the doctrine (reuse the node, never rebuild a binder from its name) until the build is green.

## The proof this rests on (already on disk — do not re-litigate)

Instrumenting `check.rs:3416`'s `None` arm on `tests/probe_kwargs_emitted_by_macro.rs` printed, at **check time**:
```
[CHECK-DBG] MISS local 'a' env_key="a" scopes={} ; same-name binders in locals: ["a\u{1}433"]
```
The checker SEES the reference `a@{}`, SEES that a binder `a@{433}` exists, and swallows the miss. The crime is
visible at compile time; the checker looks away. Your gate makes it look.

## The contract (pinned)

1. **New error variant** in `src/check/error.rs` (the `CheckErrorKind` enum, `:36`):
   ```rust
   HygieneScopeDivergence { name: String, ref_key: String, binder_key: String },
   ```
   Display + EDN render: mirror the existing variants (e.g. `TypeMismatch`). Message must name the symbol and
   both scopes, e.g. *"hygiene-scope divergence: reference `a` (scope {}) is unbound, but a binder `a` exists
   under a different scope {433} — a macro rebuilt this binder from its name instead of reusing the node; reuse
   the original AST node."* Carry the span of the reference.

2. **Detection helper in the `src/scope/` home** — `src/scope/resolution.rs` (where `env_key` lives, `:79`):
   ```rust
   /// A hygiene-scope divergence: `ident` is unbound (its env_key missed), but a binder of the SAME NAME
   /// exists under a DIFFERENT hygiene scope. Only ever a faulty macro that rebuilt a binder from its name
   /// (changing its ScopeId) — never a legitimate polymorphic placeholder. Returns the diverging binder's key.
   pub fn scope_divergent_binder<'a>(
       ident: &Identifier,
       local_keys: impl Iterator<Item = &'a str>,
   ) -> Option<String>
   ```
   Logic: let `me = env_key(ident)`. For each key `k`, take its NAME part (everything before the first
   `'\u{1}'`, or the whole key if none). If `name_part(k) == ident.as_str()` AND `k != me`, return `k.to_owned()`
   (a same-name, different-scope binder). Else `None`. (Bare key `"a"` and scoped key `"a\u{1}433"` share the
   name part `"a"`.) Add a unit test in `resolution.rs`'s test module covering: bare-ref vs scoped-binder →
   Some; scoped-ref vs bare-binder → Some; same key → None; different name → None.

3. **The thin call at `check.rs:3416`** — the `None` arm only:
   ```rust
   None => {
       match crate::scope::resolution::scope_divergent_binder(ident, locals.keys().map(|s| s.as_str())) {
           // "I AM THE LAW." — Dredd. A binder of this name exists under a different hygiene scope:
           // this reference can never bind it. A macro rebuilt the binder from its name instead of
           // reusing the node. Refuse at compile time — the program does not get to run.
           Some(binder_key) => {
               local_errors.push(CheckError { span: <ref span>, kind: CheckErrorKind::HygieneScopeDivergence {
                   name: ident.as_str().to_owned(),
                   ref_key: crate::scope::resolution::env_key(ident).into_owned(),
                   binder_key,
               }});
               CheckResult::with_errors(...)   // mirror how this fn surfaces errors (see the Some/None shape nearby)
           }
           // genuinely unbound (no same-name binder) → the existing polymorphic-placeholder behavior, untouched
           None => CheckResult::ok(fresh.fresh()),
       }
   }
   ```
   ⚠ The `WatAST::Symbol(ident, _)` arm currently drops the span (`_`). You need the ref span for the error —
   change the pattern to bind the span (`WatAST::Symbol(ident, sp)`) and use it. Confirm the error-surfacing
   shape against how this function (`infer`?) returns errors — match the local convention (`local_errors` vs a
   returned `CheckResult`); read the surrounding ~80 lines first.

## The doctrine the sweep enforces (extirpare — the failure class)

**A definition-emitting macro NEVER reconstructs a binder (or a binder-matching reference) from a name string
(`symbol-node(ast-name x)` / `Identifier::bare(name)` where the name came from a scoped node). It REUSES the
original AST node, which carries the scope.** Names → strings (for keyword accessors). Binders → nodes (reused).
Rebuilding a binder from its name strips its scope and is the crime the gate catches.

## The sweep (the gate is the worklist)

After the gate compiles, **run the build + the wat stdlib load + the test suite**. The gate now fires at check
time on every violator. Each `HygieneScopeDivergence` names a site. Fix each at its SOURCE by reusing the node:
- **`wat/core.wat`** kwargs `defn` branch — the `$impl` let-binder (`~429`, already changed `symbol-node fname-str`
  → `fname-node`; verify it's correct) AND audit the field-name flow into the Record::def.
- **`wat/Record.wat`** — the emitted constructor `(fn [fields] (Record::of … [fields]))`: build the param
  binders and the body references from the SAME nodes (consistent scope), or strip both to bare consistently.
- **`src/runtime.rs:1344`** `register_record_methods` auto-gen ctor — uses `Identifier::bare` for body refs AND
  string params; confirm they're consistent (bare both sides is fine — that's consistent).
- Audit `defstruct` / `defenum` / `defservice` / `rete` emitters for the same antipattern.

Fix at the macro/emitter, NEVER by suppressing the gate. Ride the cascade to zero (the fail-count is the
progress meter).

## STOP triggers (halt + report; do NOT improvise)

1. **STOP if the gate fires on a site that is NOT a hygiene bug** (a legitimate same-name-different-scope binder
   that SHOULD resolve) — report it; the design says same-name-different-scope is always a faulty rebuild, so a
   counterexample means the detection rule needs refining, not a workaround.
2. **STOP if the cascade is large** (many stdlib sites) — report the full site list before mass-editing; we weigh
   the blast radius. Do not blanket-rewrite.
3. **STOP if surfacing the error at `check.rs:3416` requires changing the function's signature or error-return
   convention beyond binding the span** — report the shape; do not refactor the checker's error plumbing.
4. **STOP if `probe_arc260_1b_call_sugar` or `probe_macros_unbounded_depth` or `probe_kwargs_slash_name` go red** —
   those are GREEN now and must stay green; a regression there means the fix is wrong.

## Gate (the orchestrator re-runs every line against the disk)

| what | command | expected |
|---|---|---|
| the gate fires precisely (not cryptic UnboundSymbol) | `cargo test --release -p wat --test probe_kwargs_emitted_by_macro` | GREEN — the divergence is fixed at its source, so it never fires; the kwargs fn works end-to-end |
| scope helper unit tests | `cargo test --release -p wat scope_divergent` | green |
| 260.1b not regressed | `cargo test --release -p wat --test probe_arc260_1b_call_sugar --test probe_kwargs_slash_name --test probe_macros_unbounded_depth` | all green |
| no new workspace regressions | `cargo test -p wat --no-fail-fast` | SET-diff vs HEAD = ∅ (the ~202 execve floor; weigh by failing-test SET, never absolute count) |
| logic homed, megafile thin | `git diff --stat` | new logic in `src/scope/resolution.rs` + `src/check/error.rs`; `check.rs` gains a CALL, not a block |

Runtime prediction: 60–120 min (the gate is small; the sweep's size is unknown until the gate lights it up —
that's the point). Trap-door: the sweep cascade (ride it to zero; fix at source; never suppress the law).
