# BRIEF — slice one of the rete `where` vocabulary

**Spec:** `DESIGN-STONE-slice-one-rete-vocabulary.md`. Read it first; it carries the rulings, the
affirmative cuts, and the gates. This brief is the strike path.

## You are a rider, not the orchestrator

**Ending your turn ENDS you.** It does not suspend you, and nothing will wake you — there is no
notification coming. Run every command in the FOREGROUND and block on it: your turn ends when the
numbers are in your hands, not when a command is launched.

## The work, in one paragraph

Mint four rete-namespaced ops and the fence's admission test, and — the actual deliverable — do it
through **ONE `const` table that three sites iterate**, so op #5 is a row rather than three edits.
The four are chosen as one-of-each-mechanism-class so the table's shape is pinned by its hardest
member: `:wat::rete::i64::>` (plain alias), `:wat::rete::i64::+` (fallback-carrying, takes
`:undefined`), `:wat::rete::core::and` (lazy special form), plus the module-set admission test in
`head_ok`. The rete fence's whitelist is and stays a **hand-managed list** — builder-ruled; that is
the design, not a stopgap.

## Read these rooms, in order, and why you are being sent

1. **`src/check.rs:15420-15441`** — the i64 comparison family registered from an array in a `for`
   loop. **This is THE PATTERN the table generalises**; copy its shape, do not invent one.
2. **`src/check.rs:15862-15872`** — `:wat::core::not`'s `TypeScheme`. What a plain-alias
   registration looks like when it is not part of a family loop.
3. **`src/check.rs:4230`** — the `and`/`or` inference arm. A special form gets an *arm*, never a
   `TypeScheme`. Your `:wat::rete::core::and` mirrors this.
4. **`src/runtime.rs:5062`** — `":wat::core::i64::>" => eval_compare(…, |o| o == Ordering::Greater)`.
   The alias's dispatch: same routine, different name.
5. **`src/runtime.rs:5110-5111`** — `and`/`or` → `eval_and`/`eval_or`. Lazy; this is why `and` is a
   form and not an alias.
6. **`src/runtime.rs:4829`** — the inline `eval_i64_arith` arm. **PROVEN by run to be the path a
   `where` traverses** (`wat-scripts/scratch-pad/probe-stop-a-where-arith-path.wat`). This is where
   `:wat::rete::i64::+` must reach.
7. **`src/runtime.rs:9753` + `:9857`** — `dispatch_substrate_impl`'s `arith_i64_i64_inner` /
   `I64ArithErr`: the clean typed-domain factoring, on the **`apply`-reachable** path, NOT the one a
   `where` uses. The shared kernel to move `:4829` onto.
8. **`src/rete/purity.rs:159-270`** — `intrinsic_meta`, the whitelist. Note the `string::` arm sets
   `total` per-verb with a stated reason each; that is the standard of evidence.
9. **`src/rete/purity.rs:520-560`** — `head_ok`'s three doors and `constructor_meta`. The admission
   test is a fourth consideration on the `else` branch, **not** a replacement for the others.
10. **`src/rete/purity.rs:725`** — `classify_fn`. The composition property. You do not build it; you
    must not break it, and gate 3 proves it still holds.

## Implementation sketch — fill this in, do not invent the shape

```rust
// src/rete/vocabulary.rs  (new)

/// One rete-surface op. The SINGLE place any rete op is named.
pub(crate) struct ReteOp {
    /// The rete-surface FQDN, e.g. ":wat::rete::i64::>".
    pub rete_name: &'static str,
    /// The core routine this surfaces, e.g. ":wat::core::i64::>". For a FORM this is the
    /// core form whose arm is mirrored.
    pub core_name: &'static str,
    pub class: OpClass,
    /// The whitelist row — what the fence answers for this head.
    pub meta: OpMeta,          // { pure, deterministic, total } — reuse purity.rs's type
}

pub(crate) enum OpClass {
    /// Plain strict fn: a TypeScheme + a dispatch arm to `core_name`'s routine.
    Alias,
    /// Lazy: a checker inference arm + an eval arm mirroring `core_name`'s. No TypeScheme.
    Form,
    /// Alias PLUS a terminal handler substituting the caller's `:undefined` value.
    Fallback,
}

pub(crate) const RETE_OPS: &[ReteOp] = &[ /* the four */ ];

/// The admission test. NOT a bare prefix — see STOP-1.
pub(crate) const RETE_MODULES: &[&str] =
    &[":wat::rete::core::", ":wat::rete::i64::", ":wat::rete::f64::",
      ":wat::rete::string::", ":wat::rete::holon::"];
```

