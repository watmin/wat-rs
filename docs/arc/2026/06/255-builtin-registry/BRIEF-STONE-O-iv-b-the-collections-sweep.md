# STONE O-iv-b — the collections sweep

> Row O-iv-b of `WORKLIST-open-stones.md`. The machine this uses was built and proven by **O-iii**
> (`BRIEF-STONE-O-iii-the-macro-generates-both-doors.md`) on `src/intrinsic/vector.rs` — read that
> brief and that file's current state first. **This wave introduces nothing new.**

## The work

Migrate five sibling collection namespaces to ALGEBRA, exactly as O-iii migrated `vector`:

```
src/intrinsic/map.rs           8 verbs   ← 8 gain a value door for the FIRST time
src/intrinsic/hashmap.rs       8 verbs   ← 8 collapse from TWO hand-written fns into one
src/intrinsic/vec.rs           7 verbs   ← 7 collapse
src/intrinsic/linkedlist.rs    5 verbs   ← 5 collapse
src/intrinsic/hashset.rs       4 verbs   ← 4 collapse
                              32 total     8 new doors · 24 collapses
```

★ **This wave is mostly a COLLAPSE, not a new-door sweep.** 24 of the 32 already carry a
hand-written `value = <path>` twin from Stone N — two fns, one algebra, plus a cross-reference
comment. After this stone each is ONE declaration and the macro generates both doors. That is the
builder's *"two calling conventions are forced by the language; two registrations are not"*, cashed
24 times.

⚠ **And 24 `expect("arity-checked")` sites go with them** — measured: `hashmap` 8, `vec` 7,
`linkedlist` 5, `hashset` 4, `map` 0. Those are the exact panic sites Stone O-i had to guard
centrally; a generated value door arity-checks itself, so they stop existing rather than stopping
being reachable.

## The disposition question is CLOSED for this wave — measured, not assumed

The design's **THE THIRD CATEGORY** section requires each sweep wave to classify before it migrates,
because a *span-carrying* verb (one that raises errors at its own arguments' spans, like
`:wat::f64::max-of`) would trade per-element span fidelity for `apply` reachability — a cost that is
the builder's call, not a rider's.

**None of these 32 is span-carrying.** Measured two ways:
- `grep -n span src/intrinsic/{map,hashmap,vec,linkedlist,hashset}.rs` returns **only**
  `use crate::span::Span;` imports and `_span: &Span, // rune:lint(unused-span)` params. No error
  path in any of the five files reads an argument's span.
- The shell census classifies all 32 `SHELL` (`wat-scripts/hunt/stone-o-shell-census.awk`), the same
  verdict it gave `vector`'s six before O-iii migrated them cleanly.

Each verb's body is byte-identical in shape to the `vector` verbs already migrated:

```rust
#[wat_intrinsic(":wat::map::length")]
pub(crate) fn eval_persistentmap_length_home(
    m: &WatAST, env: &Environment, sym: &SymbolTable,
    _span: &Span, // rune:lint(unused-span) — the only error (TypeMismatch) locates at `m`'s own eval
) -> Result<Value, EvalBreak> {
    let m = eval_inner(m, env, sym)?.value_owned();
    crate::collection::eval::persistentmap_length_inner(&m)
}
```

becomes

```rust
#[wat_intrinsic(":wat::map::length")]
pub(crate) fn persistentmap_length(m: &Value) -> Result<Value, EvalBreak> {
    crate::collection::eval::persistentmap_length_inner(m)
}
```

**The algebra already exists with the right signature** — every one of these ends in a
`crate::collection::eval::*_inner(&Value…)` call. The migration is mostly deletion.

## Rooms — verified against `59c872786`

```
src/intrinsic/vector.rs             ★ THE WORKED EXAMPLE. Six verbs already in the target shape,
                                      including `concat`, which was a two-fn collapse exactly like
                                      the 24 here. Copy it; do not re-derive it.
src/intrinsic/{map,hashmap,vec,linkedlist,hashset}.rs   the five files
src/collection/eval.rs              the `*_inner(&Value…)` algebra every one of them calls
crates/wat-macros/src/wat_intrinsic.rs   the generator — READ ONLY. It is proven; this wave does
                                      not touch it. If you think it needs a change, that is STOP-1.
```

## What this wave does NOT touch

`crates/wat-macros/` · `src/runtime.rs` · `src/intrinsic/mod.rs` · any other namespace. If the
generator needs an edit to handle a collection verb, the generator is wrong or the verb is not what
this brief says it is — either way, STOP.

## STOP triggers — each REJECTS. Ship nothing and report the gap.

1. **You need to change the macro.** O-iii proved it on six verbs of this exact shape. A needed
   change means this wave's population is not what was measured — STOP and name the verb.
