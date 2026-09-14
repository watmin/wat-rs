;; Fixture BESIDE probe_arc278_accumulate_divergences.rs.
;; TWO live native-vs-$oracle divergences, both found by the rete fuzzer
;; (wat-scripts/fuzz/rete-differential.wat) on its first widened run, both in the
;; ACCUMULATE family, both silent.

(:wat::core::defrecord :user::W  [k <- :wat::core::i64])
(:wat::core::defrecord :user::P1 [k <- :wat::core::i64])
(:wat::core::defrecord :user::S1 [k <- :wat::core::i64])
(:wat::core::defrecord :user::S2 [k <- :wat::core::i64])
(:wat::core::defrecord :user::S3 [k <- :wat::core::i64])

;; ── B — a SECOND `where` after an accumulate matches NOTHING ────────────────
;; qB1 (one where) agrees at 1. qB2 differs ONLY by a trailing, trivially-true
;; second where — and native drops to 0 while the oracle holds at 1.
(:wat::rete::defquery :user::qB1 :params []
  :when [(:user::P1 (?a <- :k))
         (?n <- (:wat::rete::acc::count) :from (:user::W))
         (:wat::rete::where (:wat::rete::i64::>= ?n 2))])

(:wat::rete::defquery :user::qB2 :params []
  :when [(:user::P1 (?a <- :k))
         (?n <- (:wat::rete::acc::count) :from (:user::W))
         (:wat::rete::where (:wat::rete::i64::>= ?n 2))
         (:wat::rete::where (:wat::rete::i64::> 1 0))])

;; ── A — a LEADING accumulate emits one row per FIXPOINT ROUND ───────────────
;; The chain is inert: it derives S2/S3 and touches nothing the query reads. Its
;; only role is to make the fixpoint iterate. Rows track that count exactly.
(:wat::rete::defrule :user::r1 :when [(:user::S1 (?k <- :k))] :then [(:user::S2 :k ?k)])
(:wat::rete::defrule :user::r2 :when [(:user::S2 (?k <- :k))] :then [(:user::S3 :k ?k)])

(:wat::rete::defquery :user::qA :params []
  :when [(?n <- (:wat::rete::acc::count) :from (:user::W))
         (:wat::rete::where (:wat::rete::i64::>= ?n 2))])

;; ── C — `:not` over a DERIVED class ignores the derivation ─────────────────
;; STRATIFIED NEGATION. `S2` exists only because `r1` derives it from `S1`, so a
;; `not S2` must BLOCK once the chain has run. The oracle blocks (0). Native passes
;; (1) — while its OWN query confirms S2 is present. Both engines derive the fact;
;; only one of them lets the negation see it.
(:wat::rete::defquery :user::qC :params []
  :when [(:wat::rete::not (:user::S2))])

;; The control that makes the claim airtight: is S2 actually there?
(:wat::rete::defquery :user::qS2 :params []
  :when [(?fact <- :user::S2)])

(:wat::core::defn :user::two
  [q <- :wat::rete::Query  rules <- (:wat::core::PersistentVector :- [:wat::rete::Rule])]
  -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::let [s0 (:wat::rete::compile-all rules (:wat::core::PersistentVector q))
                    s1 (:wat::rete::insert-all s0 (:wat::core::PersistentVector (:user::W 7) (:user::W 7)))
                    s2 (:wat::rete::insert-all s1 (:wat::core::PersistentVector (:user::P1 1)))
                    st (:wat::rete::insert-all s2 (:wat::core::PersistentVector (:user::S1 1)))
                    nf (:wat::rete::fire-rules st)
                    of (:wat::rete::fire-rules$oracle st)]
    (:wat::core::mapv
      (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64 n)
      (:wat::core::PersistentVector
        (:wat::core::length (:wat::rete::query nf q))
        (:wat::core::length (:wat::rete::query of q))))))

(:wat::core::defn :user::norules [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector))

(:wat::core::defn :user::one-rule [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:wat::rete::Rule :name "r1"
      :lhs (:wat::core::PersistentVector (:wat::core::quasiquote (:user::S1 (?k <- :k))))
      :rhs (:wat::core::PersistentVector (:wat::core::quasiquote (:user::S2 ?k))))))

;; [B1-native B1-oracle  B2-native B2-oracle  A2-native A2-oracle  A3-native A3-oracle]
;; A2 = 1-rule chain (2 rounds), A3 = 2-rule chain (3 rounds).
(:wat::core::defn :user::rows [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::into
    (:wat::core::into
      (:wat::core::into (:user::two (:user::qB1) (:user::norules))
                        (:user::two (:user::qB2) (:user::norules)))
      (:user::two (:user::qA) (:user::one-rule)))
    (:user::two (:user::qA) (:wat::rete::collect-rules :user))))

;; [C-noChain-native C-noChain-oracle | C-chain-native C-chain-oracle | S2-native S2-oracle]
;; With no chain S2 is absent and both must pass (1,1). With the chain S2 exists
;; and both must block (0,0). The last pair proves S2 really is derived in BOTH.
(:wat::core::defn :user::rows-c [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::let [both (:wat::core::PersistentVector (:user::qC) (:user::qS2))
                    s0 (:wat::rete::compile-all (:user::one-rule) both)
                    st (:wat::rete::insert-all s0 (:wat::core::PersistentVector (:user::S1 1)))
                    nf (:wat::rete::fire-rules st)
                    of (:wat::rete::fire-rules$oracle st)
                    n0 (:wat::rete::compile-all (:user::norules) both)
                    t0 (:wat::rete::insert-all n0 (:wat::core::PersistentVector (:user::S1 1)))
                    nf0 (:wat::rete::fire-rules t0)
                    of0 (:wat::rete::fire-rules$oracle t0)]
    (:wat::core::mapv
      (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::i64 n)
      (:wat::core::PersistentVector
        (:wat::core::length (:wat::rete::query nf0 (:user::qC)))
        (:wat::core::length (:wat::rete::query of0 (:user::qC)))
        (:wat::core::length (:wat::rete::query nf (:user::qC)))
        (:wat::core::length (:wat::rete::query of (:user::qC)))
        (:wat::core::length (:wat::rete::query nf (:user::qS2)))
        (:wat::core::length (:wat::rete::query of (:user::qS2)))))))
