<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { api } from '$lib/api/api';
	import type { Contact, Note, Project, Group, CalendarEvent, Attachment } from '$lib/api/types';

	let contact: Contact | null = null;
	let notes: Note[] = [];
	let projects: Project[] = [];
	let groups: Group[] = [];
	let events: CalendarEvent[] = [];
	let attachments: Attachment[] = [];
	let loading = true;
	let editing = false;

	// Form fields
	let formFirstName = '';
	let formLastName = '';
	let formEmail = '';
	let formPhone = '';
	let formOrganization = '';
	let formTitle = '';
	let formNotes = '';
	let formTags: string[] = [];
	let formSocialHandles: { platform: string; handle: string }[] = [];

	$: contactId = $page.params.id as string;

	onMount(async () => {
		await loadContact();
		setupWebSocket();
	});

	async function loadContact() {
		try {
			// Try to fetch the specific contact directly via API
			const response = await fetch(`/api/contacts/${contactId}`, {
				headers: {
					'Authorization': `Bearer ${localStorage.getItem('auth_token')}`
				}
			});

			if (!response.ok) {
				console.error(`Failed to fetch contact ${contactId}: ${response.status}`);
				alert(`Contact not found (ID: ${contactId})`);
				goto('/contacts');
				return;
			}

			contact = await response.json();
			console.log('Loaded contact:', contact);

			// Load related data with graceful error handling
			try {
				notes = await api.getContactNotes(contactId);
			} catch (e) {
				console.warn('Failed to load notes:', e);
				notes = [];
			}

			try {
				const allProjects = await api.getProjects();
				projects = allProjects.filter(p => p.contacts.includes(contactId));
			} catch (e) {
				console.warn('Failed to load projects:', e);
				projects = [];
			}

			try {
				const allGroups = await api.getGroups();
				groups = allGroups.filter(g => g.contact_ids.includes(contactId));
			} catch (e) {
				console.warn('Failed to load groups:', e);
				groups = [];
			}

			try {
				const allEvents = await api.getEvents();
				events = allEvents.filter(e => e.contacts.includes(contactId));
			} catch (e) {
				console.warn('Failed to load events:', e);
				events = [];
			}

			try {
				attachments = await api.getAttachments('Contact', contactId);
			} catch (e) {
				console.warn('Failed to load attachments:', e);
				attachments = [];
			}

			resetForm();
		} catch (error: unknown) {
			console.error('Failed to load contact:', error);
			const message = error instanceof Error ? error.message : String(error);
			alert(`Error loading contact: ${message}`);
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
		formNotes = contact.notes || '';
		formTags = [...contact.tags];
		formSocialHandles = contact.social_handles.map(sh => ({
			platform: typeof sh.platform === 'string' ? sh.platform : 'Twitter',
			handle: sh.handle
		}));
	}

	function setupWebSocket() {
		api.onWebSocketMessage('contact_updated', (data) => {
			const updated = data as Contact;
			if (updated.id === contactId) {
				contact = updated;
				resetForm();
			}
		});
		api.onWebSocketMessage('contact_deleted', (data) => {
			const deleted = data as { id: string };
			if (deleted.id === contactId) {
				alert('Contact was deleted');
				goto('/contacts');
			}
		});
	}

	async function handleSave() {
		if (!contact || !formFirstName.trim()) {
			alert('First name is required');
			return;
		}

		try {
			const updated = await api.updateContact(contactId, {
				first_name: formFirstName,
				last_name: formLastName || undefined,
				email: formEmail || undefined,
				phone: formPhone || undefined,
				organization: formOrganization || undefined,
				title: formTitle || undefined,
				notes: formNotes || undefined,
				tags: formTags,
				social_handles: formSocialHandles.filter(sh => sh.handle.trim()).map(sh => ({
					platform: sh.platform,
					handle: sh.handle
				}))
			});
			contact = updated;
			editing = false;
		} catch (error: unknown) {
			alert('Failed to save: ' + (error instanceof Error ? error.message : String(error)));
		}
	}

	async function handleDelete() {
		if (!confirm('Delete this contact permanently?')) return;
		try {
			await api.deleteContact(contactId);
			goto('/contacts');
		} catch (error: unknown) {
			alert('Failed to delete: ' + (error instanceof Error ? error.message : String(error)));
		}
	}

	function addSocialHandle() {
		formSocialHandles = [...formSocialHandles, { platform: 'Twitter', handle: '' }];
	}

	function removeSocialHandle(index: number) {
		formSocialHandles = formSocialHandles.filter((_, i) => i !== index);
	}

	function addTag() {
		const tag = prompt('Enter tag:');
		if (tag && tag.trim()) {
			formTags = [...formTags, tag.trim()];
		}
	}

	function removeTag(tag: string) {
		formTags = formTags.filter(t => t !== tag);
	}

	async function handleUploadAttachment(event: Event) {
		const input = event.target as HTMLInputElement;
		if (!input.files || input.files.length === 0) return;

		try {
			for (const file of Array.from(input.files)) {
				await api.uploadAttachment(file, 'Contact', contactId);
			}
			attachments = await api.getAttachments('Contact', contactId);
			input.value = '';
		} catch (error: unknown) {
			alert('Failed to upload: ' + (error instanceof Error ? error.message : String(error)));
		}
	}

	async function handleDownloadAttachment(id: string, filename: string) {
		try {
			// downloadAttachment handles the download internally
			await api.downloadAttachment(id, filename);
		} catch (error: unknown) {
			alert('Failed to download: ' + (error instanceof Error ? error.message : String(error)));
		}
	}
</script>

<div class="page">
	{#if loading}
		<div>Loading...</div>
	{:else if !contact}
		<div>Contact not found</div>
	{:else}
		<div class="header">
			<div>
				<h1>{contact.first_name} {contact.last_name || ''}</h1>
				{#if contact.title || contact.organization}
					<p class="subtitle">{contact.title || ''}{contact.title && contact.organization ? ' at ' : ''}{contact.organization || ''}</p>
				{/if}
			</div>
			<div class="header-actions">
				{#if !editing}
					<button on:click={() => editing = true} class="btn btn-primary">Edit</button>
					<button on:click={handleDelete} class="btn btn-danger">Delete</button>
				{:else}
					<button on:click={handleSave} class="btn btn-primary">Save</button>
					<button on:click={() => { editing = false; resetForm(); }} class="btn btn-secondary">Cancel</button>
				{/if}
			</div>
		</div>

		<div class="content-grid">
			<div class="main-content">
				{#if editing}
					<div class="card">
						<h2>Edit Contact</h2>
						<div class="form-group">
							<label for="edit-first-name">First Name *</label>
							<input id="edit-first-name" type="text" bind:value={formFirstName} required />
						</div>
						<div class="form-group">
							<label for="edit-last-name">Last Name</label>
							<input id="edit-last-name" type="text" bind:value={formLastName} />
						</div>
						<div class="form-group">
							<label for="edit-email">Email</label>
							<input id="edit-email" type="email" bind:value={formEmail} />
						</div>
						<div class="form-group">
							<label for="edit-phone">Phone</label>
							<input id="edit-phone" type="tel" bind:value={formPhone} />
						</div>
						<div class="form-group">
							<label for="edit-organization">Organization</label>
							<input id="edit-organization" type="text" bind:value={formOrganization} />
						</div>
						<div class="form-group">
							<label for="edit-title">Title</label>
							<input id="edit-title" type="text" bind:value={formTitle} />
						</div>
						<div class="form-group">
							<label for="edit-notes">Notes</label>
							<textarea id="edit-notes" bind:value={formNotes} rows="4"></textarea>
						</div>
						<div class="form-group">
							<span class="form-label-text">Social Handles</span>
							{#each formSocialHandles as handle, i}
								<div class="social-handle-row">
									<select bind:value={handle.platform}>
										<option value="Twitter">Twitter</option>
										<option value="LinkedIn">LinkedIn</option>
										<option value="Facebook">Facebook</option>
										<option value="Instagram">Instagram</option>
										<option value="GitHub">GitHub</option>
									</select>
									<input type="text" bind:value={handle.handle} placeholder="@username" />
									<button type="button" on:click={() => removeSocialHandle(i)} class="btn btn-sm btn-danger">Remove</button>
								</div>
							{/each}
							<button type="button" on:click={addSocialHandle} class="btn btn-sm btn-secondary">Add Social Handle</button>
						</div>
						<div class="form-group">
							<span class="form-label-text">Tags</span>
							<div class="tags">
								{#each formTags as tag}
									<span class="tag">
										{tag}
										<button type="button" on:click={() => removeTag(tag)} class="tag-remove">×</button>
									</span>
								{/each}
								<button type="button" on:click={addTag} class="btn btn-sm btn-secondary">Add Tag</button>
							</div>
						</div>
					</div>
				{:else}
					<div class="card">
						<h2>Contact Information</h2>
						<div class="info-grid">
							{#if contact.email}
								<div class="info-item">
									<span class="label">Email:</span>
									<a href="mailto:{contact.email}">{contact.email}</a>
								</div>
							{/if}
							{#if contact.phone}
								<div class="info-item">
									<span class="label">Phone:</span>
									<a href="tel:{contact.phone}">{contact.phone}</a>
								</div>
							{/if}
							{#if contact.social_handles.length > 0}
								<div class="info-item">
									<span class="label">Social:</span>
									<div class="social-handles">
										{#each contact.social_handles as handle}
											<span class="social-handle">{typeof handle.platform === 'string' ? handle.platform : 'Social'}: @{handle.handle}</span>
										{/each}
									</div>
								</div>
							{/if}
							{#if contact.tags.length > 0}
								<div class="info-item">
									<span class="label">Tags:</span>
									<div class="tags">
										{#each contact.tags as tag}
											<span class="tag">{tag}</span>
										{/each}
									</div>
								</div>
							{/if}
							{#if contact.notes}
								<div class="info-item full-width">
									<span class="label">Notes:</span>
									<p class="notes-text">{contact.notes}</p>
								</div>
							{/if}
						</div>
					</div>

					<div class="card">
						<h2>Projects ({projects.length})</h2>
						{#if projects.length === 0}
							<p class="empty-state">Not part of any projects</p>
						{:else}
							<div class="related-list">
								{#each projects as project}
									<div class="related-item">
										<a href="/projects/{project.id}">{project.name}</a>
										<span class="badge status-{project.status.toLowerCase()}">{project.status}</span>
									</div>
								{/each}
							</div>
						{/if}
					</div>

					<div class="card">
						<h2>Groups ({groups.length})</h2>
						{#if groups.length === 0}
							<p class="empty-state">Not part of any groups</p>
						{:else}
							<div class="related-list">
								{#each groups as group}
									<div class="related-item">
										<a href="/groups/{group.id}">{group.name}</a>
										<span class="meta">{group.contact_ids.length} members</span>
									</div>
								{/each}
							</div>
						{/if}
					</div>

					<div class="card">
						<h2>Upcoming Events ({events.length})</h2>
						{#if events.length === 0}
							<p class="empty-state">No upcoming events</p>
						{:else}
							<div class="related-list">
								{#each events as event}
									<div class="related-item">
										<a href="/calendar?event={event.id}">{event.title}</a>
										<span class="meta">{new Date(event.start_time).toLocaleDateString()}</span>
									</div>
								{/each}
							</div>
						{/if}
					</div>
				{/if}
			</div>

			<div class="sidebar">
				<div class="card">
					<h2>Notes ({notes.length})</h2>
					{#if notes.length === 0}
						<p class="empty-state">No notes</p>
					{:else}
						<div class="notes-list">
							{#each notes as note}
								<div class="note-item">
									{#if note.title}
										<h4>{note.title}</h4>
									{/if}
									<p class="note-content">{note.content}</p>
									<span class="note-date">{new Date(note.created_at).toLocaleDateString()}</span>
								</div>
							{/each}
						</div>
					{/if}
				</div>

				<div class="card">
					<div class="card-header">
						<h2>Attachments ({attachments.length})</h2>
						<label class="btn btn-sm btn-secondary file-upload-btn">
							<input type="file" multiple on:change={handleUploadAttachment} style="display: none;" />
							Upload
						</label>
					</div>
					{#if attachments.length === 0}
						<p class="empty-state">No attachments</p>
					{:else}
						<div class="attachments-list">
							{#each attachments as attachment}
								<div class="attachment-item">
									<span class="attachment-name">{attachment.filename}</span>
									<button
										on:click={() => handleDownloadAttachment(attachment.id, attachment.filename)}
										class="btn btn-sm btn-link"
									>
										Download
									</button>
								</div>
							{/each}
						</div>
					{/if}
				</div>
			</div>
		</div>
	{/if}
</div>

<style>
	.page { padding: 2rem; max-width: 1400px; margin: 0 auto; }
	.header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 2rem; }
	.subtitle { color: var(--gray-600); margin-top: 0.5rem; font-size: 1.125rem; }
	.header-actions { display: flex; gap: 0.5rem; }

	.content-grid { display: grid; grid-template-columns: 1fr 350px; gap: 1.5rem; }
	.main-content { display: flex; flex-direction: column; gap: 1.5rem; }
	.sidebar { display: flex; flex-direction: column; gap: 1.5rem; }

	.card { padding: 1.5rem; background: white; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
	.card h2 { margin: 0 0 1rem; font-size: 1.25rem; }
	.card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
	.card-header h2 { margin: 0; }

	.info-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }
	.info-item { display: flex; flex-direction: column; gap: 0.25rem; }
	.info-item.full-width { grid-column: 1 / -1; }
	.label { font-weight: 600; font-size: 0.875rem; color: var(--gray-600); }
	.notes-text { color: var(--gray-700); margin: 0.5rem 0 0; white-space: pre-wrap; }

	.social-handles { display: flex; flex-direction: column; gap: 0.25rem; }
	.social-handle { font-size: 0.875rem; color: var(--gray-700); }

	.tags { display: flex; flex-wrap: wrap; gap: 0.5rem; align-items: center; }
	.tag { display: inline-flex; align-items: center; gap: 0.25rem; padding: 0.25rem 0.75rem; background: var(--gray-100); border-radius: 12px; font-size: 0.875rem; }
	.tag-remove { background: none; border: none; cursor: pointer; font-size: 1.25rem; line-height: 1; padding: 0; color: var(--gray-600); }
	.tag-remove:hover { color: var(--gray-900); }

	.form-group { margin-bottom: 1.5rem; }
	.form-group label { display: block; margin-bottom: 0.5rem; font-weight: 500; }
	.form-label-text { display: block; margin-bottom: 0.5rem; font-weight: 500; }
	.form-group input, .form-group select, .form-group textarea {
		width: 100%;
		padding: 0.75rem;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		font-size: 1rem;
	}
	.form-group textarea { resize: vertical; font-family: inherit; }

	.social-handle-row { display: flex; gap: 0.5rem; margin-bottom: 0.5rem; align-items: center; }
	.social-handle-row select { width: 140px; }
	.social-handle-row input { flex: 1; }

	.related-list { display: flex; flex-direction: column; gap: 0.5rem; }
	.related-item { display: flex; justify-content: space-between; align-items: center; padding: 0.75rem; background: var(--gray-50); border-radius: 6px; }
	.related-item a { color: var(--primary); text-decoration: none; font-weight: 500; }
	.related-item a:hover { text-decoration: underline; }
	.meta { font-size: 0.875rem; color: var(--gray-600); }

	.badge { padding: 0.25rem 0.75rem; border-radius: 12px; font-size: 0.75rem; font-weight: 600; }
	.status-active { background: #d1fae5; color: #065f46; }
	.status-completed { background: #e0e7ff; color: #3730a3; }
	.status-onhold { background: #fed7aa; color: #92400e; }
	.status-archived { background: #e5e7eb; color: #374151; }

	.notes-list { display: flex; flex-direction: column; gap: 0.75rem; }
	.note-item { padding: 0.75rem; background: var(--gray-50); border-radius: 6px; }
	.note-item h4 { margin: 0 0 0.25rem; font-size: 0.875rem; }
	.note-content { margin: 0.25rem 0; font-size: 0.875rem; color: var(--gray-700); }
	.note-date { font-size: 0.75rem; color: var(--gray-500); }

	.attachments-list { display: flex; flex-direction: column; gap: 0.5rem; }
	.attachment-item { display: flex; justify-content: space-between; align-items: center; padding: 0.5rem; background: var(--gray-50); border-radius: 4px; }
	.attachment-name { font-size: 0.875rem; color: var(--gray-700); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

	.empty-state { color: var(--gray-500); font-size: 0.875rem; padding: 1rem 0; text-align: center; }

	.btn { padding: 0.75rem 1.5rem; border: none; border-radius: 6px; cursor: pointer; font-weight: 500; }
	.btn-primary { background: var(--primary); color: white; }
	.btn-secondary { background: var(--gray-200); color: var(--gray-800); }
	.btn-danger { background: #dc2626; color: white; }
	.btn-link { background: none; color: var(--primary); padding: 0.25rem 0.5rem; }
	.btn-sm { padding: 0.375rem 0.75rem; font-size: 0.875rem; }
	.file-upload-btn { display: inline-block; cursor: pointer; }

	@media (max-width: 1024px) {
		.content-grid { grid-template-columns: 1fr; }
	}
</style>
