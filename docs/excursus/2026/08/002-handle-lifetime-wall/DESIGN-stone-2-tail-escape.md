# DESIGN — stone 2: the tail escape

Stone 1 catches a peer escaping as a scope's VALUE. Stone 2 catches the shape that actually cost
38 days, where the peer never escapes at all — the scope dies underneath it:

```wat
(:wat::core::let
  [h (:app::svc/start …)
   c (:app::conn h)]
  (:app::drive c))          ;; tail position: this scope ends BEFORE the call runs
```

`drive` receives a live peer to a service that no longer has an owner.

## Why the checker cannot express this today

The scope dies iff **both**:

1. the let's tail expression is a **user-function call** (only those emit `EvalSignal::TailCall`;
   a builtin head does not), and
2. **the let itself is in tail position**, so it was evaluated by `eval_let_tail` rather than
   `eval_let` — the non-tail twin keeps its scope alive across the body.

Condition 2 is the one the checker cannot answer. All 21 `tail` hits in `check.rs` are
`strip_prefix` string tails; the concept does not exist there.

## The runtime already owns the definition — do not write a second one

`eval_tail` (`src/runtime.rs:4360`) is the authority, and its table is **closed and small**. Seven
forms carry tail position into a sub-expression, each with an `eval_*_tail` sibling:

| form | tail evaluator |
|---|---|
| `:wat::core::if` | `eval_if_tail` (`runtime.rs:4560`) |
| `:wat::core::match` | `eval_match_tail` (`:4719`) |
| `:wat::core::let` | `eval_let_tail` (`:4618`) |
| `:wat::core::do` | `eval_do_tail` (`:4693`) |
| `:wat::core::and` | `eval_and_tail` (`:4808`) |
| `:wat::core::or` | `eval_or_tail` (`:4843`) |
| `:wat::core::ann-form` | `eval_ann_form_tail` (`:4879`) |

Dispatched at `runtime.rs:4415-4438`.

★ **THE CONTRACT DECISION, AND THE WHOLE RISK OF THIS STONE.** A checker-side list that merely
*happens* to match this one is a second source of truth, and the two will drift — a form gains a
tail variant, the checker does not learn, and the wall goes quietly wrong in BOTH directions (it
misses real escapes and invents false ones). The failure would be invisible: the floor stays green
either way.

So: **one list, two consumers.** Extract the tail-carrying set into a single shared constant that
`eval_tail`'s dispatch and the checker both read. If the shapes genuinely forbid sharing, the
fallback is a drift gate — a test that fails when the two disagree — and the strike must say plainly
which it built and why. **A duplicated list with no gate is a FAIL even with a green floor.**

## The rule

> At a `let` that CREATES a `Handle` of service S, if that let is in tail position AND its tail
> expression is a user-function call taking an argument of type `(Peer :- [S::Op S::Reply])`, reject.

Reuses stone 1's machinery unchanged: creation detection (a call whose scheme returns a service
Handle aggregate and takes none) and the peer→surface relation. The ONLY new concept is tail
position.

## ⚠ The collision, and the principle it forces

`wat-scripts/scratch-pad/probe-self-sched-bisect.wat` is the instrument that diagnosed this whole
excursus. It contains **three** deliberate tail escapes — `hold-in-body`, `hold-as-param`,
`plain-service-tail` — because measuring the defect requires constructing it. The wall rejects all
three, and the loader gate then turns the floor red.

Stone 1 hit the same wall and the answer was to MOVE the file. **That answer is wrong here**: the
bisect probe is not a static target, it is a program that RUNS and prints the discrimination table
(`A-binding=3 B-body-tail=-11 …`). A rejected file cannot run, so moving it does not save it.

The right answer is a `rune:`, and it sharpens the stone-1 lesson into a rule worth keeping:

> **Rune the INSTRUMENT. Never rune the ACCEPTANCE CRITERION.**

The bisect probe and `probe_severed_reaches_the_client` are instruments — they must construct the
forbidden state to measure it, and a rune with a stated reason is exactly right. The red probe is
the acceptance criterion — runing it would produce a green floor from a wall that fires on nothing,
which is the trap the executor correctly refused on stone 1.

## The census cannot be a grep

Stone 1's acceptance criterion was a static grep (18 Peer-returning functions). This shape is not
greppable — it depends on tail position, which is a property of the AST, not the text. **The census
must be run WITH the wall in place**: build it, run `--check` across the corpus, and read what it
rejects.

Expected rejections, all deliberate, all in probes/instruments:
- `probes/red-tail-escape.wat` (to be written — the acceptance criterion)
- `probe-self-sched-bisect.wat` ×3 (`hold-in-body`, `hold-as-param`, `plain-service-tail`)
- `probe-tail-scope-sees-bindings.wat` — `:c2::the-tail-escape-the-wall-must-reject`

**Anything else is a finding, not a nuisance.** A real site would mean live code is severing a
service today, and it must be reported rather than runed.

Out of scope = REJECTED: any runtime change (TCO stays exactly as it is — it is correct and
load-bearing); any change to stone 1's rule; `LociDiedError` and the severed sentinel.
