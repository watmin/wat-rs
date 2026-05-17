;; wat-tests/counter-service-capability-N3.wat — Capability-wrapped multi-user counter.
;;
;; Arc 203 slice 3f — sixth stepping stone (in-place update of slice 3e).
;; Replaces panic-on-error semantics with honest Result-bearing wrappers.
;;
;; Previously (slice 3e): wrappers returned raw T; transport errors panicked via Result/expect.
;; Now (slice 3f): every wrapper returns Result<T,:counter::ServiceError>; callers match Ok/Err.
;;
;; Wraps slice 3b's bare-channel multi-user flow in struct-restricted capability values.
;;
;; Extends slice 3e (counter-service-capability-N3.wat) with:
;;   :counter::ServiceError — typed error enum for all client-facing error paths
;;   Result-bearing wrappers — every send/recv site becomes explicit match+propagate
;;   Test body demonstrates BOTH paths:
;;     - Happy path: pattern-match Ok, extract value, assert
;;     - Err path: call after Stop → PeerDied or Disconnected
;;
;; ─── ERROR TYPE DESIGN ────────────────────────────────────────────────────────
;;
;; :counter::ServiceError carries typed substrate errors (no String escape):
;;   AccessDenied  — server rejected server-id (wire-level)
;;   PeerDied      — thread peer dropped/panicked; carries chain (Vector<ThreadDiedError>)
;;   Disconnected  — clean recv-returned-None (sender dropped normally)
;;
;; send returns: Result<(),Vector<ThreadDiedError>>  (Err = ChannelDisconnected chain)
;; recv returns: Result<Option<T>,Vector<ThreadDiedError>>  (Err = Shutdown chain)
;;   recv Ok(None)   = clean disconnect (sender dropped)
;;   recv Ok(Some v) = value
;;   recv Err(chain) = shutdown signal
;;
;; PeerDied carries the Vector<ThreadDiedError> chain directly (no collapse to one).
;;
;; ─── THREAD-TIER VALIDATION SEMANTICS ────────────────────────────────────────
;;
;; At the thread tier, channel ownership already prevents wire forgery structurally.
;; The admin-tx Sender is stored inside the struct-restricted Admin; the user-tx
;; Sender is stored inside the struct-restricted Client. Code outside :counter::*
;; cannot obtain these senders. Therefore, any Wire that arrives at the server
;; ALREADY came from a :counter::* wrapper — the sender is a structural proof.
;;
;; The server-id check in this file is DEFENSE IN DEPTH: a harmless redundancy
;; that mirrors the process-tier pattern uniformly. The check adds a validation
;; step that would catch bugs (e.g. a wrapper accidentally embedding the wrong
;; server-id) and makes the code a consistent model for the process tier where
;; the check is load-bearing.
;;
;; In production, mint server-id via :wat::telemetry::uuid::v4 for unguessability.
;; The constant string "server-counter-thread-0" demonstrates the validation flow.
;;
;; ─────────────────────────────────────────────────────────────────────────────
;;
;; Key lessons from prior slices:
;;   - Registry MUST be Vector<RegistryEntry> (HashMap order non-deterministic)
;;   - Inner type aliases in :() are bare (no leading colon)
;;   - Fold primitive is foldl (not reduce)
;;   - Inline :() annotations on ONE line (no whitespace inside)
;;   - first/second/third tuple accessors (no Tuple/N)
;;   - Thread/drain-and-join (not Thread/join-result)
;;   - Inner/outer let for scope-deadlock compliance at drain-and-join sites
;;   - AdminResp::Provisioned.rx is Receiver<UserResp> (not Receiver<Wire>)
;;   - Two-level match required when matching enums carrying enum payloads
;;   - send returns Result<(),Vector<ThreadDiedError>>  (Err = chain, not single)
;;   - recv returns Result<Option<T>,Vector<ThreadDiedError>>  (same chain shape)
;;   - Thread/drain-and-join returns Result<nil,Vector<ThreadDiedError>>

