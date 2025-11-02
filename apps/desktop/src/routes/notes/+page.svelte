<script lang="ts">
	import { onMount } from 'svelte';
	import { tauriApi } from '$lib/api/tauri-api';
	import type { Note } from '$lib/api/types';

	let notes: Note[] = [];
	let loading = true;

	onMount(async () => {
		await loadNotes();
	});

	async function loadNotes() {
		try {
			loading = true;
			// Get all notes - the command returns empty for now since we only have list_by_contact
			// This is a placeholder implementation
			notes = await tauriApi.getNotes();
		} catch (error) {
			console.error('Failed to load notes:', error);
			notes = [];
		} finally {
			loading = false;
		}
	}
</script>

<div class="page">
	<div class="header">
		<div>
			<h1>Notes</h1>
			<p class="subtitle">View and manage all notes</p>
		</div>
	</div>

	{#if loading}
		<div class="loading">Loading notes...</div>
	{:else if notes.length === 0}
		<div class="empty card">
			<h2>📝 Notes Feature</h2>
			<p>Notes are displayed on individual contact and project detail pages.</p>
			<p>To view notes:</p>
			<ul>
				<li>Navigate to a contact from the Contacts page</li>
				<li>Notes attached to that contact will appear in the contact detail view</li>
			</ul>
			<p class="note">
				Currently, the global notes list is not yet implemented in the repository layer.
				Notes can be viewed per-contact or per-project.
			</p>
		</div>
	{:else}
		<div class="notes-list">
			{#each notes as note}
				<div class="note-card card">
					<div class="note-header">
						<h3>{note.title}</h3>
						<span class="note-date">{new Date(note.created_at).toLocaleDateString()}</span>
					</div>
					<p class="note-content">{note.content}</p>
					{#if note.contact_id}
						<div class="note-meta">
							<span class="meta-badge">👤 Contact Note</span>
						</div>
					{/if}
					{#if note.project_id}
						<div class="note-meta">
							<span class="meta-badge">📁 Project Note</span>
						</div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page {
		padding: 2rem;
		max-width: 1200px;
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
		padding: 3rem;
	}

	.empty h2 {
		margin: 0 0 1rem;
		font-size: 1.5rem;
		color: var(--gray-900);
	}

	.empty p {
		margin: 0 0 1rem;
		color: var(--gray-700);
		line-height: 1.6;
	}

	.empty ul {
		margin: 0 0 1.5rem;
		padding-left: 2rem;
		color: var(--gray-700);
		line-height: 1.8;
	}

	.empty li {
		margin-bottom: 0.5rem;
	}

	.note {
		padding: 1rem;
		background: #fef3c7;
		border-left: 4px solid #f59e0b;
		border-radius: 4px;
		font-size: 0.875rem;
		color: #92400e;
		margin-top: 1.5rem;
	}

	.notes-list {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.note-card {
		padding: 1.5rem;
	}

	.note-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	.note-header h3 {
		margin: 0;
		font-size: 1.25rem;
		color: var(--gray-900);
	}

	.note-date {
		font-size: 0.75rem;
		color: var(--gray-500);
	}

	.note-content {
		margin: 0 0 1rem;
		color: var(--gray-700);
		line-height: 1.6;
	}

	.note-meta {
		display: flex;
		gap: 0.5rem;
	}

	.meta-badge {
		padding: 0.25rem 0.75rem;
		background: var(--primary-light);
		color: var(--primary);
		border-radius: 999px;
		font-size: 0.75rem;
		font-weight: 500;
	}

	.card {
		background: white;
		border-radius: 8px;
		box-shadow: var(--shadow-sm);
	}
</style>
