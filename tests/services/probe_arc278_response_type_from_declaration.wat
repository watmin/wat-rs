;; Arc 278 BRIEF-client-validates-locally — the `RequestTooLarge`/`RequestMalformed` ctors are
;; built from the op's DECLARED response type (`build_op_response_type_constants`,
;; src/types.rs), never guessed by `<OpPascal>Response` concatenation. `:probe::Odd::Verdict`
;; (for op `put`) is deliberately NOT named `PutResponse` — the acceptance shape (mirrors
;; `wat-scripts/scratch-pad/probe-repl-durable-forms.wat`'s `EvalResponse`): a still-guessing
;; call site fails on it first.
;;
;; This is the PROMOTED throwaway from the deliberate-break verification — kept, not deleted,
;; because nothing else in the corpus exercises either generated path with a non-conventional
;; response name:
;;   - every existing fixture calls the SURFACE name (`:probe::Big/put`), which resolves through
;;     the Path-B runtime intrinsic (`src/runtime.rs`), never through `op-methods`'s own
;;     generated `<service>/<op>` function;
;;   - the one existing per-op-guard fixture (`probe_arc278_per_op_enforcement_codegen.wat`)
;;     uses a conventionally-named response type, so it cannot tell a correct read from a guess
;;     that happens to land on the same string.
;;
;; Three probes, one per ctor reference now built off the constant:
;;   (1) op-methods' own RequestTooLarge  — client-side, via the SERVICE-namespaced call.
;;   (2) serve-op-arms' RequestTooLarge   — server-side, via a RAW send' that bypasses every
;;       client-side check, sized between `:max-request-bytes` and `:max-frame-bytes` (FOO) so
;;       the per-op CODEGEN guard fires, never the transport-level FOO check.
;;   (3) serve-op-arms' RequestMalformed  — server-side, via a RAW send' of a right-tag,
;;       wrong-shape request (an i64 field carrying a String).

(:wat::core::defsurface :probe::Odd :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Odd::PutRequest [payload <- :wat::core::String  count <- :wat::core::i64])
   (:wat::core::defenum :probe::Odd::Verdict :wat::enum::Pure
     :Ok              [ok <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- :wat::core::Vector<wat::core::String>  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(put [self <- :probe::Odd  req <- :probe::Odd::PutRequest] -> :probe::Odd::Verdict :max-request-bytes 100)])

;; FOO (65536) is generous relative to the 100-byte contract: a payload sized between the two
;; (probes (1) and (2) below use ~640 bytes) reaches the server without tripping the transport
;; frame check, so it is the per-op CODEGEN guard — not FOO — that must catch it.
(:wat::service::defservice :probe::oddsvc
  :satisfies :probe::Odd
  :max-frame-bytes 65536
  :durable   []
  :ephemeral []
  :impls
  [(put [s req] (:wat::service::Outcome::Reply s (:probe::Odd::Verdict::Ok 7)))])

(:wat::core::defn :probe::odd-payload-of [n <- :wat::core::i64] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String  _i <- :wat::core::i64] -> :wat::core::String
      (:wat::core::string::concat acc "0123456789ABCDEF0123456789ABCDEF"))
    ""
    (:wat::core::range 0 n)))

;; (1) op-methods' OWN generated function, called by its SERVICE-namespaced name
;;     (`:probe::oddsvc/put`, never `:probe::Odd/put` — the surface name would route through
;;     Path B and prove nothing about this generator). Payload ~640 bytes: over the 100-byte
;;     contract, far under FOO — refused LOCALLY, by op-methods' own budget check.
(:wat::core::defn :user::op-methods-over-budget-refused-locally [] -> :wat::core::i64
  (:wat::core::let
    [big (:probe::odd-payload-of 20)   ;; 640 bytes > cap(100), << FOO(65536)
     h   (:probe::oddsvc/start :locus (:wat::spawn::process) :record (:probe::oddsvc::Record))
     c   (:wat::core::match (:wat::kernel::connect (:probe::oddsvc::Handle/addr h))
           ((:wat::kernel::ConnectOutcome::Connected p) p)
           ((:wat::kernel::ConnectOutcome::Refused cz) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))
           ((:wat::kernel::ConnectOutcome::Rejected cz) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))
           ((:wat::kernel::ConnectOutcome::Failed cz) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None)))
     r   (:probe::oddsvc/put c (:probe::Odd::PutRequest :payload big :count 1))]
    (:wat::core::match r
      ((:wat::kernel::RecvOutcome::Message resp)
        (:wat::core::match resp
          ((:probe::Odd::Verdict::RequestTooLarge bytes cap) bytes)
          ((:probe::Odd::Verdict::Ok _ok) (:wat::kernel::assertion-failed! "expected RequestTooLarge, got Ok" :wat::core::None :wat::core::None))
          ((:probe::Odd::Verdict::RequestMalformed _mpath _mexp _mgot) (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::core::string::concat "DISCRIMINATOR: op-methods did not refuse locally — " (:wat::kernel::LociDiedError/message cause)) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "DISCRIMINATOR: op-methods did not refuse locally (closed)" :wat::core::None :wat::core::None)))))

