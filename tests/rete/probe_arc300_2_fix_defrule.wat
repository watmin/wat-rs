;; tests/rete/probe_arc300_2_fix_defrule.wat — arc 300.2: the fix conversion as PURE rete defrules.
;;
;; rete is ALWAYS pure in wat: rules DEDUCE, the deductions are QUERIED OUT and ACTIONED
;; outside rete. A :then NEVER transforms a value — it inserts a classification fact whose
;; fields are only ?var bindings (offset/len/name), which the v1 RHS resolver already handles.
;;
;; The transformation (keyword/to-symbol, keyword/to-type-form, ":-") and the I/O live in the
;; DRIVE (wat-scripts/fixes/to-faithful-clojure-rete.wat), OUTSIDE rete — the consumer's job.
;;
;; This fixture is the rete-firing unit test: assert :fix::Node facts, fire, and confirm the
;; three PURE classification facts (:fix::HeadConv / :fix::ArrowConv / :fix::TypeConv) deduce.

;; ── fact model ──────────────────────────────────────────────────────────────
;; Node — one per leaf AST node the walk visits (position-aware: post-arrow tracked).
(:wat::core::defrecord :fix::Node
  [kind       <- :wat::core::String
   name       <- :wat::core::String
   offset     <- :wat::core::i64
   len        <- :wat::core::i64
   post-arrow <- :wat::core::bool])

;; The three PURE classification facts — offset/len (+ name where the drive needs it to
;; reconstruct the keyword). No transformed text: the drive does keyword/to-symbol etc.
(:wat::core::defrecord :fix::HeadConv
  [offset <- :wat::core::i64
   len    <- :wat::core::i64
   name   <- :wat::core::String])

(:wat::core::defrecord :fix::ArrowConv
  [offset <- :wat::core::i64
   len    <- :wat::core::i64])

(:wat::core::defrecord :fix::TypeConv
  [offset <- :wat::core::i64
   len    <- :wat::core::i64
   name   <- :wat::core::String])

;; ── pure string predicates (used in :where guards) ──────────────────────────
;; head-keyword-str? — name string is ::-namespaced.
(:wat::rete::core::defn :fix::head-keyword-str?
  [name <- :wat::core::String] -> :wat::core::bool
  (:wat::rete::core::String/contains? name "::"))

;; type-shaped-keyword-str? — name has matching "<" + ">" OR "(" + ")".
(:wat::rete::core::defn :fix::type-shaped-keyword-str?
  [name <- :wat::core::String] -> :wat::core::bool
  (:wat::rete::core::if (:wat::rete::core::if (:wat::rete::core::String/contains? name "<")
                    (:wat::rete::core::String/contains? name ">")
                    false)
    true
    (:wat::rete::core::if (:wat::rete::core::String/contains? name "(")
      (:wat::rete::core::String/contains? name ")")
      false)))

;; ── the rules: each :then is PURE (bindings only, no transform) ──────────────

;; head-keyword→conv: kind=keyword ∧ contains "::" ∧ ¬post-arrow ∧ ¬type-shaped
;;   → deduce HeadConv(offset, len, name). The drive turns name into (keyword/to-symbol name).
(:wat::rete::defrule :fix::head-keyword->conv
  :when
  [(:fix::Node
     (?offset     <- :offset)
     (?len        <- :len)
     (?kind       <- :kind)
     (?name       <- :name)
     (?post-arrow <- :post-arrow)
     (:wat::rete::string::= ?kind "keyword"))
   (:wat::rete::where (:fix::head-keyword-str? ?name))
   (:wat::rete::where (:wat::rete::core::not ?post-arrow))
   (:wat::rete::where (:wat::rete::core::not (:fix::type-shaped-keyword-str? ?name)))]
  :then
  [(:fix::HeadConv :offset ?offset :len ?len :name ?name)])

;; arrow→conv: kind=symbol ∧ (name="<-" ∨ name="->") → deduce ArrowConv(offset, len).
;;   The drive emits the literal ":-".
(:wat::rete::defrule :fix::arrow->conv
  :when
  [(:fix::Node
     (?offset <- :offset)
     (?len    <- :len)
     (?kind   <- :kind)
     (?name   <- :name)
     (:wat::rete::string::= ?kind "symbol"))
   (:wat::rete::where (:wat::rete::core::or
                        (:wat::rete::string::= ?name "<-")
                        (:wat::rete::string::= ?name "->")))]
  :then
  [(:fix::ArrowConv :offset ?offset :len ?len)])

