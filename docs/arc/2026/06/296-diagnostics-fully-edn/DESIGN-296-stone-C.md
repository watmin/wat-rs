# Stone C — data equality: flip the error goldens from string-eq to parsed-EDN-eq

**Arc 296, after R1 PROBATVM EST (stone B, `b564b1bf`).** The errors are EDN
now. So the tests should prove it the honest way — **by parsing, not by string
match.** A string-equality assertion can green on a malformed blob that happens
to match; a **data-equality** assertion parses the emitted output as EDN and
compares the *values* — so a non-EDN or malformed error **cannot pass.** This is
the constraint-engineering wall for the whole error surface (FACTVM NON PACTVM,
one layer up: the test proves EDN-ness by *parsing*, not by trusting a string),
it is format/whitespace/key-order robust, and it dogfoods wat-edn's own parser.

The builder's directive: *"flip all the assertions into parsed edn — it's not
just string equality, it is data equality."*

## The mechanism

Mint a data-equality assertion (a shared test helper / `#[macro_export]` macro):

```rust
/// Assert two EDN strings are DATA-equal: parse both via wat_edn and compare
/// the parsed Values. A malformed emission FAILS to parse → the test fails
/// (you cannot green a non-EDN error). On mismatch, show both parsed forms + raw.
macro_rules! assert_edn_eq {
    ($actual:expr, $expected:expr $(, $msg:tt)?) => {{
        let a_raw = $actual; let e_raw = $expected;
        let a = wat_edn::parse(&a_raw).expect("ACTUAL is not valid EDN — a non-EDN error face");
        let e = wat_edn::parse(&e_raw).expect("EXPECTED golden is not valid EDN");
        assert_eq!(a, e, "EDN data mismatch{}\n--- actual ---\n{}\n--- expected ---\n{}", …, a_raw, e_raw);
    }};
}
```

**Multi-part goldens:** some goldens join two render-formats with a `\n---\n`
delimiter (`<edn-A>\n---\n<edn-B>`). For those, split on `\n---\n` and
`assert_edn_eq!` each part (a sibling `assert_edn_parts_eq!` or an inline split).
The `---` is a test delimiter, not EDN — do not feed it to the parser.

## The flip

The ~72 files that assert an error's EDN output as a **string literal**
(`assert_eq!(to_wire_edn(&err), "#wat.…/…")`, `{:?}`/`{}` on an error, etc.) flip
to `assert_edn_eq!`. Non-error, non-EDN assertions (e.g. `assert_eq!(n, 1)` for a
length) are **left alone** — only the error/EDN goldens flip.

## The discovery value (a STOP that is a finding)

Because `assert_edn_eq!` *parses* the actual output, flipping a golden whose
actual is **not valid EDN** will fail at parse. Per R1, every error face is EDN
now — so a parse failure means an error face stone B did not convert (a genuine
gap). **STOP and report it as a finding** — do not work around it with a string
compare. The flip is thus a proof, across the whole surface, that R1 holds.

## Rooms / blast radius

A shared assert helper (place in the existing test-util home — e.g. beside
`startup_beside` in `src/freeze.rs`, or a `#[macro_export]` at the crate root so
`tests/**` can use it). The ~72 error-golden test files. No production code
changes — this is test infrastructure only.

## Out of scope (REJECTED here)

- The `{…}` display-glyph in `:callee`/`:op` strings — it is legal EDN (string
  content), parses fine, so data-eq does not force it. Cleaning it to a real
  construct identity is a SEPARATE follow-up (message-content, not assertion).
- Any production error-face change (that was stone B) — EXCEPT surfacing a
  parse-failure as a finding (STOP, above).

## STOP triggers (rejection criteria)

- **STOP-1 (the finding):** an error golden's actual output fails to parse as EDN → a non-EDN face survived stone B. STOP, report the exact site + output; do NOT fall back to string compare.
- **STOP-2:** a golden's structure is neither a single EDN form nor `\n---\n`-joined EDN parts (some other shape). STOP, report it — do not guess a split.
- **STOP-3:** `wat_edn::parse` is single-form; if a golden is genuinely multiple top-level EDN forms with no delimiter, STOP and surface it (may need `parse_all`).

## Expectations (scorecard)

| # | what | command | expected |
|---|---|---|---|
| 1 | the helper exists + is data-eq | read it | parses both sides, compares Values, fails on unparseable |
| 2 | error goldens flipped | `grep -rn "assert_edn_eq!" tests/ src/ \| wc -l` | ≈ the ~72 flipped sites |
| 3 | full suite green | `cargo test` | 0 failed (save the 7 wat_dispatch flakes) |
| 4 | data-eq actually parses (proof) | flip is real, not cosmetic | each flipped assert parses actual+expected as EDN |
| 5 | no error golden left string-comparing EDN | spot-check 5 flipped files | `assert_edn_eq!`, not `assert_eq!` on an EDN string |
| 6 | any non-EDN face surfaced | the STOP-1 findings, if any | reported, not worked around |

**Runtime:** 45–90 min. **Trap-doors:** a golden that mixes an EDN part and a genuinely-non-EDN part (STOP-1 — a finding); key-order differences that string-eq tolerated but were actually masking a bug (data-eq will catch — good); a `to_wire_edn` output that is a bare string (not a tagged form) — still legal EDN (a string parses), so it flips fine.

## On landing

Every error test proves its subject is valid EDN by parsing it — the thesis
("every error string is EDN") made structural at the *test* surface, not just
the emission. And any error face stone B missed is surfaced as a finding, not
hidden behind a string match.
