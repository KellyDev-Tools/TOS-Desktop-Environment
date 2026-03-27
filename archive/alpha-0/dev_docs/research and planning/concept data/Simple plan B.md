This is a visionary concept. You’re essentially describing a Spatial Dynamic Tiling Compositor with an LCARS aesthetic. Combining the infinite canvas of Prezi with the rigid, functional beauty of Star Trek’s interface could solve the "clutter" problem of traditional desktop environments.
Since you are aiming for a fully functional Linux desktop that rivals Gnome or KDE, here is a breakdown of your technical questions.
1. Best Language for the Desktop Environment
For a modern, high-performance desktop environment (DE), you need a balance of low-level hardware access and high-level UI flexibility.
C++ with Qt: This is the industry standard for DEs (like KDE Plasma). Qt has excellent support for custom graphics, touch gestures, and "fluid" UI elements via QML.
Rust: If you want modern memory safety and high performance. Many new Wayland compositors (like Smithay) are being built in Rust.
Python (for prototyping only): Great for logic, but likely too slow for the "infinite zoom" rendering you're describing.
2. Wayland vs. Others
Wayland is the only logical choice. X11 is legacy and struggles with complex scaling, touch gestures, and the "smooth" transitions you’re envisioning. Wayland allows the compositor to have total control over how windows are rendered, which is essential for your "zoom into the icon" transition.
3. Existing LCARS Libraries
While there aren't many "plug-and-play" desktop libraries, these resources can accelerate your development:
LCARS SDK / Frameworks: There are various web-based LCARS frameworks (HTML/CSS/JS) that can be wrapped in an interface, but for a native DE, you’ll likely be building custom QML (Qt Modeling Language) components.
Libvast: A library sometimes used for LCARS-style layouts.
Community Assets: Search GitHub for "LCARS CSS" or "LCARS Qt" to find the specific hex codes and sound font protocols used in the series.
4. Shell Support (Terminal)
You mentioned a terminal prompt always at the bottom.
Zsh: Excellent for customization and auto-suggestions, which fits your "touch a command to transfer to prompt" idea.
Fish (Friendly Interactive Shell): Actually might be better for an LCARS feel. It has "web-based configuration" and world-class auto-completions out of the box.
Comparison: While Zsh is the power-user standard, Fish is more "intelligent" by default, which aligns with the Star Trek "Computer, do X" philosophy.
5. Aspects Needing More Definition
To turn this from a "dream" into a technical roadmap, consider these missing pieces:
Window Management Logic: If the canvas is infinite, how do you handle "focus"? If I zoom out, does the application pause or keep running?
The "Spatial" Index: If you have 50 windows open on an infinite plane, you need a "Mini-map" or a way to jump-to-coordinates so the user doesn't get lost in the void.
Hardware Acceleration: Zooming into a web browser until the icon becomes the app requires heavy GPU interpolation. You'll need to define how the "LOD" (Level of Detail) scales as you zoom.
Legacy App Scaling: How does a standard, non-LCARS app (like LibreOffice) look when placed in this environment? Does it get a "frame" or a "skin"?
Comparison of Backend Tech