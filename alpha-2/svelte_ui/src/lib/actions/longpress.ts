export function longpress(node: HTMLElement, threshold = 600) {
    let timer: number | null = null;
    let didTrigger = false;

    const handleDown = (event: Event) => {
        didTrigger = false;
        timer = window.setTimeout(() => {
            didTrigger = true;
            node.dispatchEvent(new CustomEvent('longpress', { detail: event }));
        }, threshold);
    };

    const handleUp = (event: Event) => {
        if (timer !== null) {
            clearTimeout(timer);
            timer = null;
        }
        if (didTrigger) {
            // Prevent default click actions if the long press triggered
            event.preventDefault();
        }
    };

    // Attach listeners
    node.addEventListener('mousedown', handleDown);
    node.addEventListener('mouseup', handleUp);
    node.addEventListener('mouseleave', handleUp);
    node.addEventListener('touchstart', handleDown);
    node.addEventListener('touchend', handleUp);
    node.addEventListener('touchcancel', handleUp);

    return {
        update(newThreshold: number) {
            threshold = newThreshold;
        },
        destroy() {
            node.removeEventListener('mousedown', handleDown);
            node.removeEventListener('mouseup', handleUp);
            node.removeEventListener('mouseleave', handleUp);
            node.removeEventListener('touchstart', handleDown);
            node.removeEventListener('touchend', handleUp);
            node.removeEventListener('touchcancel', handleUp);
        }
    };
}
