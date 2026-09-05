# STYLE TABLE — draft. **EVERY UNRULED ROW IS MINE AND MEANT TO BE SHOT AT.**

> Builder: *"let's just work through the rules ..... we need stuff to critique.... i have no doubt
> that we can easily eye ball good from bad"*

Status column: **RULED** = the builder said it. **MINE** = my proposal, no authority, argue with it.
**OPEN** = I could not pick a side and the reason is stated.

---

## R1 · `defn` — **RULED** 2026-09-05

```
(wat.core/defn user/some-fn :- [I O]   ;; head + name + PARAM-SPEC on line 1
  [x :- wat.type/i64                   ;; ARG-SPEC own line(s), one arg per line,
   y :- wat.type/i64]                  ;;   continuations aligned under the first
  :- wat.type/i64                      ;; RET-TYPE own line
  (wat.core/+ x y))                    ;; BODY own line
```
Empty `[]` is **not** an exception — own line like any other arg-spec.
Blast radius ~4,202 `defn` sites. Practiced already at `wat/bracket.wat:32`.

---

## ⛔ R2 · `fn` — **OPEN, AND IT IS THE FIRST REAL FIGHT**

R1 applied verbatim to `fn` turns this — the single most common lambda shape in the corpus —

```
(:wat::core::fn [acc <- :wat::core::i64  x <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ acc x))
```

into this:

```
(:wat::core::fn
  [acc <- :wat::core::i64
   x   <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::i64::+ acc x))
```

**1,106 lambdas**, and `fn` is already the most contested form in the language (32 styles, no
shape above 37%). Most of them are one-expression reducers living inside a `foldl` argument list,
where four lines of ceremony wrap one line of work.

Three positions, and I do not think it is mine to pick:
- **(a) `fn` follows R1 exactly.** One rule, no exceptions to remember; the formatter is dumber
  and the reader always knows where the return type is.
- **(b) `fn` gets a budget.** Rides one line if it fits, breaks to R1's shape when it does not.
  Introduces the first WIDTH rule in the language, and therefore a number to argue about.
- **(c) `fn` is a distinct form with its own rule** — arg-spec stays on the head line, only the
  body breaks. Preserves what the corpus does; costs an exception.

⚠ Whichever wins decides something bigger: **whether this style system has a width budget at all.**
R1 has none — it is purely structural. (b) would be the camel's nose.

---

## R3 · `let` — **MINE**: binding vector on its own line, one binding pair per line

```
(:wat::core::let
  [m2   (:wat::hashmap::assoc m :bar 99)
   name (:wat::core::first xs)]
  (:wat::hashmap::length m2))
```
**Why:** a `let` binding vector is the exact analogue of an arg-spec — a bracketed list of
name/value pairs — so R1's treatment transfers with no new idea. The corpus is already 56% here.
⚠ **Weakest point:** 36% put the vector on the head line (`(let [x 1]` …), which is genuinely
tighter for a single binding. If R2 lands on a budget, this probably inherits it.

## R4 · `match` — **MINE**: scrutinee on the head line, ONE ARM PER LINE

```
(:wat::core::match msg
  ((:user::Op::Compute n) (:user::compute n))
  ((:user::Op::Halt)      nil))
```
**Why:** the scrutinee is not an arm and reads as part of the question. Arms are peers and get
peer treatment. Corpus is already 63% here. **88 distinct styles — the second most fragmented
form — so this rule earns its keep more than most.**
⚠ Unresolved: what happens when ONE arm's body is long. Sub-rule needed; a rider is looking.

## R5 · `if` — **MINE**: condition on the head line, then-branch and else-branch one per line
```
(:wat::core::if (:wat::kernel::stopped?)
  nil
  (:demo::loop))
```
**Why:** exactly R4's shape (a scrutinee and its peers) and the corpus is 76% here already. The
two branches must be visually parallel — that is the whole readability of an `if`.

## R6 · `do` — **MINE**: one form per line, always
89% already, 5 styles. Nearly settled; the rule just ratchets it.

---

## R7 · keyword arguments — **MINE**: one `:key value` PAIR per line when the form breaks