Then: `check.rs` iterates `RETE_OPS` filtering `Alias | Fallback` to register schemes; `runtime.rs`
iterates for dispatch; `purity.rs`'s `intrinsic_meta` consults it for rete-namespaced heads before
its own `matches!`.

## Blast radius

`src/rete/vocabulary.rs` (new) · `src/rete/purity.rs` · `src/rete/mod.rs` (one `mod` line) ·
`src/check.rs` · `src/runtime.rs` · `tests/rete/`. **No `wat/` files. No `crates/`. No new types
outside `vocabulary.rs`.**

## STOP triggers — rejection criteria. Ship nothing, report the gap.

1. **STOP-1 — the bare prefix.** `starts_with(":wat::rete::")` admits `fire-rules` inside a `where`;
   `:wat::rete::` is already the engine's own API (`fire-rules`, `insert`, `compile`, `Session`,
   `AlphaNode`, `activate-fact`…). The test is the **module set**. If you find yourself writing the
   bare prefix, halt.
2. **STOP-2 — a second table.** If the work wants a rete op named in two places, halt and report.
   One list, three readers, is the stone's contract; violating it is the stone failing, not a
   detail.
3. **STOP-3 — the `:4829` refactor is larger than a contained move.** Measure it and report the
   number. Do NOT half-move it, and do NOT duplicate the arithmetic to get green. Ops #1, #3 and #4
   do not depend on it — land those and report #2 as blocked.
4. **STOP-4 — the form classes.** `and` is a checker-arm + eval-arm mirror. `cond`/`match`/`fn` are
   structural guards matched in `classify_expr` and are OUT of this slice. If your `and` work starts
   touching `classify_expr`'s structural arms, halt.
5. **STOP-5 — the `_` wildcard.** `check.rs:5700`'s exhaustiveness error offers
   `"(or include \`_\` wildcard)"`. The `_`-arm-on-an-enum ban is doctrine whose checker rule is
   unbuilt, so nothing mechanical will stop you. Taking it is a rejected strike. Name every variant.
6. **STOP-6 — the corpus moves.** The fence is NOT armed in this slice, so the accepted-`where` set
   must not shift by one row. If `check-where-shapes.sh` reports any change, halt: something leaked
   into `compile-condition`.

## Gates — run each, in the foreground, and report every result line

```
cargo build --release --all-targets          # exit 0, ZERO warnings
cargo clippy --release --all-targets         # likewise
cargo test --release --test rete             # the fence's consumers
cargo test --release --test lint             # repo lints — a build-only gate is blind to these
./wat-scripts/perf/grid/check-where-shapes.sh   # 9 pairs, 99 forms, UNMOVED (STOP-6's proof). ~35s
```

**Do NOT run `cargo nextest run`.** The orchestrator weighs the whole floor centrally, once, after
your tree is quiescent. A narrow filtered `cargo test --release --test <target> -- <filter>` is
fine and expected.

### The two gates that decide whether this shipped

- **The admission test must be shown to ADMIT, not only to refuse.** Three cases: a rete-module head
  admitted · bare `:wat::rete::fire-rules` refused · a `:wat::core::` head refused. A test with only
  refusals proves nothing about admission and is the vacuous-gate class this arc has hit three
  times.
- **Composition, proven by a RUN.** A user `defn` composed of the four ops must classify admissible
  transitively. This is the property the whole design rests on ("users must be able to compose any
  amount of complexity"). Assert it; do not assume `classify_fn` still recurses.

## Prior comparable result to copy for shape

`BRIEF-total-column-honest.md` + its strike (#52, commit `77bbfb67`) — same file
(`src/rete/purity.rs`), same discipline of one-reason-per-classified-verb, same
scope-is-classification-only posture.

## Do not

Do not commit. Do not push. Do not stash. Do not revert anything you did not write. Do not add
`#[allow(dead_code)]` to silence a signal — if a field has no reader yet, say so in the report.
