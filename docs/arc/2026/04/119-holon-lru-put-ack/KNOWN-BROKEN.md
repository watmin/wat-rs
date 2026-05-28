# KNOWN-BROKEN tests (surfaced by arc 240, 2026-05-27)

Arc 239 made the workspace test-build compile; the full
`cargo test --workspace` then surfaced runtime failures hidden behind the
old `--lib`-only metric. Three of them belong to **this arc's in-flight
consumer sweep** (`#208`: "consumer sweep across both crates' wat-tests").
Per user direction 2026-05-27 — *"if they are broken because we are actively
building their arc's dependencies then leave them"* — arc 240 did NOT fix
these; they are this arc's to close.

**Red tests (wat-lru crate):**
- `deftest_wat_lru_test_local_cache_holon_key_roundtrip`
- `deftest_wat_lru_test_local_cache_holon_key_distinguishes`
- `deftest_wat_lru_test_local_cache_holon_key_structural_equal`

**File:** `crates/wat-lru/wat-tests/lru/HolonKey.wat`

**Immediate cause:** `(:wat::holon::Atom <WatAST>)` — `Atom` narrowed to
`HolonAST→HolonAST` (arc 225); the holon-key construction needs migration to
`to-holon` (and the arc-230 Bind-composition shape for keyword/quoted forms).

**Why deferred here (not fixed by arc 240):** the lru wat-tests are arc 119's
active `#208` consumer-sweep turf, and **arc 130 (`#226`) is actively
reshaping the `:wat::lru::LocalCache` substrate** — fixing the holon-key shape
now would step on that in-flight reshape and risk re-breaking. Close it as
part of arc 119 step 7 / arc 130, when the LocalCache surface settles.

Cross-ref: `docs/arc/2026/05/240-runtime-rot-remediation/DESIGN.md` (root cause A / DEFER set).
