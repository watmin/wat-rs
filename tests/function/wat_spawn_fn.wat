;; tests/function/wat_spawn_fn.wat — positive fixture for spawn-thread tests.
;; Three distinct :my::compute_tN functions (no user::main needed for eval_in_frozen).

;; T1: named-define body — :app::increment worker + compute that spawns it.
(:wat::core::defn :app::increment [in <- :wat::kernel::Receiver<wat::core::i64> out <- :wat::kernel::Sender<wat::core::i64>] -> :wat::core::nil
  (:wat::core::let
              [value
                (:wat::core::match (:wat::kernel::recv in)
                  
                  ((:wat::core::Ok (:wat::core::Some n)) n)
                  ((:wat::core::Ok :wat::core::None)
                   (:wat::kernel::raise! (:wat::core::Fault/of "input closed")))
                  ((:wat::core::Err _)
                   (:wat::kernel::raise! (:wat::core::Fault/of "parent died"))))
               sum (:wat::core::i64::+ value 1)]
              (:wat::core::match (:wat::kernel::send out sum)
                
                ((:wat::core::Ok _) ())
                ((:wat::core::Err _)
                 (:wat::kernel::raise! (:wat::core::Fault/of "output closed"))))))

(:wat::core::defn :my::compute_t1 [] -> :wat::core::i64
  (:wat::core::let
              [thr
                (:wat::kernel::spawn-thread :app::increment)
               tx
                (:wat::kernel::Thread/input thr)
               rx
                (:wat::kernel::Thread/output thr)
               _ack
                (:wat::core::match (:wat::kernel::send tx 41)
                  
                  ((:wat::core::Ok _) ())
                  ((:wat::core::Err _) (:wat::kernel::raise! (:wat::core::Fault/of "send died"))))
               result
                (:wat::core::match (:wat::kernel::recv rx)
                  
                  ((:wat::core::Ok (:wat::core::Some n)) n)
                  ((:wat::core::Ok :wat::core::None)    (:wat::kernel::raise! (:wat::core::Fault/of "early close")))
                  ((:wat::core::Err _)       (:wat::kernel::raise! (:wat::core::Fault/of "thread died"))))
               _join
                (:wat::core::match (:wat::kernel::Thread/drain-and-join thr)
                  
                  ((:wat::core::Ok _) ())
                  ((:wat::core::Err _) (:wat::kernel::raise! (:wat::core::Fault/of "join failed"))))]
              result))

;; T2: inline fn literal body.
(:wat::core::defn :my::compute_t2 [] -> :wat::core::i64
  (:wat::core::let
              [thr
                (:wat::kernel::spawn-thread
                  (:wat::core::fn
                    [in  <- :wat::kernel::Receiver<wat::core::i64>
                     out <- :wat::kernel::Sender<wat::core::i64>]
                     -> :wat::core::nil
                    (:wat::core::let
                      [value
                        (:wat::core::match (:wat::kernel::recv in)
                          
                          ((:wat::core::Ok (:wat::core::Some n)) n)
                          ((:wat::core::Ok :wat::core::None)
                           (:wat::kernel::raise! (:wat::core::Fault/of "input closed")))
                          ((:wat::core::Err _)
                           (:wat::kernel::raise! (:wat::core::Fault/of "parent died"))))
                       doubled (:wat::core::i64::* value 2)]
                      (:wat::core::match (:wat::kernel::send out doubled)
                        
                        ((:wat::core::Ok _) ())
                        ((:wat::core::Err _)
                         (:wat::kernel::raise! (:wat::core::Fault/of "output closed")))))))
               tx
                (:wat::kernel::Thread/input thr)
               rx
                (:wat::kernel::Thread/output thr)
               _ack
                (:wat::core::match (:wat::kernel::send tx 21)
                  
                  ((:wat::core::Ok _) ())
                  ((:wat::core::Err _) (:wat::kernel::raise! (:wat::core::Fault/of "send died"))))
               result
                (:wat::core::match (:wat::kernel::recv rx)
                  
                  ((:wat::core::Ok (:wat::core::Some n)) n)
                  ((:wat::core::Ok :wat::core::None)    (:wat::kernel::raise! (:wat::core::Fault/of "early close")))
                  ((:wat::core::Err _)       (:wat::kernel::raise! (:wat::core::Fault/of "thread died"))))
               _join
                (:wat::core::match (:wat::kernel::Thread/drain-and-join thr)
                  
                  ((:wat::core::Ok _) ())
                  ((:wat::core::Err _) (:wat::kernel::raise! (:wat::core::Fault/of "join failed"))))]
              result))

;; T3: closure capture — body captures `delta` from enclosing let.
(:wat::core::defn :my::compute_t3 [] -> :wat::core::i64
  (:wat::core::let
              [delta 100
               body
                (:wat::core::fn
                  [in  <- :wat::kernel::Receiver<wat::core::i64>
                   out <- :wat::kernel::Sender<wat::core::i64>]
                   -> :wat::core::nil
                  (:wat::core::let
                    [n
                      (:wat::core::match (:wat::kernel::recv in)
                        
                        ((:wat::core::Ok (:wat::core::Some v)) v)
                        ((:wat::core::Ok :wat::core::None)
                         (:wat::kernel::raise! (:wat::core::Fault/of "input closed")))
                        ((:wat::core::Err _)
                         (:wat::kernel::raise! (:wat::core::Fault/of "parent died"))))
                     sum (:wat::core::i64::+ n delta)]
                    (:wat::core::match (:wat::kernel::send out sum)
                      
                      ((:wat::core::Ok _) ())
                      ((:wat::core::Err _)
                       (:wat::kernel::raise! (:wat::core::Fault/of "output closed"))))))
               thr
                (:wat::kernel::spawn-thread body)
               tx
                (:wat::kernel::Thread/input thr)
               rx
                (:wat::kernel::Thread/output thr)
               _ack
                (:wat::core::match (:wat::kernel::send tx 23)
                  
                  ((:wat::core::Ok _) ())
                  ((:wat::core::Err _) (:wat::kernel::raise! (:wat::core::Fault/of "send died"))))
               result
                (:wat::core::match (:wat::kernel::recv rx)
                  
                  ((:wat::core::Ok (:wat::core::Some n)) n)
                  ((:wat::core::Ok :wat::core::None)    (:wat::kernel::raise! (:wat::core::Fault/of "early close")))
                  ((:wat::core::Err _)       (:wat::kernel::raise! (:wat::core::Fault/of "thread died"))))
               _join
                (:wat::core::match (:wat::kernel::Thread/drain-and-join thr)
                  
                  ((:wat::core::Ok _) ())
                  ((:wat::core::Err _) (:wat::kernel::raise! (:wat::core::Fault/of "join failed"))))]
              result))
