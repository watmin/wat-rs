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

## Three migration patterns — pick by what's actually changing

The substrate-as-teacher discipline applies in three shapes,
distinguished by where the diagnostic naturally surfaces. Picking
the right one keeps the diagnostic stream coherent — every error
*kind* names a real rule, no synthetic dressing.

### Pattern 1 — Type-shape change (arcs 111 / 112 / 113)

**When:** the change is to a TYPE SIGNATURE that ripples through
unification (`:Option<T>` → `:Result<:Option<T>,E>`,
`:E` → `:Vec<:E>`). Old call sites mismatch *naturally* against
the new signature.

**Mechanism:** keep `CheckError::TypeMismatch` (already firing
from unification); add `arc_NNN_migration_hint(callee, expected,
got) -> Option<String>` that detects the shape pair and appends
the brief.

```rust
fn arc_111_migration_hint(_callee: &str, expected: &str, got: &str) -> Option<String> {
    let arc_marker = "wat::kernel::ThreadDiedError";
    let expected_has = expected.contains(arc_marker);
    let got_has = got.contains(arc_marker);
    if expected_has == got_has { return None; }
    Some("arc 111 — send/recv lifted to Result<Option<T>,ThreadDiedError>; ...".into())
}
```

The hint rides through `collect_hints` → existing
`TypeMismatch::diagnostic()` → user-facing diagnostic. Nothing
synthetic. The diagnostic kind ("TypeMismatch") is honest — types
genuinely mismatched.

### Pattern 2 — Verb retirement (arc 114)

**When:** a NAMED verb retires; the new verb has a different shape
or a different name. Old call sites still parse, but the verb's
dispatcher is the natural choke point.

**Mechanism:** push synthetic `CheckError::TypeMismatch` from the
verb's dispatcher (or a new arm in `infer_list` before scheme
lookup) with carefully-chosen `expected` / `got` strings the hint
detects:

```rust
":wat::kernel::join" => {
    errors.push(CheckError::TypeMismatch {
        callee: ":wat::kernel::join".into(),
        param: "(retired verb)".into(),
        expected: ":wat::kernel::Thread/join-result".into(),
        got: ":wat::kernel::join".into(),
    });
    for arg in args { let _ = infer(arg, ...); }
    return Some(fresh.fresh());
}
```

Pair with `arc_NNN_migration_hint` that detects the retired verb
in the `callee` field. The diagnostic kind ("TypeMismatch") is a
small honesty stretch — no actual type mismatched, the verb is
gone — but the dispatcher convention is well-established and the
hint clarifies.

### Pattern 3 — Symbol migration (arc 109's class)

**When:** a SYMBOL (type name, namespace path) retires; the new
symbol has the same shape but a different keyword path. No type-
shape break; no verb dispatcher to hook into. Examples:
`:i64` → `:wat::core::i64` (slice 1c), `:()` →
`:wat::core::unit` (1d), `:wat::std::stream::*` →
`:wat::stream::*` (9d).

**Mechanism:** mint a dedicated `CheckError` variant per
migration class; add a walker that visits the program after type
inference and emits the variant per offending site. The variant's
`Display` IS the migration brief — no `arc_NNN_migration_hint`
helper, no `collect_hints` involvement.

```rust
pub enum CheckError {
    BareLegacyPrimitive {
        primitive: String,    // ":i64"
        fqdn: String,         // ":wat::core::i64"
        span: Span,
    },
    // ...
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckError::BareLegacyPrimitive { primitive, fqdn, span } => {
                write!(f, "{}: bare primitive type '{}' is retired; \
                          canonical form is '{}'. Arc 109 § A.",
                       span, primitive, fqdn)
            }
            // ...
        }
    }
}

fn validate_bare_legacy_primitives(program: &Program, errors: &mut Vec<CheckError>) {
    // walk every TypeExpr in the program; emit one error per bare site
}
```

Same recipe as arcs 110 (`CommCallOutOfPosition`), 115
(`InnerColonInCompoundArg`), 117 (`ScopeDeadlock`). Each variant
names its own rule. The diagnostic kind tells the truth about
what was violated — no synthesis.

**Tone for arc 109's remaining slices.** Every symbol-migration
slice (1c, 1d, 9d, 9e, 9f-9i) gets its own dedicated variant.
Walkers live next to their variants. Sonnet sweeps consume the
diagnostic stream the same way — cargo test → grep
`BareLegacyPrimitive`/etc. → fix per site. Three audiences, one
stream, still clean.

### Picking the pattern — flow chart

```
Does the change naturally produce TypeMismatch from unification?
├─ YES → Pattern 1 (type-shape change). Add migration hint helper.
└─ NO → Does the change retire a verb with a dispatcher hook?
        ├─ YES → Pattern 2 (verb retirement). Synthetic TypeMismatch.
        └─ NO  → Pattern 3 (symbol migration). New CheckError variant + walker.
```

When in doubt, prefer Pattern 3 — the diagnostic is most honest
and the variant name self-documents at the call site.

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
- `arc/2026/04/110-kernel-comm-expect/INSCRIPTION.md` —
  pattern 3 precedent: `CommCallOutOfPosition` variant + walker.
- `arc/2026/04/115-no-inner-colon-in-parametric-args/INSCRIPTION.md`
  — pattern 3: `InnerColonInCompoundArg` variant detected at
  parse-to-TypeExpr time.
- `arc/2026/04/117-scope-deadlock-prevention/INSCRIPTION.md` —
  pattern 3: `ScopeDeadlock` variant + `validate_scope_deadlock`
  walker. Closest precedent for arc 109's symbol-migration class.
- `src/check.rs::collect_hints` — the wiring point for Pattern 1
  / Pattern 2 hints. Pattern 3 doesn't use it — variants display
  themselves.
- `WAT-CHEATSHEET.md` — the snapshot of currently-active rules
  the language enforces. Substrate-as-teacher tightens the
  language; the cheatsheet records what the language is.