```
(:wat::grep::Match
  :file     ?f
  :line     ?l
  :end-line ?el
  :rule     "layout")
```
**Why:** a keyword-arg call is a record of named slots, and a named slot is a conceptual unit —
R1's own principle. **And the values want a column**, which is the same instinct as your
*"comments are aligned"*.
⚠ This is the rule most likely to be wrong in its details, and a rider is sampling it now.

## R8 · trailing comment alignment — **RULED in spirit** (*"comments are aligned"*), UNSPECIFIED in detail
Aligned to which column — the widest line in the block? a fixed stop? and what is a "block"?
⛔ **Cannot be built yet regardless**: comments only became visible to the reader this week
(`lex_with_comments`, 9a16b68e6) and `wat/grep.wat` does not assert a comment fact at all.

## R9 · no trailing whitespace — **MINE**, and it should be uncontroversial
Real instance: `tests/comms/probe_arc209_bound_listener.wat:20` ends `(:wat::core::match msg `.
Not a layout shape; a hygiene rule that costs nothing.

## R10 · indent unit — **MINE**: 2 spaces, and indentation is STRUCTURAL, never alignment-to-a-paren
```
BAD   (:wat::core::fn [acc <- …  i <- …]
                      -> …               ← indent depends on the length of "fn "
        body)
GOOD  (:wat::core::fn [acc <- …  i <- …]
        -> …
        body)
```
**Why:** an indent that depends on a name's length re-indents the whole form when the name
changes — the diff noise that makes a formatter worth having. Real instance of the bad form:
`tests/rete/probe_arc278_concurrent_retes.wat:90` (indents at +1, +15, +18).
★ I believe this is the single highest-value rule after R1, because it is the one that makes
every other rule's output STABLE under renaming.

---

## ⛔ WHAT NO RULE HERE CAN YET EXPRESS

- **Alignment of any kind** — R1's aligned binders, R7's aligned values, R8's aligned comments.
  The layout probe records a child's indent COLUMN; it cannot see siblings AGREEING on a column.
  Three ruled-or-proposed rules rest on a fact nothing emits yet.
- **Comments** — see R8.
- **Whether a width budget exists at all** — R2 decides it.

---

# RIDER FINDINGS — 4 riders, 4 form families, 2026-09-05. Verified against the disk.

## ⭐⭐⭐ R11 · SIBLING BREAKING IS ALL-OR-NOTHING — **MINE, and it is the most valuable rule here**

**The corpus's monster lines are not caused by width. They are caused by half-broken sibling sets.**

`tests/services/probe_arc170_m1_teeth_revoked.wat:95` — **1,096 columns, verified**. It is a `match`
whose INNER arms were each given their own line and whose OUTER arms were then appended to the tail
of the last inner one, closing parens and all:

```
…(:wat::kernel::assertion-failed! "unexpected RequestMalformed" …)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::…
                                                                   ↑ four closes, then the NEXT ARM, same line
```

Whoever formatted it broke the inner `match` and never came back to the outer one. The rider found
the same shape at `:78` (800 cols), `probe_arc170_m1_teeth_admitted.wat:67` (778), and in ~6 more
arc-170 probes — copy-pasted, so it is ONE authoring mistake replicated, not six judgements.

**The rule:** if ANY sibling in a form's child list is on its own line, EVERY sibling is. Arms of one
`match`, branches of one `if`, pairs of one `let`, fields of one record. No form is half-broken.

⛔ **THIS IS THE ANSWER TO R2's WIDTH QUESTION, AND IT ARGUES AGAINST A BUDGET.** A width budget
would not have caught line 95 — the author was already breaking lines; they just stopped partway.
R11 catches it structurally, with no number to argue about. **A width rule is neither necessary nor
sufficient for the corpus's worst damage.**

## R12 · CROSS-CALL TABLE ALIGNMENT — **OPEN. Real, deliberate, and nobody wrote it down.**

`wat-scripts/scratch-pad/rules-corpus-02-gates-and-unknowns.wat:176-180`, verbatim:

```
(:m::Member :id 1 :prefix "String" :base "concat"    :style "slash")
(:m::Member :id 2 :prefix "string" :base "length"    :style "colons")
(:m::Member :id 3 :prefix "i64"    :base "to-string" :style "colons")
```

