# wat-rs arc 051 — SimHash — BACKLOG

**Shape:** four slices. Algorithm + runtime, type checker,
tests, INSCRIPTION + docs.

---

## Slice 1 — `simhash` helper + `eval_simhash` runtime + dispatch

**Status: ready.**

`src/runtime.rs`:
- New private `fn simhash(vec, encoders) -> i64`. Charikar
  variant: for i in 0..64, bit i = sign(vec · Atom(i)_at_d).
  Pack 64 bits into i64.
- New `fn eval_simhash(args, env, sym) -> Result<Value, RuntimeError>`:
  - Arity check: 1 arg.
  - Eval the arg (must be `:wat::holon::HolonAST`).
  - Materialize via existing `encode_holon_ast` path at the
    current d (from `EncoderRegistry`).
  - Apply simhash over the materialized vector.
  - Return `Value::i64(key)`.
- Dispatch arm: `":wat::holon::simhash" => eval_simhash(args, env, sym),`.

`src/vm_registry.rs` (likely no change):
- Verify `Encoders` exposes a way to fetch `Atom(i)` as a
  vector. The existing atom-vector cache should handle this;
  may need a small public helper if access is currently
  through encode-path only.

**Sub-fogs:**
- **1a — ternary sign semantics.** What does `sign(0)` map to?
  Decision: bit 0 (off). Document in the simhash helper's doc
  comment.
- **1b — atom basis access.** If Encoders doesn't cleanly
  expose `Atom(i)` lookup, add a `pub fn atom_vector(&self, i:
  i64) -> &Vector` method that materializes-or-fetches.

## Slice 2 — check.rs scheme

**Status: ready** (slice 1 unblocks).

`src/check.rs`:
- One-line scheme registration in `register_builtins`:
  ```rust
  env.register(
      ":wat::holon::simhash".to_string(),
      TypeScheme {
          type_params: vec![],
          params: vec![holon_ty()],
          ret: i64_ty(),
      },
  );
  ```

## Slice 3 — integration tests

**Status: obvious in shape** (once slices 1 – 2 land).

New `tests/wat_simhash.rs`:

1. **Determinism** — same AST, two calls → same i64.
2. **Atom identity** — `(simhash (Atom 0))` is stable.
3. **Cosine-near-1 → small hamming distance** — same AST or
   epsilon-perturbed produces near-zero bit distance.
4. **Cosine-near-0 → ~32-bit hamming distance** — orthogonal
   atoms produce hamming distance in [20, 44] band (binomial
   variance for K=64).
5. **Anti-parallel → ~64-bit hamming** — orthogonalize/flip
   pair tested.
6. **Type system** — return is `:i64`; works with
   `:wat::core::+` and downstream arithmetic.

**Sub-fogs:**
- **3a — variance band tuning.** Initial bands may need
  widening if d=10000 tests flake. Adjust based on first run.

## Slice 4 — INSCRIPTION + USER-GUIDE + lab CHANGELOG row

**Status: obvious in shape** (once slices 1 – 3 land).

- `wat-rs/docs/arc/2026/04/051-simhash-vec-to-int/INSCRIPTION.md` —
  what shipped, the BOOK Chapter 36 unification, sub-fog
  resolutions, deferral of bidirectional cache.
- `wat-rs/docs/USER-GUIDE.md` Forms appendix gains a row for
  `:wat::holon::simhash`.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — row documenting wat-rs arc 051 + the lattice
  direction-space-axis completion.
- wat-rs commit + push.
- Lab repo separate commit + push for the CHANGELOG row.

**Sub-fogs:**
- (none.)
