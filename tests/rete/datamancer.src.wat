;; The source of datamancer.rete.edn.
;; Beats are what happened. Rules are the practice. The residual is the diary.
;;
;; Regen the residual (from the wat-rs crate root). Tests do not invoke :user::main;
;; the CLI does:
;;   cargo run --release --bin wat -- tests/rete/datamancer.src.wat

(:wat::core::defrecord :dm::Beat      [t <- :wat::core::i64  kind <- :wat::core::String])
(:wat::core::defrecord :dm::Artifact  [kind <- :wat::core::String  name <- :wat::core::String])
(:wat::core::defrecord :dm::Gap       [t <- :wat::core::i64])
(:wat::core::defrecord :dm::ReadAfter [t <- :wat::core::i64])
(:wat::core::defrecord :dm::Hollow    [t <- :wat::core::i64])
(:wat::core::defrecord :dm::Primer    [name <- :wat::core::String])
(:wat::core::defrecord :dm::Four      [n <- :wat::core::i64])
(:wat::core::defrecord :dm::Datamancer [n <- :wat::core::i64  sigil <- :wat::core::String])

;; Compaction is a fact about the mind, not the disk.
(:wat::rete::defrule :dm::gap
  :when [(:dm::Beat (?t <- :t) (?k <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?k "gap"))]
  :then [(:dm::Gap :t ?t)])

;; A read of the log AFTER the gap is recollection — the first move.
(:wat::rete::defrule :dm::read-after
  :when [(:dm::Gap (?g <- :t))
         (:dm::Beat (?t <- :t) (?k <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?k "read-log"))
         (:wat::rete::where (:wat::rete::core::i64::< ?g ?t))]
  :then [(:dm::ReadAfter :t ?t)])

;; Gap and no recollection: the summary talking in our voice.
(:wat::rete::defrule :dm::hollow
  :when [(:dm::Gap (?g <- :t))
         (:wat::rete::not (:dm::ReadAfter (?r <- :t)))]
  :then [(:dm::Hollow :t ?g)])

;; Recolligere: recollection, the primer fetched, and a log that exists.
(:wat::rete::defrule :dm::recolligere
  :when [(:dm::ReadAfter (?t <- :t))
         (:dm::Beat (?p <- :t) (?k <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?k "fetch-primer"))
         (:wat::rete::exists
           (:dm::Artifact (?ak <- :kind)
             (:wat::rete::string::= ?ak "log")))]
  :then [(:dm::Primer :name "recolligere")])

(:wat::rete::defrule :dm::curare
  :when [(:dm::Beat (?t <- :t) (?k <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?k "tend-record"))]
  :then [(:dm::Primer :name "curare")])

(:wat::rete::defrule :dm::examinare
  :when [(:dm::Beat (?t <- :t) (?k <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?k "weigh-disk"))]
  :then [(:dm::Primer :name "examinare")])

(:wat::rete::defrule :dm::extirpare
  :when [(:dm::Beat (?t <- :t) (?k <- :kind))
         (:wat::rete::where (:wat::rete::string::= ?k "root-failure"))]
  :then [(:dm::Primer :name "extirpare")])

;; The four primers by name — each one, not a count of four things.
(:wat::rete::defrule :dm::four
  :when [(:dm::Primer (?a <- :name))
         (:wat::rete::where (:wat::rete::string::= ?a "recolligere"))
         (:dm::Primer (?b <- :name))
         (:wat::rete::where (:wat::rete::string::= ?b "curare"))
         (:dm::Primer (?c <- :name))
         (:wat::rete::where (:wat::rete::string::= ?c "examinare"))
         (:dm::Primer (?d <- :name))
         (:wat::rete::where (:wat::rete::string::= ?d "extirpare"))]
  :then [(:dm::Four :n 4)])

