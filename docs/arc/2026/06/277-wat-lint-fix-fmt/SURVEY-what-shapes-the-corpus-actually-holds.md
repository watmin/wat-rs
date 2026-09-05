# SURVEY — what shapes the corpus ACTUALLY holds (2026-09-05)

> **Builder's ask:** *"maybe we ... have grok or sonnet go hunt out 'good forms' and 'bad forms'
> .... as a starting sample set of what we should keep and what we shouldn't"*

**No rider was spawned, because no judgement was needed and a rider would have supplied one.**
"Good" and "bad" are the builder's call — `[[DESIGN-wat-fmt-the-rule-set-is-the-product]]`'s
dominating requirement is *"i will absolutely spot stuff i don't like."* A rider asked to hunt
good/bad forms returns **taste wearing a measurement's clothes**. So this survey **counts**; it
does not grade. Every row below is a shape that exists, with how often, and real source at
`file:line` for the builder to point at.

## HOW — self-hosted. The corpus answers about itself.

`wat-scripts/scratch-pad/277-layout-shape-probe.wat`, driven by the **existing** `wat --grep`:

```bash
git ls-files '*.wat' | sed 's/.*/"&"/' | tr '\n' ' ' | sed 's/^/[/;s/ $/]/' \
  | ./target/release/wat --grep ./wat-scripts/scratch-pad/277-layout-shape-probe.wat
```

```
1,740 files · 327,897 layout facts · 65s · 0 errors · 82,352 keyword-headed form instances
```

⭐ **NOTHING WAS BUILT FOR THIS.** `wat/grep.wat` already declares the exact fact base a layout
engine needs — `Node {id parent index kind}` · `Named {id name}` · `Span {id line col end-line
end-col}` — and `--grep` already drives rules over a file list from stdin. The DESIGN predicted
layout would need *"ORDER and NESTING → :parent :index :depth ON THE FACT."* It was already there,
shipped by arc 278. examinare: *the thing you would build almost always already exists.*

## ⭐ AND THIS DOUBLES AS SEQUENCING STEP 2 — the rete-shape probe. IT PASSED, FIRST RUN.

The probe's real job was to fail cheaply if rete could not express layout. Every layout rule
reduces to ONE capability: **join two children of the same parent and compare their LINES.** The
probe does exactly that, and needs a DERIVED fact (`:ls::Head`, asserted by rule 1) to feed rule 2
— the forward chaining the DESIGN said width-propagation would require.

```
:wat::rete::defrule's own shape, read off the probe's output:
  idx0 dline0 col2    the head
  idx1 dline0 col22   the rule name — same line
  idx2 dline1 col3    :when — next line, col 3
  idx3 dline1 col9    its vector — same line as :when
  idx4 dline5 col3    :then
  idx5 dline5 col9    its vector
```

That tuple set **is** the layout shape of a form. The shape is right; the style table is expressible.

## ⛔ THE NUMBER I NEARLY PUBLISHED WAS WRONG BY 7×

My first fingerprint was the raw `(dline, col)` tuple per child — which **includes arity**. A
6-child `defn` and a 9-child `defn` counted as different "shapes" even when both are perfectly
formatted. That said:

```
:wat::core::defn   5422 instances   775 distinct shapes   top shape 16%     ← WRONG. Arity noise.
```

read as *"the corpus is chaos, we have never adhered to anything."* The honest fingerprint is
arity-INDEPENDENT — how many children share the head's line, the set of relative indent columns,
and whether broken-out children get one line each:

```
:wat::core::defn   5422 instances   108 distinct styles   top style  66%    ← MEASURED.
```

**The corpus is far more consistent than the naive count claimed.** The pattern was right; the
population was wrong. `[[feedback_a_pattern_that_matches_a_subset_is_not_a_census]]`

## THE TABLE — keyword-headed forms, by frequency

