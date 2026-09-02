# DESIGN — STONE: the seven that need no extraction

> Next strike after `[[DESIGN-STONE-the-membership-gap-gets-a-ratchet]]` (`10021fd35`).
> The ratchet exists and has moved once; this is the first strike it *measures* rather than
> demonstrates.

## The pick is derived from the ratchet, not chosen

Gap A's 94 names, split by whether an eval handler already exists **with a name to annotate**:

```
24   named eval arm       — registering is ANNOTATION, no extraction        ⬅ the strike lives here
11   inline eval arm      — needs extraction first (the megafile shape)
59   no eval arm at all   — mostly :wat::rete::* aliases dispatched via RETE_OPS, a different mechanism
```

Of the 24, this stone takes the **seven whose axes are groundable in one sentence each** — pure
conversions and predicates. All seven verified individually against their dispatch arms:

| verb | eval arm | check side |
|---|---|---|
| `:wat::core::bool::to-string` | `eval_bool_to_string` | `register_builtins` scheme |
| `:wat::core::i64/to-f64` | `crate::numeric::convert::eval_i64_to_f64` | scheme |
| `:wat::core::i64/to-string` | `crate::numeric::convert::eval_i64_to_string` | scheme |
| `:wat::core::u8` | `crate::numeric::convert::eval_u8_cast` | scheme |
| `:wat::core::record?` | `crate::record::access::eval_record_q` | scheme |
| `:wat::core::not` | `eval_not` | scheme |
| `:wat::core::show` | `eval_show` | scheme |

⚠ **`subtype?` and `conforms?` were candidates and are CUT.** Both carry an **inline** check arm
(not a scheme) and both sit in `KNOWN_UNREVIEWED` — a different shape needing its own ruling. Taking
them because they were adjacent is how a batch turns into two half-finished ones.

★ Four of the seven are already homed by this session's own megafile work
(`src/numeric/convert.rs` ×3, `src/record/access.rs`) — the decomposition made the registration
cheap, which is the two campaigns paying each other back.

## THE ONE CONTRACT DECISION — pinned

**A registration is only cheap when the handler already has a name. Extraction is a different
stone and does not ride along.**

The 11 inline-arm names (`Vector`, `HashMap`, `HashSet`, `Tuple`, `filter`, `foldl`, …) are visibly
next and deliberately excluded. A stone that registers seven and extracts two has two failure modes
and one commit message.

## ★★ Expect THREE ledgers to move, and one of them belongs to someone else

The previous stone's red was `every_dispatched_verb_is_classified_or_disposed` naming `fn`/`match` as
*"no longer unreviewed — DELETE their lines."* Registering a verb gives it declared axes, so
`intrinsic_meta` classifies it from the registry and it leaves `KNOWN_UNREVIEWED`.

**`:wat::core::show` is on `KNOWN_UNREVIEWED` today.** So this stone should trip that gate again, by
name. That is the ratchet working — but it means the stone's acceptance must **predict** it rather
than discover it:

```
REGISTRY_MEMBERSHIP_GAP_A   94 -> 87
REGISTRY_MEMBERSHIP_GAP_B  119 -> 118      (only bool::to-string is in the 121 corpus census)
KNOWN_UNREVIEWED            -1             (show), named by the completeness gate
```

⚠ **A prediction that fires is worth more than one that holds.** If the completeness gate does NOT
name `show`, the model of how registration interacts with `intrinsic_meta` is wrong, and that is a
finding.

## ⛔ The axis that must not be guessed — the `fn` lesson

Last stone, the honest-looking answer was the dangerous one: marking `fn`'s `@ExpandTime` as
`Unreviewed` would have **silently made `(fn ...)` illegal inside macro bodies**, because the
registry-first check in `macros/eval.rs` now intercepts before its hand-list residue is reached.

**So for every verb here, `@ExpandTime` must be checked against `is_expand_time_legal`'s residue
list before it is written.** A name on that list today is legal today; declaring `Unreviewed` for it
is a behaviour change wearing humility's clothes.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **the seven with named arms + groundable axes** | YES | YES | YES | YES | ✅ **ADMITTED** |
| all 24 named-arm names in one stone | YES | **NO** | YES | — | ⛔ DISQUALIFIED |
| the seven + extract the 11 inline arms | YES | **NO** | YES | — | ⛔ DISQUALIFIED |
| include `subtype?`/`conforms?` | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| the `:wat::eval-*!` family (10, also named arms) | YES | YES | **NO** | — | ⛔ DISQUALIFIED |

- **all-24 Simple? NO** — 24 grounded directive blocks in one diff; a wrong axis could not be
  attributed, and one wrong `@ExpandTime` is a silent behaviour change.
- **plus-extraction Simple? NO** — two stone shapes, one commit.
- **subtype?/conforms? Honest? NO** — inline check arms and `KNOWN_UNREVIEWED` purity; registering
  them means ruling them, which this stone has not measured.
- **eval-\* family Honest? NO** — `KNOWN_UNREVIEWED`'s own note says their purity *"depends on what
  they are handed, exactly like `apply`"*. Ten verbs whose central axis is a known open question is
  not a cheap batch; it is a ruling wearing a batch's clothes.

## Acceptance — rows chosen to be unfakeable

| what | command | expected |
|---|---|---|
| the seven are rows | `registry().lookup_entry` each | `Some`, `Kind::Intrinsic` |
| Gap A moved | `REGISTRY_MEMBERSHIP_GAP_A` | 94 → **87**, and its gate green |
| Gap B moved | `REGISTRY_MEMBERSHIP_GAP_B` | 119 → **118** |
| ⛔ the predicted third ledger | `every_dispatched_verb_is_classified_or_disposed` | names **`:wat::core::show`**, then its line is deleted |
| ⛔ no `@ExpandTime` regression | each verb vs `is_expand_time_legal`'s residue | a residue name declares `Legal`/`Preserving`, never `Unreviewed` |
| ⛔ the hole stays open | `wat --check` on `(:wat::holon::Bogus 1 2)` | still ACCEPTED |
| ⛔ the blanket-accept untouched | `grep -c "if is_reserved_prefix(head)" src/resolve/walk.rs` | **1** |
| bodies verbatim | the seven handlers | unchanged; attributes only |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5118+/5118+, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
