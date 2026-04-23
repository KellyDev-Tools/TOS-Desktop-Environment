/**
 * Svelte action to trap focus within an element.
 */
export function focusTrap(node: HTMLElement) {
	const focusableElements = 'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])';
	
	function getFocusable() {
		return Array.from(node.querySelectorAll(focusableElements)) as HTMLElement[];
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key !== 'Tab') return;

		const elements = getFocusable();
		if (elements.length === 0) return;

		const first = elements[0];
		const last = elements[elements.length - 1];

		if (e.shiftKey) {
			if (document.activeElement === first) {
				last.focus();
				e.preventDefault();
			}
		} else {
			if (document.activeElement === last) {
				first.focus();
				e.preventDefault();
			}
		}
	}

	// §5.7: Ensure focus starts inside the trap
	setTimeout(() => {
		const focusable = getFocusable();
		if (focusable.length > 0) {
			focusable[0].focus();
		} else {
			node.focus();
		}
	}, 0);

	node.addEventListener('keydown', handleKeydown);

	return {
		destroy() {
			node.removeEventListener('keydown', handleKeydown);
		}
	};
}