2. **A verb turns out to read an argument's span in its own error path.** It is then SPAN-CARRYING,
   the disposition above is wrong for it, and migrating it silently trades a real diagnostic
   property. STOP and name it; the builder rules that class, not this brief.
3. **Any migrated verb's behaviour changes** — value OR error text — on the direct call. All 32 must
   be byte-identical before and after. If one differs, STOP; do not adjust the expectation.
4. **A verb needs `env`/`sym` for anything but evaluating its own arguments.** It is BINDING, the
   census is wrong, and I want to know before it is migrated.
5. **An `expect("arity-checked")` survives in these five files.** Every one of the 24 must be gone —
   not guarded, gone. If one cannot be removed, STOP and say why.

## Acceptance — run each, report the actual output

```
 0. ★ ALL 32 REACH apply. Write ONE scratch .wat under wat-scripts/scratch-pad/ that calls every
    one of the 32 verbs through `(:wat::core::apply …)` and prints the outcome. Every row must
    succeed. Before the strike, the 8 `:wat::map::` rows report the O-iv-a diagnostic and the
    other 24 already work; after, all 32 work. Paste both runs. (It must `--check` clean —
    wat-scripts/ is loader-gated.)

 1. ★ DIRECT CALLS ARE BYTE-IDENTICAL. For all 32, the direct call before and after — value AND
    error text. Build the pre-image with `git show HEAD:src/intrinsic/<f>.rs`, as the O-iii rider
    did, and `diff` the two transcripts. Include at least one type-mismatch error path per file,
    since that is where a span change would show.

 2. ★ THE 24 TWO-FN COLLAPSES ARE REAL.
      git diff --stat src/intrinsic/{map,hashmap,vec,linkedlist,hashset}.rs
      grep -c 'expect("arity-checked")' src/intrinsic/{map,hashmap,vec,linkedlist,hashset}.rs
    The expect count must be 0 in all five (was: hashmap 8, vec 7, linkedlist 5, hashset 4, map 0).
    Report the `value = ` count in these files before and after (24 → 0).

 3. ★ PROVE ONE BY SABOTAGE, ON THE THING ITSELF. Pick one MAP verb (the 8 that gain a door, so
    the sabotage cannot be masked by a pre-existing twin). Make it return a wrong constant; show
    BOTH doors return it — direct AND apply; restore. Confirm the edit LANDED before reading the
    output.

 4. ★ WRONG ARITY IS AN ERROR ON BOTH DOORS, FROM THE GENERATED CHECK. For one migrated verb show
    the direct and apply wrong-arity calls give the identical ArityMismatch. ⚠ O-i's central guard
    would also produce this, so ALSO show the generated check fires standing alone (neuter the
    central guard for one run, or point at the generated code). A row that passes for someone
    else's reason is not evidence.

 5. THE REGISTRY POPULATION IS UNCHANGED at 380 — this wave moves verbs between kinds, it adds and
    removes none:
      grep -rhoP '^\s*#\[wat_intrinsic\(\s*"\K[^"]+' src/ --include=*.rs | sort -u | wc -l
    ⚠ Use that ANCHORED form. A grep that matches attribute text anywhere in a file counts doc
    comments as registrations — it has been wrong four times in this arc.

 6. cargo build --release --all-targets — clean.

 7. cargo nextest run --release -E 'binary_id(wat::wat_lang)' plus any test naming map, vector,
    hashmap, hashset, list, or apply. Report the Summary lines verbatim.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. Verify with `pwd` first. Any path containing
  `.claude/worktrees/` is harness state — never operate on it.
- Everything FOREGROUND. Ending your turn ENDS you; nothing will wake you. Your turn ends when the
  numbers are in your hands, not when a command is launched. **A previous rider on this chain was
  lost mid-strike and its work had to be re-verified from scratch, because it left an
  implementation and no evidence.**
- You may run `cargo build`, `cargo nextest run --release -E '<filter>'`,
  `./target/release/wat --check <file>` and `./target/release/wat <file>`. The orchestrator runs the
  full floor and clippy centrally — leave those two alone.
- You may not spawn sub-agents.
- Do not commit, push, stash, revert, or create a worktree. Leave the tree dirty.
- New scratch `.wat` goes under `wat-scripts/scratch-pad/` and must `--check` clean.

## Report back with

Row-by-row: the command, its actual output, PASS/FAIL. Then the honest deltas — what surprised you,
what this brief got wrong, what you had to decide that it did not settle. Every rider on this chain
has caught a real defect in an orchestrator brief: a dead-code warning, an unsettled probe shape, a
refuted opening premise, a stone about to ship its own deliverable unreadable. That is the most
useful thing you can hand back.
