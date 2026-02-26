This is a significant conceptual leap. You are moving from a standard "WIMP" (Windows, Icons, Menus, Pointer) paradigm to a Zoomable User Interface (ZUI) where "depth" replaces "overlays."
In a traditional desktop, a right-click creates a new layer (the pop-up menu) that obscures your work. In your LCARS SDE, a long-press should reveal detail by zooming into the object's specific functions, maintaining the "Infinite Canvas" immersion.
Here is an expansion on the "Context Actions" and other missing desktop features, designed specifically for power users on a touchscreen.
1. The "Orbital Context" System (Replacing Right-Click)
Instead of a dropdown list, we will use Semantic Zooming. When you "Long Press" (or "Deep Press") an object, the system interprets this as a request to "inspect the object's controls."
How it works:
 * The Trigger: You long-press a file icon (e.g., document.txt).
 * The Response: The camera zooms in slightly (1.5x), and the file icon "explodes" or is surrounded by an LCARS Orbital Interface—curved, touch-friendly bars that encircle the object.
 * The Layout:
   * North Arc (Execute): Open, Open With...
   * East Arc (Modify): Rename, Edit Tags.
   * South Arc (Destructive): Delete, Shred.
   * West Arc (Transport): Move, Copy (activates "Payload Mode"—see below).
The Nushell Integration (The "Touch-to-Syntax" Bridge):
This is where the power user feature shines. Tapping a graphical arc doesn't just magically perform the action; it constructs the command in the always-visible terminal at the bottom.
 * Example (Renaming):
   * User Long-Presses project.json.
   * User taps the "Rename" arc.
   * Action: The Command Bar auto-types mv project.json | and focuses the cursor.
   * Completion: The user types the new name (or speaks it) and hits Enter.
     Why this is better: It teaches the user the CLI syntax while offering the speed of GUI. It allows for complex piped commands (power user capability) initiated by a simple touch.
2. "Payload Mode" (Copy/Paste & Drag-and-Drop)
Standard desktops use a hidden clipboard. A Spatial Desktop should make the clipboard visible.
 * The Feature: When you select "Copy" or "Move" from the Orbital Context:
   * The file is not just "copied to memory."
   * It becomes a floating widget (a "Payload") attached to the edge of your Viewport (like a docked shuttlecraft).
 * The Action: You zoom out, pan across your infinite canvas to a new directory/cluster, and drag the Payload from the viewport edge onto the new area.
 * Terminal Output: As you drop it, the terminal executes cp /source/path/file /destination/path/.
3. Spatial Grouping (The "Cluster" Concept)
You mentioned selecting multiple windows to see them together. In a ZUI, we don't "minimize" windows; we simply zoom away from them. To organize them, we need Clusters.
 * Logic: A "Cluster" is a named region of the coordinate system (e.g., x: 5000, y: 2000).
 * Interaction: You can lasso-select three web browsers and a terminal window.
 * Command: "Create Cluster: Web Dev".
 * Visual: The system draws an LCARS frame around these four windows.
 * Navigation: The Command Bar now has a "Jump to" button. Tapping "Web Dev" triggers a smooth camera flight (pan + zoom) directly to that coordinate cluster.
4. System Status & Notifications (The "Subspace Layer")
Standard desktops have a "System Tray" (WiFi, Battery, Time) usually stuck in a corner. In an immersive ZUI, these shouldn't clutter the view.
 * The "Subspace" Layer: These elements exist on the HUD Layer (The Layer Shell mentioned in your Unified Plan). They do not zoom with the canvas; they stick to the glass of the screen.
 * Notifications: Instead of toast popups covering your work:
   * An LCARS "Alert Color" pulses on the edge of the screen (off-screen indicator).
   * Action: You swipe in from that edge to drag the notification onto your canvas. It becomes a temporary object (a "Comm Badge") you can read, interact with, or dismiss (delete).
5. Global Search (The "Computer" Query)
A power user needs to find things instantly on an infinite map.
 * The Interaction: A three-finger tap anywhere on the screen (or saying "Computer").
 * The Result: The background dims. The Command Bar highlights.
 * Behavior:
   * User types/says: "Where is the budget file?"
   * System runs fd budget (using the nu data engine).
   * The Reveal: The camera zooms out to show the entire canvas, highlighting the matches with bright vectors, then zooms in automatically to the "Best Match."
Summary of Missing Features to Add to Roadmap
| Feature | Standard Desktop Metaphor | LCARS Spatial Metaphor | Underlying Command (Nushell) |
|---|---|---|---|
| Context Menu | Right-Click Dropdown | Orbital Zoom (Radial menu around object) | Generates mv, rm, open syntax |
| Clipboard | Invisible Memory | Payload Dock (Floating icons on viewport edge) | cp $source $dest |
| Taskbar | Minimized Icons | Cluster Bookmarks (Saved x,y,z coordinates) | view jump --tag "work" |
| Alerts | Pop-up Toasts | Edge Pulses (Swipe in to materialize) | dunst or similar wired to UI |
Recommended Next Step
To make this actionable, I can draft the "Orbital Menu Logic" for your lcars-theme (React) plugin.
Would you like me to generate the React/Typescript pseudo-code for the "Orbital Context Menu" component, showing how it triggers the JSON-RPC call to the Nushell bridge?
