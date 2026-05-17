;; wat-tests/counter-service-thread-N1.wat — Server dispatch loop, thread tier, N=1 user.
;;
;; Arc 203 slice 3a — first stepping stone toward the server-pattern proofs.
;; Proves:
;;   - Server actor selects across admin-rx + ONE user-rx via :wat::kernel::select
;;   - Select is uniform-T (∀T. Vec<Receiver<T>>): requires unified Wire enum
;;     (STOP 1 fired; see SCORE-SLICE-3A.md)
;;   - Admin can Stop the server (returns final state via the admin channel)
;;   - User can Get/Increment/Reset (full counter protocol via the user channel)
;;   - Admin and user clients logically separated by Wire variant;
;;     protocol enforces per-client discipline at the server (behavior enforces,
;;     not type system — honest delta from arc 198/203 "behavior enforces" lesson)
;;
;; Wire enum pivot (STOP 1):
;;   :wat::kernel::select is ∀T — all receivers in the Vec must have the same T.
;;   AdminReq ≠ UserReq, so heterogeneous [Receiver<AdminReq>, Receiver<UserReq>]
;;   is rejected by the type checker. Resolution: unified :counter::Wire enum
;;   wraps both admin and user requests; server selects on Vec<Receiver<Wire>>;
;;   routes by Wire variant (Admin idx=0, User idx=1 or vice versa by arrival).
;;   The Wire-variant match inside dispatch is the separation mechanism.
;;
;; Channel architecture (post-pivot):
;;   spawn-thread auto-channels carry (I=Wire, O=AdminResp):
;;     Thread/input(thread)  = Sender<Wire>   → admin-tx (parent sends admin Wire msgs)
;;     Thread/output(thread) = Receiver<AdminResp> → admin-resp-rx (parent recvs admin replies)
;;   make-bounded-channel for user wire + user resp:
;;     user-wire-pair = (user-tx: Sender<Wire>, user-wire-rx: Receiver<Wire>)
;;     user-resp-pair = (user-resp-tx: Sender<UserResp>, user-resp-rx: Receiver<UserResp>)
;;
;;   Server dispatch closes over user-wire-rx + user-resp-tx from outer scope;
;;   auto-channels provide admin-wire-rx + admin-resp-tx.
;;   select set: [admin-wire-rx, user-wire-rx] — both Receiver<Wire>. ✓
;;
;; Spawn return shape (3-tuple, first/second/third accessible):
;;   first  = thread          (Thread<Wire, AdminResp> — admin channels via Thread/input+output)
;;   second = user-tx         (Sender<Wire> — user client sends Wire::User variants)
;;   third  = user-resp-rx    (Receiver<UserResp> — user client recvs responses)
;;
;; Deftest exercises:
;;   User: Increment 5 → assert Ok 15
;;   User: Increment 7 → assert Ok 22
;;   User: Get         → assert Value 22
;;   User: Reset       → assert Ok 0
;;   Admin: Stop       → assert Stopped 0

