<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { Concept, ConceptKeyword } from '$lib/api/types';

	let concepts: Concept[] = [];
	let loading = true;
	let showModal = false;
	let editing: Concept | null = null;
	let formName = '';
	let formDescription = '';
	let formParentId = '';

	// Keyword management
	let selectedConcept: Concept | null = null;
	let newKeyword = '';

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

	function openCreate(parentId?: string) {
		editing = null;
		formName = '';
		formDescription = '';
		formParentId = parentId || '';
		showModal = true;
	}

	function openEdit(concept: Concept) {
		editing = concept;
		formName = concept.name;
		formDescription = concept.description || '';
		formParentId = concept.parent_id || '';
		showModal = true;
	}

	async function handleSubmit() {
		try {
			const data: any = {
				name: formName,
				description: formDescription || undefined,
				parent_id: formParentId || undefined
			};
			if (editing) {
				const updated = await api.updateConcept(editing.id, data);
				concepts = concepts.map(c => c.id === editing!.id ? updated : c);
			} else {
				const created = await api.createConcept(data);
				concepts = [created, ...concepts];
			}
			showModal = false;
		} catch (error: any) {
			alert('Failed: ' + error.message);
		}
	}

	async function handleDelete(id: string) {
		if (!confirm('Delete this concept and all its keywords?')) return;
		try {
			await api.deleteConcept(id);
			concepts = concepts.filter(c => c.id !== id);
			if (selectedConcept?.id === id) selectedConcept = null;
		} catch (error: any) {
			alert('Failed: ' + error.message);
		}
	}

	function selectConcept(concept: Concept) {
		selectedConcept = selectedConcept?.id === concept.id ? null : concept;
		newKeyword = '';
	}

	async function addKeyword() {
		if (!selectedConcept || !newKeyword.trim()) return;
		try {
			const kw = await api.addConceptKeyword(selectedConcept.id, newKeyword.trim());
			concepts = concepts.map(c => {
				if (c.id === selectedConcept!.id) {
					return { ...c, keywords: [...c.keywords, kw] };
				}
				return c;
			});
			selectedConcept = concepts.find(c => c.id === selectedConcept!.id) || null;
			newKeyword = '';
		} catch (error: any) {
			alert('Failed to add keyword: ' + error.message);
		}
	}

	async function removeKeyword(kwId: string) {
		if (!selectedConcept) return;
		try {
			await api.removeConceptKeyword(selectedConcept.id, kwId);
			concepts = concepts.map(c => {
				if (c.id === selectedConcept!.id) {
					return { ...c, keywords: c.keywords.filter(k => k.id !== kwId) };
				}
				return c;
			});
			selectedConcept = concepts.find(c => c.id === selectedConcept!.id) || null;
		} catch (error: any) {
			alert('Failed to remove keyword: ' + error.message);
		}
	}

	// Build tree structure
	$: rootConcepts = concepts.filter(c => !c.parent_id);
	$: childrenMap = concepts.reduce((acc, c) => {
		if (c.parent_id) {
			if (!acc[c.parent_id]) acc[c.parent_id] = [];
			acc[c.parent_id].push(c);
		}
		return acc;
	}, {} as Record<string, Concept[]>);

	function getChildren(parentId: string): Concept[] {
		return childrenMap[parentId] || [];
	}

	function getParentName(parentId: string): string {
		const parent = concepts.find(c => c.id === parentId);
		return parent ? parent.name : '';
	}
</script>

