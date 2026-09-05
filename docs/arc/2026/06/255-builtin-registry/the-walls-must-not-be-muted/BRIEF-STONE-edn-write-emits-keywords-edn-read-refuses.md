# BRIEF — `edn::write` stops emitting keywords `edn::read` refuses

Design: `[[DESIGN-STONE-edn-write-emits-keywords-edn-read-refuses]]` (same dir). **Read its
AMENDED section first** — it retracts a rule that is not this language's, and it says which half of
the problem must NOT be solved here. Anchor: `/home/john/work/holon/wat-rs`; verify with `pwd`.

Three parts. Part 2 is the one that matters; Part 3 is small and is a defect of the same class the
campaign has paid for eight times.

---

## PART 1 — one renderer

`src/edn/render.rs:4154` `keyword_from_wat_path` — the arm `value_to_edn_with` uses for every
keyword VALUE — builds `ns` from `path()` and `name` from `leaf()`, leaving a `/` inside the name.
`Keyword::try_ns` accepts it and mints a two-slash keyword EDN cannot spell.

`src/edn/render.rs:3264` `wat_keyword_to_clojure_symbol` does it CORRECTLY, folding the receiver
into the namespace. **Share that implementation; do not re-derive it.** Two renderers for one
question is the defect.

Then regenerate the five goldens that currently hold unreadable EDN (`UPDATE_EDN=1`):

```
tests/reflection/wat_arc144_uniform_reflection__type_defstruct.edn
tests/reflection/wat_arc144_uniform_reflection__special_form.edn
tests/reflection/wat_arc144_uniform_reflection__primitive_empty.edn
tests/wat_lang/wat_arc144_hardcoded_primitives__length_primitive.edn
tests/wat_lang/wat_arc144_lookup_form__struct_head.edn
```

⛔ **`UPDATE_EDN` rewrites the WHOLE file.** `git diff` each one and confirm the ONLY change is the
keyword gaining its fold. Anything else moved is a second change riding along — STOP and report it.

## PART 2 — the wall

`Keyword::try_ns` must REFUSE a name containing `/`. Then this class cannot recur: a future caller
that forgets to fold gets an `Err`, and `keyword_from_wat_path`'s existing `Err` arm already carries
such a keyword verbatim rather than lying about its type.

⚠ This touches `crates/wat-edn/`. That is deliberate and distinct from the docstring stone's rule
against changing the WRITER: this makes a CONSTRUCTOR reject a value the format cannot represent. It
narrows what can be built and changes how nothing is written.

**Prove it fires.** A unit test constructing a slash-bearing name and asserting the `Err`. And a
witness that `write` → `read` now round-trips for a `Type/method` name.

## PART 3 — the comment that is false while its code is right

`crates/wat-macros/src/edn_doc.rs`, `fqdn_of` (committed `0582f1919`) says *"a method name does not
start uppercase, a type (and an enum variant) does."* **That is not a rule this language has** —
`:wat::core::i64` is a type and is lowercase. Every answer the function gives today is CORRECT; its
stated reason is not, and no test can catch that.

Rewrite the doc to state what it actually keys on — **the last NAMESPACE segment being uppercase**,
which correlates with record types (`Hologram`, `Bytes`, `HandlePool`) and never touches
`i64`/`String` because those sit in the NAME position — and name its limit: a record type spelled
lowercase, or a method spelled uppercase, defeats it. Cite arc 251's cutover as why that limit is
acceptable. **Do not change the code.**

---

## STOP TRIGGERS — rejections. Ship nothing, report, let me re-plan.

**STOP-1 — a live caller may depend on constructing a slash-bearing name.** If Part 2's refusal
breaks one, STOP and name it. That is a finding, not an obstacle to route around.

**STOP-2 — the goldens change in ONE way only.** See Part 1. A blessed golden that quietly absorbed
a second change is worse than the red it replaced.

**STOP-3 — do not touch the reverse direction.** No renames, no lint, no heuristic hardening, and
nothing done to `:wat::core::Bytes::{from-hex,to-hex}` or `:wat::kernel::HandlePool::{new,pop,finish}`.
The reverse ambiguity belongs to arc 251's cutover, which makes it not exist rather than solving it.
This stone makes the wire READABLE; it does not make it UNAMBIGUOUS, and it must not pretend to.

**STOP-4 — a red is a red.** Do not re-run. Capture the whole block, name the arm, report.

## What you run, and what you do not

FOREGROUND: `cargo build --release`, `cargo test --release -p wat-edn`, `target/release/wat --check`,
scoped `cargo nextest run --release -E '<expr>'`. **And `cargo nextest run --release --test lint`
before you yield** — `-p` runs do not build `tests/lint/`, which is how 20 loose assertions reached
my floor last round. No full floor (mine, centrally). No commit/push/stash/revert.

## Report

`(:wat::edn::write :wat::holon::Hologram/make)` before and after, verbatim, and the read-back ·
each golden's `git diff`, showing only the fold · the `try_ns` refusal test · the `--test lint`
result · anything that surprised you.
