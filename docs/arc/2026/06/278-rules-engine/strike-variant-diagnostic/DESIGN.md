# DESIGN-STONE — a misspelled variant must be told it is a misspelled variant

> **Origin (2026-08-31).** D1's residual. D1 (`2733b9bd9`) made the typo REFUSE; this makes the
> refusal name the mistake. Driven here at HEAD `e23659a15`, both sides.

## ⛔ FIRST, A CORRECTION TO MY OWN BREADCRUMB

The work list and my last three stamps record this row as *"a `UnknownEnumVariant` kind so rete's
refusal names the same thing core's does."* Two errors in one sentence:

1. **`UnknownEnumVariant` does not exist.** `grep -rn 'UnknownEnumVariant' src/` → nothing. I named
   a kind I had never grepped — the same mistake A3's stone made with `callee_program`, twice in
   four strikes.
2. **"Agreement with core" is the wrong target, because core does not name it either.** Driven:

| side | the diagnostic for `:evt::G::Hii` where `:evt::G` has variants `Hi`/`Lo` |
|---|---|
| core | `#wat.check/TypeMismatch` — *"`:wat::core::=`: parameter #2 expects `:wat::core::keyword`; got `:evt::G`"*, `:remedies []` |
| rete (post-D1) | `#wat.rete/UnknownField` — *"defrule `evt::good`: `:evt::Req` has no field `:evt::G::Hii`; available fields: [k, grade]"* |

Both refuse. **Neither says what is actually wrong**, which is: *`:evt::G` has no variant `Hii`;
available variants: [Hi, Lo]*. Rete's is the worse of the two, because its remedy list —
`available fields: [k, grade]` — sends the author hunting for a **field** when they mistyped a
**variant**. A confidently wrong remedy costs more than none.

**So the target is not agreement. It is naming the mistake.**

## Why — the fifth catch-all in this arc

`validate/typing.rs:231`:

```rust
fn keyword_constant_segment(k: &str, types: &TypeEnv) -> &'static str {
    match crate::rete::matcher::enum_variant_ctor(types, k) {
        Some((_, _, 0)) => "enum",
        _ => "keyword",          // ← holds TWO facts
    }
}
```

`"keyword"` means both *"this is a genuine keyword constant"* and *"this is a `::`-qualified name
whose prefix is a known enum, and whose variant does not exist."* The second is a diagnosable
mistake being typed as the first. Its own doc says the routing out loud — *"Everything else falls to
`keyword`, where the existing `UnknownField` / `ConstraintTypeMismatch` machinery produces the
located diagnostic"* — which is true about the LOCATION and silent about the message being wrong.

Same shape as A2b's `Option` (two facts), D3's missing arity (three faces), A6's `None => true`, and
A5's `Ok(())` (three states). **Fifth time. Same cure: climb to the type.**

## ★ THE ONE CONTRACT DECISION

**A `::`-qualified constant whose prefix names a known enum is either a variant that exists or a
diagnosable mistake — never a keyword.** The third state gets its own name and its own refusal,
carrying the enum, the variant as written, and the variants that exist — the way `UnknownField`
already carries `available-fields`.

## The algorithm

1. `keyword_constant_segment`'s `_` arm splits: `enum_variant_ctor` → `None`, **and** the `::`-prefix
   resolves to a `TypeDef::Enum` → that is the mistake. Anything else stays `keyword`.
2. A refusal in `validate/error.rs` carrying `enum`, `variant`, `available-variants`. `UnknownField`
   is the shape to copy — same file, same located-diagnostic machinery, same remedy-list habit.

## Blast radius

`src/rete/validate/typing.rs`, `src/rete/validate/error.rs`, and probes. The three D1 fixtures
(`probe_arc278_enum_variant_typo{,_bad,_tagged}.wat`) already exist and are the drive.

## Out of scope — AFFIRMATIVELY CUT

- **Core's `TypeMismatch` and its empty `:remedies []`.** Core has the same blind spot and it is a
  different surface with its own error machinery. Recorded above as driven; **not this strike**, and
  a strike that "fixes both" would be two strikes wearing one scorecard.
- **The tagged-variant arm.** D1 already refuses it (`probe_arc278_enum_variant_typo_tagged.wat`);
  whether ITS message names the mistake is the same question and should be answered by the same
  refusal, but **drive it before assuming the new arm reaches it** — the tagged path resolves
  differently, which is why D1 needed a separate fixture for it.
