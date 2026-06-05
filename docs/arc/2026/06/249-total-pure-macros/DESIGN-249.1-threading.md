# Arc 249 — threading macros `->` (thread-first) + `->>` (thread-last) — DESIGN

**Status:** OPEN 2026-06-04. **Verdict = BUILD both forms.** The STUB framed 249 as an
open WHETHER-verdict leaning decline; grounding this session **reversed that to BUILD** and
**corrected the cost down** (it is a macro-expansion-time desugar, not a substrate special
form). In the pre-232 gate: **245 ✓ → 249 (this) → 235 → rejoin 232.**

## The verdict — BUILD, and the load-bearing why

wat's design telos is **immediate knowability to a model that has seen Clojure** (CLAUDE.md:
"Clojure-faithful … same family as Ruby-on-C / Clojure-on-Java"; LLM-first one-canonical-path).
`->` / `->>` are **bedrock Clojure vocabulary** — a Clojure-trained model arrives *expecting*
thread-last next to fn-first seq-HOFs. Their **absence is the surprise and the friction**, not
their presence.

This is the same conformare / *be-what-you-claim* principle that drove arc 243 (the error-shape
campaign) and **arc 247 this very chain** — 247 flipped the seq-HOFs to fn-first on exactly this
logic: *"we claim Clojure; Clojure's HOFs are fn-first; ours must be. Preference loses to
honesty."* Clojure's fn-first HOFs are ergonomic **because `->>` exists**. Shipping fn-first
(247) without `->>` (249) recognizes the model **halfway** — it takes the dialect's
inside-out-reading cost without its threading payoff. 249 completes 247.

### One-canonical-path is *satisfied*, not violated

The objection "threading is a second way to write a call" measures against the wrong stick. A
Clojure model already holds the convention: **threading for multi-step pipelines, direct
application for single calls.** That is the idiomatic partition, not two competing paths. The
task "express a ≥2-step pipeline" has one canonical form (`->>`); "single call" has another
(direct). Following Clojure's own conventions **is** one-canonical-path here. And threading
**desugars to the one call mechanism** (see below) — it is convenience-over-mechanism, the same
relationship `is-X?` has to `conforms?`; one-canonical-path governs *mechanisms*, and there is
still exactly one.

### The four questions (correct telos)

- **Obvious?** YES — to a Clojure model `(->> xs (map f) (filter p))` is the *recognized* form,
  more obvious than inside-out nesting.
- **Simple?** YES — a bounded expansion-time rewrite; `N` steps → `N`-deep nesting, fully
  predictable (the "an LLM can see the expansion" virtue arc 248 prized).
- **Honest?** YES — we **claim** clojure-on-rust; `->`/`->>` are core Clojure. Omitting them
  makes wat lie about what it is — the exact defect 243 + 247 exist to kill.
- **Good UX?** YES — the Clojure-knowing model reaches for `->>` and it is there.

## The mechanism — a built-in desugar in the macro-expansion pass

Threading is a **left fold producing variable-depth nesting**
(`(->> x s1 s2)` → `(s2 (s1 x))`). The arc-248 `for`-comprehension is deliberately
**bounded-not-Turing** — *"Map, not eval — no recursion, no conditionals"* — so a map+splice
cannot build variable-depth nesting. Therefore threading **cannot be a wat-level `defmacro`**.

It does **not** need to be a runtime special form either: threading has no semantics of its own —
it is a pure **source-to-source rewrite** that disappears into ordinary nested calls *before*
type-check. The home is the macro-expansion pass, `src/macros.rs` `expand_form`, mirroring the
existing **`:wat::core::keyword/of` built-in** (macros.rs:548–569):

> recognized AFTER child recursion (so inner forms / unquotes are already expanded) and BEFORE
> the generic registered-macro dispatch, returning a rewritten `WatAST` that is re-expanded to
> fixpoint.

Threading adds one analogous arm in the `WatAST::List` case of `expand_form`. The head is a bare
`WatAST::Symbol("->")` / `Symbol("->>")` (the `keyword/of` precedent matches a `Keyword` head;
threading matches a `Symbol` head — the only structural difference).

**Consequence: zero changes to `src/check.rs`, `src/runtime.rs`, or `src/special_forms.rs`.**
The checker and runtime never see `->`/`->>`; they see the desugared nested calls.

### The rewrite

`(-> x step1 step2 … stepN)` left-folds the accumulator `acc` (initially `x`) through each step:

| step shape | `->` (thread-**first**) | `->>` (thread-**last**) |
|---|---|---|
| bare `f` (Symbol/Keyword) | `(f acc)` | `(f acc)` |
| list `(f a b)` | `(f acc a b)` — inject acc as **first** arg | `(f a b acc)` — inject acc as **last** arg |

