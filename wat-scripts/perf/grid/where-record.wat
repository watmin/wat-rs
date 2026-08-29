;; wat-scripts/perf/grid/where-record.wat — the RECORDS-AND-ACCESSOR-CHAINS family of the
;; `where`-expressivity corpus, wat side. Twin of where-record.clj. Copied in SHAPE from
;; where-shapes.wat/.clj (read BOTH before touching this file) per
;; docs/arc/2026/06/278-rules-engine/BRIEF-where-corpus-families.md.
;;
;; FAMILY: records and accessor chains. where-shapes.wat row 2 already landed the single FLAT
;; accessor (`Client/rep`). This family starts past that — it is about the DEPTH and SHAPE of data
;; reached THROUGH a bound var: multi-level accessor chains, a record field that is itself a
;; collection, a record-holding-a-record-holding-a-collection, two chains off one var compared to
;; each other, chains off two different vars compared to each other, a pure fn taking the whole
;; record vs the caller reaching in and handing over a scalar, an enum/variant field matched inside
;; a `where`, and an Option-typed field matched inside a `where`.
;;
;; ── HOW IT RUNS ───────────────────────────────────────────────────────────────────────────────
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-record.wat   > /tmp/ours
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-record.clj              > /tmp/theirs
;;     diff /tmp/ours /tmp/theirs        # empty ⇒ every row agrees
;;
;; `check-where-shapes.sh where-record` is that, wrapped.
;;
;; ── THE FOUR RULES (see BRIEF-where-corpus-families.md) ──────────────────────────────────────
;;
;; 1. THE SHARED CONDITION BINDS EVERY FIELD (?k ?c ?c2 ?st ?nt), identical in every rule; only the
;;    trailing `where` differs per row.
;; 2. EVERY ROW MUST DISCRIMINATE A PROPER SUBSET — 0 < |derived| < items. Expected counts below are
;;    from a hand-run simulation of the EXACT seed formulas (not guessed), and are checked against
;;    what each row actually emits (rule 2 of the four).
;; 3. SEED FROM A FORMULA OVER `i`, NEVER A DATA TABLE — both engines compute the identical stream
;;    independently via `:wat::core::i64::mod` / Clojure `mod` (flooring on both sides, validated
;;    equivalent in the arc-278 numeric-tower stone — see where-multivar.wat's note).
;; 4. MIRROR THE OPERATION, DO NOT IDIOMATISE IT — every predicate is written the same way on both
;;    sides; enum/Option construction and `match` are the SAME vocabulary shape in wat and Clara
;;    (`match` <-> `cond`+accessor, spelled out per row rather than idiomatised).
;;
;; ── THE RECORD SHAPE ──────────────────────────────────────────────────────────────────────────
;;
;;   L4 [v]                                   — innermost scalar holder
;;   L3 [l4 <- L4, w]                         — one level up
;;   L2 [l3 <- L3, u]                         — two levels up
;;   Bag [items <- (Vector :- [i64]), label]        — a record whose field IS a collection
;;   Client [l2 <- L2, rep, tags <- (Vector :- [i64]), bag <- Bag]
;;   Status  — enum: Active[level] | Inactive | Pending[reason]
;;   Req [k, client <- Client, client2 <- Client, status <- Status, note <- (Option :- [i64])]
;;
;; `Client/l2 -> L2/l3 -> L3/l4 -> L4/v` is a 4-level accessor chain reached off ONE bound var.
;; `client2` is a SECOND, independently-seeded Client on the same Req, for the cross-var rows.
;;
;; ── SEED (rule 3: formula over i, never a table) — items = 200 ──────────────────────────────────
;;   v4(i)  = i mod 9                     w3(i)  = (i mod 11) + 1          u2(i) = i mod 13
;;   rep(i) = (i mod 5) - 2               tagslen(i) = i mod 5             bagitemslen(i) = i mod 4
;;   j(i)   = (i + 97) mod 200            — client2(i) := client-of(j(i)), same formulas, shifted i
;;   status(i): m = i mod 3;  m=0 -> Active(i mod 5);  m=1 -> Inactive;  m=2 -> Pending(i mod 4)
;;   note(i): (i mod 4) == 0 -> None,  else Some(i mod 6)
;;
;; ── ROWS (8-15 required; 13 landed) — counts from /tmp/sim.sh, an exact integer simulation of the
;;    formulas above (not hand arithmetic), then checked against this program's own `n=` ─────────
;;    1  2-level chain   : u2(client)      > 8                          -> 60 of 200
;;    2  3-level chain   : w3(client)      > 7                          -> 72 of 200
;;    3  4-level chain   : v4(client)      > 5                          -> 66 of 200
;;    4  collection field: length(tags)    > 2                          -> 80 of 200
;;    5  record-in-record collection: length(bag.items) > 1             -> 100 of 200
;;    6  SAME var, two different chains compared: rep(c) > v4-chain(c)  -> 15 of 200
;;    7  TWO vars, same one-level chain compared: rep(c) > rep(c2)      -> 80 of 200
;;    8  pure fn takes the WHOLE record: rep-pos?(c)                    -> 80 of 200
;;    9  caller reaches in, passes a SCALAR: pos?(rep(c))  — contrast with row 8, SAME count/set
;;   10  enum/variant field, matched inside `where`: is-risky?(status)  -> 46 of 200
;;   11  Option-typed field, matched inside `where`: note-positive?(nt)-> 82 of 200
;;   12  combined: deep chain AND shallow field, same var               -> 44 of 200
;;   13  TWO vars, the SAME 2-level chain compared: u2-chain(c) > u2-chain(c2) -> 55 of 200

(:wat::core::defn :wr::items [] -> :wat::core::i64 200)   ;; the stream size, both sides

(:wat::core::defn :wr::row-count [] -> :wat::core::i64 13)

(:wat::core::defrecord :wr::L4 [v <- :wat::core::i64])
(:wat::core::defrecord :wr::L3 [l4 <- :wr::L4  w <- :wat::core::i64])
(:wat::core::defrecord :wr::L2 [l3 <- :wr::L3  u <- :wat::core::i64])
(:wat::core::defrecord :wr::Bag
  [items <- (:wat::core::PersistentVector :- [:wat::core::i64])
   label <- :wat::core::String])
(:wat::core::defrecord :wr::Client
  [l2   <- :wr::L2
   rep  <- :wat::core::i64
   tags <- (:wat::core::PersistentVector :- [:wat::core::i64])
   bag  <- :wr::Bag])

;; row 10's field type — Active carries a level, Pending carries a reason, Inactive is a unit variant.
(:wat::core::defenum :wr::Status :wat::enum::Pure
  :Active  [level <- :wat::core::i64]
  :Inactive
  :Pending [reason <- :wat::core::i64])

(:wat::core::defrecord :wr::Req
  [k       <- :wat::core::i64
   client  <- :wr::Client
   client2 <- :wr::Client
   status  <- :wr::Status
   note    <- (:wat::core::Option :- [:wat::core::i64])])

(:wat::core::defrecord :wr::Hit [k <- :wat::core::i64])   ;; the single production type

;; ── client-of i — builds ONE Client from the four scalar formulas plus the two collection fields.
;; Used TWICE per Req: once for `client` (i), once for `client2` (j(i)) — same constructor, so the
;; two Clients are structurally identical shapes seeded from different indices, never a hand-synced
;; second table.
(:wat::core::defn :wr::client-of [i <- :wat::core::i64] -> :wr::Client
  (:wat::core::let
    [v4          (:wat::i64::mod i 9)
     w3          (:wat::i64::+ (:wat::i64::mod i 11) 1)
     u2          (:wat::i64::mod i 13)
     rep         (:wat::i64::- (:wat::i64::mod i 5) 2)
     tagslen     (:wat::i64::mod i 5)
     bagitemslen (:wat::i64::mod i 4)
     l4          (:wr::L4 :v v4)
     l3          (:wr::L3 :l4 l4 :w w3)
     l2          (:wr::L2 :l3 l3 :u u2)
     tags        (:wat::core::into (:wat::core::PersistentVector)
                   (:wat::core::into (:wat::core::Vector :- [:wat::core::i64]) (:wat::core::range 0 tagslen)))
     bagitems    (:wat::core::into (:wat::core::PersistentVector)
                   (:wat::core::into (:wat::core::Vector :- [:wat::core::i64]) (:wat::core::range 0 bagitemslen)))
     bag         (:wr::Bag :items bagitems :label (:wat::string::concat "b" (:wat::i64::to-string i)))]
    (:wr::Client :l2 l2 :rep rep :tags tags :bag bag)))

;; row 10's field-value builder. Active/Pending are TAGGED variants, constructed POSITIONALLY
;; (`(:wr::Status::Active level)`), mirroring tests/types/enums_tagged_variant.wat; Inactive is a
;; unit variant referenced BARE (`:wr::Status::Inactive`), mirroring :wat::program::PeerKind::thread.
(:wat::core::defn :wr::status-of [i <- :wat::core::i64] -> :wr::Status
  (:wat::core::let [m (:wat::i64::mod i 3)]
    (:wat::core::cond
      ((:wat::core::= m 0) (:wr::Status::Active  (:wat::i64::mod i 5)))
      ((:wat::core::= m 1) :wr::Status::Inactive)
      (:else               (:wr::Status::Pending (:wat::i64::mod i 4))))))

;; row 11's field-value builder. None is bare (mirrors :wat::core::None used bare elsewhere); Some
;; wraps a value positionally.
(:wat::core::defn :wr::note-of [i <- :wat::core::i64] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::core::let [nm (:wat::i64::mod i 4)]
    (:wat::core::if (:wat::core::= nm 0)
      :wat::core::None
      (:wat::core::Some (:wat::i64::mod i 6)))))

