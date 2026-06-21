# EXPECTATIONS — strike 5 (MapContainer registry + route assoc)

Independent scorecard, written before the strike. Graded against my own re-run, not the worker's report.

| # | what | command | expected |
|---|------|---------|----------|
| 1 | builds clean | `cargo build --release` | green, no new warnings (baseline 26) |
| 2 | **compile-forcing holds** | add `ProbeMapDummy` to `MapContainer`, `cargo build` | errors at `eval_assoc`'s `match m` (runtime.rs) + `infer_assoc`'s `match m` (infer.rs) + the 4 capability methods (map_container.rs) = 6 sites. Remove → green. |
| 3 | no test regression | `cargo test --release` | lib `941 passed; 36 failed; 1 ignored` (36 stays 36; 941 stays 941) |
| 4 | new probe green | `cargo test --release --test probe_map_container` | green — all 3 map kinds classify + assoc round-trips; capability methods exercised; non-keyed → TypeMismatch |
| 5 | seq registry intact | `cargo test --release --test probe_seq_container_registry --test probe_seq_container_parity` | green |
| 6 | clippy clean | `cargo clippy --release` | no new warnings |
| 7 | assoc behavior unchanged | the collection + record suites | identical pass set to HEAD (behavior-preserving) |

**HEAD baseline:** lib `941 / 36 / 1` (captured this session; strike 4 held it). #3 must match.

**Runtime prediction:** 12–20 min (new file + 2 routing sites + a new probe; meatier than strike 4's mechanical sweep).

**Trap-doors (named):**
- `infer_assoc` routing is the risk site — `of_type` must replace **only the classification**; every K/V
  extraction + unify body + the unresolved-`Var` backstop stays byte-identical (STOP-1). If the `&TypeEnv`
  borrow won't compose with `env.types()`, STOP (STOP-3) — don't clone to force it.
- The `Record` runtime arm moves OWNED `arg0_val`/`arg1_val`/`arg2_val` into `record_assoc_inner` — fine because
  the `of_value(&arg0_val)` borrow ends before the inner `match m` (`MapContainer: Copy`). Watch the borrow.
- `of_value` maps BOTH `wat__Record` and `wat__holon__Record` → `MapContainer::Record` (one variant, two Value
  variants — like `WatAstList`).
- `keyed_lookup`/`has_key`/`measurable` are unused by ops this strike — keep them live via the probe (exercise
  them), NOT via `#[allow(dead_code)]`.

**The contract (from DESIGN):** MapContainer = `{HashMap, PersistentMap, Record}`, capability table (current
truth), Form-1 exhaustive `match map_container` on both sides, behavior byte-identical. `get`/`contains?`/
`length`/`empty?` are strike 6.
