# BRIEF — ctx is MANDATORY: the name discriminates, arity is a consequence

> **Design + rulings:** `DESIGN-STONE-mandatory-ctx-and-lifecycle-ops.md`. Read its § "THE SHAPE,
> RULED" and § "PRECONDITION (a)" first. **Do not re-derive them.**
>
> The context records already exist and are proven — `b79b17a3` shipped `InvocationCore`,
> `SelfInvocation`, `LifecycleInvocation` and the splice-first `Invocation`, with a probe that
> RUNS (`wat-scripts/scratch-pad/probe-arc278-invocation-family.wat`, prints `-tick78`).
> **This strike is the macro + the migration, not the types.**

## The work, one paragraph

A `defservice` op arm's calling convention is currently inferred from its PARAMETER COUNT. Replace
that with the op's NAME: a leading `-` means internal. Every public arm becomes `[s ctx req]` and
receives an `Invocation` — **no longer opt-in**. Every internal arm becomes `[s ctx]` and receives a
`SelfInvocation`, which today it silently does not (see STOP-0). Arity stops being an input to any
decision and becomes a consequence of the kind.

## The ONE contract decision, pinned

**The discriminator is the leading `-` on the op name. Arity is never consulted.** After this
strike there are exactly two arm shapes and each is fully determined by the name:

```
(op-name  [s ctx req])   public   → ctx : :wat::service::Invocation
(-op-name [s ctx])       internal → ctx : :wat::service::SelfInvocation
```

Any other arity is a **compile error naming the op**, not a fallback.

## ⛔ STOP-0 — THE PRECONDITION. Read this before anything else.

`(-tick [s ctx])` **type-checks green today and silently drops the second binder.** A body that
REFERENCES that binder also type-checks (`--check` exit 0, verified directly, not through a pipe).

Mechanism, at `wat/service.wat:1149-1153`:

```clojure
binding-items (:wat::core::conj … s-binder …)          ;; TWO items: [s-binder, state]
let-bindings  (:wat::core::with-children param-vec binding-items)
```

`with-children` takes `param-vec`'s **shape** and `binding-items`' **contents** — so the emitted
`let` binds `s`→`state` and nothing else, whatever the param vector holds. **Fixing this is the
first move.** Until it is fixed, any `.wat` written to this design compiles and quietly does
nothing, and no test will notice.

## The worklist — MEASURED, by a form-aware census, not a grep

`wat-scripts/census-defservice-arm-arity.wat` (committed with this brief) walks the parse tree: for
each top-level `defservice` it finds the `:impls` child, takes its children as arms, and reads each
arm's param-vector length. Run it:

```
grep -rl 'defservice' --include=*.wat . | grep -v '^./target' | sed 's/^/"/;s/$/"/' | tr '\n' ' ' \
  | xargs -0 printf '[%s]\n' | ./target/release/wat wat-scripts/census-defservice-arm-arity.wat
```

| shape | count | disposition |
|---|---|---|
| `[s req]` public | **166** | → `[s ctx req]`, the codemod's worklist |
| `[s]` internal | **2** | → `[s ctx]` |
| `[s ctx req]` public | **1** | already correct (`probe_arc278_call_context.wat`) |
| | **169 arms / 98 files** | |

⚠ **Do not re-derive this with grep.** It was attempted and produced 52, then 179, then 44, and an
earlier brief asserted 120/65 — all wrong. An arm is a STRUCTURE: the first arm in every `:impls`
begins `[(name …` not ` (name …`, binder names vary, and `(make [self x])` — an extend-type method —
is indistinguishable from an op arm on a line. The census has nothing to positive-control because it
matches no pattern.

## Read in order

1. **`wat/service.wat:1149-1153`** — `binding-items` / `let-bindings`. **The STOP-0 defect.** The
   internal branch must bind a ctx here.
2. **`wat/service.wat:1193`** — `(:wat::core::if is-internal …)`, the branch point. `is-internal`
   is computed at `:1107` from the name (`starts-with? op-str "-"`); `param-vec` at `:1108`.
   The name axis already exists — you are DELETING the arity axis, not adding one.
3. **`wat/service.wat:1225-1240`** — the public branch's arity dispatch: `has-ctx?`
   (`= arity 3`, `:1233`), `req-binder` (`:1234`), `ctx-binder` (`:1240`). **`has-ctx?` and every
   conditional hanging off it are deleted** — ctx is unconditional.
