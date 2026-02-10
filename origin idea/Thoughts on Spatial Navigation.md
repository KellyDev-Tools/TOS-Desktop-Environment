# Thoughts on Spatial Navigation (Refined)

The goal is to provide the **aesthetic and feel** of a spatial environment without the complexity of a 3D canvas. We use a **Recursive Zoom Hierarchy** to achieve this.

## The Illusion of Space
We don't need a true infinite canvas. Instead, we use "Zoom In" and "Zoom Out" transitions to move between structural layers, creating a sense of physical depth.

### Transition Logic: Morphing Spatial Frames
The transition between organizational levels and interactive content is achieved through visual morphing:
1.  **Level 2 (Sector) $\rightarrow$ Level 3 (Focus)**: 
    - The user clicks an App button in the launcher.
    - **Visual Morph**: The button's outer LCARS border fluidly scales and transforms into the window's **SSD (Server-Side Decoration) frame**.
    - The button's inner thumbnail expands to become the live application surface.
    - This creates a seamless transition where the navigation container *becomes* the application interface.
2.  **Level 3 (Window Picker)**:
    - If multiple windows are open for an app, zooming out from Level 3 does not go straight to Level 2.
    - Instead, it shows a grid of window thumbnails (Level 3a).
    - The user selects a target window, which then expands to fill the focus view.

## Window Management & Tiling
Window management is handled via the **Split** action.
- Triggering a split from Level 3 divides the current viewport into two panes (Sectors).
- One pane retains the current app focus.
- The new pane/sector reverts to a higher-level view (Level 1 or Level 2) to allow the user to select the content for the second pane.

## Conclusion

This model moves away from "panning" and "arbitrary dragging" for navigation in favor of a strictly vertical navigation stack. This creates a predictable mental model, fits the LCARS "tactical console" aesthetic, and allows for clean state management across multiple viewports. Dragging remains a valid interaction for high-level resource management (e.g., moving apps between Sectors at Level 1).