Values padded into columns **across sibling calls** — a fact table, not a value list. Also at
`rules-corpus-01-node-facts.wat:121-125`, and the same instinct in a data literal at
`wat-scripts/fixes/reclaim-service-fixture-names.wat:40-58`.

⚠ **Every rule in this table so far reasons about ONE form's own children. This one reasons about a
form's NEIGHBOURS** — it needs a fact no rule has ("my sibling is a call to the same head with the
same keyword sequence"). If wat-fmt does not learn it, canonical formatting **destroys** these three
hand-built tables. That is a real cost of "canonical", and it should be paid knowingly or not at all.

## R13 · NO BLANK LINE BETWEEN A HEAD AND ITS FIRST ARM — **MINE**
59 occurrences of a `match` head followed by a blank line before arm 1 (`wat/query/sqlite-store.wat`
×5, `wat/rete/acc.wat:151`), and the same tic on `if` (`wat/core.wat:1687,1694,1706,1793`). Consistent
within a file, absent everywhere else — a local habit, not a convention.

## R9 · trailing whitespace — **MEASURED: 941 lines across the corpus.**
Includes a distinct flavour the riders named: a space left *before* a deliberate wrap
(`(:wat::core::Option/expect  ⏎`) at `wat/rete/acc.wat:63,82,115,147`. Free to fix, invisible in most
diff viewers, and it will touch many files on the first run.

## ★ R14 · `defservice` — clause keyword per line, values aligned. **20/20 UNIFORM in the corpus.**
```
(:wat::service::defservice :wat-tests::recorder
  :satisfies :wat-tests::Recorder
  :durable   [total <- :wat::core::i64]
  :ephemeral []
  :impls     [...])
```
The most uniform named-slot form found. It already IS R1's principle (one conceptual slot per line,
values in a column) applied to keyword clauses — so R7 should be stated as *generalising* R14, not
inventing something.

## ⛔ THE CENSUS THAT TESTS THE DESIGN'S CENTRAL CLAIM — head-symbol dispatch

`wat-scripts/scratch-pad/277-head-kind-census.wat`. The DESIGN rests extensibility on *"a layout
rule dispatches on HEAD SYMBOL"*, which presumes every form HAS one.

```
OF 89,968 FORMS WHOSE PARENT IS A LIST:
  literal keyword head    81,722   90%   ← head-dispatch works
  LIST head                4,840    5%   ← cannot dispatch on a name
  bare symbol head         3,185    3%   ← nameable, but no registry row
```

⚠ **The 5% is still confounded and I am not publishing it as the answer.** A `match` ARM
`((:wat::core::Ok v) body)` is a list whose child-0 is a list, and arms are counted here. The v1 rule
was worse — it fired on child-0 of *every* node, counting the first element of every vector and map,
and reported **9,115**. That number is retracted; the amended rule joins the parent's kind.

What survives: **90% of forms dispatch cleanly on a literal head, and the remainder is real but
small.** Match arms need dispatch on their PARENT's head, not their own — which the fact base
already supports (`Node.parent`) and no rule has used yet.

## THE SMALL FINDINGS WORTH KEEPING

- **`:wat::core::when` is DEAD** — zero call sites corpus-wide; its only mention is a probe naming it
  an orphan (`255-probe-are-the-orphans-live.wat`). Not a formatter matter; a registry one.
- **`wat/sqlite.wat:52-55` is the best comment-alignment exemplar in the corpus** — better than
  `bracket.wat`'s. Variant names padded so every `[` lines up AND every `;;` lines up.
- **The stdlib already violates R1** — `wat/fix.wat:1028` and `:1083` disagree with each other about
  crowding two args, 55 lines apart in one file.
- **`wat/core.wat:1405` vs `:673`** — `defn`'s own defining macro splits `name` and `& rest` onto
  separate lines; `->`'s puts `acc` and `& steps` together. Same file, same era, two answers for the
  fixed-arg-plus-variadic shape.

---

# ⛔ R15 · THE LINE BUDGET — **RULED 120-200. Recommending 120.**

> **Builder, 2026-09-05:** *"i'd say chars per line should be limited to like... 120 - 200 .... the
> thing we're optimizing for is two fold - good code is readable code - this is for setting the tone
> for future code as we build only exemplars and cognition of what's written for both llm and human
> -- all minds find beautiful code more readable..... this is not an optional thing.... the code
> must look good for us to be successful."*

