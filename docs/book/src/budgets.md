# Usage and budgets

Pleiades tracks live runtime usage and can stop active tasks when process-local
budgets are exceeded.

Inside the live workspace:

```text
/budget
/budget show
/budget tokens <count>
/budget cost <amount>
/budget time <duration>
/budget tools <count>
/budget reset
```

The report includes:

- input, output, cache-read, and cache-write tokens;
- total token usage;
- tool-call count;
- provider latency;
- tool execution time;
- elapsed runtime time;
- active token, cost, time, and tool limits;
- rate-limit status when a provider reports one.

Token, time, and tool-call budgets are enforced by the runtime. When a limit is
exceeded, the current task is stopped with a clear failure event and the session
is saved.

Cost budgets can be recorded, but cost enforcement requires reliable model
pricing data attached to provider usage events. Until that pricing source is
available, Pleiades reports cost as unavailable rather than inventing an
estimate.

External commands such as:

```bash
pleiades budget show
pleiades budget time 10m
```

run in a separate process. They validate syntax and explain the matching
workspace command, but they cannot mutate an already-running TUI runtime. Use
`/budget ...` inside `pleiades` for live task control.
