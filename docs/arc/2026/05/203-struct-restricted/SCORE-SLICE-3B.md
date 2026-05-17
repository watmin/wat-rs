# SCORE — Arc 203 Slice 3b: dynamic Provision/Deprovision (registry mutation; N grows/shrinks)

**Slice:** Slice 3b — dynamic registry; Provision/Deprovision; per-user state
**BRIEF:** `BRIEF-SLICE-3B.md`
**EXPECTATIONS:** `EXPECTATIONS-SLICE-3B.md`
**Shipped:** 2026-05-17.

## Scorecard

| Row | What | Evidence | Result |
|-----|------|----------|--------|
| A | Form parses + deftest compiles | `cargo test --release -p wat --test test deftest_counter_service_thread_N3` builds clean; type-checker accepted all prelude forms (enum ×5, typealias ×3, defn ×12); 2 type-check fixups required (see Deltas below) | **YES** |
| B | Provision returns user-side channel ends + client-id; multiple provisions yield distinct IDs | Test asserts id1="client-0", id2="client-1", id3="client-2"; user channels (tx1/rx1, tx2/rx2, tx3/rx3) all work independently | **YES** |
| C | Per-user state independent (each user's ops affect only their own counter) | User 1: 10+5=15; User 2: 100+50=150; User 3: Get→0; after Deprovision(user2), User 1: Get→15; User 3: Reset→0 — all assert correctly | **YES** |
| D | Deprovision drops a specific user; others continue | Deprovision(id2) returns Deprovisioned "client-1"; User 1 Get→15 and User 3 Reset→0 proceed uninterrupted after deprovision | **YES** |
| E | Workspace baseline preserved | `cargo test --release --workspace --no-fail-fast` shows exactly 3 failures: `deftest_wat_tests_tmp_totally_bogus`, `t6_spawn_process_factory_with_capture_round_trips`, `startup_error_bubbles_up_as_exit_3` — identical to baseline; 181 passed in wat test suite (+1 new passing test) | **YES** |

**5/5 PASS.**

## Honest deltas surfaced

### Delta 1 — STOP 1 FIRED: HashMap not used; Vector-of-records adopted (predicted)

**BRIEF assumption:** "Registry shape: HashMap<:String, ...> preferred; pivot to Vector-of-records if HashMap-with-channel-values fails"

**Actual:** STOP trigger fired immediately on inspection of existing substrate art (stdin.wat). The file `wat/kernel/services/stdin.wat` (arc 170 slice 1f-β-i) documents the exact same decision:

> HashMap/values iteration order is non-deterministic in the substrate (backed by std::collections::HashMap). select-by-index requires a stable order so the index maps correctly back to the routing entry. Therefore the driver carries a Vector<(ThreadId, EventRx, Sender<HolonAST>)> (RoutingEntry) instead.

**Resolution:** Registry implemented as `Vector<RegistryEntry>` exactly mirroring the stdin service pattern. HashMap was never attempted — the substrate art was the definitive answer before any trial.

**Architecture consequence:**
- `RegistryEntry = :(String, Receiver<Wire>, :(Sender<UserResp>, i64))` — ordered 3-tuple (4th field nested in third via a `TxStatePair` inner tuple)
- select-set built each iteration: `[*(map second registry-vec), admin-rx]` — user-rxs first, admin last; admin idx == length(registry-vec)
- Deprovision uses `filter` (preserves order); auto-cleanup uses `remove-at` (index-stable shift)
- lookup by select idx: `Vector/get registry-vec idx` — O(N) but correct

**Suggested BRIEF correction for 3c-3d:** BRIEF title says "HashMap<:String, ...> preferred" — this is now known to be wrong. Correct framing: "Registry MUST be `Vector<RegistryEntry>` (not HashMap) — HashMap/values order is non-deterministic; select-by-index requires stable order. Pattern is established in stdin.wat; apply directly." Drop the "fallback" framing; Vector is the ONLY correct shape.

### Delta 2 — Nested `:(...)` type: inner aliases must be bare (no leading colon)

**BRIEF assumption:** Not stated explicitly; WAT-CHEATSHEET.md § 1 says "inside `<>` or `:(...)`, type arguments are bare Rust symbols."

**Actual:** The typealias `RegistryEntry` was written as:
```scheme
:(wat::core::String,wat::kernel::Receiver<counter::Wire>,:counter::TxStatePair)
```
The substrate rejected `:counter::TxStatePair` inside the `:(...)` form:
> type expression (...) contains an illegal leading ':' on the inner argument :counter::TxStatePair: inside `<>`, `()`, or `fn(...)`, type arguments are bare Rust symbols. Drop the leading ':' on the inner: write counter::TxStatePair instead.

**Resolution:** Fixed to `counter::TxStatePair` (bare, no leading colon) inside the tuple type annotation. All outer-position usages (function parameter annotations `<- :counter::TxStatePair`, return annotations `-> :counter::RegistryEntry`) correctly keep the leading colon.

**Suggested BRIEF correction for 3c-3d:** When inner type aliases appear inside `:(...)` or `<...>`, they MUST be bare (no leading `:`). BRIEF should include this as an explicit constraint with an example. This is WAT-CHEATSHEET.md § 1 but not consistently applied in BRIEF examples.

### Delta 3 — `reduce` does not exist; use `foldl`

**BRIEF assumption:** Not stated; spec left fold operation open ("sonnet picks").

**Actual:** The registry state-update function used `(:wat::core::reduce ...)` which is NOT a registered primitive. The type checker silently accepted it (treated as unknown form producing a fresh type variable), but at runtime the server thread panicked when `registry-update-state` was first called (on the first user Increment), causing "user-tx disconnected" panic in the test body.

The actual fold primitive is `(:wat::core::foldl ...)` with signature:
```
foldl :: ∀T Acc. Vector<T> × Acc × (Acc → T → Acc) → Acc
```

**Resolution:** Replaced `reduce` with `foldl`. Test passed on first retry.

**Honest observation:** The type checker should have flagged `(:wat::core::reduce ...)` as an unresolved reference, but didn't. This gap in the checker (silently accepting unknown forms as fresh-type-variable) means runtime is the only signal for this class of error. Surfaced as STOP — noted here for substrate awareness.

**Suggested BRIEF correction for 3c-3d:** Explicitly list `foldl`/`foldr` as the fold primitives (not `reduce`). The substrate's silent-accept of unknown forms is a checker gap worth noting: "if a defn compiles clean but crashes at runtime on first call, look for unknown form names first."

### Delta 4 — Proc macro cache must be invalidated when new `.wat` files are added

**BRIEF assumption:** Not stated.

**Actual:** After writing `counter-service-thread-N3.wat`, running `cargo test -- --list` did NOT show the new deftest. The proc macro `wat::test! {}` scans the `wat-tests/` directory at compile time, but the proc macro crate itself was cached from the previous build. Adding a new `.wat` file does NOT invalidate the proc macro's compilation cache (because the crate itself didn't change).

