import { writable } from 'svelte/store';

interface OptimisticUpdate {
	id: string;
	type: 'create' | 'update' | 'delete';
	entity: string;
	data: any;
	timestamp: number;
}

function createOptimisticStore() {
	const { subscribe, set, update } = writable<OptimisticUpdate[]>([]);

	return {
		subscribe,

		add(type: 'create' | 'update' | 'delete', entity: string, data: any) {
			const id = `${type}_${entity}_${Date.now()}`;
			update(updates => [...updates, { id, type, entity, data, timestamp: Date.now() }]);
			return id;
		},

		remove(id: string) {
			update(updates => updates.filter(u => u.id !== id));
		},

		clear() {
			set([]);
		},

		async execute<T>(
			optimisticData: any,
			apiCall: () => Promise<T>,
			onSuccess?: (result: T) => void,
			onError?: (error: Error) => void
		): Promise<T | null> {
			try {
				const result = await apiCall();
				if (onSuccess) onSuccess(result);
				return result;
			} catch (error) {
				if (onError) onError(error as Error);
				return null;
			}
		}
	};
}

export const optimisticUpdates = createOptimisticStore();
