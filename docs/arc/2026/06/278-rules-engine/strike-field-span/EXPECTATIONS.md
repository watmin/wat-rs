# EXPECTATIONS — three docs promise the field's span; three sites pass a form's

> **Every row's command was run against HEAD and its pre-value recorded.**

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,220 plus every arm you drive. Exceeding it is a PASS.**

## The scorecard, with pre-values measured at HEAD `9c4748b4d`

| # | what | pre-value AT HEAD | expected after |
|---|---|---|---|
| 1 | the inline-constraint caret | **cols 31–76** (46 chars) on `..._tagged.wat:26`, keyword at **col 65 len 10** (driven) | the keyword's own extent |
| 2 | the nested-constructor caret | `mod.rs:790` passes the enclosing form's span | the keyword's own extent |
| 3 | the kwargs-fact caret | `mod.rs:945` passes `fact_span` | the keyword's own extent |
| 4 | ★ the wrong span is UNWRITABLE | the param is `span: Span` — any span fits | the producer takes the node (or a newtype); **a bare `Span` no longer compiles at the call** |
| 5 | the dead arm | `mod.rs:1039` carries `bad.span` and never runs | **driven** first (trap 1), then deleted — or kept, with the drive showing it live |
| 6 | `UnknownEnumVariant` unmoved | D1's residual probes green | still green, same spans |
| 7 | blast radius | — | `validate/typing.rs` + `validate/mod.rs` + probes |
| 8 | lints | **116/116** (measured) | green — the rider runs this |
| 9 | floor | **5220/5220** (measured) | ≥ 5,220 + every new arm, zero FAIL rows |
| 10 | clippy | **rc=0** (measured) | silent |

## The mutation proofs — three producers, three mutations

They are three call paths and one mutation proves one:

1. Pass the clause's span at the **inline-constraint** producer → row 1's probe RED, rows 2–3 green.
2. Pass the form's span at the **nested-constructor** producer → row 2 RED, others green.
3. Pass `fact_span` at the **kwargs** producer → row 3 RED, others green.

**If any mutation reddens more than its own row, the probes are not on separate paths** — say so.

Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.

## Runtime prediction

50–65 minutes. The type change is small; three fixtures and trap 1's reachability drive are the work.

## What would make this strike a failure even if every test passes

**Fixing the spans without changing the type.** Three sites would be correct and the fourth author to
add a producer would pass whatever span is nearest — which is exactly how three docs came to promise
a behaviour three sites did not have. Row 4 is the strike; rows 1–3 are its evidence.

The second: **deleting the dead arm on the strength of my reading.** I asserted it is unreachable
from the shape of `:976`. Trap 1 says drive it. If it can fire, deleting it removes a real refusal
and every probe here still passes.
