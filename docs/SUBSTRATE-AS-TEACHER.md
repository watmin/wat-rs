# Substrate-as-teacher — the migration discipline

The pattern wat-rs uses to ship breaking changes without writing
extreme-detail briefs for every consumer sweep. Originated arc 109
slice 1c; named arc 111 (REALIZATIONS.md); applied across arcs
109 / 111 / 112 / 113 / 114.

The thesis: **the substrate's own diagnostic stream IS the
migration brief.** When a structural arc lands a breaking
change, the type-checker's error messages encode the migration
path. Sonnet (or a human) reading the diagnostic learns the rule,
the canonical form, and the edge cases at every error site —
without consulting the arc doc.

## The four-step recipe

When a structural arc lands that mass-mismatches existing code:

### 1. Add a hint to the relevant error variant

Implement `arc_NNN_migration_hint(callee, expected, got) ->
Option<String>` in `src/check.rs`. Detect the shape pair the arc
introduces and return the self-describing fix path. Keep
detection surgical — fire ONLY when the arc-specific shape is
involved; non-arc type errors must not trigger it.

Example (arc 113):

```rust
fn arc_113_migration_hint(_callee: &str, expected: &str, got: &str) -> Option<String> {
    let bare = "wat::kernel::ThreadDiedError";
    let vec_form = "Vec<wat::kernel::ThreadDiedError>";
    let stripped = expected.replace(vec_form, "");
    let expected_bare = stripped.contains(bare);
    let got_vec = got.contains(vec_form);
    // ... fire only when one side wraps in Vec and the other doesn't ...
}
```

Wire into `collect_hints`:

```rust
fn collect_hints(callee: &str, expected: &str, got: &str) -> Option<String> {
    let hints: Vec<String> = [arc_NNN_migration_hint(callee, expected, got)]
        .into_iter().flatten().collect();
    if hints.is_empty() { None } else { Some(hints.join("\n\n")) }
}
```

### 2. Verify the hint fires on a real error

Hand-craft a broken probe; run the substrate; read the output.
The hint should make the fix obvious without consulting the arc
doc. The integ-test for "is the diagnostic teaching well?" is
**"can a fresh sonnet sweep the fixtures from the hint output
alone?"**

If the answer is no, refine the hint until it does. The hint is
the brief; it has to be self-contained.

### 3. Brief the sonnet

The brief becomes:

> "run `cargo build --release` (or `./target/release/wat <file>`);
> read the hints; apply the migration; iterate until green; report
> what wasn't obvious."

That's the entire delegation contract. No arc-doc to forward, no
patterns inventory, no file list (the substrate emits per-site
errors that name their own files). The agent's success rate
becomes a function of substrate-message quality, not brief
verbosity.

For sweeps that affect substrate-bundled stdlib, sweep stdlib
FIRST so the binary boots clean, then sweep test/example files.
The substrate's stdlib loads at every wat invocation; if it has
unmigrated sites, every probe fires those errors before
producing anything useful.

### 4. Retire the hint when its window closes

Once no consumer wat code emits the arc-specific shape error in
practice (the sweep is structurally complete), the hint stops
firing. Retire the helper:

- Remove the `arc_NNN_migration_hint` function body.
- Remove its entry from `collect_hints`'s array.
- Add a retirement comment to the section header explaining what
  it shipped + when it retired (preserves the scaffold for the
  next migration arc).

The hint did its job. Future arcs reintroduce the pattern with
new helpers.

## The substrate is the progress meter

A second consumer of the same diagnostic stream: the
**orchestrator** monitoring the agent's progress.

```bash
target/release/wat /tmp/probe.wat 2>&1 | grep -c "hint: arc N"
```

The result IS the progress bar. The substrate's bundled stdlib
flows through the type checker on every boot; every remaining
arc-N mismatch produces one hint line; converging to zero means
the substrate sweep is complete.

A non-arc type error doesn't match the hint pattern and doesn't
pollute the count. The signal is clean by construction.

## The substrate has three audiences for one stream

The diagnostic stream the type checker emits is consumed by:

| Audience | What they read | Lifetime |
|---|---|---|
| Humans | Compile error + embedded fix path | Permanent |
| Agents | Same error + arc-N migration hint | Permanent during migration; retired after |
| Orchestrators | `grep -c "hint: arc N"` as progress bar | Implicit; falls out of production output |

Three audiences, one stream. No separate metrics layer, no
progress callbacks, no JSONL sub-protocols. The substrate's
self-describing diagnostic is the lingua franca for the entire
loop.

A fourth audience surfaces during DEBUGGING the substrate
itself: the **substrate-author**. Transient `eprintln!` calls
that print internal `TypeExpr` shape at each inference step turn
the substrate into a tutorial about its own behavior. Added /
used / removed in one debug session; not shipped.

## When the discipline applies

Apply substrate-as-teacher when ALL of these hold:

- The change ripples across many call sites (≥ ~10 sites).
- The migration is mostly mechanical (the rule is uniform; per-
  site judgment is minimal).
- The substrate can detect the old shape vs the new shape via
  type-mismatch on a clear shape pair.

When ANY fail, the discipline doesn't apply:

- One-off changes don't need a hint.
- Heavily judgment-driven refactors (where each site needs
  domain reasoning) get a manual flag (`;; ARC NNN MANUAL`) and
  the agent skips them — substrate-author judgment calls don't
  auto-sweep.
- Non-detectable changes (e.g., behavioral semantics with the
  same type signature) need a different mechanism — usually a
  specifically-named test harness or a runtime assertion.

## Poison patterns

For arcs that retire a verb outright (e.g., arc 114's
`:wat::kernel::spawn`), the substrate poisons the verb at the
type-checker so every call site fails. Two patterns:

### Synthetic TypeMismatch in a special-case dispatcher

When the verb has its own special-case (variadic, polymorphic),
push a synthetic `CheckError::TypeMismatch` at the start of the
dispatcher with carefully-chosen `expected` / `got` strings the
hint detects:

```rust
fn infer_spawn(...) -> Option<TypeExpr> {
    errors.push(CheckError::TypeMismatch {
        callee: ":wat::kernel::spawn".into(),
        param: "(retired verb)".into(),
        expected: ":wat::kernel::spawn-thread".into(),
        got: ":wat::kernel::spawn".into(),
    });
    // ... continue inferring args so additional mismatches surface ...
}
```

### New dispatcher arms in `infer_list`

When the verb goes through normal scheme dispatch, intercept it
in `infer_list` BEFORE scheme lookup:

```rust
":wat::kernel::join" => {
    errors.push(CheckError::TypeMismatch { ... });
    for arg in args { let _ = infer(arg, ...); }
    return Some(fresh.fresh());
}
```

Both shapes leave the `callee` field set to the retired verb
name so `arc_NNN_migration_hint` can detect it.

## Cross-references

- `arc/2026/04/111-result-option-recv/REALIZATIONS.md` — the
  pattern's first naming. The substrate-as-progress-meter
  observation is captured in detail.
- `arc/2026/04/112-inter-process-result-shape/REALIZATIONS.md` —
  the pattern's second application; layered with the substrate-
  author audience observation.
- `arc/2026/04/113-cascading-runtime-errors/INSCRIPTION.md` —
  third application; verified the integ-test ("can a fresh
  sonnet sweep from the hint?") via direct delegation.
- `src/check.rs::collect_hints` — the wiring point; current
  active hints live in its array. Retired helpers leave a
  retirement note in the section header.
- `WAT-CHEATSHEET.md` — the snapshot of currently-active rules
  the language enforces. Substrate-as-teacher tightens the
  language; the cheatsheet records what the language is.
