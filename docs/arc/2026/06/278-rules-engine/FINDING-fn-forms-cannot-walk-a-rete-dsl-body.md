# FINDING — `fn-forms` cannot walk a body containing rete PATTERN VARIABLES

**Arc 278, 2026-08-12. Surfaced by the child-entry strike, which is REVERTED.** This is the
blocking prerequisite: the one-entry model cannot land until this is resolved, and it is a
substrate gap the hand-enumerated manifest was HIDING.

## The failure, verbatim

```
malformed :wat::kernel::fn-forms form:
  tests/services/probe_arc278_sift_rules.wat:30:33:
  free symbol `?c` does not resolve to a parent define or substrate primitive

malformed :wat::kernel::fn-forms form:
  tests/services/probe_arc278_sift_rules_arena.wat:89:36:
  free symbol `?client` does not resolve to a parent define or substrate primitive
```

The offending source is ordinary, correct rete:

```clojure
(:wat::rete::defrule :usr::hot-rule
  :when [(:usr::Temp (?c <- :c) (:wat::rete::core::i64::> ?c 50))]
  :then [(:usr::Hot :c ?c)])
```

`?c` is a **rete pattern variable** — DSL binding syntax, not a program reference. `fn-forms`'s
closure walker treats every free symbol it meets as something that must resolve to a parent define
or a substrate primitive, so it **raises** instead of recognising `?c` as data belonging to the
rules DSL.

## Why this is the arc's problem, not a corner case

**The chaos engine (R25 `MACHINA CHAOS DOMAT`) — the arc's own target — is a rete service.** A
`defservice` whose state is a rete `Session`, layering `:wat::query::sift-rules-defsvc`, is exactly
the shape `fn-forms` cannot walk. So the one-entry model works for a plain service and fails for
the service this arc exists to build.

## EXPOSED, not created

Today the process arm ships a **hand-enumerated** `<fqdn>::service-forms` manifest, so `fn-forms` is
never asked to walk a rules-DSL body. Route form-shipping through the walk and the gap surfaces
immediately. This is the arc's recurring shape once more: **a workaround was masking a defect, and
removing the workaround is what found it** (R57 `IGNORANTIAM DELEMVS` — a law is completed by USE).

## It reaches BOTH tiers — the surprise

`own-forms-call` is spliced into `start`/`resume`, which run for **every** locus; the thread arm
ignores the resulting value but the call is still **evaluated**. So replacing that call with
`fn-forms` makes the thread tier pay the walk too:

**4 thread-tier failures, 4 process-tier, 2 tier-less** (10 of 4391; baseline was 4391/0):

```
sift_rules_defsvc_fails_closed_on_unknown_message_type_thread    sift_rules_arena_…_thread
sift_rules_defsvc_counts_exact_deductions_on_thread              sift_rules_arena_…_paged_on_thread
sift_rules_defsvc_fails_closed_on_unknown_message_type_process   sift_rules_arena_…_process
sift_rules_defsvc_counts_exact_deductions_on_process             sift_rules_arena_…_paged_on_process
every_wat_scripts_file_loads_on_the_current_runtime              a_forked_service_that_cannot_decode_a_message_speaks_its_reason
```

⚠ **The rider scored "thread tier untouched" GREEN** on one passing thread test
(`probe_arc209_c2_defservice_dispatch`) plus an empty `git diff wat/spawn.wat`. Both facts are
TRUE and neither can SEE the violation — four thread-tier tests were red at that moment. A pass
answers only the question the instrument asks
([[feedback_a_pass_answers_only_the_question_the_instrument_asks]]). Any future scorecard row of
the form "tier X untouched" must be measured by **the tier's whole test set**, never by a
representative.

## The other two reds

- **`every_wat_scripts_file_loads`** — `wat-scripts/scratch-pad/probe-arc278-fnforms-reaches-program-types.wat`
  calls `(:probe::ffx::service-forms)`, which the strike deleted. Real rot, correctly caught by the
  loader gate; it is a consumer of the retired API and would need migrating with the strike.
- **`a_forked_service_that_cannot_decode_a_message_speaks_its_reason_to_the_caller`** —
  **UNCHARACTERIZED.** It observed `#probe.Outcome/Message []` where it requires
  `#probe.Outcome/Lost [true]`. It exercises the forked-child startup/serve path the strike
  rewrote, so it is *plausibly* downstream of the same change — **but no mechanism was isolated,
  and "probably the same root" is not a disposition** ([[feedback_not_reproducible_is_not_a_disposition]]).
  Whoever takes the next attempt must characterize this arm on its own.

## The prerequisite, stated

**`fn-forms` must not raise on a free symbol that belongs to a DSL's binding syntax.** The shape of
the fix is a RULING, not an implementation detail, and the two candidates differ in where the
knowledge lives:

1. **The walker learns the boundary** — `fn-forms` stops descending into forms it does not own
   (a `defrule`'s `:when`/`:then` are rete data, compiled by the rete layer, not references to
   resolve). Risk: "forms I do not own" needs a principled definition, or it becomes a hardcoded
   list — the exact shape this arc keeps deleting.
2. **The DSL declares its own binders** — the rules layer marks pattern variables so any walker can
   skip them, the way `let`/`fn` binders are already understood.

Do not pick by convenience. Ground which layer legitimately OWNS the knowledge that `?c` is a
binder, then the walker's behaviour follows.

## What the strike DID prove, and should be kept

Reverted, but not wasted — three facts are now on the record and cost nothing to re-establish:

- **STOP-1 answered.** The abstract `:wat::spawn::Locus` arm of `infer_listener_prime`
  (`src/check.rs:9421`) is pinned to exactly 3 args (no budget slot); only the `ProcessOpts` arm
  accepts 3-or-4. `child-entry` needs 4, so its locus parameter types as
  **`:wat::spawn::ProcessOpts`** — checked and ran first try.
- **STOP-2 did not fire.** The `symbol-node`+unquote binders carry into a real `<fqdn>::` defn with
  no hygiene complaint; the `ProgramBodyIntroducesName` pressure is specific to `:user::main`.
- **`fn-forms`'s 2nd arg needs `keyword/from-string`, not a spliced literal.** A spliced literal
  keyword naming a registered fn auto-lifts to a `Fn` (arc-009 names-are-values) — fine for arg0
  (`f`), fatal for arg1, whose `name` param is typed `keyword`:
  `expects :wat::core::keyword; got :wat::core::Fn(wat::spawn::ProcessOpts)->()`. The same idiom
  `dispatch-admin-name-str`/`serve-name-str` already use.
- **Row 4's own result:** `manifest − walk = {<fqdn>::extract-addr}`. Traced: `extract-addr` is
  applied **parent-side only**, by `ProcessOpts`'s `Locus/launch` on the received `Status`
  (`wat/spawn.wat:575`), and is never referenced from `serve`/`dispatch-admin`/`child-entry` — so
  no walk rooted there can reach it, structurally. Whether it needs to ship at all is a real
  question for the next attempt, **not** a licence to hand-append it.

## Disposition

**Strike REVERTED** (`git checkout wat/service.wat`; tree clean at `2fff3749`, floor back to its
measured 4391/0). The DESIGN-STONE, BRIEF and EXPECTATIONS stay on disk — they are correct about
*what* to build; this finding is the prerequisite they did not know about. The red evidence is
preserved at `/tmp/…/RED-child-entry-evidence` (copied out of `.floor/`, which rotates) and quoted
verbatim above.
