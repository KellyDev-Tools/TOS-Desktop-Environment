import { render } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { tick } from 'svelte';
import PlacementExecutor from './PlacementExecutor.svelte';

// The executor watches Brain state for launch directives and
// forwards each — once per id — to the face shell's bridge.
// The store is mocked; the bridge is a stub on window.
const fixture = vi.hoisted(() => ({
    state: { placement_directive: null as unknown } as Record<string, unknown>
}));

vi.mock('$lib/stores/ipc.svelte', () => ({
    getTosState: () => fixture.state
}));

const DIRECTIVE = {
    id: 'launch-1',
    app: 'Editor',
    exec: 'editor',
    placement: 'host'
};

describe('PlacementExecutor', () => {
    let postMessage: ReturnType<typeof vi.fn>;

    beforeEach(() => {
        fixture.state = { placement_directive: null };
        postMessage = vi.fn();
        (window as unknown as { webkit: unknown }).webkit = {
            messageHandlers: { tosFace: { postMessage } }
        };
    });

    it('forwards a directive to the shell bridge as a launch message', async () => {
        fixture.state = { placement_directive: DIRECTIVE };
        render(PlacementExecutor);
        await tick();
        expect(postMessage).toHaveBeenCalledTimes(1);
        const message = postMessage.mock.calls[0][0] as string;
        expect(message.startsWith('launch:')).toBe(true);
        expect(JSON.parse(message.substring('launch:'.length))).toEqual(
            DIRECTIVE
        );
    });

    it('does nothing without a directive', async () => {
        render(PlacementExecutor);
        await tick();
        expect(postMessage).not.toHaveBeenCalled();
    });
});
