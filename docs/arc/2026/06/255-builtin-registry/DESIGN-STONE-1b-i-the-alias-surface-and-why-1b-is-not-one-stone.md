# DESIGN — STONE 1b-i: the 28 Alias rows, and why 1b is NOT one stone

> Governed by `[[RULING-the-registry-is-the-sole-authority]]`, phase **1b** of
> `[[DESIGN-CAMPAIGN-the-registry-becomes-the-sole-authority]]`. Rests entirely on
> `[[DESIGN-STONE-2a-b-an-alias-inherits-it-does-not-declare]]` — without it this stone would be
> 28 five-axis arguments instead of 28 one-line facts.

## ⛔ THE SEAM'S OWN NEXT WAS WRONG, AND THIS IS THE MEASUREMENT

The breadcrumb said: *"Phase 1b — the 54 ALIAS rows (Alias 35 · Form 9 · Redispatch 10) — a name
and a target each, no axis authoring… 66 of the 107 live here."*

**A name and a target each is true for 37 of the 54, and 66 is not a number this phase can
deliver.** Measured 2026-09-02 by intersecting `RETE_OPS`'s `core_name` column against the 490
names the registry actually holds (extracted from `#[wat_intrinsic]`/`#[wat_special_form]`
attributes — the ONLY two registration paths; every other `inventory::submit!` in the tree feeds
a different collector):

```
                rows   target REGISTERED   BLOCKED
Alias             35          29              6     → :wat::core::= / not=
Form               9           6              3     → cond · = · not=
Redispatch        10           2              8     → PersistentVector · Vector · PersistentMap ·
                                                       Tuple · foldl · map · filter · reduce
                  ──         ──             ──
                  54          37             17
```

★ The block is not a judgement call — `no_dangling_or_chained_aliases` (`src/intrinsic/mod.rs`)
is a live floor test that reds with `DANGLING @alias … target names no registered row`. **You
cannot alias to a name the registry cannot vouch for.** That is the RULING's forced order
(*registry answers → consumer asks → duplicate dies*) asserting itself for the third time in this
arc, and this time it was caught before a brief shipped rather than after.

⚠ And every one of the 11 blocking targets is **itself among the 107** — `=` (688 corpus call
sites), `PersistentVector` (483), `foldl` (380), `first` (362), `Tuple` (271), `PersistentMap`
(222), `map` (65), `filter` (9), plus `Vector`, `reduce`, `cond`. The WORKLIST already warned
that these *"are GAP_B's own population and need their own stones"* and that **"counting them as
one number is how a plan gets written that cannot be executed."** The SEAM's NEXT did exactly
that, in my own voice, four commits after the WORKLIST said not to.

## ★★★ THE PROBE — run BEFORE this design was written, and it moved the cut

Three rows added, one per class, then the full floor (`scripts/floor.sh`). Not committed;
removed after. The point was to find out what reading cannot tell you: does registering a
`RETE_OPS` name as an `@alias` row trip any existing gate?

```
:wat::rete::i64::<          Alias       plain strict fn
:wat::rete::core::and       Form        lazy AND variadic — exercises the @syntax arity path
:wat::rete::core::List      Redispatch  container constructor
```

```
Summary [119.518s] 5127 tests run: 5124 passed, 3 failed, 17 skipped
```

**Every structural wall PASSED** — `no_dangling_or_chained_aliases`,
`every_special_form_carries_check_and_eval_impls`, `every_registered_syntax_parses`,
`no_two_submissions_claim_the_same_fqdn`,
`registry_first_door_owns_every_handler_row_no_literal_arm_survives`,
`unevaluated_purity_carries_no_route_to_evaluation` — **and the entire rete test surface.** The
three reds were the ledger ratchets, each naming its own edit by name.

