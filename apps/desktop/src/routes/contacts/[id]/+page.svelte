<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { tauriApi } from '$lib/api/tauri-api';
	import type { Contact, Note, Communication } from '$lib/api/types';

	let contact: Contact | null = null;
	let notes: Note[] = [];
	let communications: Communication[] = [];
	let loading = true;
	let editing = false;

	// Form fields
	let formFirstName = '';
	let formLastName = '';
	let formEmail = '';
	let formPhone = '';
	let formOrganization = '';
	let formTitle = '';
	let formNotesText = '';

	$: contactId = $page.params.id;

	onMount(async () => {
		await loadContact();
	});

	async function loadContact() {
		try {
			loading = true;
			contact = await tauriApi.getContact(contactId);

			// Load notes
			try {
				notes = await tauriApi.getNotes('Contact', contactId);
			} catch (e) {
				console.warn('Failed to load notes:', e);
				notes = [];
			}

			// Load communications
			try {
				communications = await tauriApi.getCommunications(contactId);
			} catch (e) {
				console.warn('Failed to load communications:', e);
				communications = [];
			}

			resetForm();
		} catch (error: any) {
			console.error('Failed to load contact:', error);
			alert(`Error loading contact: ${error.message || error}`);
			goto('/contacts');
		} finally {
			loading = false;
		}
	}

	function resetForm() {
		if (!contact) return;
		formFirstName = contact.first_name;
		formLastName = contact.last_name || '';
		formEmail = contact.email || '';
		formPhone = contact.phone || '';
		formOrganization = contact.organization || '';
		formTitle = contact.title || '';
		formNotesText = contact.notes || '';
	}

	async function saveContact() {
		if (!contact) return;

		try {
			const updates = {
				first_name: formFirstName,
				last_name: formLastName,
				email: formEmail,
				phone: formPhone,
				organization: formOrganization,
				title: formTitle,
				notes: formNotesText
			};

			contact = await tauriApi.updateContact(contact.id, updates);
			editing = false;
			await tauriApi.showNotification('Contact Updated', `${formFirstName} has been updated`);
		} catch (error: any) {
			console.error('Failed to update contact:', error);
			alert(`Failed to update contact: ${error.message || error}`);
		}
	}

	function cancelEdit() {
		editing = false;
		resetForm();
	}

	async function deleteContact() {
		if (!contact) return;
		if (!confirm(`Are you sure you want to delete ${contact.first_name}?`)) return;

		try {
			await tauriApi.deleteContact(contact.id);
			await tauriApi.showNotification('Contact Deleted', `${contact.first_name} has been deleted`);
			goto('/contacts');
		} catch (error) {
			console.error('Failed to delete contact:', error);
			alert('Failed to delete contact');
		}
	}
</script>