<div class="page">
	<div class="header">
		<h1>Concepts</h1>
		<p class="subtitle">Hierarchical concept graph with keyword detection</p>
		<button class="btn-primary" on:click={() => openCreate()}>+ New Concept</button>
	</div>

	{#if loading}
		<p class="loading">Loading...</p>
	{:else}
		<div class="layout">
			<div class="tree-panel">
				{#if rootConcepts.length === 0}
					<div class="empty-state">
						<p>No concepts yet. Create one to start building your concept graph.</p>
					</div>
				{:else}
					{#each rootConcepts as concept}
						<div class="tree-node">
							<div class="node-row" class:selected={selectedConcept?.id === concept.id}
								on:click={() => selectConcept(concept)}>
								<span class="node-name">{concept.name}</span>
								{#if concept.keywords.length > 0}
									<span class="kw-count">{concept.keywords.length} kw</span>
								{/if}
								<div class="node-actions">
									<button class="btn-xs" on:click|stopPropagation={() => openCreate(concept.id)} title="Add child">+</button>
									<button class="btn-xs" on:click|stopPropagation={() => openEdit(concept)}>Edit</button>
									<button class="btn-xs btn-danger" on:click|stopPropagation={() => handleDelete(concept.id)}>Del</button>
								</div>
							</div>
							{#if getChildren(concept.id).length > 0}
								<div class="children">
									{#each getChildren(concept.id) as child}
										<div class="tree-node child">
											<div class="node-row" class:selected={selectedConcept?.id === child.id}
												on:click={() => selectConcept(child)}>
												<span class="node-name">{child.name}</span>
												{#if child.keywords.length > 0}
													<span class="kw-count">{child.keywords.length} kw</span>
												{/if}
												<div class="node-actions">
													<button class="btn-xs" on:click|stopPropagation={() => openCreate(child.id)} title="Add child">+</button>
													<button class="btn-xs" on:click|stopPropagation={() => openEdit(child)}>Edit</button>
													<button class="btn-xs btn-danger" on:click|stopPropagation={() => handleDelete(child.id)}>Del</button>
												</div>
											</div>
											{#if getChildren(child.id).length > 0}
												<div class="children">
													{#each getChildren(child.id) as grandchild}
														<div class="tree-node child">
															<div class="node-row" class:selected={selectedConcept?.id === grandchild.id}
																on:click={() => selectConcept(grandchild)}>
																<span class="node-name">{grandchild.name}</span>
																{#if grandchild.keywords.length > 0}
																	<span class="kw-count">{grandchild.keywords.length} kw</span>
																{/if}
																<div class="node-actions">
																	<button class="btn-xs" on:click|stopPropagation={() => openEdit(grandchild)}>Edit</button>
																	<button class="btn-xs btn-danger" on:click|stopPropagation={() => handleDelete(grandchild.id)}>Del</button>
																</div>
															</div>
														</div>
													{/each}
												</div>
											{/if}
										</div>
									{/each}
								</div>
							{/if}
						</div>
					{/each}
				{/if}
			</div>

			<div class="detail-panel">
				{#if selectedConcept}
					<div class="detail-card">
						<h2>{selectedConcept.name}</h2>
						{#if selectedConcept.description}
							<p class="detail-desc">{selectedConcept.description}</p>
						{/if}
						{#if selectedConcept.parent_id}
							<p class="detail-parent">Parent: <strong>{getParentName(selectedConcept.parent_id)}</strong></p>
						{/if}

						<div class="keywords-section">
							<h3>Keywords <span class="count">({selectedConcept.keywords.length})</span></h3>
							<p class="kw-help">Keywords are used to auto-detect this concept in communications.</p>
							<div class="keyword-list">
								{#each selectedConcept.keywords as kw}
									<span class="keyword-tag">
										{kw.keyword}
										<button class="kw-remove" on:click={() => removeKeyword(kw.id)} title="Remove">&times;</button>
									</span>
								{/each}
							</div>
							<div class="add-keyword">
								<input type="text" bind:value={newKeyword} placeholder="Add keyword..."
									on:keydown={(e) => e.key === 'Enter' && addKeyword()} />
								<button class="btn-primary btn-sm" on:click={addKeyword}>Add</button>
							</div>
						</div>
					</div>
				{:else}
					<div class="detail-placeholder">
						<p>Select a concept to view details and manage keywords.</p>
					</div>
				{/if}
			</div>
		</div>
	{/if}

	{#if showModal}
		<div class="modal-overlay" on:click|self={() => showModal = false}>
			<div class="modal">
				<h2>{editing ? 'Edit Concept' : 'New Concept'}</h2>
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
						Parent Concept
						<select bind:value={formParentId}>
							<option value="">None (root concept)</option>
							{#each concepts.filter(c => c.id !== editing?.id) as c}
								<option value={c.id}>{c.name}</option>
							{/each}
						</select>
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
	.page { padding: 1.5rem; max-width: 1400px; }
	.header { display: flex; align-items: center; gap: 1rem; flex-wrap: wrap; margin-bottom: 1.5rem; }
	.header h1 { margin: 0; font-size: 1.5rem; }
	.subtitle { color: #6b7280; margin: 0; flex: 1; }
	.loading { color: #6b7280; }
	.empty-state { text-align: center; padding: 3rem; color: #6b7280; background: #f9fafb; border-radius: 8px; }

	.layout { display: grid; grid-template-columns: 1fr 1fr; gap: 1.5rem; }
	@media (max-width: 768px) { .layout { grid-template-columns: 1fr; } }

	/* Tree */
	.tree-panel { background: white; border: 1px solid #e5e7eb; border-radius: 8px; padding: 1rem; max-height: 70vh; overflow-y: auto; }
	.tree-node { margin-bottom: 2px; }
	.children { margin-left: 1.25rem; border-left: 2px solid #e5e7eb; padding-left: 0.5rem; }
	.node-row { display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem 0.75rem; border-radius: 6px; cursor: pointer; }
	.node-row:hover { background: #f3f4f6; }
	.node-row.selected { background: #dbeafe; }
	.node-name { flex: 1; font-weight: 500; font-size: 0.9rem; }
	.kw-count { font-size: 0.7rem; color: #9ca3af; background: #f3f4f6; padding: 1px 6px; border-radius: 8px; }
	.node-actions { display: flex; gap: 0.25rem; opacity: 0; }
	.node-row:hover .node-actions { opacity: 1; }
	.btn-xs { padding: 1px 6px; font-size: 0.7rem; border: 1px solid #d1d5db; background: white; border-radius: 3px; cursor: pointer; }
	.btn-xs:hover { background: #f3f4f6; }
	.btn-xs.btn-danger { color: #dc2626; border-color: #fca5a5; }

	/* Detail panel */
	.detail-panel { }
	.detail-card { background: white; border: 1px solid #e5e7eb; border-radius: 8px; padding: 1.5rem; }
	.detail-card h2 { margin: 0 0 0.5rem; font-size: 1.25rem; }
	.detail-desc { color: #6b7280; margin: 0 0 0.5rem; }
	.detail-parent { font-size: 0.85rem; color: #6b7280; margin: 0 0 1rem; }
	.detail-placeholder { text-align: center; padding: 3rem; color: #9ca3af; background: #f9fafb; border-radius: 8px; }

	/* Keywords */
	.keywords-section { margin-top: 1.25rem; border-top: 1px solid #e5e7eb; padding-top: 1rem; }
	.keywords-section h3 { margin: 0 0 0.25rem; font-size: 1rem; }
	.keywords-section .count { color: #9ca3af; font-weight: normal; }
	.kw-help { font-size: 0.8rem; color: #9ca3af; margin: 0 0 0.75rem; }
	.keyword-list { display: flex; flex-wrap: wrap; gap: 0.5rem; margin-bottom: 0.75rem; }
	.keyword-tag { display: inline-flex; align-items: center; gap: 0.25rem; background: #eff6ff; color: #1d4ed8; padding: 4px 10px; border-radius: 16px; font-size: 0.8rem; }
	.kw-remove { background: none; border: none; color: #93c5fd; cursor: pointer; font-size: 1rem; padding: 0; line-height: 1; }
	.kw-remove:hover { color: #dc2626; }
	.add-keyword { display: flex; gap: 0.5rem; }
	.add-keyword input { flex: 1; padding: 0.375rem 0.75rem; border: 1px solid #d1d5db; border-radius: 6px; font-size: 0.85rem; }
	.btn-sm { padding: 0.375rem 0.75rem; font-size: 0.8rem; }

	/* Modal */
	.btn-primary { background: #3b82f6; color: white; border: none; padding: 0.5rem 1rem; border-radius: 6px; cursor: pointer; font-size: 0.875rem; }
	.btn-primary:hover { background: #2563eb; }
	.btn-secondary { background: #e5e7eb; color: #374151; border: none; padding: 0.5rem 1rem; border-radius: 6px; cursor: pointer; }
	.modal-overlay { position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 50; }
	.modal { background: white; border-radius: 12px; padding: 1.5rem; width: 90%; max-width: 500px; max-height: 90vh; overflow-y: auto; }
	.modal h2 { margin: 0 0 1rem; }
	.modal label { display: block; margin-bottom: 0.75rem; font-size: 0.875rem; font-weight: 500; }
	.modal input, .modal textarea, .modal select { width: 100%; padding: 0.5rem; border: 1px solid #d1d5db; border-radius: 6px; margin-top: 0.25rem; font-size: 0.875rem; box-sizing: border-box; }
	.modal-actions { display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 1rem; }
</style>
