# NOTE — P6-c is a CAMPAIGN, not a stone. Measured 2026-08-28 at `2502bf09b`.

> The worklist row reads *"the eval (111 arms) and tail (8 arms) matches collapse into registry
> lookups."* **The 111 is wrong, the framing is wrong, and the size is wrong by an order of
> magnitude.** Measured below with a stated, validated instrument. No row gets struck on a size
> nobody measured — and no row gets DRAWN on one either.

## The measurement

`dispatch_keyword_head_value` (`src/runtime.rs:5321-6885`), whose giant match spans `5365-6884`:

```
real arms ......... 136        distinct FQDNs ... 148
  :wat::core   82     :wat::rete    28     :wat::runtime  13     :wat::config   6
  :wat::stream  4     :wat::program  3     misc/other      12
```

> ⚠ **CORRECTED 2026-08-28 — this said 142, and 142 WAS WRONG.** P6-c-0's rider measured 148 with an
> independent structural tokenizer and reconstructed my error exactly: the arm at `runtime.rs:6168`
> is a `head @ (alt | alt | …)` bind-pattern spanning **three lines with TEN FQDNs**, and my
> instrument read only the line carrying the `=>`, capturing three. 142 + the 7 it never saw = 149
> literal occurrences, minus `:rust::` (a prefix guard, not a dispatched head) = **148.**
> Re-derived by my own second route and confirmed. **Fourth wrong count of this one population.**
>
> ★ **And the sharp part: I NAMED that hazard in this very file** — the paragraph below calls out
> "a multi-line pattern's continuation" as one of the two lines to discount. I saw it, wrote it
> down, and then still read only the one line. **Naming a hazard is not handling it; it can even
> substitute for handling it, because the write-up feels like the work.**
> `[[feedback_naming_a_hazard_is_not_handling_it]]`

**The instrument, and why it took three tries** — the number is only as good as this:

1. A regex for `"lit" =>` at depth 1 said **133 arms, no wildcard**. A `&str` match with no wildcard
   cannot compile, so the instrument was wrong, not the code. The fallback is written `other =>`.
2. Widened to any depth-1 arm at a hardcoded 12-space indent: **0 arms.** The indent is 8.
3. Deriving the indent from a histogram instead of assuming it: **137**, of which one is a COMMENT
   line containing `=>` and one is a multi-line pattern's continuation. **136 real arms.**

`[[feedback_validate_a_search_pattern_before_trusting_its_count]]` — three wrong counts of one
population, each corrected by a property of the answer (a match must be exhaustive) rather than by
re-reading the pattern.

## Why it is not "collapse into registry lookups"

**The registry-first door already exists, at `runtime.rs:5362`** — and again at `:5246` for the
TrackedValue path. Retiring an arm means *registering the verb and deleting the arm*; the door
catches it. `HOME-11` and `HOME-12` already did exactly this for thirteen verbs.

So P6-c is not an architecture change. **It is the HOME-* homing sweep, continued, over ~142 more
FQDNs** — and at HOME-*'s realised rate (10–15 per wave) that is **eight to twelve stones**, not one.

## ⛔ THREE HAZARDS, each already proven on a real site

**1. THE CENSUS MUST BE PER-FQDN ACROSS ALL DISPATCH SITES, NOT PER-ARM IN ONE MATCH.**
`:wat::config::set-redef!` is dispatched **twice** — `runtime.rs:2655` at FREEZE time, where it
mutates `sym.redef_allowed`, and `:5481` at EVAL time, where it is a deliberate no-op because `sym`
is immutable there. Both are correct and the second says so in its own comment. **Home the eval arm
naively and you move a no-op into the registry while the real behaviour stays behind.**
`[[feedback_a_slot_with_two_implementations_is_two_slots]]`

**2. SOME ARMS ARE SPECIAL FORMS AND MUST NOT BECOME INTRINSICS.** `:wat::stream::lazy` is labelled
in place as *"a SPECIAL FORM (capture-don't-eval)"*; `:wat::holon::literal` is already on
`eval_apply`'s `SPECIAL_FORMS` list. These want P6-a's `#[wat_special_form_impl]` mechanism, not
`#[wat_intrinsic]` — a different destination, not a harder version of the same one.

