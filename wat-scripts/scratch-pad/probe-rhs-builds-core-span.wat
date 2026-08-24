;; PROBE — row 4 of the Span-fact stone, RE-RUN on a quiescent tree.
;;
;; Does a rete RHS construct a record with a NESTED record field, with LHS bindings flowing into
;; the nested constructor? Specifically: the user's RHS assembling a `:wat::core::Span` (which
;; carries `:file` — a property of the run, not of a node) from bound `?line`/`?col` plus a
;; filename supplied in the RHS itself. This is the assembly step the Span-fact stone's whole
;; ergonomics argument rests on — `:fx::Span` is flat because the USER reassembles the nested
;; record downstream, not the extractor.
;;
;; The predecessor of this probe ran beside a live writer mid-flight in the same tree (recorded
;; in DESIGN-STONE-the-span-fact.md) and is UNCREDITED for exactly that reason. This is the fresh,
;; quiescent-tree run. If the output differs from the recorded shape in any way, the difference is
;; the finding, not the old record.
;;
;; Recorded shape to reproduce:
;;   #p/Hit {:span #wat.core/Span {:file "a.wat" :line 7 :col 1 :end :wat.core/None} :why "…"}
;; `:end` = None; `:wat::core::Pos` is not reached (it is registered Rust-side, not a `defrecord`).

(:wat::core::defrecord :p::Loc [line <- :wat::core::i64  col <- :wat::core::i64])

(:wat::core::defrecord :p::Hit
  [span <- :wat::core::Span
   why  <- :wat::core::String])

;; the RHS: LHS binds ?l / ?c from a plain fact; the filename "a.wat" is supplied IN the RHS
;; (it is a property of the run, exactly as the DESIGN argues); :end is None (no Pos in play).
(:wat::rete::defrule :p::build-hit
  :when [(:p::Loc (?l <- :line) (?c <- :col))]
  :then [(:p::Hit
           :span (:wat::core::Span :file "a.wat" :line ?l :col ?c :end :wat::core::None)
           :why "nested Span construction from bound LHS vars")])

(:wat::rete::defquery :p::q-Hit
  :params []
  :when [(?fact <- :p::Hit)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules (:wat::core::PersistentVector (:p::build-hit))
     s0    (:wat::rete::insert
             (:wat::rete::compile-all rules (:wat::core::PersistentVector (:p::q-Hit)))
             (:p::Loc :line 7 :col 1))
     fired (:wat::rete::fire-rules s0)
     hits  (:wat::rete::query fired (:p::q-Hit))
     hit   (:wat::core::Option/expect
             (:wat::core::PersistentMap/get (:wat::core::first hits) "?fact")
             "q-Hit: ?fact")]
    (:wat::kernel::println hit)))