★ **The stated objective changes what the formatter IS.** Not tidiness — the corpus is the training
set for every mind that will read it, ours included. A malformed exemplar teaches malformation to
the next reader, and most of this project's readers are models reading the corpus to learn the
house style. **Beauty is a correctness property here, not a preference.**

## Measured, to choose WITHIN the ruled range

```
102,828 lines    mean 60.8    median 59    p90 98    p95 109    p99 172    max 17,462

>100  7,894  (7.7%)      >150  1,514  (1.5%)      >300  466  (0.5%)
>120  3,436  (3.3%)      >200    794  (0.8%)
```

**120 sits just above p95 (109).** The corpus already writes to roughly that width — 95% of lines
comply today — so 120 ratchets a discipline that mostly exists and surfaces **3,436 real offenders**.
200 would bless the 2,642 lines between 120 and 200, which is where a lot of the genuinely
unreadable material lives.

⚠ **This is a COST measurement, not a norm derivation.** The builder ruled the range; the
distribution only says where inside it the blast radius sits. (I made the other mistake earlier
today and it is recorded above — frequency has no authority over what the rule *should* be.)

# ⛔⛔ R16 · THE ONE-LINERS ARE MANUFACTURED, NOT AUTHORED — FIX THE MANUFACTURER

> **Builder:** *"the giant oneliners are a result of various code mods that get used as references
> to be repeated - i have had very minimal tolerance for this stuff."*

**Confirmed, and it is bigger than the riders found.** One expansion, verbatim, one line:

```
(:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) …
```

```
135 FILES carry it.   median 414 columns.   max 615.
  85 of them are wat-scripts/fixes/ — THE CODEMODS THEMSELVES
```

And it is not alone. The `__`-prefix is **macro-hygiene naming** — these are *expansions* sitting in
hand-maintained source:

```
__cause 530   ·   __recv 290   ·   __datum 278   ·   __forms 243      ≈1,341 occurrences
```

★ **This is extirpare's exact shape: the stem is 135 ugly files; the root is that they are
REPLICATING.** Each new codemod is copied from the last, so the ugliness is manufactured faster than
any sweep can clean it. Reformatting the 135 without fixing the manufacturer buys one commit of
relief and then regenerates.

**The rule: a codemod's OUTPUT is corpus, and must be canonical.** `wat fmt` runs as the closing
step of any tool that writes `.wat` — `wat/fix.wat`'s appliers included. A tool that emits
un-canonical text is a tool that manufactures findings for its own linter.

⭐ **AND THIS RELOCATES wat-fmt's FIRST CONSUMER.** The DESIGN names doc examples as the payoff
(*"an ugly example is a red floor, not a matter of taste"*). Doc examples are 609 rows in one
generated fence. **The codemod corpus is 135 hand-maintained files that actively breed.** The
formatter earns more, sooner, pointed there.

⚠ **NOT YET ANSWERED, and I am not guessing:** why is macro-EXPANDED text in source at all? Either
a codemod once rewrote `readln` call sites into their desugared form (and the corpus lost an
abstraction, which is a defect of a different and possibly worse kind), or the expansion is the
required no-hidden-failures handling and `__datum` is incidental. **That is a measurement, and it
belongs to whoever takes R16.**

---

# ⬜ THE THREE OPEN DECISIONS — everything else is ruled or is mine to defend

Nothing in this table waits on the `__` sweep, on the registry, or on the reader. These three are
the only things blocking a rule set that can be written down and driven.

| # | decision | my recommendation | why it is not mine to settle |
|---|---|---|---|
| **1** | **R15 — 120 or 200?** | **120** | p95 is 109, so 95% of the corpus already complies; 120 surfaces 3,436 offenders while 200 blesses the 2,642 lines between, where much of the unreadable material lives. But the range was ruled, not the number. |
| **2** | **R2 — does `fn` follow R1?** | **(c), its own rule** — arg-spec stays on the head line, only the body breaks | 1,106 lambdas, most of them one-expression reducers inside a `foldl` arg list. R1 verbatim makes every one of them 4 lines. But an exception is a thing to remember forever, and (a) keeps the system dumb. |
| **3** | **R12 — cross-call table alignment: preserve or destroy?** | **preserve** | Three sites hand-align values into columns across sibling calls. Canonical formatting flattens them unless the engine learns a NEIGHBOUR fact — the only rule here that reasons past one form's own children. It is a real cost of "canonical" and should be paid knowingly. |

