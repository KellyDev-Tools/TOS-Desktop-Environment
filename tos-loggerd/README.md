# TOS Log Service (`tos-loggerd`)

A high-performance event collection and query engine for system-wide telemetry.

## Design Drivers
- **Architecture Spec §19**: Defines the "TOS Log" as the unified stream for all component events.
- **Architecture Spec §3.3.4**: Specifies the `log_query` IPC contract for Face-driven log inspection.
- **Architecture Spec §6.2**: FEeds the "System Output Area" (Brain Console) rendered at Level 1.

## Responsibilities
- Structured JSONL logging for all daemons.
- High-speed filtering by surface, level, and timestamp.
- Log retention and rotation management.
