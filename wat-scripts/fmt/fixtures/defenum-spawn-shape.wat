(wat.core/defenum fix/SE :- [I O A] wat.enum/Impure
  :Shutdown
  :Admin [msg :- A]
  :Connection [peer :- (wat.kernel/Peer :- [I O])]
  :Message [idx :- wat.type/i64  msg :- O]
  :Closed [idx :- wat.type/i64]
  :Lost [idx :- wat.type/i64  cause :- wat.kernel/Failure]
  :Malformed [idx :- wat.type/i64  cause :- wat.kernel/Failure]
  :Rejected [idx :- wat.type/i64  cause :- wat.kernel/Failure])
