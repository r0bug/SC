<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { Location } from '$lib/api/types';

	let locations: Location[] = [];
	let loading = true;
	let showModal = false;
	let editing: Location | null = null;

	let formName = '';
	let formAddress = '';
	let formCity = '';
	let formState = '';
	let formZip = '';
	let formLat = '';
	let formLng = '';

	onMount(async () => {
		await loadLocations();
	});

	async function loadLocations() {
		try {
			locations = await api.getLocations();
		} catch (error) {
			console.error('Failed to load locations:', error);
		} finally {
			loading = false;
		}
	}

	function openCreate() {
		editing = null;
		formName = '';
		formAddress = '';
		formCity = '';
		formState = '';
		formZip = '';
		formLat = '';
		formLng = '';
		showModal = true;
	}

	function openEdit(loc: Location) {
		editing = loc;
		formName = loc.name;
		formAddress = loc.address || '';
		formCity = loc.city || '';
		formState = loc.state || '';
		formZip = loc.zip || '';
		formLat = loc.coordinates_lat != null ? String(loc.coordinates_lat) : '';
		formLng = loc.coordinates_lng != null ? String(loc.coordinates_lng) : '';
		showModal = true;
	}

	async function handleSubmit() {
		try {
			const data: Partial<Location> = {
				name: formName,
				address: formAddress || undefined,
				city: formCity || undefined,
				state: formState || undefined,
				zip: formZip || undefined,
				coordinates_lat: formLat ? parseFloat(formLat) : undefined,
				coordinates_lng: formLng ? parseFloat(formLng) : undefined
			};

			if (editing) {
				const updated = await api.updateLocation(editing.id, data);
				locations = locations.map(l => l.id === editing!.id ? updated : l);
			} else {
				const created = await api.createLocation(data);
				locations = [created, ...locations];
			}
			showModal = false;
		} catch (error: unknown) {
			alert('Failed: ' + (error instanceof Error ? error.message : String(error)));
		}
	}

	async function handleDelete(id: string) {
		if (!confirm('Delete this location?')) return;
		try {
			await api.deleteLocation(id);
			locations = locations.filter(l => l.id !== id);
		} catch (error: unknown) {
			alert('Failed: ' + (error instanceof Error ? error.message : String(error)));
		}
	}

	function formatAddress(loc: Location): string {
		const parts = [loc.address, loc.city, loc.state, loc.zip].filter(Boolean);
		return parts.join(', ');
	}
</script>

<div class="page">
	<div class="header">
		<h1>Locations</h1>
		<p class="subtitle">Physical places linked to picks, contacts, and events</p>
		<button class="btn-primary" on:click={openCreate}>+ New Location</button>
	</div>

	{#if loading}
		<p class="loading">Loading locations...</p>
	{:else if locations.length === 0}
		<div class="empty-state">
			<p>No locations yet. Add one to start tracking places.</p>
		</div>
	{:else}
		<div class="grid">
			{#each locations as loc}
				<div class="card">
					<div class="card-header">
						<h3>{loc.name}</h3>
					</div>
					{#if formatAddress(loc)}
						<p class="address">{formatAddress(loc)}</p>
					{/if}
					{#if loc.coordinates_lat != null && loc.coordinates_lng != null}
						<p class="coords">{loc.coordinates_lat.toFixed(4)}, {loc.coordinates_lng.toFixed(4)}</p>
					{/if}
					<div class="card-actions">
						<button class="btn-sm" on:click={() => openEdit(loc)}>Edit</button>
						<button class="btn-sm btn-danger" on:click={() => handleDelete(loc.id)}>Delete</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}

	{#if showModal}
		<div class="modal-overlay" role="button" tabindex="0" on:click|self={() => showModal = false} on:keydown={(e) => e.key === 'Enter' && (showModal = false)}>
			<div class="modal">
				<h2>{editing ? 'Edit Location' : 'New Location'}</h2>
				<form on:submit|preventDefault={handleSubmit}>
					<label>
						Name *
						<input type="text" bind:value={formName} required />
					</label>
					<label>
						Address
						<input type="text" bind:value={formAddress} />
					</label>
					<div class="row">
						<label class="flex-2">
							City
							<input type="text" bind:value={formCity} />
						</label>
						<label class="flex-1">
							State
							<input type="text" bind:value={formState} />
						</label>
						<label class="flex-1">
							ZIP
							<input type="text" bind:value={formZip} />
						</label>
					</div>
					<div class="row">
						<label class="flex-1">
							Latitude
							<input type="number" step="any" bind:value={formLat} />
						</label>
						<label class="flex-1">
							Longitude
							<input type="number" step="any" bind:value={formLng} />
						</label>
					</div>
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
	.card-header { margin-bottom: 0.5rem; }
	.card-header h3 { margin: 0; font-size: 1.1rem; }
	.address { color: #374151; font-size: 0.875rem; margin: 0.25rem 0; }
	.coords { color: #9ca3af; font-size: 0.8rem; margin: 0.25rem 0; }
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
	.modal input { width: 100%; padding: 0.5rem; border: 1px solid #d1d5db; border-radius: 6px; margin-top: 0.25rem; font-size: 0.875rem; box-sizing: border-box; }
	.modal-actions { display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 1rem; }
	.row { display: flex; gap: 0.75rem; }
	.flex-1 { flex: 1; }
	.flex-2 { flex: 2; }
</style>
