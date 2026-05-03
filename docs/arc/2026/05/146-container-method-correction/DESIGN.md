# Arc 146 — Container-method correction (Type/verb shape per arc 109 convention)

**Status:** drafted 2026-05-03 (mid-arc-144-slice-3 closure, after
the realization that arc 144's slice 3b would have bridged a
substrate inconsistency that should be REMOVED, not bridged).

## The discovery this arc captures

Arc 144 slice 3 registered TypeScheme fingerprints for 15
hardcoded callable primitives. The slice 6 length canary stayed
red with a precise NEW diagnostic:
```
TypeMismatch on :wat::core::length
  expected: "Vec<T> | HashMap<K,V> | HashSet<T>"
  got:      ":T"
```

Surface reading: the type-checker rejects polymorphic input.
Bridge fix: defer-on-Var (slice 3b options A-D).

**Deeper reading** (surfaced by user pressure during slice 3
score, after THREE wrong framings on my part — see arc 144's
REALIZATIONS Realization 6 for the discipline lesson):

The substrate has two parts that disagree on polymorphic input —
scheme-based primitives (foldl, map) and handler-based primitives
(length, get, conj, etc.).

**The handlers do NOT exist because the substrate is missing
features.** The substrate's type system is INTENTIONALLY rank-1
parametric polymorphism — that's the design. The handlers exist
because 10 primitives are NAMED in a way that VIOLATES the
substrate's Type/verb convention. `length` claims to be ONE
polymorphic operation across three container types — that breaks
rank-1 (rank-1 can't express "across three container types as one
name"). Someone compensated with a handler. The handler is a
manifestation of the naming violation, not a workaround for a
missing feature.

The user's framing reframed the question: "if a thing says it of
a generic kind then it is" — the primitive's DECLARED contract
must match the substrate's actual capability. If a primitive
claims to be one polymorphic operation but is actually N different
operations dispatched by container type, the **primitive is
poorly defined**.

**`length` is poorly defined.** It claims a unified surface
across three containers, but the substrate's design constraint
(rank-1 parametric polymorphism — one scheme per name) doesn't
hold one-name-across-three-shapes. Someone compensated with a
handler. The handler is the real contract; the scheme is a lie.

Same shape applies to 9 other primitives.

**The correction: redefine each as Type/verb primitives**, one
per container type. Each becomes a clean rank-1 scheme:

- `Vector/length : forall T. Vec<T> -> :i64`
- `HashMap/length : forall K,V. HashMap<K,V> -> :i64`
- `HashSet/length : forall T. HashSet<T> -> :i64`

Every container-method primitive after correction fits the
substrate's existing rank-1 model cleanly. No new substrate
features needed. The handlers retire because they have nothing
to do. The substrate's design coherence is restored.

This arc is the EDUCATION VEHICLE for the discovery. The arc record
exists so future-us remembers the WHY of the redefinition, not
just the WHAT.

## What gets corrected

```
:wat::core::length         →  :wat::core::Vector/length
                              :wat::core::HashMap/length
                              :wat::core::HashSet/length

:wat::core::empty?         →  :wat::core::Vector/empty?
                              :wat::core::HashMap/empty?
                              :wat::core::HashSet/empty?

:wat::core::contains?      →  :wat::core::Vector/contains?    (T)
                              :wat::core::HashMap/contains-key?  (K — different verb)
                              :wat::core::HashSet/contains?   (T)

:wat::core::get            →  :wat::core::Vector/get          (i64 -> Option<T>)
                              :wat::core::HashMap/get         (K -> Option<V>)
                              ;; HashSet's get-by-equality is just contains?

:wat::core::assoc          →  :wat::core::HashMap/assoc       (K -> V -> HashMap<K,V>)
                              :wat::core::Vector/set          (i64 -> T -> Vec<T>)
                              ;; verbs differ — assoc is K-V; set is index-value

:wat::core::dissoc         →  :wat::core::HashMap/dissoc      (K -> HashMap<K,V>)
                              ;; HashMap-only; the polymorphism was always fake

:wat::core::keys           →  :wat::core::HashMap/keys        (HashMap-only)
:wat::core::values         →  :wat::core::HashMap/values      (HashMap-only)

:wat::core::conj           →  :wat::core::Vector/conj         (T -> Vec<T>)
                              :wat::core::HashSet/conj        (T -> HashSet<T>)
                              ;; HashMap doesn't conj; it asocs

:wat::core::concat         →  :wat::core::Vector/concat       (Vec<T> -> Vec<T> -> Vec<T>)
                              ;; String::concat already exists per arc 109 namespacing

:wat::core::string::concat →  ALREADY CORRECT (String:: namespace per arc 109)
```

After the correction:
- 10 ad-hoc-polymorphic primitives become ~20 clean monomorphic
  Type/verb primitives.
- Each new primitive has a proper TypeScheme registration.
- The 10 hardcoded `infer_*` handlers in `check.rs` retire.
- `lookup_form` finds every primitive via the scheme path
  uniformly.
- Aliases work for every primitive (no handler-vs-scheme
  collision because there's only one type-system model left).

## Why this is consistent with arc 109's wind-down direction

Arc 109's slice 1j shipped the Type/verb convention for the
Option/Result method forms:
- `:wat::core::Option/expect`, `Option/try`
- `:wat::core::Result/expect`, `Result/try`

Arc 109 slice K.kernel-channel renamed Queue* → Channel/Sender/Receiver.
Arc 109 slice K.console flattened service Console::* → :wat::console::*.
Arc 109 slice K.lru, K.holon-lru, K.thread-process — ongoing.

This arc continues the same shape: container-method primitives
adopt Type/verb naming, matching the rest of the substrate's
post-arc-109 surface.

## Slice plan (sketch)

The detailed slicing emerges as the arc starts. Initial sketch:

### Slice 1 — Mint the new Type/verb primitives

For each container × verb combination, register a TypeScheme in
`register_builtins` AND add a runtime dispatch arm. Each is
monomorphic; no handler needed.

Group by container for simpler review:
- **Vector**: length, empty?, contains?, get, set, conj, concat
- **HashMap**: length, empty?, contains-key?, get, assoc, dissoc,
  keys, values
- **HashSet**: length, empty?, contains?, conj

~150-300 LOC across check.rs (schemes) + runtime.rs (dispatch).
Each new primitive's runtime impl delegates to the existing
hardcoded handler internally — ZERO behavior change at runtime;
just naming + scheme cleanup.

### Slice 2 — Add deprecation poison to the old names

Each retired primitive (`:wat::core::length`, etc.) gets Pattern
2 poison per arc 110's discipline:
- Synthetic `TypeMismatch` flagging "use :Vector/length"
- Continue to dispatch (no cliff for unswept call sites)

So users get a clear migration message at every call site that
hasn't been swept.

### Slice 3 — Sweep substrate stdlib + lab call sites

Mechanical find-and-replace per primitive. Each call site picks
the right Type/verb based on the container's static type.

Substrate stdlib (`wat/`, `crates/wat-*/wat/`) + lab
(`holon-lab-trading/wat-tests/`, etc.) — substantial sweep but
each change is local + obvious.

### Slice 4 — Retire the 10 hardcoded `infer_*` handlers

Once the deprecated names have no callers (verified by sweep
completion), delete:
- `infer_length`, `infer_empty_q`, `infer_contains_q`,
  `infer_get`, `infer_assoc`, `infer_dissoc`, `infer_keys`,
  `infer_values`, `infer_conj`, `infer_concat` (10 handlers)
- The dispatch arms in `infer_list` for these
- The Pattern 2 poison registrations from slice 2

The substrate is now scheme-only.

### Slice 5 — Verify arc 144's reflection foundation works end-to-end

- `lookup_form` finds every Type/verb primitive via CheckEnv path
- Slice 6 length canary (arc 143's): re-orient to alias
  `:wat::core::Vector/length` instead of `:wat::core::length` —
  turns green via the scheme path
- Arc 144 slice 4 (originally pending): becomes a final-verification
  slice; the wrapper / defer-on-Var work that slice 3b would have
  done is unnecessary

### Slice 6 — Closure

INSCRIPTION (this arc) + 058 row + USER-GUIDE entry +
end-of-work-ritual review. Arc 144's INSCRIPTION updates to
reference this arc as the gating closure.

## Open questions

### Q1 — Verb consistency across containers

For some operations, the natural verb differs per container:
- `assoc` is K-V (HashMap) vs index-value (Vec); should Vec use
  `set` instead?
- `contains?` is element-membership (Vector / HashSet) vs key-
  membership (HashMap); should HashMap use `contains-key?`?

Per arc 109's Type/verb convention, each Type's verbs are NAMED
FOR THAT TYPE — different verbs are honest if the operations
actually differ semantically.

Slice 1's BRIEF audits each per-Type verb for semantic clarity.

### Q2 — Should `conj` exist for HashMap?

HashMap doesn't have a "push" operation; `assoc` is the analog.
`Vector/conj` and `HashSet/conj` are clean; `HashMap/conj`
shouldn't exist.

Slice 1: `:wat::core::HashMap/conj` is NOT registered.

### Q3 — Sweep ordering

The lab uses these primitives heavily. Slice 3's sweep is
substantial.

Sequencing:
- Substrate stdlib first (smaller scope; verifies the new names work)
- Lab next (driven off the verified substrate)

Or by container:
- Vector first (most pervasive)
- HashMap second
- HashSet third

Defer to slice 3 brief.

### Q4 — Retire timeline for the deprecated names

Per arc 110's discipline + the no-cliff sweep model, deprecation
poison stays until the workspace is provably swept.

After slice 3 ships, the deprecated names should have ZERO callers
in the substrate + lab (sonnet's grep verifies). Slice 4 retires
them.

If new callers appear post-arc-146 (e.g., from external lab
contributions), the poison stays as a teaching diagnostic until
swept.

## Cross-references

- `docs/arc/2026/05/144-uniform-reflection-foundation/REALIZATIONS.md`
  — the discovery this arc captures
- `docs/arc/2026/05/144-uniform-reflection-foundation/SCORE-SLICE-3.md`
  — the slice that surfaced the diagnostic
- `docs/arc/2026/04/109-fqdn-substrate-symbols/...` (if exists) —
  the arc this corrects within
- `docs/arc/2026/04/108-typed-expect-special-forms/INSCRIPTION.md`
  — the Type/verb precedent (Option/expect, Result/try, etc.)
- `docs/arc/2026/04/110-kernel-comm-expect/INSCRIPTION.md` — the
  Pattern 2 poison + deprecation discipline this arc uses
- `src/check.rs:7243+`, `7354+`, `7460+`, `7523+`, `7575+`,
  `7629+`, `7682+`, `7761+`, `7815+`, `8054+`, `8096+` — the 10
  handlers being retired
- `src/check.rs:3036-3082` — the dispatch arms being deleted

## Why this arc must land before arc 109 v1 closes

Arc 109's wind-down rule: arc 109 v1 doesn't close until all
post-109 arcs implement (no deferrals). The container-methods are
the last cluster of substrate primitives that DON'T follow the
Type/verb convention shipped throughout arc 109's slices 1j +
K.*.

Closing arc 109 with these primitives in their poorly-defined
shape would lock in the inconsistency.

## Status notes

- DESIGN drafted as the education vehicle.
- Implementation deferred until arc 144 ships through slice 4
  (verification).
- Arc 144 slice 3b CANCELLED — superseded by this arc.
- Arc 144 slice 4's verification step now incorporates arc 146's
  redefinitions (length canary turns green via Vector/length).
- Arc 109 v1 closure now blocks on arc 144 + arc 130 + arc 145 +
  arc 146.
