<script lang="ts">
	import { createEventDispatcher, onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { MatcherInput, LabelListEntry } from '$lib/api/api';
	import { toasts } from '$lib/stores/toast';

	export let criteria: MatcherInput[] = [];

	const dispatch = createEventDispatcher();

	let searchQuery = '';
	let labels: LabelListEntry[] = [];
	let filteredLabels: LabelListEntry[] = [];
	let loading = true;
	let creating = false;
	let newLabelName = '';
	let showCreateForm = false;

	onMount(async () => {
		try {
			const result = await api.listLabels();
			labels = result.labels;
		} catch {
			labels = [];
		} finally {
			loading = false;
		}
	});

	$: {
		const q = searchQuery.toLowerCase();
		filteredLabels = labels.filter(l =>
			!q || l.name.toLowerCase().includes(q)
		);
	}

	async function selectLabel(label: LabelListEntry) {
		try {
			// Add criteria as a new group to this label
			await api.addMatcherGroup(label.concept_id, criteria);
			toasts.success(`Criteria added to "${label.name}"`);
			dispatch('labelSelected', { conceptId: label.concept_id, isNew: false });
		} catch (e: any) {
			toasts.error(e.message || 'Failed to add criteria');
		}
	}

	async function createAndSelect() {
		if (!newLabelName.trim()) return;
		creating = true;
		try {
			// Create concept
			const concept = await api.createConcept({
				name: newLabelName.trim(),
				description: ''
			});

			// Add criteria as a group
			if (criteria.length > 0) {
				await api.addMatcherGroup(concept.id, criteria);
			}

			toasts.success(`Label "${concept.name}" created`);
			dispatch('labelSelected', { conceptId: concept.id, isNew: true });
		} catch (e: any) {
			toasts.error(e.message || 'Failed to create label');
		} finally {
			creating = false;
		}
	}

	function close() {
		dispatch('close');
	}
</script>

<div class="label-picker">
	<div class="picker-header">
		<h4>Select Label</h4>
		<button class="btn-close" on:click={close}>&times;</button>
	</div>

	<input
		type="text"
		class="search-input"
		placeholder="Search labels..."
		bind:value={searchQuery}
	/>

	{#if criteria.length > 0}
		<div class="criteria-preview">
			<span class="preview-label">Adding:</span>
			{#each criteria as c}
				<span class="preview-chip">{c.element_type} {c.match_mode || 'contains'} "{c.match_value}"</span>
			{/each}
		</div>
	{/if}

	<div class="label-list">
		{#if loading}
			<div class="loading">Loading...</div>
		{:else if filteredLabels.length === 0}
			<div class="empty">No matching labels</div>
		{:else}
			{#each filteredLabels as label}
				<button class="label-item" on:click={() => selectLabel(label)}>
					<span class="label-name">{label.name}</span>
					<span class="label-meta">
						{label.matcher_count} criteria
						&middot; {label.confirmed_count} confirmed
						&middot; {label.suggested_count} suggested
					</span>
				</button>
			{/each}
		{/if}
	</div>

	<div class="create-section">
		{#if showCreateForm}
			<div class="create-form">
				<input
					type="text"
					placeholder="New label name..."
					bind:value={newLabelName}
					class="create-input"
				/>
				<button class="btn btn-primary btn-sm" on:click={createAndSelect} disabled={creating || !newLabelName.trim()}>
					{creating ? 'Creating...' : 'Create'}
				</button>
				<button class="btn btn-sm" on:click={() => showCreateForm = false}>Cancel</button>
			</div>
		{:else}
			<button class="btn btn-outline btn-sm" on:click={() => showCreateForm = true}>+ Create New Label</button>
		{/if}
	</div>
</div>

<style>
	.label-picker {
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
		background: white;
		box-shadow: 0 4px 12px rgba(0,0,0,0.1);
		width: 320px;
		max-height: 400px;
		display: flex;
		flex-direction: column;
	}

	.picker-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.5rem 0.75rem;
		border-bottom: 1px solid #e5e7eb;
	}

	.picker-header h4 {
		margin: 0;
		font-size: 0.9rem;
	}

	.btn-close {
		border: none;
		background: none;
		font-size: 1.2rem;
		cursor: pointer;
		color: #6b7280;
	}

	.search-input {
		margin: 0.5rem 0.75rem;
		padding: 0.4rem 0.6rem;
		border: 1px solid #d1d5db;
		border-radius: 0.25rem;
		font-size: 0.85rem;
	}

	.criteria-preview {
		padding: 0.25rem 0.75rem;
		display: flex;
		flex-wrap: wrap;
		gap: 0.25rem;
		align-items: center;
	}

	.preview-label {
		font-size: 0.7rem;
		color: #6b7280;
	}

	.preview-chip {
		font-size: 0.7rem;
		background: #dbeafe;
		color: #1d4ed8;
		padding: 0.1rem 0.3rem;
		border-radius: 0.25rem;
	}

	.label-list {
		flex: 1;
		overflow-y: auto;
		padding: 0.25rem 0;
	}

	.label-item {
		display: flex;
		flex-direction: column;
		width: 100%;
		padding: 0.5rem 0.75rem;
		border: none;
		background: none;
		cursor: pointer;
		text-align: left;
	}

	.label-item:hover {
		background: #f3f4f6;
	}

	.label-name {
		font-size: 0.85rem;
		font-weight: 500;
	}

	.label-meta {
		font-size: 0.7rem;
		color: #9ca3af;
	}

	.loading, .empty {
		padding: 1rem;
		text-align: center;
		color: #9ca3af;
		font-size: 0.85rem;
	}

	.create-section {
		padding: 0.5rem 0.75rem;
		border-top: 1px solid #e5e7eb;
	}

	.create-form {
		display: flex;
		gap: 0.4rem;
		align-items: center;
	}

	.create-input {
		flex: 1;
		padding: 0.3rem 0.5rem;
		border: 1px solid #d1d5db;
		border-radius: 0.25rem;
		font-size: 0.8rem;
	}

	.btn {
		padding: 0.3rem 0.6rem;
		border-radius: 0.25rem;
		border: 1px solid #d1d5db;
		background: white;
		cursor: pointer;
		font-size: 0.8rem;
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-primary {
		background: #3b82f6;
		color: white;
		border-color: #3b82f6;
	}

	.btn-outline {
		background: transparent;
		color: #6b7280;
	}

	.btn-sm {
		padding: 0.2rem 0.5rem;
		font-size: 0.75rem;
	}
</style>
