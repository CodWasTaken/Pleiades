# ADR 0008: Doom-loop detection

## Status

Accepted.

## Context

Autonomous agents can repeat the same failed action when the model does not
learn from tool output. Pleiades already had a global tool-iteration cap, but
that only stopped long runs after many iterations and did not explain repeated
identical failures clearly.

## Decision

Add a runtime doom-loop detector with a configurable repeat cap:

```toml
[agent]
max_repeats = 3
```

The detector records normalized loop signals such as identical tool failures,
stream errors, repeated reads, and repeated commands. When the same signal
appears `max_repeats` times in the sliding window, the runtime stops the task,
saves the session, and emits a `TaskFailed` event with a clear reason.

The first integrated path is identical tool failure detection in the agent
loop. This directly prevents repeated failing tool calls from exhausting the
full iteration budget.

## Consequences

Repeated failures now stop with evidence such as `Stopping: identical failure
... repeated 3 times`. Future repair-loop work can reuse the same detector for
verification commands, patch hashes, read loops, and provider stream errors.
