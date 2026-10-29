---
kind: method
id: std.async.SchedulerInterface::chooseNext
title: SchedulerInterface::chooseNext
summary: Selects one currently ready coroutine ID.
name: chooseNext
order: 1
typeParameters: []
parameters:
  - name: ready
    type: vector<int>
    description: The IDs THP can run now, in ready-queue order.
returns:
  type: int
  description: One ID supplied in ready.
errors:
  - description: Returning any other ID throws Error and fails the request.
related: []
status: experimental
availability: implemented
version: "0.1"
owner: std.async.SchedulerInterface
visibility: public
modifiers: []
---

THP calls `chooseNext()` only when at least one coroutine is ready. The callback
must return one supplied ID. It cannot call `await()`, `suspend()`, or `delay()`.
If it throws, the request fails. The runtime retains ownership of scheduling
state and cleanup.
