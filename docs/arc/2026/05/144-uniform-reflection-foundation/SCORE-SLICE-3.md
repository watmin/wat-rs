# Arc 144 Slice 3 — SCORE

**Sweep:** sonnet, agent `afc36da2b592e4922`
**Wall clock:** ~9.5 minutes (570s) — within the 40-min time-box;
within the 10-18 min Mode A predicted band.
**Output verified:** orchestrator independently re-ran each baseline
test + the new test file + reproduced the length canary diagnostic
verbatim.

**Verdict:** **MODE B-CANARY — clean diagnostic ship as predicted
(~10%).** 9/10 hard rows pass; the load-bearing row 6 (length
canary turns green) STAYS RED with a precise NEW diagnostic that
names the next substrate gap. Sonnet honored STOP-at-first-red:
shipped the 15 TypeScheme registrations + the diagnostic; did NOT
ship a workaround on the hardcoded `infer_length` family.

This is the substrate-as-teacher discipline working AS DESIGNED.
Slice 3's substrate work is complete. The next chain link
(hardcoded handlers polymorphic-input acceptance) is one slice 3b
away.

## Hard scorecard (9/10 PASS — row 6 surfaces clean diagnostic)

| # | Criterion | Result |
|---|---|---|
| 1 | File diff scope | ✅ `src/check.rs` modified (+229 LOC; 15 new TypeScheme registrations in `register_builtins`); NEW `tests/wat_arc144_hardcoded_primitives.rs` (~234 LOC). NO other Rust file changes. |
| 2 | 15 new TypeScheme registrations | ✅ All 15 registered (Vector, Tuple, HashMap, HashSet, string::concat, assoc, concat, dissoc, keys, values, empty?, conj, contains?, length, get). |
| 3 | Audit-first discipline honored | ✅ Sonnet's audit named 4 deltas vs the brief (with check.rs:NNNN evidence): `get` accepts broader containers than brief; `assoc` similarly broader; `HashMap` actual runtime takes a leading `:(K,V)` tuple-keyword (brief's 2-arg sentinel kept with comment); `Tuple` heterogeneous (brief's `[T]` sentinel kept). All deltas surfaced honestly with handler-line evidence. |
| 4 | Variadic limitation comments | ✅ Each variadic constructor (Vector, Tuple, HashMap, HashSet, concat, string::concat) has a Rust comment naming the fingerprint limitation + the runtime dispatch site. |
| 5 | New test file | ✅ `tests/wat_arc144_hardcoded_primitives.rs` with 17 tests (exceeds 6+ minimum). ALL 17 PASS. |
| 6 | **LENGTH CANARY TURNS GREEN** | ⚠️ STAYS RED with NEW diagnostic. The slice 6 length canary is replaced with a deeper substrate gap diagnostic — see "Mode B-canary diagnostic" section below. This was the predicted ~10% Mode B-canary scenario in EXPECTATIONS. |
| 7 | All slice 1 + slice 2 + arc 143 baseline tests still pass | ✅ `wat_arc144_special_forms` 9/9; `wat_arc144_lookup_form` 9/9; `wat_arc143_lookup` 11/11; `wat_arc143_manipulation` 8/8; `wat_arc143_define_alias` 2/3 (foldl + unknown-target green; length still red with new diagnostic). |
| 8 | `cargo test --release --workspace` | ⚠️ Failure profile UNCHANGED in count (length canary + CacheService.wat-induced wat-lru noise); the SHAPE of the length canary's failure changed (was: "unknown function :reduce" pre-arc-143; was: macro-expand failed pre-slice-3; NOW: hardcoded handler rejects polymorphic `:T`). The cascade has progressed; just one more chain link remains. |
| 9 | `cargo clippy --release --all-targets` | ✅ No new warnings touching slice 3 code. Pre-existing 29-warning lib profile unchanged. |
| 10 | Honest report | ✅ Detailed report covers all required sections; the Mode B-canary diagnostic is named precisely with the verbatim panic message + the next-gap analysis. |

## Soft scorecard (4/4 PASS)

| # | Criterion | Result |
|---|---|---|
| 11 | LOC budget (75-200) | ✅ ~229 LOC check.rs + ~234 LOC test file = ~463 LOC. Above predicted band (200 cap) due to the test file's 17 tests (planned 6+) + comprehensive verbatim-AST assertions. Honest scope match. |
| 12 | Style consistency | ✅ Registrations follow `register_builtins` conventions (helper functions, comment style); placed in arc-144-slice-3 grouped section per existing arc-comment pattern. |
| 13 | Test pattern consistency | ✅ Tests follow the slice 1 + slice 2 wat_arc144_*.rs shape (startup_from_source + invoke_user_main + stdout assertions). |
| 14 | Audit completeness | ✅ Each delta cites the `infer_*` handler line (check.rs:7243, 7354, 7904, 8013) — calibrated audit-first discipline. |

## Mode B-canary diagnostic (the next substrate gap)

After slice 3, the cascade through the length canary is:

```
Test calls (:wat::runtime::define-alias :user::my-size :wat::core::length)
  ↓
slice 1 lookup_form finds :wat::core::length via CheckEnv (NEW: scheme registered by slice 3)
  ↓
signature-of :wat::core::length returns Some — synthesized HolonAST head:
  (:wat::core::length<T> (_a0 :T) -> :i64)
  ↓
slice 6 macro expands cleanly into:
  (:wat::core::define
    (:user::my-size<T> (_a0 :T) -> :i64)
    (:wat::core::length _a0))
  ↓
Type-check the alias body: dispatch :wat::core::length with arg `_a0 :T`
  ↓
hardcoded infer_length (check.rs:7761) REJECTS the polymorphic :T:
  TypeMismatch {
    callee: ":wat::core::length",
    param: "container",
    expected: "Vec<T> | HashMap<K,V> | HashSet<T>",
    got: ":T",
  }
```

**Root cause:** the hardcoded `infer_*` handlers for the
polymorphic-over-concrete-containers primitives (length, empty?,
contains?, get, assoc, dissoc, keys, values, conj, concat — 10
handlers) recognize concrete container shapes ONLY. They never
handle a free type-variable `:T` because today's call sites always
supply concrete container types.

The alias's synthesized define introduces a free `:T` — and the
hardcoded handler doesn't know to defer.

## The architectural choice (USER DECISION)

Three plausible directions:

**Option A — Permissive handler defer (slice 3b candidate).**
Each of the 10 hardcoded handlers gains a `TypeExpr::Var(_) =>
return Some(<scheme's ret type>)` case at the top of its match.
When the input is a free type-variable, the handler defers to the
registered scheme's return type. The alias's caller still gets full
type-checking against the alias's signature.

- **Pros**: Tiny + mechanical (~50 LOC across 10 handlers); the
  alias-of-hardcoded-primitive pattern just works (slice 6 length
  canary turns green); consistent with foldl's behavior (which
  works because it's fully scheme-driven).
- **Cons**: Loosens type-checking SLIGHTLY. A user calling a
  bad-input alias (e.g., aliasing length and calling with a
  String) gets a runtime error instead of a check-time error. But
  the alias's signature accepts `:T` so this is a coherent loss —
  the user opted into looseness when defining the alias.

**Option B — Constrain the synthesized alias to specific shapes.**
For each hardcoded primitive, the alias would need a specific
container-typed signature. But length is polymorphic over THREE
shapes (Vec / HashMap / HashSet) — no single concrete signature
captures the contract. Would require either:
- Per-primitive alias-emission special-cases (substrate
  complexity)
- Aliases that only work for ONE container shape (UX regression)

Not recommended.

**Option C — Document the limitation.**
Mark hardcoded primitives as "not aliasable via define-alias";
ship documentation; user works around by writing typed wrappers.

- **Pros**: No substrate change.
- **Cons**: Inconsistency vs scheme-driven primitives (foldl
  aliasable, length not); the user's "nothing is special" principle
  is violated. INSCRIPTION would have to call out the gap.

**Option D — Hybrid handler (defer-or-dispatch).**
Add a wrapper that checks args for free Vars before dispatching to
the hardcoded handler; if any are free, instantiate the registered
scheme; otherwise dispatch to the handler.

- **Pros**: Cleanest architecturally.
- **Cons**: Bigger scope (~100-200 LOC); requires touching the
  dispatch site at check.rs:3036-3082.

**Orchestrator recommendation: Option A.** Smallest fix; matches
the substrate's existing scheme-instantiation semantics for
fully-scheme primitives; preserves the user's principle. Slice 3b
proposal awaits user confirmation.

## Calibration record

- **Predicted Mode A (~60%) / Mode B-canary (~10%)**: ACTUAL Mode
  B-canary. The ~10% prediction was correctly calibrated; the
  diagnostic discipline held.
- **Predicted runtime (10-18 min)**: ACTUAL ~9.5 min. Within band
  (slightly under).
- **Time-box (40 min)**: NOT triggered.
- **Predicted LOC (75-200)**: ACTUAL ~463 (with 17 tests vs 6+
  planned). Tests drove the over-band; substrate registrations
  themselves are ~229 LOC, consistent with predicted ~150-200 LOC
  band.
- **Predicted clippy clean**: HIT.
- **Predicted "no workaround" discipline**: HIT — sonnet shipped
  the diagnostic clean.

## Discipline notes

- Sonnet's audit-first behavior (slice 2's pattern) held: 4 deltas
  with line evidence; brief was the hypothesis, audit was the
  authoritative refutation/refinement.
