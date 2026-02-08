<script lang="ts">
	import { api, ApiError } from '$lib/api/api';
	import type { LabelListEntry } from '$lib/api/api';
	import { toasts } from '$lib/stores/toast';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { hashColor } from '$lib/utils/colors';

	let labels: LabelListEntry[] = [];
	let loading = true;
	let scanning = false;
	let showCreateModal = false;
	let newName = '';
	let newDescription = '';
	let creating = false;
	let searchQuery = '';

	onMount(async () => {
		await loadLabels();
	});

	async function loadLabels() {
		loading = true;
		try {
			const result = await api.listLabels();
			labels = result.labels;
		} catch (e: unknown) {
			if (e instanceof ApiError) {
				toasts.error(e.getUserMessage());
			}
		} finally {
			loading = false;
		}
	}

	async function scanAll() {
		scanning = true;
		try {
			const result = await api.scanAllLabels();
			toasts.success(`Scan complete: ${result.new_suggestions} new suggestions`);
			await loadLabels();
		} catch (e: unknown) {
			toasts.error(e instanceof Error ? e.message : 'Scan failed');
		} finally {
			scanning = false;
		}
	}

	async function createLabel() {
		if (!newName.trim()) return;
		creating = true;
		try {
			const concept = await api.createConcept({
				name: newName.trim(),
				description: newDescription.trim() || undefined
			});
			toasts.success(`Label "${concept.name}" created`);
			showCreateModal = false;
			newName = '';
			newDescription = '';
			goto(`/labels/${concept.id}`);
		} catch (e: unknown) {
			toasts.error(e instanceof Error ? e.message : 'Failed to create');
		} finally {
			creating = false;
		}
	}

	$: filteredLabels = labels.filter(l => {
		if (!searchQuery) return true;
		const q = searchQuery.toLowerCase();
		return l.name.toLowerCase().includes(q) || (l.description || '').toLowerCase().includes(q);
	});
</script>