;; (2) serve-op-arms' per-op SIZE guard, driven by a RAW send'+recv' that bypasses op-methods'
;;     AND Path B's local checks entirely (no generated client method called at all) — so an
;;     over-cap, under-FOO request actually reaches the server, and the CODEGEN guard (not the
;;     transport FOO check) must be what flags it.
(:wat::core::defn :user::serve-op-arms-size-guard-fires [] -> :wat::core::i64
  (:wat::core::let
    [big (:probe::odd-payload-of 20)
     h   (:probe::oddsvc/start :locus (:wat::spawn::process) :record (:probe::oddsvc::Record))
     c   (:wat::core::match (:wat::kernel::connect (:probe::oddsvc::Handle/addr h))
           ((:wat::kernel::ConnectOutcome::Connected p) p)
           ((:wat::kernel::ConnectOutcome::Refused cz) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))
           ((:wat::kernel::ConnectOutcome::Rejected cz) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))
           ((:wat::kernel::ConnectOutcome::Failed cz) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None)))
     _s  (:wat::core::match (:wat::kernel::send c (:probe::Odd::Op::Put (:probe::Odd::PutRequest :payload big :count 1)))
           (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
     r   (:wat::kernel::recv c)]
    (:wat::core::match r
      ((:wat::kernel::RecvOutcome::Message resp)
        (:wat::core::match resp
          ((:probe::Odd::Reply::Put verdict)
            (:wat::core::match verdict
              ((:probe::Odd::Verdict::RequestTooLarge bytes cap) bytes)
              ((:probe::Odd::Verdict::Ok _ok) (:wat::kernel::assertion-failed! "expected RequestTooLarge, got Ok" :wat::core::None :wat::core::None))
              ((:probe::Odd::Verdict::RequestMalformed _mpath _mexp _mgot) (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
          (_ (:wat::kernel::assertion-failed! "misrouted reply variant" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::core::string::concat "DISCRIMINATOR: serve-op-arms' size guard did not fire — " (:wat::kernel::LociDiedError/message cause)) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "DISCRIMINATOR: serve-op-arms' size guard did not fire (closed)" :wat::core::None :wat::core::None)))))

;; (3) serve-op-arms' shape guard (`:wat::edn::validate`), driven by a RAW send' of a right-tag,
;;     wrong-shape request: `count` (declared `i64`) arrives as a String. `:wat::edn::read`
;;     decodes it loosely into a `PutRequest`-shaped value (the polymorphic return unifies with
;;     the `Op::Put` ctor's expected field type — the SAME technique
;;     `wat-tests/service-parametric-messages.wat` probe (2) uses); the server's declared-type
;;     walk is what actually catches the mismatch.
(:wat::core::defn :user::serve-op-arms-shape-guard-fires [] -> :wat::core::String
  (:wat::core::let
    [h   (:probe::oddsvc/start :locus (:wat::spawn::process) :record (:probe::oddsvc::Record))
     c   (:wat::core::match (:wat::kernel::connect (:probe::oddsvc::Handle/addr h))
           ((:wat::kernel::ConnectOutcome::Connected p) p)
           ((:wat::kernel::ConnectOutcome::Refused cz) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))
           ((:wat::kernel::ConnectOutcome::Rejected cz) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))
           ((:wat::kernel::ConnectOutcome::Failed cz) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None)))
     bad (:wat::edn::read "#probe.Odd/PutRequest {:payload \"ok\" :count \"not-a-number\"}")
     _s  (:wat::core::match (:wat::kernel::send c (:probe::Odd::Op::Put bad))
           (:wat::kernel::SendOutcome::Sent nil) (:wat::kernel::SendOutcome::Closed nil) ((:wat::kernel::SendOutcome::Lost _c) nil))
     r   (:wat::kernel::recv c)]
    (:wat::core::match r
      ((:wat::kernel::RecvOutcome::Message resp)
        (:wat::core::match resp
          ((:probe::Odd::Reply::Put verdict)
            (:wat::core::match verdict
              ((:probe::Odd::Verdict::RequestMalformed mpath mexpected mgot)
                (:wat::core::string::concat (:wat::edn::write mpath)
                  (:wat::core::string::concat "/" (:wat::core::string::concat mexpected (:wat::core::string::concat "/" mgot)))))
              ((:probe::Odd::Verdict::Ok _ok) (:wat::kernel::assertion-failed! "expected RequestMalformed, got Ok" :wat::core::None :wat::core::None))
              ((:probe::Odd::Verdict::RequestTooLarge _bytes _cap) (:wat::kernel::assertion-failed! "unexpected RequestTooLarge" :wat::core::None :wat::core::None))))
          (_ (:wat::kernel::assertion-failed! "misrouted reply variant" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::core::string::concat "DISCRIMINATOR: serve-op-arms' shape guard did not fire — " (:wat::kernel::LociDiedError/message cause)) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "DISCRIMINATOR: serve-op-arms' shape guard did not fire (closed)" :wat::core::None :wat::core::None)))))