;; row 8's whole-record fn: takes the Client itself and reaches inside it.
(:wat::rete::core::defn :wr::rep-pos? [c <- :wr::Client] -> :wat::core::bool
  (:wat::rete::i64::> (:wr::Client/rep c) 0))

;; row 9's contrast: the SAME constraint, but the caller reaches in and hands over a bare scalar.
(:wat::rete::core::defn :wr::pos? [x <- :wat::core::i64] -> :wat::core::bool
  (:wat::rete::i64::> x 0))

;; row 10's predicate over the enum field — `match` over a user-defined enum, called from `where`.
(:wat::rete::core::defn :wr::is-risky? [st <- :wr::Status] -> :wat::core::bool
  (:wat::rete::core::match st
    ((:wr::Status::Active lvl)    (:wat::rete::i64::> lvl 3))
    (:wr::Status::Inactive        false)
    ((:wr::Status::Pending reason) (:wat::rete::i64::> reason 1))))

;; row 11's predicate over the Option field — `match` over Some/None, called from `where`.
(:wat::rete::core::defn :wr::note-positive? [nt <- (:wat::core::Option :- [:wat::core::i64])] -> :wat::core::bool
  (:wat::rete::core::match nt
    ((:wat::core::Some v) (:wat::rete::i64::> v 2))
    (:wat::core::None     false)))

