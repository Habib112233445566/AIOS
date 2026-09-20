# Research Evidence - T-01871: Network Bootstrap Observability Research

- Domain: Linux network telemetry, sysfs/procfs interfaces, health evaluation, time-series ring buffer.
- Authoritative References: `man 5 proc` (`/proc/net/dev`), Linux `Documentation/ABI/testing/sysfs-class-net`, RFC 2863.
- Invariants Formulated: `NOBS1..NOBS6`.
- Decisions: 64-bit unsigned counters, 60-sample historical ring buffer, deterministic health classification (`Healthy`, `Degraded`, `Critical`).
