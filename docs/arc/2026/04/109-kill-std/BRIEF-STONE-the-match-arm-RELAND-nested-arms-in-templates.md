# BRIEF — RELAND: the codemod converts at one level and leaves nested arms behind

> The stone is ~95% landed and **the executor said so plainly** rather than claiming a green.
> ~1867 `.wat` files converted; the reader, checker and wat-fix moved; row 4 (one grammar, not two)
> PASSED; row 5 (`#[ignore]`s gone) PASSED. **The stdlib does not parse**, so rows 1–3 could not run.

## THE DEFECT — mismatched delimiters, and it is LOUD

`wat/service.wat:1880-1884`, verbatim from disk:

```clojure
[~admin-deny-peer-kw {:pids pids}                       ← OUTER arm: converted
   (:wat::core::match (:wat::kernel::send self ~kw)
     (:wat::kernel::SendOutcome::Sent   (~serve-name …))   ← INNER arms: NOT converted
     ((:wat::kernel::SendOutcome::Lost _c) (~serve-name …)))]   ← `]` closes a `[` whose
                                                                  inner clauses stayed parens
```

The codemod converts an arm and does **not** recurse into a `match` nested inside that arm's body
when the whole thing sits in a `defservice` quasiquote template. The result is a parse error —
`#wat.parse/UnexpectedRBracket` at `service.wat:1884` — which is the good kind of failure: loud,
located, and impossible to ship past.

## THE RESIDUE — measured 2026-09-06, after the strike

```
63  .wat files still carrying an unconverted `((:` arm
 7  of them in the STDLIB, which is why the freeze dies:
      wat/service.wat · wat/telemetry.wat · wat/grep.wat · wat/rete.wat
      wat/rete/compile.wat · wat/rete/oracle/pass.wat · wat/rete/oracle/accum-pass.wat
30  .wat files contain BOTH an unquote (`~`) and a match — the template population
```

## THE WORK

Extend `wat-scripts/fixes/match-arm-to-bracket-map-pattern.wat` so it reaches **every** match arm,
including ones nested inside a converted arm's body and inside quasiquote templates. Re-run on the
63. The wat-fix already reconstructs `~name` / `~@name` from the unquote child (the executor fixed a
truncation-to-`~` in this same strike) — this is the recursion, not the binder handling.

## STOP TRIGGERS

- **STOP-1 — hand-editing a `.wat` arm.** R21. The 63 are exactly the shapes that prove the codemod
  incomplete; fixing them by hand destroys the evidence and leaves the next nested arm unconverted.
- **STOP-2 — converting only the 7 stdlib files to make the freeze pass.** The floor would go green
  with 56 files still in the old grammar, and row 4 cannot see them (it checks one planted fixture).
  Drive the `((:` count to its true floor and report what remains and why.
- **STOP-3 — the positional control gets converted.** `probe_arc109_match_arm__positional_control.wat`
  is deliberately excluded and must STAY in the retired form; it is row 4's whole subject.
- **STOP-4 — a residual `((:` that is NOT a match arm.** The pattern is a text approximation; a
  `((` opener can be an ordinary nested call. Report the count of true non-arm hits rather than
  forcing them.

## WHAT ALREADY HELD — do not re-derive it

Row 4 PASSED: the retired `(pattern body)` clause is refused by the new reader. Row 6: the codemod
exists, its dry-run diff is recorded, and it is idempotent on a second pass. Row 7: cond's clauses
were not rewritten (the diffs in shared files are match arms only — cond and match live together,
so `git diff --stat` over those files is legitimately non-zero).
