#pragma once
#include <string>

// Definitions for the Recursive Zoom Hierarchy
// Based on "Thoughts on Spatial Navigation.md"

enum class ZoomLevel {
    LEVEL_1_ROOT,      // Overview of all sectors
    LEVEL_2_SECTOR,    // Group of related apps/tasks (e.g. Work, Media)
    LEVEL_3_FOCUS,     // Active application window
    LEVEL_3A_PICKER    // Window picker for an app with multiple windows
};

inline std::string zoomLevelToString(ZoomLevel level) {
    switch (level) {
        case ZoomLevel::LEVEL_1_ROOT: return "Level 1: Root (Overview)";
        case ZoomLevel::LEVEL_2_SECTOR: return "Level 2: Sector (Group)";
        case ZoomLevel::LEVEL_3_FOCUS: return "Level 3: Focus (App)";
        case ZoomLevel::LEVEL_3A_PICKER: return "Level 3a: Picker (Windows)";
        default: return "Unknown Level";
    }
}
