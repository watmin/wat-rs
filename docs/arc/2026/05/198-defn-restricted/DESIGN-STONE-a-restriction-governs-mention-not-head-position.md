# 198 · DESIGN STONE — a restriction governs MENTION, not head position

> **STATUS: DRAWN, NOT BUILT.** Security. Builder ruling 2026-08-15: *"security issues take
> precedence."* Drawn against HEAD `f0fd823f`.
>
> **Filed in arc 198 deliberately** — this arc minted the mechanism and its `SCORE-STONE-4-LOOP-CLOSURE.md`
> declared it closed. The stone belongs next to the score it refutes. Whether this deserves its own arc
> number is the builder's call, not the apparatus's.

## ⛔ THE FINDING — every `:restricted-to` in the substrate is bypassable in one line

Measured live this session with `./target/release/wat`, not read:

```clojure
;; REFUSED — the head-position call, as designed
(:wat::core::defn :user::sneaky [] -> :wat::core::String
  (:wat::kernel::str-double "x" 2))
;; => #wat.check/DefRestrictedCallerNotAllowed  ✅

;; ACCEPTED — the same function, named one line earlier
(:wat::core::defn :user::sneaky [] -> :wat::core::String
  (:wat::core::let [f :wat::kernel::str-double]
    (f "AB" 3)))
;; => --check EXIT=0, no error.   run EXIT=0.   THE KERNEL FN EXECUTED FROM :user::
```

`str-double` is gated `{:restricted-to [:wat::kernel:: :wat::test::]}`. The alias form type-checks and
**runs**. The same route was confirmed (check-only, never executed) against
**`:wat::kernel::write-fd-raw`** — the arbitrary-fd danger seal — which resolves and reports
`ArityMismatch`/`TypeMismatch` (so it is fully resolved as a call) with **no restriction error**.

Every restriction in the substrate is reachable this way:

