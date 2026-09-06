# FFLO-0002. Saturation Evidence

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FFLO-0002

## Context

Average throughput obscures queue aging, retained capacity and the onset of unstable congestion.

## Decision

Capacity and scheduling decisions use measured queue behavior and explicit overload responses.

R1 [6]: Queue telemetry MUST distinguish items, bytes, active tasks, waiting producers, completed results and oldest-work age.
R2 [6]: Measurements MUST record build, workload, concurrency, machine conditions, date and high-water marks with explicit exclusions.
R3 [5]: Scheduling policy MUST define fairness and overload escalation; tuning SHOULD measure latency and throughput trade-offs rather than copy utilization or batch constants.

## Consequences

Operational evidence guides capacity choices, but finite measurements do not establish universal worst-case bounds.
