---
title: Distributed Consensus Trade-offs
tags: [distributed, consensus, networking]
created: 2026-02-01
last_validated: 2026-02-01
confidence: low
origins:
  - project: config-service
    date: 2026-02-01
    context: "Evaluating whether to use Raft for config propagation"
supersedes: []
---

Raft is simpler to implement and debug than Paxos but requires a stable leader. In environments with frequent network partitions, leaderless protocols (like EPaxos) avoid the leader-election storm problem.

For configuration propagation specifically, eventual consistency via gossip protocols is usually sufficient and dramatically simpler. Reserve consensus protocols for cases requiring linearizable reads — most services do not.

The CAP theorem is often misapplied. The real question is not "consistency or availability" but "how much inconsistency is tolerable for how long." Quantify the tolerance before choosing a protocol.
