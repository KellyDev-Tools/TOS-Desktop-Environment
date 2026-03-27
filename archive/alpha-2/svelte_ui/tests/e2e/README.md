# TOS E2E Test Suite

Comprehensive end-to-end testing suite for the TOS (Terminal Operating System) application using Playwright.

## Overview

This test suite covers all user stories from the TOS roadmap, organized into three phases:

- **Phase 1 — Core Features**: Brain connection, command execution, file browser, navigation, search
- **Phase 2 — Advanced Features**: Shell pipelines, settings UI, desktop integration, viewport generation
- **Phase 3 — Integration Features**: Threaded tests, orchestration, compositor mapping, buffer view, navigator logic

## Test Structure

```
e2e/
├── index.spec.ts           # Main entry point and exports
├── README.md               # This file
├── core/
│   ├── brain.spec.ts       # Brain connection & sector navigation
│   ├── command.spec.ts     # Command hub input & execution
│   ├── visual.spec.ts      # Visual file browser & navigation
│   ├── navigation.spec.ts  # Visual navigation & intelligent zoom
│   └── search.spec.ts      # Search integration
├── advanced/
│   ├── shell.spec.ts       # Shell pipeline execution
│   ├── settings.spec.ts    # Settings UI and audio controls
│   ├── desktop.spec.ts     # Desktop environment integration
│   ├── viewport.spec.ts    # Viewport generation
│   └── edge.spec.ts        # Edge cases and error handling
├── integration/
│   ├── threads.spec.ts     # Threaded integration tests
│   ├── orchestration.spec.ts
│   ├── compositor.spec.ts  # Compositor mapping
│   ├── buffer.spec.ts      # Buffer view
│   ├── navigator.spec.ts   # Navigator logic
│   └── integrity.spec.ts   # Comprehensive integrity checks
└── roadmap.spec.ts         # All tests organized by user stories
```

## Setup

### Prerequisites

- Node.js 18+
- Playwright installed
- TOS application running locally

### Installation

```bash
# Install Playwright browsers
npx playwright install

# Install system dependencies (if needed)
npx playwright install-deps
```

## Running Tests

### Run All Tests

```bash
npx playwright test
```

### Run Specific Test File

```bash
npx playwright test core/brain.spec.ts
```

### Run Specific Test

```bash
npx playwright test --grep "should connect to brain"
```

### Run with Headless Mode

```bash
npx playwright test --headless
```

### Run with Debug Mode

```bash
npx playwright test --debug
```

### Run with Trace

```bash
npx playwright test --trace on
```

## Test Configuration

### Playwright Configuration

The test suite uses the following configuration:

```typescript
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
    testDir: './e2e',
    fullyParallel: true,
    forbidOnly: !!process.env.CI,
    retries: process.env.CI ? 2 : 0,
    workers: process.env.CI ? 1 : undefined,
    reporter: [
        ['html'],
        ['json', { outputFile: 'test-results.json' }],
        ['junit', { outputFile: 'test-results-junit.xml' }],
    ],
    use: {
        trace: 'on-first-retry',
        screenshot: 'only-on-failure',
        video: 'retain-on-failure',
    },
    projects: [
        {
            name: 'chromium',
            use: { ...devices['Desktop Chrome'] },
        },
        {
            name: 'firefox',
            use: { ...devices['Desktop Firefox'] },
        },
        {
            name: 'webkit',
            use: { ...devices['Desktop Safari'] },
        },
    ],
});
```

## Test Selectors

### Common Selectors

```typescript
// Command input
const cmdInput = page.locator('input#cmd-input');

// Terminal output
const termLine = page.locator('.term-line');

// Brain button
const brainBtn = page.locator('.pill-btn', { hasText: 'BRAIN' });

// Visual button
const visualBtn = page.locator('.pill-btn', { hasText: 'VISUAL' });

// Search button
const searchBtn = page.locator('.pill-btn', { hasText: 'SEARCH' });

// Sector tile
const sectorTile = page.locator('.sector-tile');

// File entry
const fileEntry = page.locator('.file-entry');

// Directory entry
const dirEntry = page.locator('.dir-entry');

// File browser
const fileBrowser = page.locator('.file-browser');

// Navigation panel
const navPanel = page.locator('.nav-panel');

// Zoom controls
const zoomBtn = page.locator('.zoom-btn');

// Settings button
const settingsBtn = page.locator('button:has-text("SETTINGS")');
```

## Test Writing Guidelines

### Best Practices

1. **Use descriptive test names**: Tests should clearly describe what they verify
2. **Keep tests independent**: Each test should not depend on other tests
3. **Use consistent selectors**: Follow the naming conventions in the codebase
4. **Include error handling**: Handle potential async operations gracefully
5. **Add comments**: Explain complex test logic and edge cases

### Test Structure

```typescript
test.describe('Feature Name', () => {
    test('should perform expected behavior', async ({ page }) => {
        // Setup
        // Actions
        // Assertions
    });

    test('should handle error case', async ({ page }) => {
        // Setup
        // Actions
        // Assertions
    });
});
```

### Helper Functions

```typescript
// Boot to command hub
async function bootToCommandHub(page: Page) {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);
}

// Click brain button
async function clickBrainButton(page: Page) {
    const brainBtn = page.locator('.pill-btn', { hasText: 'BRAIN' });
    await brainBtn.click();
    await page.waitForTimeout(500);
}

// Type command and execute
async function typeCommand(page: Page, command: string) {
    const cmdInput = page.locator('input#cmd-input');
    await cmdInput.fill(command);
    await cmdInput.press('Enter');
    await page.waitForTimeout(500);
}
```

## Debugging

### Manual Debugging

```bash
# Launch browser manually
npx playwright test --debug
```

### Trace Viewer

```bash
# Generate traces
npx playwright test --trace on

# View traces
npx playwright show-trace trace.zip
```

### Screenshot on Failure

```bash
# Generate screenshots
npx playwright test --screenshot on
```

## CI/CD Integration

### GitHub Actions

```yaml
name: E2E Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
      - run: npm ci
      - run: npx playwright install --with-deps
      - run: npx playwright test
      - uses: actions/upload-artifact@v3
        if: failure()
        with:
          name: playwright-report
          path: playwright-report/
```

## Reporting

### HTML Report

```bash
npx playwright show-report
```

### JSON Report

```bash
npx playwright show-report --reporter json
```

### Coverage Report

```bash
npx playwright test --reporter json --output-json coverage.json
```

## Troubleshooting

### Common Issues

1. **Tests fail to connect to browser**
   - Run `npx playwright install` to install browsers
   - Check browser paths in Playwright config

2. **Tests timeout**
   - Increase timeout in test configuration
   - Add `page.waitForTimeout()` for async operations

3. **Selectors not found**
   - Use `page.locator()` instead of `page.$()`
   - Add `await` before locator operations

4. **Tests flaky**
   - Add proper waits between actions
   - Use `retry` option in configuration

## Contributing

1. Create a new test file in the appropriate directory
2. Write descriptive test names
3. Include error handling
4. Add comments for complex logic
5. Run tests locally before committing

## License

MIT
