import { writable } from 'svelte/store';

interface WSMessage {
	type: string;
	payload: any;
}

function createWebSocketStore() {
	const { subscribe, set, update } = writable<{
		connected: boolean;
		reconnecting: boolean;
	}>({ connected: false, reconnecting: false });

	let ws: WebSocket | null = null;
	let reconnectTimeout: ReturnType<typeof setTimeout>;
	const listeners = new Map<string, Set<(data: any) => void>>();

	function connect(url: string, token?: string) {
		if (ws) return;

		const wsUrl = token ? `${url}?token=${token}` : url;
		ws = new WebSocket(wsUrl);

		ws.onopen = () => {
			console.log('WebSocket connected');
			set({ connected: true, reconnecting: false });
		};

		ws.onmessage = (event) => {
			try {
				const message: WSMessage = JSON.parse(event.data);
				const typeListeners = listeners.get(message.type);
				if (typeListeners) {
					typeListeners.forEach(listener => listener(message.payload));
				}
			} catch (error) {
				console.error('Failed to parse WebSocket message:', error);
			}
		};

		ws.onerror = (error) => {
			console.error('WebSocket error:', error);
		};

		ws.onclose = () => {
			console.log('WebSocket disconnected');
			set({ connected: false, reconnecting: true });
			ws = null;

			// Attempt reconnection after 5 seconds
			reconnectTimeout = setTimeout(() => {
				console.log('Attempting WebSocket reconnection...');
				connect(url, token);
			}, 5000);
		};
	}

	function disconnect() {
		if (reconnectTimeout) clearTimeout(reconnectTimeout);
		if (ws) {
			ws.close();
			ws = null;
		}
		set({ connected: false, reconnecting: false });
	}

	function on(type: string, callback: (data: any) => void) {
		if (!listeners.has(type)) {
			listeners.set(type, new Set());
		}
		listeners.get(type)!.add(callback);

		// Return unsubscribe function
		return () => {
			listeners.get(type)?.delete(callback);
		};
	}

	function send(type: string, payload: any) {
		if (ws && ws.readyState === WebSocket.OPEN) {
			ws.send(JSON.stringify({ type, payload }));
		} else {
			console.warn('WebSocket not connected, cannot send message');
		}
	}

	return {
		subscribe,
		connect,
		disconnect,
		on,
		send
	};
}

export const websocket = createWebSocketStore();
