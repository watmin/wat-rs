# BRIEF — W2a: auto-mint `<fqdn>::kwargs-check` at the kwargs-defn codegen site

> **Arc 170, the W2 (Path B) flight.** Redo clean off the strike-ready recipe (drawn 2026-07-10;
> written-then-reverted with the main-wall mess). Mechanism PROVEN (the committed colocation
> fixture); this stone AUTO-mints the checker so `bracket/uses` (W2) becomes a thin macro.
> **Executor: sonnet shadowdancer.** Weighed by the orchestrator's own re-run.

---

## The work, in one paragraph

At the site where `defn`'s **kwargs branch** already emits `record-def` + the `$impl` fn + the
companion macro, ALSO auto-mint a **fourth** form: `<fqdn>::kwargs-check` — a kwargs fn whose
field-ordered params are the `::Kwargs` field types with each `Peer'<S,R>` **head-swapped** to
`Address'<S,R>` (data-typed fields pass through untouched). Because the checker is *itself* a
kwargs defn, guard against minting a checker-of-a-checker. Then two committed tests prove the
auto-mint: a correct kwargs call to the auto-minted checker freezes CLEAN; a swapped one is a
located `TypeMismatch` at freeze. Recapture the one golden that shifts. **No `bracket/uses` yet
— that is W2, the next stone.**

## The contract decision (pinned)

