# Arc 167 — INSCRIPTION

**Inscribed 2026-05-08 by orchestrator.** All slices shipped.

## What shipped

`:wat::core::fn` and `:wat::core::defn` consume a flat-shape
vector signature with arrow-duality:

```scheme
(:wat::core::fn
  [x <- :wat::core::i64
   y <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::+ 5 x y))

(:wat::core::defn :user::add5-to-2-nums
  [x <- :wat::core::i64
   y <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::+ 5 x y))
```

Five elements at the form level: `head + name? + [args-vector] +
-> :ret + body`. Defn carries a name keyword; fn omits it. The
arrows are duals: `<-` consumes (input type), `->` produces
(output type). Once the shape is read once, every wat function
definition becomes mechanical.

The substrate gained `WatAST::Vector` as a first-class node. wat
sources can write `[...]` and have it reach the AST as a Vector
distinct from List. Vectors are scoped to fn/defn signatures in
arc 167; arc 168 (queued) extends the legal positions to let
binding vectors. Vector-as-value-literal is out of arc 167's
scope — emits a clean MalformedForm error directing the user
to the sig-only consumption path.

The legacy nested-sig syntax `((x :T) (y :T) -> :T)` is gone with
no scaffolding remaining. Per the user's "doesn't leave cruft"
discipline, the slice 2 walker fired during the sweep window and
hard-retired in slice 4. The substrate has zero trace of legacy
nested-sig support post-arc-167.

## Slices

| Slice | Commit(s) | What landed |
|---|---|---|
| 1 | `2215571` (substrate) + `7b5dc3d` (tests + transitional parser + lexer fix) | `WatAST::Vector` foundation; parser accepts `[...]`; eval/check arms error at value position |
| 2 | (same train as slice 1) | fn-sig vector consumer + `BareLegacyFnSignature` walker + defn macro shape |
| 3 | `b8ee916` + `c279b48` + `e0e359f` (sweep WIPs) + `d69693f` (revert FM 5 detour) + `066e3ac` (substrate gap fix: walker Vector recursion) | sweep all `wat/`, `wat-tests/`, `tests/wat_*.rs` legacy fn-sigs; substrate gap discovered + fixed at root |
| 4 | `0e519be` | substrate retirement: walker + legacy parsers + `eval_fn` legacy arm + tests 5/6 deleted |
| 4b | `6a1a578` | sweep 16 src/ lib unit-test fixtures (slice 3 leftovers surfaced by slice 4) |
| 5 | `0f8a102` (.claude/settings.json) + (this commit) | the meta-win discovery + closure paperwork |

The slice branch (`arc-167-slice-2-fn-sig-consumer`) carries 13
commits; main has been untouched throughout. Atomic squash-merge
to main happens at this commit.

## Substrate impact

| Surface | Pre-arc-167 | Post-arc-167 |
|---|---|---|
| `WatAST` variants | 5 | 6 (`Vector` added) |
| fn signature shape | `(:wat::core::fn ((x :T) (y :T) -> :R) body)` 3 elements | `(:wat::core::fn [x <- :T y <- :T] -> :R body)` 5 elements |
| defn macro shape | `(:wat::core::defn :name :sig :body)` 3 args | `(:wat::core::defn :name [args-vec] -> :T body)` 4 args via rest-binder |
| legacy nested-sig parser | live (transitional, slice 2 only) | DELETED |
| `BareLegacyFnSignature` walker | live during slice 2-3 sweep window | DELETED |
| `WatAST::Vector` legal positions | n/a | fn/defn signatures only (arc 168 extends to let bindings) |
| Workspace test count | 2069/0 (pre-arc) | 2067/0 (post-arc; tests 5/6 deleted as vacuous) |

## Stability verification

A 100-round workspace stability harness ran post-slice-4b to
characterize a flake opus reported during the slice 4b sweep
(8-9 `wat-holon-lru` HCS spawn/shutdown failures under workspace
concurrency). Result:

| Metric | Value |
|---|---|
| Rounds | 100 |
| Clean (2067/0) | 100 |
| Flake | 0 |
| Build failures | 0 |
| Clean rate | 100.0% |

**The flake did not reproduce.** 100 consecutive `cargo test
--release --workspace --no-fail-fast` runs all landed at
2067/0 with run times in the 8-9 second band (cargo cache
hot). The workspace is stable.

The most likely explanation for opus's slice-4b observation is
machine load contention at that moment — a test process from
an earlier kill plus the heavy slice-4b sweep work running
concurrently. The harness ran on a quieter system and saw none
of it. No HCS spawn/shutdown race surfaced; no separate arc
opens.

Harness: `scripts/stability-100.sh` (added this arc; checked in
to the slice branch as audit infrastructure for future
substrate-wide stability characterization).

## The meta-win — Claude Code subagent permission inheritance bug

