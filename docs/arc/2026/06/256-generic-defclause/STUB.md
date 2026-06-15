# Arc 256 — generic defclause: the clause-ification of the intrinsics (BANKED)

> **Status: BANKED — blocked by 214 (the forever-fix campaign) + enabled by 255
> (builtin registry / FunctionBody::Native). Do not strike mid-214.**
> Born 2026-06-07, mid-4.6a-ii, from the builder's chase: *"how could we move
> those kernel/spawn funcs to clauses — that's the thing i'm chasing."*

## The one-sentence bridge

The bridge to wat-authored kernel verbs is NOT "run wat inside the checker" —
it is **make the checker's one unification engine drivable by wat-authored
signatures**: N hand-written Rust `infer_<op>` fns collapse into ONE
generic-clause inference rule + N wat declarations. Inference stays Rust (the
engine); the KNOWLEDGE moves to wat.

## What already exists (grounded 2026-06-07)

1. **Arc 139 — wat mints ∀ today.** `defn :name<T,U>` (turbofish on the NAME)
   splits the params off and stores them in the Function struct "so the type
   checker can instantiate them for generic functions" (`runtime.rs:1935`,
   `split_name_and_type_params`). Live proof: `wat/stream.wat:133`
   (`map<T,U>`), `wat/kernel/sandbox.wat:77` (`drive-sandbox<I,O>`). Only the
   anonymous fn-form path hard-codes `type_params: Vec::new()`
   (`function/eval.rs:64`).
2. **defclause** — multi-body, type-dispatched, but MONOMORPHIC: the matcher is
   `assignable(arg_i, param_i)` against fixed named types (docs/OP-PLACEMENT.md).
3. **The HM machinery** — `Subst`/`unify`/`fresh` in check.rs (058-030), already
   instantiating the 91 Rust-synthesized ∀ schemes + arc-139 generic defns.
4. **The arithmetic recipe** — wat defclause surface over Rust leaves
   (`wat/core.wat:58`, `:wat::core::+` → `:i64::+`/`:f64::+`).
5. **Arc 255 FunctionBody::Native** — the representation for kernel leaves as
   first-class registered, reflectable entities (255.1a landed).
6. **The reference semantics** — 214's 4.6a-i/ii hand-written intrinsics
   (`infer_spawn_program_prime`, `infer_send_prime`/`recv'`/`try-recv'`/`close'`
   + the four eval arms) define correct behavior, and their probes
   (`tests/nursery/probe_arc214_stone46i_typed_peer.rs`,
   `probe_arc214_stone46aii_peer_verbs.rs`) are the ACCEPTANCE TESTS any
   clause-ified rewrite must pass unchanged.

## The keystone stone

**Generic defclause** = arc-139 turbofish on the defclause name + the clause
matcher upgraded from per-position-assignable to per-clause
**instantiate-then-unify** (first-match-wins kept; type vars bind by
unification and flow to the return):

```
(:wat::core::defclause :wat::kernel::recv'<I,O>
  [p <- :wat::kernel::Thread'<I,O>]  -> O (:wat::kernel::Thread'/recv  p))
  [p <- :wat::kernel::Process'<I,O>] -> O (:wat::kernel::Process'/recv p))
```

Bounded checker work: reuses `Subst`/`unify`/`fresh`; one derived-inference rule
replaces every per-verb infer fn.

## What falls out

- **The peer verbs become wat** (declarations over Native leaves
  `:wat::kernel::Thread'/send|recv|try-recv|close` + `Process'/...` — the
  leaves stay Rust FOREVER: they are the FFI/machine boundary, same reason
  `:i64::+` is Rust; Ruby-on-C lineage).
- **`spawn-program'` dissolves** — it dispatches on a keyword VALUE (`:tier`),
  which clauses cannot; split into `spawn-thread'<I,O>` / `spawn-process'<I,O>`
  — each a single-body arc-139 generic defn over a Native leaf (expressible
  TODAY once the leaves exist). The tier becomes the name.
- **The collection intrinsics sweep** — `get`/`conj`/`assoc`/`contains` become
  generic clauses per container head. Even equality's relational
  `[a <- T, b <- T]` is expressible under unification.
- **docs/OP-PLACEMENT.md forward-corrects**: "clause" grows generic; "intrinsic"
  shrinks to the true residue — genuinely COMPUTED types (variadic arities,
  conditional shapes, value-dispatch).

## Why intrinsics are Rust-only today (the answer to "why can't wat do this")