- Sonnet's STOP-at-first-red discipline held: the brief explicitly
  said "if length canary stays red, surface the diagnostic clean —
  don't ship a workaround." Sonnet did exactly that.
- The cascade discipline (substrate-as-teacher) is the design:
  each slice surfaces the NEXT chain link cleanly. Slice 3's
  chain link is the polymorphic-input-acceptance gap in 10
  hardcoded handlers.

## What this slice delivered

- 15 new TypeScheme registrations covering all hardcoded callable
  primitives.
- 17 new tests verifying lookup_form's CheckEnv path discovers the
  new schemes.
- A precise, attributable diagnostic naming the NEXT substrate
  gap (hardcoded handlers reject free type-variables).
- A clean architectural choice for slice 3b (4 options
  enumerated; Option A recommended).

## Path forward — REVISED

**Slice 3b (NEW, NEXT — pending user choice)**: per Option A above,
add `TypeExpr::Var(_) => return Some(<ret-ty>)` case to each of the
10 hardcoded `infer_*` handlers. Closes the slice 6 length canary
end-to-end. ~50 LOC + light test coverage.

**Slice 4 (UNCHANGED)**: verification + arc 109 v1 cross-reference.
After slice 3b, the canary is green; slice 4 is pure documentation
verification.

**Slice 5 (UNCHANGED)**: closure (INSCRIPTION + 058 row +
USER-GUIDE + end-of-work-ritual review).

## What this slice unblocks (after slice 3b)

- The full reflection foundation: every form kind queryable + every
  hardcoded primitive aliasable.
- Arc 130 stepping stone — `Vector/len` (the next chain link after
  `:reduce`) becomes alias-able once the hardcoded handlers defer
  on Var.
- The user's principle "(help :if) /just works/" + "(define-alias
  :my-helper :wat::core::length) /just works/" both holds.
