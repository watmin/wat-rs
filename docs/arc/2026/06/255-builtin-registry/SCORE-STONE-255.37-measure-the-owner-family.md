# SCORE — STONE 255.37: measure the owner family

Measurement only, in a worktree of draw `020dfa8e5` (parent `e19e60cd3`).
Nothing from the worktree is on `main`. The diff is
`/home/john/.grok/sessions/%2Fhome%2Fjohn%2Fwork%2Fholon/01a07020-eac7-73c3-b82d-41a43dfd9416/255.37-worktree.diff`
(2 files: `wat/spawn.wat`, `src/check.rs`).

`(Thread :- [S R])` satisfies `(Spawned :- [S R])` once Spawned is a real
parametric head and the old derive edges stay. A send or recv on a Thread or
a Process still typechecks. The reds are places that declare `Peer` and are
handed an owner handle.

## The declaration

An empty `(:wat::core::defstruct :wat::spawn::Spawned :- [S R] [])` is refused:

```
#wat.type/UnconsumedTypeParam {:message "type parameter \"S\" in :wat::spawn::Spawned's param-spec is declared but never used — every parameter in a type declaration's param-spec must be consumed by a field, variant, or body type …" :decl ":wat::spawn::Spawned" :param "S"}
```

A record would be pure, and a handle marker is not EDN. The form that carries
the parameters and stays impure is a struct whose fields write the
discrimination the param-spec demands:

```
(:wat::core::defstruct :wat::spawn::Spawned :- [S R]
  [sent <- S
   recv <- R])
```

`sent` and `recv` are not a runtime layout. Thread and Process still derive
the head. The two `derive … Peer` edges are deleted. `recv`'s outcome type
is still `RecvOutcome`.

## The family rule

`project_peer_io` (send, recv, try-send), `select`, and `poll` accept a
2-argument head that derives `:wat::spawn::Spawned`, or the Peer head through
`is_peer_head`. `close` accepts only the Spawned family. `signal` stays
Process-only. The purity match on the three heads, the thread-program self
parameter, and the assignable Peer arms are unchanged.

`== "wat::kernel::{Thread,Process,Peer}"` in `src/check.rs`: 16 before, 6
after. The family rule replaced 10: `project_peer_io` 3, `close` 2, `select`
3, `poll` 2. The brief's 33 is not the count in this file.

The heresy ledger counts every compare in those functions, not only these
three names. It went 208 → 198:

```
⭐ THE LEDGER SHRANK — 208 → 198.
CURED:
  ↓ src/check.rs  fn infer_poll_prime  3 [Ex3] → 1 [Ex1]
  ↓ src/check.rs  fn infer_select_prime  4 [Ex4] → 1 [Ex1]
GONE ENTIRELY:
  − src/check.rs  fn infer_close_prime  was 2 [Ex2], now 0
  − src/check.rs  fn project_peer_io  was 3 [Ex3], now 0
```

Assertion `tests/lint/keyword_heresy_ledger.rs:1345`. The one compare left in
`select` and in `poll` is a compare this edit did not touch. The frozen total
was not updated. This is a measurement.

## The sweep

2652 tracked `*.wat` and `*.wat.bad`, both binaries. The pre-change
`target/release/wat` on a blank file is rc=0. The worktree binary on the same
file is rc=1, and the stderr is the four errors below. 2075 files produce
exactly that stderr and nothing of their own. Four files that are rc=0 on the
pre-change binary add their own mismatches. No other file does.

A `(Thread :- [i64 String])` ascribed as `(Spawned :- [i64 String])` adds no
error of its own.

### (a) an owner handle declared as Peer

The stdlib contributes four, and they are the whole failure of a blank file
(`BLANK_RC=1`):

```
#wat.check/CheckErrors {:message "4 type-check errors" :location nil :causes [] :errors [#wat.check/ReturnTypeMismatch {:message ":wat::spawn::ThreadOpts/spawn-runner: body produces (:wat::kernel::Thread :- [(:wat::bracket::PoolMsg :- [:D :I]) :(wat::core::i64,O)]); signature declares (:wat::kernel::Peer :- [(:wat::bracket::PoolMsg :- [:D :I]) :(wat::core::i64,O)])" :location #wat.core/Span {:file "wat/bracket.wat" :line 241 :col 5 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 243 :col 51}}} :causes [] :function ":wat::spawn::ThreadOpts/spawn-runner" :expected "(:wat::kernel::Peer :- [(:wat::bracket::PoolMsg :- [:D :I]) :(wat::core::i64,O)])" :got "(:wat::kernel::Thread :- [(:wat::bracket::PoolMsg :- [:D :I]) :(wat::core::i64,O)])" :remedies []} #wat.check/ReturnTypeMismatch {:message ":wat::spawn::ProcessOpts/spawn-runner: body produces (:wat::kernel::Process :- [_ _]); signature declares (:wat::kernel::Peer :- [(:wat::bracket::PoolMsg :- [:D :I]) :(wat::core::i64,O)])" :location #wat.core/Span {:file "wat/bracket.wat" :line 321 :col 5 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 321 :col 83}}} :causes [] :function ":wat::spawn::ProcessOpts/spawn-runner" :expected "(:wat::kernel::Peer :- [(:wat::bracket::PoolMsg :- [:D :I]) :(wat::core::i64,O)])" :got "(:wat::kernel::Process :- [_ _])" :remedies []} #wat.check/TypeMismatch {:message ":wat::spawn::Launched: parameter #1 expects (:wat::kernel::Peer :- [_ _]); got (:wat::kernel::Thread :- [:Sh :Lu])" :location #wat.core/Span {:file "wat/spawn.wat" :line 566 :col 38 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 566 :col 40}}} :causes [] :callee ":wat::spawn::Launched" :param "#1" :expected "(:wat::kernel::Peer :- [_ _])" :got "(:wat::kernel::Thread :- [:Sh :Lu])" :remedies []} #wat.check/TypeMismatch {:message ":wat::spawn::Launched: parameter #1 expects (:wat::kernel::Peer :- [_ _]); got (:wat::kernel::Process :- [:Sh _])" :location #wat.core/Span {:file "wat/spawn.wat" :line 630 :col 38 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 630 :col 41}}} :causes [] :callee ":wat::spawn::Launched" :param "#1" :expected "(:wat::kernel::Peer :- [_ _])" :got "(:wat::kernel::Process :- [:Sh _])" :remedies []}]}
```

