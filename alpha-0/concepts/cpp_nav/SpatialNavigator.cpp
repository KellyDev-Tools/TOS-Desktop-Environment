#include "SpatialNavigator.h"
#include <iostream>

SpatialNavigator::SpatialNavigator()
    : m_currentLevel(ZoomLevel::LEVEL_1_ROOT)
    , m_activeSectorIndex(-1)
    , m_activeAppIndex(-1)
    , m_activeWindowIndex(-1)
{
    // Initial state setup
}

void SpatialNavigator::zoomIn(int targetIndex) {
    if (m_currentLevel == ZoomLevel::LEVEL_1_ROOT) {
        // From Root to Sector
        m_activeSectorIndex = targetIndex;
        m_currentLevel = ZoomLevel::LEVEL_2_SECTOR;
        std::cout << "[Zoom In] Entering Sector " << targetIndex << std::endl;
    } else if (m_currentLevel == ZoomLevel::LEVEL_2_SECTOR) {
        // From Sector to App (Focus)
        m_activeAppIndex = targetIndex;
        m_currentLevel = ZoomLevel::LEVEL_3_FOCUS;
        std::cout << "[Zoom In] Focusing on App " << targetIndex << " (Morphing SSD Frame...)" << std::endl;
    } else if (m_currentLevel == ZoomLevel::LEVEL_3A_PICKER) {
        // From Picker to Specific Window
        m_activeWindowIndex = targetIndex;
        m_currentLevel = ZoomLevel::LEVEL_3_FOCUS;
        std::cout << "[Zoom In] Selected Window " << targetIndex << " from Picker." << std::endl;
    } else {
        std::cout << "[Navigate] Already at deepest level (Level 3 Focus)." << std::endl;
    }
    printStatus();
}

void SpatialNavigator::zoomOut() {
    if (m_currentLevel == ZoomLevel::LEVEL_3_FOCUS) {
        // Check if multiple windows exist (simulated condition)
        bool hasMultipleWindows = (m_activeAppIndex % 2 == 0); // Mock condition: even index apps have multiple windows

        if (hasMultipleWindows) {
            m_currentLevel = ZoomLevel::LEVEL_3A_PICKER;
            std::cout << "[Zoom Out] Multiple windows detected -> Entering Window Picker (Level 3a)." << std::endl;
        } else {
            m_currentLevel = ZoomLevel::LEVEL_2_SECTOR;
            m_activeAppIndex = -1; // Reset app selection
            std::cout << "[Zoom Out] Returning to Sector View (Level 2)." << std::endl;
        }
    } else if (m_currentLevel == ZoomLevel::LEVEL_3A_PICKER) {
        // From Picker back to Sector list
        m_currentLevel = ZoomLevel::LEVEL_2_SECTOR;
        m_activeAppIndex = -1;
        std::cout << "[Zoom Out] Returning to Sector View (Level 2) from Picker." << std::endl;
    } else if (m_currentLevel == ZoomLevel::LEVEL_2_SECTOR) {
        // From Sector to Root
        m_currentLevel = ZoomLevel::LEVEL_1_ROOT;
        m_activeSectorIndex = -1;
        std::cout << "[Zoom Out] Returning to Root Overview (Level 1)." << std::endl;
    } else {
        std::cout << "[Navigate] Already at top level (Level 1 Root)." << std::endl;
    }
    printStatus();
}

void SpatialNavigator::splitView() {
    if (m_currentLevel == ZoomLevel::LEVEL_3_FOCUS) {
        // Implementation of split logic: One pane stays focused, other reverts
        std::cout << "[Split] Splitting Viewport..." << std::endl;
        std::cout << "  -> Left Pane: Retains App Focus (Level 3)" << std::endl;
        std::cout << "  -> Right Pane: Reverts to Level 2 (Sector Selection)" << std::endl;
        // In a real implementation, this would create a new Viewport object
    } else {
        std::cout << "[Split] Can only split from a focused app (Level 3)." << std::endl;
    }
}

ZoomLevel SpatialNavigator::getCurrentLevel() const {
    return m_currentLevel;
}

void SpatialNavigator::printStatus() const {
    std::cout << "Current State: " << (int)m_currentLevel << std::endl;
}