```
HEAD                                   N ARITIES STYLES   TOP  the dominant style
:wat::core::defn                    5422       4    108   66%  5 on head line, indent +1, one-per-line
:wat::core::let                     3031      12     13   56%  1 on head line, indent +1, one-per-line
:wat::core::Vector                  2965      22     28   72%  3 on head line, ALL ONE LINE
:wat::kernel::println               2432       1      3   86%  2 on head line, ALL ONE LINE
:wat::core::match                   2141       8     88   63%  2 on head line, indent +1, one-per-line
:wat::kernel::assertion-failed!     1798       1     11   87%  4 on head line, ALL ONE LINE
:wat::string::concat                1786      10     43   55%  3 on head line, ALL ONE LINE
:wat::core::if                      1762       1     29   76%  2 on head line, indent +1, one-per-line
:wat::core::PersistentVector        1453      11     23   48%  3 on head line, ALL ONE LINE
:wat::core::=                       1312       1      7   96%  3 on head line, ALL ONE LINE
:wat::core::defrecord               1114       2      4   84%  3 on head line, ALL ONE LINE
:wat::core::fn                      1106       9     32   37%  4 on head line, indent +1, one-per-line
:wat::core::unquote                  697       1      1  100%  2 on head line, ALL ONE LINE
:wat::core::do                       572      28      5   89%  1 on head line, indent +1, one-per-line
:wat::rete::defrule                  427       2      3   58%  2 on head line, indent +1, one-per-line
:wat::test::deftest                  404       1      2   99%  3 on head line, indent +1, one-per-line
```

⛔ **THE `defrule` ROW WAS WRONG IN THIS FILE'S FIRST DRAFT** — I typed `24 styles / 15%` into the
arity-INDEPENDENT table by copying it out of the arity-DEPENDENT run, the exact run this section
retracts. Its real numbers are `3 styles / 58%`. A correction stated in prose does not correct the
table beside it. `[[feedback_a_patch_fixes_one_copy_of_a_claim]]`

**Read the STYLES and TOP columns together.** `unquote` at 1 style / 100% is settled and needs no
rule. `do` at 5 styles / 89% is nearly settled. `fn` at 32 styles / **37%** is the most contested
form in the language — no shape holds a majority. `defn` and `match` carry long tails (108, 88)
that are worth a rule precisely because the dominant shape is already strong enough to be a default.

## ★ THE RULE — the builder's, 2026-09-05. UNIFORM, and it is not a majority vote.

> *"i think the rule needs to be nuanced...."*

```
(wat.core/defn user/some-fn :- [I O]   ;; param-spec on same line as fn-name
  [x :- wat.type/i64                   ;; comments are aligned
   y :- wat.type/i64]                  ;; arg-spec with one arg per line
  :- wat.type/i64                      ;; ret-type on own line
  (wat.core/+ x y))                    ;; body....
```

**and the empty arg-spec is NOT an exception** — builder, same session:

```
(wat.core/defn user/some-fn
  []
  :- wat.type/i64
  42)
```

One rule, no special case:

```
line 1     head + fn-name + param-spec (`:- [P…]`) IF PRESENT
next       ARG-SPEC on its own line(s) — one argument per line, continuations aligned
           under the first. EMPTY `[]` INCLUDED: it gets its own line like any other.
next       RET-TYPE on its own line
then       BODY on its own line
```

**The nuance the fingerprint could not see: `defn` has TWO bracketed slots taking OPPOSITE rules.**
The **param-spec** (`:- [I O]`, generic type parameters) rides the head line. The **arg-spec**
always breaks out. The fingerprint keys children by INDEX, so it saw two vectors and treated them
as interchangeable — which is why a generic `defn` and a plain one landed in different "styles" for
a reason that has nothing to do with style.

⭐ **Already practiced** — `wat/bracket.wat:32`, live corpus, live syntax:

```
(:wat::core::defn :wat::bracket::runner-loop :- [I O]
  [self    <- (:wat::kernel::ThreadSelfPeer :- [O I])
   work-fn <- [I :-> O]]
  -> :wat::core::nil
  (:wat::core::match (:wat::kernel::recv self) …))
```

The `<-` binders are **column-aligned** (`self    <-` / `work-fn <-`) — the same discipline as
*"comments are aligned"* on a different column. **Alignment is a rule of its own and this survey's
fingerprint cannot see it at all** (it records indent columns, never intra-line column agreement).

## ⛔⛔ I ARGUED FROM FREQUENCY, AND FREQUENCY HAS NO AUTHORITY HERE

