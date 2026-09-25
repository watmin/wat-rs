# WEIGH — STONE 255.32: the server reaps a dead client — ACCEPTED

**Executor: grok via pulsare, commit `6a198e131`.** Weighed by the orchestrator against disk on 2026-09-25.

## Re-measured independently

| row | measured | result |
|---|---|---|
| floor | `.floor/2026-09-25T07-14-45Z` | `6118 tests run: 6118 passed`; both `arc255_32` tests PASS |
| the IDE's "unterminated character literal" (`…_lost_arm_expansion.rs:30`) | read the line | **false positive**: a raw string `r#"…"#`, and the test compiled and passed |
| the crash probe | `wat tests/services/probe_arc255_32_two_clients_see_the_crash.wat` | reproduced exactly (below) |

The emitted `Lost` arm is now `{:idx idx :cause _cause} (serve … (remove-at selectables idx) …)`: it evicts
and recurses, with no raise and no `eprintln`. Pre-stone it was `assertion-failed!`. Taken from the SCORE:
clippy 0; census `no STOP-8` (215); delta NEW 2 / RECOVERY 0; ledger 208. `bracket.wat`'s `Lost` arms are
untouched and correct, because the pool is the supervisor of its runners.

## The ruling, measured against the code

| vantage | ruling | measured |
|---|---|---|
| **the server** | a dead client just goes away, with no reason | ✅ **holds**: every server path (`poll'` and peers-only `select'`, both loci) turns a dead client into `ServiceEvent.Closed`. A server-side client has no crash channel |
| **a client** of a crashed service | an optimistic, **reasonless** notice | ✅ thread client-a and client-b, and the process client that sent the op, get the reason-free `PeerCrashed` sentence. ⛔ **The idle process client gets `Lost "io_uring read failed"`**: a transport-failure text, **not the notice**. **Violates the ruling** |
| **the owner/supervisor** | the reason | ✅ both loci: `Lost "P32-SERVICE-PANIC-REASON"` |

```
"thread client-a Lost peer crashed (abnormal far-side crash — no reason; …)"
"thread client-b Lost peer crashed (…)"
"thread owner Lost P32-SERVICE-PANIC-REASON"
"process client-a Lost peer crashed (…)"
"process client-b Lost io_uring read failed"        ⛔
"process owner Lost P32-SERVICE-PANIC-REASON"
```

## Carried

1. ⛔ **The idle process client's notice** (`src/comms/process.rs:751`/`:931`, then `message.rs:603`): the
   `PeerCrashed` broadcast reaches only the client whose op triggered the crash, and the other clients read
   a raw transport failure. **Its own stone.** Every connected client must get the reasonless notice.
2. **The vocabulary stones** (V1): the variant names still say `Lost` for every row above. Under the
   ruling:
   - the clients' fact is a **reasonless "the service died"**;
   - the owner's is **`Died [LociDiedError]`**;
   - the server's is **`Closed`**.