4. **`wat/service.wat:1208`** — `ctx-ctor-expr`, the `Invocation` constructor. It already splices
   `~fqdn-kw` / `~op-str` as literals and reads `selectables`/`idx` at runtime. The internal branch
   needs its `SelfInvocation` sibling, which has **no** `conn-id` (a timer has no connection — that
   absence is structural, and it is the whole reason the three-type split exists).
5. **`tests/services/probe_arc278_call_context.wat:46`** — the one already-correct 3-param arm, the
   shape all 166 are migrating to.
6. **`tests/services/probe_arc278_self_scheduling.wat:54`** — the live `-tick [s]`, one of the two
   internal arms.

## Implementation sketch

```
STEP 1  Fix STOP-0: the internal branch binds ctx.
        binding-items becomes [s-binder, state, ctx-binder, <SelfInvocation ctor>]
        exactly as the public branch's arm-let-bindings already does for its 3-param case.

STEP 2  Delete has-ctx?. Public arms bind ctx unconditionally; arity 2 becomes an ERROR.

STEP 3  Make the arity check a located refusal, keyed on is-internal:
          internal  → require 2  ([s ctx])
          public    → require 3  ([s ctx req])
        The error must NAME the op and say the expected shape.

STEP 4  Codemod the 166 public arms [s req] → [s ctx req] and the 2 internal [s] → [s ctx].
        A wat-fix codemod in wat-scripts/fixes/, dry-run on /tmp copies and diffed BEFORE the
        corpus is touched. This is a STRUCTURAL edit (insert a binder into a param vector), so
        the census program's tree-walk is the model, not a text substitution.
```

## Blast radius

`wat/service.wat` + the 98 files the census names + the acceptance gate. **No `src/` change is
expected** — this is macro emission plus a corpus sweep. If you find yourself editing Rust, STOP and
report why.

## ⛔ STOP triggers

1. **STOP-0 (above) is a precondition, not a step you may skip.** If the internal branch still drops
   its binder when you finish, the strike has produced a silent lie at every internal op.
2. **STOP-1 — do NOT hand-edit the 166 arms.** `.wat` corpus migrations go through a wat-fix codemod,
   dry-run and diffed first. If the codemod cannot express the edit, STOP and report — do not fall
   back to sed or hand edits.
3. **STOP-2 — arity must never again be a discriminator.** If your implementation branches on
   param-count to decide what an arm MEANS, you have rebuilt the defect. Arity is checked only to
   REFUSE a wrong shape.
4. **STOP-3 — an internal op gets `SelfInvocation`, never `Invocation`.** Do not give it a `conn-id`
   with a sentinel value. The `-1` sentinel currently in `selectables` for timers is a vector-uniformity
   artifact and is explicitly NEVER read; do not promote it into a context.
5. **STOP-4 — do NOT build `-on-connect` / `-on-disconnect` in this strike.** Lifecycle ops need
   `ServiceEvent`→op dispatch wiring that does not exist. `LifecycleInvocation` is declared and
   unused on purpose; it is the NEXT stone.
6. **STOP-5 — do NOT add `caller-invocation-id`.** It is mandatory-by-design and the client mechanism
   that populates it does not exist; a mandatory field with nothing to fill it is a lie from birth.
7. **STOP-6 — if the floor moves for any reason other than your new tests, STOP** and report the
   failing test's whole block verbatim plus the exact assertion. No re-run first.

## The acceptance gate

Extend `tests/services/probe_arc278_call_context.{rs,wat}` — do not start a new one.

1. **A public arm receives a populated `Invocation`** — `conn-id`, `namespace` == the service fqdn,
   `operation` == the op's own name. (Exists; keep it green.)
2. **★ An internal arm receives a populated `SelfInvocation`** — assert `operation` == `"-tick"` and
   `namespace` == the service's fqdn, read THROUGH the ctx binder. **This is the test that would
   have caught STOP-0**, and nothing in the suite can catch it today.
3. **A 2-param public arm is now a located compile ERROR** naming the op — a `.wat.bad` fixture, red
   IS pass. (A hole-demonstration cannot live where everything must load.)
4. **The stability gate still holds** — three clients, evict the middle, a survivor keeps its
   original `conn-id`.

## Weigh

`cargo build --release` → `cargo nextest run --release -E 'test(call_context)'` →
`./scripts/floor.sh` (read the **Summary line**, never a piped exit code) →
`cargo clippy --release --all-targets`. Expect the floor at **4384 + your new tests**.

⚠ `--check` on `wat/service.wat` is NOT a valid arbiter — it re-registers an already-loaded macro
(`DuplicateMacro`), and `service.wat` is baked into the binary at build time. Rebuild, then the floor.