★ The mechanism question the probe closed, which reading had left open: **for `Alias | Form |
Redispatch` the alias door does EXACTLY what `dispatch_rete_op` already does** — re-invoke
`dispatch_keyword_head_value(core_name, args, list_span, env, sym)` with the same unevaluated
args and span. So laziness, scope-opening and arity are not at risk: there is no second
behaviour to diverge. **Only `Fallback` differs** (it strips the `:undefined` marker + fallback
arg and catches the target's raise), which is precisely why `Fallback`'s 20 may never be
registered as aliases — the mechanism, restated in `rete_i64_gt_alias.rs`'s own header.

## ★★★ AND THE PROBE CUT THE STONE IN A PLACE I HAD GUESSED WRONG

I proposed splitting 1b at *registerable vs blocked*. The ratchets split it somewhere better:

```
                       has a CheckEnv TypeScheme?      ledger effect of registering
Alias      29   YES — check.rs registers one from      GAP_A ↓  GAP_B ↓  DEBT unchanged
                      RETE_OPS' own params/ret
Form        6   NO  — no scheme by construction        GAP_B ↓  DEBT ↑
Redispatch  2   NO  — no scheme by construction        GAP_B ↓  DEBT ↑
```

The DEBT ratchet said it in its own words: *"NEW — registered but absent from `CheckEnv` …
`doc_arg_ret_types_match_checker_scheme` is silently skipping these: `[":wat::rete::core::List",
":wat::rete::core::and"]`"* — and named **no** `Alias` row.

★★★ **So 1b-i is the campaign's first stone that moves two ledgers DOWN and none up.** Every
prior registration stone traded GAP_B for DEBT (an invisible absence converted to a named one —
correct, but not the same deliverable). These 28 already have their schemes; registering them is
pure drain. Mixing them with the 8 that pay DEBT would hide that, and would make one number
report two different things — the defect the WORKLIST names.

## The stone

**Register the 28 remaining `OpClass::Alias` rows whose target is a registered row.** (35 minus
the 6 blocked on `:wat::core::=`/`not=`, minus `:wat::rete::i64::>` which Stone 2a already
registered as the witness.)

Each row is a doc-only unit struct carrying `@added`, `@alias`, `@arg`×N, `@ret`, `@example` —
**and none of the five axes**, because an alias inherits them (2a-b) and declaring one is a
`DocError::AliasDeclaresAxis` compile error. `@arg`/`@ret` types transcribe from that row's own
`params`/`ret` in `RETE_OPS`; there is nothing to argue and nothing to decide.

## THE FOUR QUESTIONS

| | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **1b-i as scoped (28 Alias rows)** | YES | YES | YES | YES | ✅ **PICKED** |
| all 37 registerable in one stone | YES | **NO** | **NO** | — | ⛔ |
| all 54 in one stone | **NO** | NO | NO | — | ⛔ |

- **all 37 — Simple NO:** two mechanisms wearing one number. 29 drain three ledgers; 8 trade one
  for another. *"Medium"* is not available here: the ledger ratchets answer flatly and they
  answer differently.
- **all 37 — Honest NO:** the stone's own acceptance rows could not state a DEBT bar without
  splitting the population first, so a single reported figure would describe neither half.
- **all 54 — Obvious NO:** 17 of them cannot be written down at all; the gate refuses them. A
  plan that names work the floor forbids is not a plan.

## Acceptance — every bar DERIVED, none expected

Measured from the ledgers on disk (comment-stripped extraction; the raw one over-counted GAP_A
by 2 and was discarded — the five verified totals reproduce the SEAM's meter exactly):

```
                     before      after      why this exact number
registry rows          487        515       +28 new #[wat_special_form] rows
                                            ⛔ CORRECTED post-stone: this row read "490 → 518"
                                            and BOTH numbers were wrong. My census grepped for
                                            the SUBSTRING `wat_intrinsic("` anywhere, so it
                                            counted three PROSE PLACEHOLDERS as registered
                                            names: `<fqdn>`, `…`, and — the one that defeats a
                                            "starts with `:`" filter — `:wat::holon::…`, an
                                            ellipsis in a module doc comment. Anchoring the
                                            pattern to an attribute SITE (`^\s*#\[wat_…`)
                                            gives 515, which is exactly what the rider measured
                                            live. The SEAM's ground block carried 490 too.
GAP_A                   88         60       all 28 are on GAP_A (each has a scheme, no row)
GAP_B                  106         78       all 28 are on GAP_B
DEBT                    95         95       ⬅ UNCHANGED. Each already resolves in CheckEnv.
                                               A DEBT rise means an Alias row was mis-transcribed
                                               or a non-Alias row was registered.
KNOWN_UNREVIEWED        20         20       an alias inherits; it declares no Totality
the 107                 79                  28 of the 107 are exactly this set
floor              5127/5127   5127/5127    ⛔ CORRECTED: this row read "→ 5155/5155", derived
                                            by adding 28 to the current total. That was a PIN,
                                            not a bar: registering a registry row mints no
                                            `#[test]` fn. The membership gates are single tests
                                            that iterate `registry().all_entries()` internally,
                                            so the count cannot move. The rider caught it.
```

⛔ **DEBT unchanged is this stone's sharpest instrument.** It is the one row that cannot be
satisfied by doing the work sloppily: any row whose `@arg`/`@ret` do not match the scheme
`register_builtins` already holds shows up there, by name.

## Out of scope — affirmatively CUT, not deferred

- **`Fallback`'s 20.** Not an alias. Its `total: true` is the machinery's, not the verb's, and
  aliasing it makes the 4-arg `:undefined` form unreachable. Phase 2b.
- **The 6 Form + 2 Redispatch rows** (`1b-ii`). Registerable today; they pay DEBT, so they get
  their own stone and their own acceptance row.
- **The 17 blocked rows.** Gated on 11 core targets that are themselves GAP_B population.
- **`RETE_OPS` itself.** Untouched. This stone makes the registry ABLE to answer; it does not
  make any consumer ask, and it deletes nothing. That is Phase 3/4.
