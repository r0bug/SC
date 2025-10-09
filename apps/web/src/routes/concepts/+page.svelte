<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { Concept } from '$lib/api/types';

	let concepts: Concept[] = [];
	let loading = true;
	let showModal = false;
	let editing: Concept | null = null;
	let formName = '';
	let formDescription = '';

	onMount(async () => {
		await loadConcepts();
		api.onWebSocketMessage('concept_updated', (data) => {
			concepts = concepts.map(c => c.id === data.id ? data : c);
		});
		api.onWebSocketMessage('concept_created', (data) => {
			concepts = [data, ...concepts];
		});
	});

	async function loadConcepts() {
		try {
			concepts = await api.getConcepts();
		} catch (error) {
			console.error('Failed to load concepts:', error);
		} finally {
			loading = false;
		}
	}

	async function handleSubmit() {
		try {
			const conceptData = { name: formName, description: formDescription };
			if (editing) {
				const updated = await api.updateConcept(editing.id, conceptData);
				concepts = concepts.map(c => c.id === editing!.id ? updated : c);
			} else {
				const created = await api.createConcept(conceptData as any);
				concepts = [created, ...concepts];
			}
			showModal = false;
		} catch (error: any) {
			alert('Failed: ' + error.message);
		}
	}

	async function handleDelete(id: string) {
		if (!confirm('Delete?')) return;
		try {
			await api.deleteConcept(id);
			concepts = concepts.filter(c => c.id !== id);
		} catch (error: any) {
			alert('Failed: ' + error.message);
		}
	}
</script>

<div class="page">
	<div class="header">
		<h1>Concepts</h1>
		<button on:click={() => { editing = null; formName = ''; formDescription = ''; showModal = true; }} class="btn btn-primary">+ New Concept</button>
	</div>

	{#if loading}
		<div>Loading...</div>
	{:else}
		<div class="grid">
			{#each concepts as concept}
				<div class="card">
					<h3>{concept.name}</h3>
					{#if concept.description}
						<p>{concept.description}</p>
					{/if}
					<div class="actions">
						<button on:click={() => { editing = concept; formName = concept.name; formDescription = concept.description || ''; showModal = true; }} class="btn btn-sm">Edit</button>
						<button on:click={() => handleDelete(concept.id)} class="btn btn-sm">Delete</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

{#if showModal}
	<div
		class="modal-overlay"
		on:click={() => showModal = false}
		on:keydown={(e) => e.key === 'Escape' && (showModal = false)}
		role="button"
		tabindex="-1"
	>
		<div class="modal" on:click|stopPropagation>
			<h2>{editing ? 'Edit' : 'New'} Concept</h2>
			<form on:submit|preventDefault={handleSubmit}>
				<div>
					<label for="concept-name">Name</label>
					<input id="concept-name" type="text" bind:value={formName} required />
				</div>
				<div>
					<label for="concept-description">Description</label>
					<textarea id="concept-description" bind:value={formDescription} rows="4"></textarea>
				</div>
				<div class="actions">
					<button type="button" on:click={() => showModal = false} class="btn">Cancel</button>
					<button type="submit" class="btn btn-primary">Save</button>
				</div>
			</form>
		</div>
	</div>
{/if}

<style>
	.page { padding: 2rem; max-width: 1200px; }
	.header { display: flex; justify-content: space-between; margin-bottom: 2rem; }
	.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 1.5rem; }
	.card { padding: 1.5rem; }
	.actions { display: flex; gap: 0.5rem; margin-top: 1rem; }
	.btn-sm { padding: 0.375rem 0.75rem; font-size: 0.875rem; }
	.modal-overlay { position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000; }
	.modal { background: white; padding: 2rem; border-radius: 8px; width: 90%; max-width: 500px; }
	label { display: block; margin-bottom: 0.5rem; font-weight: 500; }
	input, textarea { width: 100%; padding: 0.75rem; border: 1px solid var(--gray-300); border-radius: 6px; margin-bottom: 1rem; }
</style>