The head-swap runs inside `defn`'s **bootstrap-critical** macro body (`wat/core.wat`), where
`take`/`drop`/`map` are known-unavailable (lazy-flip wall). GROUNDED SAFE: `string::split`,
`string::join`, `string::subs`, `string::concat`, `string::contains?`, `ast-name`, `keyword-node`
are **Rust-native** (`src/string_ops.rs`, `src/edn_shim.rs`; `stdlib.rs:268` — string.wat
*consumes* them). Use ONLY those primitives in the new code. Do NOT reach for a wat-defined helper
(it won't be resolvable this early).

---

## Read in order (the rooms)

1. **`wat/core.wat:721`–`899`** — `defn`'s kwargs branch. THE mint site.
   - `:756` `record-def = `(:wat::core::defstruct ~kwargs-ty ~kw-argvec)`` — the shape template.
   - `:734` `kw-ch = (ast->children kw-argvec)` — the flat kwargs children, in **triples**
     `[fname@i·3, arrow@i·3+1, type@i·3+2]`; `:735` `kw-len`, `:736` `n-kw-fields = kw-len/3`.
   - `:725` `name-str` — the fn's name string (for the guard + the checker name).
   - `:876`–`897` the emit **`do` block**: `~record-def` · the `$impl` `def` · the companion
     `defmacro`. **You add a 4th form here.**
2. **`wat/bracket.wat:338`** — the EXACT head-swap `process-work-forms` already proves:
   ```clojure
   addr   (:wat::core::string::join "Address'" (:wat::core::string::split c-nm "Peer'"))
   ;; c-nm = (ast-name type-node) → ":wat::kernel::Peer'<S,R>"; addr → ":wat::kernel::Address'<S,R>"
   ;; addr RETAINS the leading ':' (split/join don't touch it) → (keyword-node addr) is the new type node.
   ```
   (bracket strips the colon via `subs` only because it embeds addr into a larger string — you do
   NOT need that; `(keyword-node addr)` directly.)
3. **`tests/services/probe_arc170_wrong_service_colocation.wat.bad`** — COPY its two-surface +
   two-defservice + `Dialable/coord` shape. It proves the swap→TypeMismatch mechanism the checker
   reuses; your test replaces its hand-written positional `:probe::dial-all` with the AUTO-minted
   kwargs-keyed checker.
4. **`tests/services/probe_arc170_wrong_service_compile_error.rs`** — COPY its structural negative
   assertion (`StartupError::Check(CheckErrors(errs))` → `match errs[0].kind { TypeMismatch {
   expected, got, .. } => assert_eq!(expected, ":wat::kernel::Address'<…>") … }`). Structural, so
   it passes BOTH lints (`no_inlined_wat` — a bare keyword string is not an inlined form;
   `no_loose_string_assert` — a match + `assert_eq!` is not `contains`/`starts_with`).
5. **`tests/wat_lang/wat_core_cond.rs:71`** + **`…/wat_core_cond__cond_refuses_missing_else.edn`** —
   the ONLY golden that embeds a `core.wat` span (`:line 970` + `:line 980`, the cond macro-error
   site at `core.wat:961+`). Adding N lines at `:876` shifts both down by N → recapture.

---

## Implementation sketch (fill it; do not invent the shape)

**In the `let` (before the `do` at `:876`), add these bindings:**

```clojure
;; ── W2a: the kwargs-check name + the recursion guard ──
kwargs-check-name-str (:wat::core::string::concat name-str "::kwargs-check")
kwargs-check-kw       (:wat::core::keyword-node (:wat::core::string::concat ":" kwargs-check-name-str))
;; GUARD: this defn is ITSELF a kwargs-check (it has `& [...]`) → do NOT mint its checker (infinite mint).
;; Prefer a SUFFIX test; string::ends-with? if native, else compare (subs name-str (len-13) len) == "::kwargs-check".
;; string::contains? is a grounded-native fallback (no user fn legitimately contains that substring).
is-check (<suffix test on name-str for "::kwargs-check">)

;; ── the head-swapped argvec: fold kw-ch, swap Peer' TYPE nodes only ──
swapped-ch (:wat::core::foldl
             (:wat::core::fn [acc <- :wat::core::Vector<wat::WatAST> j <- :wat::core::i64] -> :wat::core::Vector<wat::WatAST>
               (:wat::core::let
                 [child (:wat::core::Option/expect (:wat::core::get kw-ch j) "w2a swapped-ch index")
                  is-type (:wat::core::= (:wat::core::i64::mod j 3) 2)   ;; type position (i·3+2). i64::mod is native (runtime.rs:4323).
                  nm      (:wat::core::ast-name child)                    ;; ":wat::kernel::Peer'<S,R>" or a data type
                  is-peer (:wat::core::if is-type (:wat::core::string::contains? nm "Peer'") false)
                  swapped (:wat::core::if is-peer
                            (:wat::core::keyword-node
                              (:wat::core::string::join "Address'" (:wat::core::string::split nm "Peer'")))
                            child)]
                 (:wat::core::conj acc swapped)))
             (:wat::core::Vector :wat::WatAST)
             (:wat::core::range 0 kw-len))
swapped-argvec (:wat::core::with-children kw-argvec swapped-ch)

;; ── the checker form (guarded) ──
;; GUARD NO-OP = (do nil), NOT an empty (do): an empty `(:wat::core::do)` is ILLEGAL —
;; "do form requires at least one form; got zero" (runtime.rs:3488, check.rs:8145;
;; Record.wat:108 hit exactly this). `(do nil)` has one form, evaluates to nil, is discarded.
kwargs-check-def (:wat::core::if is-check
                   `(:wat::core::do nil)                                 ;; no-op: don't recurse on a checker
                   `(:wat::core::defn ~kwargs-check-kw [& ~swapped-argvec] -> :wat::core::nil nil))
```
*(`i64::mod` confirmed native at runtime.rs:4323. Or drop modulo entirely: iterate
`(range 0 n-kw-fields)`, index `fname@fi·3 / arrow@fi·3+1 / type@fi·3+2`, swap only the type —
mirrors `let-binder-items` at core.wat:801. Either is grounded.)*

**In the emit `do` block (`:876`), splice the 4th form:**

```clojure
`(:wat::core::do
   ~record-def
   (:wat::core::def ~impl-name-node …)
   (:wat::core::defmacro ~name …)
   ~kwargs-check-def)          ;; ← W2a. Order-independent (the checker refs only literal Address' types).
```
The `(:wat::core::do)` no-op for a checker is a harmless top-level form (defines nothing) — the
recursion terminates at depth 1.

## Blast radius

`wat/core.wat` (the kwargs branch only) · 2 new test files + 2 fixtures under `tests/services/` ·
1 golden re-blessed (`tests/wat_lang/wat_core_cond__cond_refuses_missing_else.edn`). **No other
file.** No new Rust. The nil body of `<fqdn>::kwargs-check` is FINE — the UselessMain wall is
`:user::main`-ONLY.

## The gate (committed tests — prove the AUTO-mint)

Copy the colocation fixture's two surfaces + two defservices (`:probe::Echo`/`:probe::echo'`,
`:probe::Kv`/`:probe::kv'`), then:

```clojure
;; the kwargs work-fn → AUTO-mints :probe::enrich::kwargs-check
(:wat::core::defn :probe::enrich
  [item <- :wat::core::String
   & [echo <- :wat::kernel::Peer'<probe::Echo::Op,probe::Echo::Reply>
      kv   <- :wat::kernel::Peer'<probe::Kv::Op,probe::Kv::Reply>]]
  -> :wat::core::String
  (:probe::Echo::EchoResponse/reply (:probe::Echo/echo echo (:probe::Echo::EchoRequest item))))

;; POSITIVE (…_ok.wat) — the correct kwargs call to the auto-minted checker → freezes CLEAN
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [eh  (:probe::echo'/start :locus (:wat::spawn::process) :record (:probe::echo'::Record))
     kvh (:probe::kv'/start   :locus (:wat::spawn::process) :record (:probe::kv'::Record))]
    (:probe::enrich::kwargs-check :echo (:wat::capability::Dialable/coord eh)
                                  :kv   (:wat::capability::Dialable/coord kvh))))

;; NEGATIVE (…_swap.wat.bad) — SWAPPED handles → a located TypeMismatch at freeze:
;;   :echo (Dialable/coord kvh) :kv (Dialable/coord eh)
;;   expected :wat::kernel::Address'<probe::Echo::Op,probe::Echo::Reply>
;;   got      :wat::kernel::Address'<probe::Kv::Op,probe::Kv::Reply>
```

Tests (`tests/services/probe_arc170_w2a_kwargs_check_mint.rs`):
- `startup_from_file(ok.wat)` → `.expect(...)` (freeze type-checks; main is not run at startup).
- `startup_from_file(swap.wat.bad)` → `Err`; STRUCTURAL match on `StartupError::Check` →
  `errs[0].kind == TypeMismatch { expected: ":wat::kernel::Address'<probe::Echo::Op,probe::Echo::Reply>",
  got: ":wat::kernel::Address'<probe::Kv::Op,probe::Kv::Reply>" }`.

If the swap yields a NAME error (extra/missing `:key`) instead of a TypeMismatch, that's a
different (also-valid) rejection — but the swap here keeps both names, so it MUST be a TypeMismatch.

## Recapture the golden (LAST, after everything else is green)

```
UPDATE_EDN=1 cargo nextest run -p wat --test wat_core_cond -E 'test(cond_refuses_missing_else)'
```
Then **diff the `.edn`** and CONFIRM the only change is the two line numbers shifting by exactly N
(970→970+N, 980→980+N), same `:file "wat/core.wat"`, same `:col`, same error structure. A change
to `:col`, `:file`, or the error shape means a real regression — STOP, do not bless it. Re-run
without `UPDATE_EDN` → green.

---

## STOP triggers (rejection criteria — ship nothing, surface the gap)

1. **No i64 modulo verb resolves** (`i64::%` / `mod` / `rem`) for the type-position test → STOP.
   (Alternative: iterate `(range 0 n-kw-fields)` and index `j = fi·3+2` directly, no modulo — use
   this if cleaner; it mirrors `let-binder-items` at `:801`.)
2. **`:wat::capability::Dialable/coord` does not resolve** to `Address'<S,R>` → STOP (do NOT
   hand-declare a `Dialable` — it auto-emits per defservice; hand-declaring DUPLICATE-defines).
3. **The head-swap primitives error inside `defn`'s body** (a lazy-flip / unresolvable-this-early
   failure the grounding missed) → STOP and report the exact form + error; do NOT swap in a
   wat-defined helper.