Two claims stood here before the builder cut them. The second is the real error.

**1 — The artifact.** I had written that *"the arc's own rule #1 is the corpus's 12% minority"* —
that the dominant `defn` shape had outgrown `[[NOTE-wat-fmt-structural-autoformat]]`. **60% of
every `defn` is ZERO-ARG**, and those 3,247 functions were being counted as votes against breaking
out an argspec they do not have:

```
defn forms measured                            5406
  EMPTY argspec  []                            3277   (60%)
  has actual parameters                        2129   (39%)
      of those: argspec ALREADY broken out     1174   (55%)
                argspec riding the head line    955   (44%)
```

⚠ Caught by a control: the first run reported **`0% empty argspecs`** while I had read
`(:wat::core::defn :user::main [] -> …)` by eye ten minutes earlier. An impossible zero is a broken
instrument, not a finding (the predicate compared an ABSOLUTE end-line to a RELATIVE line-delta).
The re-run names the control — *must find the instance I read by eye* — and prints whether it did.
`[[feedback_a_green_test_can_prove_nothing]]`

**2 — ⛔ AND THEN I USED THE FIXED NUMBER THE SAME WRONG WAY.** I proposed *"an empty arg-spec stays
on the head line"* as a flagged assumption, and my stated evidence was **that 99% of the corpus does
it**. That is a majority vote wearing a measurement's clothes, and this arc's first line forbids it:
*"i will never have all the rules that matter.. but i will absolutely spot stuff i don't like."*
**The rule is the builder's; the corpus heals to match.** A corpus records what happened to get
written before the tool existed — it cannot be evidence about what should be.
`[[feedback_optimize_for_the_expressivity_surface_not_the_corpus]]`

The honest reading of the same numbers: **~4,202 `defn` sites reformat** (3,247 zero-arg keeping
`[]` on the head line, plus 955 with parameters riding it). That is not a cost to weigh against the
rule — **it is the tool doing its job.** Step 3 of the SELF-FIXING-TOOLCHAIN loop is *"the floor
goes RED everywhere the old form lives"*, and *"we do not fear the cost of refactors."* 4,202 reds
is what a real rule looks like on its first run.

### What the rule implies for the rule set

- **A layout rule dispatches on the head symbol AND on NAMED SLOTS, never child indices.** `defn`'s
  slots are `name` · `param-spec?` · `arg-spec` · `ret-type` · `body…`, and the OPTIONAL param-spec
  shifts every later index by two. Index-keyed rules would need one variant per optionality
  combination — exactly how a rule set stops being "a new file and nothing else."
- ⭐ **The registry may already carry the grammar.** `Row.syntax` holds the `@syntax (...)` string
  and `src/intrinsic/mod.rs:3002` parses it *through the substrate's own reader*. If `defn`'s row
  names its slots, a rule can bind them by name instead of re-declaring the grammar.
  **NOT VERIFIED for `defn` — that is the next measurement, not a claim.**
- **Alignment needs facts this probe does not emit.** One-argument-per-line is a dline rule; the
  `<-` binders agreeing on a column is a rule about siblings' columns *within* the arg-spec; and
  aligned trailing comments needs the comment token the reader only learned to see this week.


## THE EXEMPLARS — real source, shortest instance of each style


====================================================================================================
## :wat::core::defn — 5422 instances, 108 distinct styles
====================================================================================================

