import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import PlacementChoice from './PlacementChoice.svelte';

// The component reads Brain state through the IPC store; the
// store is mocked with a mutable fixture so each case renders
// against exactly the state it names.
const fixture = vi.hoisted(() => ({
    state: { placement_pending: null as unknown } as Record<string, unknown>,
    choose: vi.fn(async (_id: string, _side: 'host' | 'server') => {})
}));

vi.mock('$lib/stores/ipc.svelte', () => ({
    getTosState: () => fixture.state,
    placementChoose: fixture.choose
}));

const PENDING_BOTH = {
    id: 'pending-1',
    command: 'editor',
    app_name: 'Editor',
    host_exec: 'editor',
    server_exec: 'editor'
};

describe('PlacementChoice', () => {
    beforeEach(() => {
        fixture.state = { placement_pending: null };
        fixture.choose.mockClear();
    });

    it('renders nothing when no launch is pending', () => {
        const { queryByTestId } = render(PlacementChoice);
        expect(queryByTestId('placement-choice')).toBeNull();
    });

    it('names the app and offers both sides when both have it', async () => {
        fixture.state = { placement_pending: PENDING_BOTH };
        const { getByTestId } = render(PlacementChoice);
        expect(getByTestId('placement-choice').textContent).toContain('Editor');
        const host = getByTestId('placement-choose-host') as HTMLButtonElement;
        const server = getByTestId('placement-choose-server') as HTMLButtonElement;
        expect(host.disabled).toBe(false);
        expect(server.disabled).toBe(false);
        // Touch-first: choice targets meet the 48px class of
        // targets the component styles promise (min-height in
        // its stylesheet); behaviourally, both are real buttons.
        await fireEvent.click(server);
        expect(fixture.choose).toHaveBeenCalledWith('pending-1', 'server');
        await fireEvent.click(host);
        expect(fixture.choose).toHaveBeenCalledWith('pending-1', 'host');
    });

    it('disables the side that does not have the app', () => {
        fixture.state = {
            placement_pending: { ...PENDING_BOTH, host_exec: null }
        };
        const { getByTestId } = render(PlacementChoice);
        expect(
            (getByTestId('placement-choose-host') as HTMLButtonElement).disabled
        ).toBe(true);
        expect(
            (getByTestId('placement-choose-server') as HTMLButtonElement)
                .disabled
        ).toBe(false);
    });
});
