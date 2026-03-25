// E2E Test Suite Index
// ====================
// This file serves as the entry point for the comprehensive E2E test suite.
// It exports all test files and provides a structured overview of the test coverage.

import {
    test as base,
    expect,
    describe,
    beforeAll,
    afterAll,
} from '@playwright/test';

// Export base test function for use in other test files
export const test = base;
export { expect, describe, beforeAll, afterAll };

// Import all test suites
import './core/brain.spec.ts';
import './core/command.spec.ts';
import './core/visual.spec.ts';
import './core/navigation.spec.ts';
import './core/search.spec.ts';
import './advanced/shell.spec.ts';
import './advanced/settings.spec.ts';
import './advanced/desktop.spec.ts';
import './advanced/viewport.spec.ts';
import './advanced/edge.spec.ts';
import './integration/threads.spec.ts';
import './integration/orchestration.spec.ts';
import './integration/compositor.spec.ts';
import './integration/buffer.spec.ts';
import './integration/navigator.spec.ts';
import './integration/integrity.spec.ts';
import './roadmap.spec.ts';

// Test Suite Statistics
// ======================
// Total test files: 17
// Total test suites: 17
// Total individual tests: 100+
// Coverage: All user stories from the roadmap

// Test Categories
// ===============
// Core Features (Phase 1):
//   - Brain Connection & Sector Navigation
//   - Command Hub Input & Execution
//   - Visual File Browser & Navigation
//   - Visual Navigation & Intelligent Zoom
//   - Search Integration
//
// Advanced Features (Phase 2):
//   - Shell Pipeline Execution
//   - Settings UI and Audio Controls
//   - Desktop Environment Integration
//   - Viewport Generation
//   - Edge Cases and Error Handling
//
// Integration Features (Phase 3):
//   - Threaded Integration Tests
//   - Orchestration and Audio
//   - Compositor Mapping
//   - Buffer View
//   - Navigator Logic
//   - Comprehensive Integrity Checks

// Usage
// =====
// Run all tests:
//   npx playwright test
//
// Run specific test file:
//   npx playwright test core/brain.spec.ts
//
// Run specific test:
//   npx playwright test --grep "should connect to brain"
//
// Generate report:
//   npx playwright show-report
