# FFLO-0002. Saturation Evidence

Date: 2026-09-06
Last-reviewed: 2026-09-06
Tier: B
Status: Accepted

## Related

Root: FFLO-0002

## Context

Average throughput obscures queue aging, retained capacity and the onset of unstable congestion. This independent root governs the evidence used for capacity and scheduling decisions, not a chosen admission mechanism. Shared flow applicability is not a parent dependency.

## Decision

Capacity and scheduling decisions use measured queue behavior and explicit overload responses.

R1 [6]: Queue telemetry MUST distinguish items, bytes, active tasks, waiting producers, completed results and oldest-work age.
R2 [6]: Queue measurements MUST record build, workload, concurrency, machine conditions, date and high-water marks with explicit exclusions.
R3 [5]: Scheduling policy MUST define fairness and overload escalation; tuning SHOULD measure latency and throughput trade-offs rather than copy utilization or batch constants.

## Consequences

+ becomes easier: comparing saturation observations under named conditions.

− becomes harder: collecting queue age, retained results and machine conditions adds instrumentation and analysis cost.

risks/migration: finite measurements do not establish universal worst-case bounds; copied utilization targets can hide workload differences.

Evidence: [SOURCES](../../../SOURCES.md#repository-families) excludes upstream utilization and failure heuristics. R1–R3 require adopter telemetry, dated measurement records and scheduling tradeoff review; this corpus supplies no benchmark or high-water measurement.
