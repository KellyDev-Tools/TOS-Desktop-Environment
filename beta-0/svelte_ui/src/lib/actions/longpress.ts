export function longpress(node: HTMLElement, options: { threshold?: number; onLongPress?: (e: Event) => void } = {}) {
    let threshold = options.threshold || 600;
    let timer: number | null = null;
    let didTrigger = false;

    const handleDown = (event: Event) => {
        didTrigger = false;
        timer = window.setTimeout(() => {
            didTrigger = true;
            if (options.onLongPress) {
                options.onLongPress(event);
            } else {
                node.dispatchEvent(new CustomEvent('longpress', { detail: event }));
            }
        }, threshold);
    };

    const handleUp = (event: Event) => {
        if (timer !== null) {
            clearTimeout(timer);
            timer = null;
        }
        if (didTrigger) {
            event.preventDefault();
        }
    };

    node.addEventListener('mousedown', handleDown);
    node.addEventListener('mouseup', handleUp);
    node.addEventListener('mouseleave', handleUp);
    node.addEventListener('touchstart', handleDown);
    node.addEventListener('touchend', handleUp);
    node.addEventListener('touchcancel', handleUp);

    return {
        update(newOptions: { threshold?: number; onLongPress?: (e: Event) => void }) {
            options = { ...options, ...newOptions };
            if (options.threshold) threshold = options.threshold;
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