**3. THE SIGNATURES DO NOT ALREADY FIT.** `:wat::program::env` calls `eval_program_env(args,
list_span)` — no `env`/`sym`. `#[wat_intrinsic]`'s BINDING arm emits `#fn_name(args, env, sym,
list_span)` **unconditionally** (`wat_intrinsic.rs:726`, the fact Stone P7 turned on), so that
handler does not fit BINDING as written. Every such arm needs the H-1a/H-1b treatment — a real
declared shape — before it can be registered. **The sweep is not mechanical.**

## ⚠ And one suspicion I raised and then refuted, recorded so nobody re-raises it

The preamble comment at `:5328` says a rete op is *"never a per-op match arm added to the giant match
that follows"* — while the match holds 28 `:wat::rete::` arms. That reads as a comment shipping a
lie. **It is not.** `RETE_OPS` rows are `rete_name: ":wat::rete::i64::>"`-shaped **vocabulary
operators** for `where` clauses; the 28 are **engine verbs** (`fire-rules`, `insert-all`, `export`,
`lower`…). Two populations sharing a prefix. *"op"* there is a term of art and I read the general
meaning into it. Checked before publishing.
`[[feedback_an_adjacent_implementation_is_not_the_subject]]`

## The shape this wants

**Wave 0 — the disposition census, with its instrument COMMITTED**, before any arm moves. Per FQDN,
across every dispatch site, answering: homeable-as-intrinsic · special-form · multi-site · needs a
signature change first. Its output is the wave plan; its instrument must outlive the number.
`[[feedback_an_instrument_must_outlive_the_number_it_produced]]`

Then waves by namespace, smallest and most heterogeneous first — a mixed wave surfaces the
disposition axis early, which is what O-iv's nineteen correct refusals were worth.

## The census came back: SIX dispositions, not four — and one number reframes the whole campaign

The brief named four. The rider held STOP-1 twice and returned two more, both real:

- **DECLARATION-GUARD** — `:wat::core::def` and `:wat::core::defclause` are unconditional
  `Err(DeclarationInExpressionPosition)` arms. The real processing is freeze-time
  (`register_runtime_defs_form`); these exist only to give a clean diagnostic if a declaration
  reaches expression position. **There is no shape to fix** — homing one would register a function
  that always errors, duplicating the checker's own hard cut. The likely disposition is *delete*,
  once that cut is confirmed exhaustive.
- **CONTROL-FLOW-MULTI-MODE** — `if` · `let` · `do` · `match` · `and` · `or` · `ann-form` each have
  **three or more simultaneously-live implementations**, one per execution context. Verified:
  `:wat::core::if` is the giant-match arm, `eval_if_tail` (`runtime.rs:4411`, the TCO trampoline),
  and `step_if` (`:23501`, the stepper model). **Homing the eval arm alone leaves two siblings
  behind indefinitely.** ★ The precedent exists and is written down at `:4415`:
  `serve-dispatch-op` moved to the registry and the tail fallthrough reaches it *via registry
  lookup, calling the same delegate*. So the shape is proven — but it must be chosen, not stumbled
  into.

### ★★ 129 of 148 are NEEDS-SHAPE — and they may not need shaping at all

The rider's headline: **~87% of the arms fail `#[wat_intrinsic]` for ONE reason** — this file's
overwhelming convention is `eval_x(args, list_span, env, sym)` while BINDING's emit passes
`(args, env, sym, list_span)`. **The tail is the same three params in a different ORDER.**

Read as a sweep, that is a parameter reorder across ~120 callees, each needing its other call sites
re-verified — the campaign's dominant cost, and a reorder that compiles is not proof nothing called
it positionally.

**Read as a generator question, it may be one macro change.** `sniff_args` already walks the params
and marks `seen_context` at the first non-`&WatAST` one — it simply does not RECORD which context
params appeared or in what order, and `emit` then hardcodes `env, sym, list_span`
(`wat_intrinsic.rs:726`). Teaching the sniff to record the declared tail order and the emit to
honour it would retire the reason 87% of this population is blocked.

⛔ **Not drawn, and NOT to be assumed.** It is the same shape as Stone P7 (a classifier too narrow
for a legal declaration) and the same shape as P5-a→P5-b (the prerequisite that SHRANK its
dependent) — which is exactly why it must be *measured* rather than pattern-matched. If it holds,
the campaign is far smaller than eight to twelve stones. **That measurement is the next thing to
draw, before any wave.**

⛔ **This NOTE does not open those waves.** It sizes the row so the builder can decide whether arc
255 closes here with the campaign named, or carries it.
