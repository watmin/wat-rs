;; Co-located fixture for probe_arc278_dead_child_speaks.rs — arc 278: wat NEVER HIDES A FAILURE.
;;
;; A journal' service forked to a PROCESS receives a client message it cannot decode: a Log whose
;; `message` is a user record (:probe::Note) absent from the forked child's baked type registry.
;; At HEAD the child dies decoding it — its real, located reason
;;   "poll' (process tier): client message decode failed: ... unknown tag #probe/Note (body shape:
;;    map); no matching struct or enum in the type registry"
;; is written to an ALREADY-CLOSED err pipe (EPIPE) and LOST; the caller's write-logs recv' raises a
;; MUTE "recv failed: peer closed / channel disconnected". The .rs harness asserts the raised error
;; CARRIES THE REASON. RED at HEAD (mute); GREEN when the masking is pulled out by the root.

(:wat::core::defrecord :probe::Note [text <- :wat::core::String])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [sh    (:wat::query::mem-store'/start :locus (:wat::spawn::process)
             :record (:wat::query::mem-store'::Record :rows (:wat::core::PersistentVector)))
     saddr (:wat::query::mem-store'::Handle/addr sh)
     jh    (:wat::telemetry'::journal'/start
             :locus (:wat::spawn::process/post-spawn
                      (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                        (:wat::query::mem-store'/grant sh
                          (:wat::core::Vector :wat::core::i64 (:wat::spawn::ProcessLaunch/pid pl)))))
             :record (:wat::telemetry'::journal'::Record) :store-addr saddr)
     journal (:wat::kernel::connect' (:wat::telemetry'::journal'::Handle/addr jh))
     tags  (:wat::core::HashMap :wat::core::keyword :wat::core::String)
     l1    (:wat::telemetry'::Log :namespace "probe-ns" :uuid (:wat::core::Uuid/nil) :tags tags
             :time-ns 1000000000 :caller :c1 :level :wat::telemetry'::Level::Info
             :message (:probe::Note :text "one"))
     _wr   (:wat::telemetry'::Journal/write-logs journal
             (:wat::telemetry'::Journal::WriteLogsRequest (:wat::core::Vector :wat::telemetry'::Log l1)))]
    2))