;; THE SHARED LEADING CONDITION, quoted once and reused by every row — only `where-c` varies.
(:wat::core::defn :wr::conds [] -> :wat::WatAST
  (:wat::core::quasiquote
    (:wr::Req (?k <- :k) (?c <- :client) (?c2 <- :client2) (?st <- :status) (?nt <- :note))))

(:wat::core::defn :wr::ins [] -> :wat::WatAST
  (:wat::core::quasiquote (:wr::Hit ?k)))

;; ROW 1 — 2-level accessor chain. u2(i) > 8 <=> i mod 13 in {9,10,11,12} -> 60 of 200.
(:wat::rete::defrule :wr::chain2
  :when
  [(:wr::Req (?k <- :k) (?c <- :client) (?c2 <- :client2) (?st <- :status) (?nt <- :note)) (:wat::rete::where (:wat::rete::i64::> (:wr::L2/u (:wr::Client/l2 ?c)) 8))]
  :then
  [(:wr::Hit ?k)])

;; ROW 2 — 3-level accessor chain. w3(i) > 7 <=> i mod 11 in {7,8,9,10} -> 72 of 200.
(:wat::rete::defrule :wr::chain3
  :when
  [(:wr::Req (?k <- :k) (?c <- :client) (?c2 <- :client2) (?st <- :status) (?nt <- :note)) (:wat::rete::where
                 (:wat::rete::i64::> (:wr::L3/w (:wr::L2/l3 (:wr::Client/l2 ?c))) 7))]
  :then
  [(:wr::Hit ?k)])