--- DOMINANT: 3604 (66%) — 5 on head line, indent [1], one-per-line
    crates/wat-edn/demo/probe-oneshot.wat:13
    (:wat::core::defn :user::main [] -> :wat::core::nil
      (:wat::kernel::println

--- #2: 673 (12%) — 2 on head line, indent [1, 4], SHARED lines
    crates/wat-edn/wat-edn-clj/wat/shared.wat:31
    (:wat::core::defn :enterprise::observer::market::TradeSignal/show
      [sig <- :enterprise::observer::market::TradeSignal]
      -> :wat::core::String
      (:wat::core::format "[{asset}] {side} @ {size}"

--- #3: 577 (10%) — 6 on head line, ALL ONE LINE
    crates/wat-edn/demo/repl-daemon.wat:45
    (:wat::core::defn :user::main [] -> :wat::core::nil (:repl::serve))

--- #4: 81 (1%) — 3 on head line, indent [1, 4], SHARED lines
    tests/program/wat_arc170_program_contracts_t18b_recv_assert_fail.wat:20
    (:wat::core::defn :my::test::recv-assert-fail []
      -> (:wat::core::Result :- [(:wat::core::Vector :- [:wat::core::i64]) :wat::kernel::LociDiedError])
      (:wat::core::let

====================================================================================================
## :wat::core::fn — 1106 instances, 32 distinct styles
====================================================================================================

--- DOMINANT: 413 (37%) — 4 on head line, indent [1], one-per-line
    tests/collection/probe_collection_transform_ops.wat:71
        (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
          (:wat::i64::+ acc x))

--- #2: 300 (27%) — 2 on head line, indent [1, 4], SHARED lines
    tests/reflection/wat_arc201_extract_arg_types_arity.wat:6
                  [f     (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::String c <- :wat::core::i64]
                           -> :wat::core::String
                           b)

--- #3: 259 (23%) — 5 on head line, ALL ONE LINE
    tests/collection/probe_arc278_0c_persistent_parity.wat:16
        (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::+ acc x))

--- #4: 34 (3%) — 2 on head line, indent [1, 15, 18], SHARED lines
    tests/rete/probe_arc278_concurrent_retes.wat:90
          (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                          -> (:wat::core::PersistentVector :- [:wat::core::Record])
            (:wat::vector::conj acc (:cc::Item i)))

====================================================================================================
## :wat::core::let — 3031 instances, 13 distinct styles
====================================================================================================

--- DOMINANT: 1698 (56%) — 1 on head line, indent [1], one-per-line
    tests/cli/wat_cli__echo_program.wat:7
      (:wat::core::let
        [line (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))]
        (:wat::kernel::println line)))

--- #2: 1097 (36%) — 2 on head line, indent [1], one-per-line
    tests/collection/probe_arc216_stone5c_hashmap_native_storage.wat:40
        (:wat::core::let [m2 (:wat::hashmap::assoc m :bar 99)]
          (:wat::hashmap::length m2))))

--- #3: 137 (4%) — 3 on head line, ALL ONE LINE
    tests/cli/wat_repl__toplevel_expr.wat:1
    (:wat::core::let [x 1] x)

--- #4: 55 (1%) — 1 on head line, indent [11], one-per-line
    tests/function/tco.wat:40
      (:wat::core::let
                  [next (:wat::i64::- n 1)]
                  (:wat::core::if (:wat::core::<= n 0) 

====================================================================================================
## :wat::core::match — 2141 instances, 88 distinct styles
====================================================================================================

--- DOMINANT: 1350 (63%) — 2 on head line, indent [1], one-per-line
    tests/comms/probe_arc209_bound_listener.wat:20
          (:wat::core::match msg 
            ((:user::Op::Compute n)

--- #2: 267 (12%) — 6 on head line, ALL ONE LINE
    tests/comms/probe_arc209_bound_listener.wat:41
         c1   (:wat::core::match (:wat::kernel::connect addr) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))

--- #3: 101 (4%) — 1 on head line, indent [1], one-per-line
    tests/kernel/wat_run_sandboxed.wat:185
                 (:wat::core::match
                   (:wat::eval-file! "/nonexistent-in-child-loader.wat")
                   ((:wat::core::Ok h) (:wat::kernel::println "ok"))
                   ((:wat::core::Err _) (:wat::kernel::eprintln "err"))))))]

--- #4: 100 (4%) — 5 on head line, ALL ONE LINE
    crates/wat-edn/demo/probe-oneshot.wat:18
              (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))))))

====================================================================================================
## :wat::core::if — 1762 instances, 29 distinct styles
====================================================================================================

--- DOMINANT: 1351 (76%) — 2 on head line, indent [1], one-per-line
    tests/cli/wat_cli__sigterm_polling_loop.wat:12
      (:wat::core::if (:wat::kernel::stopped?)
        nil                                      ; observed stop → return clean
        (:demo::loop)))                          ; tight poll loop

