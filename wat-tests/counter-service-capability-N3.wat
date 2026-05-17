;; wat-tests/counter-service-capability-N3.wat — Capability-wrapped multi-user counter.
;;
;; Arc 203 slice 3e — fifth stepping stone (in-place update of slice 3c).
;; Wires server-id validation from dead-data into live validation.
;;
;; Previously (slice 3c): server-id stored in Admin + Client structs but never validated.
;; Now (slice 3e): server-id embedded in every Wire payload; server validates on receipt.
;;
;; Wraps slice 3b's bare-channel multi-user flow in struct-restricted capability values.
;;
;; Extends slice 3b (counter-service-thread-N3.wat) with:
;;   :counter::Admin   — struct-restricted handle for the admin role
;;                       (holds server-id, admin-tx, admin-rx, thread)
;;   :counter::Client  — struct-restricted handle for each provisioned user
;;                       (holds server-id, client-id, user-tx, user-rx)
;;
;; Design choice FIXED: unified Wire enum continues. Split AdminWire/UserWire
;; disqualified per BRIEF (select is ∀T; splitting forces two server loops or
;; polling). Behavior enforces protocol separation; struct-restricted enforces
;; capability minting.
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
   ;; server-id is now the FIRST field on both variants — the secret witness.
   ;; Every wrapper embeds the capability's server-id when constructing Wire.
   ;; The server validates server-id against its own before processing.
   (:wat::core::enum :counter::Wire
     (Admin (server-id :wat::core::String) (req :counter::AdminReq))
     (User  (server-id :wat::core::String) (id :wat::core::String) (req :counter::UserReq)))

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
   ;; Holds:
   ;;   server-id — constant string naming this server instance
   ;;   admin-tx  — Sender<Wire>: admin sends requests to server
   ;;   admin-rx  — Receiver<AdminResp>: admin receives responses from server
   ;;   thread    — Thread<Wire,AdminResp>: server thread handle (for drain-and-join)
   ;;
   ;; No public fields — callers hold Admin opaquely; all ops go via wrappers.
   ;;
   ;; Inner type aliases in struct-restricted must NOT include field-per-line
   ;; grouping syntax beyond the 4-item chunks: [wlist] field <- :Type.
   ;; Each field is exactly one group of 4 items in the restricted section.
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
   ;; Holds:
   ;;   server-id  — identifies which server issued this client
   ;;   client-id  — server-minted unique id for this user slot
   ;;   user-tx    — Sender<Wire>: user sends requests to server
   ;;   user-rx    — Receiver<UserResp>: user receives responses from server
   ;;
   ;; No public fields.
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
   ;; Server-id uses constant "server-counter-thread-0" (no telemetry dep needed
   ;; for this proof; uniqueness irrelevant in single-server test).
   ;; In production, mint via :wat::telemetry::uuid::v4 for unguessability.
   ;;
   ;; Thread<I=Wire, O=AdminResp>:
   ;;   Thread/input(thread)  = Sender<Wire>       = admin-tx
   ;;   Thread/output(thread) = Receiver<AdminResp> = admin-rx
   ;;
   ;; Admin struct fields (all restricted to :counter::*):
   ;;   server-id, admin-tx, admin-rx, thread
   ;;
   ;; Thread is stored IN Admin so :counter::stop can drain-and-join it
   ;; without exposing Thread to the caller.
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
   ;; Reads restricted fields (admin-tx, admin-rx, server-id) from Admin struct.
   ;; Constructs Wire::Admin with the admin's server-id embedded as the secret witness.
   ;; Constructs and returns Client capability with restricted fields populated.
   ;;
   ;; Note on accessor semantics: accessors clone the field value (channel ends
   ;; are Arc-wrapped in the runtime). The Admin struct retains its internal copy
   ;; of admin-tx. After provision, admin's admin-tx is still live for further ops.
   (:wat::core::defn :counter::provision
     [admin!  <- :counter::Admin
      initial <- :wat::core::i64]
     -> :counter::Client
     (:wat::core::let
       [adm-tx  (:counter::Admin/admin-tx  admin!)
        adm-rx  (:counter::Admin/admin-rx  admin!)
        sid     (:counter::Admin/server-id admin!)
        _sent
          (:wat::core::Result/expect -> :wat::core::nil
            (:wat::kernel::send adm-tx
              (:counter::Wire::Admin sid (:counter::AdminReq::Provision initial)))
            "provision: admin-tx disconnected")
        resp
          (:wat::core::Option/expect -> :counter::AdminResp
            (:wat::core::Result/expect -> :wat::core::Option<counter::AdminResp>
              (:wat::kernel::recv adm-rx)
              "provision: recv peer died")
            "provision: clean disconnect")]
       (:wat::core::match resp -> :counter::Client
         ((:counter::AdminResp::Provisioned id user-tx user-rx)
           (:counter::Client/new sid id user-tx user-rx))
         ((:counter::AdminResp::Deprovisioned _id)
           (:wat::kernel::assertion-failed! "provision: expected Provisioned, got Deprovisioned" :wat::core::None :wat::core::None))
         ((:counter::AdminResp::Stopped)
           (:wat::kernel::assertion-failed! "provision: expected Provisioned, got Stopped" :wat::core::None :wat::core::None))
         ((:counter::AdminResp::AccessDenied)
           (:wat::kernel::assertion-failed! "provision: server refused — server-id mismatch" :wat::core::None :wat::core::None)))))

   ;; :counter::deprovision — sends Deprovision, receives Deprovisioned ack, returns nil.
   ;;
   ;; Reads client-id from Client capability (restricted accessor — :counter:: ok).
   ;; Reads server-id from Admin capability; embeds it in the Wire::Admin payload.
   ;; Sends Deprovision(client-id) via admin-tx; receives Deprovisioned ack.
   (:wat::core::defn :counter::deprovision
     [admin!  <- :counter::Admin
      client! <- :counter::Client]
     -> :wat::core::nil
     (:wat::core::let
       [adm-tx  (:counter::Admin/admin-tx  admin!)
        adm-rx  (:counter::Admin/admin-rx  admin!)
        sid     (:counter::Admin/server-id admin!)
        cid     (:counter::Client/client-id client!)
        _sent
          (:wat::core::Result/expect -> :wat::core::nil
            (:wat::kernel::send adm-tx
              (:counter::Wire::Admin sid (:counter::AdminReq::Deprovision cid)))
            "deprovision: admin-tx disconnected")
        resp
          (:wat::core::Option/expect -> :counter::AdminResp
            (:wat::core::Result/expect -> :wat::core::Option<counter::AdminResp>
              (:wat::kernel::recv adm-rx)
              "deprovision: recv peer died")
            "deprovision: clean disconnect")]
       (:wat::core::match resp -> :wat::core::nil
         ((:counter::AdminResp::Deprovisioned _id) ())
         ((:counter::AdminResp::Provisioned _id _tx _rx)
           (:wat::kernel::assertion-failed! "deprovision: expected Deprovisioned, got Provisioned" :wat::core::None :wat::core::None))
         ((:counter::AdminResp::Stopped)
           (:wat::kernel::assertion-failed! "deprovision: expected Deprovisioned, got Stopped" :wat::core::None :wat::core::None))
         ((:counter::AdminResp::AccessDenied)
           (:wat::kernel::assertion-failed! "deprovision: server refused — server-id mismatch" :wat::core::None :wat::core::None)))))

   ;; :counter::stop — sends Stop, receives Stopped, drains thread, returns nil.
   ;;
   ;; Reads server-id from Admin capability; embeds it in the Wire::Admin payload.
   ;;
   ;; SERVICE-PROGRAMS lockstep absorbed inside this wrapper:
   ;;   inner-let: extracts and uses admin-tx (Sender<Wire>) + admin-rx + thread
   ;;              → adm-tx clone drops at inner-let exit
   ;;   outer-let: holds only `thread` (Thread type, not Sender) → safe to drain-and-join
   ;;
   ;; Note: Admin struct's internal adm-tx clone remains alive until admin! drops
   ;; (in the caller's scope). Server has already exited cleanly by that point
   ;; (it returned nil after sending Stopped), so drain-and-join succeeds immediately.
   (:wat::core::defn :counter::stop
     [admin! <- :counter::Admin]
     -> :wat::core::nil
     (:wat::core::let
       [thread
          (:wat::core::let
            [adm-tx  (:counter::Admin/admin-tx  admin!)
             adm-rx  (:counter::Admin/admin-rx  admin!)
             sid     (:counter::Admin/server-id admin!)
             thr     (:counter::Admin/thread    admin!)
             _sent
               (:wat::core::Result/expect -> :wat::core::nil
                 (:wat::kernel::send adm-tx
                   (:counter::Wire::Admin sid (:counter::AdminReq::Stop)))
                 "stop: admin-tx disconnected")
             _resp
               (:wat::core::Option/expect -> :counter::AdminResp
                 (:wat::core::Result/expect -> :wat::core::Option<counter::AdminResp>
                   (:wat::kernel::recv adm-rx)
                   "stop: recv peer died")
                 "stop: clean disconnect")]
            ;; adm-tx clone drops at inner-let exit; thr returned to outer
            thr)
        _drained
          (:wat::core::Result/expect -> :wat::core::nil
            (:wat::kernel::Thread/drain-and-join thread)
            "stop: thread died")]
       ()))

   ;; ─── Privileged wrappers: User ops ────────────────────────────────────────
   ;;
   ;; Each user wrapper reads user-tx, user-rx, server-id, and client-id from
   ;; Client (restricted accessors — :counter:: namespace matches [:counter::] whitelist).
   ;; Constructs Wire::User with server-id as the secret witness + client-id as routing key.
   ;; Sends the appropriate Wire::User variant; receives UserResp; extracts value.

   (:wat::core::defn :counter::get
     [client! <- :counter::Client]
     -> :wat::core::i64
     (:wat::core::let
       [utx  (:counter::Client/user-tx  client!)
        urx  (:counter::Client/user-rx  client!)
        sid  (:counter::Client/server-id client!)
        cid  (:counter::Client/client-id client!)
        _sent
          (:wat::core::Result/expect -> :wat::core::nil
            (:wat::kernel::send utx (:counter::Wire::User sid cid (:counter::UserReq::Get)))
            "get: user-tx disconnected")
        resp
          (:wat::core::Option/expect -> :counter::UserResp
            (:wat::core::Result/expect -> :wat::core::Option<counter::UserResp>
              (:wat::kernel::recv urx)
              "get: recv peer died")
            "get: clean disconnect")]
       (:wat::core::match resp -> :wat::core::i64
         ((:counter::UserResp::Value v) v)
         ((:counter::UserResp::Ok    v) v)
         ((:counter::UserResp::AccessDenied)
           (:wat::kernel::assertion-failed! "get: server refused — server-id mismatch" :wat::core::None :wat::core::None)))))

   (:wat::core::defn :counter::increment
     [client! <- :counter::Client
      n       <- :wat::core::i64]
     -> :wat::core::i64
     (:wat::core::let
       [utx  (:counter::Client/user-tx  client!)
        urx  (:counter::Client/user-rx  client!)
        sid  (:counter::Client/server-id client!)
        cid  (:counter::Client/client-id client!)
        _sent
          (:wat::core::Result/expect -> :wat::core::nil
            (:wat::kernel::send utx (:counter::Wire::User sid cid (:counter::UserReq::Increment n)))
            "increment: user-tx disconnected")
        resp
          (:wat::core::Option/expect -> :counter::UserResp
            (:wat::core::Result/expect -> :wat::core::Option<counter::UserResp>
              (:wat::kernel::recv urx)
              "increment: recv peer died")
            "increment: clean disconnect")]
       (:wat::core::match resp -> :wat::core::i64
         ((:counter::UserResp::Ok    v) v)
         ((:counter::UserResp::Value v) v)
         ((:counter::UserResp::AccessDenied)
           (:wat::kernel::assertion-failed! "increment: server refused — server-id mismatch" :wat::core::None :wat::core::None)))))

   (:wat::core::defn :counter::reset
     [client! <- :counter::Client]
     -> :wat::core::i64
     (:wat::core::let
       [utx  (:counter::Client/user-tx  client!)
        urx  (:counter::Client/user-rx  client!)
        sid  (:counter::Client/server-id client!)
        cid  (:counter::Client/client-id client!)
        _sent
          (:wat::core::Result/expect -> :wat::core::nil
            (:wat::kernel::send utx (:counter::Wire::User sid cid (:counter::UserReq::Reset)))
            "reset: user-tx disconnected")
        resp
          (:wat::core::Option/expect -> :counter::UserResp
            (:wat::core::Result/expect -> :wat::core::Option<counter::UserResp>
              (:wat::kernel::recv urx)
              "reset: recv peer died")
            "reset: clean disconnect")]
       (:wat::core::match resp -> :wat::core::i64
         ((:counter::UserResp::Ok    v) v)
         ((:counter::UserResp::Value v) v)
         ((:counter::UserResp::AccessDenied)
           (:wat::kernel::assertion-failed! "reset: server refused — server-id mismatch" :wat::core::None :wat::core::None)))))

   ;; ─── Forge demonstration: adversarial test ───────────────────────────────
   ;;
   ;; NOTE: Code outside :counter::* CANNOT construct Wire variants directly —
   ;; Wire variants are enum forms, not struct-restricted, so they ARE constructible
   ;; by any code that has access to the enum definition... but the Sender<Wire>
   ;; that delivers to the server IS restricted (inside Admin/Client structs).
   ;; No code outside :counter::* can obtain a Sender<Wire> to send a forged Wire.
   ;;
   ;; WITHIN :counter::* we CAN construct a deliberately wrong Wire to test the
   ;; rejection path. The helper below intentionally builds Wire with a BAD server-id
   ;; and sends it via the admin channel; expects AccessDenied response.
   ;;
   ;; This is a contrived adversarial test FROM WITHIN the privileged namespace.
   ;; It documents what happens if a wrapper bug accidentally embeds the wrong id.
   (:wat::core::defn :counter::test-forge-admin-rejection
     [admin! <- :counter::Admin]
     -> :wat::core::nil
     (:wat::core::let
       [adm-tx  (:counter::Admin/admin-tx  admin!)
        adm-rx  (:counter::Admin/admin-rx  admin!)
        ;; Intentionally WRONG server-id — simulates a forged or mis-routed message
        _sent
          (:wat::core::Result/expect -> :wat::core::nil
            (:wat::kernel::send adm-tx
              (:counter::Wire::Admin "WRONG-SERVER-ID" (:counter::AdminReq::Provision 99)))
            "forge-test: admin-tx disconnected")
        resp
          (:wat::core::Option/expect -> :counter::AdminResp
            (:wat::core::Result/expect -> :wat::core::Option<counter::AdminResp>
              (:wat::kernel::recv adm-rx)
              "forge-test: recv peer died")
            "forge-test: clean disconnect")]
       (:wat::core::match resp -> :wat::core::nil
         ((:counter::AdminResp::AccessDenied) ())   ;; expected — server correctly rejected
         ((:counter::AdminResp::Provisioned _id _tx _rx)
           (:wat::kernel::assertion-failed! "forge-test: server should have rejected WRONG-SERVER-ID, got Provisioned" :wat::core::None :wat::core::None))
         ((:counter::AdminResp::Deprovisioned _id)
           (:wat::kernel::assertion-failed! "forge-test: server should have rejected WRONG-SERVER-ID, got Deprovisioned" :wat::core::None :wat::core::None))
         ((:counter::AdminResp::Stopped)
           (:wat::kernel::assertion-failed! "forge-test: server should have rejected WRONG-SERVER-ID, got Stopped" :wat::core::None :wat::core::None))))))

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
  ;; Scenario:
  ;;   1. Spawn server → admin!
  ;;   2. Provision 3 users: initial 10, 100, 0 → client-a!, client-b!, client-c!
  ;;   3. Increment a by 5  → 15
  ;;   4. Increment b by 50 → 150
  ;;   5. Get c             → 0
  ;;   6. Deprovision b
  ;;   7. Get a             → 15  (still alive after b deprovisioned)
  ;;   8. Reset c           → 0   (still alive)
  ;;   9. Forge test: send wrong-server-id to admin; assert AccessDenied
  ;;  10. Stop admin!       → drains thread inside wrapper
  (:wat::core::let
    [admin!    (:counter::spawn-cap)
     client-a! (:counter::provision admin! 10)
     client-b! (:counter::provision admin! 100)
     client-c! (:counter::provision admin! 0)

     ;; Each user independent — ops affect only their own counter
     a1        (:counter::increment client-a! 5)
     _         (:wat::test::assert-eq a1 15)

     b1        (:counter::increment client-b! 50)
     _         (:wat::test::assert-eq b1 150)

     c1        (:counter::get client-c!)
     _         (:wat::test::assert-eq c1 0)

     ;; Deprovision client-b mid-flight; a and c continue
     _dep      (:counter::deprovision admin! client-b!)

     ;; client-a still works
     a2        (:counter::get client-a!)
     _         (:wat::test::assert-eq a2 15)

     ;; client-c still works
     c2        (:counter::reset client-c!)
     _         (:wat::test::assert-eq c2 0)

     ;; Forge test: adversarial helper sends Wire with WRONG server-id;
     ;; server should respond AccessDenied; wrapper asserts the rejection.
     _forge    (:counter::test-forge-admin-rejection admin!)

     ;; Admin Stop — sends Stop, receives Stopped, drains thread; all inside wrapper
     _stop     (:counter::stop admin!)]
    :wat::core::nil))
