;; wat-tests/counter-service-process-N3.wat — Capability-wrapped multi-user counter, process tier.
;;
;; Arc 203 slice 3e — fifth stepping stone (in-place update of slice 3d).
;; Wires server-id validation from dead-data into live validation at the process tier.
;;
;; Previously (slice 3d): server-id stored in AdminProc + ClientProc structs but never validated.
;; Now (slice 3e): server-id embedded in every Wire payload; subprocess validates on receipt.
;;
;; Same architecture as slice 3c (Admin + Client capability structs, struct-restricted,
;; dynamic Provision/Deprovision, per-user independent state) but at the PROCESS TIER:
;;   - Server runs as a subprocess (spawn-process + :wat::core::forms)
;;   - All communication via stdio multiplexed through Wire (parent→sub) + WireResp (sub→parent)
;;   - Admin holds :counter::AdminProc (server-id, peer!, proc!)
;;   - Each user holds :counter::ClientProc (server-id, client-id, peer!) — Arc-clone of admin's peer
;;   - Sequential request-response (single-threaded body; no concurrent demux needed)
;;
;; ─── PROCESS-TIER VALIDATION SEMANTICS ──────────────────────────────────────
;;
;; At the process tier, the server-id check is LOAD-BEARING — not merely defense in depth.
;; Unlike the thread tier (where the Sender<Wire> is enclosed in struct-restricted Admin/Client
;; so only :counter::* code can obtain it), the process tier has NO such transport-level
;; guarantee. The subprocess accepts data over stdio; a bug, a future multiplexer, or a
;; malicious caller could write bytes to the subprocess's stdin outside of the :counter::*
;; wrappers. The server CANNOT rely on transport-level identity.
;;
;; The secret-witness pattern makes validation STRUCTURAL: only a caller who obtained
;; an AdminProc or ClientProc capability (minted exclusively by :counter::spawn-proc and
;; :counter::provision-proc) knows the server-id. The subprocess validates every incoming
;; Wire against its own server-id. A Wire with the wrong server-id is rejected with
;; AccessDenied — the request is never processed.
;;
;; In production, mint server-id via :wat::telemetry::uuid::v4 for unguessability.
;; The constant string "server-counter-proc-0" demonstrates the validation flow.
;;
;; ─────────────────────────────────────────────────────────────────────────────
;;
;; Design choices:
;;   1. Multiplexed single-stream — all admin + user ops share one ProcessPeer
;;      Wire::User carries client-id so server can route; WireResp tags Admin vs User
;;   2. AdminProc holds proc! for drain-and-join in stop-proc (inner/outer let pattern)
;;   3. ClientProc.peer! is same peer as AdminProc.peer! (Arc-clone; accessor clones)
;;   4. -proc suffix avoids collision with slice 3c's thread-tier wrappers
;;
;; Lessons applied from prior slices (zero type-check fixups target):
;;   - Inner type aliases in :() are bare (no leading colon)
;;   - foldl not reduce
;;   - first/second tuple accessors only (registry entries are 2-tuples)
;;   - Process/drain-and-join not join-result
;;   - Inner/outer let for scope-deadlock in stop-proc
;;   - One-line :() annotations (no whitespace inside)
;;   - ProcessPeer/new(rx, tx) where rx = Receiver/from-pipe(stdout), tx = Sender/from-pipe(stdin)
;;   - Subprocess declares :user::main via (:wat::core::define ...) not defn
;;   - Two-level match required when matching enums carrying enum payloads

