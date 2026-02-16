#include <iostream>
#include "SpatialNavigator.h"

int main() {
    std::cout << "==========================================" << std::endl;
    std::cout << "TOS Traditional Environment - Concept Demo" << std::endl;
    std::cout << "==========================================" << std::endl;
    
    SpatialNavigator nav;

    // 1. Initial State
    std::cout << "\n[User Action] System Startup" << std::endl;
    nav.printStatus();

    // 2. Zoom into a Sector
    std::cout << "\n[User Action] Click 'Work' Sector (Index 0)" << std::endl;
    nav.zoomIn(0); 

    // 3. Zoom into an App
    std::cout << "\n[User Action] Launch 'Terminal' (Index 1)" << std::endl;
    nav.zoomIn(1); // Assume index 1 is terminal

    // 4. Split View
    std::cout << "\n[User Action] Split Screen (Trigger tiling)" << std::endl;
    nav.splitView();

    // 5. Back Navigation
    std::cout << "\n[User Action] Zoom Out (Gesture/Key)" << std::endl;
    nav.zoomOut();

    return 0;
}
