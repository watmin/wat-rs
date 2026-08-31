# DESIGN-STONE — slice one of the rete `where` vocabulary

> **Status: DRAWN 2026-08-02.** The first slice of #55 (S3b+S4). Parent:
> `DESIGN-STONE-where-admits-only-rete-ops.md` (S0 ruled — law A, namespace-based). This stone does
> not re-open the parent's rulings; it cuts the first buildable piece out of them.

## The objective, in the builder's words

> *"the priority is expressivity in rete where forms… that expressivity must be compilable. that's
> the objective for now."*
>
> *"i'm completely fine having a hand managed whitelist of what funcs and forms are allowed in rete
> exprs… users must be able to compose any amount of complexity from these… we just impose their
> compositions are legal."*
>
> *"255 will make maintaining such a list easier… but we just need some enforcement mechanism
> before 255 arrives."*

Three things follow and they set this stone's whole shape:

1. **A hand-managed whitelist is the accepted mechanism.** Not a compromise pending 255 — the
   ruled design. 255 will make it cheaper to maintain; it does not change what it is.
2. **Composition is the user's, legality is ours.** The whitelist is the alphabet. Users write
   arbitrarily complex predicates as ordinary `defn`s over it and each is admissible *because* it
   bottoms out in the list. **The mechanism already exists and needs nothing built:** `head_ok`
   (`purity.rs:545`) consults `sym.functions` before the intrinsic table and hands off to
   `classify_fn` (`:725`), which walks the fn body on the same axis with cycle detection.
3. **Enforcement before 255.** So: nothing in this stone touches `wat_doc`, `#[wat_intrinsic]`, or
   `IntrinsicEntry`.

## ★ THE ONE CONTRACT DECISION — one list, three readers, no second table

Every rete op today would land at **three** sites: a `TypeScheme` registration (`check.rs`), a
dispatch arm (`runtime.rs`), and a whitelist row (`rete/purity.rs`). At ~50 ops that is ~150 edits,
and *that multiplier* — not the vocabulary — is the thing slice one exists to kill.

**The contract: ONE `const` table of rete-op rows, and all three sites iterate it.** No op is
named in more than one place.

**This is assembly, not invention** — the pattern is already in the tree on the exact first family:

```rust
// src/check.rs:15425 — the i64 comparison family, registered from a list, in a loop
for op_name in &[":wat::core::i64::>", ":wat::core::i64::<",
                 ":wat::core::i64::>=", ":wat::core::i64::<="] {
    env.register(op_name.to_string(), TypeScheme { params: vec![i64_ty(), i64_ty()],
                                                   ret: bool_ty(), .. });
}
```

`check.rs` already loops. `intrinsic_meta` is a per-verb `matches!` and `runtime.rs` is a per-op
arm — those two are what become list-fed.

**And it makes 255 EASIER, not harder.** 255's drain is what `purity.rs:17-20` already prescribes:
*delete the map, point the predicates at `metadata-of`*. That is a **deletion**, indifferent to row
count — and a data table is a cleaner thing to drain than 147 hand-written match arms. What would
make the drain harder is **two sources for one verb**, so slice one enrols nothing in 255's
registry. (Findings paid for while confirming this are filed in
`255/NOTE-purity-is-definition-time-queryable-metadata.md`.)

## The four ops — one of each mechanism class, and why that is the minimum

**The table's shape is pinned by the hardest class in it.** Build the first slice from aliases
alone and the table gets designed for aliases; the fallback class then does not fit and the shape
is redone. So slice one is narrow in breadth and full in depth:

| op | class | what it proves | corpus demand |
|---|---|---|---|
| `:wat::rete::i64::>` | **alias** — plain strict fn, rete name → same routine, zero new logic | the cheap path, and the table's baseline row | **51** of 99 `where` forms use an i64 comparator |
| `:wat::rete::i64::+` | **fallback-carrying** — a second terminal handler taking `:undefined` | the `:undefined` shape, and the only class touching runtime semantics | 16 forms |
| `:wat::rete::core::and` | **special form** — lazy, short-circuiting | that a form can be mirrored at all (checker arm + eval arm, not a scheme) | **23** forms — the most-demanded form by ~4× |
| the module-set admission test | **the fence** | that the whitelist is consulted and discriminates | — |

