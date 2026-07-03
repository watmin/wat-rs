# RED probes — rete fixpoint truth-maintenance flaw (ALIVS ARGVIT, 300 interstitial 2026-07-02)

Surfaced building 300's conversion as a forward-chaining rete network. Confirmed vs Clara.

- `chain.wat`  — R1: A→B, R2: B⋈A→C. `fire-rules'` → C=0 (single-pass, no cascade).
- `chain-fp.wat` — same, `fire-fixpoint` → C=2 (cascades).
- `neg.wat`    — negation over a derived fact + `fire-fixpoint` → Bad=2, Ok=2 (NO dedup — BUG).
- `chain.clj` / `neg.clj` — Clara reference: C=2, Bad=1, Ok=1 (correct truth-maintenance).
  Run: clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}} :paths ["<dir>"]}' -M -m chain

THE FLAW: `fire-fixpoint` re-inserts a re-derived fact every round (no truth-maintenance).
`fire-rules'` is single-pass (no fixpoint). Fix in the rete arc (278): idempotent derived-fact
insertion + ideally a native `fire-fixpoint'`. Then 300 resumes.