;; ROW 3 — 4-level accessor chain. v4(i) > 5 <=> i mod 9 in {6,7,8} -> 66 of 200.
(:wat::rete::defrule :wr::chain4
  :when
  [(:wr::Req (?k <- :k) (?c <- :client) (?c2 <- :client2) (?st <- :status) (?nt <- :note)) (:wat::rete::where
                 (:wat::rete::i64::>
                   (:wr::L4/v (:wr::L3/l4 (:wr::L2/l3 (:wr::Client/l2 ?c))))
                   5))]
  :then
  [(:wr::Hit ?k)])

;; ROW 4 — a record field that IS a collection, reached and then measured. tagslen(i) > 2
;; <=> i mod 5 in {3,4} -> 80 of 200.
(:wat::rete::defrule :wr::collection
  :when
  [(:wr::Req (?k <- :k) (?c <- :client) (?c2 <- :client2) (?st <- :status) (?nt <- :note)) (:wat::rete::where
                 (:wat::rete::i64::> (:wat::rete::vector::length (:wr::Client/tags ?c)) 2))]
  :then
  [(:wr::Hit ?k)])

;; ROW 5 — a record field that holds ANOTHER RECORD holding a collection: Client/bag -> Bag/items.
;; bagitemslen(i) > 1 <=> i mod 4 in {2,3} -> 100 of 200.
(:wat::rete::defrule :wr::record-collection
  :when
  [(:wr::Req (?k <- :k) (?c <- :client) (?c2 <- :client2) (?st <- :status) (?nt <- :note)) (:wat::rete::where
                 (:wat::rete::i64::>
                   (:wat::rete::vector::length (:wr::Bag/items (:wr::Client/bag ?c)))
                   1))]
  :then
  [(:wr::Hit ?k)])

;; ROW 6 — the SAME bound var, TWO DIFFERENT accessor chains, compared to each other:
;; rep(c) > v4-via-4-level-chain(c). rep in [-2,2], v4-chain in [0,8] -> 15 of 200.
(:wat::rete::defrule :wr::same-var-two-chains
  :when
  [(:wr::Req (?k <- :k) (?c <- :client) (?c2 <- :client2) (?st <- :status) (?nt <- :note)) (:wat::rete::where
                 (:wat::rete::i64::>
                   (:wr::Client/rep ?c)
                   (:wr::L4/v (:wr::L3/l4 (:wr::L2/l3 (:wr::Client/l2 ?c))))))]
  :then
  [(:wr::Hit ?k)])

;; ROW 7 — TWO DIFFERENT bound vars (?c and ?c2), the SAME one-level accessor chain off each,
;; compared to each other: rep(c) > rep(c2). rep(i) > rep(j(i)) -> 80 of 200.
(:wat::rete::defrule :wr::cross-var-scalar
  :when
  [(:wr::Req (?k <- :k) (?c <- :client) (?c2 <- :client2) (?st <- :status) (?nt <- :note)) (:wat::rete::where (:wat::rete::i64::> (:wr::Client/rep ?c) (:wr::Client/rep ?c2)))]
  :then
  [(:wr::Hit ?k)])

;; ROW 8 — a PURE FN taking the WHOLE RECORD and reaching inside it: (rep-pos? ?c).
;; rep(i) > 0 <=> i mod 5 in {3,4} -> 80 of 200.
(:wat::rete::defrule :wr::whole-record-fn
  :when
  [(:wr::Req (?k <- :k) (?c <- :client) (?c2 <- :client2) (?st <- :status) (?nt <- :note)) (:wat::rete::where (:wr::rep-pos? ?c))]
  :then
  [(:wr::Hit ?k)])

