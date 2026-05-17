;; wat-tests/counter-service-thread-N3.wat — Server dispatch loop, thread tier, dynamic N users.
;;
;; Arc 203 slice 3b — second stepping stone.
;; Builds on slice 3a's foundation (Wire enum, select-based dispatch, admin/user routing).
;; Adds dynamic registry: admin Provisions new users (server mints client-id, creates channel
;; pair, registers in ordered Vector), Deprovisions existing ones (server drops entry).
;;
;; Key architectural lesson from stdin.wat (arc 170 slice 1f-β-i):
;;   HashMap/values iteration order is NON-DETERMINISTIC. select-by-index requires a
;;   STABLE order so the index maps correctly back to the registry entry.
;;   Therefore the registry is a Vector<RegistryEntry>, NOT HashMap<String, ...>.
;;   The Routing comment below documents the conceptual HashMap per the BRIEF;
;;   the driver state is RegistryVec (ordered, index-stable).
;;
;; Registry entry shape (3-tuple, 4th field nested to stay within first/second/third):
;;   RegistryEntry = :(String, Receiver<Wire>, :(Sender<UserResp>, i64))
;;     first  = client-id  : String
;;     second = server-rx  : Receiver<Wire>   (server reads user requests)
;;     third  = tx-state   : :(Sender<UserResp>, i64)
;;       first(third)  = server-tx : Sender<UserResp>  (server sends user responses)
;;       second(third) = state     : i64               (per-user counter state)
;;
;; Select-set construction each iteration:
;;   [admin-wire-rx, *(map second registry-vec)]
;;   idx==0        → admin message → handle AdminReq variant; update registry; recur
;;   idx>0         → user message  → look up registry[idx-1]; handle UserReq; recur
;;   Disconnected on user idx → drop registry entry (remove-at); recur
;;   Disconnected on admin   → exit (clean shutdown without explicit Stop)
;;
;; AdminReq protocol grows from 3a's (Stop) to add:
;;   (Provision   (initial i64)) — mint new client-id, create channel pair, register
;;   (Deprovision (id String))   — drop registry entry by id; sends Deprovisioned
;;
;; AdminResp protocol grows from 3a's (Stopped(final)) to add:
;;   (Provisioned (id String) (tx Sender<Wire>) (rx Receiver<UserResp>))
;;   (Deprovisioned (id String))
;;   NOTE: BRIEF had (rx :wat::kernel::Receiver<counter::Wire>) for Provisioned —
;;   that was a BRIEF error. User receives UserResp, not Wire. Corrected here.
;;   NOTE: Stopped(final) simplified to Stopped (no final-state aggregation per design choice 4).
;;
;; Wire enum: UNCHANGED from slice 3a.
;; UserReq + UserResp: UNCHANGED from slice 3a.
;;
;; Service-programs lockstep (SERVICE-PROGRAMS.md):
;;   All Senders must drop BEFORE Thread/drain-and-join.
;;   Inner let holds all Senders + does all communication; outer scope joins.
;;
;; Per-user state is INDEPENDENT — each user's counter is tracked separately
;; in their registry entry's state field (design choice 1).

