# DESIGN — STONES 1a-δ and 1a-ε: three shapes of "not really evaluating"

> The next six registrations under the builder's sequencing ruling (*register the population first,
> attack the hand-lists second*). Together they take the mutation pair's population from **7 of 13**
> registered to **12 of 13**.

## ★★★ THE FINDING THAT SPLITS THEM — measured, and it is a third shape

`@Purity Unevaluated` was minted for a form that never reaches evaluation. Registering these six
found that "does not really evaluate" has **three** distinct shapes, and only two of them are
`Unevaluated`:

```
① REFUSED at eval          def, and the declaration family
                           runtime.rs answers with DeclarationInExpressionPosition
                           → Unevaluated. The arm ENFORCES the pole.

② NEVER REACHES eval       load-file!, digest-load!, signed-load!
                           no eval arm at all; only is_mutation_head/is_mutation_form
                           → Unevaluated. Nothing to enforce; nothing to reach.

③ EVALUATES TO A NO-OP     use!, config::set-redef!, config::set-eval-redef!
                           runtime.rs:2120 / :2947 — `=> Ok(Value::Unit)`, and the arm's own
                           comment says why: "the flag has already been processed at freeze
                           time; return Unit as a no-op"
                           → NOT Unevaluated. It evaluates. It just does nothing.
```

★ **Shape ③ is proven by running it, not by reading the arm.** Both forms in expression position
inside a live program return and the program completes:

```
(:wat::core::let [q (:wat::config::set-redef! true)]  7)   →  7
(:wat::core::let [q (:wat::core::use! "wat/list.wat")] 7)  →  7
```

Compare `def` in the identical position, which raises. **Three arms, three different answers, and
only a probe separates them.**

## THE ONE CONTRACT DECISION — pinned

**A no-op that returns is an EVALUATION.** Shape ③ takes `role = eval` and **`@Purity Pure`** — not
`Unevaluated`.

⚠ And `Pure` is not the soft option here: it is the *harder* one. `purity_mandated_examples` demands
a **runnable** `@example` of every `Pure`-and-`Deterministic` row — the exact mandate that made
`Pure` dishonest for `defsurface`, which cannot be run. **These three can**, and each must ship an
example that actually executes. If one of them turns out not to be runnable, that is a finding that
its shape was misread, not a licence to fall back to `Unevaluated`.

★★ The rule that emerges, and it is the one to carry forward: **`Unevaluated` means the axis has no
runtime verdict to give. A verdict of "nothing happens" is still a verdict.**

## The decomposition — two stones, because the axis verdict differs

### 1a-δ — the loaders (shape ②)

```
:wat::load-file!    parse_unverified_load    (src/load/loader.rs:692)
:wat::digest-load!  parse_digest_load_file   (src/load/loader.rs:720)
:wat::signed-load!  parse_signed_load_file   (src/load/loader.rs:763)
```

`role = declare` only. `@Purity Unevaluated`. Routed by `match_load_form` (`loader.rs:669`) — a
router with a dedicated fn per arm, the same shape `parse_type_decl` had, so each row names its own.

⬜ **`@Category` is the open question and must be argued, not assumed.** A load reads a file (`Io`?)
and registers the forms it finds (`Declaration`?). The Category axis is *the DOING* — and these do
both. Whichever is picked, the ground must say why the other was refused.

### 1a-ε — the no-ops (shape ③)

```
:wat::core::use!                dispatch_keyword_head_value:2947 · infer_list:4766
:wat::config::set-redef!        dispatch_keyword_head_value:2120 · infer_config_set_bool
:wat::config::set-eval-redef!   (shares both arms with set-redef!)
```

`role = eval` **and** `role = check` **and** `role = declare` (their freeze-time processors are
`config.rs`'s `collect_entry_file_inner` and `declare/register.rs`'s `register_runtime_defs_form`).
The first rows in this campaign to carry all three, which is what the gate's deliberately
non-exclusive rule was built for.

`@Purity Pure` · runnable `@example` per row.

## THE FOUR QUESTIONS

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **two stones, split on the axis verdict** | YES | YES | YES | YES | ✅ **PICKED** |
| one stone, all six | YES | **NO** | YES | — | ⛔ DISQUALIFIED |
| all six `Unevaluated`, no `role = eval` | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| six stones, one per row | YES | YES | YES | **NO** | ⛔ DISQUALIFIED |

- **one-stone Simple? NO** — two axis arguments and two role sets is two concerns. The whole reason
  the type-declaration five were one stone is that they shared every verdict; these do not.
- **all-`Unevaluated` Honest? NO** — measured: shape ③ evaluates and returns. Declaring otherwise
  would also trip `unevaluated_purity_carries_no_route_to_evaluation` the moment `role = eval` is
  annotated, and omitting that role to keep the pole would be the tail wagging the dog.
- **six-stones Good UX? NO** — three rows sharing one argument, split three ways.

## What this unblocks, and what it does not

```
mutation pair population       7 of 13 registered  →  12 of 13
still missing                  :wat::core::defstruct — a stdlib MACRO, unregisterable
```

⛔ **It does not unblock the mutation-pair flip.** That still waits on the fourth-registry fork
(`[[NOTE-there-is-a-FOURTH-registry-and-it-holds-defn]]`). Saying so here, rather than letting the
next self discover it, is the point of writing it down.

## Acceptance (both stones)

| what | expected |
|---|---|
| ⛔ shape ③'s examples RUN | each `@example` executes; `purity_mandated_examples` is satisfied for real |
| ⛔ shape ② has no eval route | `unevaluated_purity_carries_no_route_to_evaluation` inspects them and passes |
| ⛔ the gate can still FAIL | annotate a loader `role = eval` → RED |
| the DEBT ledger moves | +6, the named-absence conversion, unless a scheme exists (check per row) |
| `@syntax` FQDN-headed and real | `--check` a concrete instantiation of each |
| floor · clippy | green · 0 |