⭐ **R11 (sibling breaking is all-or-nothing) is the one I will argue for hardest**, and it is not in
this table because I do not think it is contentious: it is the rule that fixes the corpus's actual
worst damage (the 1,096-column half-broken `match`), and it does so structurally, with no number.

---

# ✅ THE THREE DECISIONS, RULED 2026-09-05

## 1 · R15 — **120. RULED.** *"120 chars is fine for now"*

## 2 · R2 — `fn` is BUDGET-TIERED. **RULED in substance:**

> *"can you fit param-spec, arg-spec and ret-spec on the same line?... the body may be placed on the
> second or more lines then?"* · *"if it can be expressed as a oneliner within char budget its
> allowable?.. other wise it needs full breakout?"*

⛔ **ONE THING HAD TO BE RESOLVED TO MAKE THIS BUILDABLE, and it is the design's ruled model.**
*"Allowable"* is a **lint** word — several forms pass. *"Canonical"* is a **formatter** word — one
input, one output. If a one-liner AND a two-line form both merely *pass*, `wat fmt` has nothing to
PRODUCE, and it silently becomes cljfmt's normalise-only model that
`[[DESIGN-wat-fmt-the-rule-set-is-the-product]]` explicitly overruled.

**Resolution — GREEDY: put the most on the head line that the budget allows.** Deterministic, total,
and it is what the builder's own three sentences describe:

```
tier 1   whole form fits 120                    →  one line
tier 2   else signature (head .. ret-type) fits →  signature on the head line, body below
tier 3   else                                   →  full R1 breakout
```

### Measured at 120 — `fn`, 1,104 forms
```
tier 1  whole form fits          209  18%
tier 2  signature fits           449  40%
tier 3  signature exceeds 120     38   3%   ← the ONLY lambdas needing full breakout
        signature currently split 408  36%   ← greedy RE-JOINS most of these

signature width (open-paren .. ret-type):  median 68 · p90 112 · p95 126 · max 190
```
**97% of lambdas stay compact.** And 120 genuinely bites — it sits between p90 and p95 of signature
width, so the budget is a real constraint on this form rather than a no-op.

### ★ WHY `defn` IS UNCONDITIONAL AND `fn` IS TIERED — my reading, offered for confirmation

R1 breaks a `defn` ALWAYS, empty `[]` included. R2 breaks an `fn` only when the budget forces it.
That is not an inconsistency if the distinction is:

> **A `defn` is a LANDMARK** — a top-level definition, found by scanning, read far more often than
> written. A predictable shape is worth vertical space.
> **An `fn` is INLINE** — an argument inside an expression, read in the flow of the form containing
> it. Ceremony there costs the reader more than it buys.

⚠ **If that principle is right it is worth more than either rule**, because it classifies FUTURE
forms without another ruling: `defrecord`/`defstruct`/`defenum`/`defservice` are landmarks
(unconditional); an inline `let` binding's lambda is not. **Stated as MY inference from two
rulings, not as something the builder said.**

## 3 · R12 — cross-call tables **PASS** when within 120 and column-aligned. **RULED.**

> *"i think if you can express them in 120 chars and they are all space aligned... they pass?"*

⛔ **Same canonical/allowable split, and it needs the same answer.** "They pass" makes an aligned
table a **fixpoint** — `fmt(x) == x`, so the formatter leaves it alone. But a formatter that only
*tolerates* alignment cannot be canonical, because an UNALIGNED sibling group would also pass, and
then two renderings of the same input are both correct.

**Resolution: canonical means the formatter PRODUCES the alignment.** An aligned group is already a
fixpoint (so the three hand-built tables survive untouched, which is what the ruling protects); an
unaligned one gets aligned (so `wat-scripts/perf/grid/where-boolean.wat:259` heals instead of
staying blessed). One rule, one output, and the ruling's intent preserved exactly.

