;; Arc 278 #57 round 1b — sanity probe for the eight newly-minted rows: the parametric
;; PersistentVector trio (Alias, type_params ["T"]) and, originally, five Redispatch-class
;; higher-order combinators — now four (arc 118.B6b retired `:wat::rete::core::foldr`'s row: a
;; `Redispatch` alias pointing at a core verb, `:wat::core::foldr`, that no longer exists is a
;; dangling declaration, so the row went with it; see the STOP block at `:probe-foldr`'s old
;; site below). Not a test file itself (see tests/rete for the durable gate); this is a
;; loadable, type-checked reference proving the new spellings resolve and, for the PV trio,
;; that a real parametric TypeScheme is now attached (round 1a's rows were all monomorphic).
(def :probe-pv-length (:wat::rete::core::PersistentVector/length (:wat::core::PersistentVector 1 2 3)))
(def :probe-pv-contains (:wat::rete::core::PersistentVector/contains? (:wat::core::PersistentVector 1 2 3) 2))
;; BRIEF-get-is-total-by-fallback.md (2026-08-05) — `PersistentVector/get` converted from
;; `Alias` (2-arg: container, index) to `Fallback` (4-arg: container, index, `:undefined`,
;; fallback) — STOP-5's own predicted collateral, found here: this call site was the one
;; existing caller (of 276 corpus files) still on the old 2-arg shape. Updated in place.
(def :probe-pv-get (:wat::rete::core::PersistentVector/get (:wat::core::PersistentVector 1 2 3) 1 :undefined -1))

;; ── T=String — the instantiation that makes this file test PARAMETRICITY, not resolution ────
;;
;; Added 2026-08-05. Every PV line above is i64-only, so a MONOMORPHIC `i64` scheme would pass
;; all three identically — the header above claimed to prove "a real parametric TypeScheme is
;; now attached" and did not. These go RED if the trio's `type_params: ["T"]` ever regresses.
;;
;; `get` is the strongest of the three: `T` unifies across the container's element type, the
;; `:undefined` fallback argument, AND the return, so a monomorphic scheme fails it in three
;; places at once. Verified by run — in-range -> "b", out-of-range -> "missing".
;;
;; The matching NEGATIVE control lives in `tests/`, not here: a `wat-scripts/` file must LOAD
;; (`every_wat_scripts_file_loads`), so a deliberately type-broken form cannot live in this
;; directory. Measured for the record — `(… /get (PersistentVector "a" "b") 1 :undefined -1)`
;; is refused at `--check`: "parameter #4 expects :wat::core::String; got :wat::core::i64",
;; exit 1. `T` is inferred String from the container and the i64 fallback is rejected.
(def :probe-pv-length-str
  (:wat::rete::core::PersistentVector/length (:wat::core::PersistentVector "a" "b")))
(def :probe-pv-contains-str
  (:wat::rete::core::PersistentVector/contains? (:wat::core::PersistentVector "a" "b") "a"))
(def :probe-pv-get-str
  (:wat::rete::core::PersistentVector/get (:wat::core::PersistentVector "a" "b") 1 :undefined "missing"))

(def :probe-foldl
  (:wat::rete::core::foldl
    (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ acc x))
    0
    (:wat::core::PersistentVector 1 2 3)))

;; ⛔ `:probe-foldr` REMOVED, arc 118.B6b — `foldr` retired from core (it was `reverse`+`foldl`
;; wearing a name borrowed from Haskell, distinct only under laziness wat does not have), so
;; `:wat::rete::core::foldr`'s `Redispatch` row was deleted alongside it (a dangling alias
;; otherwise). The replacement composition, `(reduce f init (reverse coll))`, has NO rete
;; spelling — the rete vocabulary has `foldl`/`mapv`/`filterv`/`reduce` but no `reverse` row (it
;; has never had `map`/`filter`: those are the lazy core heads, corrected here 2026-09-01), and
;; minting one is a language-surface addition, the builder's ruling, not this stone's. So this
;; file drops from five HOF probes to four; there is currently no rete-spelled right fold to
;; probe here at all.

;; ⛔ `:probe-map` / `:probe-filter` RE-POINTED at `mapv`/`filterv`, 2026-09-01 — and the two
;; lines below are NOT the old bodies with a renamed head. What was here named
;; `:wat::rete::core::map` and `:wat::rete::core::filter`, which have NEVER been RETE_OPS rows:
;; the rete surface's eager/lazy split (`src/rete/vocabulary.rs`, 2026-08-28) took `mapv`/`filterv`
;; because `map`/`filter` return a lazy `Stream` and a compiled `where` fence has no stream
;; machinery. So these two probes proved nothing for three months — an unforced `def` body is
;; never resolved, so the invented heads type-checked and this file loaded green.
;;
;; They are re-pointed rather than deleted because `mapv`/`filterv` ARE the Redispatch HOF rows
;; this file exists to sanity-check, so the re-pointed probes prove exactly what the header
;; claims. The `(:wat::core::into (:wat::core::PersistentVector) …)` wrapper is GONE with them:
;; it was there to force a lazy `Stream` to a Vector, and the whole reason rete took the `v`
;; forms is that they are already eager — `wat/seq.wat`: "the eager forms". Wrapping an eager
;; Vector in `into` would misdescribe the row under test.
;;
;; The container is a `Vector` literal, not a `PersistentVector`: `wat/seq.wat`'s `filterv` clauses
;; are `(Vector :- [T])` and `(Stream :- [T])` only. Measured, not assumed — the first re-point kept
;; the old `PersistentVector` argument and `every_wat_scripts_file_loads` went RED on it with
;; `NoMatchingClauseAtCallSite`. That red is itself the point: with a head that RESOLVES, the loader
;; gate finally has something to check, which it never did while the head was invented.
;;
;; The HOF count is unchanged at four: `foldl` · `mapv` · `filterv` · `reduce`.
(def :probe-mapv
  (:wat::rete::core::mapv
    (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::* x 2))
    [1 2 3]))

(def :probe-filterv
  (:wat::rete::core::filterv
    (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::core::i64::> x 1))
    [1 2 3]))

;; `reduce` is a wat-level `defclause` (`wat/seq.wat`), not a checker special form like its
;; three siblings above (foldl/mapv/filterv) — its rete row re-dispatches by AST head-substitution
;; into the SAME defclause-dispatch machinery a core-spelled call reaches (see check.rs's
;; infer_rete_form, the `:wat::core::reduce` arm).
(def :probe-reduce
  (:wat::rete::core::reduce
    (:wat::core::fn [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64 (:wat::core::i64::+ acc x))
    0
    (:wat::core::PersistentVector 1 2 3)))
