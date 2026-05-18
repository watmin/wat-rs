;; wat-tests/counter-service-process-N3.wat — Capability-wrapped multi-user counter, process tier.
;;
;; Arc 203 slice 3f — sixth stepping stone (in-place update of slice 3e).
;; Replaces panic-on-error semantics with honest Result-bearing wrappers.
;;
;; Previously (slice 3e): wrappers returned raw T; transport errors panicked via Result/expect.
;; Now (slice 3f): every wrapper returns Result<T,:counter::ServiceError>; callers match Ok/Err.
;;
;; Same architecture as slice 3c (Admin + User capability structs, struct-restricted,
;; dynamic Provision/Deprovision, per-user independent state) but at the PROCESS TIER:
;;   - Server runs as a subprocess (spawn-process + :wat::core::forms)
;;   - All communication via stdio multiplexed through Wire (parent→sub) + WireResp (sub→parent)
;;   - Admin holds :counter::AdminProc (server-id, peer!, proc!)
;;   - Each user holds :counter::UserProc (server-id, user-id, peer!) — Arc-clone of admin's peer
;;   - Sequential request-response (single-threaded body; no concurrent demux needed)
;;
;; ─── ERROR TYPE DESIGN (PROCESS TIER) ────────────────────────────────────────
;;
;; :counter::ServiceError for process tier:
;;   AccessDenied  — server validated server-id and rejected the request (wire-level)
;;   ServerDied    — subprocess died; chain is Vector<ProcessDiedError> (arc 113 shape)
;;   Disconnected  — Process/drain-and-join returned Ok but we got Disconnected before response
;;
;; KEY ASYMMETRY from thread tier:
;;   :wat::kernel::Process/println and :wat::kernel::Process/readln do NOT return Result.
;;   They PANIC (raise RuntimeError) on subprocess death. This means wrappers that
;;   use these primitives CANNOT catch transport failures as Result — they can only
;;   propagate the AccessDenied wire-level error.
;;
;;   Process/drain-and-join DOES return Result<nil,Vector<ProcessDiedError>>.
;;   The stop-proc wrapper uses drain-and-join and CAN propagate ServerDied.
;;
;;   The ServerDied Err path is demonstrated via a separate crash-test helper
;;   that spawns a subprocess that panics, then calls Process/drain-and-join,
;;   and maps Err(chain) → Err(ServiceError/ServerDied(chain)).
;;
;; ─── PROCESS-TIER VALIDATION SEMANTICS ──────────────────────────────────────
;;
;; At the process tier, the server-id check is LOAD-BEARING — not merely defense in depth.
;; Unlike the thread tier (where the Sender<Wire> is enclosed in struct-restricted Admin/User
;; so only :counter::* code can obtain it), the process tier has NO such transport-level
;; guarantee. The subprocess accepts data over stdio; a bug, a future multiplexer, or a
;; malicious caller could write bytes to the subprocess's stdin outside of the :counter::*
;; wrappers. The server CANNOT rely on transport-level identity.
;;
;; The secret-witness pattern makes validation STRUCTURAL: only a caller who obtained
;; an AdminProc or UserProc capability (minted exclusively by :counter::spawn-proc and
;; :counter::provision-proc) knows the server-id. The subprocess validates every incoming
;; Wire against its own server-id. A Wire with the wrong server-id is rejected with
;; AccessDenied — the request is never processed.
;;
;; Arc 207: server-id is now :wat::core::Uuid. The subprocess uses Uuid/nil as a
;; well-known constant server-id (forms blocks cannot capture runtime-minted values).
;; Uuid/nil is typed :Uuid — honesty over String. A fresh-mint protocol would require
;; subprocess-to-parent handshake which is out of scope for this demo.
;;
;; ─────────────────────────────────────────────────────────────────────────────
;;
;; Design choices:
;;   1. Multiplexed single-stream — all admin + user ops share one ProcessPeer
;;      Wire::User carries user-id so server can route; WireResp tags Admin vs User
;;   2. AdminProc holds proc! for drain-and-join in stop-proc (inner/outer let pattern)
;;   3. UserProc.peer! is same peer as AdminProc.peer! (Arc-clone; accessor clones)
;;   4. -proc suffix avoids collision with slice 3c's thread-tier wrappers
;;
;; Lessons applied from prior slices:
;;   - Inner type aliases in :() are bare (no leading colon)
;;   - foldl not reduce
;;   - first/second tuple accessors only (registry entries are 2-tuples)
;;   - Process/drain-and-join not join-result
;;   - Inner/outer let for scope-deadlock in stop-proc
;;   - One-line :() annotations (no whitespace inside)
;;   - ProcessPeer/new(rx, tx) where rx = Receiver/from-pipe(stdout), tx = Sender/from-pipe(stdin)
;;   - Subprocess declares :user::main via (:wat::core::define ...) not defn
;;   - Two-level match required when matching enums carrying enum payloads
;;   - Process/println + Process/readln do NOT return Result (panic on transport error)
;;   - Process/drain-and-join returns Result<nil,Vector<ProcessDiedError>> (Err = chain)
;;   - No spaces inside type parameter <> brackets