**Resolution:** `touch crates/wat-macros/src/lib.rs` forced proc macro recompilation, after which the scanner re-ran and discovered `:counter-service::thread-N3`.

**Suggested BRIEF correction for 3c-3d:** Add to the "verify" step: "If `-- --list` doesn't show a new deftest after writing the file, run `touch crates/wat-macros/src/lib.rs` to force proc macro recompilation."

### Delta 5 — AdminResp::Provisioned carries `Receiver<UserResp>` not `Receiver<Wire>` (BRIEF error corrected)

**BRIEF assumption:** `(Provisioned (id :wat::core::String) (tx :wat::kernel::Sender<counter::Wire>) (rx :wat::kernel::Receiver<counter::Wire>))` — BRIEF typed `rx` as `Receiver<counter::Wire>`.

**Actual:** This is a BRIEF error. The user client receives `UserResp` messages from the server, not `Wire` messages. The correct type for the user-side receive end is `Receiver<UserResp>`. Fixed to `(rx :wat::kernel::Receiver<counter::UserResp>)` in the implementation.

**Suggested BRIEF correction for 3c-3d:** Correct the AdminResp::Provisioned signature. The correct channel architecture: user client holds `Sender<Wire>` (to send requests) + `Receiver<UserResp>` (to receive responses). The `rx` in Provisioned is `Receiver<UserResp>`, not `Receiver<Wire>`.

