# BRIEF — the key-eligibility wall: make "interior-mutable AND hashable" unrepresentable

> **Status: DRAWN, NOT STARTED.** Drawn 2026-07-30, stone D2 of the clippy-to-zero campaign.
> Siblings: A `5b59f061`, B1 `8acbf23a`, B2 `5b4d8d75`, C `38911c4d`, D0 `cea76d71`, D1 `e51480cf`.
> Campaign state: **79 clippy warnings**, of which **18 are `mutable_key_type`** — this stone's subject.
>
> **Home:** arc 109 (kill-std).
>
> **This stone ships a WALL, not a fix.** The 18 warnings are false positives and we have proven it.
> What we have *not* got is anything preventing them from becoming true. That is what this builds.

---

## The finding this rests on (grounded, do not re-derive)

All 18 `mutable_key_type` warnings are one root. clippy names the chain itself:

```
Value → Arc<SenderInner> → SenderInner → AtomicBool → UnsafeCell<u8>
```

One variant — `Value::wat__kernel__Sender`, a channel handle carrying a closed-flag — marks the whole
`Value` enum interior-mutable, tainting every `HashMap<Value, _>` / `HashSet<Value>` in the tree (15
distinct sites, 18 diagnostics).

**The hazard cannot currently occur**, because the interior-mutable variants are exactly the
non-atomizable ones:

- `impl Hash for Value` gives `Sender` / `Receiver` / `fn` an `unreachable!()` arm carrying a
  predicate-citation message (`src/value/value.rs`, the "Non-atomizable variants" block).
- `impl PartialEq for Value` uses `Arc::ptr_eq` for those variants — identity is pointer-based, so it
  is stable regardless of interior mutation.
- `is_atomizable` (`src/check.rs:1487`) statically rejects them from key positions before runtime.

**But the invariant binding those three has no gate.** Arc 216 says so in its own words
(`tests/value/probe_arc216_stone5a_value_hash.rs:334`):

> *"The `unreachable!()` arms are verified by the doc-comment contract: `is_atomizable` at
> `check.rs:3623` never admits these variants… This skip is an honesty delta."*

Verified **by a doc comment**, plus a panic that fires only *after* the mistake has shipped. On the
extirpare ladder that is the bottom rung — a convention — and zero test files reference the `Sender`
variant at all. If someone adds a `Value` variant that is both interior-mutable and atomizable, nothing
stops them.

## Why the skip no longer applies

Arc 216's stated reason for skipping was real: `Function` and `ThreadOwnedCell` have no public
constructor outside wat eval, so you cannot *build* a `Sender` at the test layer to prove its panic.

**This gate never constructs one.** `is_atomizable` takes a `&TypeExpr` — a type-name path — not a
`Value`. So `is_atomizable(&TypeExpr::Path("wat::kernel::Sender".into()))` is answerable with no value
in hand. The classification is a compile-time and type-name-level question throughout. Different
mechanism, not more effort — which is why this succeeds where Probe 10 could not.

## THE ONE CONTRACT DECISION — the bad combination gets no constructor

The wall is the **return type**, not the test:

```rust
/// Whether a `Value` variant may be used as a hash key.
///
/// The wall: there is deliberately NO way to spell "carries interior mutability AND is
/// hashable". That state has no constructor, so it cannot be written down — which is the
/// point of this type existing at all rather than a bare `bool`.
pub enum KeyEligibility {
    /// Pure data. May be a key; `is_atomizable` MUST accept this variant's `type_name()`.
    Hashable,
    /// Never a key. Its `Hash` arm is `unreachable!()` and `is_atomizable` MUST reject it.
    NeverAKey(NotAKeyReason),
}

pub enum NotAKeyReason {
    /// Carries interior mutability (an `AtomicBool`, `UnsafeCell`, `Mutex`, …) reachable
    /// from the value, so a hash taken now may not hold later.
    InteriorMutable,
    /// An opaque handle whose identity is pointer-based (`Arc::ptr_eq`), not structural.
    OpaqueHandle,
    /// Structurally hashable in principle, but deliberately excluded — state the reason.
    ExcludedByDesign,
}
```

A `bool` would let someone write `true` for a `Sender`. This shape means the wrong classification is
**unrepresentable**, and the only way to mark something hashable is to assert it is pure data.

## The forcing function already exists — reuse it, do not build a parallel one

`Value::type_name()` (`src/value/value.rs:1154`) is already an **exhaustive 46-arm match with no
wildcard**, covering `Sender`, `Receiver` and `fn`. A new variant already breaks compilation there.

So this stone adds a **sibling** exhaustive match, `Value::key_eligibility()`, beside it. Same shape,
same discipline: no wildcard arm, ever. A `_ =>` catch-all would silently classify every future variant
and destroy the wall — that is STOP-1.

**Single source of truth.** `type_name()` and `key_eligibility()` must not drift. Prefer declaring both
from ONE list via a `macro_rules!` that emits both matches, so a variant cannot appear in one and be
missing from the other. If that proves to fight the existing `type_name` body more than it is worth,
STOP-2 — report, and do not silently ship two hand-maintained matches that can diverge.

## The gate