`spawn-runner`'s signature still says Peer, and the body is Thread or
Process. `Launched.handle` is still `(Peer :- [Sh Lu])`, and the two
constructors pass a Thread and a Process. Downstream of `spawn-runner` stays
quiet because callers see the signature, which still says Peer. That is why
`bracket.wat`'s fold at the peer vector (the brief's :762) and a defservice
`Handle.handle` do not appear. The brief's list named consumers. The checker
fires at the seam where an owner value meets a Peer declaration.

Four more files, each rc=0 on the pre-change binary:

| file | line | message |
|---|---|---|
| `tests/program/wat_arc170_program_contracts_t18_echo_doubled.wat` | 42 | `recv-all` expects `(Peer :- [_ _])`, got `(Process :- [i64 _])` |
| `tests/program/wat_arc170_program_contracts_t18b_recv_assert_fail.wat` | 41 | the same |
| `tests/program/wat_arc170_program_contracts_t18c_recv_all_multi.wat` | 34 | the same |
| `wat-tests/counter-actor-proof-process.wat` | 233, 235, 237, 239, 241, 243 | `counter-proc::{increment,get,reset,shutdown}` expect `(Peer :- [counter::Request counter::Response])`, got `(Process :- [_ _])` |
| same file | 247 | `recv-all` expects Peer, got Process |

One of those, whole:

```
#wat.check/TypeMismatch {:message ":wat::kernel::recv-all: parameter #1 expects (:wat::kernel::Peer :- [_ _]); got (:wat::kernel::Process :- [:wat::core::i64 _])" :location #wat.core/Span {:file "tests/program/wat_arc170_program_contracts_t18_echo_doubled.wat" :line 42}}
```

`recv-all` and `recv-all-loop` are still declared `(Peer :- [I O])`. The
checked calls that fail pass a Process. A blind rewrite of every
`(Peer :- […])` to Spawned would also retype client peers, and those calls
still check. Each red site is an owner handle. A codemod can find a Peer
annotation whose value is headed Thread or Process. It cannot decide the
annotation from the type name alone.

### (b) a vector that mixes the two families

None. `select` and `poll` accept either family as the element. A mixed vector
would fail to unify before the verb. No file did.

### (c) one piece of code that receives from either family

Not in the checked corpus. Every new mismatch is an owner value at a Peer
parameter. `recv-all` is the function written so that an owner handle could
be passed where a Peer is declared. Its checked failing callers are all
Process. No checked call passed a counterparty Peer and failed.

### (d) the checker

Send, recv, try-send, select, poll, and close on a Thread or a Process do not
themselves go red. The ledger shrink above is the checker change the floor
refuses. `recv`'s outcome type was not touched.

### (e) other

The first Spawned declaration, the empty struct, is the UnconsumedTypeParam
above. It is not in the tree that was swept.

## The floor

Worktree `.floor/2026-09-25T18-46-08Z`. Not re-run.

```
Summary [ 316.773s] 6124 tests run: 2903 passed (1 slow), 3221 failed, 22 skipped
```

`FLOOR_RC=100`.

Two mechanisms.

The mass is startup. `call_beside_value` panics at `src/freeze.rs:1176` when
`startup_beside` returns the four stdlib errors above. 1211 of the captured
panics are that line. `every_ungated_wat_checks` panics at
`tests/lint/every_ungated_wat_checks.rs:74`: `10 of 10 ungated *.wat file(s)
do not type-check`. Same four errors. A world does not freeze, so a test that
starts one fails before its own assertion.

The ledger is the other mechanism, whole block:

```
FAIL [   2.683s] ( 242/6124) wat::lint keyword_heresy_ledger::the_heresy_ledger_matches_its_frozen_census
stdout:
    test keyword_heresy_ledger::the_heresy_ledger_matches_its_frozen_census ... FAILED
    test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 363 filtered out; finished in 2.68s
stderr:
    thread 'keyword_heresy_ledger::the_heresy_ledger_matches_its_frozen_census' (4025873) panicked at /tmp/wt-255.37/tests/lint/keyword_heresy_ledger.rs:1345:5:

    ⭐ THE LEDGER SHRANK — 208 → 198. This is the good direction, and the
    ratchet is deliberate: the frozen census below has to be tightened by hand so the number
    can never drift back up silently. Update `LEDGER_TOTAL` and the rows named here.

    CURED:
      ↓ src/check.rs  fn infer_poll_prime  3 [Ex3] → 1 [Ex1]
      ↓ src/check.rs  fn infer_select_prime  4 [Ex4] → 1 [Ex1]
    GONE ENTIRELY:
      − src/check.rs  fn infer_close_prime  was 2 [Ex2], now 0
      − src/check.rs  fn project_peer_io  was 3 [Ex3], now 0
```