(:wat::test::deftest :counter-service::capability-N3
  (;; ─── Admin protocol ──────────────────────────────────────────────────────
   (:wat::core::enum :counter::AdminReq
     (Provision (initial :wat::core::i64))
     (Deprovision (id :wat::core::String))
     (Stop))

   (:wat::core::enum :counter::AdminResp
     (Provisioned (id :wat::core::String) (tx :wat::kernel::Sender<counter::Wire>) (rx :wat::kernel::Receiver<counter::UserResp>))
     (Deprovisioned (id :wat::core::String))
     (Stopped)
     (AccessDenied))                          ;; server refused — server-id mismatch

   ;; ─── User protocol ───────────────────────────────────────────────────────
   (:wat::core::enum :counter::UserReq
     (Get)
     (Increment (n :wat::core::i64))
     (Reset))

   (:wat::core::enum :counter::UserResp
     (Value (v :wat::core::i64))
     (Ok    (v :wat::core::i64))
     (AccessDenied))                          ;; server refused — server-id mismatch

   ;; ─── Wire enum ────────────────────────────────────────────────────────────
   ;; server-id is the FIRST field on both variants — the secret witness.
   ;; Every wrapper embeds the capability's server-id when constructing Wire.
   ;; The server validates server-id against its own before processing.
   (:wat::core::enum :counter::Wire
     (Admin (server-id :wat::core::String) (req :counter::AdminReq))
     (User  (server-id :wat::core::String) (id :wat::core::String) (req :counter::UserReq)))

   ;; ─── ServiceError enum ────────────────────────────────────────────────────
   ;;
   ;; The honest error type for all client-facing wrappers.
   ;;
   ;;   AccessDenied  — server validated server-id and rejected the request
   ;;   PeerDied      — send/recv/drain-and-join returned Err with a chain
   ;;                   (chain is Vector<ThreadDiedError> — substrate arc 113 shape)
   ;;   Disconnected  — recv returned Ok(None) — clean sender dropout
   ;;
   ;; PeerDied carries chain (not String): callers can inspect cause variants
   ;; (Panic / RuntimeError / ChannelDisconnected / Shutdown) without string parsing.
   ;;
   ;; At the thread tier:
   ;;   - send Err → PeerDied (ChannelDisconnected chain)
   ;;   - recv Ok(None) → Disconnected
   ;;   - recv Err → PeerDied (Shutdown chain — rare, process-wide shutdown)
   ;;   - drain-and-join Err → PeerDied (chain)
   (:wat::core::enum :counter::ServiceError
     (AccessDenied)
     (PeerDied   (chain :wat::core::Vector<wat::kernel::ThreadDiedError>))
     (Disconnected))

   ;; ─── Registry types ───────────────────────────────────────────────────────
   (:wat::core::typealias :counter::TxStatePair
     :(wat::kernel::Sender<counter::UserResp>,wat::core::i64))

   (:wat::core::typealias :counter::RegistryEntry
     :(wat::core::String,wat::kernel::Receiver<counter::Wire>,counter::TxStatePair))

   (:wat::core::typealias :counter::RegistryVec
     :wat::core::Vector<counter::RegistryEntry>)

   ;; ─── Capability structs ───────────────────────────────────────────────────
   ;;
   ;; :counter::Admin — the privileged admin handle.
   ;;
   ;; Minted ONLY by :counter::spawn-cap (constructor whitelist [:counter::]).
   ;; All fields are restricted to :counter::* reads.
   (:wat::core::struct-restricted :counter::Admin
     [:counter::]
     ([:counter::] server-id <- :wat::core::String
      [:counter::] admin-tx  <- :wat::kernel::Sender<counter::Wire>
      [:counter::] admin-rx  <- :wat::kernel::Receiver<counter::AdminResp>
      [:counter::] thread    <- :wat::kernel::Thread<counter::Wire,counter::AdminResp>)
     ())

   ;; :counter::Client — per-user capability handle, server-issued via Provision.
   ;;
   ;; Minted ONLY by :counter::provision (constructor whitelist [:counter::]).
   ;; All fields are restricted to :counter::* reads.
   (:wat::core::struct-restricted :counter::Client
     [:counter::]
     ([:counter::] server-id <- :wat::core::String
      [:counter::] client-id <- :wat::core::String
      [:counter::] user-tx   <- :wat::kernel::Sender<counter::Wire>
      [:counter::] user-rx   <- :wat::kernel::Receiver<counter::UserResp>)
     ())

   ;; ─── Registry helpers ────────────────────────────────────────────────────
   (:wat::core::defn :counter::registry-rxs
     [registry-vec <- :counter::RegistryVec]
     -> :wat::core::Vector<wat::kernel::Receiver<counter::Wire>>
     (:wat::core::map registry-vec
       (:wat::core::fn
         [entry <- :counter::RegistryEntry]
          -> :wat::kernel::Receiver<counter::Wire>
         (:wat::core::second entry))))

   (:wat::core::defn :counter::registry-provision
     [registry-vec <- :counter::RegistryVec
      id          <- :wat::core::String
      server-rx   <- :wat::kernel::Receiver<counter::Wire>
      server-tx   <- :wat::kernel::Sender<counter::UserResp>
      initial     <- :wat::core::i64]
     -> :counter::RegistryVec
     (:wat::core::conj registry-vec
       (:wat::core::Tuple id server-rx
         (:wat::core::Tuple server-tx initial))))

   (:wat::core::defn :counter::registry-deprovision
     [registry-vec <- :counter::RegistryVec
      id           <- :wat::core::String]
     -> :counter::RegistryVec
     (:wat::core::filter registry-vec
       (:wat::core::fn
         [entry <- :counter::RegistryEntry]
          -> :wat::core::bool
         (:wat::core::not
           (:wat::core::= (:wat::core::first entry) id)))))

   (:wat::core::defn :counter::registry-update-state
     [registry-vec <- :counter::RegistryVec
      target-idx   <- :wat::core::i64
      new-state    <- :wat::core::i64]
     -> :counter::RegistryVec
     (:wat::core::let
       [init (:wat::core::Tuple
               (:wat::core::Vector :counter::RegistryEntry)
               0)
        result
         (:wat::core::foldl registry-vec init
           (:wat::core::fn
             [acc   <- :(wat::core::Vector<counter::RegistryEntry>,wat::core::i64)
              entry <- :counter::RegistryEntry]
              -> :(wat::core::Vector<counter::RegistryEntry>,wat::core::i64)
             (:wat::core::let
               [new-vec  (:wat::core::first acc)
                cur-pos  (:wat::core::second acc)
                updated-entry
                  (:wat::core::if (:wat::core::= cur-pos target-idx)
                    -> :counter::RegistryEntry
                    (:wat::core::let
                      [eid    (:wat::core::first  entry)
                       erx    (:wat::core::second entry)
                       etx    (:wat::core::first  (:wat::core::third entry))]
                      (:wat::core::Tuple eid erx (:wat::core::Tuple etx new-state)))
                    entry)
                next-vec (:wat::core::conj new-vec updated-entry)
                next-pos (:wat::core::i64::+'2 cur-pos 1)]
               (:wat::core::Tuple next-vec next-pos))))]
       (:wat::core::first result)))

   ;; ─── Server dispatch loop ─────────────────────────────────────────────────
   ;;
   ;; The server holds its OWN server-id = "server-counter-thread-0".
   ;; Every Wire that arrives carries a wire-server-id field.
   ;; Dispatch functions check wire-server-id vs self-server-id before routing.
   ;;
   ;; DEFENSE IN DEPTH: at the thread tier, the Sender<Wire> is stored inside
   ;; struct-restricted Admin/Client — code outside :counter::* cannot forge a Wire.
   ;; The check is harmless redundancy that mirrors the process-tier pattern uniformly.
   ;;
   ;; NOTE: server-side sends still use Result/expect (send to admin-resp-tx / server-tx).
   ;; If those sends fail, it means the recipient dropped — a structural protocol violation.
   ;; The server continues dispatch on such failures (same behavior as pre-3f).
   ;; The 3f Result-bearing change applies ONLY to client-facing wrappers.
   (:wat::core::defn :counter::dispatch3
     [admin-wire-rx <- :wat::kernel::Receiver<counter::Wire>
      admin-resp-tx <- :wat::kernel::Sender<counter::AdminResp>
      registry-vec  <- :counter::RegistryVec
      next-id       <- :wat::core::i64]
     -> :wat::core::nil
     (:wat::core::let
       [user-rxs     (:counter::registry-rxs registry-vec)
        admin-vec    (:wat::core::Vector :wat::kernel::Receiver<counter::Wire> admin-wire-rx)
        select-set   (:wat::core::concat user-rxs admin-vec)
        registry-len (:wat::core::length registry-vec)
        chosen       (:wat::kernel::select select-set)
        idx          (:wat::core::first chosen)
        result       (:wat::core::second chosen)
        is-admin     (:wat::core::= idx registry-len)]
       (:wat::core::match result -> :wat::core::nil
         ((:wat::core::Ok (:wat::core::Some wire))
           (:wat::core::if is-admin -> :wat::core::nil
             (:counter::handle-admin3
               admin-wire-rx admin-resp-tx registry-vec next-id wire)
             (:counter::handle-user3
               admin-wire-rx admin-resp-tx registry-vec next-id idx wire)))
         ((:wat::core::Ok :wat::core::None)
           (:wat::core::if is-admin -> :wat::core::nil
             ()
             (:counter::dispatch3
               admin-wire-rx admin-resp-tx
               (:wat::std::list::remove-at registry-vec idx)
               next-id)))
         ((:wat::core::Err _died)
           (:wat::core::if is-admin -> :wat::core::nil
             ()
             (:counter::dispatch3
               admin-wire-rx admin-resp-tx
               (:wat::std::list::remove-at registry-vec idx)
               next-id))))))

   (:wat::core::defn :counter::handle-admin3
     [admin-wire-rx <- :wat::kernel::Receiver<counter::Wire>
      admin-resp-tx <- :wat::kernel::Sender<counter::AdminResp>
      registry-vec  <- :counter::RegistryVec
      next-id       <- :wat::core::i64
      wire          <- :counter::Wire]
     -> :wat::core::nil
     (:wat::core::match wire -> :wat::core::nil
       ((:counter::Wire::Admin wire-sid req)
         ;; Server-id validation (defense in depth at thread tier).
         ;; Channel ownership already prevents forge structurally; this
         ;; check is a uniform harmless redundancy mirroring process-tier.
         (:wat::core::if (:wat::core::= wire-sid "server-counter-thread-0")
           -> :wat::core::nil
           (:wat::core::match req -> :wat::core::nil
             ((:counter::AdminReq::Provision initial)
               (:wat::core::let
                 [id-str    (:wat::core::string::concat "client-"
                              (:wat::core::i64::to-string next-id))
                  uwp       (:wat::kernel::make-bounded-channel :counter::Wire 1)
                  user-tx   (:wat::core::first  uwp)
                  server-rx (:wat::core::second uwp)
                  urp       (:wat::kernel::make-bounded-channel :counter::UserResp 1)
                  server-tx (:wat::core::first  urp)
                  user-rx   (:wat::core::second urp)
                  new-registry
                    (:counter::registry-provision
                      registry-vec id-str server-rx server-tx initial)
                  _sent
                    (:wat::core::Result/expect -> :wat::core::nil
                      (:wat::kernel::send admin-resp-tx
                        (:counter::AdminResp::Provisioned id-str user-tx user-rx))
                      "handle-admin3: admin-resp-tx disconnected on Provision")]
                 (:counter::dispatch3
                   admin-wire-rx admin-resp-tx
                   new-registry
                   (:wat::core::i64::+'2 next-id 1))))
             ((:counter::AdminReq::Deprovision dep-id)
               (:wat::core::let
                 [new-registry
                    (:counter::registry-deprovision registry-vec dep-id)
                  _sent
                    (:wat::core::Result/expect -> :wat::core::nil
                      (:wat::kernel::send admin-resp-tx
                        (:counter::AdminResp::Deprovisioned dep-id))
                      "handle-admin3: admin-resp-tx disconnected on Deprovision")]
                 (:counter::dispatch3
                   admin-wire-rx admin-resp-tx
                   new-registry
                   next-id)))
             ((:counter::AdminReq::Stop)
               (:wat::core::Result/expect -> :wat::core::nil
                 (:wat::kernel::send admin-resp-tx
                   (:counter::AdminResp::Stopped))
                 "handle-admin3: admin-resp-tx disconnected on Stop")))
           ;; Mismatch: emit AccessDenied; continue dispatch (do not process request)
           (:wat::core::let
             [_denied
                (:wat::core::Result/expect -> :wat::core::nil
                  (:wat::kernel::send admin-resp-tx
                    (:counter::AdminResp::AccessDenied))
                  "handle-admin3: admin-resp-tx disconnected on AccessDenied")]
             (:counter::dispatch3
               admin-wire-rx admin-resp-tx
               registry-vec next-id))))
       ((:counter::Wire::User _wire-sid _id _req)
         (:counter::dispatch3
           admin-wire-rx admin-resp-tx
           registry-vec next-id))))

   (:wat::core::defn :counter::handle-user3
     [admin-wire-rx <- :wat::kernel::Receiver<counter::Wire>
      admin-resp-tx <- :wat::kernel::Sender<counter::AdminResp>
      registry-vec  <- :counter::RegistryVec
      next-id       <- :wat::core::i64
      idx           <- :wat::core::i64
      wire          <- :counter::Wire]
     -> :wat::core::nil
     (:wat::core::let
       [entry-opt (:wat::core::get registry-vec idx)]
       (:wat::core::match entry-opt -> :wat::core::nil
         ((:wat::core::Some entry)
           (:wat::core::let
             [tx-state  (:wat::core::third entry)
              server-tx (:wat::core::first tx-state)
              state     (:wat::core::second tx-state)]
             (:wat::core::match wire -> :wat::core::nil
               ((:counter::Wire::User wire-sid _id req)
                 ;; Server-id validation (defense in depth at thread tier).
                 (:wat::core::if (:wat::core::= wire-sid "server-counter-thread-0")
                   -> :wat::core::nil
                   (:wat::core::match req -> :wat::core::nil
                     ((:counter::UserReq::Get)
                       (:wat::core::do
                         (:wat::core::Result/expect -> :wat::core::nil
                           (:wat::kernel::send server-tx
                             (:counter::UserResp::Value state))
                           "handle-user3: server-tx disconnected on Get")
                         (:counter::dispatch3
                           admin-wire-rx admin-resp-tx
                           registry-vec next-id)))
                     ((:counter::UserReq::Increment n)
                       (:wat::core::let
                         [new-n     (:wat::core::i64::+'2 state n)
                          _sent
                            (:wat::core::Result/expect -> :wat::core::nil
                              (:wat::kernel::send server-tx
                                (:counter::UserResp::Ok new-n))
                              "handle-user3: server-tx disconnected on Increment")
                          new-registry
                            (:counter::registry-update-state registry-vec idx new-n)]
                         (:counter::dispatch3
                           admin-wire-rx admin-resp-tx
                           new-registry next-id)))
                     ((:counter::UserReq::Reset)
                       (:wat::core::do
                         (:wat::core::Result/expect -> :wat::core::nil
                           (:wat::kernel::send server-tx
                             (:counter::UserResp::Ok 0))
                           "handle-user3: server-tx disconnected on Reset")
                         (:counter::dispatch3
                           admin-wire-rx admin-resp-tx
                           (:counter::registry-update-state registry-vec idx 0)
                           next-id))))
                   ;; Mismatch: emit AccessDenied; continue dispatch
                   (:wat::core::let
                     [_denied
                        (:wat::core::Result/expect -> :wat::core::nil
                          (:wat::kernel::send server-tx
                            (:counter::UserResp::AccessDenied))
                          "handle-user3: server-tx disconnected on AccessDenied")]
                     (:counter::dispatch3
                       admin-wire-rx admin-resp-tx
                       registry-vec next-id))))
               ((:counter::Wire::Admin _wire-sid _req)
                 (:counter::dispatch3
                   admin-wire-rx admin-resp-tx
                   registry-vec next-id)))))
         (:wat::core::None
           ()))))

   ;; ─── Privileged wrappers: Admin ops ───────────────────────────────────────
   ;;
   ;; :counter::spawn-cap — creates server, returns Admin capability.
   ;;
   ;; No send/recv → returns Admin directly (no Result wrapper needed).
   ;; In production, mint server-id via :wat::telemetry::uuid::v4 for unguessability.
   (:wat::core::defn :counter::spawn-cap
     []
     -> :counter::Admin
     (:wat::core::let
       [thread   (:wat::kernel::spawn-thread
                   (:wat::core::fn
                     [admin-wire-rx <- :wat::kernel::Receiver<counter::Wire>
                      admin-resp-tx <- :wat::kernel::Sender<counter::AdminResp>]
                      -> :wat::core::nil
                     (:counter::dispatch3
                       admin-wire-rx admin-resp-tx
                       (:wat::core::Vector :counter::RegistryEntry)
                       0)))
        adm-tx   (:wat::kernel::Thread/input  thread)
        adm-rx   (:wat::kernel::Thread/output thread)]
       (:counter::Admin/new "server-counter-thread-0" adm-tx adm-rx thread)))

   ;; :counter::provision — sends Provision(initial), receives Provisioned, returns Client.
   ;;
   ;; Now returns Result<Client,ServiceError> — honest about transport errors.
   ;; Each send/recv site explicitly matches and propagates.
   ;;
   ;; Shape (two outer send/recv levels, one inner response-type level):
   ;;   send Err(chain) → Err(PeerDied(chain))
   ;;   recv Err(chain) → Err(PeerDied(chain))
   ;;   recv Ok(None)   → Err(Disconnected)
   ;;   recv Ok(Some resp):
   ;;     Provisioned(id, tx, rx) → Ok(Client/new ...)
   ;;     AccessDenied            → Err(AccessDenied)
   ;;     Deprovisioned / Stopped → assertion-failed! (programmer error)
   (:wat::core::defn :counter::provision
     [admin!  <- :counter::Admin
      initial <- :wat::core::i64]
     -> :wat::core::Result<counter::Client,counter::ServiceError>
     (:wat::core::let
       [adm-tx (:counter::Admin/admin-tx  admin!)
        adm-rx (:counter::Admin/admin-rx  admin!)
        sid    (:counter::Admin/server-id admin!)]
       (:wat::core::match
         (:wat::kernel::send adm-tx
           (:counter::Wire::Admin sid (:counter::AdminReq::Provision initial)))
         -> :wat::core::Result<counter::Client,counter::ServiceError>
         ((:wat::core::Ok _)
           (:wat::core::match (:wat::kernel::recv adm-rx)
             -> :wat::core::Result<counter::Client,counter::ServiceError>
             ((:wat::core::Ok opt)
               (:wat::core::match opt
                 -> :wat::core::Result<counter::Client,counter::ServiceError>
                 ((:wat::core::Some resp)
                   (:wat::core::match resp
                     -> :wat::core::Result<counter::Client,counter::ServiceError>
                     ((:counter::AdminResp::Provisioned id user-tx user-rx)
                       (:wat::core::Ok (:counter::Client/new sid id user-tx user-rx)))
                     ((:counter::AdminResp::AccessDenied)
                       (:wat::core::Err (:counter::ServiceError::AccessDenied)))
                     ((:counter::AdminResp::Deprovisioned _id)
                       (:wat::kernel::assertion-failed! "provision: expected Provisioned, got Deprovisioned" :wat::core::None :wat::core::None))
                     ((:counter::AdminResp::Stopped)
                       (:wat::kernel::assertion-failed! "provision: expected Provisioned, got Stopped" :wat::core::None :wat::core::None))))
                 (:wat::core::None
                   (:wat::core::Err (:counter::ServiceError::Disconnected)))))
             ((:wat::core::Err chain)
               (:wat::core::Err (:counter::ServiceError::PeerDied chain)))))
         ((:wat::core::Err chain)
           (:wat::core::Err (:counter::ServiceError::PeerDied chain))))))

   ;; :counter::deprovision — sends Deprovision, receives Deprovisioned ack.
   ;;
   ;; Now returns Result<nil,ServiceError>.
   ;; Reads client-id from Client capability (restricted accessor — :counter:: ok).
   ;; Reads server-id from Admin capability; embeds it in the Wire::Admin payload.
   (:wat::core::defn :counter::deprovision
     [admin!  <- :counter::Admin
      client! <- :counter::Client]
     -> :wat::core::Result<wat::core::nil,counter::ServiceError>
     (:wat::core::let
       [adm-tx (:counter::Admin/admin-tx  admin!)
        adm-rx (:counter::Admin/admin-rx  admin!)
        sid    (:counter::Admin/server-id admin!)
        cid    (:counter::Client/client-id client!)]
       (:wat::core::match
         (:wat::kernel::send adm-tx
           (:counter::Wire::Admin sid (:counter::AdminReq::Deprovision cid)))
         -> :wat::core::Result<wat::core::nil,counter::ServiceError>
         ((:wat::core::Ok _)
           (:wat::core::match (:wat::kernel::recv adm-rx)
             -> :wat::core::Result<wat::core::nil,counter::ServiceError>
             ((:wat::core::Ok opt)
               (:wat::core::match opt
                 -> :wat::core::Result<wat::core::nil,counter::ServiceError>
                 ((:wat::core::Some resp)
                   (:wat::core::match resp
                     -> :wat::core::Result<wat::core::nil,counter::ServiceError>
                     ((:counter::AdminResp::Deprovisioned _id)
                       (:wat::core::Ok ()))
                     ((:counter::AdminResp::AccessDenied)
                       (:wat::core::Err (:counter::ServiceError::AccessDenied)))
                     ((:counter::AdminResp::Provisioned _id _tx _rx)
                       (:wat::kernel::assertion-failed! "deprovision: expected Deprovisioned, got Provisioned" :wat::core::None :wat::core::None))
                     ((:counter::AdminResp::Stopped)
                       (:wat::kernel::assertion-failed! "deprovision: expected Deprovisioned, got Stopped" :wat::core::None :wat::core::None))))
                 (:wat::core::None
                   (:wat::core::Err (:counter::ServiceError::Disconnected)))))
             ((:wat::core::Err chain)
               (:wat::core::Err (:counter::ServiceError::PeerDied chain)))))
         ((:wat::core::Err chain)
           (:wat::core::Err (:counter::ServiceError::PeerDied chain))))))

   ;; :counter::stop — sends Stop, receives Stopped, drains thread.
   ;;
   ;; Now returns Result<nil,ServiceError>.
   ;;
   ;; SERVICE-PROGRAMS lockstep absorbed inside this wrapper:
   ;;   inner-let: extracts and uses admin-tx (Sender<Wire>) + admin-rx + thread
   ;;              → adm-tx clone drops at inner-let exit; returns Result<Thread,ServiceError>
   ;;   outer-let: holds only `result` (Result<Thread,ServiceError>)
   ;;              matches result → on Ok(thr), calls drain-and-join
   ;;                             → on Err(e), propagates error
   ;;
   ;; Note: Admin struct's internal adm-tx clone remains alive until admin! drops
   ;; (in the caller's scope). Server has already exited cleanly by that point
   ;; (it returned nil after sending Stopped), so drain-and-join succeeds immediately.
   (:wat::core::defn :counter::stop
     [admin! <- :counter::Admin]
     -> :wat::core::Result<wat::core::nil,counter::ServiceError>
     (:wat::core::let
       [result
          (:wat::core::let
            [adm-tx (:counter::Admin/admin-tx  admin!)
             adm-rx (:counter::Admin/admin-rx  admin!)
             sid    (:counter::Admin/server-id admin!)
             thr    (:counter::Admin/thread    admin!)]
            ;; inner-let: send Stop + recv Stopped + return thread or error
            (:wat::core::match
              (:wat::kernel::send adm-tx
                (:counter::Wire::Admin sid (:counter::AdminReq::Stop)))
              -> :wat::core::Result<wat::kernel::Thread<counter::Wire,counter::AdminResp>,counter::ServiceError>
              ((:wat::core::Ok _)
                (:wat::core::match (:wat::kernel::recv adm-rx)
                  -> :wat::core::Result<wat::kernel::Thread<counter::Wire,counter::AdminResp>,counter::ServiceError>
                  ((:wat::core::Ok opt)
                    (:wat::core::match opt
                      -> :wat::core::Result<wat::kernel::Thread<counter::Wire,counter::AdminResp>,counter::ServiceError>
                      ((:wat::core::Some _)
                        ;; received Stopped (any AdminResp is fine — server sent Stopped)
                        (:wat::core::Ok thr))
                      (:wat::core::None
                        (:wat::core::Err (:counter::ServiceError::Disconnected)))))
                  ((:wat::core::Err chain)
                    (:wat::core::Err (:counter::ServiceError::PeerDied chain)))))
              ((:wat::core::Err chain)
                (:wat::core::Err (:counter::ServiceError::PeerDied chain)))))]
       ;; adm-tx clone dropped at inner-let exit; outer matches result
       (:wat::core::match result
         -> :wat::core::Result<wat::core::nil,counter::ServiceError>
         ((:wat::core::Ok thr)
           (:wat::core::match (:wat::kernel::Thread/drain-and-join thr)
             -> :wat::core::Result<wat::core::nil,counter::ServiceError>
             ((:wat::core::Ok _)
               (:wat::core::Ok ()))
             ((:wat::core::Err chain)
               (:wat::core::Err (:counter::ServiceError::PeerDied chain)))))
         ((:wat::core::Err e)
           (:wat::core::Err e)))))

   ;; ─── Privileged wrappers: User ops ────────────────────────────────────────
   ;;
   ;; Each user wrapper reads user-tx, user-rx, server-id, and client-id from
   ;; Client (restricted accessors — :counter:: namespace matches [:counter::] whitelist).
   ;; Constructs Wire::User with server-id as the secret witness + client-id as routing key.
   ;; Returns Result<i64,ServiceError> — propagates send/recv errors as typed variants.
   ;;
   ;; Shape:
   ;;   send Err(chain) → Err(PeerDied(chain))
   ;;   recv Err(chain) → Err(PeerDied(chain))
   ;;   recv Ok(None)   → Err(Disconnected)
   ;;   recv Ok(Some(Value v)) | Ok(Some(Ok v)) → Ok(v)
   ;;   recv Ok(Some(AccessDenied))              → Err(AccessDenied)

   (:wat::core::defn :counter::get
     [client! <- :counter::Client]
     -> :wat::core::Result<wat::core::i64,counter::ServiceError>
     (:wat::core::let
       [utx (:counter::Client/user-tx   client!)
        urx (:counter::Client/user-rx   client!)
        sid (:counter::Client/server-id client!)
        cid (:counter::Client/client-id client!)]
       (:wat::core::match
         (:wat::kernel::send utx (:counter::Wire::User sid cid (:counter::UserReq::Get)))
         -> :wat::core::Result<wat::core::i64,counter::ServiceError>
         ((:wat::core::Ok _)
           (:wat::core::match (:wat::kernel::recv urx)
             -> :wat::core::Result<wat::core::i64,counter::ServiceError>
             ((:wat::core::Ok opt)
               (:wat::core::match opt
                 -> :wat::core::Result<wat::core::i64,counter::ServiceError>
                 ((:wat::core::Some resp)
                   (:wat::core::match resp
                     -> :wat::core::Result<wat::core::i64,counter::ServiceError>
                     ((:counter::UserResp::Value v) (:wat::core::Ok v))
                     ((:counter::UserResp::Ok    v) (:wat::core::Ok v))
                     ((:counter::UserResp::AccessDenied)
                       (:wat::core::Err (:counter::ServiceError::AccessDenied)))))
                 (:wat::core::None
                   (:wat::core::Err (:counter::ServiceError::Disconnected)))))
             ((:wat::core::Err chain)
               (:wat::core::Err (:counter::ServiceError::PeerDied chain)))))
         ((:wat::core::Err chain)
           (:wat::core::Err (:counter::ServiceError::PeerDied chain))))))

   (:wat::core::defn :counter::increment
     [client! <- :counter::Client
      n       <- :wat::core::i64]
     -> :wat::core::Result<wat::core::i64,counter::ServiceError>
     (:wat::core::let
       [utx (:counter::Client/user-tx   client!)
        urx (:counter::Client/user-rx   client!)
        sid (:counter::Client/server-id client!)
        cid (:counter::Client/client-id client!)]
       (:wat::core::match
         (:wat::kernel::send utx (:counter::Wire::User sid cid (:counter::UserReq::Increment n)))
         -> :wat::core::Result<wat::core::i64,counter::ServiceError>
         ((:wat::core::Ok _)
           (:wat::core::match (:wat::kernel::recv urx)
             -> :wat::core::Result<wat::core::i64,counter::ServiceError>
             ((:wat::core::Ok opt)
               (:wat::core::match opt
                 -> :wat::core::Result<wat::core::i64,counter::ServiceError>
                 ((:wat::core::Some resp)
                   (:wat::core::match resp
                     -> :wat::core::Result<wat::core::i64,counter::ServiceError>
                     ((:counter::UserResp::Ok    v) (:wat::core::Ok v))
                     ((:counter::UserResp::Value v) (:wat::core::Ok v))
                     ((:counter::UserResp::AccessDenied)
                       (:wat::core::Err (:counter::ServiceError::AccessDenied)))))
                 (:wat::core::None
                   (:wat::core::Err (:counter::ServiceError::Disconnected)))))
             ((:wat::core::Err chain)
               (:wat::core::Err (:counter::ServiceError::PeerDied chain)))))
         ((:wat::core::Err chain)
           (:wat::core::Err (:counter::ServiceError::PeerDied chain))))))

   (:wat::core::defn :counter::reset
     [client! <- :counter::Client]
     -> :wat::core::Result<wat::core::i64,counter::ServiceError>
     (:wat::core::let
       [utx (:counter::Client/user-tx   client!)
        urx (:counter::Client/user-rx   client!)
        sid (:counter::Client/server-id client!)
        cid (:counter::Client/client-id client!)]
       (:wat::core::match
         (:wat::kernel::send utx (:counter::Wire::User sid cid (:counter::UserReq::Reset)))
         -> :wat::core::Result<wat::core::i64,counter::ServiceError>
         ((:wat::core::Ok _)
           (:wat::core::match (:wat::kernel::recv urx)
             -> :wat::core::Result<wat::core::i64,counter::ServiceError>
             ((:wat::core::Ok opt)
               (:wat::core::match opt
                 -> :wat::core::Result<wat::core::i64,counter::ServiceError>
                 ((:wat::core::Some resp)
                   (:wat::core::match resp
                     -> :wat::core::Result<wat::core::i64,counter::ServiceError>
                     ((:counter::UserResp::Ok    v) (:wat::core::Ok v))
                     ((:counter::UserResp::Value v) (:wat::core::Ok v))
                     ((:counter::UserResp::AccessDenied)
                       (:wat::core::Err (:counter::ServiceError::AccessDenied)))))
                 (:wat::core::None
                   (:wat::core::Err (:counter::ServiceError::Disconnected)))))
             ((:wat::core::Err chain)
               (:wat::core::Err (:counter::ServiceError::PeerDied chain)))))
         ((:wat::core::Err chain)
           (:wat::core::Err (:counter::ServiceError::PeerDied chain))))))

   ;; ─── Forge demonstration: adversarial test ───────────────────────────────
   ;;
   ;; Now returns Result<nil,ServiceError> — demonstrates the AccessDenied Err path.
   ;;
   ;; Builds Wire with a BAD server-id and sends it via the admin channel.
   ;; Server responds AccessDenied; this wrapper now RETURNS that as Err(AccessDenied)
   ;; rather than panicking. Callers pattern-match the Err variant.
   ;;
   ;; NOTE: Code outside :counter::* CANNOT construct Wire variants directly —
   ;; Wire variants are enum forms, not struct-restricted, so they ARE constructible
   ;; by any code that has access to the enum definition... but the Sender<Wire>
   ;; that delivers to the server IS restricted (inside Admin/Client structs).
   ;; No code outside :counter::* can obtain a Sender<Wire> to send a forged Wire.
   ;;
   ;; WITHIN :counter::* we CAN construct a deliberately wrong Wire to test the
   ;; rejection path. This is a contrived adversarial test FROM WITHIN the privileged
   ;; namespace that documents what happens if a wrapper bug embeds the wrong server-id.
   (:wat::core::defn :counter::test-forge-admin-rejection
     [admin! <- :counter::Admin]
     -> :wat::core::Result<wat::core::nil,counter::ServiceError>
     (:wat::core::let
       [adm-tx (:counter::Admin/admin-tx admin!)
        adm-rx (:counter::Admin/admin-rx admin!)]
       (:wat::core::match
         (:wat::kernel::send adm-tx
           (:counter::Wire::Admin "WRONG-SERVER-ID" (:counter::AdminReq::Provision 99)))
         -> :wat::core::Result<wat::core::nil,counter::ServiceError>
         ((:wat::core::Ok _)
           (:wat::core::match (:wat::kernel::recv adm-rx)
             -> :wat::core::Result<wat::core::nil,counter::ServiceError>
             ((:wat::core::Ok opt)
               (:wat::core::match opt
                 -> :wat::core::Result<wat::core::nil,counter::ServiceError>
                 ((:wat::core::Some resp)
                   (:wat::core::match resp
                     -> :wat::core::Result<wat::core::nil,counter::ServiceError>
                     ((:counter::AdminResp::AccessDenied)
                       (:wat::core::Err (:counter::ServiceError::AccessDenied)))
                     ((:counter::AdminResp::Provisioned _id _tx _rx)
                       (:wat::kernel::assertion-failed! "forge-test: server should have rejected WRONG-SERVER-ID, got Provisioned" :wat::core::None :wat::core::None))
                     ((:counter::AdminResp::Deprovisioned _id)
                       (:wat::kernel::assertion-failed! "forge-test: server should have rejected WRONG-SERVER-ID, got Deprovisioned" :wat::core::None :wat::core::None))
                     ((:counter::AdminResp::Stopped)
                       (:wat::kernel::assertion-failed! "forge-test: server should have rejected WRONG-SERVER-ID, got Stopped" :wat::core::None :wat::core::None))))
                 (:wat::core::None
                   (:wat::core::Err (:counter::ServiceError::Disconnected)))))
             ((:wat::core::Err chain)
               (:wat::core::Err (:counter::ServiceError::PeerDied chain)))))
         ((:wat::core::Err chain)
           (:wat::core::Err (:counter::ServiceError::PeerDied chain)))))))

  ;; ─── Test body ─────────────────────────────────────────────────────────────
  ;;
  ;; Exercises ALL ops via capability wrappers ONLY.
  ;; This namespace is :counter-service::capability-N3 — NOT :counter::*.
  ;; The test body CANNOT:
  ;;   - call :counter::Admin/new or :counter::Client/new (restricted ctor)
  ;;   - call :counter::Admin/server-id, :counter::Admin/admin-tx, etc. (restricted accessors)
  ;;   - call :counter::Client/server-id, :counter::Client/client-id, etc.
  ;;
  ;; SERVICE-PROGRAMS lockstep is absorbed into :counter::stop — test body
  ;; does NOT need inner/outer let structure. admin! and client-X! are
  ;; struct types (not raw Senders), so the scope-deadlock checker does
  ;; not fire on them.
  ;;
  ;; All Result-returning wrappers are pattern-matched. Happy-path assertions
  ;; extract Ok values explicitly. Err paths are demonstrated:
  ;;   - AccessDenied: forge test → Result<nil,ServiceError> → match Err(AccessDenied)
  ;;   - PeerDied/Disconnected: call get AFTER stop → match Err
  ;;
  ;; Scenario:
  ;;   1.  Spawn server → admin!
  ;;   2.  Provision 3 users: initial 10, 100, 0 → client-a!, client-b!, client-c!
  ;;   3.  Increment a by 5  → 15
  ;;   4.  Increment b by 50 → 150
  ;;   5.  Get c             → 0
  ;;   6.  Deprovision b
  ;;   7.  Get a             → 15  (still alive after b deprovisioned)
  ;;   8.  Reset c           → 0   (still alive)
  ;;   9.  Forge test: send wrong-server-id to admin; assert Err(AccessDenied) returned
  ;;  10.  Stop admin!       → drains thread inside wrapper; returns Ok(nil)
  ;;  11.  Err path: call get on client-a! AFTER stop → Err(PeerDied) or Disconnected
  (:wat::core::let
    [admin!    (:counter::spawn-cap)

     ;; Step 2: provision — each returns Result<Client,ServiceError>; match Ok
     client-a-res (:counter::provision admin! 10)
     client-a!
       (:wat::core::match client-a-res -> :counter::Client
         ((:wat::core::Ok c) c)
         ((:wat::core::Err _e)
           (:wat::kernel::assertion-failed! "provision a: expected Ok" :wat::core::None :wat::core::None)))

     client-b-res (:counter::provision admin! 100)
     client-b!
       (:wat::core::match client-b-res -> :counter::Client
         ((:wat::core::Ok c) c)
         ((:wat::core::Err _e)
           (:wat::kernel::assertion-failed! "provision b: expected Ok" :wat::core::None :wat::core::None)))

     client-c-res (:counter::provision admin! 0)
     client-c!
       (:wat::core::match client-c-res -> :counter::Client
         ((:wat::core::Ok c) c)
         ((:wat::core::Err _e)
           (:wat::kernel::assertion-failed! "provision c: expected Ok" :wat::core::None :wat::core::None)))

     ;; Step 3: increment a — returns Result<i64,ServiceError>; match Ok; assert
     a1-res (:counter::increment client-a! 5)
     a1
       (:wat::core::match a1-res -> :wat::core::i64
         ((:wat::core::Ok v) v)
         ((:wat::core::Err _e)
           (:wat::kernel::assertion-failed! "increment a: expected Ok" :wat::core::None :wat::core::None)))
     _  (:wat::test::assert-eq a1 15)

     ;; Step 4: increment b
     b1-res (:counter::increment client-b! 50)
     b1
       (:wat::core::match b1-res -> :wat::core::i64
         ((:wat::core::Ok v) v)
         ((:wat::core::Err _e)
           (:wat::kernel::assertion-failed! "increment b: expected Ok" :wat::core::None :wat::core::None)))
     _  (:wat::test::assert-eq b1 150)

     ;; Step 5: get c
     c1-res (:counter::get client-c!)
     c1
       (:wat::core::match c1-res -> :wat::core::i64
         ((:wat::core::Ok v) v)
         ((:wat::core::Err _e)
           (:wat::kernel::assertion-failed! "get c: expected Ok" :wat::core::None :wat::core::None)))
     _  (:wat::test::assert-eq c1 0)

     ;; Step 6: deprovision b — returns Result<nil,ServiceError>; assert Ok
     dep-res (:counter::deprovision admin! client-b!)
     _dep
       (:wat::core::match dep-res -> :wat::core::nil
         ((:wat::core::Ok _) ())
         ((:wat::core::Err _e)
           (:wat::kernel::assertion-failed! "deprovision b: expected Ok" :wat::core::None :wat::core::None)))

     ;; Step 7: get a (still alive after b deprovisioned)
     a2-res (:counter::get client-a!)
     a2
       (:wat::core::match a2-res -> :wat::core::i64
         ((:wat::core::Ok v) v)
         ((:wat::core::Err _e)
           (:wat::kernel::assertion-failed! "get a after deprovision b: expected Ok" :wat::core::None :wat::core::None)))
     _  (:wat::test::assert-eq a2 15)

     ;; Step 8: reset c (still alive)
     c2-res (:counter::reset client-c!)
     c2
       (:wat::core::match c2-res -> :wat::core::i64
         ((:wat::core::Ok v) v)
         ((:wat::core::Err _e)
           (:wat::kernel::assertion-failed! "reset c: expected Ok" :wat::core::None :wat::core::None)))
     _  (:wat::test::assert-eq c2 0)

     ;; Step 9: Forge test — adversarial helper returns Err(AccessDenied)
     ;; Server correctly rejected the wrong-server-id; wrapper returns typed error.
     ;; This demonstrates the AccessDenied Err path: match the Result and assert variant.
     forge-res (:counter::test-forge-admin-rejection admin!)
     _forge
       (:wat::core::match forge-res -> :wat::core::nil
         ((:wat::core::Err err)
           (:wat::core::match err -> :wat::core::nil
             ((:counter::ServiceError::AccessDenied) ())   ;; expected — forge correctly rejected
             ((:counter::ServiceError::PeerDied _chain)
               (:wat::kernel::assertion-failed! "forge: expected AccessDenied, got PeerDied" :wat::core::None :wat::core::None))
             ((:counter::ServiceError::Disconnected)
               (:wat::kernel::assertion-failed! "forge: expected AccessDenied, got Disconnected" :wat::core::None :wat::core::None))))
         ((:wat::core::Ok _)
           (:wat::kernel::assertion-failed! "forge: expected Err(AccessDenied), got Ok" :wat::core::None :wat::core::None)))

     ;; Step 10: Stop — returns Result<nil,ServiceError>; assert Ok
     stop-res (:counter::stop admin!)
     _stop
       (:wat::core::match stop-res -> :wat::core::nil
         ((:wat::core::Ok _) ())
         ((:wat::core::Err _e)
           (:wat::kernel::assertion-failed! "stop: expected Ok" :wat::core::None :wat::core::None)))

     ;; Step 11: Err path after stop — server thread has exited; user-tx is disconnected.
     ;; Calling get on client-a! now: send to user-tx will return Err(ChannelDisconnected chain).
     ;; This demonstrates the PeerDied Err path at the thread tier.
     after-stop-res (:counter::get client-a!)
     _after-stop
       (:wat::core::match after-stop-res -> :wat::core::nil
         ((:wat::core::Err err)
           ;; After stop, we expect either PeerDied (send to dead server-tx)
           ;; or Disconnected (recv sees None because server-tx dropped).
           ;; Both are legitimate — demonstrates that wrappers surface errors as Result.
           (:wat::core::match err -> :wat::core::nil
             ((:counter::ServiceError::PeerDied _chain) ())     ;; send failed — ChannelDisconnected
             ((:counter::ServiceError::Disconnected) ())        ;; recv saw None — sender dropped
             ((:counter::ServiceError::AccessDenied)
               (:wat::kernel::assertion-failed! "after-stop get: unexpected AccessDenied" :wat::core::None :wat::core::None))))
         ((:wat::core::Ok _)
           (:wat::kernel::assertion-failed! "after-stop get: expected Err (server stopped), got Ok" :wat::core::None :wat::core::None)))]
    :wat::core::nil))