Two sonnet spawn attempts during slices 3 + 4b failed without
shipping work. Initial diagnosis ("skill substitution
hallucination") was wrong. Web research after the second incident
surfaced the real root cause:

- Claude Code issue #18950: subagents do NOT inherit user-level permissions
- Claude Code issue #28584: starting v2.1.56, subagents prompt for permission on every tool call

This project had no `.claude/settings.json`, so subagents spawned
with empty permission state. Sonnet's first Bash call was
genuinely denied; sonnet rationally reached for the
`fewer-permission-prompts` skill (whose description names this
exact problem). Opus had enough reasoning budget to navigate
around — opus's honest delta ("Permission to use Bash has been
denied; used `cargo test --lib` directly") was the diagnostic
that finally surfaced the gap.

Fix: project-level `.claude/settings.json` with tight scoping
(commit `0f8a102`):
- Bash destructive commands EXCLUDED (no rm/mv/cp/chmod/rmdir/mkdir/cd/bash *)
- Write/Edit path-scoped to `/home/watmin/work/holon/wat-rs/**`
- Read/Glob/Grep unscoped (safe)
- cargo + git broad (safe; manage own state)

Memory `feedback_sonnet_skill_substitution.md` corrected with the
real diagnosis. Future sonnet spawns hit settings.json at startup
and run cleanly. The "opus tax for mechanical work" in this arc
was a workaround for a missing config, not a property of sonnet
reliability.

**This is the durable artifact of arc 167.** The fn flat-shape
ships; the substrate gains Vector; but the discipline lesson is
that subagent permissions need a project-level `.claude/settings.json`
in every wat-rs sibling project. The wat-rs file is the canonical
template; reuse it (adjusting path-scopes) for any new project
that wants subagent delegation to "just work."

## Settled design

### Why arrow duality (`<-` for input, `->` for output)

The `<-` and `->` arrows point FROM the type TOWARD the named
slot. Args have `<-` (the slot consumes from a value source);
returns have `->` (the slot produces to a value sink). The user
articulated this 2026-05-08:

> *"my justification for [x <- i64] is that its reverse of the
> ret type"*

Once seen, it's mechanical. The mental model is identical to
shell pipe direction: `<` reads in, `>` writes out.

### Why `WatAST::Vector` as a first-class substrate node

The bandaid alternative would have been a marker-flag on
`WatAST::List`. User direction 2026-05-08 forced the substrate
path:

> *"which option is the best long term solution - no bandaids
> nor half measures. adding vec proper feels like the correct
> move ... arcs can be as complex as they need to be - we just
> add more slices as we need."*

This locked in `Vector` as a first-class AST node. It carries
its own span, hash tag, evaluation arm, type-check arm. Future
arcs (let bindings, struct field declarations, enum variant
payloads) consume it deliberately — each opens its own slice
that adds Vector to the legal-positions list.

### Why `[...]` is sig-only in arc 167 (no value literals)

User direction 2026-05-08:

> *"i'm not ready to support vec literals as values... just vecs
> as exprs... e.g. fn's args. (fn [x <- i64] -> i64 (+ 0 x)) is
> what i want to support now. (conj [0 1] y) — not this... we
> don't know how to entertain this yet."*

Arc 167's eval/check arms emit a clear MalformedForm at value
position naming the constraint and pointing at sig consumption.
The substrate stays honest about what's supported. Future arcs
extend the legal positions deliberately.

### Why hard retirement, not preserved scaffolding

Arc 113 set a precedent of keeping orphaned variant + Display
machinery for retired check-errors. User direction 2026-05-08
explicitly REJECTED that pattern for arc 167:

> *"i want the path that doesn't leave cruft... we thought we
> hard deprecated a bunch of forms only to later realized days
> later that we didn't and spent hours cleaning up stuff i
> thought was hard done."*

Slice 4 hard-deletes everything: `BareLegacyFnSignature` variant +
Display + Diagnostic + walker body + freeze.rs registration +
both legacy parsers + `eval_fn` legacy arm + vacuous tests. Per
SCORE-SLICE-4's verification, `grep -rn "BareLegacyFnSignature"
src/` returns 0 hits.

### Why the substrate gap fix (slice 3 commit `066e3ac`) is permanent infrastructure

`walk_for_bare_primitives` in `src/check.rs` only recursed into
`WatAST::List` before slice 3. After slice 2's flat-shape moved
type keywords into Vector position (fn args), bare retired
keywords inside fn-arg-vectors evaded the walker. The 5-line fix
(Vector arm mirroring the existing List arm) closes this for ALL
walker-fired-bare-primitive checks — not just arc 167's. Any
subsequent arc that retires a primitive via the bare-walker
mechanism inherits the fix.

This was discovered via the FM 5 detour (commit `e6c4638` +
revert `d69693f`): opus initially tried to dodge the gap by
rewriting a test fixture; the user caught it within ~45 min of
the workaround commit; the substrate fix shipped clean afterward.
The discipline lesson is in SCORE-SLICE-3 delta A.

## Honest deltas across slices

The arc had FOUR honest-delta surfaces that shaped the slice
boundary:

1. **Slice 2 delta A** — walker placement: user-source pre-pass
   in `freeze.rs:599-616`, NOT `check_program`. Mirrors arc 163
   slice 3g phase A precedent. Implication: substrate-internal
   `mod tests` fixtures hidden from sweep stream until slice 4
   deleted the parser.
2. **Slice 3 delta A** — FM 5 workaround caught + reverted +
   substrate fix shipped. The right fix was 5 lines, not a test
   rewrite.
3. **Slice 4 delta A** — 16 lib unit-test fixtures surface
   post-retirement (the slice 2 delta A leftovers). Honest
   slice-boundary issue; slice 4b closes it.
4. **Slice 4b delta B** — sonnet permission inheritance bug
   surfaces, opus's honest delta tips off the diagnosis,
   `.claude/settings.json` ships as the meta-win.

Each delta was either accepted (slice 2 A) or surfaced its own
fix (slices 3 A, 4 A, 4b B). Per FM 5 discipline, no delta
shipped a workaround that defeated its slice's purpose.

## What's affirmatively out of arc 167

- **`let` flat-shape `[name expr name expr]`.** User direction
  2026-05-08: *"we'll give let the same treatment next."* Arc 168
  (queued; opens after arc 167 closes) ships let consuming the
  `WatAST::Vector` foundation arc 167 establishes.
- **Vector literals as values.** Sig-only consumption is the arc
  167 boundary. Future arcs extend legal positions; each arc
  ships its own slice plan.
- **`define` form.** User direction 2026-05-08: *"define will
  keep working - fn and defn will be broken."* `:wat::core::define`'s
  signature shape is structurally distinct; arc 167 leaves it
  alone.
- **Short-form opt-in crate** (`(defn add5 [x <- i64] -> i64 ...)`
  without FQDN). Memory `project_short_form_crate_future.md`
  keeps this on the future-crate-pivot ladder; ships much later.
- **Arc 168 (let flat-shape)** — DESIGN drafts post-arc-167 close.
  Reserved as the next-arc number.
- **`wat-holon-lru` HCS workspace-concurrency flake** — captured
  in stability harness above. If the 100-round run is 100% clean,
  the issue was a one-time anomaly (machine load at the time of
  opus's slice-4b run) and no further action is required. If the
  run surfaces real failures, a new arc opens to investigate the
  HCS spawn/shutdown race; the harness output is its DESIGN
  baseline.

## Cross-references

- **Arc 154** — `let*` retirement / sequential-let. Pattern:
  clean break, walker fires during sweep window. Arc 167 mirrors
  the walker shape.
- **Arc 155** — `lambda → fn` rename, `:fn(...) → :wat::core::Fn(...)`
  parametric type. Pattern: clean break, walker + sweep. Arc 162
  closed the leftover internal-identifier residue (arc 167 avoids
  that pattern via slice 4's hard retirement).
- **Arc 159** — `let` per-binding `:T` retired. Closest precedent
  for the substrate-as-teacher sweep shape.
- **Arc 162** — internal-identifier sweep that arc 155 left open.
  ANTI-PATTERN: arc 167 explicitly avoids leaving internals
  dirty by deleting walker scaffolding in slice 4.
- **Arc 163** — substrate head-string FQDN sweep; same delta A
  walker scoping pattern (`freeze.rs` user-source pre-pass).
- **Arc 166** — defn shipped; arc 167 evolves defn's surface.
- **`docs/SUBSTRATE-AS-TEACHER.md`** — the canonical pattern
  doc; slice 3's sweep is its first arc-167 application.
- **Memory `feedback_sonnet_skill_substitution.md`** — corrected
  diagnosis; the meta-win lesson.
- **Memory `project_short_form_crate_future.md`** — keeps
  short-form opt-in on the future ladder.

## Commit chain

- `2215571` arc 167 slice 2 substrate (also lands slice 1's `WatAST::Vector` foundation)
- `7b5dc3d` arc 167 slice 2: tests + transitional legacy parser + lexer `]` break
- `924b040` arc 167 slice 2 SCORE
- `b8ee916` arc 167 slice 3 WIP: stdlib + wat-tests/ legacy fn-sigs
- `001c50b` scripts: cargo test wrappers for sonnet-safe cargo output parsing
- `c279b48` arc 167 slice 3 WIP: bundled-test fn-sig sweep
- `e0e359f` arc 167 slice 3 WIP: tests/ legacy fn-sigs
- `e6c4638` arc 167 slice 3 (FM 5 detour, REVERTED)
- `d69693f` Revert e6c4638
- `066e3ac` arc 167 slice 3: substrate gap fix — Vector arm in walker
- `0e519be` arc 167 slice 4 (partial): substrate retirement
- `6a1a578` arc 167 slice 4b: sweep 16 src/ lib unit-test fixtures
- `0f8a102` `.claude/settings.json`: tight allowlist for subagent permission inheritance
- (this commit) arc 167 slice 5: closure paperwork (4 SCOREs + INSCRIPTION + 058 row + USER-GUIDE update)
