# BRIEF — STONE 255.29: a thread address is data

**Drawn 2026-09-25 against `main` @ `388e30c12`.** Floor 6106/6106, clippy 0, census 215 non-zero, delta
**NEW 2 / RECOVERY 0**, ledger 211. **Executor: grok, via pulsare.** Strike, write the SCORE, then
`pulsare_yield kind=scored`.

## Read first, in order

1. `FINDING-what-relies-on-the-thread-escape-hatch.md`: the only resources crossing thread peers are
   thread-tier `Address`es, in the lineage `Status.Started` (2461 sends, 35 services) and in the bracket
   pool's `PoolMsg.Setup` (3 sends).
2. `FINDING-B3-a-thread-address-can-be-data.md`, with its prototype in
   `probes-b3/b3-prototype.diff.txt`: a portable thread address, with every thread payload round-tripping
   as data and no measurable cost.
3. `WEIGH-STONE-255.28-purity-sees-through-a-generic.md`: the purity wall now sees through generics.

## Rulings (builder, 2026-09-25)

- ⭐ **Only data crosses a comm, on every locus.** Resources are declared only in `:ephemeral` (the
  defservice header: `:durable` *crosses the wire*; `:ephemeral` *resources + peer clients; never
  crosses*).
- ⭐ **"auth on threads is nonsense."** A thread address's data form needs **no token, no nonce, no
  authentication**. It only has to be a protocol-compliant value. *"mtls or some auth … will be required"*
  for networked peers, later and not here.

## The work

1. **The thread-tier `CommAddress` gets a data form.** Start from the B3 prototype and **simplify** it:
   - **no nonce, no random authentication**: the ruling;
   - a listener id resolved through an in-process registry;
   - **keep the registry weak**, so dropping the last `Address` still closes the listener (the prototype
     measured this preserves today's `Closed`/"address was dropped" behaviour).
   - A decoded thread address dialed **in another process** does not resolve. Answer with a routing fact
     (`Rejected`, "a thread address is dialable only inside its minting process"; the prototype's
     message). This is not authentication; keep only what is needed to say it (e.g. the minter pid).
   - ⚠ **ZERO-MUTEX:** the prototype used a `Mutex`. Prefer a lock-free structure. If a lock is truly
     required, measure why and **report it** in the SCORE; do not hide it.
2. **Encode it through the same trusted door as the socket tier** (`src/capability/registry.rs`
   `address_codec`, and `decode_trusted_wire`). General decode still refuses a capability tag. A
   `ThreadAddressWire` record, or whatever shape you choose; name it in the SCORE.
3. **Portable is not the same as dialable from another locus.** `address-wire?`
   (`src/kernel/identity.rs` ~:245–292) reads `portable_form()` as "a process may dial this". Keep that
   meaning. The prototype added a separate `thread_portable_form`. Keep the two properties separate, and
   assert that `address-wire?` stays false for a thread address.
4. **A `Shared` address is data now, so it is pure.** Flip the `Address` purity arm in `is_pure_type`
   (`src/check.rs`, the `"wat::kernel::Address"` arm keyed on `is_shared_marker`). Measured: this reddens
   exactly one fixture, `tests/types/probe_arc255_25_transport_family_address_shared_field.wat.bad`,
   whose row asserts the old rule. **Invert that row honestly:** the fixture becomes an accepted `.wat`,
   with its test's assertion and comment updated to say why (255.29 ruling). Do not delete it.
5. **Rows** (`tests/…/probe_arc255_29_*`):
   - a Shared address inside a **pure** record: accepted (pre-stone refused);
   - a thread address sent to a process child: the child's dial is `Rejected` with the routing message
     (pre-stone the child dies decoding: `unsupported substrate tag ##wat.kernel/Address`; B3 measured
     both);
   - a dangling id in the minting process: which `ConnectOutcome`? Measured `Refused` in B3, but
     `Refused` is labelled retryable and a dangling id never returns. **Measure and report**; do not
     change the outcome kinds in this stone;
   - every thread payload round-trips through encode → trusted decode (the B3 probe: 2592 identical).

## STOP triggers

1. The thread tier starts **serialising** on every send → STOP, report it. B3 measured that values still
   pass straight through crossbeam; encoding happens only at a process wire.
2. A launch or round-trip timing moves outside noise (B3: launch 826–873 µs, round trip 112.8–117.0 µs,
   6 interleaved runs, `probes-b3/timing-ab.txt`) → report it.
3. A census file changes rc → report each with its first error.

## Out of scope

**Closing the `ThreadSelfPeer` hatch** is stone 3 (255.30). Leave `ThreadSelfPeer` and its `derive` in
place. The lineage `Status`/`Admin` shape and the self-peer mechanism are unchanged.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| the Shared-in-pure-record row | accepted; pre-stone refused |
| a thread address to a process child | `Rejected` with the routing message; pre-stone the child dies decoding |
| `address-wire?` on a thread address | false |
| the inverted 255.25 fixture | accepted, test updated with the reason |
| floor · clippy · census · delta · ledger | green · 0 · `no STOP-8` · NEW 2 / RECOVERY 0 · ≤ 211 |

## Doctrine (the house rules; `wat-rs/CLAUDE.md` is not injected, so they are here)

- The floor is **`scripts/floor.sh`** (`cargo nextest run --release`), captured to `.floor/`. Read the
  Summary line, never a piped exit code.
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Never re-run to green. Copy the failing block
  **verbatim** from the captured log, name the assertion that fired, fix it at its cause if it is this
  stone's, then re-run the whole floor and say so.
- Also run clippy (force a recheck by touching a file), `scripts/replay/census.sh --diff` against a
  pre-change census you take first, `scripts/replay/delta.sh`, and the keyword heresy ledger (it may
  shrink, never grow; update it as its message says).
- ⭐ **Prove every row can say BOTH words** on a pre-stone binary built from this brief's commit. Capture
  `rc=$?` into a variable on the **next** statement; `echo "$(…) rc=$?"` prints the substitution's status.
- A `.wat` corpus migration is a **wat-fix codemod** run with `./target/release/wat` (the `cargo wat` in
  `~/.cargo/bin` is stale). Never python/sed for `.wat`. Scratch `.wat` goes in `wat-scripts/scratch-pad/`
  and must load.
- **If this brief contradicts the code, the code wins — say so plainly** in the SCORE.
- Commit locally with `git add -- <paths>`; **do not push**. Write
  `SCORE-STONE-255.29-a-thread-address-is-data.md` beside this brief with every row's pre/new rc and
  verbatim output, every floor Summary line, and every gate.
