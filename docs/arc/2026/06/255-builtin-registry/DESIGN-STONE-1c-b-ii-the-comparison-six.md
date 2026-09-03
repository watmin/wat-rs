# DESIGN — STONE 1c-b-ii: `=` · `not=` · `<` · `>` · `<=` · `>=`

> Phase 1c-b, second stone. **The largest lever left in the campaign**, and not only for itself.

## Why these six, now

```
47 names · 1,796 call sites remain    (was 121 · ~5,000 when the campaign opened)

core::   11 names  1,162 sites    ← `=` alone is 688
rete::   30 names    278 sites
misc      2 names    333 sites    eval-ast! 330
type      4 names     23 sites    the arc-251 rendering fork
```

The six are **812 corpus sites**, and `=`/`not=` are the blocker for **eight rete rows** that have
waited since Phase 1b:

```
Alias  rete::string::=   rete::string::not=   rete::core::bool::=   rete::core::bool::not=
       rete::core::keyword::=   rete::core::keyword::not=
Form   rete::core::enum::=      rete::core::enum::not=
```

★ **Six authored rows take fourteen names off the corpus** — the six here, plus eight that become
pure transcription the moment `=` and `not=` have registry rows. Nothing else remaining is close.

⚠ Corrected from my own earlier statement to the builder: I said "six rete aliases." It is **eight
rete rows — six `Alias` and two `Form`.** The six was the Alias-class count, reported as the whole.

## The measured shape — a proven pattern, one new argument

```
rt:2672  ":wat::core::="    => eval_eq(head, args, list_span, env, sym)
rt:2673  ":wat::core::not=" => eval_not_eq(head, …)
rt:2674  ":wat::core::<"    => eval_compare(head, …, |o| o == Ordering::Less)
rt:2677  ":wat::core::>"    => eval_compare(head, …, |o| o == Ordering::Greater)
rt:2680  ":wat::core::<="   => eval_compare(head, …, |o| o != Ordering::Greater)
rt:2683  ":wat::core::>="   => eval_compare(head, …, |o| o != Ordering::Less)
```

All three handlers take `head` as their **first** parameter, and `eval_compare` additionally takes
a **predicate closure** — so none of the six can be annotated in place. **Six wrappers**, and the
four ordering wrappers must each carry their arm's closure verbatim. This is the identical
shim-collision shape 1c-b-i already solved (`format_ident!("__wat_intrinsic_shim_{}", fn_name)`
is keyed on the fn identifier), now with a closure to carry as well.

Checker side, one slot each, both through named helpers:

```
check.rs:3797  ":wat::core::=" | ":wat::core::not="  -> infer_equality   (inside infer_list)
check.rs:3813  "<" | ">" | "<=" | ">="               -> infer_ordering   (inside infer_list)
```

⛔ **`check.rs:2423` is NOT one of these.** It carries the same two FQDNs but lives inside
**`infer_rete_form`** — the rete-side routing for `:wat::rete::core::enum::=`. Two sites, two
slots, one of them out of scope. `[[feedback_a_slot_with_two_implementations_is_two_slots]]`.

## ★★★ The one genuinely new argument: what is `=` total ON?

Both runtime handlers carry an arity guard (outside totality's domain, the established carve-out)
**and a real `TypeMismatch` raise** — `eval_eq` on a `None` from its equality attempt,
`eval_compare` on a non-orderable operand. Whether that makes them `Partial` or `Total` turns
entirely on whether the checker makes those raises unreachable.

The evidence points both ways and must be read, not assumed:

- `infer_ordering`'s own comment says the orderings *"unify the two args (**strict same-type, no
  subtype path**), then gate on the orderable class"* — which reads like checker-unreachable, the
  same carve-out `:wat::core::get` was graded `Total` on this campaign.
- `infer_equality` is a different helper and may be more permissive. **Read it.**

⚠ The precedent for getting this wrong in the safe direction is on the record: 1c-a-ii graded
`conforms?` **`Partial`** precisely because its checker arm validated only syntax and never
resolved the type, so a well-typed call could still raise. **The question is not "does the runtime
raise" but "can a WELL-TYPED call reach that raise."**

## Acceptance — DERIVED

```
                  before   after   why
registry rows       542     548    +6 attribute sites (ANCHORED count)
GAP_A                49      49    none of the six is on it
GAP_B                52      46    all six are on it
DEBT                111     117    ⬅ +6, all six. The honest cost of Phase 1c-b.
KNOWN_UNREVIEWED     14      14    none of the six is on it — CHECKED against the constant,
                                   not assumed. (Two acceptance tables this campaign got this
                                   wrong by assuming; the third got it right by measuring.)
literal arms deleted  —       6
floor          5128/5128  5128/5128  registering a row mints no `#[test]` fn
the corpus           47      41    −6 names, −812 sites — and 8 more become unblocked
```

## Out of scope — CUT

- **The eight rete rows.** They become registerable the moment this lands, as pure transcription
  with a name and a target each. Their own stone (1b-iii), immediately after.
- `check.rs:2423` (`infer_rete_form`) — a different slot, serving the rete surface.
- `infer_equality`, `infer_ordering`, `eval_eq`, `eval_not_eq`, `eval_compare` — every shared
  implementation stays untouched. This stone adds wrappers and deletes arms.
- The declarations three, `str`, `None`, `eval-ast!`. Each its own stone.
