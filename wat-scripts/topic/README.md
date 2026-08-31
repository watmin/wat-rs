# `wat-scripts/topic/` — **wat-topic**, maturing toward a wat feature

A **topic**: one publisher, N subscribers, fan-out on publish. The SNS half of excursus 001.

Built here, in userland, on the `wat-grep` / `wat-gen` precedent — *"that's where we host our
repo's scripts; wat-grep is maturing into a wat feature"* (builder, arc 278). **It is promoted
to `wat/topic.wat` when it demonstrates excellence, and that promotion is the builder's
ruling** — never a side effect of the work.

The grep precedent also sets the standard for the move itself: *"mostly a MOVE of proven code,
and **the counts are the proof it moved intact**"* — the same numbers before and after,
re-run by the orchestrator rather than taken from a report.

## What is here

- **`sns-fanout.wat`** — one topic, three subscribers, and **one file that runs BOTH loci and
  prints both counts**, so the thread/process differential is the artifact rather than
  something a reader has to remember to run twice. Prints `"3 3"`.

```bash
./target/release/wat wat-scripts/topic/sns-fanout.wat     # => "3 3"
```

Its header carries two facts that are not written down elsewhere: a subscriber's birth-seed
allow-set holds only `getppid()`, so on the process locus the topic is a **stranger** and must
be granted; and a forked child's bundle does not carry the program's other `defn`s, so `:init`
must dial inline. It also documents the `bijection-anchor` wart and why it exists.

## Sibling

`wat-scripts/queue/` — **wat-queue**, the SQS half. A topic fanning out to N durable queues is
the shape the pair exists for.