;; ROW 9 — the CONTRAST with row 8: the CALLER reaches in and hands the fn a bare SCALAR:
;; (pos? (Client/rep ?c)). Same constraint, same derived set as row 8 (80 of 200) — the point is the
;; CALL SHAPE (record-arg vs scalar-arg), which a compiler treats very differently, not the count.
(:wat::rete::defrule :wr::scalar-fn
  :when
  [(:wr::Req (?k <- :k) (?c <- :client) (?c2 <- :client2) (?st <- :status) (?nt <- :note)) (:wat::rete::where (:wr::pos? (:wr::Client/rep ?c)))]
  :then
  [(:wr::Hit ?k)])

;; ROW 10 — an ENUM/VARIANT field, matched inside a `where`: (is-risky? ?st).
;; m=i mod 3: m=0 needs level>3 (i mod 5 in {4}), m=1 never, m=2 needs reason>1 (i mod 4 in {2,3})
;; -> 46 of 200.
(:wat::rete::defrule :wr::enum-match
  :when
  [(:wr::Req (?k <- :k) (?c <- :client) (?c2 <- :client2) (?st <- :status) (?nt <- :note)) (:wat::rete::where (:wr::is-risky? ?st))]
  :then
  [(:wr::Hit ?k)])

;; ROW 11 — an OPTION-TYPED field, matched inside a `where`: (note-positive? ?nt).
;; (i mod 4)==0 -> None (never); else Some(i mod 6), positive needs i mod 6 in {3,4,5} -> 82 of 200.
(:wat::rete::defrule :wr::option-match
  :when
  [(:wr::Req (?k <- :k) (?c <- :client) (?c2 <- :client2) (?st <- :status) (?nt <- :note)) (:wat::rete::where (:wr::note-positive? ?nt))]
  :then
  [(:wr::Hit ?k)])

;; ROW 12 — COMBINED: a deep chain AND a shallow field, same var, joined with `and`.
;; rep(i) > 0 AND v4(i) > 3 -> 44 of 200.
(:wat::rete::defrule :wr::combined-and
  :when
  [(:wr::Req (?k <- :k) (?c <- :client) (?c2 <- :client2) (?st <- :status) (?nt <- :note)) (:wat::rete::where
                 (:wat::rete::core::and
                   (:wat::rete::i64::> (:wr::Client/rep ?c) 0)
                   (:wat::rete::i64::>
                     (:wr::L4/v (:wr::L3/l4 (:wr::L2/l3 (:wr::Client/l2 ?c))))
                     3)))]
  :then
  [(:wr::Hit ?k)])

;; ROW 13 — TWO DIFFERENT bound vars, the SAME 2-level accessor chain off each, compared:
;; u2-chain(c) > u2-chain(c2). u2(i) > u2(j(i)) -> 55 of 200.
(:wat::rete::defrule :wr::cross-var-chain
  :when
  [(:wr::Req (?k <- :k) (?c <- :client) (?c2 <- :client2) (?st <- :status) (?nt <- :note)) (:wat::rete::where
                 (:wat::rete::i64::>
                   (:wr::L2/u (:wr::Client/l2 ?c))
                   (:wr::L2/u (:wr::Client/l2 ?c2))))]
  :then
  [(:wr::Hit ?k)])

(:wat::rete::defquery :wr::q-Hit
  :params []
  :when [(?fact <- :wr::Hit)])


;; build-rules row — THE ROW DISPATCH. An unknown row is a located failure, never a silent fallback.
(:wat::core::defn :wr::build-rules [row <- :wat::core::i64] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:wat::core::cond
      ((:wat::core::= row 1)  (:wr::chain2))
      ((:wat::core::= row 2)  (:wr::chain3))
      ((:wat::core::= row 3)  (:wr::chain4))
      ((:wat::core::= row 4)  (:wr::collection))
      ((:wat::core::= row 5)  (:wr::record-collection))
      ((:wat::core::= row 6)  (:wr::same-var-two-chains))
      ((:wat::core::= row 7)  (:wr::cross-var-scalar))
      ((:wat::core::= row 8)  (:wr::whole-record-fn))
      ((:wat::core::= row 9)  (:wr::scalar-fn))
      ((:wat::core::= row 10) (:wr::enum-match))
      ((:wat::core::= row 11) (:wr::option-match))
      ((:wat::core::= row 12) (:wr::combined-and))
      ((:wat::core::= row 13) (:wr::cross-var-chain))
      (:else
        (:wat::kernel::assertion-failed!
          (:wat::string::concat "where-record: unknown row " (:wat::i64::to-string row))
          :wat::core::None :wat::core::None)))))

