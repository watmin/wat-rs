# BRIEF — STONE 255.31: a thread address is inert data — the registry goes

**Drawn 2026-09-25 against `main` @ `aca395318`.** Floor 6116/6116, clippy 0, census 216 non-zero, delta
**NEW 2 / RECOVERY 0**, ledger 208. **Executor: grok, via pulsare.** Strike, write the SCORE, then
`pulsare_yield kind=scored`.

## Ruling (builder, 2026-09-25)

> *"mutexs are illegal in wat.... the address of a thread is a useless concept .... a value is necessary
> to be protocol compliant but we need a mutex in the system now?.... a 'disconnect' has no meaning in
> threads?.... this only has meaning when we are not using shared memory where processes or networked
> callers can go"*

## Why — measured by the orchestrator on `aca395318`

The 255.29 registry (`src/kernel/address.rs`: `register_rendezvous` ~:137, `resolve_rendezvous` ~:150, a
`Mutex<HashMap<u64, Weak<…>>>`) exists **only** to turn a *decoded* thread address back into its channel.
But:

- **On a thread channel a thread address is never encoded or decoded.** `typed_send`
  (`src/channel/transfer.rs`) passes the live `Value` through crossbeam. The lineage `Status.Started` and
  the pool's `PoolMsg.Setup` travel as live values.
- **The only decoder is the capability codec** (`src/capability/registry.rs:289` →
  `Address::from_thread_wire`), which runs only when a thread address has crossed a **process** wire.

So the registry serves one path: a thread address that went out to a process and came back to its minter
to be dialed again (255.29's "echoed copy connects in the parent" row). That gives a useless concept a
working implementation, at the price of the only `Mutex` in the address layer.

## The work

1. **Delete the registry:** `register_rendezvous`, `resolve_rendezvous`, the map, the `Mutex`, and the id
   counter, if nothing else needs it.
   - A thread address's data form is a **protocol-compliant inert value**. Keep the wire record and its
     trusted-door encoding (so a `Status`/`PoolMsg` holding one stays encodable), but carry only what it
     needs to be well-formed. Say what you kept and why.
   - **Decoding one anywhere yields an undialable address.** `connect` on it answers with one honest
     sentence: *a thread address is dialable only through the live value; one that crossed a wire is
     inert.* Choose the `ConnectOutcome` and say why.
   - **Census first:** everything that touches the registry or `from_thread_wire`. The orchestrator
     expects only 255.29's echo row and unit tests. That row becomes a **negative** row (the echoed copy
     is undialable), not a deleted one.
2. **Rename 255.30's negative row** `tests/kernel/probe_arc255_30_struct_on_thread_peer.wat` →
   `.wat.bad`, and update its driver. It is rc 3 by design, and the `.wat.bad` gate must see it. The census
   non-zero count should return to **215**. Measure it.
3. **Measurement, reported and NOT changed: the outcome vocabulary per locus.** For every variant of
   `ConnectOutcome`, `RecvOutcome`, `SendOutcome`, `AcceptFail`/`ConnectFail` and `ServiceEvent`, a table
   of:
   - **which loci can produce it**, thread and process, read from the producing code (file:line);
   - **what it claims:** terminal vs **retryable** (e.g. `ConnectFail::Refused` is documented *"RETRYABLE
     transport; the server may come up"*);
   - **whether that claim is true on each locus.** Can a dropped thread listener ever come back?

   This table is for the builder's ruling on whether the thread tier should answer "gone" where it now
   claims "retryable". **Do not change any outcome kind in this stone.**

## STOP triggers

1. Something besides the echo row and the unit tests needs a decoded thread address to be dialable →
   STOP, and report it verbatim.
2. A census file other than the renamed row changes rc → report each with its first error.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| registry, `Mutex`, id counter | gone (grep) |
| a thread-locus defservice, the kwargs bracket pool (255.30's rows) | still run |
| the echoed thread address | undialable, with the honest sentence; pre-stone it connected |
| a thread address sent to a process child | still encodes; the child's dial is refused |
| census non-zero | **215** |
| floor · clippy · delta · ledger | green · 0 · NEW 2 / RECOVERY 0 · ≤ 208 |
| the outcome-vocabulary table | in the SCORE, every variant, with file:line |

## Doctrine (the house rules; `wat-rs/CLAUDE.md` is not injected, so they are here)

- The floor is **`scripts/floor.sh`** (`cargo nextest run --release`), captured to `.floor/`. Read the
  Summary line, never a piped exit code.
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Never re-run to green. Copy the failing block
  **verbatim** from the captured log, name the assertion that fired, fix it at its cause if it is this
  stone's, then re-run the whole floor and say so.
- Also run clippy (force a recheck by touching a file), `scripts/replay/census.sh --diff` against a
  pre-change census you take first, `scripts/replay/delta.sh`, and the keyword heresy ledger (it may
  shrink, never grow).
- ⭐ **Prove every row can say BOTH words** on a pre-stone binary built from this brief's commit. Capture
  `rc=$?` into a variable on the **next** statement.
- **Timing, if you measure it:** A/B the pre and new binaries interleaved in one session.
- `.wat` edits across files use a **wat-fix codemod** run with `./target/release/wat`. Never python/sed
  for `.wat`.
- **If this brief contradicts the code, the code wins — say so plainly** in the SCORE.
- Commit locally with `git add -- <paths>` (renames with `git mv`); **do not push**. Write
  `SCORE-STONE-255.31-a-thread-address-is-inert-data.md` beside this brief.
