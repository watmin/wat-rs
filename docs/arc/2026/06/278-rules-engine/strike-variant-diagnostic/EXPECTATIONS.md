# EXPECTATIONS — a confidently wrong remedy costs more than none

> Written **before** the strike. **Every row's command was run against HEAD and its pre-value is
> recorded.**

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,216 plus every arm you drive. Exceeding it is a PASS.**

## The scorecard, with pre-values measured at HEAD `e23659a15`

| # | what | pre-value AT HEAD | expected after |
|---|---|---|---|
| 1 | the misspelling refuses | **`#wat.rete/UnknownField` — "`:evt::Req` has no field `:evt::G::Hii`; available fields: [k, grade]"** (driven) | refuses naming **`:evt::G` has no variant `Hii`**, with the real variants |
| 2 | the remedy is variants, not fields | **offers `[k, grade]`** (driven) | offers `Hi`/`Lo`; **asserts the message does NOT contain the field names** |
| 3 | the control still runs | `probe_arc278_enum_variant_typo.wat` green | green — the anti-vacuity guard |
| 4 | the tagged arm | D1 refuses it; **which arm it lands on is UNKNOWN** | driven and **named**, whichever it is |
| 5 | a plain keyword stays a keyword | — | a `::`-free constant, and a `::` name whose prefix is not an enum, both still type as `keyword` |
| 6 | the `_` arm no longer holds two facts | `_ => "keyword"` covers both | the enum-prefix case has its own arm |
| 7 | blast radius | — | `typing.rs` + `error.rs` + probes |
| 8 | lints | **116/116** (measured) | green — the rider runs this |
| 9 | floor | **5216/5216** (measured) | ≥ 5,216 + every new arm, zero FAIL rows |
| 10 | clippy | **rc=0** (measured) | silent |

## The mutation proofs — one per arm

1. **The new arm** — make it fall back to `"keyword"`. Row 1's probe must go RED.
2. **The remedy list** — populate it with field names instead of variants. Row 2's probe must go
   RED. *If it does not, row 2 is asserting the presence of variants without asserting the absence
   of fields, and the confidently-wrong-remedy finding is ungated.*
3. **The narrowing** — widen the new arm to fire on any `::` name. Row 5's probe must go RED, or the
   arm is refusing legitimate keywords and nothing notices.

Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.

## Runtime prediction

30–45 minutes. The split is small; the three fixtures already exist. Trap 1 (the tagged path) is
where the unknown is, and row 4 is deliberately written as a question rather than an assertion.

## What would make this strike a failure even if every test passes

**A refusal that fires on legitimate keywords.** Row 5 and mutation 3 exist for it. `:alpha` and
any `::` name whose prefix is not an enum are correct code, and an over-wide arm would refuse them
while every new probe went green.

The second: **asserting the new remedy without asserting the old one is gone.** The finding is that
the message named FIELDS for a VARIANT mistake. A probe that only checks "the variants appear"
passes on a message that lists both, which is still the wrong remedy wearing a right one.