4. **The swap fixture fails with something other than a `TypeMismatch` on the two `Address'`
   types** → STOP and report the actual diagnostic (do not weaken the assert to match it).

## How to work (the hard lessons, baked in)

- **Run the floor FOREGROUND-blocking**: `cargo nextest run --release` (or targeted `-p wat
  --test …`) — NEVER background it, never `&`/`disown`/`setsid`/double-fork. A backgrounded floor
  orphans and you return a fragment report.
- **A mid-edit file is a PHANTOM**: a rust-analyzer / rustc cascade on a just-edited tree is a
  stale snapshot. A `cargo build` clean + a suite that RAN N tests COMPILED. Ground the real
  signature before crying cascade.
- Negative-test asserts are STRUCTURAL (both lints). Never rune a lint your change trips — fix it.
- Report the floor as `cargo nextest run` shows it (the full Summary line), not a grepped subset.

## Expectations (scorecard — verify each, report the real result)

| what | command | expected |
|---|---|---|
| the mint is well-formed | `cargo build --release` | clean |
| the auto-minted checker accepts the correct call | `cargo nextest run -p wat -E 'test(w2a_kwargs_check_mint)'` (ok) | PASS (freezes) |
| a swapped handle is a compile error | same (swap) | PASS (TypeMismatch on the two Address' types) |
| no recursion / no infinite mint | build + the whole floor | terminates; 0-new |
| the golden shift is pure | diff the `.edn` | ONLY 970/980 → +N |
| the full floor holds | `cargo nextest run --release` (FOREGROUND) | prior floor + these; 0-new (modulo the 1 known `no_inlined_wat` tracker) |

Runtime band: ~15–25 min. Report the real Summary line and the honest deltas.
