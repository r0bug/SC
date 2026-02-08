import { writable } from 'svelte/store';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface Toast {
	id: string;
	type: ToastType;
	message: string;
	title?: string;
	duration?: number;
	action?: {
		label: string;
		callback: () => void;
	};
}

function createToastStore() {
	const { subscribe, update } = writable<Toast[]>([]);

	let idCounter = 0;

	function addToast(toast: Omit<Toast, 'id'>) {
		const id = `toast-${++idCounter}`;
		const duration = toast.duration ?? 5000;

		update(toasts => [...toasts, { ...toast, id }]);

		if (duration > 0) {
			setTimeout(() => {
				removeToast(id);
			}, duration);
		}

		return id;
	}

	function removeToast(id: string) {
		update(toasts => toasts.filter(t => t.id !== id));
	}

	return {
		subscribe,
		success: (message: string, title?: string, options?: Partial<Toast>) =>
			addToast({ type: 'success', message, title, ...options }),
		error: (message: string, title?: string, options?: Partial<Toast>) =>
			addToast({ type: 'error', message, title, duration: options?.duration ?? 7000, ...options }),
		warning: (message: string, title?: string, options?: Partial<Toast>) =>
			addToast({ type: 'warning', message, title, ...options }),
		info: (message: string, title?: string, options?: Partial<Toast>) =>
			addToast({ type: 'info', message, title, ...options }),
		dismiss: removeToast,
		remove: removeToast,
		clear: () => update(() => [])
	};
}

export const toasts = createToastStore();
export const toast = toasts;