(:wat::test::deftest :counter-service::thread-N3
  (;; ─── Admin protocol ──────────────────────────────────────────────────
   ;;
   ;; AdminReq grows from 3a's (Stop) to add Provision/Deprovision.
   ;; Provision: admin requests new user with given initial state.
   ;; Deprovision: admin requests removal of a specific client by id.
   ;; Stop: admin requests server shutdown.
   (:wat::core::enum :counter::AdminReq
     (Provision (initial :wat::core::i64))
     (Deprovision (id :wat::core::String))
     (Stop))

   ;; AdminResp grows: Provisioned carries user-side channel ends + client-id.
   ;; Server mints the id; creates channel pair; hands user-side ends to admin.
   ;; Admin is the broker — it hands tx+rx to the user client.
   ;; NOTE: rx is Receiver<UserResp> (not Receiver<Wire>) — user receives responses.
   (:wat::core::enum :counter::AdminResp
     (Provisioned (id :wat::core::String) (tx :wat::kernel::Sender<counter::Wire>) (rx :wat::kernel::Receiver<counter::UserResp>))
     (Deprovisioned (id :wat::core::String))
     (Stopped))

   ;; ─── User protocol (unchanged from 3a) ───────────────────────────────
   (:wat::core::enum :counter::UserReq
     (Get)
     (Increment (n :wat::core::i64))
     (Reset))

   (:wat::core::enum :counter::UserResp
     (Value (v :wat::core::i64))
     (Ok    (v :wat::core::i64)))

   ;; ─── Wire enum (unchanged from 3a) ───────────────────────────────────
   ;;
   ;; Unified request type — select is ∀T, all receivers must share T.
   ;; Wire wraps both admin and user requests.
   (:wat::core::enum :counter::Wire
     (Admin (req :counter::AdminReq))
     (User  (req :counter::UserReq)))

   ;; ─── Registry types ───────────────────────────────────────────────────
   ;;
   ;; Conceptual routing shape (per BRIEF), NOT used as the driver state.
   ;; HashMap/values order is non-deterministic; select-by-index needs stable order.
   ;; Driver uses RegistryVec (ordered Vector) instead — mirrors stdin.wat pattern.
   ;;
   ;; RegistryEntry = :(String, Receiver<Wire>, :(Sender<UserResp>, i64))
   ;;   first  entry = client-id  (String)
   ;;   second entry = server-rx  (Receiver<Wire>    — server reads user requests via select)
   ;;   third  entry = tx-state   (:(Sender<UserResp>, i64))
   ;;     first  tx-state = server-tx  (Sender<UserResp> — server sends user responses)
   ;;     second tx-state = state      (i64             — per-user counter value)
   ;;
   ;; No whitespace inside :() per WAT-CHEATSHEET.md § 2.
   (:wat::core::typealias :counter::TxStatePair
     :(wat::kernel::Sender<counter::UserResp>,wat::core::i64))

   (:wat::core::typealias :counter::RegistryEntry
     :(wat::core::String,wat::kernel::Receiver<counter::Wire>,counter::TxStatePair))

   (:wat::core::typealias :counter::RegistryVec
     :wat::core::Vector<counter::RegistryEntry>)

   ;; ─── Helper: build select-set rxs from registry-vec ─────────────────
   ;;
   ;; Extracts the server-rx (second field) from each RegistryEntry.
   ;; Result is a Vector<Receiver<Wire>> parallel to registry-vec.
   ;; Fed to select alongside [admin-wire-rx] (prepended by caller).
   ;; Index i in result → registry-vec[i]; admin-wire-rx is at idx == length(registry-vec).
   (:wat::core::defn :counter::registry-rxs
     [registry-vec <- :counter::RegistryVec]
     -> :wat::core::Vector<wat::kernel::Receiver<counter::Wire>>
     (:wat::core::map registry-vec
       (:wat::core::fn
         [entry <- :counter::RegistryEntry]
          -> :wat::kernel::Receiver<counter::Wire>
         (:wat::core::second entry))))

   ;; ─── Helper: provision new entry ─────────────────────────────────────
   ;;
   ;; Appends a new RegistryEntry to registry-vec. Returns new vec.
   ;; Channel pair created by caller (provision-user), passed in.
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

   ;; ─── Helper: deprovision entry by id ─────────────────────────────────
   ;;
   ;; Filters out the entry whose client-id matches id. Returns new vec.
   ;; The dropped entry's server-tx is released → user-resp-rx sees Disconnect.
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

   ;; ─── Helper: update state for entry at index ─────────────────────────
   ;;
   ;; Returns a new RegistryVec with entry[idx] having new-state.
   ;; Used after handling user messages that mutate state.
   ;; Implemented as map-with-index: entries before/after idx unchanged;
   ;; entry at idx reconstructed with new-state.
   ;; Note: Vector has no direct set-at; use filter+conj alternative below.
   ;; Using rebuild-vec approach: split, update, rejoin via remove-at + insert.
   ;;
   ;; Actually the cleanest is to rebuild via map with a closure that tracks
   ;; the current index. But map doesn't provide index.
   ;; Alternative: use Vector/get to get old entry, reconstruct it, use
   ;; two removes + appends — expensive. Simpler: carry a helper that
   ;; rebuilds the tx-state pair inline.
   ;;
   ;; Design: split off old entry via remove-at (yields new vec without idx),
   ;; then conj the updated entry back. conj appends at end — order changes.
   ;; For select-by-index, ORDER must be preserved. So we need index-preserving update.
   ;;
   ;; Real solution: map over indices. wat has no map-indexed, but we can
   ;; fold over a range. Let's use a different approach: since we don't have
   ;; map-indexed, we'll carry state as the fold over entries-with-position.
   ;;
   ;; Simplest correct approach: rebuild via fold — fold over registry-vec,
   ;; accumulating the new vec; at position current-idx, emit updated entry;
   ;; else emit entry unchanged.
   ;;
   ;; wat has :wat::core::foldl (left fold). Let's use it.
   ;; foldl(vec, init, fn(acc, elem) -> acc) -> acc
   (:wat::core::defn :counter::registry-update-state
     [registry-vec <- :counter::RegistryVec
      target-idx   <- :wat::core::i64
      new-state    <- :wat::core::i64]
     -> :counter::RegistryVec
     (:wat::core::let
       [;; Accumulator shape: :(RegistryVec, i64) = (new-vec, current-pos)
        ;; Start with empty vec + pos=0
        init (:wat::core::Tuple
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
                ;; Update entry if this is the target index
                updated-entry
                  (:wat::core::if (:wat::core::= cur-pos target-idx)
                    -> :counter::RegistryEntry
                    ;; Rebuild entry with new-state
                    (:wat::core::let
                      [eid    (:wat::core::first  entry)
                       erx    (:wat::core::second entry)
                       etx    (:wat::core::first  (:wat::core::third entry))]
                      (:wat::core::Tuple eid erx (:wat::core::Tuple etx new-state)))
                    ;; Unchanged
                    entry)
                next-vec (:wat::core::conj new-vec updated-entry)
                next-pos (:wat::core::i64::+'2 cur-pos 1)]
               (:wat::core::Tuple next-vec next-pos))))]
       (:wat::core::first result)))

   ;; ─── Server dispatch loop ─────────────────────────────────────────────
   ;;
   ;; :counter::dispatch3 — recursive server loop for N-user registry.
   ;;
   ;; Takes:
   ;;   admin-wire-rx  — Receiver<Wire> for admin logical channel (fixed; always in select)
   ;;   admin-resp-tx  — Sender<AdminResp> to reply to admin
   ;;   registry-vec   — RegistryVec (ordered; index-stable for select-by-index)
   ;;   next-id        — monotonic i64 counter for client-id generation
   ;;
   ;; Each iteration:
   ;;   1. Build select set: [*(registry-rxs registry-vec), admin-wire-rx]
   ;;      Admin is at idx == length(registry-vec) (appended last).
   ;;   2. select blocks until one receiver fires.
   ;;   3. Match idx:
   ;;      idx < len  → user message at registry-vec[idx]
   ;;      idx == len → admin message
   ;;   4. Dispatch accordingly; recur with updated registry + next-id.
   ;;
   ;; Admin stop: send Stopped; return nil (thread exits).
   ;; User disconnect: remove-at idx; recur (auto-cleanup).
   ;; Admin disconnect: return nil (graceful exit without explicit Stop).
   ;;
   ;; Per one-let-per-function rule: dispatch helpers are factored out below.
   ;;
   ;; NOTE: scope-deadlock checker — registry-vec contains Sender<UserResp> values
   ;; (in the tx-state pairs). These are inside the RegistryVec, not top-level
   ;; Senders in the function scope. The checker fires on top-level Senders in scope
   ;; at drain-and-join call sites. Here, dispatch3 is called from spawn3's thread
   ;; body — the admin-resp-tx Sender is the only top-level Sender in scope.
   ;; The registry Senders are enclosed within RegistryVec, not visible to checker.

   (:wat::core::defn :counter::dispatch3
     [admin-wire-rx <- :wat::kernel::Receiver<counter::Wire>
      admin-resp-tx <- :wat::kernel::Sender<counter::AdminResp>
      registry-vec  <- :counter::RegistryVec
      next-id       <- :wat::core::i64]
     -> :wat::core::nil
     (:wat::core::let
       [;; Build select set: user rxs first, then admin-rx last
        ;; Admin idx = length(registry-vec)
        user-rxs     (:counter::registry-rxs registry-vec)
        admin-vec    (:wat::core::Vector :wat::kernel::Receiver<counter::Wire> admin-wire-rx)
        select-set   (:wat::core::concat user-rxs admin-vec)
        registry-len (:wat::core::length registry-vec)
        ;; Block until one receiver fires
        chosen       (:wat::kernel::select select-set)
        idx          (:wat::core::first chosen)
        result       (:wat::core::second chosen)
        ;; Is this the admin channel?
        is-admin     (:wat::core::= idx registry-len)]
       (:wat::core::match result -> :wat::core::nil
         ;; Got a message
         ((:wat::core::Ok (:wat::core::Some wire))
           (:wat::core::if is-admin -> :wat::core::nil
             ;; Admin message — handle AdminReq
             (:counter::handle-admin3
               admin-wire-rx admin-resp-tx registry-vec next-id wire)
             ;; User message — handle UserReq at registry[idx]
             (:counter::handle-user3
               admin-wire-rx admin-resp-tx registry-vec next-id idx wire)))
         ;; Clean disconnect (sender dropped)
         ((:wat::core::Ok :wat::core::None)
           (:wat::core::if is-admin -> :wat::core::nil
             ;; Admin disconnected: graceful exit
             ()
             ;; User disconnected: auto-cleanup; drop entry; recur
             (:counter::dispatch3
               admin-wire-rx admin-resp-tx
               (:wat::std::list::remove-at registry-vec idx)
               next-id)))
         ;; Thread died
         ((:wat::core::Err _died)
           (:wat::core::if is-admin -> :wat::core::nil
             ;; Admin channel panicked: exit
             ()
             ;; User channel panicked: drop entry; recur
             (:counter::dispatch3
               admin-wire-rx admin-resp-tx
               (:wat::std::list::remove-at registry-vec idx)
               next-id))))))

   ;; ─── Helper: handle admin message ────────────────────────────────────
   ;;
   ;; Called when select fires on admin-rx.
   ;; Dispatches on AdminReq variant.
   ;;
   ;; Provision:
   ;;   - mint client-id = "client-" ++ to-string(next-id)
   ;;   - make-bounded-channel for user-wire (server-rx, user-tx)
   ;;   - make-bounded-channel for user-resp (server-tx, user-rx)
   ;;   - register in registry-vec
   ;;   - send Provisioned(id, user-tx, user-rx) to admin
   ;;   - recur with updated registry + incremented next-id
   ;;
   ;; Deprovision:
   ;;   - filter registry by id (releases server-tx → user-resp-rx sees Disconnect)
   ;;   - send Deprovisioned(id) to admin
   ;;   - recur with updated registry + same next-id
   ;;
   ;; Stop:
   ;;   - send Stopped to admin
   ;;   - return nil (thread exits; registry drops → all user Senders released)
   (:wat::core::defn :counter::handle-admin3
     [admin-wire-rx <- :wat::kernel::Receiver<counter::Wire>
      admin-resp-tx <- :wat::kernel::Sender<counter::AdminResp>
      registry-vec  <- :counter::RegistryVec
      next-id       <- :wat::core::i64
      wire          <- :counter::Wire]
     -> :wat::core::nil
     (:wat::core::match wire -> :wat::core::nil
       ;; Only Admin variants arrive on admin-rx (protocol discipline)
       ((:counter::Wire::Admin req)
         (:wat::core::match req -> :wat::core::nil
           ;; Provision: mint id, create channel pair, register, respond
           ((:counter::AdminReq::Provision initial)
             (:wat::core::let
               [;; Mint client-id string
                id-str    (:wat::core::string::concat "client-"
                            (:wat::core::i64::to-string next-id))
                ;; User-wire channel: user → server
                uwp       (:wat::kernel::make-bounded-channel :counter::Wire 1)
                user-tx   (:wat::core::first  uwp)
                server-rx (:wat::core::second uwp)
                ;; User-resp channel: server → user
                urp       (:wat::kernel::make-bounded-channel :counter::UserResp 1)
                server-tx (:wat::core::first  urp)
                user-rx   (:wat::core::second urp)
                ;; Register in registry-vec
                new-registry
                  (:counter::registry-provision
                    registry-vec id-str server-rx server-tx initial)
                ;; Send Provisioned to admin (hands user-side ends to caller)
                _sent
                  (:wat::core::Result/expect -> :wat::core::nil
                    (:wat::kernel::send admin-resp-tx
                      (:counter::AdminResp::Provisioned id-str user-tx user-rx))
                    "handle-admin3: admin-resp-tx disconnected on Provision")]
               ;; Recur with updated registry + incremented next-id
               (:counter::dispatch3
                 admin-wire-rx admin-resp-tx
                 new-registry
                 (:wat::core::i64::+'2 next-id 1))))
           ;; Deprovision: filter registry, release server-tx, respond
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
           ;; Stop: send Stopped; return nil (server exits)
           ((:counter::AdminReq::Stop)
             (:wat::core::Result/expect -> :wat::core::nil
               (:wat::kernel::send admin-resp-tx
                 (:counter::AdminResp::Stopped))
               "handle-admin3: admin-resp-tx disconnected on Stop"))))
       ;; Wire::User on admin-rx is a protocol violation; ignore + recur
       ((:counter::Wire::User _req)
         (:counter::dispatch3
           admin-wire-rx admin-resp-tx
           registry-vec next-id))))

   ;; ─── Helper: handle user message ─────────────────────────────────────
   ;;
   ;; Called when select fires on a user-rx at registry-vec[idx].
   ;; Extracts the entry, dispatches on UserReq, sends UserResp,
   ;; updates state if needed, recurs.
   (:wat::core::defn :counter::handle-user3
     [admin-wire-rx <- :wat::kernel::Receiver<counter::Wire>
      admin-resp-tx <- :wat::kernel::Sender<counter::AdminResp>
      registry-vec  <- :counter::RegistryVec
      next-id       <- :wat::core::i64
      idx           <- :wat::core::i64
      wire          <- :counter::Wire]
     -> :wat::core::nil
     (:wat::core::let
       [;; Extract entry at idx
        entry-opt (:wat::core::get registry-vec idx)]
       (:wat::core::match entry-opt -> :wat::core::nil
         ;; Entry exists — dispatch on user request
         ((:wat::core::Some entry)
           (:wat::core::let
             [tx-state  (:wat::core::third entry)
              server-tx (:wat::core::first tx-state)
              state     (:wat::core::second tx-state)]
             (:wat::core::match wire -> :wat::core::nil
               ;; Only User variants arrive on user-rx (protocol discipline)
               ((:counter::Wire::User req)
                 (:wat::core::match req -> :wat::core::nil
                   ;; Get: reply Value(state); state unchanged; recur
                   ((:counter::UserReq::Get)
                     (:wat::core::do
                       (:wat::core::Result/expect -> :wat::core::nil
                         (:wat::kernel::send server-tx
                           (:counter::UserResp::Value state))
                         "handle-user3: server-tx disconnected on Get")
                       (:counter::dispatch3
                         admin-wire-rx admin-resp-tx
                         registry-vec next-id)))
                   ;; Increment: compute new-n; reply Ok(new-n); update state; recur
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
                   ;; Reset: reply Ok(0); update state to 0; recur
                   ((:counter::UserReq::Reset)
                     (:wat::core::do
                       (:wat::core::Result/expect -> :wat::core::nil
                         (:wat::kernel::send server-tx
                           (:counter::UserResp::Ok 0))
                         "handle-user3: server-tx disconnected on Reset")
                       (:counter::dispatch3
                         admin-wire-rx admin-resp-tx
                         (:counter::registry-update-state registry-vec idx 0)
                         next-id)))))
               ;; Wire::Admin on user-rx is a protocol violation; ignore + recur
               ((:counter::Wire::Admin _req)
                 (:counter::dispatch3
                   admin-wire-rx admin-resp-tx
                   registry-vec next-id)))))
         ;; idx out of range — degenerate; cannot happen; exit
         (:wat::core::None
           ()))))

   ;; ─── Server constructor ───────────────────────────────────────────────
   ;;
   ;; :counter::spawn3 — creates empty registry, spawns server thread.
   ;;
   ;; spawn-thread(I=Wire, O=AdminResp):
   ;;   Thread/input(thread)  = Sender<Wire>       = admin-tx for admin client
   ;;   Thread/output(thread) = Receiver<AdminResp> = admin-resp-rx for admin client
   ;;
   ;; Server starts with empty RegistryVec + next-id=0.
   ;; Admin uses Provision messages to grow the registry.
   ;;
   ;; Returns Thread<Wire,AdminResp>.
   ;; Admin channels extracted via Thread/input + Thread/output.
   (:wat::core::defn :counter::spawn3
     []
     -> :wat::kernel::Thread<counter::Wire,counter::AdminResp>
     (:wat::kernel::spawn-thread
       (:wat::core::fn
         [admin-wire-rx <- :wat::kernel::Receiver<counter::Wire>
          admin-resp-tx <- :wat::kernel::Sender<counter::AdminResp>]
          -> :wat::core::nil
         (:counter::dispatch3
           admin-wire-rx admin-resp-tx
           (:wat::core::Vector :counter::RegistryEntry)
           0))))

   ;; ─── Admin client wrappers ────────────────────────────────────────────

   ;; admin-provision: send Provision(initial), recv Provisioned(id, tx, rx).
   ;; Returns :(String, Sender<Wire>, Receiver<UserResp>).
   (:wat::core::defn :counter::admin-provision3
     [admin-tx      <- :wat::kernel::Sender<counter::Wire>
      admin-resp-rx <- :wat::kernel::Receiver<counter::AdminResp>
      initial       <- :wat::core::i64]
     -> :(wat::core::String,wat::kernel::Sender<counter::Wire>,wat::kernel::Receiver<counter::UserResp>)
     (:wat::core::let
       [_sent
         (:wat::core::Result/expect -> :wat::core::nil
           (:wat::kernel::send admin-tx
             (:counter::Wire::Admin (:counter::AdminReq::Provision initial)))
           "admin-provision3: admin-tx disconnected")
        resp
         (:wat::core::Option/expect -> :counter::AdminResp
           (:wat::core::Result/expect -> :wat::core::Option<counter::AdminResp>
             (:wat::kernel::recv admin-resp-rx)
             "admin-provision3: recv peer died")
           "admin-provision3: clean disconnect")]
       (:wat::core::match resp -> :(wat::core::String,wat::kernel::Sender<counter::Wire>,wat::kernel::Receiver<counter::UserResp>)
         ((:counter::AdminResp::Provisioned id tx rx)
           (:wat::core::Tuple id tx rx))
         ((:counter::AdminResp::Deprovisioned _id)
           (:wat::kernel::assertion-failed! "admin-provision3: expected Provisioned, got Deprovisioned" :wat::core::None :wat::core::None))
         ((:counter::AdminResp::Stopped)
           (:wat::kernel::assertion-failed! "admin-provision3: expected Provisioned, got Stopped" :wat::core::None :wat::core::None)))))

   ;; admin-deprovision3: send Deprovision(id), recv Deprovisioned(id). Returns id.
   (:wat::core::defn :counter::admin-deprovision3
     [admin-tx      <- :wat::kernel::Sender<counter::Wire>
      admin-resp-rx <- :wat::kernel::Receiver<counter::AdminResp>
      id            <- :wat::core::String]
     -> :wat::core::String
     (:wat::core::let
       [_sent
         (:wat::core::Result/expect -> :wat::core::nil
           (:wat::kernel::send admin-tx
             (:counter::Wire::Admin (:counter::AdminReq::Deprovision id)))
           "admin-deprovision3: admin-tx disconnected")
        resp
         (:wat::core::Option/expect -> :counter::AdminResp
           (:wat::core::Result/expect -> :wat::core::Option<counter::AdminResp>
             (:wat::kernel::recv admin-resp-rx)
             "admin-deprovision3: recv peer died")
           "admin-deprovision3: clean disconnect")]
       (:wat::core::match resp -> :wat::core::String
         ((:counter::AdminResp::Deprovisioned dep-id) dep-id)
         ((:counter::AdminResp::Provisioned _id _tx _rx)
           (:wat::kernel::assertion-failed! "admin-deprovision3: expected Deprovisioned, got Provisioned" :wat::core::None :wat::core::None))
         ((:counter::AdminResp::Stopped)
           (:wat::kernel::assertion-failed! "admin-deprovision3: expected Deprovisioned, got Stopped" :wat::core::None :wat::core::None)))))

   ;; admin-stop3: send Stop, recv Stopped. Returns nil.
   (:wat::core::defn :counter::admin-stop3
     [admin-tx      <- :wat::kernel::Sender<counter::Wire>
      admin-resp-rx <- :wat::kernel::Receiver<counter::AdminResp>]
     -> :wat::core::nil
     (:wat::core::let
       [_sent
         (:wat::core::Result/expect -> :wat::core::nil
           (:wat::kernel::send admin-tx
             (:counter::Wire::Admin (:counter::AdminReq::Stop)))
           "admin-stop3: admin-tx disconnected")
        resp
         (:wat::core::Option/expect -> :counter::AdminResp
           (:wat::core::Result/expect -> :wat::core::Option<counter::AdminResp>
             (:wat::kernel::recv admin-resp-rx)
             "admin-stop3: recv peer died")
           "admin-stop3: clean disconnect")]
       (:wat::core::match resp -> :wat::core::nil
         ((:counter::AdminResp::Stopped) ())
         ((:counter::AdminResp::Provisioned _id _tx _rx)
           (:wat::kernel::assertion-failed! "admin-stop3: expected Stopped, got Provisioned" :wat::core::None :wat::core::None))
         ((:counter::AdminResp::Deprovisioned _id)
           (:wat::kernel::assertion-failed! "admin-stop3: expected Stopped, got Deprovisioned" :wat::core::None :wat::core::None)))))

   ;; ─── User client wrappers ─────────────────────────────────────────────
   ;;
   ;; Identical in shape to slice 3a (user-tx: Sender<Wire>, user-rx: Receiver<UserResp>).

   (:wat::core::defn :counter::user-increment3
     [user-tx <- :wat::kernel::Sender<counter::Wire>
      user-rx <- :wat::kernel::Receiver<counter::UserResp>
      n       <- :wat::core::i64]
     -> :wat::core::i64
     (:wat::core::let
       [_sent
         (:wat::core::Result/expect -> :wat::core::nil
           (:wat::kernel::send user-tx (:counter::Wire::User (:counter::UserReq::Increment n)))
           "user-increment3: user-tx disconnected")
        resp
         (:wat::core::Option/expect -> :counter::UserResp
           (:wat::core::Result/expect -> :wat::core::Option<counter::UserResp>
             (:wat::kernel::recv user-rx)
             "user-increment3: recv peer died")
           "user-increment3: clean disconnect")]
       (:wat::core::match resp -> :wat::core::i64
         ((:counter::UserResp::Ok    v) v)
         ((:counter::UserResp::Value v) v))))

   (:wat::core::defn :counter::user-get3
     [user-tx <- :wat::kernel::Sender<counter::Wire>
      user-rx <- :wat::kernel::Receiver<counter::UserResp>]
     -> :wat::core::i64
     (:wat::core::let
       [_sent
         (:wat::core::Result/expect -> :wat::core::nil
           (:wat::kernel::send user-tx (:counter::Wire::User (:counter::UserReq::Get)))
           "user-get3: user-tx disconnected")
        resp
         (:wat::core::Option/expect -> :counter::UserResp
           (:wat::core::Result/expect -> :wat::core::Option<counter::UserResp>
             (:wat::kernel::recv user-rx)
             "user-get3: recv peer died")
           "user-get3: clean disconnect")]
       (:wat::core::match resp -> :wat::core::i64
         ((:counter::UserResp::Ok    v) v)
         ((:counter::UserResp::Value v) v))))

   (:wat::core::defn :counter::user-reset3
     [user-tx <- :wat::kernel::Sender<counter::Wire>
      user-rx <- :wat::kernel::Receiver<counter::UserResp>]
     -> :wat::core::i64
     (:wat::core::let
       [_sent
         (:wat::core::Result/expect -> :wat::core::nil
           (:wat::kernel::send user-tx (:counter::Wire::User (:counter::UserReq::Reset)))
           "user-reset3: user-tx disconnected")
        resp
         (:wat::core::Option/expect -> :counter::UserResp
           (:wat::core::Result/expect -> :wat::core::Option<counter::UserResp>
             (:wat::kernel::recv user-rx)
             "user-reset3: recv peer died")
           "user-reset3: clean disconnect")]
       (:wat::core::match resp -> :wat::core::i64
         ((:counter::UserResp::Ok    v) v)
         ((:counter::UserResp::Value v) v)))))

  ;; ─── Test body ─────────────────────────────────────────────────────────
  ;;
  ;; SERVICE-PROGRAMS lockstep: all Senders (admin-tx, user-tx1/2/3) must
  ;; drop BEFORE Thread/drain-and-join. Inner let holds all Senders + does
  ;; all communication; returns thread to outer scope. Outer scope joins.
  ;;
  ;; Scenario:
  ;;   1. Spawn server with empty registry
  ;;   2. Provision 3 users: initial states 10, 100, 0
  ;;   3. User 1: Increment 5 → Ok 15
  ;;   4. User 2: Increment 50 → Ok 150
  ;;   5. User 3: Get → Value 0
  ;;   6. Admin: Deprovision user 2 → Deprovisioned id2
  ;;   7. User 1: Get → Value 15
  ;;   8. User 3: Reset → Ok 0
  ;;   9. Admin: Stop → Stopped
  ;;  10. Thread/drain-and-join
  (:wat::core::let
    [;; Inner let: holds all Senders + does all communication
     thread
       (:wat::core::let
         [;; Spawn server (empty registry, next-id=0)
          thread        (:counter::spawn3)
          admin-tx      (:wat::kernel::Thread/input  thread)
          admin-resp-rx (:wat::kernel::Thread/output thread)

          ;; Provision user 1 (initial state = 10)
          p1-result     (:counter::admin-provision3 admin-tx admin-resp-rx 10)
          id1           (:wat::core::first  p1-result)
          tx1           (:wat::core::second p1-result)
          rx1           (:wat::core::third  p1-result)
          _id1-ok       (:wat::test::assert-eq id1 "client-0")

          ;; Provision user 2 (initial state = 100)
          p2-result     (:counter::admin-provision3 admin-tx admin-resp-rx 100)
          id2           (:wat::core::first  p2-result)
          tx2           (:wat::core::second p2-result)
          rx2           (:wat::core::third  p2-result)
          _id2-ok       (:wat::test::assert-eq id2 "client-1")

          ;; Provision user 3 (initial state = 0)
          p3-result     (:counter::admin-provision3 admin-tx admin-resp-rx 0)
          id3           (:wat::core::first  p3-result)
          tx3           (:wat::core::second p3-result)
          rx3           (:wat::core::third  p3-result)
          _id3-ok       (:wat::test::assert-eq id3 "client-2")

          ;; Per-user state independent:
          ;; User 1: Increment 5 → expect Ok 15
          after-u1-inc  (:counter::user-increment3 tx1 rx1 5)
          _             (:wat::test::assert-eq after-u1-inc 15)

          ;; User 2: Increment 50 → expect Ok 150
          after-u2-inc  (:counter::user-increment3 tx2 rx2 50)
          _             (:wat::test::assert-eq after-u2-inc 150)

          ;; User 3: Get → expect Value 0
          u3-get        (:counter::user-get3 tx3 rx3)
          _             (:wat::test::assert-eq u3-get 0)

          ;; Deprovision user 2 → server drops registry entry
          dep-id        (:counter::admin-deprovision3 admin-tx admin-resp-rx id2)
          _             (:wat::test::assert-eq dep-id "client-1")

          ;; User 1 still works: Get → expect Value 15
          u1-get        (:counter::user-get3 tx1 rx1)
          _             (:wat::test::assert-eq u1-get 15)

          ;; User 3 still works: Reset → expect Ok 0
          u3-reset      (:counter::user-reset3 tx3 rx3)
          _             (:wat::test::assert-eq u3-reset 0)

          ;; Admin: Stop → Stopped
          _stop         (:counter::admin-stop3 admin-tx admin-resp-rx)]
         ;; Return thread — all Senders (admin-tx, tx1, tx2, tx3) drop here
         thread)
     ;; Outer scope: join after all Senders dropped
     _drained (:wat::core::Result/expect -> :wat::core::nil
                (:wat::kernel::Thread/drain-and-join thread)
                "counter-service-N3: thread died unexpectedly")]
    :wat::core::nil))
