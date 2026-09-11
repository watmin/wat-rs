;; Fixture BESIDE probe_arc278_fence_binder_shadow.rs — THE OVER-REJECTION CONTROL.
;;
;; ⛔ LOAD-BEARING (EXPECTATIONS.md's ⭐ row): the ONE real `let`-in-a-fence in the whole corpus,
;; `wat-scripts/perf/grid/where-inline-computed.wat:166` —
;;   (:wat::rete::where (:wat::rete::core::let [x ?k] (:wat::rete::core::i64::> x 100)))
;; — is a PLAIN binder `x` holding a rete variable's VALUE. A cure that refuses ANY binder, not
;; only a `?`-prefixed one, breaks this. Row 1 below reproduces it exactly.
;;
;; Rows 2-3 reproduce the OTHER plain-binder shapes the re-derived corpus census found actually
;; live in `wat-scripts/perf/grid/where-control.wat` (a DESIGN.md count this strike corrected —
;; see SCORE.md): a `let` binder feeding a boolean composition, and a `match` arm's VARIANT
;; PAYLOAD binder (`(:wat::core::Some v)`) — the deep-scan case `check_match_pattern_for_shadow`
;; exists to still admit, since `v` is plain.

(:wat::core::defrecord :fbso::N [k <- :wat::core::i64 o <- (:wat::core::Option :- [:wat::core::i64])])

;; row 1 — the real corpus shape: plain binder holding a rete var's VALUE.
(:wat::rete::defrule :fbso::plain-let-value
  :when [(:fbso::N (?k <- :k))
         (:wat::rete::where (:wat::rete::core::let [x ?k] (:wat::rete::core::i64::> x 100)))]
  :then [])

;; row 2 — a plain `let` binder feeding a boolean composition (where-control.wat's `let-twice`).
(:wat::rete::defrule :fbso::plain-let-bool
  :when [(:fbso::N (?k <- :k))
         (:wat::rete::where
           (:wat::rete::core::let [s (:wat::rete::core::i64::+ ?k 1 :undefined 0)]
             (:wat::rete::core::and
               (:wat::rete::core::i64::> s 0)
               (:wat::rete::core::i64::< s 1000))))]
  :then [])

;; row 3 — a `match` arm's variant PAYLOAD binder is plain (`v`), not `?`-prefixed. Must still
;; compile: the deep pattern scan refuses only a `?`-prefixed name, at any depth, not every name.
(:wat::rete::defrule :fbso::plain-match-payload
  :when [(:fbso::N (?o <- :o))
         (:wat::rete::where
           (:wat::rete::core::match ?o
             ((:wat::core::Some v) (:wat::rete::core::i64::> v 0))
             (:wat::core::None false)))]
  :then [])
