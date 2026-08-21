# DESIGN — arc 109, β-ii-c: a generated type carries only the params it CONSUMES

**Status: DRAWN 2026-08-21.** Written against `7fc7379c1`. Its acceptance test is the parked
consumption wall (`PATCH-param-spec-consumption-wall.patch`).

## The defect, found by the wall rather than argued

```
type parameter "K" in :wat::cache::lru-svc::Record's param-spec is declared but never used
```

`defservice` stamps the SERVICE's full param list onto every type it generates. `lru-svc::Record`
is built from `:durable [capacity <- :wat::core::i64]` — one field, typed `i64` — and carries
`<K,V>` for no reason.

## ★ The house style already does this correctly, in the same file

The hand-written message types in `wat/cache.wat`'s own surface each carry exactly what they use:

```clojure
(defrecord :wat::cache::Cache::GetRequest<K>  [probes <- :wat::core::Vector<K>])   ; <K> only
(defenum   :wat::cache::Cache::GetResult<V>   :Hit [value <- :V] :Miss [])         ; <V> only
(defenum   :wat::cache::Cache::GetResponse<V> …)                                    ; <V> only
```

**The rule is already the convention. `defservice` is the sole violator**, and it violates it only
because a macro found it easier to stamp everything than to ask what each clause uses.

Per-clause the answer is unambiguous:

| generated type | built from | consumes |
|---|---|---|
| `Record` | `:durable [capacity <- i64]` | **nothing** — should be monomorphic |
| `State` | `:ephemeral [cache <- Lru<K,V>]` + the durable ref | `K`, `V` |
| `Op` / `Reply` | the surface's message types | whatever those mention |
| `Handle` / `Admin` / status | their own field sets | likewise |

## ⛔ THE OBSTACLE — a macro body cannot compute consumption

To stamp per-clause, the macro must answer *"does param `K` appear anywhere in this field vector?"*
That is an **arbitrary-depth search over an AST**, and a program-body macro cannot express one:

- **No recursion.** `foldl` walks one level. Depth needs a self-call.
- **No helper `defn`.** The F5 gate is default-deny and refuses a user-defined head AT DEFINITION —
  measured, 3029 tests red (`NOTE-the-F5-allow-list-and-what-a-macro-body-may-call.md`).
- **No `mapv` over a bare primitive.** Also measured, also refused.

And with types spelled as KEYWORDS, the param is *inside* a token — `:wat::cache::Lru<K,V>` is ONE
node whose `ast-name` is that whole string. So even a full structural walk would end at a string and
need a substring test, which is the very thing `:829`'s `contains? fqdn-tp "<T>"` does and this arc
is removing.

★ **This is where the builder's earlier question — *"what needs to become an intrinsic?"* — finally
has a real candidate.** Last time the answer was "nothing, once the binder arrives as a form." Here
the need survives the form flip, because it is a *search*, not an extraction.

## The options — FOUR QUESTIONS, and the shared premise first

⚠ **Shared premise:** all three assume `defservice` must compute consumption ITSELF. It need not —
option **D** below lets the author state it. Checked deliberately, because two of the options are
expensive only under the assumption.

**A — a new intrinsic: `type-params-used-in(syms, ast) -> Vector<WatAST>`.**
Obvious YES (its name is its contract) · Simple YES (pure, total, structural — the F5 allow-list's
own admission criterion) · Honest YES (it computes the answer instead of approximating it) ·
Good UX YES (every future declarator macro needs the same question). **4–0** — but it mints
language surface, and the arc has twice found that an intrinsic it was about to draw was unnecessary.

**B — substring interim: test the rendered clause text for each param name.**
Obvious NO (a word-boundary substring test over rendered source reads as a hack, because it is one) ·
Simple YES · **Honest NO** — it is exactly `:829`'s `contains? "<T>"` reborn, in the stone whose job
is deleting that, and `NOTE-generic-bracket-syntax-edn` already killed one interim on the
"scaffolding we are about to delete" rule. **Fails.**

**C — do nothing; scope the wall to exempt generated declarations.**
Already ruled out at `7fc7379c1` — fails Honest, exempts the code nobody reads.

**D — the author declares each generated type's params in the clause.**
Obvious ? · Simple ? · Honest ? · Good UX ? — **not yet scored, because it needs a shape.** Something
like `:durable :- [] [capacity <- i64]` / `:ephemeral :- [K V] [cache <- Lru<K,V>]`. It removes the
computation entirely and matches "you say what it is", but it adds ceremony to every service and
the answer is derivable, which cuts against making a human restate it.

## What this stone is NOT

- **Not "emit forms".** That was the earlier framing and it was wrong; it is *stop over-stamping*,
  the same deletion β-ii-b made for the 18 generated FUNCTION names, one level up.
- **Not blocked by the user-parametric form gap.** Generated types keep their keyword spelling here.
  `NOTE-a-user-parametric-type-has-no-working-form-spelling.md` is a separate blocker for a later
  stone, not this one.

## Acceptance

The parked wall, re-applied. It found the defect; it is what proves the fix. **No new scorecard row
is invented, and there is no way to call this done while it is not.**