| binding | what it guards | status |
|---|---|---|
| `:wat::kernel::write-fd-raw` | arbitrary-fd unbounded raw write | **bypassable** |
| `:wat::kernel::flood-stdout-raw` | fixed-fd flood | **bypassable** |
| `:wat::kernel::str-double` | 2ⁿ amplification helper | **bypassable, proven executing** |
| `wat/spawn.wat:329` | the IPC wall (task #13, a whole stone) | **bypassable** |
| every per-field accessor whitelist | arc 203 capability fields | **bypassable** |

## THE MECHANISM — one `if`

`walk_for_restricted_call` (`src/check.rs:1403`):

```rust
if let WatAST::List(items, _) = node {
    if let Some(WatAST::Keyword(head, head_span)) = items.first() {
        if let Some(meta) = env.get_binding_metadata(head) { … }
    }
}
for child in node.children().iter() { walk_for_restricted_call(child, …) }
```

It checks **only the first element of a List**. The walk *does* recurse into every child — but a
restricted FQDN sitting in any non-head position is a bare `WatAST::Keyword`, not a `List`, so the
outer `if let` fails and the node is passed over in silence.

**The check is syntactic where the property is semantic.**

## THREE INSTANCES OF ONE ROOT — do not treat them as three bugs

1. **Value-position mention** (above). The general escape. Applies to every restricted binding.
2. **The constructor trampoline.** `defstruct`'s synthesized companion macro
   (`src/macros/parse.rs:329`) rewrites `(:my::Token :id 7)` into
   `(:wat::core::kwargs-construct :my::Token :id 7)`. The head becomes a substrate builtin that carries
   no whitelist; the real callee sits at `items[1]` where the walker never looks. Dark since
   `310aa793` (2026-06-28).
3. **A written safety claim that was never attacked.** `wat/kernel/services/stdio.wat:358` argues:
   > *"The gate is SAFE: the reserved-prefix gate forbids `:user::` code from authoring a `:wat::`
   > caller, so no user program can construct a passing call site."*

   The premise is TRUE — the reserved-prefix gate is real and holds. The conclusion does not follow,
   because **you never need a passing call site.** The argument reasons about the *authoring* surface
   and is silent about the *reference* surface. It is the reason the funnel it protects
   (`write-fd-raw` → `flood-stdout-raw` → `flood-own-stdout`, each deliberately narrower) has been open
   at the top the whole time.

## ⛔ WHAT IS **NOT** BROKEN — do not "fix" these

Measured, and each was something the apparatus asserted wrongly earlier in this session before
checking:

- **Registration is CORRECT.** `src/runtime.rs:1453` writes the ctor whitelist under
  `struct_def.name` — the type name, the right key — and `git log -S` shows that line was born in
  `310aa793`, the same commit that annihilated `/new`. **Nothing needs re-registering.** An earlier
  claim in this session that "the metadata's carrier vanished" was false.
- **The reserved-prefix gate is real.** `:user::` code genuinely cannot author a `:wat::` FQDN.
- **Head-position enforcement works.** Direct calls to restricted bindings are refused correctly, and
  the per-field accessor path fires exactly as designed when called as a head.
- **Nothing propagates into stdlib.** `binding_metadata` is a per-program map on the `SymbolTable`,
  built at startup from all forms; the Rust checker reads it. No stdlib code is mutated and no
  whitelist is baked into any builtin.

## THE RULE — derived from what a whitelist IS

Builder, this session:

> *"the purpose of the whitelist is to restrict who can call the thing being defined"*

To call a thing you must first **name** it. Therefore:

> ### A restricted FQDN may not be NAMED by a function outside its whitelist — in any position.

Not "called as a head." **Named.** This is derived from the meaning of the declaration, not enumerated
from a list of forms, and it subsumes without naming them: head calls, `kwargs-construct` arg 1,
`aggregate-new`, `let` aliases, arguments to higher-order functions, collection literals, map values,
and every trampoline nobody has invented yet.

## ⛔ WHY NOT "ALSO CHECK `kwargs-construct` AND `aggregate-new`"

That was the apparatus's first proposal and the builder cut it: *"this is a hack... we need it to be
general... we don't know that this only applies to kwargs-construct and aggregate-new."*

He was right, and the memory already existed:
`[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`. A two-name special case is the **convention**
rung of the extirpare ladder wearing a wall's clothes — it patches two stems of a plant whose root is
elsewhere, and the value-position escape (which involves neither name) proves the root is elsewhere.
**The special case IS the bug.**

## THE FOUR QUESTIONS

- **Obvious?** YES — "a restricted name may not appear outside its whitelist" is one sentence with no
  exceptions to memorize. The current rule requires knowing which syntactic position the checker
  happens to inspect.
- **Simple?** YES — and it makes the code *smaller*: the `List`/`first()` special-casing is deleted,
  not extended. One predicate over every keyword node, no form registry.
- **Honest?** YES — today the substrate declares a capability, accepts the declaration, documents why
  it is safe, and does not enforce it. This closes the gap between what a `:restricted-to` says and
  what it does.
- **Good UX?** YES — the diagnostic already exists and is excellent (`DefRestrictedCallerNotAllowed`
  names the callee, the enclosing fn, the whitelist, and both matching rules). It will simply fire
  where it should have all along.

## THE IMPLEMENTATION SHAPE

In `walk_for_restricted_call`: drop the `WatAST::List` + `items.first()` guard and check **every**
`WatAST::Keyword` node the walk encounters against `env.get_binding_metadata`. Same registry, same
error variant, same span source, no new storage, nothing added to any builtin.

## ⛔ OPEN QUESTIONS THE RIDER MUST SETTLE BY MEASUREMENT — NOT ASSUME

1. **The declaration site names its own FQDN.** `(:wat::core::defn :wat::kernel::str-double {…} …)`
   mentions the restricted name. Needs a principled exemption at the *binding site* — and note the
   asymmetry: a fn whose whitelist excludes its own namespace would otherwise trip on its own
   declaration. Derive the exemption; do not hardcode a position.
2. **Type-annotation and metadata positions** also carry keywords. Only FQDNs that *have* a
   `:restricted-to` entry are ever checked, so the surface should be small — **measure it, do not
   reason about it.**
3. **Self- and mutual recursion.** `str-double` calls itself; `flood-stdout-raw` calls `write-fd-raw`.
   Both are inside whitelisted namespaces today and should pass — confirm they still do.
4. **What fires across `wat/`.** Impose the check and read the screams; do not survey first for a
   worklist (`[[feedback_impose_the_check_and_read_the_screams]]`). Every scream is either a real
   escape or a needed exemption, and both are findings.

## THE WALL — the extirpare rung, so the class cannot regrow

Two gates, both derived from the registry rather than from a hand-list:

- **W1 — an unenforceable restriction fails at startup.** Every key registered in
  `binding_metadata` with a `:restricted-to` entry must be reachable by the enforcement path. A
  declaration that nothing can consult is an error at registration, not a silent no-op. Armed, this
  goes red on 2026-06-28 the instant the constructor moves behind a trampoline, instead of going quiet
  for 48 days.
- **W2 — every written safety claim gets an attacking probe.** `stdio.wat:358` reasoned about the
  authoring surface and never tested the reference surface. **Sweep for other claims of the form
  "X is safe because Y cannot be authored" and probe each one adversarially.** If that reasoning
  pattern was used once it was probably used elsewhere; a safety claim in a comment is a hypothesis
  until something attacks it.

## STOP TRIGGERS — rejections. Report and leave the site.

- **STOP-1 — the general rule cannot be made to work and a form list looks necessary.** That is a
  finding about the AST, not a licence. Report the exact shape that defeats it. Do not ship a
  two-name special case.
- **STOP-2 — more than a handful of legitimate sites need exemptions.** A rule needing many exemptions
  is the wrong rule. Stop and report the list; the orchestrator re-draws.
- **STOP-3 — a `src/` change outside the walker + its registration wall looks necessary.** Registration
  is correct (see above). If it looks wrong, re-measure before touching it.
- **STOP-4 — you are tempted to weaken a restriction, or to widen a whitelist, to make something
  pass.** Never. A screaming site is the finding.

## ⛔ WHAT IS UNKNOWN AND MUST BE MEASURED — do not inherit a number from this document

- **WHEN the value-position hole opened is NOT KNOWN.** The constructor case is dated `310aa793`
  (2026-06-28) by `git log -S`. The value-position escape is **undated** — it may have been open since
  arc 198 slice 1 (2026-05-17) or since the walker's arc-212 rewrite. **Bisect it.** The apparatus was
  already wrong about a date once today and corrected itself; do not let "48 days" travel to the
  reference-surface hole, where it has not been established.
- **The full inventory of restricted FQDNs is not established.** Census: 7 `:restricted-to` in `wat/`,
  32 in `tests/` fixtures, 63 in `src/` (Rust-side `#[restricted_to]` + the inventory channel) —
  counted by string occurrence, which is not the same as counting *bindings*. Re-count things, not
  lines (`[[feedback_a_file_count_is_not_an_item_count]]`).
- **Whether other trampolines exist** beyond the kwargs companion macro. The general rule makes the
  question moot for enforcement, but the answer belongs in the record.

## BLAST RADIUS

`src/check.rs` (`walk_for_restricted_call` and its doc), the startup wall's registration site, new
probes, and whatever the imposed check screams about across `wat/` and `tests/`. **Expect the corpus
to scream — that fire is the worklist, not a crisis** (`examinare`: the fail-count is the progress
meter).

## HOW THIS WAS FOUND

Wave B batch 1 of the 296 recapture cascade un-ignored 33 tests and fired STOP-2 with 11 findings.
One of them — `struct_restricted_ctor_restriction_fires_on_illegal_caller`, *"expected startup
failure; got Ok"* — had been muted since 2026-07-02 under the blanket reason
*"296-recapture-pending: golden asserts pre-stone-B rust-debug face"*, **which was never true for it**:
it fails before any golden is compared.

The campaign's law is what surfaced it — *only the expected-staleness class gets recaptured; anything
else is a finding.* A blanket `UPDATE_EDN=1` would have captured `Ok` and painted it green forever.

**That is the campaign's stated thesis, paid in full on its first real wave:** *a regression found here
is worth more than a hundred tests turning green.*

---

# ⚖ RULING — A1 + B2 (builder, 2026-08-15): *"B2 and A1 - they have been reasoned"*

## What the first strike measured

The walker change landed (mention rule, `List`/`first()` guard deleted) and the floor was run:

```
Summary [ 193.457s] 4531 tests run: 4528 passed (2 slow), 3 failed, 154 skipped
```

**Three tests, one family, zero corpus impact.** An in-flight report characterised this as *"breaks
arc-203 restricted-struct construction entirely… the runtime's own ctor and predicate companions live
in the bare type namespace"* — **both halves are wrong and the correction matters:**

- **Scale:** 3 tests, not "entirely". Consistent with the measured fact that **no `defstruct`/
  `defrecord` outside `tests/` declares a ctor `:restricted-to`.**
- **"Bare type namespace" is not a thing.** The two companions are `:my::Token'` and
  `:my::is-Token?` — both in `:my::`, the type's **own** namespace.

All three failures are identical: exactly two errors, from exactly two sites.

| enclosing fn | mint site | names |
|---|---|---|
| `:my::Token'` (positional prime ctor) | `src/runtime.rs:1557` | `:my::Token` |
| `:my::is-Token?` (membership predicate, arc 237.6) | `src/runtime.rs:2006` | `:my::Token` |

**The accessors are INNOCENT — measured, not assumed.** `:my::Token/id`'s body uses
`Record/field-at` plus a *string* `class_no_colon`; it never names the type as a keyword. The
companion set that trips is **two**.

`contract_03_defstruct_with_field_metadata` declares `:restricted-to []` — the empty whitelist that by
design matches nothing — and trips too. **The propagation must be unconditional**, not "when the
whitelist is non-empty".

## The two decisions, four-questioned

### Direction (a) — do companions inherit T's restriction?

| | option | Obvious | Simple | Honest | UX |
|---|---|---|---|---|---|
| **A1 ✅** | `T'` and `is-T?` inherit T's whitelist | YES | YES | YES | YES¹ |
| A2 | leave `T'` unguarded (status quo) | YES | YES | **NO** | — |

A2 fails Honest outright: the type declares itself restricted while a public alias constructs it
freely — and `runtime.rs:15835` **advertises that alias** in its own remedy text.

¹ A1's UX holds only if the remedy message ships with it. *"or use the positional prime `:ns::P'`"*
becomes a wall the user walks into. **Fixing the message is part of A1, not a follow-up.**

### Direction (b) — how may a companion name its own restricted type?

| | option | Obvious | Simple | Honest | UX |
|---|---|---|---|---|---|
| B1 | append companion FQDNs into T's `:restricted-to` list at mint | **NO** | YES | **NO** | — |
| **B2 ✅** | record `synthesized_for: Some(T)` on the `Function`; walker exempts owner-type mentions | YES | YES | YES | YES |
| B3 | exempt by name pattern at check time (`ends_with("'")`, `is-…?`, `T/…`) | **NO** | **NO** | **NO** | — |
| B4 | don't walk synthesized bodies at all | YES | YES | **NO** | — |
| B5 | make the body not *name* the type; carry it structurally | **NO** | **NO** | YES | — |

- **B1 fails Obvious + Honest.** `DefRestrictedCallerNotAllowed` **prints the whitelist back to the
  user**. B1 would quote entries they never wrote and attribute them to their binding site.
- **B3 fails all three, and one failure is a FORGERY.** It infers provenance from spelling: a
  user-authored fn named `:my::Token'` would inherit the exemption — a capability earned by choosing a
  name. Same class as 251.8a-ii's `$bound/x`, and the recurring string-comparison failure this repo
  has a standing note about.
- **B4 IS THE TRAP.** It reads simplest of all — one flag, one early return — and fails the only axis
  that matters: it exempts generated code from **every** restriction, not just its own type's. A
  future companion naming `:wat::kernel::write-fd-raw` would pass unchecked. A blanket hole where a
  narrow exemption is needed, and it would never look wrong in review.
- **B5 is the deepest rung and correctly loses.** Extirpare's *never construct the situation that needs
  the patch* says remove the mention rather than authorize it. But a constructor naming the type it
  constructs is **correct and legible** — stripping it makes a dumped AST stop saying what it builds
  and forces `aggregate-new` to source its type from a new side channel. Recorded because it is the
  option that would otherwise be missed.

**B2 is the only clean sweep**: one sentence a reader can hold — *a generated companion may name the
type it was generated for* — provenance stated as fact rather than inferred, the declared list quoted
back untouched.

## The wall arrives free

A third companion minted later without propagation will **name its own restricted type from a
non-whitelisted FQDN**, trip the mention rule, and fail startup loudly on the first restricted type in
the corpus. **The mention rule is its own drift detector.** Nobody can add a companion and quietly
forget — which is precisely how this class was born.
