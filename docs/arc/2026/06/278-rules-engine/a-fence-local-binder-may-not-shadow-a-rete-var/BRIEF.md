# BRIEF — a fence-local binder may not shadow a rete variable

## The work

Refuse a `?`-prefixed name as a **binder** in a fence-local `(:wat::rete::core::let …)` binding
vector or a `(:wat::rete::core::match …)` arm pattern, at rule-compile time, with its own error
variant. **Read `DESIGN.md` beside this file first** — it pins the one contract decision and states
plainly what this does *not* do.

## Read in order

| room | why you are here |
|---|---|
| `.../a-fence-local-binder-may-not-shadow-a-rete-var/DESIGN.md` | the contract decision, the measured corpus counts, what is affirmatively out of scope |
| `src/rete/validate/typing.rs:516-533` | the exact site — the `let` arm walks binding VALUES and returns; the `match` arm walks the SUBJECT and returns. Your refusal goes in both, before those returns |
| `src/rete/validate/typing.rs:~470-515` | `check_fence_interior`'s doc comment — it states the shadowing hazard and why the bound was drawn. **Rewrite it**; it is the claim this stone retires |
| `src/rete/validate/error.rs:244-261` | `ConstraintTypeMismatch` / `FenceConstraintTypeMismatch` — the shape and register to match. Yours carries no `fact_type`, same reason |
| `tests/rete/probe_arc278_fence_interior_types.rs` | ⭐ the shape to copy: the gap test, the control that gives it meaning, the over-rejection guard |
| `wat-scripts/perf/grid/where-inline-computed.wat:166` | ⛔ **the one real `let`-in-a-fence in the whole corpus.** `(let [x ?k] …)` — a plain binder. It must keep compiling; make it a fixture row |

## Sketch

```rust
// typing.rs:516 — before walking the binding values
if head == ":wat::rete::core::let" {
    if let Some(WatAST::Vector(pairs, _)) = items.get(1) {
        for pair in pairs.chunks(2) {
            if let [name, val] = pair {
                // NEW: a `?`-prefixed binder shadows a rete variable — refuse it.
                // Then walk `val` as today (outer scope, still checked).
            }
        }
    }
    return;
}
```

Same for each `match` arm's pattern. `match` patterns bind through `lower_pat`'s bare-`Symbol` arm,
so a `?`-prefixed symbol in pattern position is the same hazard.

## Blast radius

`src/rete/validate/{typing,error}.rs`, plus new fixtures under `tests/rete/`. No change to
`clause.rs`, `mod.rs`, `reachability.rs`, or any `.wat` under `wat-scripts/`.

## STOP triggers

Partitioned — the refusal, the corpus, the scope.

**STOP-1 (the refusal) — a `?`-prefixed binder turns out to be reachable in a legal, useful form
you can construct.** The DESIGN's case rests on it having no defensible reading. If you find one,
STOP and report it with the program; that refutation is worth more than the stone.

**STOP-2 (the corpus) — anything under `wat-scripts/` or `tests/` stops compiling.** The measured
count of `?`-binders is **0**, anchored on a known positive. If the floor reds on a corpus file,
the count was wrong — capture it, report it, do not migrate the corpus to fit the check.

**STOP-3 (scope) — the fix seems to need a scope stack, a local type environment, or a change
outside the fence.** All three are affirmatively out of scope in the DESIGN. Report and stop.

## The open question — answer it either way

`match` arm patterns: I have **not** driven whether a `?`-prefixed symbol in a `match` pattern
inside a fence actually binds, or whether it is refused earlier by some other path. The `let` case
IS driven — `(let [?k "str"] …)` compiles and runs. **Establish the `match` case by driving it
before you add the refusal there.** If `match` patterns cannot express the hazard, say so and
refuse only in `let` — a narrower cure with evidence beats a wider one without.

## Working rules

- `cargo nextest run --release`, never `cargo test`. One cargo build at a time.
- Commit only on green. Stage explicit paths. `git commit -F -` with a quoted heredoc.
- ⛔ Run `./scripts/floor.sh` and **read the result yourself**. If it backgrounds, poll to
  completion inside one tool call, matching on process name (`ps -eo comm`), never a `pgrep -f`
  pattern that appears in your own command line — that matches your own shell and waits forever.
- **Your final message is incomplete unless it quotes the `Summary [...]` line from
  `.floor/latest/clean.log` verbatim with that file's `FAIL` count.**
- On any red: do NOT re-run. Copy the failing test's whole stdout and stderr, name the arm, surface it.