--- #2: 185 (10%) — 4 on head line, ALL ONE LINE
    tests/collection/vector_algebra.wat:18
                (:wat::core::if (:wat::core::= c1 c2)  "yes" "no"))

--- #3: 71 (4%) — 3 on head line, indent [1], one-per-line
    tests/resolve/probe_arc251_decl_migrator.wat:13
      (:wat::core::if (:wat::core::= head-name ":wat::core::typealias") true
        (:wat::core::if (:wat::core::= head-name ":wat::core::newtype") true

--- #4: 67 (3%) — 1 on head line, indent [1], one-per-line
    tests/cli/wat_cli__presence_proof.wat:29
             (:wat::core::if
               (:wat::holon::presence? program-atom bound)
               "present"
               "absent"))

====================================================================================================
## :wat::core::do — 572 instances, 5 distinct styles
====================================================================================================

--- DOMINANT: 513 (89%) — 1 on head line, indent [1], one-per-line
    tests/macros/probe_kwargs_emitted_by_macro.wat:6
      `(:wat::core::do
         (:wat::core::defn :t::svc/add

--- #2: 38 (6%) — 3 on head line, ALL ONE LINE
    tests/program/probe_arc259_program_init_fn.wat:43
                    (:wat::core::do (:wat::core::/ 1 0) (:wat::program::EmptyEnv))))

--- #3: 13 (2%) — 2 on head line, ALL ONE LINE
    tests/cli/wat_grep__with_tilde.wat:6
      `(:wat::core::do ~x))

--- #4: 7 (1%) — 4 on head line, ALL ONE LINE
    tests/cli/wat_repl__toplevel_expr.wat:2
    (:wat::core::do 1 2 3)

====================================================================================================
## :wat::rete::defrule — 427 instances, 3 distinct styles
====================================================================================================

--- DOMINANT: 248 (58%) — 2 on head line, indent [1, 7], SHARED lines
    tests/rete/probe_arc278_5b_collect_rules.wat:11
    (:wat::rete::defrule :weather::cold-temp
      :when [(:weather::Temperature (?loc <- :location) (?c <- :celsius) (:wat::rete::i64::< ?c 0))]
      :then [(:weather::ColdAndWindy :location ?loc)])

--- #2: 173 (40%) — 2 on head line, indent [1], one-per-line
    tests/rete/probe_arc278_P12_explain_walk.wat:16
    (:wat::rete::defrule :weather::alert
      :when
      [(:weather::ColdAndWindy (?c <- :celsius) (?k <- :kph))]
      :then
      [(:weather::WeatherAlert :celsius ?c :kph ?k)])

--- #3: 6 (1%) — 6 on head line, ALL ONE LINE
    tests/rete/probe_arc278_leading_filter_multiplicity.wat:37
    (:wat::rete::defrule :lf2::r2 :when [(:lf2::S1 (?k <- :k))] :then [(:lf2::S2 :k ?k)])

====================================================================================================

## WHAT THE SURVEY DOES NOT SAY

- **It grades nothing, and it CANNOT.** Frequency is what we wrote before a formatter existed. The
  rule is the builder's; the corpus heals to match. Every sentence in this file that reads as
  "the corpus prefers X" is a description, never an argument.
- **It cannot see ALIGNMENT.** The fingerprint records a child's indent column, not whether
  siblings AGREE on a column. `self    <-` / `work-fn <-` and aligned trailing comments are both
  invisible to it. Two of the builder's four stated rules are therefore unmeasured here.
- **It cannot see COMMENTS at all** — they are facts the reader learned to emit this week
  (`lex_with_comments`, 9a16b68e6) and `wat/grep.wat` does not yet assert them.
- **The "short rides, long breaks" width hypothesis is DEAD.** I raised it; the builder's rule is
  uniform, so width does not select the shape. Nothing measured it and nothing needs to.
- **A form is keyed by `(file, head, head-line)`.** Two forms with the same head starting on the
  same line collapse into one — rare, and it undercounts one-liners rather than inventing shapes.
- **Trailing whitespace is visible in the exemplars** (`match msg ` at probe_arc209_bound_listener
  :20) and is a separate rule, not a layout shape.
