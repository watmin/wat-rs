# Arc 004 — Lazy Sequences + Pipelines — INSCRIPTION

**Status:** shipped 2026-04-20. Every backlog item answered —
shipped OR rejected-with-audit-record. No items blocked.
**Design:** [`DESIGN.md`](./DESIGN.md) — the planning and
reference notes.
**Backlog:** [`BACKLOG.md`](./BACKLOG.md) — the narrative of the
work including both lessons captured.
**This file:** completion marker.

Same *inscription* pattern: DESIGN.md was the intent; BACKLOG.md
is the narrative; this INSCRIPTION.md is the shipped contract.
If these disagree, INSCRIPTION.md wins.

---

## What shipped

### Prerequisites (arc's own prereqs — not part of the original scope but load-bearing)

**`feat: reduce — one canonical type-normalization pass`**
(`b10f002`): wat-rs had two half-passes (`apply_subst` for Vars,
`expand_alias` for typealiases) that every shape-inspection site
had to chain manually. Half did; half didn't. Added `reduce` as
the single normalization pass; shape-inspection sites converted;
error-display sites keep `apply_subst` (preserves the user's
surface alias name). See
[`BACKLOG.md`'s resolution section](./BACKLOG.md#resolved--the-reduce-pass)
and the lesson below.

**`feat: stdlib wat files use wat-native typealiases`**
(`4473ddc`): Cache's request-shape tuple appeared ~20 times as raw
Rust types. Now `Cache::Request<K,V>`. Console got `Console::Message`.
Stream got `Producer<T>`. Stdlib wat files speak their own
protocol vocabulary.

### Backlog items

**#1 — Typealias expansion at unification** (`7f90760`):
`:MyAlias<K,V>` and its expansion are interchangeable at every
unify site. The `_types` parameter threaded through `CheckEnv`.
Cyclic aliases rejected at `TypeEnv::register` time. Subsumed
later by `reduce`.

**#2 — `:wat::kernel::spawn` accepts lambda values** (`5fbdb87`):
First argument may be a keyword path OR any expression
evaluating to a lambda value. Closes the asymmetry with
`apply_value`. Enables stream combinators to spawn user-provided
lambdas directly.

**#3 — Stream stdlib combinators** (`b5c5962`, `16ddc7b`):

Slice B (first):
- `:wat::std::stream::Stream<T>` typealias for
  `(Receiver<T>, ProgramHandle<()>)`.
- `spawn-producer` — spawns a producer function, returns a Stream.
- `map` — 1:1 transform.
- `for-each` — terminal, drives the pipeline, joins the handle.
- `collect` — terminal, accumulates to Vec.

Slice C (second):
- `filter` — 1:0..1 pure.
- `fold` — terminal aggregator, generalizes collect.
- `chunks` — N:1 batcher with EOS flush, the canonical stateful
  stage.

**#4 — Variadic defmacro** (`c9612be`):
`&` rest-param syntax. `MacroDef` gains `rest_param: Option<String>`.
`expand_macro_call` splits args into fixed + rest; rest is wrapped
in `WatAST::List` and bound to the rest-name. Existing `,@name`
unquote-splicing drops the list elements into the template —
no template-walker changes needed.

**#6 — `:wat::core::conj`** (inside slice C commit): one-line
primitive for immutable Vec append. Needed by `chunks`'s
accumulator. Scheme `∀T. Vec<T> × T -> Vec<T>`.

### Support changes (not in the original backlog but shipped in this arc)

- `feat: :wat::kernel::send returns :Option<()>` (`df3ca03`):
  Symmetric with `recv`. One endpoint-disconnect shape across
  every channel primitive.
- `feat: :wat::std::LocalCache<K,V> typealias as wat source`
  (`edba119`): The stdlib LocalCache gets its own wat-native type
  name. Honest test harnesses (stdlib loaded in `runtime::tests` +
  `check::tests`).
- `docs: drop API-stability / backward-compat narration`
  (`8823b08`): We own every caller; phrases like "stable surface"
  and "backward-compatible" belong to libraries with external
  consumers. Dropped.

### What was REJECTED (with audit record)

**#5 — Pipeline composer**: `let*` already IS the pipeline. The
"boilerplate" a pipeline macro would eliminate was actually
per-stage type annotations — information, not ceremony. Hiding
those behind a macro trades wat's typed-binding discipline
(058-030) for conciseness, and wat has consistently picked
honesty over brevity. See
[`BACKLOG.md`'s pipeline rejection section](./BACKLOG.md#5-pipeline-composer--rejected-doesnt-earn-its-slot)
for the full audit trail and the numbered discipline derived
from it.

## Tests

- `tests/wat_typealias.rs` — 8 cases (alias expansion, cycle
  rejection, shape-site flow).
- `tests/wat_spawn_lambda.rs` — 6 cases (closure survival,
  lambda-arg dispatch).
- `tests/wat_stream.rs` — 11 cases (round-trip, map/filter/fold,
  chunks with EOS flush, three-stage pipeline, chunks→map compose).
- `tests/wat_variadic_defmacro.rs` — 6 cases (splice, zero-rest,
  mixed fixed+rest, malformed-signature refusals).
- Plus: existing suites unchanged, every test still green.

**639 tests passing; clippy clean.**

## Lessons captured (both with numbered procedures)

### Lesson 1: Absence is signal

When a feature expected in a mature language isn't there, ask
*why is this missing?* before patching. The gap often points at
real substrate work — not a one-line edit.

`reduce` was the concrete instance. wat-rs had two half-passes
(`apply_subst` + `expand_alias`); the stream stdlib tripped on
the gap at `infer_positional_accessor`. The cheap move was a
one-site patch + a BACKLOG note listing the other sites.
The honest move was adding the single normalization pass every
mature type system has.

Procedure:
1. When drafting a fix for a surprise gap, hold the question
   *"why is this missing?"* for a beat.
2. If the answer is "we just haven't gotten there, no deeper
   reason" — ship the patch.
3. If the answer is "the substrate hasn't settled here" — the
   substrate work IS the fix, and patching is adding scar tissue.

Captured in: `BACKLOG.md` (resolved section), memory
`feedback_absence_is_signal.md`.

### Lesson 2: Verbose is honest

Before adding a new "ergonomic" form, ask what it ELIMINATES.
If those things carried information, the verbose form is the
honest form.

Pipeline composer was the concrete instance. The design doc
sketched a one-liner that would "eliminate threading
boilerplate." But the boilerplate — per-stage type annotations,
named bindings, explicit upstream references — was information,
not ceremony. Removing it would trade wat's typed-binding
discipline for conciseness. Rejected on those grounds.

Procedure:
1. Write out what the new form expands to.
2. List what the new form ELIMINATES.
3. For each eliminated thing: ceremony or information?
4. If information → rejected (or needs redesign).
5. If ceremony → earns its slot.

Captured in: `BACKLOG.md` (pipeline rejection section), memory
`feedback_verbose_is_honest.md`.

### Two directions of "absence"

The two lessons are opposite shapes of the same observation:
absences mean something.

- **`reduce`**: absence = real gap. Close it.
- **Pipeline**: absence = feature that shouldn't exist. Don't
  close it.

Discipline: ask which direction the gap points BEFORE reaching
for a patch.

## Not shipped (intentionally — stdlib-as-blueprint discipline)

The DESIGN.md's stream stdlib sketched a larger surface than
shipped. Deferred-until-demanded:

- `chunks-by`, `window`, `time-window` — N:1 batchers with
  non-size-based boundaries. Chunks by key, sliding window,
  time-window. None have a concrete caller yet.
- `inspect` — 1:1 side-effect pass-through.
- `flat-map` — 1:N.
- `first` — take first N, drop rest.
- `from-iterator`, `from-fn`, `from-receiver` — alternate
  construction paths. spawn-producer covers the main case.
- Level 2 iterator surfacing (`:rust::std::iter::Iterator<T>`
  via `#[wat_dispatch]`). The cross-thread channel flavor
  covers the main app need; in-process lazy chains haven't been
  demanded by a caller.

Stdlib-as-blueprint discipline: each combinator ships when a
real caller demands it, with a citation. Until then, the
surface stays small.

## Downstream inscriptions owed to the 058 batch

The 058 proposal batch may want retrofits for changes this arc
made:

- **058-030 (types)** — typealias expansion at unification.
  Amend with an inscription section noting the reduce pass
  landed and aliases now flow through every shape-inspection
  site.
- **058-031 (defmacro)** — variadic `&` rest-param support.
  Amend with an inscription section noting the syntax and
  semantics (see this arc's variadic-defmacro commit).
- **FOUNDATION conformance contract** — `:wat::kernel::spawn`
  accepts lambda values. Relaxation to the "Programs are
  userland" section's "function named by keyword path"
  language. Inscription amendment.
- **058-026 (array) or a list-ops slice** — `:wat::core::conj`
  inscription.
- **New 058-034 stream-stdlib** — inscription for the first
  slice of combinators.

These lag this arc; each lands when the 058 batch is next
revisited. The arc is complete on the wat-rs side even before
the 058 catch-up.

---

**Arc 004 — complete.** The trading-lab can compose pipelines
today via `let*` + the shipped combinators. 639 tests passing.
Two cross-session lessons written down with numbered procedures.
Next level: [arc 005 — stdlib naming audit](../005-stdlib-naming-audit/DESIGN.md).
