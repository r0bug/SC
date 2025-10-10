import { writable } from 'svelte/store';

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface Toast {
	id: string;
	type: ToastType;
	message: string;
	duration?: number;
}

function createToastStore() {
	const { subscribe, update } = writable<Toast[]>([]);

	return {
		subscribe,
		show: (message: string, type: ToastType = 'info', duration = 5000) => {
			const id = Math.random().toString(36).substring(2, 9);
			const toast: Toast = { id, type, message, duration };

			update(toasts => [...toasts, toast]);

			if (duration > 0) {
				setTimeout(() => {
					update(toasts => toasts.filter(t => t.id !== id));
				}, duration);
			}

			return id;
		},
		success: (message: string, duration = 5000) => {
			const id = Math.random().toString(36).substring(2, 9);
			const t: Toast = { id, type: 'success', message, duration };
			update(toasts => [...toasts, t]);
			if (duration > 0) {
				setTimeout(() => update(toasts => toasts.filter(toast => toast.id !== id)), duration);
			}
			return id;
		},
		error: (message: string, duration = 7000) => {
			const id = Math.random().toString(36).substring(2, 9);
			const t: Toast = { id, type: 'error', message, duration };
			update(toasts => [...toasts, t]);
			if (duration > 0) {
				setTimeout(() => update(toasts => toasts.filter(toast => toast.id !== id)), duration);
			}
			return id;
		},
		warning: (message: string, duration = 6000) => {
			const id = Math.random().toString(36).substring(2, 9);
			const t: Toast = { id, type: 'warning', message, duration };
			update(toasts => [...toasts, t]);
			if (duration > 0) {
				setTimeout(() => update(toasts => toasts.filter(toast => toast.id !== id)), duration);
			}
			return id;
		},
		info: (message: string, duration = 5000) => {
			const id = Math.random().toString(36).substring(2, 9);
			const t: Toast = { id, type: 'info', message, duration };
			update(toasts => [...toasts, t]);
			if (duration > 0) {
				setTimeout(() => update(toasts => toasts.filter(toast => toast.id !== id)), duration);
			}
			return id;
		},
		dismiss: (id: string) => {
			update(toasts => toasts.filter(t => t.id !== id));
		},
		clear: () => {
			update(() => []);
		}
	};
}

export const toast = createToastStore();
