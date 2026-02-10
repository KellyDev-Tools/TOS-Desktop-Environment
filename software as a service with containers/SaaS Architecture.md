# SaaS Architecture: TOS-as-a-Service

This document outlines the high-level architecture for delivering the Tactical Operating System (TOS) as a Software-as-a-Service (SaaS) using containerized environments.

---

## 1. The Cloud Hybrid Model

TOS is uniquely suited for SaaS delivery because its UI is already web-based (HTML/CSS/JS). The architecture splits the system into two distinct layers:

### A. The Slim Client (User's Browser)
*   **Role**: Renders the LCARS interface and handles user gestures/keyboard input.
*   **Technology**: Standard web browser.
*   **Optimization**: The browser only receives the UI layer and the video stream of the "Active Surface" (Level 3 apps).

### B. The Containerized Edge (Backend)
*   **Role**: Runs the Rust-based Wayland compositor, the Fish shell modules, and all standard Linux applications.
*   **Technology**: Docker/Podman containers orchestrated via Kubernetes.
*   **Graphic Stack**: Utilizing `wgpu` with headless Vulkan or GPU passthrough to handle the spatial hierarchy rendering.

---

## 2. Infrastructure Stack

| Component | Responsibility |
|-----------|----------------|
| **Orchestrator** | Kubernetes (K8s) for spawning/terminating per-user sessions. |
| **Streaming Protocol** | WebRTC for low-latency delivery of the application framebuffers. |
| **API Gateway** | Manages authentication and routes the user's web client to their specific container instance. |
| **Session Manager** | Preserves the `path` stack and `currentDepth` of a user's Sectors even if they disconnect. |

---

## 3. Scalability & Multi-Tenancy

*   **Instance Segregation**: Each user gets a dedicated, isolated container. There is no resource sharing at the OS level, ensuring maximum privacy and security.
*   **Dynamic Resource Allocation**: Containers can be spawned with varying GPU/CPU footprints based on the user's "subscription level" or the complexity of the apps they are running.
*   **Global Overview Scaling**: Level 1 (Sectors) are virtualized. A user can have many "virtual monitors" (Viewports) spanning across different browser windows or physical devices, all talking to the same backend container.

---

## 4. Why Containerization?

1.  **Isolation**: Prevents host system contamination and provides a clean "Security/Logs" sector for every session.
2.  **Portability**: Allows TOS to run on any cloud provider (AWS, GCP, Azure) or on-prem hardware with minimal configuration.
3.  **Snapshotting**: Users can "suspend" their entire desktop state. The container's filesystem and VRAM state can be snapshotted to persistent storage and resumed instantly on another device.