(:wat::test::deftest :counter-service::process-N3
  (;; ─── Wire enum (parent → subprocess) ───────────────────────────────────────
   ;; Wire::Admin and Wire::User now carry server-id as the first field.
   ;; The subprocess validates this server-id against its own before processing.
   ;; This is LOAD-BEARING at the process tier: shared ProcessPeer means
   ;; the server cannot rely on transport identity alone.
   (:wat::core::enum :counter::Wire
     (Admin (server-id :wat::core::String) (req :counter::AdminReq))
     (User  (server-id :wat::core::String) (id :wat::core::String) (req :counter::UserReq)))

   ;; ─── WireResp enum (subprocess → parent) ────────────────────────────────────
   ;; Tags Admin vs User responses so the parent can demux by category.
   (:wat::core::enum :counter::WireResp
     (Admin (resp :counter::AdminResp))
     (User  (resp :counter::UserResp)))

   ;; ─── AdminReq / AdminResp ────────────────────────────────────────────────────
   ;; AdminResp::Provisioned returns ONLY the minted id.
   ;; No channels at process tier; user ops go via the shared peer.
   (:wat::core::enum :counter::AdminReq
     (Provision   (initial :wat::core::i64))
     (Deprovision (id :wat::core::String))
     (Stop))

   (:wat::core::enum :counter::AdminResp
     (Provisioned  (id :wat::core::String))
     (Deprovisioned (id :wat::core::String))
     (Stopped)
     (AccessDenied))                          ;; server refused — server-id mismatch

   ;; ─── UserReq / UserResp ─────────────────────────────────────────────────────
   (:wat::core::enum :counter::UserReq
     (Get)
     (Increment (n :wat::core::i64))
     (Reset))

   (:wat::core::enum :counter::UserResp
     (Value (v :wat::core::i64))
     (Ok    (v :wat::core::i64))
     (AccessDenied))                          ;; server refused — server-id mismatch

   ;; ─── Capability structs ───────────────────────────────────────────────────────
   ;;
   ;; :counter::AdminProc — admin handle wrapping the shared ProcessPeer + Process.
   ;;   server-id — names this server instance
   ;;   peer!     — ProcessPeer<counter::WireResp, counter::Wire>:
   ;;                 parent reads WireResp from subprocess stdout (peer.rx)
   ;;                 parent writes Wire to subprocess stdin (peer.tx)
   ;;   proc!     — Process<counter::Wire, counter::WireResp>:
   ;;                 raw process handle; held so stop-proc can drain-and-join
   ;;
   ;; Minted ONLY by :counter::spawn-proc (constructor whitelist [:counter::]).
   ;; All fields restricted to :counter::* reads.
   (:wat::core::struct-restricted :counter::AdminProc
     [:counter::]
     ([:counter::] server-id <- :wat::core::String
      [:counter::] peer!     <- :wat::kernel::ProcessPeer<counter::WireResp,counter::Wire>
      [:counter::] proc!     <- :wat::kernel::Process<counter::Wire,counter::WireResp>)
     ())

   ;; :counter::ClientProc — per-user capability handle.
   ;;   server-id  — identifies which server issued this client
   ;;   client-id  — server-minted unique id for this user slot
   ;;   peer!      — shared ProcessPeer (Arc-clone of AdminProc's peer!)
   ;;
   ;; Minted ONLY by :counter::provision-proc (constructor whitelist [:counter::]).
   ;; All fields restricted to :counter::* reads.
   (:wat::core::struct-restricted :counter::ClientProc
     [:counter::]
     ([:counter::] server-id <- :wat::core::String
      [:counter::] client-id <- :wat::core::String
      [:counter::] peer!     <- :wat::kernel::ProcessPeer<counter::WireResp,counter::Wire>)
     ())

   ;; ─── Privileged wrappers ──────────────────────────────────────────────────────
   ;;
   ;; :counter::spawn-proc — spawns subprocess, builds ProcessPeer, returns AdminProc.
   ;;
   ;; Subprocess program declared inline via :wat::core::forms.
   ;; The subprocess declares its own independent copies of all enum types.
   ;; Same enum names → same EDN tags → interoperable across process boundary.
   ;;
   ;; The subprocess's own server-id = "server-counter-proc-0" (constant string).
   ;; In production, mint via :wat::telemetry::uuid::v4 for unguessability.
   ;; The parent stores this SAME id in AdminProc.server-id so that wrappers
   ;; can embed it in every Wire they construct.
   ;;
   ;; ProcessPeer construction (verbose-is-honest composition per Stone C2):
   ;;   rx = Receiver/from-pipe(Process/stdout proc)   ← reads subprocess stdout (WireResp)
   ;;   tx = Sender/from-pipe(Process/stdin proc)      ← writes to subprocess stdin (Wire)
   ;;   peer = ProcessPeer/new(rx, tx)
   ;;
   ;; Process/stdin must be extracted in an inner-let so the IOWriter drops before
   ;; drain-and-join. Here in spawn-proc we extract it only to build tx — the IOWriter
   ;; is consumed by Sender/from-pipe immediately and doesn't live past the let.
   ;; The peer! holds the typed Sender (not the raw IOWriter), so scope-deadlock
   ;; checker sees ProcessPeer (struct type) not a raw Sender at the outer scope.
   (:wat::core::defn :counter::spawn-proc
     []
     -> :counter::AdminProc
     (:wat::core::let
       [proc
          (:wat::kernel::spawn-process
            (:wat::core::forms
              ;; ── Subprocess type declarations (independent from parent's) ──
              ;; Same names + shapes → same EDN tags → round-trip works.
              ;; Wire now carries server-id as the first field on both variants.
              (:wat::core::enum :counter::Wire
                (Admin (server-id :wat::core::String) (req :counter::AdminReq))
                (User  (server-id :wat::core::String) (id :wat::core::String) (req :counter::UserReq)))

              (:wat::core::enum :counter::WireResp
                (Admin (resp :counter::AdminResp))
                (User  (resp :counter::UserResp)))

              (:wat::core::enum :counter::AdminReq
                (Provision   (initial :wat::core::i64))
                (Deprovision (id :wat::core::String))
                (Stop))

              (:wat::core::enum :counter::AdminResp
                (Provisioned  (id :wat::core::String))
                (Deprovisioned (id :wat::core::String))
                (Stopped)
                (AccessDenied))              ;; server refused — server-id mismatch

              (:wat::core::enum :counter::UserReq
                (Get)
                (Increment (n :wat::core::i64))
                (Reset))

              (:wat::core::enum :counter::UserResp
                (Value (v :wat::core::i64))
                (Ok    (v :wat::core::i64))
                (AccessDenied))              ;; server refused — server-id mismatch

              ;; Registry type: Vector of (id, state) 2-tuples
              (:wat::core::typealias :sub::RegEntry
                :(wat::core::String,wat::core::i64))

              ;; ── Subprocess helpers ──────────────────────────────────────

              ;; Find the state for a given id; returns -1 if not found (sentinel)
              (:wat::core::defn :sub::find-state
                [registry <- :wat::core::Vector<sub::RegEntry>
                 target   <- :wat::core::String]
                -> :wat::core::i64
                (:wat::core::let
                  [init   (:wat::core::Tuple -1 0)
                   result (:wat::core::foldl registry init
                             (:wat::core::fn
                               [acc   <- :(wat::core::i64,wat::core::i64)
                                entry <- :sub::RegEntry]
                                -> :(wat::core::i64,wat::core::i64)
                               (:wat::core::let
                                 [found  (:wat::core::first  acc)
                                  seen   (:wat::core::second acc)
                                  eid    (:wat::core::first  entry)
                                  estate (:wat::core::second entry)
                                  match? (:wat::core::= eid target)
                                  new-found (:wat::core::if match? -> :wat::core::i64 estate found)
                                  new-seen  (:wat::core::i64::+'2 seen 1)]
                                 (:wat::core::Tuple new-found new-seen))))]
                  (:wat::core::first result)))

              ;; Update state for a given id in the registry; returns new registry
              (:wat::core::defn :sub::update-state
                [registry  <- :wat::core::Vector<sub::RegEntry>
                 target    <- :wat::core::String
                 new-state <- :wat::core::i64]
                -> :wat::core::Vector<sub::RegEntry>
                (:wat::core::let
                  [init   (:wat::core::Tuple
                             (:wat::core::Vector :sub::RegEntry)
                             0)
                   result (:wat::core::foldl registry init
                             (:wat::core::fn
                               [acc   <- :(wat::core::Vector<sub::RegEntry>,wat::core::i64)
                                entry <- :sub::RegEntry]
                                -> :(wat::core::Vector<sub::RegEntry>,wat::core::i64)
                               (:wat::core::let
                                 [new-vec  (:wat::core::first  acc)
                                  pos      (:wat::core::second acc)
                                  eid      (:wat::core::first  entry)
                                  match?   (:wat::core::= eid target)
                                  updated  (:wat::core::if match?
                                              -> :sub::RegEntry
                                              (:wat::core::Tuple eid new-state)
                                              entry)
                                  next-vec (:wat::core::conj new-vec updated)
                                  next-pos (:wat::core::i64::+'2 pos 1)]
                                 (:wat::core::Tuple next-vec next-pos))))]
                  (:wat::core::first result)))

              ;; Remove entry by id from registry
              (:wat::core::defn :sub::remove-entry
                [registry <- :wat::core::Vector<sub::RegEntry>
                 target   <- :wat::core::String]
                -> :wat::core::Vector<sub::RegEntry>
                (:wat::core::filter registry
                  (:wat::core::fn
                    [entry <- :sub::RegEntry]
                     -> :wat::core::bool
                    (:wat::core::not
                      (:wat::core::= (:wat::core::first entry) target)))))

              ;; ── Admin handler ──────────────────────────────────────────
              ;; Called from dispatch when Wire::Admin received AND server-id matches.
              ;; Returns nil (tail-calls dispatch or exits on Stop).
              (:wat::core::defn :sub::handle-admin
                [registry  <- :wat::core::Vector<sub::RegEntry>
                 next-id   <- :wat::core::i64
                 admin-req <- :counter::AdminReq]
                -> :wat::core::nil
                (:wat::core::match admin-req -> :wat::core::nil
                  ((:counter::AdminReq::Provision initial)
                    (:wat::core::let
                      [id-str    (:wat::core::string::concat "client-"
                                   (:wat::core::i64::to-string next-id))
                       new-entry (:wat::core::Tuple id-str initial)
                       new-reg   (:wat::core::conj registry new-entry)
                       next-next (:wat::core::i64::+'2 next-id 1)]
                      (:wat::kernel::println
                        (:counter::WireResp::Admin (:counter::AdminResp::Provisioned id-str)))
                      (:sub::dispatch new-reg next-next)))
                  ((:counter::AdminReq::Deprovision dep-id)
                    (:wat::core::let
                      [new-reg (:sub::remove-entry registry dep-id)]
                      (:wat::kernel::println
                        (:counter::WireResp::Admin (:counter::AdminResp::Deprovisioned dep-id)))
                      (:sub::dispatch new-reg next-id)))
                  ((:counter::AdminReq::Stop)
                    ;; Send Stopped; return nil → subprocess exits
                    (:wat::kernel::println
                      (:counter::WireResp::Admin (:counter::AdminResp::Stopped))))))

              ;; ── User handler ───────────────────────────────────────────
              ;; Called from dispatch when Wire::User received AND server-id matches.
              (:wat::core::defn :sub::handle-user
                [registry <- :wat::core::Vector<sub::RegEntry>
                 next-id  <- :wat::core::i64
                 uid      <- :wat::core::String
                 user-req <- :counter::UserReq]
                -> :wat::core::nil
                (:wat::core::match user-req -> :wat::core::nil
                  ((:counter::UserReq::Get)
                    (:wat::core::let
                      [state (:sub::find-state registry uid)]
                      (:wat::kernel::println
                        (:counter::WireResp::User (:counter::UserResp::Value state)))
                      (:sub::dispatch registry next-id)))
                  ((:counter::UserReq::Increment n)
                    (:wat::core::let
                      [old-state (:sub::find-state registry uid)
                       new-state (:wat::core::i64::+'2 old-state n)
                       new-reg   (:sub::update-state registry uid new-state)]
                      (:wat::kernel::println
                        (:counter::WireResp::User (:counter::UserResp::Ok new-state)))
                      (:sub::dispatch new-reg next-id)))
                  ((:counter::UserReq::Reset)
                    (:wat::core::let
                      [new-reg (:sub::update-state registry uid 0)]
                      (:wat::kernel::println
                        (:counter::WireResp::User (:counter::UserResp::Ok 0)))
                      (:sub::dispatch new-reg next-id)))))

              ;; ── Main dispatch loop ──────────────────────────────────────────
              ;; Reads one Wire from stdin; validates server-id first.
              ;;
              ;; SERVER-ID VALIDATION IS LOAD-BEARING at the process tier.
              ;; Users share the single ProcessPeer; the subprocess receives all
              ;; Wires over stdio. The server CANNOT rely on transport-level identity.
              ;; The server-id embedded in the Wire IS the auth mechanism.
              ;;
              ;; Validation shape:
              ;;   outer match: one arm per Wire variant (Admin | User)
              ;;   each arm: extracts wire-sid; checks against "server-counter-proc-0"
              ;;     MATCH   → call handle-admin / handle-user
              ;;     MISMATCH → emit AccessDenied WireResp; recur dispatch
              (:wat::core::defn :sub::dispatch
                [registry <- :wat::core::Vector<sub::RegEntry>
                 next-id  <- :wat::core::i64]
                -> :wat::core::nil
                (:wat::core::match (:wat::kernel::readln -> :counter::Wire)
                  -> :wat::core::nil
                  ((:counter::Wire::Admin wire-sid admin-req)
                    (:wat::core::if (:wat::core::= wire-sid "server-counter-proc-0")
                      -> :wat::core::nil
                      (:sub::handle-admin registry next-id admin-req)
                      ;; Mismatch: emit AccessDenied for admin; continue dispatch
                      (:wat::core::do
                        (:wat::kernel::println
                          (:counter::WireResp::Admin (:counter::AdminResp::AccessDenied)))
                        (:sub::dispatch registry next-id))))
                  ((:counter::Wire::User wire-sid uid user-req)
                    (:wat::core::if (:wat::core::= wire-sid "server-counter-proc-0")
                      -> :wat::core::nil
                      (:sub::handle-user registry next-id uid user-req)
                      ;; Mismatch: emit AccessDenied for user; continue dispatch
                      (:wat::core::do
                        (:wat::kernel::println
                          (:counter::WireResp::User (:counter::UserResp::AccessDenied)))
                        (:sub::dispatch registry next-id))))))

              ;; Entry point — substrate calls :user::main when subprocess starts
              (:wat::core::define (:user::main -> :wat::core::nil)
                (:sub::dispatch
                  (:wat::core::Vector :sub::RegEntry)
                  0))))

        ;; Build ProcessPeer — verbose-is-honest composition per Stone C2.
        ;; rx = Receiver/from-pipe(stdout) → reads WireResp the subprocess prints
        ;; tx = Sender/from-pipe(stdin)    → writes Wire that subprocess reads
        ;; ProcessPeer/new(rx, tx) per slice 2 SCORE delta 6: rx first, tx second
        rx      (:wat::kernel::Receiver/from-pipe (:wat::kernel::Process/stdout proc))
        tx      (:wat::kernel::Sender/from-pipe   (:wat::kernel::Process/stdin  proc))
        peer!   (:wat::kernel::ProcessPeer/new rx tx)]
       ;; proc! stored in AdminProc so stop-proc can drain-and-join
       (:counter::AdminProc/new "server-counter-proc-0" peer! proc)))

   ;; :counter::provision-proc — sends Wire/Admin Provision; reads WireResp/Admin Provisioned;
   ;; returns ClientProc with Arc-clone of admin's peer.
   ;;
   ;; Wire::Admin now carries server-id (from AdminProc capability) as first field.
   ;; AdminResp::Provisioned carries only the minted id (no channels at process tier).
   ;; ClientProc.peer! is constructed from admin.peer! — accessors clone Arc-backed values.
   ;;
   ;; Two-level match: outer covers WireResp variants (Admin | User);
   ;; inner covers AdminResp variants (Provisioned | Deprovisioned | Stopped | AccessDenied).
   ;; The exhaustiveness checker requires exactly one arm per outer variant.
   (:wat::core::defn :counter::provision-proc
     [admin!  <- :counter::AdminProc
      initial <- :wat::core::i64]
     -> :counter::ClientProc
     (:wat::core::let
       [pr      (:counter::AdminProc/peer!     admin!)
        sid     (:counter::AdminProc/server-id admin!)
        _sent
          (:wat::kernel::Process/println pr
            (:counter::Wire::Admin sid (:counter::AdminReq::Provision initial)))
        wire-resp
          (:wat::kernel::Process/readln pr)]
       (:wat::core::match wire-resp -> :counter::ClientProc
         ((:counter::WireResp::Admin admin-resp)
           (:wat::core::match admin-resp -> :counter::ClientProc
             ((:counter::AdminResp::Provisioned id)
               (:counter::ClientProc/new sid id pr))
             ((:counter::AdminResp::Deprovisioned _id)
               (:wat::kernel::assertion-failed! "provision-proc: expected Provisioned, got Deprovisioned" :wat::core::None :wat::core::None))
             ((:counter::AdminResp::Stopped)
               (:wat::kernel::assertion-failed! "provision-proc: expected Provisioned, got Stopped" :wat::core::None :wat::core::None))
             ((:counter::AdminResp::AccessDenied)
               (:wat::kernel::assertion-failed! "provision-proc: server refused — server-id mismatch" :wat::core::None :wat::core::None))))
         ((:counter::WireResp::User _resp)
           (:wat::kernel::assertion-failed! "provision-proc: expected Admin WireResp, got User" :wat::core::None :wat::core::None)))))

   ;; :counter::deprovision-proc — sends Wire/Admin Deprovision; reads WireResp/Admin Deprovisioned.
   ;; Wire::Admin carries server-id from AdminProc capability.
   ;; Two-level match: outer WireResp → Admin|User; inner AdminResp → Deprovisioned|others.
   (:wat::core::defn :counter::deprovision-proc
     [admin!  <- :counter::AdminProc
      client! <- :counter::ClientProc]
     -> :wat::core::nil
     (:wat::core::let
       [pr    (:counter::AdminProc/peer!      admin!)
        sid   (:counter::AdminProc/server-id  admin!)
        cid   (:counter::ClientProc/client-id client!)
        _sent
          (:wat::kernel::Process/println pr
            (:counter::Wire::Admin sid (:counter::AdminReq::Deprovision cid)))
        wire-resp
          (:wat::kernel::Process/readln pr)]
       (:wat::core::match wire-resp -> :wat::core::nil
         ((:counter::WireResp::Admin admin-resp)
           (:wat::core::match admin-resp -> :wat::core::nil
             ((:counter::AdminResp::Deprovisioned _id) ())
             ((:counter::AdminResp::Provisioned _id)
               (:wat::kernel::assertion-failed! "deprovision-proc: expected Deprovisioned, got Provisioned" :wat::core::None :wat::core::None))
             ((:counter::AdminResp::Stopped)
               (:wat::kernel::assertion-failed! "deprovision-proc: expected Deprovisioned, got Stopped" :wat::core::None :wat::core::None))
             ((:counter::AdminResp::AccessDenied)
               (:wat::kernel::assertion-failed! "deprovision-proc: server refused — server-id mismatch" :wat::core::None :wat::core::None))))
         ((:counter::WireResp::User _resp)
           (:wat::kernel::assertion-failed! "deprovision-proc: expected Admin WireResp, got User" :wat::core::None :wat::core::None)))))

   ;; :counter::stop-proc — sends Wire/Admin Stop; reads WireResp/Admin Stopped;
   ;; drains subprocess via Process/drain-and-join; returns nil.
   ;;
   ;; Wire::Admin carries server-id from AdminProc capability.
   ;;
   ;; SERVICE-PROGRAMS lockstep absorbed inside this wrapper:
   ;;   inner-let: extracts peer (ProcessPeer) and proc! (Process);
   ;;              does send/recv handshake; returns proc! to outer
   ;;   outer-let: holds only proc! (Process type, not raw IOWriter/Sender);
   ;;              calls Process/drain-and-join
   ;;
   ;; The peer's internal Sender (tx field) remains live until peer drops.
   ;; The Process handle (proc!) holds the IOWriter internally too.
   ;; Process/drain-and-join drains stdout+stderr then joins — subprocess
   ;; already exited after sending Stopped, so drain-and-join returns immediately.
   (:wat::core::defn :counter::stop-proc
     [admin! <- :counter::AdminProc]
     -> :wat::core::nil
     (:wat::core::let
       [raw-proc
          (:wat::core::let
            [pr      (:counter::AdminProc/peer!      admin!)
             p       (:counter::AdminProc/proc!      admin!)
             sid     (:counter::AdminProc/server-id  admin!)
             _sent
               (:wat::kernel::Process/println pr
                 (:counter::Wire::Admin sid (:counter::AdminReq::Stop)))
             _resp
               (:wat::kernel::Process/readln pr)]
            ;; pr (ProcessPeer) drops at inner-let exit; p returned to outer
            p)
        _drained
          (:wat::core::Result/expect -> :wat::core::nil
            (:wat::kernel::Process/drain-and-join raw-proc)
            "stop-proc: process died")]
       ()))

   ;; ─── User ops ────────────────────────────────────────────────────────────────
   ;;
   ;; Each user wrapper sends Wire::User carrying the server-id (secret witness),
   ;; client-id (routing key), and the UserReq variant.
   ;; Reads back WireResp::User carrying the UserResp; extracts and returns the value.
   ;; The peer is read from the ClientProc capability (restricted accessor — :counter::*).
   ;;
   ;; Two-level match — outer WireResp → User|Admin; inner UserResp → Value|Ok|AccessDenied.

   (:wat::core::defn :counter::get-proc
     [client! <- :counter::ClientProc]
     -> :wat::core::i64
     (:wat::core::let
       [pr   (:counter::ClientProc/peer!      client!)
        cid  (:counter::ClientProc/client-id  client!)
        sid  (:counter::ClientProc/server-id  client!)
        _sent
          (:wat::kernel::Process/println pr
            (:counter::Wire::User sid cid (:counter::UserReq::Get)))
        wire-resp
          (:wat::kernel::Process/readln pr)]
       (:wat::core::match wire-resp -> :wat::core::i64
         ((:counter::WireResp::User user-resp)
           (:wat::core::match user-resp -> :wat::core::i64
             ((:counter::UserResp::Value v) v)
             ((:counter::UserResp::Ok    v) v)
             ((:counter::UserResp::AccessDenied)
               (:wat::kernel::assertion-failed! "get-proc: server refused — server-id mismatch" :wat::core::None :wat::core::None))))
         ((:counter::WireResp::Admin _admin-resp)
           (:wat::kernel::assertion-failed! "get-proc: expected User WireResp, got Admin" :wat::core::None :wat::core::None)))))

   (:wat::core::defn :counter::increment-proc
     [client! <- :counter::ClientProc
      n       <- :wat::core::i64]
     -> :wat::core::i64
     (:wat::core::let
       [pr   (:counter::ClientProc/peer!      client!)
        cid  (:counter::ClientProc/client-id  client!)
        sid  (:counter::ClientProc/server-id  client!)
        _sent
          (:wat::kernel::Process/println pr
            (:counter::Wire::User sid cid (:counter::UserReq::Increment n)))
        wire-resp
          (:wat::kernel::Process/readln pr)]
       (:wat::core::match wire-resp -> :wat::core::i64
         ((:counter::WireResp::User user-resp)
           (:wat::core::match user-resp -> :wat::core::i64
             ((:counter::UserResp::Ok    v) v)
             ((:counter::UserResp::Value v) v)
             ((:counter::UserResp::AccessDenied)
               (:wat::kernel::assertion-failed! "increment-proc: server refused — server-id mismatch" :wat::core::None :wat::core::None))))
         ((:counter::WireResp::Admin _admin-resp)
           (:wat::kernel::assertion-failed! "increment-proc: expected User WireResp, got Admin" :wat::core::None :wat::core::None)))))

   (:wat::core::defn :counter::reset-proc
     [client! <- :counter::ClientProc]
     -> :wat::core::i64
     (:wat::core::let
       [pr   (:counter::ClientProc/peer!      client!)
        cid  (:counter::ClientProc/client-id  client!)
        sid  (:counter::ClientProc/server-id  client!)
        _sent
          (:wat::kernel::Process/println pr
            (:counter::Wire::User sid cid (:counter::UserReq::Reset)))
        wire-resp
          (:wat::kernel::Process/readln pr)]
       (:wat::core::match wire-resp -> :wat::core::i64
         ((:counter::WireResp::User user-resp)
           (:wat::core::match user-resp -> :wat::core::i64
             ((:counter::UserResp::Ok    v) v)
             ((:counter::UserResp::Value v) v)
             ((:counter::UserResp::AccessDenied)
               (:wat::kernel::assertion-failed! "reset-proc: server refused — server-id mismatch" :wat::core::None :wat::core::None))))
         ((:counter::WireResp::Admin _admin-resp)
           (:wat::kernel::assertion-failed! "reset-proc: expected User WireResp, got Admin" :wat::core::None :wat::core::None)))))

   ;; ─── Forge demonstration: adversarial test ───────────────────────────────────
   ;;
   ;; At the process tier, LOAD-BEARING validation means a mismatch will silently
   ;; (from a buggy wrapper's perspective) get AccessDenied. This helper
   ;; demonstrates the rejection path by intentionally sending a Wire with a
   ;; WRONG server-id and asserting AccessDenied comes back.
   ;;
   ;; Unlike the thread tier where the Sender is enclosed in struct-restricted,
   ;; here Process/println is the write path — any :counter::* code can call it.
   ;; The forgery helper is WITHIN :counter::* (the privileged namespace) so it
   ;; CAN read AdminProc/peer! and send arbitrary bytes. The test demonstrates
   ;; that the server-side validation holds regardless of what the privileged
   ;; namespace does.
   (:wat::core::defn :counter::test-forge-proc-rejection
     [admin! <- :counter::AdminProc]
     -> :wat::core::nil
     (:wat::core::let
       [pr    (:counter::AdminProc/peer!     admin!)
        ;; Intentionally WRONG server-id — simulates a forged or mis-routed message
        _sent
          (:wat::kernel::Process/println pr
            (:counter::Wire::Admin "WRONG-SERVER-ID" (:counter::AdminReq::Provision 99)))
        wire-resp
          (:wat::kernel::Process/readln pr)]
       (:wat::core::match wire-resp -> :wat::core::nil
         ((:counter::WireResp::Admin admin-resp)
           (:wat::core::match admin-resp -> :wat::core::nil
             ((:counter::AdminResp::AccessDenied) ())   ;; expected — server correctly rejected
             ((:counter::AdminResp::Provisioned _id)
               (:wat::kernel::assertion-failed! "forge-proc-test: server should have rejected WRONG-SERVER-ID, got Provisioned" :wat::core::None :wat::core::None))
             ((:counter::AdminResp::Deprovisioned _id)
               (:wat::kernel::assertion-failed! "forge-proc-test: server should have rejected WRONG-SERVER-ID, got Deprovisioned" :wat::core::None :wat::core::None))
             ((:counter::AdminResp::Stopped)
               (:wat::kernel::assertion-failed! "forge-proc-test: server should have rejected WRONG-SERVER-ID, got Stopped" :wat::core::None :wat::core::None))))
         ((:counter::WireResp::User _resp)
           (:wat::kernel::assertion-failed! "forge-proc-test: expected Admin WireResp, got User" :wat::core::None :wat::core::None))))))

  ;; ─── Test body ───────────────────────────────────────────────────────────────
  ;;
  ;; Exercises ALL ops via capability wrappers ONLY.
  ;; This namespace is :counter-service::process-N3 — NOT :counter::*.
  ;; The test body CANNOT:
  ;;   - call :counter::AdminProc/new or :counter::ClientProc/new (restricted ctor)
  ;;   - call :counter::AdminProc/server-id, :counter::AdminProc/peer!, etc. (restricted accessors)
  ;;   - call :counter::ClientProc/server-id, :counter::ClientProc/client-id, etc.
  ;;
  ;; admin! and client-X! are struct types (not raw ProcessPeer or Process);
  ;; the scope-deadlock checker does not fire on struct-typed bindings.
  ;; SERVICE-PROGRAMS lockstep is absorbed entirely into :counter::stop-proc.
  ;;
  ;; Scenario:
  ;;   1. Spawn server subprocess → admin!
  ;;   2. Provision 3 users: initial 10, 100, 0 → client-a!, client-b!, client-c!
  ;;   3. Increment a by 5  → 15
  ;;   4. Increment b by 50 → 150
  ;;   5. Get c             → 0
  ;;   6. Deprovision b
  ;;   7. Get a             → 15  (still alive after b deprovisioned)
  ;;   8. Reset c           → 0   (still alive)
  ;;   9. Forge test: send wrong-server-id to subprocess; assert AccessDenied
  ;;  10. Stop admin!       → sends Stop, reads Stopped, drains subprocess
  (:wat::core::let
    [admin!    (:counter::spawn-proc)
     client-a! (:counter::provision-proc admin! 10)
     client-b! (:counter::provision-proc admin! 100)
     client-c! (:counter::provision-proc admin! 0)

     ;; Each user independent — ops affect only their own counter
     a1        (:counter::increment-proc client-a! 5)
     _         (:wat::test::assert-eq a1 15)

     b1        (:counter::increment-proc client-b! 50)
     _         (:wat::test::assert-eq b1 150)

     c1        (:counter::get-proc client-c!)
     _         (:wat::test::assert-eq c1 0)

     ;; Deprovision client-b mid-flight; a and c continue
     _dep      (:counter::deprovision-proc admin! client-b!)

     ;; client-a still works
     a2        (:counter::get-proc client-a!)
     _         (:wat::test::assert-eq a2 15)

     ;; client-c still works
     c2        (:counter::reset-proc client-c!)
     _         (:wat::test::assert-eq c2 0)

     ;; Forge test: adversarial helper sends Wire with WRONG server-id;
     ;; subprocess should respond AccessDenied; wrapper asserts the rejection.
     _forge    (:counter::test-forge-proc-rejection admin!)

     ;; Admin Stop — sends Stop, reads Stopped, drains subprocess; all inside wrapper
     _stop     (:counter::stop-proc admin!)]
    :wat::core::nil))
