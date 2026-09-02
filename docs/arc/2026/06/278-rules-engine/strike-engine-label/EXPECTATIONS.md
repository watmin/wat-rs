# EXPECTATIONS — a benchmark row that says "engine" and times something else is a wrong number

> **Every row's command was run against HEAD and its pre-value recorded.**

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,296 plus every arm you drive.**

## The scorecard, with pre-values measured at HEAD `7ed71cf12`

| # | what | pre-value AT HEAD | expected after |
|---|---|---|---|
| 1 | `(engine)` sites | **3** — `accum_cost:1354`, `gather_probe:176`, `:289` (driven) | 2 carry `(engine: <fn>)`; the third carries no engine claim |
| 2 | the false one | `gather_probe:176` body does `s.insert(f.clone())` (driven) | relabelled; **report what to** |
| 3 | the true ones | `:289` → `super::seen_insert`; `accum_cost:1101` → `intern_val` (driven) | named, and the names resolve |
| 4 | ★ resolution excludes the test tree | — | a decoy `fn seen_insert` in a test file does **not** satisfy the gate |
| 5 | a bare `(engine)` REDs | — | driven |
| 6 | non-vacuity | — | a floor; zero sites found must RED |
| 7 | C2's second citation | `accum_cost.rs:1383` is a `const RUNS`, no label (driven) | reported as stale, not "fixed" |
| 8 | radius | — | 2 test files + one gate. No `src/` behaviour change |
| 9 | lints | **182/182** (measured) | green |
| 10 | floor | **5296/5296** (measured) | ≥ 5,296, zero FAIL rows |
| 11 | clippy | **rc=0** (measured) | silent |

## The mutation proofs — the population is 3, so every one must be driven

1. **Bare `(engine)`** restored at a converted site → RED.
2. **`(engine: not_a_real_fn)`** → RED, naming the unresolved name.
3. **★ Decoy** — define `fn seen_insert` inside `kernel/tests/` and remove the label's real target
   from resolution → must still RED. *A label satisfied by a fixture is the self-vouching failure
   this session has hit four times.*
4. **Blind the walk** → the non-vacuity floor REDs.

Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.

## Runtime prediction

40–55 minutes. Three sites and a small gate; the decoy mutation is the fiddly one.

## What would make this strike a failure even if every test passes

**Inventing a production name for the `S` arm.** It times a raw set insert; there is no engine
function it calls. Naming one would make the label pass the gate while staying exactly as false as it
is today — the gate would then be certifying the defect. Row 2 wants it **relabelled**, and the report
must say to what.

The second: **resolving the named function anywhere.** If `kernel/tests/` counts, a benchmark can
satisfy its own engine claim with a helper it wrote, and the gate proves nothing. Row 4 and mutation 3.
