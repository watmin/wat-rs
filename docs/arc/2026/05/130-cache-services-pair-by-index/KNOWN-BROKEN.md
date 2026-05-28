# KNOWN-BROKEN tests (surfaced by arc 240, 2026-05-27)

This arc (`#226`) is actively reshaping the `:wat::lru::LocalCache` substrate
(pair-by-index via HandlePool). Arc 239 surfaced three red wat-lru tests
hidden behind the old `--lib`-only metric. Per user direction 2026-05-27 —
*"if they are broken because we are actively building their arc's dependencies
then leave them"* — arc 240 did NOT touch them; they belong to this reshape +
arc 119's consumer sweep.

**Red tests (wat-lru crate) — `crates/wat-lru/wat-tests/lru/HolonKey.wat`:**
- `deftest_wat_lru_test_local_cache_holon_key_roundtrip`
- `deftest_wat_lru_test_local_cache_holon_key_distinguishes`
- `deftest_wat_lru_test_local_cache_holon_key_structural_equal`

**Immediate cause:** `(:wat::holon::Atom <WatAST>)` drift (arc 225 narrowed
`Atom`; arc 230 made keyword/quoted forms Bind-compositions). Fix requires
migrating the holon-key construction to `to-holon` — but do it once the
LocalCache surface settles under this arc, not before.

Cross-ref: `docs/arc/2026/05/240-runtime-rot-remediation/DESIGN.md` (root cause A / DEFER set)
+ `docs/arc/2026/04/119-holon-lru-put-ack/KNOWN-BROKEN.md`.