1. **The phase wall**: inference runs at CHECK time over TypeExprs — parse →
   resolve → check → freeze → eval; at that moment no runtime exists for wat to
   run in. (A compile-time wat tier is the 251/ann-type horizon, not now.)
2. **No type-level wat**: no `Type -> Type` functions; the only ∀ surface is the
   arc-139 turbofish; the anonymous-fn path mints none.
3. **The trust boundary**: the checker is the judge (substrate-as-teacher rests
   on it having only true things to say); the safe userland extension is the
   bounded clause; user-injectable inference rules would poison every
   downstream judgment. Generic defclause keeps the ENGINE Rust and only the
   declarations wat — preserving the boundary.

## Sequencing (the unwind lesson: do not fork mid-campaign)

214 finishes on the Rust intrinsics → 255 lands Native-leaf registration →
THIS ARC: generic defclause → rewrite peer verbs as wat defclauses → DELETE the
four Rust infer fns (probes must stay green unchanged) → sweep the collection
intrinsics → OP-PLACEMENT.md forward-correction. Horizon: arc 251 types-as-forms /
`(ann-type …)` makes the signatures symbolic data; the compile-time wat tier
eventually self-hosts the engine itself.

## Addendum (2026-06-07, builder dialogue): the layered endgame — "full type potential"

Layering matters — most of the potential arrives BEFORE type-forms:

1. **Arc 256 alone (today's surface):** rank-1 ∀ for users — parametric,
   multi-clause, type-dispatched functions over user types. The 4.6i `'`-lexer
   fix is the bridge syntax until type-forms retire keyword-encoding.
2. **+ Arc 251 `(type-ann …)`/`(type-form …)` + the 249 pure macro engine:**
   types become FORMS the macro layer can receive and construct — **wat
   computes at expand time, Rust judges at check time** (the phase wall never
   breaks). Users write type-level PROGRAMS (derive-style: a macro takes a
   record's type-form and GENERATES its equality clauses / serializers /
   lenses); every generated declaration is still verified by the Rust engine.
   The keyword-encoding pothole class (comma-in-keyword, `'`-generics,
   FQDN-in-tuple) dies with annotation forms.
3. **Equality delegates:** `(defclause :=<T> [a <- T b <- T] -> bool
   (:wat::core::structural-eq a b))` — the relational signature becomes
   expressible under matcher unification; the body stays ONE Native leaf
   (today's `values_equal` — the hottest loop belongs in the machine).
   OP-PLACEMENT.md's intrinsic column shrinks to nearly nothing.
4. **Process/thread control delegates as control-plane vs data-plane:**
   native forever = clone3/fds/io_uring/close-sweep/signals (~8 leaf verbs,
   the FFI boundary — Erlang-BIFs lineage). Wat = spawn policy
   (`spawn-thread'<I,O>` as an arc-139 generic defn), select loops,
   supervision, restart strategy, peer pools, backpressure. **The Slice-8
   services rebuild is the first production instance of this delegation.**

Permanent non-delegations (by design, not limitation): the unification/
judgment engine (trust boundary — substrate-as-teacher requires the judge
have only true things to say) and the syscall leaves (FFI boundary). User-
defined inference RULES and dependent types stay out; user-AUTHORED and
user-COMPUTED declarations come in.

## The second half of the keystone (added 2026-06-07 — the one non-obvious gap)

Defclause dispatch is TWO matchers, and the stub above only covered check-side:

- **Check-side** (`infer_list`'s clause path): upgrades to instantiate-then-unify
  — type vars bind and flow. (The keystone section.)
- **Runtime-side** (`dispatch_keyword_value` / `val_type_path`, runtime.rs):
  values are TYPE-ERASED — a peer value carries only its sentinel
  (`:wat::kernel::Thread'`), never I/O. The runtime matcher therefore matches
  **parametric clause params by HEAD ONLY, params ignored** — sound precisely
  because check already proved the params at the call site. (4.6a-i's
  `val_type_path` → `inner.type_path` routing is the precedent: runtime
  dispatch on the sentinel string.)

Also: `parse_defclause_form` (runtime.rs) needs the arc-139 name-turbofish
split (`split_name_and_type_params`, runtime.rs:2542 region) applied to the
DEFCLAUSE name — mirror, don't reinvent.

With this named, the stub is strike-complete: lair-study at strike time covers
the rest (exact matcher line numbers shift; OP-PLACEMENT.md § "Where it's declared"
is the stable pointer).
