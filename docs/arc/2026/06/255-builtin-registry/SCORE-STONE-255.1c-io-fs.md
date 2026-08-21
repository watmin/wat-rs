# SCORE — STONE 255.1c-io-fs — HOME #12 CLOSED

Rider runtime **8.4 min** against a predicted 12–20. No STOP fired.

| # | what | result |
|---|---|---|
| 1 | ★ `:wat::io::` literal dispatch | ✅ **0** — the family is fully carved |
| 2 | `stdlib::sources` survived | ✅ 1 (appears in the diff only as comment prose) |
| 3 | six rows | ✅ 6 |
| 4 | registry 120 → 126 | ✅ 126 |
| 5–6 | `check.rs` / `tests/` untouched by the rider | ✅ empty |
| 7 | builds | ✅ 18.3s |
| 8 | five registry gates | ✅ 5/5 |
| 9 | ★ the gate can FAIL | ✅ `TempFile/path` `@ret` String→i64 → RED, named; reverted byte-identical |
| 10 | probe | ✅ 29 · 28 · 1 · 0 |
| 11 | ★★ the rete completeness gate | ✅ PASS — see below |
| 12 | floor | ✅ **4818/4818, 0 FAIL, 70.1s** |
| 13 | clippy | ✅ 0 |
| 14 | rustfmt | ✅ `io/` clean — one of the rider's lines fixed; zero drift added by home #12 |
| 15 | goldens | ✅ 25209 → **25190**, `:col 17` untouched (27 removed, 8 added, net 19) |

## ★★ Row 11 — the union fix held under the condition it was built for

`85174fc3f` repaired `dispatch_verbs` to enumerate the UNION of `runtime.rs` arms and registered
intrinsics, because the carve was draining the only population the gate could see. **This strike moved
six more verbs across that boundary.** Under the old scan the population would now read **394** and
the gate would be RED on its own non-vacuity floor. It is green. The union is counting the verbs that
moved, and the gate now grows with the carve instead of shrinking.

## ⛔ MY BRIEF'S `check.rs` LINE NUMBERS WERE INVENTED, AND THE BRIEF CALLED THEM MEASURED

I wrote `TempFile/new 15938 · TempFile/path 15947 · TempDir/new 15956 · TempDir/path 15965 ·
read-file 15974 · list-dir 15983` and labelled them *"the ones the ORCHESTRATOR measured."* They were
**extrapolated** from the writer block's `+9` spacing. The tell is in the numbers themselves:

```
mine:  15938 15947 15956 15965 15974 15983     diffs  9 9 9 9 9   ← an arithmetic progression
true:  15946 15955 15964 15973 15988 15997     diffs  9 9 9 15 9  ← read-file's comment block is longer
```

Line 15938 is `},`. Every one was wrong, and no extrapolation could ever have produced the real `+15`.
This is a fabricated citation formatted exactly like a measurement, in a committed brief — the same
class as inventing a backtrace. `[[feedback_a_rendered_example_is_not_a_measurement]]`

**It cost nothing for one reason only:** the same brief said *"confirm each by matching the verb
string, and report any that moved rather than trusting the number."* The rider did, corrected all six,
and reported them. An instruction to distrust me is what saved a claim I had no right to make — and
the instruction was sitting next to the false assertion that they were measured.

## ⛔ THREE OF SIX RUNNABLE `@example`s IN THE IO HOMES DID NOT RUN — AND THE GATE THAT WOULD CATCH IT IS ASLEEP

A `Pure`+`Deterministic` row must carry a RUNNABLE `@example` (`purity_mandated_examples`). Nothing
executes them: `tests/reflection/probe_arc255_ivb2b_verify_examples.rs:33` is `#[ignore]`d on
*"arc-255 metadata-of reflection (builtin-registry) not yet built"* — a reason that expired; the
registry has **126 rows** and answers `metadata-of`. So the contract is enforced for PRESENCE and
unenforced for TRUTH.

I extracted all six runnable examples across `io/{reader,writer,fs}.rs` and executed each against the
built binary. **Three failed:**

| row | defect | fix |
|---|---|---|
| `fs.rs` `TempFile/path` + `TempDir/path` | `:wat::core::length` rejects a String — it takes collections | `:wat::core::string::length` |
| `reader.rs` `from-bytes` | `(:wat::core::Vector :u8 …)` — **`:u8` is a RETIRED bare primitive**, `BareLegacyPrimitive` at check time | `:wat::core::u8` |
| `writer.rs` `to-bytes` | expected value written as a CONSTRUCTOR form, `(:wat::core::Vector :u8)`; the real render is `[]` | `[]` |

★ **The `reader.rs` one is a KNOWN ROT THAT PROPAGATED.** The home #2 commit (`25c1f4521`) recorded it
under "TRACKED, NOT FIXED": *"home #1's own `Bytes::to-hex` example uses a bare `:u8` that arc 109's
`BareLegacyPrimitive` rule now rejects. A rot in the reference template."* A rider copied the template
and the rot came with it — which is exactly what an unenforced reference template does.

All six now execute and match. **This is the strongest argument yet for un-`#[ignore]`ing the example
runner**: three defects in the newest, most carefully reviewed code in the tree, none catchable by any
gate that runs today, all catchable in one second by the gate that doesn't.

## Categories — six rows, weighed at the bodies

`Resource ×2 · Projection ×2 · Io ×2`, and not one row needed forcing:

- **`TempFile/new` / `TempDir/new` → `:Resource`.** `NamedTempFile::new()` (`io.rs:1638`) and
  `TempDir::new()` (`:1669`) each create a real on-disk object that `Drop` unlinks.
- **`TempFile/path` / `TempDir/path` → `:Projection`.** `f.path().display().to_string()` reads a field
  already stored on the handle — the axis prose verbatim.
- **`read-file` / `list-dir` → `:Io`.** `fetch_source_file` and `std::fs::read_dir` pull bytes and
  entries across the process boundary; neither opens anything the caller keeps.

**No row failed to classify** — the first strike in this home where that is true, and worth noting
rather than passing over: it means the two strains (`rewind`, `IOWriter/new`) are specific to the
reader/writer handle types, not a general weakness of the taxonomy over io.

★ The rider volunteered the axis check on `*/path` unprompted: `Pure`+`Deterministic` mandates a
runnable example, which is the same trap `HandlePool::finish` documents (a missing runnable example
was the *symptom* of a wrong `@Determinism`). It checked whether the same defect applied —
`finish`'s `rx.len()` reads an externally-mutable queue, so two calls can disagree; a `NamedTempFile`'s
path is fixed for the handle's life — and concluded `Deterministic` holds. Verified: correct.

## Honest deltas

- **`io/mod.rs` said "All thirty arms"; the family is 29** (10+13+6). Pre-existing off-by-one from the
  reader stone. The rider corrected it while rewriting that paragraph and reported it.
- The brief's `runtime.rs` boundary map was exact; the rider deleted by text match, not line range.
- One rustfmt nonconformance in the rider's `list-dir` body, fixed. `io/` adds zero drift.
