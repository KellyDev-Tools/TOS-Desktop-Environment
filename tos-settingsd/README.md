# TOS Settings Daemon (`tos-settingsd`)

The authoritative store for system-wide configuration, user preferences, and module settings.

## Design Drivers
- **Architecture Spec §26**: Defines the settings data model and IPC contracts (`get_setting`, `set_setting`).
- **Features Spec §2.2.3**: Establishes the boundary between permanent preferences and transient session state.

## Responsibilities
- Persistent storage of `tos.toml` configurations.
- Real-time change notifications for all system components.
- Hierarchical value resolution (System -> User -> Module).
