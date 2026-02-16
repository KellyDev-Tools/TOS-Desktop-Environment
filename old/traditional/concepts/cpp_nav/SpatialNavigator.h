#pragma once

#include "ZoomDefinitions.h"
#include <vector>
#include <iostream>

class SpatialNavigator {
public:
    SpatialNavigator();

    // Simulate zooming into a specific target (sector or app)
    void zoomIn(int targetIndex);

    // Zoom out to the parent level
    void zoomOut();

    // Simulate splitting the current view (Level 3 -> Level 2/1 hybrid)
    void splitView();

    // Get current state
    ZoomLevel getCurrentLevel() const;
    void printStatus() const;

private:
    ZoomLevel m_currentLevel;
    int m_activeSectorIndex;
    int m_activeAppIndex;
    int m_activeWindowIndex; // For Level 3a
};