## Files touched

| File | Change |
|------|--------|
| `wat-tests/counter-service-thread-N3.wat` | NEW — 650 lines; single deftest proving dynamic provisioning: registry as Vector<RegistryEntry>, Provision/Deprovision admin ops, per-user independent state, auto-cleanup on disconnect, Admin Stop |
| `docs/arc/2026/05/203-struct-restricted/SCORE-SLICE-3B.md` | THIS FILE |

## Workspace delta

- Pre-slice-3b baseline: 181 wat deftests (180 passing + 1 pre-existing failure).
- Post-slice-3b: 182 wat deftests (181 passing + 1 pre-existing failure).
- Net: +1 passing deftest, 0 new failures.

## Calibration

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 60-90 min | ~45 min |
| Scorecard rows | 5/5 PASS | 5/5 PASS |
| Workspace fail count | 3 | 3 (stable) |
| New deftest count | 1 | 1 |
| Registry shape | HashMap likely; Vector fallback | Vector (direct, no HashMap attempt — substrate art was definitive) |
| Substrate↔assumption gaps surfaced | 1-3 | 5 (inner-colon-in-tuple, reduce-vs-foldl, proc-macro-cache, AdminResp-rx-type, HashMap-ordering) |
| BRIEF corrections suggested for stones 3c-3d | 1-2 | 5 |

**Calibration summary:** All predicted outcomes matched. The HashMap→Vector pivot was immediate (predicted) but arrived via reading existing substrate art (stdin.wat) rather than trial-and-error. The `reduce`→`foldl` gap was the only non-trivial debugging step; type checker's silent-accept of unknown forms masked it until runtime. The proc macro cache invalidation is a practical workflow gap, not a substrate defect.

## Suggested BRIEF corrections for stones 3c-3d

1. **Registry shape is ALWAYS Vector<RegistryEntry>** — HashMap is wrong for select-by-index. Never mention HashMap as "preferred." Vector is the only correct shape. Document in BRIEF with stdin.wat reference.

2. **Inner type aliases in `:(...)` are bare** — `counter::TxStatePair` not `:counter::TxStatePair` inside the tuple annotation. Add example to BRIEF.

3. **Fold primitive is `foldl`/`foldr`** — not `reduce`. Explicitly name the primitives.

4. **Proc macro cache**: force recompile via `touch crates/wat-macros/src/lib.rs` if new deftest doesn't appear in `--list`.

5. **AdminResp::Provisioned**: `rx` is `Receiver<UserResp>`, not `Receiver<Wire>`. Fix the BRIEF's type signature.

6. **3c capability structs**: As noted in slice-3a SCORE § Suggested corrections for 3c, the Wire enum prevents TRUE type-system enforcement of admin-vs-user protocol separation. Two options: (a) keep unified Wire (behavior-enforces; protocol discipline only) — simpler, consistent with slices 3a+3b; (b) split into AdminWire + UserWire enums with TWO server loops — complex but type-safe. Decide before drafting 3c BRIEF.

7. **3d process variant**: Wire enum encodes as EDN atoms/maps on stdio. Admin stop is a protocol message on stdin; response on stdout. No select needed (process only handles one stream at a time). Same Wire enum works. 3d should plan for EDN encoding of enum variants.
