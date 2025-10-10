<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { SearchHistory } from '$lib/api/types';

	let history: SearchHistory[] = [];
	let loading = true;

	onMount(async () => {
		await loadHistory();
	});

	async function loadHistory() {
		try {
			history = await api.getSearchHistory();
		} catch (error) {
			console.error('Failed to load history:', error);
		} finally {
			loading = false;
		}
	}

	async function handleClear() {
		if (!confirm('Clear all search history?')) return;
		try {
			await api.clearSearchHistory();
			history = [];
		} catch (error: any) {
			alert('Failed: ' + error.message);
		}
	}
</script>

<div class="page">
	<div class="header">
		<h1>Search History</h1>
		{#if history.length > 0}
			<button on:click={handleClear} class="btn btn-secondary">Clear History</button>
		{/if}
	</div>

	{#if loading}
		<div>Loading...</div>
	{:else if history.length === 0}
		<div class="empty">No search history</div>
	{:else}
		<div class="history-list">
			{#each history as item}
				<div class="history-item card">
					<div class="query">"{item.query}"</div>
					<div class="meta">
						<span>{item.result_count} results</span>
						<span>{new Date(item.created_at).toLocaleString()}</span>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page { padding: 2rem; max-width: 800px; }
	.header { display: flex; justify-content: space-between; margin-bottom: 2rem; }
	.history-list { display: flex; flex-direction: column; gap: 0.5rem; }
	.history-item { padding: 1rem; }
	.query { font-weight: 600; margin-bottom: 0.5rem; }
	.meta { font-size: 0.875rem; color: var(--gray-600); display: flex; gap: 1rem; }
	.empty { text-align: center; padding: 3rem; }
</style>
