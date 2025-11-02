import { writable } from 'svelte/store';

interface WSMessage {
	type: string;
	payload: any;
}

interface ConnectionState {
	connected: boolean;
	reconnecting: boolean;
	reconnectAttempts: number;
	lastError: string | null;
}

interface ReconnectConfig {
	maxReconnectAttempts: number;
	baseDelay: number;
	maxDelay: number;
	enableAutoReconnect: boolean;
}

const defaultConfig: ReconnectConfig = {
	maxReconnectAttempts: 10,
	baseDelay: 1000,
	maxDelay: 30000,
	enableAutoReconnect: true
};

function createResilientWebSocketStore(config: Partial<ReconnectConfig> = {}) {
	const finalConfig = { ...defaultConfig, ...config };

	const { subscribe, set, update } = writable<ConnectionState>({
		connected: false,
		reconnecting: false,
		reconnectAttempts: 0,
		lastError: null
	});

	let ws: WebSocket | null = null;
	let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
	let currentUrl: string = '';
	let currentToken: string | undefined;
	let intentionallyClosed = false;
	const listeners = new Map<string, Set<(data: any) => void>>();

	// Heartbeat mechanism to detect stale connections
	let heartbeatInterval: ReturnType<typeof setInterval> | null = null;
	let lastHeartbeat = Date.now();

	function startHeartbeat() {
		if (heartbeatInterval) clearInterval(heartbeatInterval);

		heartbeatInterval = setInterval(() => {
			if (ws && ws.readyState === WebSocket.OPEN) {
				// Check if we've received any message in the last 60 seconds
				if (Date.now() - lastHeartbeat > 60000) {
					console.warn('WebSocket appears stale, reconnecting...');
					reconnect();
				} else {
					// Send ping
					try {
						ws.send(JSON.stringify({ type: 'ping', payload: { timestamp: Date.now() } }));
					} catch (error) {
						console.error('Failed to send heartbeat:', error);
					}
				}
			}
		}, 30000); // Check every 30 seconds
	}

	function stopHeartbeat() {
		if (heartbeatInterval) {
			clearInterval(heartbeatInterval);
			heartbeatInterval = null;
		}
	}

	function calculateDelay(attemptNumber: number): number {
		const delay = finalConfig.baseDelay * Math.pow(2, attemptNumber);
		return Math.min(delay, finalConfig.maxDelay);
	}

	function reconnect() {
		if (!finalConfig.enableAutoReconnect || intentionallyClosed) return;

		disconnect();
		if (currentUrl) {
			connect(currentUrl, currentToken);
		}
	}

	function connect(url: string, token?: string) {
		if (ws && ws.readyState === WebSocket.CONNECTING) {
			console.log('WebSocket connection already in progress');
			return;
		}

		currentUrl = url;
		currentToken = token;
		intentionallyClosed = false;

		update(state => ({ ...state, reconnecting: true }));

		const wsUrl = token ? `${url}?token=${token}` : url;

		try {
			ws = new WebSocket(wsUrl);

			ws.onopen = () => {
				console.log('✅ WebSocket connected');
				lastHeartbeat = Date.now();
				set({
					connected: true,
					reconnecting: false,
					reconnectAttempts: 0,
					lastError: null
				});
				startHeartbeat();
			};

			ws.onmessage = (event) => {
				lastHeartbeat = Date.now();

				try {
					const message: WSMessage = JSON.parse(event.data);

					// Handle pong response
					if (message.type === 'pong') {
						return;
					}

					const typeListeners = listeners.get(message.type);
					if (typeListeners) {
						typeListeners.forEach(listener => {
							try {
								listener(message.payload);
							} catch (error) {
								console.error(`Error in WebSocket listener for ${message.type}:`, error);
							}
						});
					}
				} catch (error) {
					console.error('Failed to parse WebSocket message:', error);
				}
			};

			ws.onerror = (error) => {
				console.error('❌ WebSocket error:', error);
				update(state => ({ ...state, lastError: 'Connection error occurred' }));
			};

			ws.onclose = (event) => {
				console.log('🔌 WebSocket disconnected', event.code, event.reason);
				stopHeartbeat();
				ws = null;

				update(state => {
					const newState = {
						...state,
						connected: false,
						reconnecting: !intentionallyClosed && finalConfig.enableAutoReconnect
					};

					if (intentionallyClosed) {
						return { ...newState, reconnectAttempts: 0, lastError: null };
					}

					// Handle different close codes
					if (event.code === 1000) {
						// Normal closure
						return { ...newState, reconnecting: false, lastError: null };
					}

					// Attempt reconnection
					if (state.reconnectAttempts < finalConfig.maxReconnectAttempts) {
						const delay = calculateDelay(state.reconnectAttempts);
						console.log(`Attempting reconnection in ${delay}ms... (${state.reconnectAttempts + 1}/${finalConfig.maxReconnectAttempts})`);

						reconnectTimeout = setTimeout(() => {
							if (!intentionallyClosed && currentUrl) {
								connect(currentUrl, currentToken);
							}
						}, delay);

						return { ...newState, reconnectAttempts: state.reconnectAttempts + 1 };
					} else {
						console.error('Max reconnection attempts reached');
						return {
							...newState,
							reconnecting: false,
							lastError: 'Max reconnection attempts reached. Please refresh the page.'
						};
					}
				});
			};
		} catch (error: any) {
			console.error('Failed to create WebSocket:', error);
			set({
				connected: false,
				reconnecting: false,
				reconnectAttempts: 0,
				lastError: error.message || 'Failed to create WebSocket'
			});
		}
	}

	function disconnect() {
		intentionallyClosed = true;

		if (reconnectTimeout) {
			clearTimeout(reconnectTimeout);
			reconnectTimeout = null;
		}

		stopHeartbeat();

		if (ws) {
			try {
				ws.close(1000, 'Client disconnect');
			} catch (error) {
				console.error('Error closing WebSocket:', error);
			}
			ws = null;
		}

		set({
			connected: false,
			reconnecting: false,
			reconnectAttempts: 0,
			lastError: null
		});
	}

	function on(type: string, callback: (data: any) => void) {
		if (!listeners.has(type)) {
			listeners.set(type, new Set());
		}
		listeners.get(type)!.add(callback);

		// Return unsubscribe function
		return () => {
			listeners.get(type)?.delete(callback);
			if (listeners.get(type)?.size === 0) {
				listeners.delete(type);
			}
		};
	}

	function send(type: string, payload: any) {
		if (ws && ws.readyState === WebSocket.OPEN) {
			try {
				ws.send(JSON.stringify({ type, payload }));
			} catch (error) {
				console.error('Failed to send message:', error);
				throw new Error('Failed to send message. Connection may be unstable.');
			}
		} else {
			throw new Error('WebSocket not connected. Cannot send message.');
		}
	}

	function reset() {
		disconnect();
		set({
			connected: false,
			reconnecting: false,
			reconnectAttempts: 0,
			lastError: null
		});
	}

	// Cleanup on page unload
	if (typeof window !== 'undefined') {
		window.addEventListener('beforeunload', () => {
			disconnect();
		});

		// Handle visibility changes (browser tab switching)
		document.addEventListener('visibilitychange', () => {
			if (!document.hidden && !ws && currentUrl && !intentionallyClosed) {
				console.log('Page became visible, checking WebSocket connection...');
				connect(currentUrl, currentToken);
			}
		});
	}

	return {
		subscribe,
		connect,
		disconnect,
		on,
		send,
		reset
	};
}

export const resilientWebSocket = createResilientWebSocketStore();