<div class="page">
	<div class="header">
		<h1>Labels</h1>
		<div class="header-actions">
			<input
				type="text"
				placeholder="Search labels..."
				bind:value={searchQuery}
				class="search-input"
			/>
			<button class="btn btn-secondary" on:click={scanAll} disabled={scanning}>
				{scanning ? 'Scanning...' : 'Scan All'}
			</button>
			<button class="btn btn-primary" on:click={() => showCreateModal = true}>
				Create Label
			</button>
		</div>
	</div>

	{#if loading}
		<div class="loading-state">
			<div class="spinner"></div>
			<p>Loading labels...</p>
		</div>
	{:else if filteredLabels.length === 0}
		<div class="empty-state">
			<p>{searchQuery ? 'No labels match your search.' : 'No labels yet.'}</p>
			<p class="sub">Create a label and define criteria to start matching communications.</p>
		</div>
	{:else}
		<div class="labels-grid">
			{#each filteredLabels as label}
				<a href="/labels/{label.concept_id}" class="label-card">
					<div class="card-header">
						<span class="label-name" style="color: {hashColor(label.name)}">{label.name}</span>
						{#if label.concept_type === 'communication_domain'}
							<span class="type-badge">domain</span>
						{/if}
					</div>
					{#if label.description}
						<p class="label-description">{label.description}</p>
					{/if}
					<div class="card-stats">
						<span class="stat">
							<span class="stat-value">{label.matcher_count}</span>
							<span class="stat-label">criteria</span>
						</span>
						<span class="stat">
							<span class="stat-value confirmed">{label.confirmed_count}</span>
							<span class="stat-label">confirmed</span>
						</span>
						<span class="stat">
							<span class="stat-value suggested">{label.suggested_count}</span>
							<span class="stat-label">suggested</span>
						</span>
					</div>
				</a>
			{/each}
		</div>
	{/if}

	{#if showCreateModal}
		<div class="modal-backdrop" role="button" tabindex="0" on:click|self={() => showCreateModal = false} on:keydown={(e) => e.key === 'Enter' && (showCreateModal = false)}>
			<div class="modal">
				<h3>Create New Label</h3>
				<div class="form-group">
					<label for="label-name">Name</label>
					<input id="label-name" type="text" bind:value={newName} placeholder="e.g. Sales Invoices" />
				</div>
				<div class="form-group">
					<label for="label-desc">Description (optional)</label>
					<textarea id="label-desc" bind:value={newDescription} rows="2" placeholder="What does this label identify?"></textarea>
				</div>
				<div class="modal-actions">
					<button class="btn" on:click={() => showCreateModal = false}>Cancel</button>
					<button class="btn btn-primary" on:click={createLabel} disabled={creating || !newName.trim()}>
						{creating ? 'Creating...' : 'Create & Configure'}
					</button>
				</div>
			</div>
		</div>
	{/if}
</div>

<style>
	.page {
		padding: 2rem;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1.5rem;
		flex-wrap: wrap;
		gap: 1rem;
	}

	.header h1 {
		margin: 0;
		font-size: 1.5rem;
	}

	.header-actions {
		display: flex;
		gap: 0.5rem;
		align-items: center;
	}

	.search-input {
		padding: 0.4rem 0.8rem;
		border: 1px solid #d1d5db;
		border-radius: 0.375rem;
		font-size: 0.9rem;
		width: 200px;
	}

	.labels-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
		gap: 1rem;
	}

	.label-card {
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
		padding: 1rem;
		background: white;
		text-decoration: none;
		color: inherit;
		transition: all 0.15s;
	}

	.label-card:hover {
		border-color: #93c5fd;
		box-shadow: 0 2px 8px rgba(0,0,0,0.06);
	}

	.card-header {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin-bottom: 0.25rem;
	}

	.label-name {
		font-weight: 600;
		font-size: 1rem;
	}

	.type-badge {
		font-size: 0.65rem;
		background: #dbeafe;
		color: #1d4ed8;
		padding: 0.1rem 0.35rem;
		border-radius: 0.25rem;
		font-weight: 500;
	}

	.label-description {
		color: #6b7280;
		font-size: 0.8rem;
		margin: 0.25rem 0 0.5rem;
	}

	.card-stats {
		display: flex;
		gap: 1rem;
		margin-top: 0.5rem;
	}

	.stat {
		display: flex;
		flex-direction: column;
		align-items: center;
	}

	.stat-value {
		font-size: 1.1rem;
		font-weight: 600;
		color: #374151;
	}

	.stat-value.confirmed {
		color: #16a34a;
	}

	.stat-value.suggested {
		color: #d97706;
	}

	.stat-label {
		font-size: 0.7rem;
		color: #9ca3af;
	}

	.loading-state, .empty-state {
		text-align: center;
		padding: 3rem;
		color: #6b7280;
	}

	.empty-state .sub {
		font-size: 0.85rem;
		color: #9ca3af;
	}

	.spinner {
		width: 24px;
		height: 24px;
		border: 3px solid #e5e7eb;
		border-top-color: #3b82f6;
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
		margin: 0 auto 0.5rem;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	.modal-backdrop {
		position: fixed;
		inset: 0;
		background: rgba(0,0,0,0.3);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		background: white;
		border-radius: 0.5rem;
		padding: 1.5rem;
		width: 400px;
		max-width: 90vw;
	}

	.modal h3 {
		margin: 0 0 1rem;
	}

	.form-group {
		margin-bottom: 0.75rem;
	}

	.form-group label {
		display: block;
		font-size: 0.85rem;
		font-weight: 500;
		margin-bottom: 0.25rem;
	}

	.form-group input, .form-group textarea {
		width: 100%;
		padding: 0.4rem 0.6rem;
		border: 1px solid #d1d5db;
		border-radius: 0.25rem;
		font-size: 0.9rem;
		box-sizing: border-box;
	}

	.modal-actions {
		display: flex;
		justify-content: flex-end;
		gap: 0.5rem;
		margin-top: 1rem;
	}

	.btn {
		padding: 0.5rem 1rem;
		border-radius: 0.375rem;
		border: 1px solid #d1d5db;
		background: white;
		cursor: pointer;
		font-size: 0.85rem;
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

	.btn-primary:hover:not(:disabled) {
		background: #2563eb;
	}

	.btn-secondary {
		background: #f3f4f6;
		color: #374151;
	}

	.btn-secondary:hover:not(:disabled) {
		background: #e5e7eb;
	}
</style>