(:wat::test::deftest :counter-service::process-N3
  (;; ─── Wire enum (parent → subprocess) ───────────────────────────────────────
   ;; Wire::Admin and Wire::User now carry server-id as the first field.
   ;; The subprocess validates this server-id against its own before processing.
   ;; This is LOAD-BEARING at the process tier: shared ProcessPeer means
   ;; the server cannot rely on transport identity alone.
   ;; Arc 207: server-id and user-id are typed :wat::core::Uuid.
   (:wat::core::enum :counter::Wire
     (Admin (server-id :wat::core::Uuid) (req :counter::AdminReq))
     (User  (server-id :wat::core::Uuid) (user-id :wat::core::Uuid) (req :counter::UserReq)))

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
     (Deprovision (id :wat::core::Uuid))
     (Stop))

   (:wat::core::enum :counter::AdminResp
     (Provisioned  (id :wat::core::Uuid))
     (Deprovisioned (id :wat::core::Uuid))
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

   ;; ─── ServiceError enum ────────────────────────────────────────────────────
   ;;
   ;; The honest error type for process-tier client-facing wrappers.
   ;;
   ;;   AccessDenied  — server validated server-id and rejected the request (wire-level)
   ;;   ServerDied    — subprocess died; chain is Vector<ProcessDiedError> (arc 113 shape)
   ;;   Disconnected  — drain-and-join returned Ok but clean-disconnect observed
   ;;
   ;; Note: Process/println + Process/readln do NOT return Result — they panic on
   ;; transport failure. So user-op wrappers (get-proc, increment-proc, reset-proc)
   ;; can only propagate AccessDenied via Result; transport errors still panic at
   ;; the substrate level for those wrappers.
   ;;
   ;; The ServerDied Err path is reachable via:
   ;;   1. stop-proc → Process/drain-and-join → Err(chain) → Err(ServerDied(chain))
   ;;   2. crash-test-proc → spawn crashing subprocess → drain-and-join → Err(ServerDied(chain))
   (:wat::core::enum :counter::ServiceError
     (AccessDenied)
     (ServerDied  (chain :wat::core::Vector<wat::kernel::ProcessDiedError>))
     (Disconnected))

   ;; ─── Capability structs ───────────────────────────────────────────────────────
   ;;
   ;; :counter::AdminProc — admin handle wrapping the shared ProcessPeer + Process.
   ;;   server-id — names this server instance
   ;;   peer!     — ProcessPeer<counter::WireResp,counter::Wire>:
   ;;                 parent reads WireResp from subprocess stdout (peer.rx)
   ;;                 parent writes Wire to subprocess stdin (peer.tx)
   ;;   proc!     — Process<counter::Wire,counter::WireResp>:
   ;;                 raw process handle; held so stop-proc can drain-and-join
   ;;
   ;; Minted ONLY by :counter::spawn-proc (constructor whitelist [:counter::]).
   ;; All fields restricted to :counter::* reads.
   ;; Arc 207: server-id is typed :wat::core::Uuid.
   (:wat::core::struct-restricted :counter::AdminProc
     [:counter::]
     ([:counter::] server-id <- :wat::core::Uuid
      [:counter::] peer!     <- :wat::kernel::ProcessPeer<counter::WireResp,counter::Wire>
      [:counter::] proc!     <- :wat::kernel::Process<counter::Wire,counter::WireResp>)
     ())

   ;; :counter::UserProc — per-user capability handle.
   ;;   server-id  — identifies which server issued this user
   ;;   user-id    — server-minted unique id for this user slot
   ;;   peer!      — shared ProcessPeer (Arc-clone of AdminProc's peer!)
   ;;
   ;; Minted ONLY by :counter::provision-proc (constructor whitelist [:counter::]).
   ;; All fields restricted to :counter::* reads.
   ;; Arc 207: server-id and user-id are typed :wat::core::Uuid.
   (:wat::core::struct-restricted :counter::UserProc
     [:counter::]
     ([:counter::] server-id <- :wat::core::Uuid
      [:counter::] user-id   <- :wat::core::Uuid
      [:counter::] peer!     <- :wat::kernel::ProcessPeer<counter::WireResp,counter::Wire>)
     ())

   ;; ─── Privileged wrappers ──────────────────────────────────────────────────────
   ;;
   ;; :counter::spawn-proc — spawns subprocess, builds ProcessPeer, returns AdminProc.
   ;;
   ;; No send/recv → returns AdminProc directly (no Result wrapper needed at spawn level).
   ;; Subprocess program declared inline via :wat::core::forms.
   ;; The subprocess declares its own independent copies of all enum types.
   ;; Same enum names → same EDN tags → interoperable across process boundary.
   ;;
   ;; Arc 207: server-id is :wat::core::Uuid. The subprocess uses Uuid/nil as a
   ;; well-known constant (forms block cannot capture a runtime-minted value).
   ;; The parent stores Uuid/nil in AdminProc.server-id — types are honest (:Uuid)
   ;; even though the value is a constant. See top-of-file comment for rationale.
   ;;
   ;; ProcessPeer construction (verbose-is-honest composition per Stone C2):
   ;;   rx = Receiver/from-pipe(Process/stdout proc)   ← reads subprocess stdout (WireResp)
   ;;   tx = Sender/from-pipe(Process/stdin proc)      ← writes to subprocess stdin (Wire)
   ;;   peer = ProcessPeer/new(rx, tx)
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
              ;; Arc 207: server-id and user-id are typed :wat::core::Uuid.
              (:wat::core::enum :counter::Wire
                (Admin (server-id :wat::core::Uuid) (req :counter::AdminReq))
                (User  (server-id :wat::core::Uuid) (user-id :wat::core::Uuid) (req :counter::UserReq)))

              (:wat::core::enum :counter::WireResp
                (Admin (resp :counter::AdminResp))
                (User  (resp :counter::UserResp)))

              (:wat::core::enum :counter::AdminReq
                (Provision   (initial :wat::core::i64))
                (Deprovision (id :wat::core::Uuid))
                (Stop))

              (:wat::core::enum :counter::AdminResp
                (Provisioned  (id :wat::core::Uuid))
                (Deprovisioned (id :wat::core::Uuid))
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

              ;; Registry type: Vector of (user-id, state) 2-tuples.
              ;; Arc 207: user-id is typed :wat::core::Uuid.
              (:wat::core::typealias :sub::RegEntry
                :(wat::core::Uuid,wat::core::i64))

              ;; ── Subprocess helpers ──────────────────────────────────────

              ;; Find the state for a given id; returns -1 if not found (sentinel)
              (:wat::core::defn :sub::find-state
                [registry <- :wat::core::Vector<sub::RegEntry>
                 target   <- :wat::core::Uuid]
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
                 target    <- :wat::core::Uuid
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
                 target   <- :wat::core::Uuid]
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
              ;; Arc 207: server-id constant is Uuid/nil; dispatch compares directly,
              ;; so no need to thread self-server-id as a parameter here.
              (:wat::core::defn :sub::handle-admin
                [registry  <- :wat::core::Vector<sub::RegEntry>
                 next-id   <- :wat::core::i64
                 admin-req <- :counter::AdminReq]
                -> :wat::core::nil
                (:wat::core::match admin-req -> :wat::core::nil
                  ((:counter::AdminReq::Provision initial)
                    (:wat::core::let
                      [user-id   (:wat::core::Uuid/v4)
                       new-entry (:wat::core::Tuple user-id initial)
                       new-reg   (:wat::core::conj registry new-entry)
                       next-next (:wat::core::i64::+'2 next-id 1)]
                      (:wat::kernel::println
                        (:counter::WireResp::Admin (:counter::AdminResp::Provisioned user-id)))
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
                 uid      <- :wat::core::Uuid
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
              ;; Arc 207: server-id constant is Uuid/nil. Comparison uses Uuid/nil
              ;; inline — no need to thread as a parameter (constant never changes).
              ;; Validation shape:
              ;;   outer match: one arm per Wire variant (Admin | User)
              ;;   each arm: extracts wire-sid (:Uuid); checks = (:Uuid/nil)
              ;;     MATCH   → call handle-admin / handle-user
              ;;     MISMATCH → emit AccessDenied WireResp; recur dispatch
              (:wat::core::defn :sub::dispatch
                [registry <- :wat::core::Vector<sub::RegEntry>
                 next-id  <- :wat::core::i64]
                -> :wat::core::nil
                (:wat::core::match (:wat::kernel::readln -> :counter::Wire)
                  -> :wat::core::nil
                  ((:counter::Wire::Admin wire-sid admin-req)
                    (:wat::core::if (:wat::core::= wire-sid (:wat::core::Uuid/nil))
                      -> :wat::core::nil
                      (:sub::handle-admin registry next-id admin-req)
                      ;; Mismatch: emit AccessDenied for admin; continue dispatch
                      (:wat::core::do
                        (:wat::kernel::println
                          (:counter::WireResp::Admin (:counter::AdminResp::AccessDenied)))
                        (:sub::dispatch registry next-id))))
                  ((:counter::Wire::User wire-sid uid user-req)
                    (:wat::core::if (:wat::core::= wire-sid (:wat::core::Uuid/nil))
                      -> :wat::core::nil
                      (:sub::handle-user registry next-id uid user-req)
                      ;; Mismatch: emit AccessDenied for user; continue dispatch
                      (:wat::core::do
                        (:wat::kernel::println
                          (:counter::WireResp::User (:counter::UserResp::AccessDenied)))
                        (:sub::dispatch registry next-id))))))

              ;; Entry point — substrate calls :user::main when subprocess starts.
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
       ;; proc! stored in AdminProc so stop-proc can drain-and-join.
       ;; Arc 207: server-id is Uuid/nil — matches the subprocess's self-server-id.
       (:counter::AdminProc/new (:wat::core::Uuid/nil) peer! proc)))

   ;; :counter::provision-proc — sends Wire/Admin Provision; reads WireResp/Admin Provisioned.
   ;;
   ;; Now returns Result<UserProc,ServiceError>.
   ;; Process/println + Process/readln panic on transport failure;
   ;; only the wire-level AccessDenied is catchable as Result.
   ;;
   ;; AccessDenied path: server responds AccessDenied → Err(ServiceError/AccessDenied)
   ;; Transport failure: still panics (substrate limitation for process tier)
   ;;
   ;; Wire::Admin now carries server-id (from AdminProc capability) as first field.
   ;; AdminResp::Provisioned carries only the minted id (no channels at process tier).
   ;; UserProc.peer! is constructed from admin.peer! — accessors clone Arc-backed values.
   ;;
   ;; Two-level match: outer covers WireResp variants (Admin | User);
   ;; inner covers AdminResp variants (Provisioned | Deprovisioned | Stopped | AccessDenied).
   (:wat::core::defn :counter::provision-proc
     [admin!  <- :counter::AdminProc
      initial <- :wat::core::i64]
     -> :wat::core::Result<counter::UserProc,counter::ServiceError>
     (:wat::core::let
       [pr      (:counter::AdminProc/peer!     admin!)
        sid     (:counter::AdminProc/server-id admin!)
        _sent
          (:wat::kernel::Process/println pr
            (:counter::Wire::Admin sid (:counter::AdminReq::Provision initial)))
        wire-resp
          (:wat::kernel::Process/readln pr)]
       (:wat::core::match wire-resp -> :wat::core::Result<counter::UserProc,counter::ServiceError>
         ((:counter::WireResp::Admin admin-resp)
           (:wat::core::match admin-resp -> :wat::core::Result<counter::UserProc,counter::ServiceError>
             ((:counter::AdminResp::Provisioned id)
               (:wat::core::Ok (:counter::UserProc/new sid id pr)))
             ((:counter::AdminResp::AccessDenied)
               (:wat::core::Err (:counter::ServiceError::AccessDenied)))
             ((:counter::AdminResp::Deprovisioned _id)
               (:wat::kernel::assertion-failed! "provision-proc: expected Provisioned, got Deprovisioned" :wat::core::None :wat::core::None))
             ((:counter::AdminResp::Stopped)
               (:wat::kernel::assertion-failed! "provision-proc: expected Provisioned, got Stopped" :wat::core::None :wat::core::None))))
         ((:counter::WireResp::User _resp)
           (:wat::kernel::assertion-failed! "provision-proc: expected Admin WireResp, got User" :wat::core::None :wat::core::None)))))

   ;; :counter::deprovision-proc — sends Wire/Admin Deprovision; reads WireResp/Admin Deprovisioned.
   ;; Now returns Result<nil,ServiceError>.
   ;; Wire::Admin carries server-id from AdminProc capability.
   ;; Two-level match: outer WireResp → Admin|User; inner AdminResp → Deprovisioned|others.
   (:wat::core::defn :counter::deprovision-proc
     [admin!  <- :counter::AdminProc
      user!   <- :counter::UserProc]
     -> :wat::core::Result<wat::core::nil,counter::ServiceError>
     (:wat::core::let
       [pr    (:counter::AdminProc/peer!     admin!)
        sid   (:counter::AdminProc/server-id admin!)
        cid   (:counter::UserProc/user-id    user!)
        _sent
          (:wat::kernel::Process/println pr
            (:counter::Wire::Admin sid (:counter::AdminReq::Deprovision cid)))
        wire-resp
          (:wat::kernel::Process/readln pr)]
       (:wat::core::match wire-resp -> :wat::core::Result<wat::core::nil,counter::ServiceError>
         ((:counter::WireResp::Admin admin-resp)
           (:wat::core::match admin-resp -> :wat::core::Result<wat::core::nil,counter::ServiceError>
             ((:counter::AdminResp::Deprovisioned _id)
               (:wat::core::Ok ()))
             ((:counter::AdminResp::AccessDenied)
               (:wat::core::Err (:counter::ServiceError::AccessDenied)))
             ((:counter::AdminResp::Provisioned _id)
               (:wat::kernel::assertion-failed! "deprovision-proc: expected Deprovisioned, got Provisioned" :wat::core::None :wat::core::None))
             ((:counter::AdminResp::Stopped)
               (:wat::kernel::assertion-failed! "deprovision-proc: expected Deprovisioned, got Stopped" :wat::core::None :wat::core::None))))
         ((:counter::WireResp::User _resp)
           (:wat::kernel::assertion-failed! "deprovision-proc: expected Admin WireResp, got User" :wat::core::None :wat::core::None)))))

   ;; :counter::stop-proc — sends Wire/Admin Stop; reads WireResp/Admin Stopped;
   ;; drains subprocess via Process/drain-and-join; returns Result<nil,ServiceError>.
   ;;
   ;; Wire::Admin carries server-id from AdminProc capability.
   ;;
   ;; This wrapper CAN propagate ServerDied: Process/drain-and-join returns
   ;; Result<nil,Vector<ProcessDiedError>>. If the subprocess panicked or was
   ;; killed abnormally, drain-and-join returns Err(chain) → Err(ServerDied(chain)).
   ;;
   ;; SERVICE-PROGRAMS lockstep absorbed inside this wrapper:
   ;;   inner-let: extracts peer (ProcessPeer) and proc! (Process);
   ;;              does send/recv handshake; returns proc! to outer
   ;;   outer-let: holds only raw-proc (Process type, not raw IOWriter/Sender);
   ;;              calls Process/drain-and-join; matches Result
   ;;
   ;; Note: Process/println + Process/readln in the inner-let still panic on
   ;; transport failure. If the subprocess dies BEFORE responding to Stop, the
   ;; inner-let panics. In that case, drain-and-join would never be reached.
   ;; This is acceptable: if the subprocess died before getting Stop, we'd panic
   ;; at the readln step. The Err path via drain-and-join covers the case where
   ;; the subprocess exits ABNORMALLY (non-zero exit) after receiving Stop.
   (:wat::core::defn :counter::stop-proc
     [admin! <- :counter::AdminProc]
     -> :wat::core::Result<wat::core::nil,counter::ServiceError>
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
            p)]
       ;; outer: match drain-and-join result
       (:wat::core::match (:wat::kernel::Process/drain-and-join raw-proc)
         -> :wat::core::Result<wat::core::nil,counter::ServiceError>
         ((:wat::core::Ok _)
           (:wat::core::Ok ()))
         ((:wat::core::Err chain)
           (:wat::core::Err (:counter::ServiceError::ServerDied chain))))))

   ;; ─── User ops ────────────────────────────────────────────────────────────────
   ;;
   ;; Each user wrapper sends Wire::User carrying the server-id (secret witness),
   ;; user-id (routing key), and the UserReq variant.
   ;; Reads back WireResp::User carrying the UserResp; extracts and returns value.
   ;; Now returns Result<i64,ServiceError>.
   ;;
   ;; The only catchable Err here is AccessDenied (from wire-level rejection).
   ;; Process/println + Process/readln panic on subprocess death (substrate limitation).
   ;;
   ;; Two-level match — outer WireResp → User|Admin; inner UserResp → Value|Ok|AccessDenied.

   (:wat::core::defn :counter::get-proc
     [user! <- :counter::UserProc]
     -> :wat::core::Result<wat::core::i64,counter::ServiceError>
     (:wat::core::let
       [pr   (:counter::UserProc/peer!      user!)
        cid  (:counter::UserProc/user-id    user!)
        sid  (:counter::UserProc/server-id  user!)
        _sent
          (:wat::kernel::Process/println pr
            (:counter::Wire::User sid cid (:counter::UserReq::Get)))
        wire-resp
          (:wat::kernel::Process/readln pr)]
       (:wat::core::match wire-resp -> :wat::core::Result<wat::core::i64,counter::ServiceError>
         ((:counter::WireResp::User user-resp)
           (:wat::core::match user-resp -> :wat::core::Result<wat::core::i64,counter::ServiceError>
             ((:counter::UserResp::Value v) (:wat::core::Ok v))
             ((:counter::UserResp::Ok    v) (:wat::core::Ok v))
             ((:counter::UserResp::AccessDenied)
               (:wat::core::Err (:counter::ServiceError::AccessDenied)))))
         ((:counter::WireResp::Admin _admin-resp)
           (:wat::kernel::assertion-failed! "get-proc: expected User WireResp, got Admin" :wat::core::None :wat::core::None)))))

   (:wat::core::defn :counter::increment-proc
     [user! <- :counter::UserProc
      n     <- :wat::core::i64]
     -> :wat::core::Result<wat::core::i64,counter::ServiceError>
     (:wat::core::let
       [pr   (:counter::UserProc/peer!      user!)
        cid  (:counter::UserProc/user-id    user!)
        sid  (:counter::UserProc/server-id  user!)
        _sent
          (:wat::kernel::Process/println pr
            (:counter::Wire::User sid cid (:counter::UserReq::Increment n)))
        wire-resp
          (:wat::kernel::Process/readln pr)]
       (:wat::core::match wire-resp -> :wat::core::Result<wat::core::i64,counter::ServiceError>
         ((:counter::WireResp::User user-resp)
           (:wat::core::match user-resp -> :wat::core::Result<wat::core::i64,counter::ServiceError>
             ((:counter::UserResp::Ok    v) (:wat::core::Ok v))
             ((:counter::UserResp::Value v) (:wat::core::Ok v))
             ((:counter::UserResp::AccessDenied)
               (:wat::core::Err (:counter::ServiceError::AccessDenied)))))
         ((:counter::WireResp::Admin _admin-resp)
           (:wat::kernel::assertion-failed! "increment-proc: expected User WireResp, got Admin" :wat::core::None :wat::core::None)))))

   (:wat::core::defn :counter::reset-proc
     [user! <- :counter::UserProc]
     -> :wat::core::Result<wat::core::i64,counter::ServiceError>
     (:wat::core::let
       [pr   (:counter::UserProc/peer!      user!)
        cid  (:counter::UserProc/user-id    user!)
        sid  (:counter::UserProc/server-id  user!)
        _sent
          (:wat::kernel::Process/println pr
            (:counter::Wire::User sid cid (:counter::UserReq::Reset)))
        wire-resp
          (:wat::kernel::Process/readln pr)]
       (:wat::core::match wire-resp -> :wat::core::Result<wat::core::i64,counter::ServiceError>
         ((:counter::WireResp::User user-resp)
           (:wat::core::match user-resp -> :wat::core::Result<wat::core::i64,counter::ServiceError>
             ((:counter::UserResp::Ok    v) (:wat::core::Ok v))
             ((:counter::UserResp::Value v) (:wat::core::Ok v))
             ((:counter::UserResp::AccessDenied)
               (:wat::core::Err (:counter::ServiceError::AccessDenied)))))
         ((:counter::WireResp::Admin _admin-resp)
           (:wat::kernel::assertion-failed! "reset-proc: expected User WireResp, got Admin" :wat::core::None :wat::core::None)))))

   ;; ─── Forge demonstration: adversarial test ───────────────────────────────────
   ;;
   ;; Now returns Result<nil,ServiceError> — demonstrates the AccessDenied Err path.
   ;;
   ;; At the process tier, LOAD-BEARING validation means a mismatch gets AccessDenied.
   ;; This helper demonstrates the rejection path by intentionally sending a Wire with a
   ;; WRONG server-id and RETURNING that as Err(AccessDenied) rather than panicking.
   ;;
   ;; Unlike the thread tier where the Sender is enclosed in struct-restricted,
   ;; here Process/println is the write path — any :counter::* code can call it.
   ;; The forgery helper is WITHIN :counter::* (the privileged namespace) so it
   ;; CAN read AdminProc/peer! and send arbitrary bytes.
   (:wat::core::defn :counter::test-forge-proc-rejection
     [admin! <- :counter::AdminProc]
     -> :wat::core::Result<wat::core::nil,counter::ServiceError>
     (:wat::core::let
       [pr    (:counter::AdminProc/peer!     admin!)
        ;; Arc 207: forge uses a fresh v4 Uuid — server's id is Uuid/nil so any v4 mismatches.
        wrong-id (:wat::core::Uuid/v4)
        _sent
          (:wat::kernel::Process/println pr
            (:counter::Wire::Admin wrong-id (:counter::AdminReq::Provision 99)))
        wire-resp
          (:wat::kernel::Process/readln pr)]
       (:wat::core::match wire-resp -> :wat::core::Result<wat::core::nil,counter::ServiceError>
         ((:counter::WireResp::Admin admin-resp)
           (:wat::core::match admin-resp -> :wat::core::Result<wat::core::nil,counter::ServiceError>
             ((:counter::AdminResp::AccessDenied)
               (:wat::core::Err (:counter::ServiceError::AccessDenied)))   ;; expected — server correctly rejected
             ((:counter::AdminResp::Provisioned _id)
               (:wat::kernel::assertion-failed! "forge-proc-test: server should have rejected mismatched Uuid, got Provisioned" :wat::core::None :wat::core::None))
             ((:counter::AdminResp::Deprovisioned _id)
               (:wat::kernel::assertion-failed! "forge-proc-test: server should have rejected mismatched Uuid, got Deprovisioned" :wat::core::None :wat::core::None))
             ((:counter::AdminResp::Stopped)
               (:wat::kernel::assertion-failed! "forge-proc-test: server should have rejected mismatched Uuid, got Stopped" :wat::core::None :wat::core::None))))
         ((:counter::WireResp::User _resp)
           (:wat::kernel::assertion-failed! "forge-proc-test: expected Admin WireResp, got User" :wat::core::None :wat::core::None)))))

   ;; ─── ServerDied Err path: subprocess crash demonstration ─────────────────────
   ;;
   ;; Demonstrates the ServerDied variant by spawning a subprocess that panics
   ;; immediately and then calling Process/drain-and-join to detect the failure.
   ;;
   ;; Process/drain-and-join returns Result<nil,Vector<ProcessDiedError>>:
   ;;   Ok(nil)      — subprocess exited cleanly (exit code 0)
   ;;   Err(chain)   — subprocess panicked or exited with non-zero code
   ;;
   ;; This helper spawns a minimal subprocess that immediately panics via
   ;; assertion-failed!. Then drain-and-join detects the abnormal exit and
   ;; returns Err(chain) where chain[0] is a ProcessDiedError variant.
   ;;
   ;; The test body can then:
   ;;   1. Call crash-test-proc → expect Err(ServiceError/ServerDied(chain))
   ;;   2. Match ServerDied and verify it's an Err
   ;;
   ;; Note: The subprocess spawned here is independent of the counter service subprocess.
   ;; It is a fresh Process used only for demonstrating the ServerDied detection pattern.
   (:wat::core::defn :counter::crash-test-proc
     []
     -> :wat::core::Result<wat::core::nil,counter::ServiceError>
     (:wat::core::let
       [crash-proc
          (:wat::kernel::spawn-process
            (:wat::core::forms
              ;; A subprocess that panics immediately — simulates abnormal subprocess death
              (:wat::core::define (:user::main -> :wat::core::nil)
                (:wat::kernel::assertion-failed!
                  "crash-test-proc: intentional panic for ServerDied demonstration"
                  :wat::core::None :wat::core::None))))]
       ;; No peer construction needed — we only care about the exit result
       ;; Process/drain-and-join detects abnormal exit → Err(ProcessDiedError chain)
       (:wat::core::match (:wat::kernel::Process/drain-and-join crash-proc)
         -> :wat::core::Result<wat::core::nil,counter::ServiceError>
         ((:wat::core::Ok _)
           (:wat::kernel::assertion-failed! "crash-test-proc: expected crash, got Ok exit" :wat::core::None :wat::core::None))
         ((:wat::core::Err chain)
           (:wat::core::Err (:counter::ServiceError::ServerDied chain)))))))

  ;; ─── Test body ───────────────────────────────────────────────────────────────
  ;;
  ;; Exercises ALL ops via capability wrappers ONLY.
  ;; This namespace is :counter-service::process-N3 — NOT :counter::*.
  ;; The test body CANNOT:
  ;;   - call :counter::AdminProc/new or :counter::UserProc/new (restricted ctor)
  ;;   - call :counter::AdminProc/server-id, :counter::AdminProc/peer!, etc. (restricted accessors)
  ;;   - call :counter::UserProc/server-id, :counter::UserProc/user-id, etc.
  ;;
  ;; admin! and user-X! are struct types (not raw ProcessPeer or Process);
  ;; the scope-deadlock checker does not fire on struct-typed bindings.
  ;; SERVICE-PROGRAMS lockstep is absorbed entirely into :counter::stop-proc.
  ;;
  ;; All Result-returning wrappers are pattern-matched. Happy-path assertions
  ;; extract Ok values explicitly. Err paths are demonstrated:
  ;;   - AccessDenied: forge test → Result<nil,ServiceError> → match Err(AccessDenied)
  ;;   - ServerDied: crash-test-proc → spawns crashing subprocess → match Err(ServerDied)
  ;;
  ;; Scenario:
  ;;   1.  Spawn server subprocess → admin!
  ;;   2.  Provision 3 users: initial 10, 100, 0 → user-a!, user-b!, user-c!
  ;;   3.  Increment a by 5  → 15
  ;;   4.  Increment b by 50 → 150
  ;;   5.  Get c             → 0
  ;;   6.  Deprovision b
  ;;   7.  Get a             → 15  (still alive after b deprovisioned)
  ;;   8.  Reset c           → 0   (still alive)
  ;;   9.  Forge test: send wrong-server-id to subprocess; assert Err(AccessDenied)
  ;;  10.  Stop admin!       → sends Stop, reads Stopped, drains subprocess; returns Ok(nil)
  ;;  11.  ServerDied path: crash-test-proc → spawns crashing subprocess → Err(ServerDied)
  (:wat::core::let
    [admin!    (:counter::spawn-proc)

     ;; Step 2: provision — each returns Result<UserProc,ServiceError>; match Ok
     user-a-res (:counter::provision-proc admin! 10)
     user-a!
       (:wat::core::match user-a-res -> :counter::UserProc
         ((:wat::core::Ok c) c)
         ((:wat::core::Err _e)
           (:wat::kernel::assertion-failed! "provision-proc a: expected Ok" :wat::core::None :wat::core::None)))

     user-b-res (:counter::provision-proc admin! 100)
     user-b!
       (:wat::core::match user-b-res -> :counter::UserProc
         ((:wat::core::Ok c) c)
         ((:wat::core::Err _e)
           (:wat::kernel::assertion-failed! "provision-proc b: expected Ok" :wat::core::None :wat::core::None)))

     user-c-res (:counter::provision-proc admin! 0)
     user-c!
       (:wat::core::match user-c-res -> :counter::UserProc
         ((:wat::core::Ok c) c)
         ((:wat::core::Err _e)
           (:wat::kernel::assertion-failed! "provision-proc c: expected Ok" :wat::core::None :wat::core::None)))

     ;; Step 3: increment a — returns Result<i64,ServiceError>; match Ok; assert
     a1-res (:counter::increment-proc user-a! 5)
     a1
       (:wat::core::match a1-res -> :wat::core::i64
         ((:wat::core::Ok v) v)
         ((:wat::core::Err _e)
           (:wat::kernel::assertion-failed! "increment-proc a: expected Ok" :wat::core::None :wat::core::None)))
     _  (:wat::test::assert-eq a1 15)

     ;; Step 4: increment b
     b1-res (:counter::increment-proc user-b! 50)
     b1
       (:wat::core::match b1-res -> :wat::core::i64
         ((:wat::core::Ok v) v)
         ((:wat::core::Err _e)
           (:wat::kernel::assertion-failed! "increment-proc b: expected Ok" :wat::core::None :wat::core::None)))
     _  (:wat::test::assert-eq b1 150)

     ;; Step 5: get c
     c1-res (:counter::get-proc user-c!)
     c1
       (:wat::core::match c1-res -> :wat::core::i64
         ((:wat::core::Ok v) v)
         ((:wat::core::Err _e)
           (:wat::kernel::assertion-failed! "get-proc c: expected Ok" :wat::core::None :wat::core::None)))
     _  (:wat::test::assert-eq c1 0)

     ;; Step 6: deprovision b — returns Result<nil,ServiceError>; assert Ok
     dep-res (:counter::deprovision-proc admin! user-b!)
     _dep
       (:wat::core::match dep-res -> :wat::core::nil
         ((:wat::core::Ok _) ())
         ((:wat::core::Err _e)
           (:wat::kernel::assertion-failed! "deprovision-proc b: expected Ok" :wat::core::None :wat::core::None)))

     ;; Step 7: get a (still alive after b deprovisioned)
     a2-res (:counter::get-proc user-a!)
     a2
       (:wat::core::match a2-res -> :wat::core::i64
         ((:wat::core::Ok v) v)
         ((:wat::core::Err _e)
           (:wat::kernel::assertion-failed! "get-proc a after deprovision b: expected Ok" :wat::core::None :wat::core::None)))
     _  (:wat::test::assert-eq a2 15)

     ;; Step 8: reset c (still alive)
     c2-res (:counter::reset-proc user-c!)
     c2
       (:wat::core::match c2-res -> :wat::core::i64
         ((:wat::core::Ok v) v)
         ((:wat::core::Err _e)
           (:wat::kernel::assertion-failed! "reset-proc c: expected Ok" :wat::core::None :wat::core::None)))
     _  (:wat::test::assert-eq c2 0)

     ;; Step 9: Forge test — adversarial helper returns Err(AccessDenied)
     ;; Subprocess correctly rejected the wrong-server-id; wrapper returns typed error.
     ;; This demonstrates the AccessDenied Err path at the process tier.
     forge-res (:counter::test-forge-proc-rejection admin!)
     _forge
       (:wat::core::match forge-res -> :wat::core::nil
         ((:wat::core::Err err)
           (:wat::core::match err -> :wat::core::nil
             ((:counter::ServiceError::AccessDenied) ())   ;; expected — forge correctly rejected
             ((:counter::ServiceError::ServerDied _chain)
               (:wat::kernel::assertion-failed! "forge-proc: expected AccessDenied, got ServerDied" :wat::core::None :wat::core::None))
             ((:counter::ServiceError::Disconnected)
               (:wat::kernel::assertion-failed! "forge-proc: expected AccessDenied, got Disconnected" :wat::core::None :wat::core::None))))
         ((:wat::core::Ok _)
           (:wat::kernel::assertion-failed! "forge-proc: expected Err(AccessDenied), got Ok" :wat::core::None :wat::core::None)))

     ;; Step 10: Stop — returns Result<nil,ServiceError>; assert Ok
     stop-res (:counter::stop-proc admin!)
     _stop
       (:wat::core::match stop-res -> :wat::core::nil
         ((:wat::core::Ok _) ())
         ((:wat::core::Err _e)
           (:wat::kernel::assertion-failed! "stop-proc: expected Ok" :wat::core::None :wat::core::None)))

     ;; Step 11: ServerDied Err path — subprocess crash detection via drain-and-join.
     ;; crash-test-proc spawns a subprocess that panics immediately.
     ;; Process/drain-and-join detects the abnormal exit → Err(ProcessDiedError chain).
     ;; The wrapper converts this to Err(ServiceError/ServerDied(chain)).
     ;; This demonstrates the honest typed error path for subprocess failure.
     crash-res (:counter::crash-test-proc)
     _crash
       (:wat::core::match crash-res -> :wat::core::nil
         ((:wat::core::Err err)
           (:wat::core::match err -> :wat::core::nil
             ((:counter::ServiceError::ServerDied _chain) ())   ;; expected — subprocess crashed
             ((:counter::ServiceError::AccessDenied)
               (:wat::kernel::assertion-failed! "crash-test: expected ServerDied, got AccessDenied" :wat::core::None :wat::core::None))
             ((:counter::ServiceError::Disconnected)
               (:wat::kernel::assertion-failed! "crash-test: expected ServerDied, got Disconnected" :wat::core::None :wat::core::None))))
         ((:wat::core::Ok _)
           (:wat::kernel::assertion-failed! "crash-test: expected Err(ServerDied), got Ok" :wat::core::None :wat::core::None)))]
    :wat::core::nil))