Demand measured 2026-08-02 by paren-balanced extraction of all 99 `where` forms across the nine
`wat-scripts/perf/grid/where-*.wat` files (comment-stripped, positive control on the longest
multi-line form).

### The class table, corrected twice by grounding — do not inherit the parent stone's version

The parent stone lists `and`/`or`/`not` together as *"class-1 aliases (bool), plain."* **Two of the
three are wrong, and the real seam is `plain strict fn` vs `special form`:**

- **`and` / `or` are SPECIAL FORMS.** They must short-circuit: `runtime.rs:5110-5111` routes them to
  `eval_and` / `eval_or`, and `check.rs:4230` has a dedicated inference arm. They belong with `if`.
- **`not` IS a plain fn** — a `TypeScheme { params: [bool], ret: bool }` (`check.rs:15864`)
  dispatched to `eval_not` (`runtime.rs:5109`). Strict. It belongs with `<`.
- The `purity.rs:236-249` `matches!` lists all of them side by side, which is what makes them look
  alike. That is the *purity* table, not the implementation.

So the three mechanism classes, grounded:

```
ALIAS (plain strict fn)   i64::{> < >= <=} · = · not= · not · string/collection readers
                          → a TypeScheme row + a dispatch arm to the same routine
SPECIAL FORM (lazy)       and · or · if · let · do · when
                          → a checker inference arm + an eval arm; NO TypeScheme
STRUCTURAL GUARD          cond · match · fn
                          → matched in classify_expr (:602/:608/:628); never reaches head_ok.
                            NOT in slice one — a different edit, and the parent stone forbids
                            conflating it with the head-table forms.
FALLBACK-CARRYING         i64::{+ - * / mod rem quot} · f64::{+ - * /} · first · nth · subs
                          → an alias PLUS a second terminal handler substituting :undefined
```

## The `:4829` prerequisite — grounded by a RUN, and it gates only `i64::+`

STOP-A of the parent stone asked which arithmetic path a `where` traverses. **Closed by run**
(`wat-scripts/scratch-pad/probe-stop-a-where-arith-path.wat`): an i64 overflow on a bound variable
inside a `where` reports a span naming **the .wat operand** — `b_span`, which only the inline arm
carries. `arith_i64_i64_inner` uses `rust_caller_span!()` and would have named `src/runtime.rs`.

⇒ **A `where` traverses `runtime.rs:4829` (inline `eval_i64_arith`), NOT `:9753`.** The clean
`I64ArithErr` factoring the parent stone points at lives on the `apply`-reachable path
(`dispatch_substrate_impl`, reached from `:8925`; `purity.rs:1153` names it as such).

So `:wat::rete::i64::+` cannot simply hang a second handler at `:9753`. Either `:4829` moves onto
that shared kernel first, or the rete surface works only through `apply`. **The size of that
refactor is UNMEASURED** — the shape is known, the cost is not. It gates op #2 and nothing else.

## Files touched

| file | change |
|---|---|
| *(new)* `src/rete/vocabulary.rs` | the `const` table of rete-op rows — name, core target, class, type shape, `{pure, deterministic, total}` |
| `src/rete/purity.rs` | `intrinsic_meta` fed from the table for rete-namespaced heads; the module-set admission test in `head_ok` |
| `src/check.rs` | register rete aliases from the table (the `:15425` loop is the pattern); the `and` mirror's inference arm |
| `src/runtime.rs` | dispatch arms fed from the table; the `and` mirror's eval arm; the `:4829` shared-kernel refactor |
| `tests/rete/` | the gates below |

## ⛔ Out of scope — AFFIRMATIVE CUTS, not deferrals

