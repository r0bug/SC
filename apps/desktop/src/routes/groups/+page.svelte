<script lang="ts">
	import { onMount } from 'svelte';
	import { tauriApi } from '$lib/api/tauri-api';
	import type { Group } from '$lib/api/types';

	let groups: Group[] = [];
	let loading = true;
	let sortBy: 'name' | 'size' | 'date' = 'name';

	onMount(async () => {
		await loadGroups();
	});

	async function loadGroups() {
		try {
			loading = true;
			groups = await tauriApi.getGroups();
			sortGroups();
		} catch (error) {
			console.error('Failed to load groups:', error);
		} finally {
			loading = false;
		}
	}

	async function deleteGroup(id: string, name: string) {
		if (!confirm(`Are you sure you want to delete the group "${name}"?`)) return;

		try {
			await tauriApi.deleteGroup(id);
			groups = groups.filter(g => g.id !== id);
			await tauriApi.showNotification('Group Deleted', `${name} has been deleted successfully`);
		} catch (error) {
			console.error('Failed to delete group:', error);
			alert('Failed to delete group');
		}
	}

	function sortGroups() {
		switch (sortBy) {
			case 'name':
				groups = groups.sort((a, b) => a.name.localeCompare(b.name));
				break;
			case 'size':
				groups = groups.sort((a, b) => b.contact_ids.length - a.contact_ids.length);
				break;
			case 'date':
				groups = groups.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime());
				break;
		}
	}

	function handleSortChange() {
		sortGroups();
	}
</script>

<div class="page">
	<div class="header">
		<div>
			<h1>Groups</h1>
			<p class="subtitle">Organize your contacts into groups</p>
		</div>
		<div class="header-actions">
			<select bind:value={sortBy} on:change={handleSortChange} class="sort-select">
				<option value="name">Sort by Name</option>
				<option value="size">Sort by Size</option>
				<option value="date">Sort by Date</option>
			</select>
			<a href="/groups/new" class="btn btn-primary">+ Create Group</a>
		</div>
	</div>

	{#if loading}
		<div class="loading">Loading groups...</div>
	{:else if groups.length === 0}
		<div class="empty card">
			<p>No groups found.</p>
			<p>Groups help you organize contacts by team, project, or any category you choose.</p>
		</div>
	{:else}
		<div class="group-grid">
			{#each groups as group}
				<div class="group-card card">
					<div class="group-header">
						<div class="group-title">
							<h3>{group.name}</h3>
							<span class="contact-count">{group.contact_ids.length} contacts</span>
						</div>
						<button
							on:click={() => deleteGroup(group.id, group.name)}
							class="btn-icon btn-danger"
							title="Delete group"
						>
							🗑️
						</button>
					</div>

					{#if group.description}
						<p class="group-description">{group.description}</p>
					{/if}

					<div class="group-meta">
						<span class="meta-item">Created {new Date(group.created_at).toLocaleDateString()}</span>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page {
		padding: 2rem;
		max-width: 1400px;
		margin: 0 auto;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 2rem;
	}

	.header h1 {
		margin: 0 0 0.5rem;
		font-size: 2rem;
		color: var(--gray-900);
	}

	.subtitle {
		margin: 0;
		color: var(--gray-600);
		font-size: 1rem;
	}

	.loading {
		text-align: center;
		padding: 3rem;
		color: var(--gray-600);
		font-size: 1.125rem;
	}

	.empty {
		text-align: center;
		padding: 3rem;
	}

	.empty p {
		margin: 0 0 1rem;
		color: var(--gray-600);
		font-size: 1.125rem;
	}

	.group-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
		gap: 1.5rem;
	}

	.group-card {
		padding: 1.5rem;
		transition: transform 0.2s, box-shadow 0.2s;
	}

	.group-card:hover {
		transform: translateY(-2px);
		box-shadow: var(--shadow-md);
	}

	.header-actions {
		display: flex;
		gap: 1rem;
		align-items: center;
	}

	.sort-select {
		padding: 0.5rem 1rem;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		font-size: 0.875rem;
		background: white;
		cursor: pointer;
	}

	.sort-select:focus {
		outline: none;
		border-color: var(--primary);
		box-shadow: 0 0 0 3px var(--primary-light);
	}

	.group-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: 1rem;
	}

	.group-title {
		flex: 1;
	}

	.group-header h3 {
		margin: 0 0 0.5rem;
		font-size: 1.25rem;
		color: var(--gray-900);
	}

	.contact-count {
		background: var(--primary-light);
		color: var(--primary);
		padding: 0.25rem 0.75rem;
		border-radius: 999px;
		font-size: 0.75rem;
		font-weight: 600;
	}

	.btn-icon {
		padding: 0.5rem;
		background: none;
		border: none;
		cursor: pointer;
		font-size: 1.25rem;
		border-radius: 4px;
		transition: background-color 0.2s;
	}

	.btn-icon:hover {
		background: var(--gray-100);
	}

	.btn-danger:hover {
		background: var(--color-error-50);
	}

	.group-description {
		margin: 0 0 1rem;
		color: var(--gray-700);
		font-size: 0.875rem;
		line-height: 1.5;
	}

	.group-meta {
		display: flex;
		gap: 1rem;
		font-size: 0.75rem;
		color: var(--gray-500);
	}

	.meta-item {
		display: flex;
		align-items: center;
		gap: 0.25rem;
	}

	.card {
		background: white;
		border-radius: 8px;
		box-shadow: var(--shadow-sm);
	}
</style>
