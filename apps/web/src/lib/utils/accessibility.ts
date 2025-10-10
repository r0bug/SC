/**
 * Traps focus within a container element for modals/dialogs
 */
export function trapFocus(container: HTMLElement): () => void {
	const focusableSelectors = [
		'a[href]',
		'button:not([disabled])',
		'textarea:not([disabled])',
		'input:not([disabled])',
		'select:not([disabled])',
		'[tabindex]:not([tabindex="-1"])'
	].join(', ');

	const focusableElements = Array.from(
		container.querySelectorAll(focusableSelectors)
	) as HTMLElement[];

	const firstFocusable = focusableElements[0];
	const lastFocusable = focusableElements[focusableElements.length - 1];

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key !== 'Tab') return;

		if (e.shiftKey) {
			if (document.activeElement === firstFocusable) {
				e.preventDefault();
				lastFocusable?.focus();
			}
		} else {
			if (document.activeElement === lastFocusable) {
				e.preventDefault();
				firstFocusable?.focus();
			}
		}
	}

	container.addEventListener('keydown', handleKeyDown);
	firstFocusable?.focus();

	return () => {
		container.removeEventListener('keydown', handleKeyDown);
	};
}

/**
 * Announces a message to screen readers
 */
export function announce(message: string, priority: 'polite' | 'assertive' = 'polite'): void {
	const announcer = document.createElement('div');
	announcer.setAttribute('role', 'status');
	announcer.setAttribute('aria-live', priority);
	announcer.setAttribute('aria-atomic', 'true');
	announcer.className = 'sr-only';
	announcer.textContent = message;

	document.body.appendChild(announcer);

	setTimeout(() => {
		document.body.removeChild(announcer);
	}, 1000);
}

/**
 * Generates a unique ID for accessibility labeling
 */
let idCounter = 0;
export function generateId(prefix: string = 'a11y'): string {
	return `${prefix}-${++idCounter}-${Math.random().toString(36).substring(2, 9)}`;
}

/**
 * Creates a live region for dynamic content updates
 */
export function createLiveRegion(
	priority: 'polite' | 'assertive' = 'polite'
): HTMLElement {
	const region = document.createElement('div');
	region.setAttribute('role', 'status');
	region.setAttribute('aria-live', priority);
	region.setAttribute('aria-atomic', 'true');
	region.className = 'sr-only';
	document.body.appendChild(region);
	return region;
}

/**
 * Screen reader only CSS class helper
 */
export const srOnlyStyles = `
	position: absolute;
	width: 1px;
	height: 1px;
	padding: 0;
	margin: -1px;
	overflow: hidden;
	clip: rect(0, 0, 0, 0);
	white-space: nowrap;
	border-width: 0;
`;