;; type-keyword→conv: kind=keyword ∧ (post-arrow ∨ type-shaped)
;;   → deduce TypeConv(offset, len, name). The drive turns name into (keyword/to-type-form name).
(:wat::rete::defrule :fix::type-keyword->conv
  :when
  [(:fix::Node
     (?offset     <- :offset)
     (?len        <- :len)
     (?kind       <- :kind)
     (?name       <- :name)
     (?post-arrow <- :post-arrow)
     (:wat::rete::string::= ?kind "keyword"))
   (:wat::rete::where (:wat::rete::core::or
                        ?post-arrow
                        (:fix::type-shaped-keyword-str? ?name)))]
  :then
  [(:fix::TypeConv :offset ?offset :len ?len :name ?name)])

(:wat::rete::defquery :fix::q-HeadConv
  :params []
  :when [(:fix::HeadConv (?offset <- :offset) (?len <- :len) (?name <- :name))])

(:wat::rete::defquery :fix::q-ArrowConv
  :params []
  :when [(:fix::ArrowConv (?offset <- :offset) (?len <- :len))])

(:wat::rete::defquery :fix::q-TypeConv
  :params []
  :when [(:fix::TypeConv (?offset <- :offset) (?len <- :len) (?name <- :name))])


;; ── per-scenario named entries — one asserted Node, fired, queried ────────────────
;; Each test's node literal and query tail are fixed and enumerable — no runtime parameterization.

;; head-keyword→conv: a head keyword, not post-arrow, not type-shaped.
(:wat::core::defn :user::head-keyword-count [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :fix)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fix::q-HeadConv) (:fix::q-ArrowConv) (:fix::q-TypeConv)))
     session (:wat::rete::insert session (:fix::Node :kind "keyword" :name ":wat::core::defrecord" :offset 1 :len 21 :post-arrow false))
     fired   (:wat::rete::fire-rules session)]
    (:wat::core::length (:wat::rete::query fired (:fix::q-HeadConv)))))

(:wat::core::defn :user::head-keyword-name [] -> :wat::core::String
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :fix)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fix::q-HeadConv) (:fix::q-ArrowConv) (:fix::q-TypeConv)))
     session (:wat::rete::insert session (:fix::Node :kind "keyword" :name ":wat::core::defrecord" :offset 1 :len 21 :post-arrow false))
     fired   (:wat::rete::fire-rules session)]
    (:wat::core::Option/expect
      (:wat::map::get
        (:wat::core::first (:wat::rete::query fired (:fix::q-HeadConv)))
        "?name")
      "q-HeadConv: ?name")))

(:wat::core::defn :user::head-keyword-offset [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :fix)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fix::q-HeadConv) (:fix::q-ArrowConv) (:fix::q-TypeConv)))
     session (:wat::rete::insert session (:fix::Node :kind "keyword" :name ":wat::core::defrecord" :offset 1 :len 21 :post-arrow false))
     fired   (:wat::rete::fire-rules session)]
    (:wat::core::Option/expect
      (:wat::map::get
        (:wat::core::first (:wat::rete::query fired (:fix::q-HeadConv)))
        "?offset")
      "q-HeadConv: ?offset")))

;; post-arrow=true → excluded from head-keyword→conv (the ¬post-arrow guard).
(:wat::core::defn :user::post-arrow-keyword-headconv-count [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :fix)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fix::q-HeadConv) (:fix::q-ArrowConv) (:fix::q-TypeConv)))
     session (:wat::rete::insert session (:fix::Node :kind "keyword" :name ":wat::core::String" :offset 10 :len 18 :post-arrow true))
     fired   (:wat::rete::fire-rules session)]
    (:wat::core::length (:wat::rete::query fired (:fix::q-HeadConv)))))

;; arrow→conv: "<-".
(:wat::core::defn :user::left-arrow-count [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :fix)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fix::q-HeadConv) (:fix::q-ArrowConv) (:fix::q-TypeConv)))
     session (:wat::rete::insert session (:fix::Node :kind "symbol" :name "<-" :offset 0 :len 2 :post-arrow false))
     fired   (:wat::rete::fire-rules session)]
    (:wat::core::length (:wat::rete::query fired (:fix::q-ArrowConv)))))

