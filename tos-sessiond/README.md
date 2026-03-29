# TOS Session Service (`tos-sessiond`)

The "Workspace Memory" of TOS, ensuring the system state is preserved across reboots.

## Design Drivers
- **Features Spec §2**: Full specification for "Session Persistence & Workspace Memory".
- **Architecture Spec §4**: Defines the session service as the guardian of "Live State" and "Named Sessions".

## Responsibilities
- Atomic, debounced writes of the `_live.tos-session` snapshot.
- Management of named session files for sector-scoped portability.
- Restoring terminal scrollback histories and AI chat context.
