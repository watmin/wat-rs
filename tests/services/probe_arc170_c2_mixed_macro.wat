;; Arc 170 C2 — Strike 2: the C2 gate THROUGH THE `bracket/uses` MACRO (mixed 7 services + 5 data).
;;
;; The full user surface in one form: (bracket/uses (process) items work-fn :name val …) with RAW
;; `:name val` pairs (handles for the 7 services, values for the 5 data), SCRAMBLED order. Proves:
;;  - Part A: the checker takes `Dialable<S,R>` → raw handles type-check; it coords services internally.
;;  - Part B: the macro parses + expands to `(let [coords (…kwargs-check :name val…)] (uses' …))`.
;;  - Part C: `uses'`'s grant-boot dispatches the heterogeneous `[(Tuple :name val) …]` per val
;;    (7 service handles granted, 5 data values skipped) — mixed service+data in ONE strike.
;;  - the Strike-1 dial runtime (unchanged): ::Coords → ::Kwargs by field name, dial 7 + copy 5.
;; Expect ["a|s1:as2:as3:as4:as5:as6:as7:aD1D2D3D4D5" "b|s1:b…s7:bD1D2D3D4D5"].
;;    reconciles ::Coords → ::Kwargs by field name (connect' the 7 Peer' fields, copy the 5 data).
;;  - Strike 1c (wat/bracket.wat :wat::bracket::uses'): ONE `PoolMsg::Setup(coords-record)` per worker.

;; ── 7 heterogeneous services ─────────────────────────────────────────────────
(:wat::core::defsurface :probe::S1 :nature :wat::kernel::Peer'
  :messages [(:wat::core::defrecord :probe::S1::OpRequest [m <- :wat::core::String])
             (:wat::core::defrecord :probe::S1::OpResponse [r <- :wat::core::String])]
  :features [(op [self <- :probe::S1  req <- :probe::S1::OpRequest] -> :probe::S1::OpResponse)])
(:wat::service::defservice :probe::s1' :satisfies :probe::S1 :durable [] :ephemeral []
  :impls [(op [s req] (:wat::service::Outcome::Reply s
            (:probe::S1::OpResponse (:wat::core::string::concat "s1:" (:probe::S1::OpRequest/m req)))))])

(:wat::core::defsurface :probe::S2 :nature :wat::kernel::Peer'
  :messages [(:wat::core::defrecord :probe::S2::OpRequest [m <- :wat::core::String])
             (:wat::core::defrecord :probe::S2::OpResponse [r <- :wat::core::String])]
  :features [(op [self <- :probe::S2  req <- :probe::S2::OpRequest] -> :probe::S2::OpResponse)])
(:wat::service::defservice :probe::s2' :satisfies :probe::S2 :durable [] :ephemeral []
  :impls [(op [s req] (:wat::service::Outcome::Reply s
            (:probe::S2::OpResponse (:wat::core::string::concat "s2:" (:probe::S2::OpRequest/m req)))))])

(:wat::core::defsurface :probe::S3 :nature :wat::kernel::Peer'
  :messages [(:wat::core::defrecord :probe::S3::OpRequest [m <- :wat::core::String])
             (:wat::core::defrecord :probe::S3::OpResponse [r <- :wat::core::String])]
  :features [(op [self <- :probe::S3  req <- :probe::S3::OpRequest] -> :probe::S3::OpResponse)])
(:wat::service::defservice :probe::s3' :satisfies :probe::S3 :durable [] :ephemeral []
  :impls [(op [s req] (:wat::service::Outcome::Reply s
            (:probe::S3::OpResponse (:wat::core::string::concat "s3:" (:probe::S3::OpRequest/m req)))))])

(:wat::core::defsurface :probe::S4 :nature :wat::kernel::Peer'
  :messages [(:wat::core::defrecord :probe::S4::OpRequest [m <- :wat::core::String])
             (:wat::core::defrecord :probe::S4::OpResponse [r <- :wat::core::String])]
  :features [(op [self <- :probe::S4  req <- :probe::S4::OpRequest] -> :probe::S4::OpResponse)])
(:wat::service::defservice :probe::s4' :satisfies :probe::S4 :durable [] :ephemeral []
  :impls [(op [s req] (:wat::service::Outcome::Reply s
            (:probe::S4::OpResponse (:wat::core::string::concat "s4:" (:probe::S4::OpRequest/m req)))))])

(:wat::core::defsurface :probe::S5 :nature :wat::kernel::Peer'
  :messages [(:wat::core::defrecord :probe::S5::OpRequest [m <- :wat::core::String])
             (:wat::core::defrecord :probe::S5::OpResponse [r <- :wat::core::String])]
  :features [(op [self <- :probe::S5  req <- :probe::S5::OpRequest] -> :probe::S5::OpResponse)])
(:wat::service::defservice :probe::s5' :satisfies :probe::S5 :durable [] :ephemeral []
  :impls [(op [s req] (:wat::service::Outcome::Reply s
            (:probe::S5::OpResponse (:wat::core::string::concat "s5:" (:probe::S5::OpRequest/m req)))))])

(:wat::core::defsurface :probe::S6 :nature :wat::kernel::Peer'
  :messages [(:wat::core::defrecord :probe::S6::OpRequest [m <- :wat::core::String])
             (:wat::core::defrecord :probe::S6::OpResponse [r <- :wat::core::String])]
  :features [(op [self <- :probe::S6  req <- :probe::S6::OpRequest] -> :probe::S6::OpResponse)])
(:wat::service::defservice :probe::s6' :satisfies :probe::S6 :durable [] :ephemeral []
  :impls [(op [s req] (:wat::service::Outcome::Reply s
            (:probe::S6::OpResponse (:wat::core::string::concat "s6:" (:probe::S6::OpRequest/m req)))))])

(:wat::core::defsurface :probe::S7 :nature :wat::kernel::Peer'
  :messages [(:wat::core::defrecord :probe::S7::OpRequest [m <- :wat::core::String])
             (:wat::core::defrecord :probe::S7::OpResponse [r <- :wat::core::String])]
  :features [(op [self <- :probe::S7  req <- :probe::S7::OpRequest] -> :probe::S7::OpResponse)])
(:wat::service::defservice :probe::s7' :satisfies :probe::S7 :durable [] :ephemeral []
  :impls [(op [s req] (:wat::service::Outcome::Reply s
            (:probe::S7::OpResponse (:wat::core::string::concat "s7:" (:probe::S7::OpRequest/m req)))))])

;; ── the work-fn: item POSITIONAL; 7 Peer' service kwargs + 5 String data kwargs ──
(:wat::core::defn :probe::enrich
  [item <- :wat::core::String
   & [s1 <- :wat::kernel::Peer'<probe::S1::Op,probe::S1::Reply>
      s2 <- :wat::kernel::Peer'<probe::S2::Op,probe::S2::Reply>
      s3 <- :wat::kernel::Peer'<probe::S3::Op,probe::S3::Reply>
      s4 <- :wat::kernel::Peer'<probe::S4::Op,probe::S4::Reply>
      s5 <- :wat::kernel::Peer'<probe::S5::Op,probe::S5::Reply>
      s6 <- :wat::kernel::Peer'<probe::S6::Op,probe::S6::Reply>
      s7 <- :wat::kernel::Peer'<probe::S7::Op,probe::S7::Reply>
      d1 <- :wat::core::String
      d2 <- :wat::core::String
      d3 <- :wat::core::String
      d4 <- :wat::core::String
      d5 <- :wat::core::String]]
  -> :wat::core::String
  (:wat::core::let
    [r1  (:probe::S1::OpResponse/r (:probe::S1/op s1 (:probe::S1::OpRequest item)))
     r2  (:probe::S2::OpResponse/r (:probe::S2/op s2 (:probe::S2::OpRequest item)))
     r3  (:probe::S3::OpResponse/r (:probe::S3/op s3 (:probe::S3::OpRequest item)))
     r4  (:probe::S4::OpResponse/r (:probe::S4/op s4 (:probe::S4::OpRequest item)))
     r5  (:probe::S5::OpResponse/r (:probe::S5/op s5 (:probe::S5::OpRequest item)))
     r6  (:probe::S6::OpResponse/r (:probe::S6/op s6 (:probe::S6::OpRequest item)))
     r7  (:probe::S7::OpResponse/r (:probe::S7/op s7 (:probe::S7::OpRequest item)))
     svc (:wat::core::string::concat r1
           (:wat::core::string::concat r2
             (:wat::core::string::concat r3
               (:wat::core::string::concat r4
                 (:wat::core::string::concat r5
                   (:wat::core::string::concat r6 r7))))))
     dat (:wat::core::string::concat d1
           (:wat::core::string::concat d2
             (:wat::core::string::concat d3
               (:wat::core::string::concat d4 d5))))]
    (:wat::core::string::concat item
      (:wat::core::string::concat "|"
        (:wat::core::string::concat svc dat)))))

;; `:probe::run` (a non-main defn — no `:user::main`; only freezes + is called directly).
;; THROUGH THE MACRO: raw handles + data, scrambled order, no Dialable/coord wrapping.
(:wat::core::defn :probe::run [] -> :wat::core::Vector<wat::core::String>
  (:wat::core::let
    [h1 (:probe::s1'/start :locus (:wat::spawn::process) :record (:probe::s1'::Record))
     h2 (:probe::s2'/start :locus (:wat::spawn::process) :record (:probe::s2'::Record))
     h3 (:probe::s3'/start :locus (:wat::spawn::process) :record (:probe::s3'::Record))
     h4 (:probe::s4'/start :locus (:wat::spawn::process) :record (:probe::s4'::Record))
     h5 (:probe::s5'/start :locus (:wat::spawn::process) :record (:probe::s5'::Record))
     h6 (:probe::s6'/start :locus (:wat::spawn::process) :record (:probe::s6'::Record))
     h7 (:probe::s7'/start :locus (:wat::spawn::process) :record (:probe::s7'::Record))]
    (:wat::bracket::map (:wat::spawn::process) ["a" "b"] :probe::enrich
      :d2 "D2" :s3 h3 :s1 h1 :d5 "D5" :s7 h7 :d1 "D1"
      :s2 h2 :s5 h5 :d4 "D4" :s4 h4 :d3 "D3" :s6 h6)))
