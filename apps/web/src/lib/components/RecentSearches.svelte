<script lang="ts">
	import { api } from '$lib/api/api';
	import type { SearchHistory } from '$lib/api/types';
	import { onMount } from 'svelte';
	import { createEventDispatcher } from 'svelte';

	const dispatch = createEventDispatcher<{ search: string }>();

	let searches: SearchHistory[] = [];
	let loading = true;
	let error: string | null = null;

	onMount(async () => {
		try {
			searches = await api.getSearchHistory();
			loading = false;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load search history';
			loading = false;
		}
	});

	function handleSearchClick(query: string) {
		dispatch('search', query);
	}

	function formatDate(dateStr: string): string {
		const date = new Date(dateStr);
		const now = new Date();
		const diff = now.getTime() - date.getTime();
		const hours = Math.floor(diff / (1000 * 60 * 60));
		const days = Math.floor(hours / 24);

		if (hours < 1) return 'Just now';
		if (hours < 24) return `${hours}h ago`;
		if (days < 7) return `${days}d ago`;
		return date.toLocaleDateString();
	}

	async function handleClearHistory() {
		if (!confirm('Clear all search history?')) return;

		try {
			await api.clearSearchHistory();
			searches = [];
		} catch (e) {
			alert('Failed to clear history: ' + (e instanceof Error ? e.message : 'Unknown error'));
		}
	}
</script>

<div class="recent-searches">
	<div class="header">
		<h3>Recent Searches</h3>
		{#if searches.length > 0}
			<button class="clear-button" on:click={handleClearHistory}>Clear</button>
		{/if}
	</div>

	{#if loading}
		<div class="loading">Loading...</div>
	{:else if error}
		<div class="error">{error}</div>
	{:else if searches.length === 0}
		<div class="empty-state">No recent searches</div>
	{:else}
		<div class="search-list">
			{#each searches.slice(0, 10) as search (search.id)}
				<button class="search-item" on:click={() => handleSearchClick(search.query)}>
					<div class="search-icon">🔍</div>
					<div class="search-details">
						<div class="search-query">{search.query}</div>
						<div class="search-meta">
							<span>{search.result_count} results</span>
							<span>•</span>
							<span>{formatDate(search.created_at)}</span>
							{#if search.privacy_mode}
								<span>•</span>
								<span class="privacy-badge">🔒 Private</span>
							{/if}
						</div>
					</div>
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	.recent-searches {
		background: white;
		border: 1px solid #e0e0e0;
		border-radius: 8px;
		padding: 1rem;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	h3 {
		margin: 0;
		font-size: 1rem;
		font-weight: 600;
	}

	.clear-button {
		padding: 0.25rem 0.75rem;
		font-size: 0.75rem;
		color: #757575;
		background: none;
		border: 1px solid #e0e0e0;
		border-radius: 4px;
		cursor: pointer;
		transition: all 0.2s;
	}

	.clear-button:hover {
		background: #f5f5f5;
		border-color: #bdbdbd;
	}

	.loading,
	.error,
	.empty-state {
		text-align: center;
		padding: 2rem;
		color: #757575;
		font-size: 0.875rem;
	}

	.error {
		color: #d32f2f;
	}

	.search-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.search-item {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.75rem;
		background: #f9f9f9;
		border: 1px solid #e0e0e0;
		border-radius: 4px;
		cursor: pointer;
		text-align: left;
		transition: all 0.2s;
		width: 100%;
	}

	.search-item:hover {
		background: #f0f0f0;
		border-color: #0066cc;
	}

	.search-icon {
		font-size: 1.25rem;
		flex-shrink: 0;
	}

	.search-details {
		flex: 1;
		min-width: 0;
	}

	.search-query {
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		margin-bottom: 0.25rem;
	}

	.search-meta {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.75rem;
		color: #757575;
	}

	.privacy-badge {
		color: #ff9800;
		font-weight: 500;
	}
</style>
