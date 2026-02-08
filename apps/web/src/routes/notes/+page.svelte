<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { Note, Attachment, Contact, Project } from '$lib/api/types';

	let notes: Note[] = [];
	let contacts: Contact[] = [];
	let projects: Project[] = [];
	let attachments = new Map<string, Attachment[]>();
	let loading = true;
	let showModal = false;
	let editingNote: Note | null = null;

	let formTitle = '';
	let formContent = '';
	let formContactId = '';
	let formProjectId = '';
	let formFiles: File[] = [];

	onMount(async () => {
		await loadNotes();
		await loadContacts();
		await loadProjects();
		setupWebSocket();
	});

	async function loadNotes() {
		try {
			loading = true;
			notes = await api.getNotes();
		} catch (error) {
			console.error('Failed to load notes:', error);
		} finally {
			loading = false;
		}
	}

	async function loadContacts() {
		try {
			contacts = await api.getContacts(500, 0);
		} catch (error) {
			console.error('Failed to load contacts:', error);
		}
	}

	async function loadProjects() {
		try {
			projects = await api.getProjects();
		} catch (error) {
			console.error('Failed to load projects:', error);
		}
	}

	function setupWebSocket() {
		api.onWebSocketMessage('note_updated', (data) => {
			const updated = data as Note;
			const index = notes.findIndex(n => n.id === updated.id);
			if (index >= 0) notes[index] = updated;
			notes = notes;
		});
		api.onWebSocketMessage('note_created', (data) => {
			notes = [data as Note, ...notes];
		});
		api.onWebSocketMessage('note_deleted', (data) => {
			const deleted = data as { id: string };
			notes = notes.filter(n => n.id !== deleted.id);
		});
	}

	function openCreate() {
		editingNote = null;
		formTitle = '';
		formContent = '';
		formContactId = '';
		formProjectId = '';
		formFiles = [];
		showModal = true;
	}

	function openEdit(note: Note) {
		editingNote = note;
		formTitle = note.title;
		formContent = note.content;
		formContactId = note.contact_id || '';
		formProjectId = note.project_id || '';
		formFiles = [];
		showModal = true;
	}

	async function handleSubmit() {
		try {
			const noteData = {
				title: formTitle,
				content: formContent,
				contact_id: formContactId || undefined,
				project_id: formProjectId || undefined
			};

			if (editingNote) {
				const updated = await api.updateNote(editingNote.id, noteData);
				notes = notes.map(n => n.id === editingNote!.id ? updated : n);
			} else {
				const created = await api.createNote(noteData);
				notes = [created, ...notes];

				if (formFiles.length > 0) {
					for (const file of formFiles) {
						await api.uploadAttachment(file, 'Note', created.id);
					}
				}
			}
			showModal = false;
		} catch (error: unknown) {
			alert('Failed to save note: ' + (error instanceof Error ? error.message : String(error)));
		}
	}

	async function handleDelete(id: string) {
		if (!confirm('Delete this note?')) return;
		try {
			await api.deleteNote(id);
			notes = notes.filter(n => n.id !== id);
		} catch (error: unknown) {
			alert('Failed to delete: ' + (error instanceof Error ? error.message : String(error)));
		}
	}

	function handleFileChange(e: Event) {
		const target = e.target as HTMLInputElement;
		if (target.files) formFiles = Array.from(target.files);
	}
</script>

<div class="page">
	<div class="header">
		<h1>Notes</h1>
		<button on:click={openCreate} class="btn btn-primary">+ New Note</button>
	</div>

	{#if loading}
		<div class="loading">Loading...</div>
	{:else if notes.length === 0}
		<div class="empty">
			<p>No notes yet</p>
			<button on:click={openCreate} class="btn btn-primary">Create first note</button>
		</div>
	{:else}
		<div class="notes-grid">
			{#each notes as note}
				<div class="note-card card">
					<div class="note-header">
						<h3>{note.title}</h3>
						<div class="actions">
							<button on:click={() => openEdit(note)} class="btn btn-sm">Edit</button>
							<button on:click={() => handleDelete(note.id)} class="btn btn-sm">Delete</button>
						</div>
					</div>
					<div class="note-content">{note.content}</div>
					<div class="note-meta">
						{new Date(note.created_at).toLocaleDateString()}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

{#if showModal}
	<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
	<div class="modal-overlay" role="button" tabindex="0" on:click={() => showModal = false} on:keydown={(e) => e.key === 'Enter' && (showModal = false)}>
		<div class="modal" role="dialog" aria-modal="true" on:click|stopPropagation on:keydown|stopPropagation>
			<h2>{editingNote ? 'Edit' : 'New'} Note</h2>
			<form on:submit|preventDefault={handleSubmit}>
				<div class="form-group">
					<label for="note-title">Title</label>
					<input id="note-title" type="text" bind:value={formTitle} required />
				</div>
				<div class="form-group">
					<label for="note-content">Content</label>
					<textarea id="note-content" bind:value={formContent} rows="6" required></textarea>
				</div>
				<div class="form-group">
					<label for="note-contact">Contact</label>
					<select id="note-contact" bind:value={formContactId}>
						<option value="">None</option>
						{#each contacts as c}
							<option value={c.id}>{c.first_name} {c.last_name || ''}</option>
						{/each}
					</select>
				</div>
				<div class="form-group">
					<label for="note-project">Project</label>
					<select id="note-project" bind:value={formProjectId}>
						<option value="">None</option>
						{#each projects as p}
							<option value={p.id}>{p.name}</option>
						{/each}
					</select>
				</div>
				{#if !editingNote}
					<div class="form-group">
						<label for="note-attachments">Attachments</label>
						<input id="note-attachments" type="file" multiple on:change={handleFileChange} />
					</div>
				{/if}
				<div class="form-actions">
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
	.notes-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(350px, 1fr)); gap: 1.5rem; }
	.note-card { padding: 1.5rem; }
	.note-header { display: flex; justify-content: space-between; margin-bottom: 1rem; }
	.note-header h3 { margin: 0; }
	.actions { display: flex; gap: 0.5rem; }
	.note-content { color: var(--gray-700); line-height: 1.6; margin-bottom: 1rem; }
	.note-meta { font-size: 0.875rem; color: var(--gray-600); }
	.loading, .empty { text-align: center; padding: 3rem; }
	.modal-overlay { position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000; }
	.modal { background: white; padding: 2rem; border-radius: 8px; width: 90%; max-width: 600px; }
	.form-group { margin-bottom: 1.5rem; }
	.form-group label { display: block; margin-bottom: 0.5rem; font-weight: 500; }
	.form-group input, .form-group textarea, .form-group select { width: 100%; padding: 0.75rem; border: 1px solid var(--gray-300); border-radius: 6px; }
	.form-actions { display: flex; justify-content: flex-end; gap: 1rem; margin-top: 2rem; }
	.btn-sm { padding: 0.375rem 0.75rem; font-size: 0.875rem; }
</style>