(:wat::core::defn :user::left-arrow-offset [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :fix)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fix::q-HeadConv) (:fix::q-ArrowConv) (:fix::q-TypeConv)))
     session (:wat::rete::insert session (:fix::Node :kind "symbol" :name "<-" :offset 0 :len 2 :post-arrow false))
     fired   (:wat::rete::fire-rules session)]
    (:wat::core::Option/expect
      (:wat::map::get
        (:wat::core::first (:wat::rete::query fired (:fix::q-ArrowConv)))
        "?offset")
      "q-ArrowConv: ?offset")))

;; arrow→conv: "->" also deduces an ArrowConv.
(:wat::core::defn :user::right-arrow-count [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :fix)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fix::q-HeadConv) (:fix::q-ArrowConv) (:fix::q-TypeConv)))
     session (:wat::rete::insert session (:fix::Node :kind "symbol" :name "->" :offset 5 :len 2 :post-arrow false))
     fired   (:wat::rete::fire-rules session)]
    (:wat::core::length (:wat::rete::query fired (:fix::q-ArrowConv)))))

;; a non-arrow symbol deduces nothing (neither ArrowConv nor HeadConv).
(:wat::core::defn :user::non-arrow-arrows-count [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :fix)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fix::q-HeadConv) (:fix::q-ArrowConv) (:fix::q-TypeConv)))
     session (:wat::rete::insert session (:fix::Node :kind "symbol" :name "path" :offset 10 :len 4 :post-arrow false))
     fired   (:wat::rete::fire-rules session)]
    (:wat::core::length (:wat::rete::query fired (:fix::q-ArrowConv)))))

(:wat::core::defn :user::non-arrow-heads-count [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :fix)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fix::q-HeadConv) (:fix::q-ArrowConv) (:fix::q-TypeConv)))
     session (:wat::rete::insert session (:fix::Node :kind "symbol" :name "path" :offset 10 :len 4 :post-arrow false))
     fired   (:wat::rete::fire-rules session)]
    (:wat::core::length (:wat::rete::query fired (:fix::q-HeadConv)))))

;; type-keyword→conv: post-arrow keyword (not type-shaped, but post-arrow) → TypeConv.
(:wat::core::defn :user::post-arrow-typeconv-count [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :fix)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fix::q-HeadConv) (:fix::q-ArrowConv) (:fix::q-TypeConv)))
     session (:wat::rete::insert session (:fix::Node :kind "keyword" :name ":wat::core::String" :offset 10 :len 18 :post-arrow true))
     fired   (:wat::rete::fire-rules session)]
    (:wat::core::length (:wat::rete::query fired (:fix::q-TypeConv)))))

(:wat::core::defn :user::post-arrow-typeconv-name [] -> :wat::core::String
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :fix)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fix::q-HeadConv) (:fix::q-ArrowConv) (:fix::q-TypeConv)))
     session (:wat::rete::insert session (:fix::Node :kind "keyword" :name ":wat::core::String" :offset 10 :len 18 :post-arrow true))
     fired   (:wat::rete::fire-rules session)]
    (:wat::core::Option/expect
      (:wat::map::get
        (:wat::core::first (:wat::rete::query fired (:fix::q-TypeConv)))
        "?name")
      "q-TypeConv: ?name")))

;; a structurally-type-shaped keyword (Vector<...>) is a TypeConv even at head position, and is
;; EXCLUDED from HeadConv (the ¬type-shaped guard) — no double edit.
(:wat::core::defn :user::type-shaped-typeconv-count [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :fix)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fix::q-HeadConv) (:fix::q-ArrowConv) (:fix::q-TypeConv)))
     session (:wat::rete::insert session (:fix::Node :kind "keyword" :name ":wat::core::Vector<wat::core::i64>" :offset 0 :len 30 :post-arrow false))
     fired   (:wat::rete::fire-rules session)]
    (:wat::core::length (:wat::rete::query fired (:fix::q-TypeConv)))))

(:wat::core::defn :user::type-shaped-headconv-count [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :fix)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:fix::q-HeadConv) (:fix::q-ArrowConv) (:fix::q-TypeConv)))
     session (:wat::rete::insert session (:fix::Node :kind "keyword" :name ":wat::core::Vector<wat::core::i64>" :offset 0 :len 30 :post-arrow false))
     fired   (:wat::rete::fire-rules session)]
    (:wat::core::length (:wat::rete::query fired (:fix::q-HeadConv)))))

