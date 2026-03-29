# TOS Priority Daemon (`tos-priorityd`)

The heuristic engine driving the dynamic visual alerts and tactical prioritization.

## Design Drivers
- **Architecture Spec §21**: "Priority-Weighted Visual Indicators" define the ranking system for system alerts (Level 1-5).
- **Architecture Spec §7.4**: Drives the "Priority Stack & Actions" column in the Command Hub.

## Responsibilities
- Real-time system health monitoring (CPU, Memory, I/O).
- Alert ranking and deduplication.
- Triggering auditory/haptic alerts via the Bezel IPC.