{#if loading}
	<div class="page">
		<div class="loading">Loading contact...</div>
	</div>
{:else if contact}
	<div class="page">
		<div class="header">
			<div class="breadcrumb">
				<a href="/contacts">← Back to Contacts</a>
			</div>
			<div class="actions">
				{#if !editing}
					<button on:click={() => editing = true} class="btn btn-primary">
						✏️ Edit
					</button>
					<button on:click={deleteContact} class="btn btn-danger">
						🗑️ Delete
					</button>
				{:else}
					<button on:click={saveContact} class="btn btn-primary">
						💾 Save
					</button>
					<button on:click={cancelEdit} class="btn btn-secondary">
						❌ Cancel
					</button>
				{/if}
			</div>
		</div>

		<div class="content">
			<div class="main-section">
				<div class="card">
					<h2>Contact Information</h2>

					{#if !editing}
						<div class="info-grid">
							<div class="info-item">
								<span class="label">Name</span>
								<span class="value">{contact.first_name} {contact.last_name || ''}</span>
							</div>
							{#if contact.email}
								<div class="info-item">
									<span class="label">Email</span>
									<span class="value">{contact.email}</span>
								</div>
							{/if}
							{#if contact.phone}
								<div class="info-item">
									<span class="label">Phone</span>
									<span class="value">{contact.phone}</span>
								</div>
							{/if}
							{#if contact.organization}
								<div class="info-item">
									<span class="label">Organization</span>
									<span class="value">{contact.organization}</span>
								</div>
							{/if}
							{#if contact.title}
								<div class="info-item">
									<span class="label">Title</span>
									<span class="value">{contact.title}</span>
								</div>
							{/if}
							{#if contact.notes}
								<div class="info-item">
									<span class="label">Notes</span>
									<span class="value">{contact.notes}</span>
								</div>
							{/if}
						</div>
					{:else}
						<div class="form-grid">
							<div class="form-group">
								<label for="firstName">First Name *</label>
								<input type="text" id="firstName" bind:value={formFirstName} required />
							</div>
							<div class="form-group">
								<label for="lastName">Last Name</label>
								<input type="text" id="lastName" bind:value={formLastName} />
							</div>
							<div class="form-group">
								<label for="email">Email</label>
								<input type="email" id="email" bind:value={formEmail} />
							</div>
							<div class="form-group">
								<label for="phone">Phone</label>
								<input type="tel" id="phone" bind:value={formPhone} />
							</div>
							<div class="form-group">
								<label for="organization">Organization</label>
								<input type="text" id="organization" bind:value={formOrganization} />
							</div>
							<div class="form-group">
								<label for="title">Title</label>
								<input type="text" id="title" bind:value={formTitle} />
							</div>
							<div class="form-group full-width">
								<label for="notes">Notes</label>
								<textarea id="notes" bind:value={formNotesText} rows="4"></textarea>
							</div>
						</div>
					{/if}
				</div>

				{#if notes.length > 0}
					<div class="card">
						<h2>Notes ({notes.length})</h2>
						<div class="notes-list">
							{#each notes as note}
								<div class="note-item">
									<h4>{note.title}</h4>
									<p>{note.content}</p>
									<span class="note-date">{new Date(note.created_at).toLocaleDateString()}</span>
								</div>
							{/each}
						</div>
					</div>
				{/if}

				{#if communications.length > 0}
					<div class="card">
						<h2>Communications ({communications.length})</h2>
						<div class="communications-list">
							{#each communications.slice(0, 20) as comm}
								<div class="comm-item {comm.direction.toLowerCase()}">
									<div class="comm-header">
										<span class="comm-type">
											{#if comm.communication_type === 'Sms'}📱 SMS
											{:else if comm.communication_type === 'Call'}📞 Call
											{:else if comm.communication_type === 'Email'}📧 Email
											{:else}{comm.communication_type}
											{/if}
										</span>
										<span class="comm-direction {comm.direction.toLowerCase()}">
											{#if comm.direction === 'Inbound'}⬇️ Inbound
											{:else if comm.direction === 'Outbound'}⬆️ Outbound
											{:else}❌ Missed
											{/if}
										</span>
										<span class="comm-date">{new Date(comm.timestamp).toLocaleString()}</span>
									</div>
									{#if comm.content}
										<p class="comm-content">{comm.content}</p>
									{/if}
									{#if comm.duration_seconds}
										<span class="comm-duration">Duration: {Math.floor(comm.duration_seconds / 60)}m {comm.duration_seconds % 60}s</span>
									{/if}
								</div>
							{/each}
							{#if communications.length > 20}
								<p class="more-info">Showing 20 of {communications.length} communications</p>
							{/if}
						</div>
					</div>
				{/if}
			</div>

			<div class="sidebar">
				<div class="card">
					<h3>Quick Actions</h3>
					<div class="action-list">
						<a href="/communications?contact={contact.id}" class="action-btn">
							📧 Send Email
						</a>
						<a href="/communications?contact={contact.id}" class="action-btn">
							📱 Send SMS
						</a>
						<a href="/notes?contact={contact.id}" class="action-btn">
							📝 Add Note
						</a>
					</div>
				</div>

				{#if contact.tags && contact.tags.length > 0}
					<div class="card">
						<h3>Tags</h3>
						<div class="tags">
							{#each contact.tags as tag}
								<span class="tag">{tag}</span>
							{/each}
						</div>
					</div>
				{/if}

				<div class="card">
					<h3>Metadata</h3>
					<div class="metadata">
						<div class="metadata-item">
							<span class="label">Created</span>
							<span>{new Date(contact.created_at).toLocaleString()}</span>
						</div>
						<div class="metadata-item">
							<span class="label">Updated</span>
							<span>{new Date(contact.updated_at).toLocaleString()}</span>
						</div>
					</div>
				</div>
			</div>
		</div>
	</div>
{/if}

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

	.breadcrumb a {
		color: var(--gray-600);
		text-decoration: none;
		font-size: 0.875rem;
	}

	.breadcrumb a:hover {
		color: var(--primary);
	}

	.actions {
		display: flex;
		gap: 0.75rem;
	}

	.content {
		display: grid;
		grid-template-columns: 1fr 300px;
		gap: 2rem;
	}

	.main-section {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.sidebar {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.card {
		background: white;
		border-radius: 8px;
		box-shadow: var(--shadow-sm);
		padding: 1.5rem;
	}

	.card h2 {
		margin: 0 0 1.5rem;
		font-size: 1.5rem;
		color: var(--gray-900);
	}

	.card h3 {
		margin: 0 0 1rem;
		font-size: 1.125rem;
		color: var(--gray-900);
	}

	.info-grid {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.info-item {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.info-item .label {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--gray-600);
	}

	.info-item .value {
		font-size: 1rem;
		color: var(--gray-900);
	}

	.form-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1rem;
	}

	.form-group {
		display: flex;
		flex-direction: column;
	}

	.form-group.full-width {
		grid-column: 1 / -1;
	}

	.form-group label {
		margin-bottom: 0.5rem;
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--gray-700);
	}

	.form-group input,
	.form-group textarea {
		padding: 0.75rem;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		font-size: 1rem;
	}

	.form-group input:focus,
	.form-group textarea:focus {
		outline: none;
		border-color: var(--primary);
		box-shadow: 0 0 0 3px var(--primary-light);
	}

	.action-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.action-btn {
		padding: 0.75rem 1rem;
		background: var(--gray-50);
		border: 1px solid var(--gray-200);
		border-radius: 6px;
		text-decoration: none;
		color: var(--gray-900);
		transition: all 0.2s;
		text-align: left;
	}

	.action-btn:hover {
		background: var(--gray-100);
		border-color: var(--gray-300);
	}

	.tags {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
	}

	.tag {
		padding: 0.25rem 0.75rem;
		background: var(--primary-light);
		color: var(--primary);
		border-radius: 999px;
		font-size: 0.75rem;
		font-weight: 500;
	}

	.metadata {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.metadata-item {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		font-size: 0.875rem;
	}

	.metadata-item .label {
		font-weight: 500;
		color: var(--gray-600);
	}

	.notes-list {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.note-item {
		padding: 1rem;
		background: var(--gray-50);
		border-radius: 6px;
		border-left: 3px solid var(--primary);
	}

	.note-item h4 {
		margin: 0 0 0.5rem;
		font-size: 1rem;
		color: var(--gray-900);
	}

	.note-item p {
		margin: 0 0 0.5rem;
		color: var(--gray-700);
		font-size: 0.875rem;
	}

	.note-date {
		font-size: 0.75rem;
		color: var(--gray-500);
	}

	.loading {
		text-align: center;
		padding: 3rem;
		font-size: 1.125rem;
		color: var(--gray-600);
	}

	.btn {
		padding: 0.75rem 1.5rem;
		border: none;
		border-radius: 6px;
		cursor: pointer;
		font-weight: 500;
		transition: all 0.2s;
	}

	.btn-primary {
		background: var(--primary);
		color: white;
	}

	.btn-primary:hover {
		background: var(--primary-dark);
	}

	.btn-secondary {
		background: var(--gray-200);
		color: var(--gray-900);
	}

	.btn-secondary:hover {
		background: var(--gray-300);
	}

	.btn-danger {
		background: var(--color-error-500);
		color: white;
	}

	.btn-danger:hover {
		background: var(--color-error-600);
	}

	@media (max-width: 768px) {
		.content {
			grid-template-columns: 1fr;
		}
	}

	.communications-list {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		max-height: 600px;
		overflow-y: auto;
	}

	.comm-item {
		padding: 1rem;
		background: var(--gray-50);
		border-radius: 6px;
		border-left: 3px solid var(--gray-400);
	}

	.comm-item.inbound {
		border-left-color: #3b82f6;
		background: #eff6ff;
	}

	.comm-item.outbound {
		border-left-color: #10b981;
		background: #ecfdf5;
	}

	.comm-item.missed {
		border-left-color: #ef4444;
		background: #fef2f2;
	}

	.comm-header {
		display: flex;
		gap: 1rem;
		align-items: center;
		margin-bottom: 0.5rem;
		flex-wrap: wrap;
	}

	.comm-type {
		font-weight: 600;
		font-size: 0.875rem;
		color: var(--gray-900);
	}

	.comm-direction {
		font-size: 0.75rem;
		padding: 0.125rem 0.5rem;
		border-radius: 999px;
		font-weight: 500;
	}

	.comm-direction.inbound {
		background: #dbeafe;
		color: #1e40af;
	}

	.comm-direction.outbound {
		background: #d1fae5;
		color: #065f46;
	}

	.comm-direction.missed {
		background: #fee2e2;
		color: #991b1b;
	}

	.comm-date {
		font-size: 0.75rem;
		color: var(--gray-500);
		margin-left: auto;
	}

	.comm-content {
		margin: 0.5rem 0;
		color: var(--gray-700);
		font-size: 0.875rem;
		line-height: 1.5;
		padding: 0.5rem;
		background: white;
		border-radius: 4px;
	}

	.comm-duration {
		font-size: 0.75rem;
		color: var(--gray-600);
		font-weight: 500;
	}

	.more-info {
		text-align: center;
		padding: 1rem;
		color: var(--gray-600);
		font-size: 0.875rem;
		font-style: italic;
	}
</style>
