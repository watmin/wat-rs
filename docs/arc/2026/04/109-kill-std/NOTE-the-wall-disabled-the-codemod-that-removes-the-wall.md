# NOTE — the wall disabled the codemod whose job is removing what the wall forbids

> **2026-08-24, found while merging `grok-rete`.** Arc 109 walled the turbofish at the reader. The
> recorded migration `wat-scripts/fixes/angle-brackets-to-binder.wat` reads its input through
> `read-string` — which *is* that reader. **The codemod can no longer perform its own recorded job.**

## Measured, not reasoned — three binaries, none does both

| binary | reads turbofish | loads the codemod |
|---|---|---|
| `~/.cargo/bin/wat` (2026-08-20, pre-wall) | ✅ | ❌ 6 type-check errors — the codemod is written in `:-` syntax |
| `target/release/wat` (main, post-wall) | ❌ lex error inside `read-string` | ✅ |
| a historical codemod + the old binary | — | ❌ **no old version exists**: both commits of the codemod post-date the `:->` migration |

So a corpus predating arc 109 cannot be migrated by the tool built to migrate it.

## Why it bit, and why it will bite again

`grok-rete` branched before the wall. Merging main into it dies at a bootstrap cycle its own
`278/NOTE-main-merge-attempt.md` documents precisely: `wat_field_names_from!` parses `wat/rete.wat`
at **compile time**, so on a merged tree the unmigrated corpus is *inside* the binary and it cannot
start — you cannot build the tool you need to fix it.

**Any branch that predates `86e1b105a` hits this.** The parked branches
(`arc109-*-parked`, `origin/grok`, `origin/arc-*`) are all candidates.

## The procedure that worked — do this, do not hand-edit

A **bridge binary**: main's toolchain with the reader's wall lifted, built in a **throwaway clone**
so main's tree never holds a lifted wall.

1. `git clone --local <repo> /tmp/bridge`; symlink the sibling path-dep (`ln -s …/holon-rs`), or
   cargo cannot resolve `holon = { path = "../holon-rs" }`.
2. In the clone only, restore **pre-109 semantics** at four sites in
   `crates/wat-reader/src/lexer.rs` — `angle_depth += 1` at both type-head doors, and the
   **depth-guarded** comma at both body doors. ⛔ **Not a blanket deletion of the rejections**: that
   makes `,` an ordinary body char everywhere and breaks EDN whitespace in main's own migrated stdlib.
3. Gate the bridge four ways before trusting it — reads turbofish · still reads `:-` · loads the
   codemod · `(Vector :- [:i64] 1, 2, 3)` still lexes as **3** elements.
4. Stage the target files, run the codemod, **dry-run and `diff` one file first** (`wat/fix.wat`'s
   own instruction).
5. **Verify with the WALLED binary, never the bridge.** The bridge is a compromised instrument by
   construction; only main's real reader is the oracle.

Measured on the rete corpus: 26 files, **351 turbofish sites → 0 in code**. 57 survivors, all `;;`
comments — `fix.wat` is comment-faithful by design, so that is correct behaviour, not a miss.

## The one thing the codemod does NOT cover

The retired **tuple** spelling `:(A,B)` trips a *different* wall (`CommaInKeywordBody`) and
`angle-brackets-to-binder.wat` does not rewrite it. One site surfaced (`wat/rete.wat:291`,
`:(wat::core::Record,wat::core::i64)`); main had already migrated the identical construct at its own
line, so main's spelling was taken as the oracle. **A second codemod for the tuple form does not
exist and is the obvious gap.**

## The extirpare read

The ladder says: convention → a check that fires → a shape the mistake cannot take. Arc 109 climbed
to the top rung for *authoring* — and in doing so removed the rung the *migration tool* stood on.
**A wall that forbids a form must not blind the tool that removes that form**, or every future
migration needs a hand-built bridge.

Two candidate roots, neither taken here — the builder's call:

- **A reader mode.** The codemod opens its input through a reader that accepts the retired form and
  yields it as data. The wall stays absolute for *programs*; the *migration tool* gets a documented
  door. Cheapest, and it is what the bridge simulates by hand.
- **Keep the bridge, but make it a recorded procedure rather than a rediscovery.** This note is that,
  minus the automation.

Until one lands, this file is the procedure. It cost ~40 minutes to derive and ~2 to re-run.