Worked:
- `(-> 5 (- 3))`  → `(- 5 3)`  = **2**   (first-inject)
- `(->> 5 (- 3))` → `(- 3 5)`  = **-2**  (last-inject) — proves first/last differ
- `(->> [1 2 3] (map inc) (filter even?))` → `(filter even? (map inc [1 2 3]))` → `[2 4]`
- `(-> coll (assoc :k v) (get :k))` → `(get (assoc coll :k v) :k)` — the coll-first idiom 247 left intact

Bare single-element `(-> x)` → `x` (identity); empty `(->)` is an arity error surfaced by the
rewrite. Each emitted step is itself already child-expanded; the fold result is re-expanded to
fixpoint by the existing `expand_form` return path (so a macro *inside* a step still fires).

## Lexing + disambiguation — both already free

- **`->>` lexes today** as a single bare symbol: `is_symbol_break` (lexer.rs:428) excludes `>`,
  and `->` already lexes as a bare symbol everywhere (the return-arrow). **No lexer change.**
- **`->` is unambiguous by position**, exactly as in Clojure (`clojure.core/->` the macro vs the
  core.typed `[Params -> Return]` arrow): wat's return arrow `->` appears only as a *middle*
  element of a `defn`/signature form, **never as a list head**. Threading recognizes `->` only
  when it is `items.first()` of a call-List. The infix arrow is never `items.first()`. **No new
  mechanism.**

## FM-2-bis probe (orchestrator-authored, committed before the BRIEF)

`tests/probe_arc249_threading.rs` — must **disconfirm at HEAD** (threading does not exist yet;
`(->> …)` reaches check as an unresolved bare-symbol head and errors), then pass post-build.
Gates:
1. **thread-last evaluates:** `(->> [1 2 3] (map inc) (filter even?))` → `[2 4]`.
2. **thread-first evaluates + injects first:** `(-> 5 (- 3))` → `2`.
3. **first ≠ last:** `(->> 5 (- 3))` → `-2`.
4. **bare step:** `(-> 3 inc)` → `4` (where `inc` is a 1-arg fn) — bare-symbol step → `(inc 3)`.
5. **disambiguation regression:** a normal `(:wat::core::defn :f [a <- :i64] -> :i64 a)` still
   type-checks — the infix `->` return arrow is untouched.

## Scope / constraints

- **Edits (sonnet):** `src/macros.rs` only (the one `expand_form` arm + a `fold` helper). No
  check/runtime/special_forms edits — if the build seems to need them, STOP: threading is
  pure desugar, the need signals a design miss.
- **Both forms in one stone** (`->` and `->>` are the same fold differing by inject-position; the
  STUB's "both or neither" holds — wat now has both coll-first ops and fn-first HOFs).
- **HARD CUT semantics:** threading desugars and vanishes; no runtime `->`/`->>` entity, no
  registry entry, no Display. There is nothing to alias or shim.
- **Green-gate:** `cargo test --release --lib -p wat` + `cargo build --release --tests
  --workspace`, raw commands. No `holon-rs`.
- **Ward note:** `src/macros.rs` is a flat (un-warded) substrate file → no `vigilatum` claim;
  the change rides macros.rs's future ward. (Consistent with arc 243.7's flat-runtime.rs posture.)

## Slicing

**One stone (249.1)** — the desugar + the probe's five gates green. Predicted Mode-A: small
(one expand_form arm + helper + the cascade is *zero* — nothing else references threading).
Then **249.N INSCRIPTION** (folds this DESIGN's verdict; marks 249 resolved; the gate advances
to 235).

## Refs

- STUB (the reversed-from verdict + the false-premise correction it already carried):
  `docs/arc/2026/06/249-threading-macro-verdict/STUB.md`.
- The completed sibling: `docs/arc/2026/06/247-clojure-hof-order/{DESIGN,SCORE}.md` (fn-first;
  banked `->>` as the near-term need).
- The bounded-macro ceiling that forces desugar-not-macro:
  `docs/arc/2026/06/248-macro-comprehension/INSCRIPTION.md` ("Map, not eval … the line held").
- Integration precedent: `src/macros.rs:548–569` (`keyword/of` built-in); `expand_form`
  (macros.rs:507). Lexing: `is_symbol_break` (lexer.rs:428). Arrow sites: `src/check.rs` (the
  ~12 `-> ` signature arms), `src/closure_extract.rs:541`.
- The gate: `docs/arc/2026/05/232-defprotocol-extend-type/RESUME-CONTEXT.md`.