- **The other ~46 vocabulary names.** They slot in as rows once the table exists. By demand:
  remaining i64 comparators → i64 arith → generic compare → `or`/`if`/`not` → collections → string
  (13 forms) → f64 (2 forms) → holon.
- **`cond` / `match` / `fn`** — the structural-guard class. `#56`. `cond`/`do`/`when` have **zero**
  corpus demand and wait for a caller.
- **Arming the third conjunct.** `total?` stays UNARMED at `rete.wat:661`. The admission test is
  **built and unit-tested, not wired into `compile-condition`.** Arming is #57 and only after the
  corpus migration — a refused `first` with nowhere to go locks a user out of arithmetic.
- **Anything in arc 255.** No `@Totality`, no `Totality` enum, no `Category` variant, no registry
  enrolment, no touching `wat_doc` or `#[wat_intrinsic]`. The fence has its own `total` column
  (#52) and it works.
- **The cosine family conversion.** 4 verbs / 56 sites, names ratified, its own strike.
- **Re-measuring the refusal set.** The parent stone's "39/98 refused" predates #52 and is stale
  (direct count of genuinely-partial verbs in `where` forms is **11**). It scopes #57's codemod,
  not this stone.

## STOPs — rejection criteria. Ship nothing, report.

- **⛔ STOP-1 — the bare prefix.** `starts_with(":wat::rete::")` admits `fire-rules` inside a
  `where`. `:wat::rete::` is already the engine's own API (`fire-rules`, `insert`, `compile`,
  `Session`, `AlphaNode`…). The test is the **module set** — `{core, i64, f64, string, holon}`.
- **⛔ STOP-2 — a second table.** If the work wants a rete-op named in two places, halt. One list,
  three readers. This is the stone's contract and its violation is the stone failing.
- **⛔ STOP-3 — the `:4829` refactor turns out to be large.** Report the measurement; do not
  half-move it and do not duplicate the arithmetic. Ops #1, #3 and #4 do not depend on it.
- **⛔ STOP-4 — do not collapse the form classes.** `and` is a checker-arm + eval-arm mirror.
  `cond`/`match`/`fn` are structural guards and are OUT of this slice.
- **⛔ STOP-5 — the `_` wildcard.** `check.rs:5700`'s exhaustiveness error offers
  `"(or include \`_\` wildcard)"`. The `_`-arm-on-an-enum ban is doctrine whose checker rule is
  unbuilt, so nothing will stop a rider taking it. Taking it is a rejected strike.
- **⛔ Never a second implementation.** A rete op is a second *handler* over the shared kernel.

## Gates

1. `cargo build --release --all-targets` → exit 0, **zero warnings**; `cargo clippy --release
   --all-targets` likewise.
2. **The admission test discriminates, with BOTH controls.** A rete-module head is admitted; a bare
   `:wat::rete::fire-rules` is refused; a `:wat::core::` head is refused. A test that only shows
   refusals proves nothing about admission — that is the vacuous-gate class this arc has hit three
   times (R59, `91bbb8cd`'s 11 gates, R62's empty rejection column).
3. **Composition still works, proven by a run** — a user `defn` over the four ops classifies
   admissible transitively. This is the property the whole design rests on; assert it, do not
   assume `classify_fn` still recurses.
4. `./wat-scripts/perf/grid/check-where-shapes.sh` — 9 pairs, 99 forms, **unmoved.** The fence is
   not armed, so the accepted-`where` set must not shift by one row. This is STOP-2's proof in the
   parent stone's sense and it is how we know nothing leaked.
5. `cargo test --release --test lint` — repo lints. A build-only gate is structurally blind to
   them, and that blind spot has cost riders before.
6. The orchestrator weighs `cargo nextest run --release` **centrally, once**, and reads the Summary
   line — never a piped exit.

## What "done" means

Four ops exist and dispatch; a user `defn` composed of them classifies admissible; the admission
test discriminates in both directions and is **not** wired to the fence; the corpus is byte-for-byte
unmoved; and adding op #5 is **one row in one file**.
