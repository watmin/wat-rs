# BRIEF — close the scalar-`def` hole: the last door that never knocks

**Anchor:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`; any path containing
`.claude/worktrees/` is harness state — re-cd, and use `git -C <anchor>` for git reads.

**You are a rider, not the orchestrator. Ending your turn ENDS you** — nothing wakes you, no notification
is coming. Run every command in the FOREGROUND and block on it. Your turn ends when the numbers are in
your hands.

## The hole

`72a1ac3d` armed the namespacing wall: an un-namespaced top-level name is a located error at every door
that reaches `resolve::gate`. **A plain scalar `def` is not one of those doors.**

```clojure
(:wat::core::def :pi 3.14)      ;; compiles clean today. It should not.
(:wat::core::def :add (fn …))   ;; correctly rejected — the fn-shaped path IS gated
```

`register_defines` (`runtime.rs:862`) branches on `try_parse_fn_shape_def` — it asks *"is this a `def`
whose value is a `fn`?"*, because its job is pre-registering functions into `sym.functions`. A scalar def
has no function to pre-register, falls through to `rest`, and never touches `gate()`.

**The gate got wired to the SHAPE, not the FORM** — and `def` is the primitive that `defn` is sugar over.
This also corrects the stone's own claim: *"everything that reaches `gate()` is a top-level
registration"* is true; the converse — **not every top-level registration reaches `gate()`** — was never
checked.

## The door, and why it is this one

The scalar def's registration is check-side, not runtime-side:

- **`src/check.rs:7798`** — `":wat::core::def" if is_top =>` calls `env.register_defined_value(name, ty, span)`.
  **This is the registration.** Its own branch condition is already
  `if !env.defined_values.contains_key(&name)` — which *is* `Existing`. A `span` is already in scope.
- **`src/check.rs:7242`** `infer_def` — owns def's *redef* discipline and emits `DefRedefForbidden` at
  `:7365` from a `head_span` into `local_errors`. Read it; do not duplicate its job.

**Route the registration through `gate()`.** Do not bolt a bespoke namespacing `if` onto `infer_def` —
the whole point of the wall is one door. `resolve::is_namespaced` and `Registration::Unnamespaced` already
exist (`72a1ac3d`).

## Implementation sketch — fill it, do not invent a different shape

```rust
// src/check.rs, the ":wat::core::def" if is_top arm
let existing = if env.defined_values.contains_key(&name) {
    crate::resolve::Existing::Equivalent   // def's OWN redef discipline owns divergence
} else {
    crate::resolve::Existing::Absent
};
match crate::resolve::gate(&name, crate::resolve::Privilege::User, existing) {
    Registration::Unnamespaced => { errors.push(CheckError { span, kind: … }); }
    Registration::Reserved     => { errors.push(CheckError { span, kind: … }); }
    Registration::Insert       => { /* the existing register_defined_value path */ }
    Registration::NoOp | Registration::Duplicate => { /* the existing redef path — UNCHANGED */ }
}
```

**`Existing::Equivalent` on presence is deliberate and must not change.** It is exactly what the runtime
door does (`runtime.rs:916`, and its comment says why): mapping presence to `Equivalent → NoOp` keeps
`def`'s redef discipline authoritative, so a divergent redef still surfaces as `DefRedefForbidden` /
`DefRedefTypeChange` from `infer_def` rather than being masked by a `Duplicate` from the gate.

`CheckErrorKind::UnnamespacedName { name }` is a **fourth** taxonomy entry — `TypeErrorKind`,
`RuntimeErrorKind` and `MacroErrorKind` already have theirs from `72a1ac3d`. Mirror their message
verbatim; a diagnostic that names no remedy teaches nothing:

```
top-level name ':pi' is not namespaced — only fn arguments and let-bindings may be bare;
give it a namespace, e.g. ':my::pi'
```

## ★ Ground this before you build — is `Reserved` also unpoliced here?

If a scalar def never reaches `gate()`, then `(:wat::core::def :wat::core::pi 3.14)` from user source may
also be unchecked. **Probe it and report the answer either way.** If it is unpoliced, routing through
`gate()` closes both holes at once and that is a finding worth naming, not a silent bonus.

## RED probe — it must be NEW, and it cannot live under `wat-scripts/`

**The corpus no longer exercises this hole.** The six `wat_arc157_def_*` fixtures that used to hold bare
scalar defs were namespaced by the codemod in `72a1ac3d`, so nothing on the floor is currently RED. That
means a green floor proves nothing here and you must build the specimen.

Add a deliberately-bad fixture — `tests/**/…​.wat.bad` is the right shape (a program that must fail, with
its reason pinned) — holding a bare top-level scalar def. Wire it to whatever `.wat.bad` driver its
directory already uses; copy a sibling. **Do NOT put it in `wat-scripts/`** — every `.wat` there is loaded
and type-checked by a corpus gate, so a deliberately-bad one goes permanently RED.

**Mutation-prove it**: confirm the probe FAILS before your change and PASSES after (or, for a `.wat.bad`,
that its failure reason is `UnnamespacedName` after and something else before). A gate you have not seen
go red is a claim with nothing behind it.

## STOP triggers — rejection criteria. Ship nothing, report.

- **STOP-1 — the gate call disturbs def's redef discipline.** If `DefRedefForbidden` /
  `DefRedefTypeChange` / the `redef_allowed` type-stability path changes behaviour, HALT. Those are
  arc-157 semantics and are not yours to alter.
- **STOP-2 — a legitimate scalar def in the corpus is now rejected.** The corpus was migrated, so this
  should be zero. If the floor surfaces one, HALT and name it — it may be a case the codemod could not
  reach (a `def` inside a `(:wat::core::forms …)` block shipped to a child, for instance).
- **STOP-3 — the check-side arm is not actually the registration.** If reading `:7798` shows the binding
  is really established somewhere else, HALT and report where. Do not gate two places.

## Gate

1. `cargo build --release --all-targets` → exit 0, **zero warnings**.
2. `cargo clippy --release --all-targets` → **zero warnings**.
3. The RED probe, mutation-proven both directions (see above). Paste the verbatim before/after.
4. The `Reserved` question answered with a run, not a reading.
5. A narrow filtered run of the def suite is fine: `cargo test --release --test wat_lang -- arc157_def`.

**Do NOT run `cargo nextest`** — I weigh the full floor centrally on a quiescent tree (it is 4266/4266 at
`72a1ac3d`). Do not commit, push, stash, or revert.

## Report

The diff per file; the build + clippy results; the verbatim RED-probe output before and after; the answer
to the `Reserved` question; whether any corpus def was newly rejected; and any STOP with its evidence.