;; seed session items — stage Req(i) for i in [0, items) via the BATCH verb (one rebuild).
(:wat::core::defn :wr::seed [session <- :wat::rete::Session  items <- :wat::core::i64] -> :wat::rete::Session
  (:wat::rete::insert-all
    session
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::PersistentVector :- [:wat::core::Record])  i <- :wat::core::i64]
                      -> (:wat::core::PersistentVector :- [:wat::core::Record])
        (:wat::core::let [j (:wat::i64::mod (:wat::i64::+ i 97) items)]
          (:wat::vector::conj acc
            (:wr::Req :k i
              :client  (:wr::client-of i)
              :client2 (:wr::client-of j)
              :status  (:wr::status-of i)
              :note    (:wr::note-of i)))))
      (:wat::core::PersistentVector)
      (:wat::core::range 0 items))))

;; derived-ints fired — every derived Hit's key k, sorted ascending. THE accuracy witness.
(:wat::core::defn :wr::derived-ints
  [fired <- :wat::rete::Session] -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::sort
    (:wat::core::into (:wat::core::Vector :- [:wat::core::i64])
      (:wat::core::map
        (:wat::core::fn [p <- :wat::core::PersistentMap] -> :wat::core::i64 (:wat::core::let [f (:wat::core::Option/expect (:wat::map::get p "?fact") "query: ?fact")] (:wr::Hit/k f)))
        (:wat::rete::query fired (:wr::q-Hit))))))

;; render-ints — " 3 13 23 …". A plain space-joined rendering, NOT the EDN printer — see
;; where-shapes.wat's identical helper for why this must not be `:wat::edn::write`.
(:wat::core::defn :wr::render-ints [v <- (:wat::core::Vector :- [:wat::core::i64])] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  x <- :wat::core::i64] -> :wat::core::String
      (:wat::string::concat acc
        (:wat::string::concat " " (:wat::i64::to-string x))))
    ""
    v))

;; run-row row -> the corpus line for ONE shape, in its OWN session (mirrors where-shapes.wat's
;; per-row isolation, so a divergence names the row that caused it).
;; rule-display-name — TOTAL derivation of the printed row label from a Rule/name that may
;; now carry this file's namespace prefix (e.g. "NS::arith") after the namespacing wall.
;; `string::split` on "::" always returns >= 1 segment (the whole string, unsplit, when
;; "::" is absent); folding with SEED = full while always overwriting the accumulator
;; with the current segment lands on the LAST segment without ever calling a partial
;; verb (`first`/`nth`/`Option/expect`) — the seed also makes the no-"::" case return
;; the input UNCHANGED, and even an impossible empty split falls back to the seed
;; instead of raising.
(:wat::core::defn :wr::rule-display-name
  [full <- :wat::core::String] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  seg <- :wat::core::String] -> :wat::core::String seg)
    full
    (:wat::string::split full "::")))

(:wat::core::defn :wr::run-row [row <- :wat::core::i64] -> :wat::core::String
  (:wat::core::let
    [rules   (:wr::build-rules row)
     rule    (:wat::core::first rules)
     staged  (:wr::seed (:wat::rete::compile-all rules (:wat::core::PersistentVector (:wr::q-Hit))) (:wr::items))
     fired   (:wat::rete::fire-rules staged)
     derived (:wr::derived-ints fired)
     n       (:wat::vec::length derived)]
    (:wat::string::concat
      (:wat::string::concat
        (:wat::string::concat "row " (:wat::i64::to-string row))
        (:wat::string::concat " " (:wr::rule-display-name (:wat::rete::Rule/name rule))))
      (:wat::string::concat
        (:wat::string::concat " n=" (:wat::i64::to-string n))
        (:wat::string::concat " ->" (:wr::render-ints derived))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::nil  row <- :wat::core::i64] -> :wat::core::nil
      (:wat::kernel::println (:wr::run-row row)))
    nil
    (:wat::core::range 1 (:wat::i64::+ (:wr::row-count) 1))))
