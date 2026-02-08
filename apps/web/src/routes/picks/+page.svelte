<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { Pick } from '$lib/api/types';

	let picks: Pick[] = [];
	let loading = true;
	let showModal = false;
	let editing: Pick | null = null;

	let formName = '';
	let formDescription = '';
	let formStatus = 'upcoming';
	let formDateStart = '';
	let formDateEnd = '';

	onMount(async () => {
		await loadPicks();
	});

	async function loadPicks() {
		try {
			picks = await api.getPicks();
		} catch (error) {
			console.error('Failed to load picks:', error);
		} finally {
			loading = false;
		}
	}

	function openCreate() {
		editing = null;
		formName = '';
		formDescription = '';
		formStatus = 'upcoming';
		formDateStart = '';
		formDateEnd = '';
		showModal = true;
	}

	function openEdit(pick: Pick) {
		editing = pick;
		formName = pick.name;
		formDescription = pick.description || '';
		formStatus = pick.status.toLowerCase();
		formDateStart = pick.date_start ? pick.date_start.substring(0, 16) : '';
		formDateEnd = pick.date_end ? pick.date_end.substring(0, 16) : '';
		showModal = true;
	}

	async function handleSubmit() {
		try {
			const data: any = {
				name: formName,
				description: formDescription || undefined,
				status: formStatus,
				date_start: formDateStart ? new Date(formDateStart).toISOString() : undefined,
				date_end: formDateEnd ? new Date(formDateEnd).toISOString() : undefined
			};

			if (editing) {
				const updated = await api.updatePick(editing.id, data);
				picks = picks.map(p => p.id === editing!.id ? updated : p);
			} else {
				const created = await api.createPick(data);
				picks = [created, ...picks];
			}
			showModal = false;
		} catch (error: any) {
			alert('Failed: ' + error.message);
		}
	}

	async function handleDelete(id: string) {
		if (!confirm('Delete this pick?')) return;
		try {
			await api.deletePick(id);
			picks = picks.filter(p => p.id !== id);
		} catch (error: any) {
			alert('Failed: ' + error.message);
		}
	}

	function statusColor(status: string): string {
		switch (status.toLowerCase()) {
			case 'upcoming': return '#3b82f6';
			case 'active': return '#22c55e';
			case 'past': return '#9ca3af';
			case 'recurring': return '#a855f7';
			default: return '#6b7280';
		}
	}

	function formatDate(dateStr?: string): string {
		if (!dateStr) return '';
		return new Date(dateStr).toLocaleDateString();
	}
</script>

<div class="page">
	<div class="header">
		<h1>Picks</h1>
		<p class="subtitle">Estate sales, drop-in sellers, and buying opportunities</p>
		<button class="btn-primary" on:click={openCreate}>+ New Pick</button>
	</div>

	{#if loading}
		<p class="loading">Loading picks...</p>
	{:else if picks.length === 0}
		<div class="empty-state">
			<p>No picks yet. Create one to track a buying opportunity.</p>
		</div>
	{:else}
		<div class="grid">
			{#each picks as pick}
				<div class="card">
					<div class="card-header">
						<h3>{pick.name}</h3>
						<span class="status-badge" style="background-color: {statusColor(pick.status)}">
							{pick.status}
						</span>
					</div>
					{#if pick.description}
						<p class="description">{pick.description}</p>
					{/if}
					<div class="card-meta">
						{#if pick.date_start}
							<span>{formatDate(pick.date_start)}</span>
						{/if}
						{#if pick.date_end}
							<span> — {formatDate(pick.date_end)}</span>
						{/if}
					</div>
					<div class="card-actions">
						<button class="btn-sm" on:click={() => openEdit(pick)}>Edit</button>
						<button class="btn-sm btn-danger" on:click={() => handleDelete(pick.id)}>Delete</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}

	{#if showModal}
		<div class="modal-overlay" on:click|self={() => showModal = false}>
			<div class="modal">
				<h2>{editing ? 'Edit Pick' : 'New Pick'}</h2>
				<form on:submit|preventDefault={handleSubmit}>
					<label>
						Name *
						<input type="text" bind:value={formName} required />
					</label>
					<label>
						Description
						<textarea bind:value={formDescription} rows="3"></textarea>
					</label>
					<label>
						Status
						<select bind:value={formStatus}>
							<option value="upcoming">Upcoming</option>
							<option value="active">Active</option>
							<option value="past">Past</option>
							<option value="recurring">Recurring</option>
						</select>
					</label>
					<label>
						Start Date
						<input type="datetime-local" bind:value={formDateStart} />
					</label>
					<label>
						End Date
						<input type="datetime-local" bind:value={formDateEnd} />
					</label>
					<div class="modal-actions">
						<button type="button" class="btn-secondary" on:click={() => showModal = false}>Cancel</button>
						<button type="submit" class="btn-primary">{editing ? 'Update' : 'Create'}</button>
					</div>
				</form>
			</div>
		</div>
	{/if}
</div>

<style>
	.page { padding: 1.5rem; max-width: 1200px; }
	.header { display: flex; align-items: center; gap: 1rem; flex-wrap: wrap; margin-bottom: 1.5rem; }
	.header h1 { margin: 0; font-size: 1.5rem; }
	.subtitle { color: #6b7280; margin: 0; flex: 1; }
	.loading { color: #6b7280; }
	.empty-state { text-align: center; padding: 3rem; color: #6b7280; background: #f9fafb; border-radius: 8px; }
	.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 1rem; }
	.card { background: white; border: 1px solid #e5e7eb; border-radius: 8px; padding: 1rem; }
	.card-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 0.5rem; }
	.card-header h3 { margin: 0; font-size: 1.1rem; }
	.status-badge { color: white; padding: 2px 8px; border-radius: 12px; font-size: 0.75rem; text-transform: capitalize; }
	.description { color: #6b7280; font-size: 0.875rem; margin: 0.5rem 0; }
	.card-meta { font-size: 0.8rem; color: #9ca3af; margin: 0.5rem 0; }
	.card-actions { display: flex; gap: 0.5rem; margin-top: 0.75rem; }
	.btn-primary { background: #3b82f6; color: white; border: none; padding: 0.5rem 1rem; border-radius: 6px; cursor: pointer; font-size: 0.875rem; }
	.btn-primary:hover { background: #2563eb; }
	.btn-secondary { background: #e5e7eb; color: #374151; border: none; padding: 0.5rem 1rem; border-radius: 6px; cursor: pointer; }
	.btn-sm { padding: 0.25rem 0.75rem; font-size: 0.8rem; border: 1px solid #d1d5db; background: white; border-radius: 4px; cursor: pointer; }
	.btn-sm:hover { background: #f3f4f6; }
	.btn-danger { color: #dc2626; border-color: #fca5a5; }
	.btn-danger:hover { background: #fef2f2; }
	.modal-overlay { position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 50; }
	.modal { background: white; border-radius: 12px; padding: 1.5rem; width: 90%; max-width: 500px; max-height: 90vh; overflow-y: auto; }
	.modal h2 { margin: 0 0 1rem; }
	.modal label { display: block; margin-bottom: 0.75rem; font-size: 0.875rem; font-weight: 500; }
	.modal input, .modal textarea, .modal select { width: 100%; padding: 0.5rem; border: 1px solid #d1d5db; border-radius: 6px; margin-top: 0.25rem; font-size: 0.875rem; box-sizing: border-box; }
	.modal-actions { display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 1rem; }
</style>
