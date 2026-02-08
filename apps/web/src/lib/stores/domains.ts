import { writable, get } from 'svelte/store';
import { api } from '$lib/api/api';

export interface DomainEntry {
	concept_id: string;
	name: string;
	keywords: string[];
}

function createDomainStore() {
	const { subscribe, set, update } = writable<DomainEntry[]>([]);
	let loaded = false;

	return {
		subscribe,

		async load() {
			if (loaded) return;
			try {
				const result = await api.searchEmailDomains('');
				set(result.domains);
				loaded = true;
			} catch {
				// Silently fail — autocomplete will fall back to API search
			}
		},

		invalidate() {
			loaded = false;
			set([]);
		},

		addDomain(d: DomainEntry) {
			update((domains) => {
				if (domains.some((x) => x.concept_id === d.concept_id)) return domains;
				return [...domains, d];
			});
		},

		/** Client-side prefix search for fast autocomplete */
		search(query: string): DomainEntry[] {
			const q = query.toLowerCase();
			const all = get({ subscribe });
			return all.filter(
				(d) =>
					d.name.toLowerCase().includes(q) ||
					d.keywords.some((k) => k.toLowerCase().includes(q))
			);
		}
	};
}

export const domainStore = createDomainStore();