```rust
#[test]
fn every_interior_mutable_variant_is_rejected_as_a_key() {
    // For EVERY variant — including the ones that cannot be constructed at this layer —
    // the checker's verdict must agree with the declared eligibility. No values needed:
    // `is_atomizable` answers on a type-name path.
    for (type_name, eligibility) in Value::ALL_KEY_ELIGIBILITY {
        let checker_accepts = is_atomizable(&TypeExpr::Path((*type_name).to_string()));
        match eligibility {
            KeyEligibility::Hashable => assert!(
                checker_accepts,
                "{type_name} is declared Hashable but is_atomizable REJECTS it — a value the \
                 checker will not admit as a key is classified as one"
            ),
            KeyEligibility::NeverAKey(reason) => assert!(
                !checker_accepts,
                "{type_name} is declared NeverAKey({reason:?}) but is_atomizable ACCEPTS it. \
                 If this is InteriorMutable, that is the stranded-key bug clippy's \
                 mutable_key_type exists to prevent, and it is now reachable"
            ),
        }
    }
}
```

**Prove it is a wall, not a gate that happens to pass** (R59 `NISI FRANGAS, NIHIL PROBAS`), and report
both breaks with the exact error:

1. Flip `Sender`'s arm to `Hashable`. The test must go **RED** with the stranded-key message.
2. Add a throwaway variant to `Value` and do NOT classify it. The build must **fail** at
   `key_eligibility()` — a new variant cannot skip the decision. Delete it afterward.

A wall nobody tried to breach is a claim. This campaign already shipped one gate that passed while its
subject was broken, and D0 found three more tests that could not fail.

## Rooms

1. **`src/value/value.rs:1154`** — `type_name()`, the existing exhaustive match. The model, and the
   sibling site.
2. **`src/value/value.rs`** — `impl Hash for Value`, the "Non-atomizable variants" block with the
   `unreachable!()` arms and their predicate-citation messages. This is the classification's ground
   truth for `NeverAKey`.
3. **`src/value/value.rs`** — `impl PartialEq for Value`, the `Arc::ptr_eq` arms. These mark
   `OpaqueHandle`.
4. **`src/check.rs:1487`** — `is_atomizable`, currently **private**. Widening it to `pub(crate)` is
   expected and in scope; the invariant is cross-module and there is no other way to bind the two. Do
   not make it `pub`.
5. **`tests/value/probe_arc216_stone5a_value_hash.rs:328-340`** — arc 216's honest skip note. Once this
   gate lands, that note should point at it instead of at a doc-comment contract. Update it.

## Blast radius

`src/value/value.rs` (the two enums + `key_eligibility` + the table), `src/check.rs` (visibility only),
and one test module. **No behaviour change. No signature changes beyond the new methods. No `#[allow]`.**
Do not touch `clippy.toml` in this stone — the config entry is the FOLLOW-UP that this gate earns, and
shipping them together would let the exemption land un-gated if the gate turned out weaker than drawn.

## STOP triggers — REJECTION criteria. Ship nothing further and report.

1. **STOP-1: a wildcard arm.** If `key_eligibility()` cannot be written exhaustively without a `_ =>`,
   **STOP**. The catch-all destroys the wall — it would classify every future variant silently. Report
   what blocked exhaustiveness.
2. **STOP-2: `type_name` and `key_eligibility` cannot share one source.** If the macro approach fights
   the existing body, STOP and report rather than shipping two hand-maintained matches that can drift.
   Two matches is a convention; one source is the wall.
3. **STOP-3: a variant you cannot classify.** If a variant's interior-mutability or handle-identity is
   genuinely unclear from the `Hash`/`PartialEq` arms, **STOP and report it by name**. Do not guess, and
   do not default it to `Hashable` — a wrong `Hashable` is precisely the bug this stone exists to make
   impossible.
4. **STOP-4: `is_atomizable` disagrees with the existing arms.** If any variant's `Hash` arm is
   `unreachable!()` but `is_atomizable` ACCEPTS its type name — or the reverse — **you have found a live
   defect, not a classification problem.** Stop and report it; that is a finding worth more than this
   stone.
5. **STOP-5: the floor moves for a reason you cannot name.** Floor is `cargo nextest run --release` at
   **4193 passed / 262 skipped**; your gate makes 4194. Any other change is a STOP.

## Gates the rider runs

- `cargo build --release --all-targets` → no new warnings.
- The new gate green, plus **both** deliberate breaches above, each reported with its exact error text.
- `cargo nextest run --release` → **4194 passed**. Read the ANSI-stripped **Summary** line by hand; a
  piped `| tail` returns `tail`'s exit code.
- Clippy unchanged at **79** (this stone clears no warnings — it earns the right to clear 18 later):
  ```
  touch src/value/value.rs
  cargo clippy --release --workspace --all-targets --message-format=json \
    | grep -c '"code":"clippy::mutable_key_type"'
  ```
  Expect **18, unchanged**.

## Expectations

| what | how checked | expected |
|---|---|---|
| every variant classified | `key_eligibility()` compiles with no wildcard | 46 arms, no `_ =>` |
| classification ↔ checker | the new gate | green |
| breach 1 (Sender → Hashable) | manual flip | **RED**, stranded-key message |
| breach 2 (new unclassified variant) | manual add | **build fails** |
| `mutable_key_type` | clippy JSON | **18, unchanged** |
| floor | `nextest --release` | **4194 passed** |
| behaviour | `git diff` | none |

**Runtime prediction:** 45–90 min. Most of it is 46 classification judgements read off the existing
`Hash`/`PartialEq` arms; the macro (STOP-2) is the one genuinely open mechanism.

## What this earns (the follow-up, NOT this stone)

With the wall standing, `clippy.toml` gains `ignore-interior-mutability = ["wat::Value"]` with a comment
citing this gate by name. That exemption then rests on a compile-enforced invariant instead of a doc
comment — which is the whole difference between an earned exemption and a laundered finding, and the
reason the two must not land in one commit.

Then: 8 `ptr_arg`, the ~53 tail, and `-D warnings` in `.github/workflows/ci.yml:41-44`. Zero is a
moment; the wall is what makes it permanent.