(:wat::test::deftest :counter-service::thread-N1
  (;; ─── Admin protocol ──────────────────────────────────────────────────
   ;;
   ;; :counter::AdminReq — privileged operations.
   ;; 3a: only Stop. 3b adds Provision/Deprovision.
   (:wat::core::enum :counter::AdminReq
     (Stop))

   ;; :counter::AdminResp — server's reply to admin.
   ;; Stopped carries the server's final state (for auditing/handoff).
   (:wat::core::enum :counter::AdminResp
     (Stopped (final :wat::core::i64)))

   ;; ─── User protocol ───────────────────────────────────────────────────
   ;;
   ;; :counter::UserReq — RPC operations.
   (:wat::core::enum :counter::UserReq
     (Get)
     (Increment (n :wat::core::i64))
     (Reset))

   ;; :counter::UserResp — server's reply to user.
   ;;   Value — reply to Get (current, unchanged state)
   ;;   Ok    — reply to Increment + Reset (new state)
   (:wat::core::enum :counter::UserResp
     (Value (v :wat::core::i64))
     (Ok    (v :wat::core::i64)))

   ;; ─── Wire enum (STOP 1 pivot) ─────────────────────────────────────────
   ;;
   ;; Unified request type for both admin and user channels.
   ;; select is ∀T — all receivers must share the same T.
   ;; Wire wraps both admin and user requests so the select Vec is homogeneous.
   ;; Server dispatches by matching the Wire variant.
   (:wat::core::enum :counter::Wire
     (Admin (req :counter::AdminReq))
     (User  (req :counter::UserReq)))

   ;; ─── Server dispatch loop ─────────────────────────────────────────────
   ;;
   ;; :counter::dispatch — the server's message loop.
   ;;
   ;; Takes:
   ;;   admin-wire-rx  — server's Receiver<Wire> for the admin logical channel
   ;;   admin-resp-tx  — server's Sender<AdminResp> to reply to admin
   ;;   user-wire-rx   — server's Receiver<Wire> for the user logical channel
   ;;   user-resp-tx   — server's Sender<UserResp> to reply to user
   ;;   state          — current counter value
   ;;
   ;; Builds select set [admin-wire-rx, user-wire-rx] (both Receiver<Wire>).
   ;; Selects, matches Wire variant, dispatches to admin or user handler.
   ;; Admin/Stop → send AdminResp::Stopped(state) + return nil (thread exits).
   ;; User/* → send UserResp, recur with (possibly updated) state.
   ;;
   ;; Note: dispatch takes BOTH wire receivers to pass to select each iteration.
   ;; This is the tail-call shape: recur with same channel references + new state.
   (:wat::core::defn :counter::dispatch
     [admin-wire-rx <- :wat::kernel::Receiver<counter::Wire>
      admin-resp-tx <- :wat::kernel::Sender<counter::AdminResp>
      user-wire-rx  <- :wat::kernel::Receiver<counter::Wire>
      user-resp-tx  <- :wat::kernel::Sender<counter::UserResp>
      state         <- :wat::core::i64]
     -> :wat::core::nil
     (:wat::core::let
       [;; Build select set — both receivers carry Wire (uniform T).
        rxs     (:wat::core::Vector :wat::kernel::Receiver<counter::Wire>
                  admin-wire-rx user-wire-rx)
        ;; Block until one receiver is ready.
        chosen  (:wat::kernel::select rxs)
        idx     (:wat::core::first chosen)
        result  (:wat::core::second chosen)]
       ;; Match result — arc 111: Result<Option<Wire>, ThreadDiedError>
       (:wat::core::match result -> :wat::core::nil
         ;; Got a message
         ((:wat::core::Ok (:wat::core::Some wire))
           (:wat::core::match wire -> :wat::core::nil
             ;; Admin message — only Stop in 3a
             ((:counter::Wire::Admin req)
               (:wat::core::match req -> :wat::core::nil
                 ((:counter::AdminReq::Stop)
                   ;; Terminal: send Stopped + return nil (thread exits)
                   (:wat::core::Result/expect -> :wat::core::nil
                     (:wat::kernel::send admin-resp-tx
                       (:counter::AdminResp::Stopped state))
                     "dispatch: admin-resp-tx disconnected on Stop"))))
             ;; User message — Get / Increment / Reset
             ((:counter::Wire::User req)
               (:wat::core::match req -> :wat::core::nil
                 ;; Get — no state change; reply Value(state); recur
                 ((:counter::UserReq::Get)
                   (:wat::core::do
                     (:wat::core::Result/expect -> :wat::core::nil
                       (:wat::kernel::send user-resp-tx
                         (:counter::UserResp::Value state))
                       "dispatch: user-resp-tx disconnected on Get")
                     (:counter::dispatch
                       admin-wire-rx admin-resp-tx
                       user-wire-rx  user-resp-tx
                       state)))
                 ;; Increment — compute new state; reply Ok(new-n); recur
                 ((:counter::UserReq::Increment n)
                   (:wat::core::let [new-n (:wat::core::i64::+'2 state n)]
                     (:wat::core::Result/expect -> :wat::core::nil
                       (:wat::kernel::send user-resp-tx
                         (:counter::UserResp::Ok new-n))
                       "dispatch: user-resp-tx disconnected on Increment")
                     (:counter::dispatch
                       admin-wire-rx admin-resp-tx
                       user-wire-rx  user-resp-tx
                       new-n)))
                 ;; Reset — reply Ok(0); recur with 0
                 ((:counter::UserReq::Reset)
                   (:wat::core::do
                     (:wat::core::Result/expect -> :wat::core::nil
                       (:wat::kernel::send user-resp-tx
                         (:counter::UserResp::Ok 0))
                       "dispatch: user-resp-tx disconnected on Reset")
                     (:counter::dispatch
                       admin-wire-rx admin-resp-tx
                       user-wire-rx  user-resp-tx
                       0)))))))
         ;; Clean disconnect (sender dropped) — treat as normal exit
         ((:wat::core::Ok :wat::core::None)
           ())
         ;; Thread died — treat as normal exit
         ((:wat::core::Err _died)
           ()))))

   ;; ─── Server constructor ───────────────────────────────────────────────
   ;;
   ;; :counter::spawn — creates channel pairs, spawns thread, returns
   ;; client-side ends as a 3-tuple.
   ;;
   ;; Channel architecture:
   ;;   spawn-thread(I=Wire, O=AdminResp):
   ;;     Thread/input(thread)  = Sender<Wire>     = admin-tx for admin client
   ;;     Thread/output(thread) = Receiver<AdminResp> = admin-resp-rx for admin client
   ;;   make-bounded-channel for user wire:
   ;;     user-tx (Sender<Wire>) — user client sends Wire::User variants
   ;;     user-wire-rx (Receiver<Wire>) — server reads user requests via select
   ;;   make-bounded-channel for user resp:
   ;;     user-resp-tx (Sender<UserResp>) — server sends user responses
   ;;     user-resp-rx (Receiver<UserResp>) — user client recvs responses
   ;;
   ;; Returns 3-tuple:
   ;;   first  = thread       (Thread<Wire,AdminResp>)
   ;;   second = user-tx      (Sender<Wire>)
   ;;   third  = user-resp-rx (Receiver<UserResp>)
   ;;
   ;; Caller extracts admin channels from thread:
   ;;   admin-tx = Thread/input(thread)
   ;;   admin-resp-rx = Thread/output(thread)
   (:wat::core::defn :counter::spawn
     [initial <- :wat::core::i64]
     -> :(wat::kernel::Thread<counter::Wire,counter::AdminResp>,wat::kernel::Sender<counter::Wire>,wat::kernel::Receiver<counter::UserResp>)
     (:wat::core::let
       [;; User wire channel: parent sends Wire::User msgs; server reads them
        user-wire-pair  (:wat::kernel::make-bounded-channel :counter::Wire 1)
        user-tx         (:wat::core::first  user-wire-pair)
        user-wire-rx    (:wat::core::second user-wire-pair)
        ;; User resp channel: server sends UserResp; parent reads them
        user-resp-pair  (:wat::kernel::make-bounded-channel :counter::UserResp 1)
        user-resp-tx    (:wat::core::first  user-resp-pair)
        user-resp-rx    (:wat::core::second user-resp-pair)
        ;; Spawn server thread.
        ;; spawn-thread auto-channels carry (I=Wire, O=AdminResp):
        ;;   server gets admin-wire-rx (Receiver<Wire>) — the admin logical channel
        ;;   server gets admin-resp-tx (Sender<AdminResp>) — to reply to admin
        ;; Server closes over user-wire-rx + user-resp-tx from outer scope.
        thread
         (:wat::kernel::spawn-thread
           (:wat::core::fn
             [admin-wire-rx <- :wat::kernel::Receiver<counter::Wire>
              admin-resp-tx <- :wat::kernel::Sender<counter::AdminResp>]
             -> :wat::core::nil
             (:counter::dispatch
               admin-wire-rx admin-resp-tx
               user-wire-rx  user-resp-tx
               initial)))]
       (:wat::core::Tuple thread user-tx user-resp-rx)))

   ;; ─── User client wrappers ─────────────────────────────────────────────
   ;;
   ;; Each wrapper takes (user-tx, user-resp-rx) and wraps a send + recv
   ;; round-trip. Uses Result/expect + Option/expect per arc 110/111 discipline.
   ;;
   ;; User client sends Wire::User variants; recvs UserResp variants.

   (:wat::core::defn :counter::user-increment
     [user-tx    <- :wat::kernel::Sender<counter::Wire>
      user-rx    <- :wat::kernel::Receiver<counter::UserResp>
      n          <- :wat::core::i64]
     -> :wat::core::i64
     (:wat::core::let
       [_sent
         (:wat::core::Result/expect -> :wat::core::nil
           (:wat::kernel::send user-tx (:counter::Wire::User (:counter::UserReq::Increment n)))
           "user-increment: user-tx disconnected")
        resp
         (:wat::core::Option/expect -> :counter::UserResp
           (:wat::core::Result/expect -> :wat::core::Option<counter::UserResp>
             (:wat::kernel::recv user-rx)
             "user-increment: recv peer died")
           "user-increment: clean disconnect")]
       (:wat::core::match resp -> :wat::core::i64
         ((:counter::UserResp::Ok    v) v)
         ((:counter::UserResp::Value v) v))))

   (:wat::core::defn :counter::user-get
     [user-tx    <- :wat::kernel::Sender<counter::Wire>
      user-rx    <- :wat::kernel::Receiver<counter::UserResp>]
     -> :wat::core::i64
     (:wat::core::let
       [_sent
         (:wat::core::Result/expect -> :wat::core::nil
           (:wat::kernel::send user-tx (:counter::Wire::User (:counter::UserReq::Get)))
           "user-get: user-tx disconnected")
        resp
         (:wat::core::Option/expect -> :counter::UserResp
           (:wat::core::Result/expect -> :wat::core::Option<counter::UserResp>
             (:wat::kernel::recv user-rx)
             "user-get: recv peer died")
           "user-get: clean disconnect")]
       (:wat::core::match resp -> :wat::core::i64
         ((:counter::UserResp::Ok    v) v)
         ((:counter::UserResp::Value v) v))))

   (:wat::core::defn :counter::user-reset
     [user-tx    <- :wat::kernel::Sender<counter::Wire>
      user-rx    <- :wat::kernel::Receiver<counter::UserResp>]
     -> :wat::core::i64
     (:wat::core::let
       [_sent
         (:wat::core::Result/expect -> :wat::core::nil
           (:wat::kernel::send user-tx (:counter::Wire::User (:counter::UserReq::Reset)))
           "user-reset: user-tx disconnected")
        resp
         (:wat::core::Option/expect -> :counter::UserResp
           (:wat::core::Result/expect -> :wat::core::Option<counter::UserResp>
             (:wat::kernel::recv user-rx)
             "user-reset: recv peer died")
           "user-reset: clean disconnect")]
       (:wat::core::match resp -> :wat::core::i64
         ((:counter::UserResp::Ok    v) v)
         ((:counter::UserResp::Value v) v))))

   ;; ─── Admin client wrapper ─────────────────────────────────────────────
   ;;
   ;; Admin client sends Wire::Admin variants on admin-tx (= Thread/input(thread)).
   ;; Admin client recvs AdminResp on admin-resp-rx (= Thread/output(thread)).

   (:wat::core::defn :counter::admin-stop
     [admin-tx      <- :wat::kernel::Sender<counter::Wire>
      admin-resp-rx <- :wat::kernel::Receiver<counter::AdminResp>]
     -> :wat::core::i64
     (:wat::core::let
       [_sent
         (:wat::core::Result/expect -> :wat::core::nil
           (:wat::kernel::send admin-tx (:counter::Wire::Admin (:counter::AdminReq::Stop)))
           "admin-stop: admin-tx disconnected")
        resp
         (:wat::core::Option/expect -> :counter::AdminResp
           (:wat::core::Result/expect -> :wat::core::Option<counter::AdminResp>
             (:wat::kernel::recv admin-resp-rx)
             "admin-stop: recv peer died")
           "admin-stop: clean disconnect")]
       (:wat::core::match resp -> :wat::core::i64
         ((:counter::AdminResp::Stopped final) final)))))

  ;; ─── Test body ─────────────────────────────────────────────────────────
  ;;
  ;; Spawn server with initial state 10.
  ;; Extract admin channels from thread handle; user channels from tuple.
  ;; User exercises: Increment 5 → 15, Increment 7 → 22, Get → 22, Reset → 0.
  ;; Admin: Stop → Stopped(0).
  ;; Drain-and-join thread.
  ;;
  ;; SERVICE-PROGRAMS lockstep: all Senders (admin-tx, user-tx) must be
  ;; dropped BEFORE drain-and-join. Inner let holds senders + does all
  ;; communication; returns thread to outer scope. Outer scope calls
  ;; Thread/drain-and-join after all senders have dropped.
  (:wat::core::let
    [;; Inner let: holds all Senders; does all communication; returns thread.
     ;; When inner let exits, admin-tx and user-tx are dropped → server sees EOF.
     thread
       (:wat::core::let
         [;; Spawn and unpack 3-tuple
          spawn-result    (:counter::spawn 10)
          thread          (:wat::core::first  spawn-result)
          user-tx         (:wat::core::second spawn-result)
          user-resp-rx    (:wat::core::third  spawn-result)
          ;; Admin channels from the Thread handle
          admin-tx        (:wat::kernel::Thread/input  thread)
          admin-resp-rx   (:wat::kernel::Thread/output thread)
          ;; User round-trips — Increment 5 → 15
          after-inc-5     (:counter::user-increment user-tx user-resp-rx 5)
          _               (:wat::test::assert-eq after-inc-5 15)
          ;; Increment 7 → 22
          after-inc-7     (:counter::user-increment user-tx user-resp-rx 7)
          _               (:wat::test::assert-eq after-inc-7 22)
          ;; Get → 22
          got             (:counter::user-get user-tx user-resp-rx)
          _               (:wat::test::assert-eq got 22)
          ;; Reset → 0
          after-reset     (:counter::user-reset user-tx user-resp-rx)
          _               (:wat::test::assert-eq after-reset 0)
          ;; Admin Stop — server replies Stopped(0) and exits
          final-state     (:counter::admin-stop admin-tx admin-resp-rx)
          _               (:wat::test::assert-eq final-state 0)]
         ;; Return only the thread — all senders drop here
         thread)
     ;; Outer scope: drain Thread/output + join after senders dropped.
     ;; drain-and-join sees an empty/disconnected output channel (admin-resp-rx
     ;; already consumed the Stopped response; server exited after sending it).
     _drained (:wat::core::Result/expect -> :wat::core::nil
                (:wat::kernel::Thread/drain-and-join thread)
                "counter-service: thread died unexpectedly")]
    :wat::core::nil))
