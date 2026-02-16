# Security & Persistence in the Cloud

Deploying a desktop environment as a SaaS introduces unique security challenges, particularly regarding data isolation and remote command execution.

---

## 1. Zero-Trust Interaction

Because TOS encourages a **Persistent Unified Prompt**, the connection between the user's browser and the backend must be hardened.

*   **Encrypted Tunnels**: All traffic (Video, Commands, Input) is wrapped in TLS 1.3.
*   **Prompt Sanitization**: The **TOS Shell API** includes a validation layer. Commands staged in the browser are not executed until they pass a security check on the containerized server.
*   **Wait-to-Execute as Security**: The mandatory staging of commands (the "Wait-to-Execute" pattern) acts as a natural buffer against injection attacks or accidental destructive actions.

---

## 2. Multi-Tenant Isolation

*   **Namespace Isolation**: Using Linux namespaces (User, Network, Mount) within Docker to ensure that one user's container can never see another user's processes or files.
*   **Resource Quotas**: Hard limits on CPU, VRAM, and Disk I/O prevent "noisy neighbor" scenarios where one user's heavy application (e.g., a compile or render) slows down the entire cluster.
*   **Stateless Host**: The host servers running the containers store no user data. All persistence is handled via encrypted network-attached storage.

---

## 3. Persistent Session Logic

TOS persistence allows a user to "walk away" and resume their exact spatial state.

*   **Hibernation**: When a user closes their browser, the container enters a `SIGSTOP` state. After a configurable timeout, the state is serialized to disk (checkpointed) and the container is killed to free resources.
*   **State Reconstruction**: Upon re-login, the orchestrator spawns a new container and injects the saved `path` stack. The user sees their Sectors (Level 1) and Active Apps (Level 3) exactly as they left them.

---

## 4. Federated Identity (Single Sign-On)

*   **LCARS Login**: The initial "Level 0" view is an LCARS-styled authentication portal.
*   **OIDC Integration**: Support for logging in via standard providers (Google, GitHub, Microsoft) or private enterprise OIDC servers.
*   **Biometric Relay**: Relaying device-side biometrics (FaceID/Passkeys) from the client browser to the backend for passwordless "Authorization Approved" transitions.