⚠ **The engine cost is real and unchanged:** this is the only rule that reasons past one form's own
children — it needs a NEIGHBOUR fact ("my adjacent sibling calls the same head with the same keyword
sequence"). Everything else in this table is local.

---

# ⭐ THE BUDGET ACTIVATES THE MACHINERY THE DESIGN ALREADY PREDICTED

R15 and R2's tiering are **width-dependent**, so a rule must know a form's RENDERED WIDTH before it
can choose a tier — and a parent's width depends on its children's. That is exactly what the DESIGN
called for before any of this was ruled:

> *"a node's rendered WIDTH depends on its children's widths → bottom-up derivation to a FIXPOINT"*
> *"does this form fit the budget? → acc::sum over the matched child set"*

`fire_fixpoint_delta` and `wat/rete/acc.wat` exist. The budget ruling is what gives them a job.
Before today the layout rules were purely structural and needed neither.

---

# ★★ THE EXPLODED FORM IS THE RULE — 2026-09-05, and it retires R2's tiers

> **Builder:** *"i do not think i'll fight this one.... we add compression rules later... get a
> proper 'verbose' or 'exploded' form first... maybe we only support the exploded form..."*

```
(wat.core/foldl
  (wat.core/fn
    [acc :- wat.type/i64
     x   :- wat.type/i64]
    :- wat.type/i64
    (wat.core/+ acc x))
  0
  xs)
```

**One argument per line. No packing. `0` and `xs` each get their own line.**

## ⛔ THIS RETIRES R2 AND TAKES THE WIDTH FACT OFF THE CRITICAL PATH

R2 was ruled budget-tiered — *whole form fits 120 → one line; else signature fits → signature on the
head line; else full breakout*. **`fn` was the ONLY form whose layout depended on how wide it was.**

Under the exploded form it does not. `fn` breaks like everything else, unconditionally.

```
BEFORE   layout needs a form's rendered WIDTH -> a 5th fact from the walk -> a whole stone
AFTER    no layout rule needs a width at all
         R15 (120) demotes from a DRIVER of layout to a CHECK on it — a lint, not an input
```

⭐ `[[NOTE-width-is-a-fact-not-a-rule]]` stays true and stays worth keeping — rete still cannot
derive width, and R15-as-a-lint will still want the fact. **It is simply no longer a prerequisite for
any layout rule.** One stone leaves the critical path.

## ⚠ "EXPLODE EVERYTHING" IS NOT LITERALLY THE RULE — the unit is a SLOT, not a child

Two things pack in shapes the builder has already ruled, and they are not exceptions to be
remembered — they fall out of the existing mechanism:

```
-> :wat::core::i64      the arrow and its type share a line   (2 AST children)
[y (:wat::core::+ x 1)  a binder's name and value share a line (2 AST children)
```

★ **The mechanism already expresses this and needs no new idea.** A `Break` is asserted for the node
that STARTS a line; a child with no `Break` simply follows after a space. So:

> **"exploded" = assert a Break for every child. "packed" = do not assert one.**

The whole rule set is therefore *"which children get a Break"* — and the exploded default is
*"all of them"*. A specific rule exists only to withhold a Break where a slot spans more than one
child (`defn`'s name riding the head line, `->` and its type, a binder's name and value).

## WHAT THIS MAKES OF EACH RULE

| rule | what it actually says now |
|---|---|
| **R11 default** | assert a Break for EVERY child. This is the rule; the rest are refinements. |
| **R1 `defn`** | withhold from `:name` and the param-spec (they ride the head line) and from the ret TYPE (it follows `->`) |
| **R3 `let`** | withhold from each binder's VALUE (it follows its name) |
| **R4 `match`** | withhold from the scrutinee (it rides the head line) |
| **R2 `fn`** | **nothing to withhold — plain explode.** The tiers are dead. |

⛔ **AND THE CLAIM-GRANULARITY DEFECT SURVIVES THIS UNCHANGED.** It is about R11 racing a specific
rule inside a container the specific rule laid out; the exploded ruling changes what the rules SAY,
not who owns which node. `[[REFUTE-claim-the-forms-you-position-not-the-subtree]]` still stands and
is still the next stone.
