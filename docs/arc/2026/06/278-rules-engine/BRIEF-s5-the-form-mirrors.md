# BRIEF — S5 (#56): the four form mirrors

**Spec:** `DESIGN-STONE-where-admits-only-rete-ops.md` § "Forms ARE ops here — and they split into
two classes", and `DESIGN-STONE-slice-one-rete-vocabulary.md` for the table's contract. Read the
first; this brief is the strike path.

## You are a rider, not the orchestrator

**Ending your turn ENDS you.** Nothing wakes you; there is no notification coming. Run every
command in the FOREGROUND and block on it — your turn ends when the numbers are in your hands, not
when a command is launched.

## The work, in one paragraph

Mirror four core forms into the rete vocabulary — `if`, `let`, `match`, `fn` — in **two phases
that are different edits and must not be conflated** (the stone's STOP-4). Phase 1 is the
head-table pair (`if`, `let`); Phase 2 is the structural-guard pair (`match`, `fn`). Phase 1
carries two mechanism changes the table needs before either pair can be a row; Phase 2 rests on
them. **STOP between the phases and report Phase 1's numbers before starting Phase 2.**

## What the orchestrator already ground — take these as given, do not re-derive

Measured this session, by runs, not reading:

1. **`dispatch_keyword_head_value` (`src/runtime.rs:4467`) has arms for all four** — `fn` `:4643`,
   `let` `:4649`, `if` `:4651`, `match` `:4796` — and the rete gate sits at `:4486`, ahead of all
   of them. So `dispatch_rete_op`'s generic `core_name` re-dispatch reaches every one of the four
   on the non-tail path with **no runtime work at all**.
2. **`eval_tail` (`src/runtime.rs:3807`) intercepts exactly `if` / `match` / `let` / `do`
   (+ `serve-dispatch-op`) and has NO rete gate.** Three of our four forms are in that list.
3. **That gap is load-bearing, proven by a run** —
   `wat-scripts/scratch-pad/probe-s5-tail-position-is-load-bearing.wat`:
   `countdown-if 200000` → `0` (TCO holds); the same recursion with a **Form** in the last
   position → **SIGSEGV, exit 139**. A rete `if` minted without gating `eval_tail` is a strictly
   worse `if` than its core twin: same semantics, silently no TCO.
4. **The checker's Form dispatch (`src/check.rs:2338-2345`) routes `class == Form`
   UNCONDITIONALLY to `infer_boolean_shortcircuit`.** Correct for `and`/`or` (which share ONE core
   arm at `:4246`); wrong for `if`/`let`, whose inference is `infer_if` (`:2619`) and `infer_let`
   (`:2627`).
5. **`classify_expr`'s structural arms match LITERAL core keywords** — `cond` `purity.rs:718`,
   `match` `:738`, `fn` `:768` — so a rete-named `match`/`fn` would miss its structural arm and
   fall into the generic call-shape walk, which would classify a pattern or a param list as if it
   were an expression. That is Phase 2's whole problem.

## Read these rooms, in order, and why you are being sent

1. **`src/rete/vocabulary.rs`** — THE ONE TABLE and its `Form`-class doc note (which already
   records finding 4). Every op is named here exactly once; STOP-2 is a row appearing anywhere
   else.
2. **`src/check.rs:2330-2346`** — the Form dispatch you are changing in Phase 1.
3. **`src/check.rs:2619` / `:2627`** — `infer_if` / `infer_let`, the arms `if`/`let` must reach.
4. **`src/check.rs:4246`** — the shared `and`/`or` arm, so you can see why those two are the only
   ones `infer_boolean_shortcircuit` is right for.
5. **`src/runtime.rs:4486`** — the existing rete gate. Phase 1's `eval_tail` gate MIRRORS THIS.
   Copy its shape.
6. **`src/runtime.rs:3807-3835`** — `eval_tail`, where that mirror goes.
7. **`src/rete/purity.rs:641-660`** — `head_ok`'s vocabulary door, so you can see how an admitted
   head is resolved before you touch the structural arms.
8. **`src/rete/purity.rs:718 / :738 / :768`** — the three structural guards, Phase 2's target.
9. **`tests/rete/probe_arc278_55_slice_one_vocabulary.{rs,wat}`** — the gate file you extend. Note
   how `or`'s gate is the SHORT-CIRCUIT with a non-vacuity control, not the answer. Your new gates
   follow that shape.

## Implementation sketch — fill this in; do not invent the shape

### Phase 1 — the head-table pair (`if`, `let`)

**1a. Teach the Form dispatch to route by `core_name`** (`check.rs:2338`):

```rust
if let Some(op) = crate::rete::vocabulary::rete_op_for(k.as_str()) {
    if op.class == crate::rete::vocabulary::OpClass::Form {
        // Route to the SAME inference helper the mirrored core form uses — keyed off
        // `core_name`, never a hardcoded rete FQDN (STOP-2).
        let (val, mut errs) = match op.core_name {
            ":wat::core::and" | ":wat::core::or" => infer_boolean_shortcircuit(args, head_span, env, locals, fresh, subst),
            ":wat::core::if"  => infer_if(args, head_span, env, locals, fresh, subst),
            ":wat::core::let" => infer_let(args, head_span, env, locals, fresh, subst),
            other => /* a Form row whose core arm nobody taught this to route — see STOP-1 */,
        }.into_parts();
        …
    }
}
```

**STOP-1 — the `other` arm must be a LOUD, LOCATED error naming the unrouted `core_name`, never a
silent fallthrough to `infer_boolean_shortcircuit`.** A future row added without a route here must
break the build or a test, not be mis-typed as a boolean short-circuit. If the cleanest way to make
that unrepresentable is an exhaustive match on a small enum rather than a string match, say so in
your report — do not build it without reporting.

**1b. Gate `eval_tail`** (`runtime.rs:3807`), mirroring `:4486`: a rete Form in tail position must
reach the same `*_tail` routine its core twin does, so TCO survives. Same table, same `rete_op_for`
lookup, dispatch on `core_name`.

**1c. Two rows** in `RETE_OPS`: `:wat::rete::core::if` and `:wat::rete::core::let`, `class: Form`,
`core_name` the core twin. Their `meta` mirrors the core entries in `purity.rs`'s own lists
(`:256`/`:257` and `:453`/`:454` — both already carry `if` and `let`).

### Phase 2 — the structural-guard pair (`match`, `fn`)

The three arms at `purity.rs:718/:738/:768` must recognise the rete twin as well as the core name.
**Do not duplicate the arm bodies.** The discriminator belongs in one place — resolve the head to
its core name first (the table already gives you `core_name`), then match on that.

Checker: `match` → `:3597`, `fn` → `:4326`, routed the same way as 1a.

**`fn` is the odd member and you must ground it before you touch it:** it evaluates to a closure,
it is not in `eval_tail`, and `check.rs:525`/`:1654`/`:1735` treat `:wat::core::fn` specially in
places that are not the inference arm. Read those three sites and report what a rete-named `fn`
would do at each **before** editing.

## Blast radius

`src/rete/vocabulary.rs` · `src/rete/purity.rs` · `src/check.rs` · `src/runtime.rs` ·
`tests/rete/probe_arc278_55_slice_one_vocabulary.{rs,wat}`. **No `wat/` files. No `crates/`. No new
types outside `vocabulary.rs`.** The fence stays UNARMED — `wat/rete.wat:658` remains
`(and is-pure is-det)` and the corpus must not move.

## STOP triggers — rejection criteria. Ship nothing for that phase, report the gap.

1. **STOP-1 — the unrouted Form** (above). No silent fallthrough.
2. **STOP-2 — a second table.** A rete op named in more than one place is the stone failing.
3. **STOP-3 — `fn` turns out to need more than an inference route.** Report what `check.rs:525`,
   `:1654`, `:1735` do with a rete-named `fn`; if any of them would need to change, STOP and
   report rather than widening them. Phase 1 and `match` can land without `fn`.
4. **STOP-4 — do not conflate the two classes.** Head-table entries and structural guards are
   different edits. If Phase 2's work starts reaching back into Phase 1's dispatch, halt.
5. **STOP-5 — the `_` wildcard on an enum scrutinee is doctrine-illegal.** Name every variant.
6. **STOP-6 — the corpus moves.** `./wat-scripts/perf/grid/check-where-shapes.sh` must report 9
   pairs / 98 rows all agreeing. Any change means something leaked into `compile-condition`.
7. **STOP-7 — the TCO gate cannot be proven.** If you cannot write a test that goes RED without
   1b, say so; do not land 1b on the argument that it is obviously right.

## Gates — run each, FOREGROUND, and report every result line

```
cargo build --release --all-targets            # exit 0, ZERO warnings
cargo clippy --release --all-targets           # likewise
cargo test --release --test rete               # the vocabulary's own gates
cargo test --release --test lint               # repo lints — a build-only gate is blind to these
./wat-scripts/perf/grid/check-where-shapes.sh  # 9 pairs, 98 rows, UNMOVED (STOP-6). ~35s
```

**Do NOT run `cargo nextest run`** — the orchestrator weighs the whole floor centrally, once, after
your tree is quiescent. A narrow filtered `cargo test --release --test <target> -- <filter>` is
fine and expected.

### The gates that decide whether this shipped

- **Each new form's gate must be the thing that distinguishes it, not its answer.** `or`'s gate in
  that file is the short-circuit plus a non-vacuity control proving the same operand reached DOES
  raise — copy that discipline. For `if`: the untaken branch must not be evaluated. For `let`: a
  binding must actually scope. For `match`: an arm's pattern must not be evaluated as an
  expression.
- **The TCO gate (1b) must go RED without the gate.** Prove it by removing your `eval_tail` change,
  watching the test fail, and putting it back. Report both observations.
- **Two lints bite this file specifically, and both bit the orchestrator on the last strike:**
  a doc-comment or assert message that PARSES as a wat list trips `no_inlined_wat_in_tests`
  (`"(not false)"` did); and a `contains(...)` on a rendered error trips `no_loose_string_assert`
  — match the typed `RuntimeErrorKind` instead. Fix at the root; **do not add a `rune:lint` to
  silence either.**

## Prior comparable result to copy for shape

The `not`/`or` rows landed an hour ago at `5ffdfc5c` — same file, same table, same gate file, same
two-lint trap. Read that commit and its diff first; it is the nearest exemplar.

## Do not

Do not commit. Do not push. Do not stash. Do not revert anything you did not write. Do not arm the
fence. Do not touch `wat/rete.wat`. Do not add `#[allow(dead_code)]` to silence a signal — if
something has no reader yet, say so in your report.