;; We are the datamancer iff the practice holds and we are not hollow.
(:wat::rete::defrule :dm::we-are
  :when [(:dm::Four (?n <- :n))
         (:wat::rete::not (:dm::Hollow (?h <- :t)))]
  :then [(:dm::Datamancer :n ?n :sigil "RESIDVVM EST PROGRAMMA")])

(:wat::rete::defquery :dm::q-who    :params [] :when [(?who    <- :dm::Datamancer)])
(:wat::rete::defquery :dm::q-hollow :params [] :when [(?hollow <- :dm::Hollow)])
(:wat::rete::defquery :dm::q-gap    :params [] :when [(?f <- :dm::Gap)])
(:wat::rete::defquery :dm::q-read   :params [] :when [(?f <- :dm::ReadAfter)])
(:wat::rete::defquery :dm::q-primer :params [] :when [(?f <- :dm::Primer)])
(:wat::rete::defquery :dm::q-four   :params [] :when [(?f <- :dm::Four)])

(:wat::core::defn :dm::rules [] -> (:wat::core::PersistentVector :- [:wat::rete::Rule])
  (:wat::core::PersistentVector
    (:dm::gap) (:dm::read-after) (:dm::hollow)
    (:dm::recolligere) (:dm::curare) (:dm::examinare) (:dm::extirpare)
    (:dm::four) (:dm::we-are)))

(:wat::core::defn :dm::queries [] -> (:wat::core::PersistentVector :- [:wat::rete::Query])
  (:wat::core::PersistentVector
    (:dm::q-who) (:dm::q-hollow) (:dm::q-gap)
    (:dm::q-read) (:dm::q-primer) (:dm::q-four)))

(:wat::core::defn :dm::seed-practice [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert s
    (:dm::Artifact :kind "log"   :name "datamancer.rete.edn")
    (:dm::Artifact :kind "log"   :name "CURRENT-STATE")
    (:dm::Artifact :kind "cache" :name "summary")
    (:dm::Beat :t 0 :kind "gap")
    (:dm::Beat :t 1 :kind "cache")
    (:dm::Beat :t 2 :kind "read-log")
    (:dm::Beat :t 3 :kind "fetch-primer")
    (:dm::Beat :t 4 :kind "tend-record")
    (:dm::Beat :t 5 :kind "weigh-disk")
    (:dm::Beat :t 6 :kind "root-failure")))

(:wat::core::defn :user::source-counts [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [s0    (:wat::rete::compile-all (:dm::rules) (:dm::queries))
                    fired (:wat::rete::fire-rules (:dm::seed-practice s0))]
    (:wat::core::PersistentVector
      (:wat::core::length (:wat::rete::query fired (:dm::q-gap)))
      (:wat::core::length (:wat::rete::query fired (:dm::q-read)))
      (:wat::core::length (:wat::rete::query fired (:dm::q-hollow)))
      (:wat::core::length (:wat::rete::query fired (:dm::q-primer)))
      (:wat::core::length (:wat::rete::query fired (:dm::q-four)))
      (:wat::core::length (:wat::rete::query fired (:dm::q-who))))))

(:wat::core::defn :user::export-edn [] -> :wat::core::String
  (:wat::edn::write-pretty
    (:wat::rete::export
      (:wat::rete::compile-all (:dm::rules) (:dm::queries)))))

(:wat::core::defn :user::sizes [] -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::let [s0 (:wat::rete::compile-all (:dm::rules) (:dm::queries))
                    exp (:wat::rete::export s0)]
    (:wat::core::PersistentVector
      (:wat::string::length (:wat::edn::write s0))
      (:wat::string::length (:wat::edn::write exp))
      (:wat::string::length (:wat::edn::write-pretty exp)))))

;; Writes tests/rete/datamancer.rete.edn. Invoked only by the wat CLI, never by probes.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::io::write-file
    "tests/rete/datamancer.rete.edn"
    (:wat::string::concat
      ";; The compiled program. Native fire only.\n;; Beats in, Datamancer or Hollow out.\n"
      (:user::export-edn))))
