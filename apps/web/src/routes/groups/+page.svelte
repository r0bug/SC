<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { Group, Contact } from '$lib/api/types';

	let groups: Group[] = [];
	let contacts: Contact[] = [];
	let loading = true;
	let showCreateModal = false;
	let showEditModal = false;
	let selectedGroup: Group | null = null;

	// Form state
	let formName = '';
	let formDescription = '';
	let formMembers: string[] = [];

	onMount(async () => {
		await Promise.all([loadGroups(), loadContacts()]);
		setupWebSocket();
	});

	async function loadGroups() {
		try {
			loading = true;
			groups = await api.getGroups();
		} catch (error: any) {
			console.error('Failed to load groups:', error);
		} finally {
			loading = false;
		}
	}

	async function loadContacts() {
		try {
			contacts = await api.getContacts(500, 0);
		} catch (error: any) {
			console.error('Failed to load contacts:', error);
		}
	}

	function setupWebSocket() {
		api.onWebSocketMessage('group_updated', (data) => {
			const index = groups.findIndex(g => g.id === data.id);
			if (index >= 0) {
				groups[index] = data;
				groups = groups;
			}
		});

		api.onWebSocketMessage('group_created', (data) => {
			groups = [data, ...groups];
		});

		api.onWebSocketMessage('group_deleted', (data) => {
			groups = groups.filter(g => g.id !== data.id);
		});
	}

	function openCreateModal() {
		formName = '';
		formDescription = '';
		formMembers = [];
		showCreateModal = true;
	}

	function openEditModal(group: Group) {
		selectedGroup = group;
		formName = group.name;
		formDescription = group.description || '';
		formMembers = [...group.contact_ids];
		showEditModal = true;
	}

	function closeModals() {
		showCreateModal = false;
		showEditModal = false;
		selectedGroup = null;
	}

	async function handleCreate() {
		try {
			const newGroup = await api.createGroup({
				name: formName,
				description: formDescription,
				contact_ids: formMembers
			} as any);

			groups = [newGroup, ...groups];
			closeModals();
		} catch (error: any) {
			alert('Failed to create group: ' + error.message);
		}
	}

	async function handleUpdate() {
		if (!selectedGroup) return;

		try {
			const updated = await api.updateGroup(selectedGroup.id, {
				name: formName,
				description: formDescription,
				contact_ids: formMembers
			});

			const index = groups.findIndex(g => g.id === selectedGroup!.id);
			if (index >= 0) {
				groups[index] = updated;
				groups = groups;
			}
			closeModals();
		} catch (error: any) {
			alert('Failed to update group: ' + error.message);
		}
	}

	async function handleDelete(id: string) {
		if (!confirm('Are you sure you want to delete this group?')) return;

		try {
			await api.deleteGroup(id);
			groups = groups.filter(g => g.id !== id);
		} catch (error: any) {
			alert('Failed to delete group: ' + error.message);
		}
	}

	function toggleMember(contactId: string) {
		if (formMembers.includes(contactId)) {
			formMembers = formMembers.filter(id => id !== contactId);
		} else {
			formMembers = [...formMembers, contactId];
		}
	}

	function getContactName(contactId: string): string {
		const contact = contacts.find(c => c.id === contactId);
		return contact ? `${contact.first_name} ${contact.last_name || ''}` : 'Unknown';
	}
</script>

<div class="page">
	<div class="header">
		<div>
			<h1>Groups</h1>
			<p class="subtitle">Organize contacts into groups</p>
		</div>
		<button on:click={openCreateModal} class="btn btn-primary">+ Create Group</button>
	</div>

	{#if loading}
		<div class="loading">Loading groups...</div>
	{:else if groups.length === 0}
		<div class="empty card">
			<p>No groups yet.</p>
			<button on:click={openCreateModal} class="btn btn-primary">Create your first group</button>
		</div>
	{:else}
		<div class="group-grid">
			{#each groups as group}
				<div class="group-card card">
					<div class="group-header">
						<div>
							<h3>{group.name}</h3>
							{#if group.description}
								<p class="description">{group.description}</p>
							{/if}
						</div>
						<div class="group-actions">
							<button on:click={() => openEditModal(group)} class="btn btn-sm">Edit</button>
							<button on:click={() => handleDelete(group.id)} class="btn btn-sm btn-secondary">
								Delete
							</button>
						</div>
					</div>

					<div class="group-members">
						<h4>Members ({group.contact_ids.length})</h4>
						{#if group.contact_ids.length > 0}
							<div class="member-list">
								{#each group.contact_ids.slice(0, 5) as contactId}
									<div class="member-badge">
										{getContactName(contactId)}
									</div>
								{/each}
								{#if group.contact_ids.length > 5}
									<div class="member-badge more">
										+{group.contact_ids.length - 5} more
									</div>
								{/if}
							</div>
						{:else}
							<p class="text-muted">No members</p>
						{/if}
					</div>

					<div class="group-meta">
						<span class="text-sm">Created {new Date(group.created_at).toLocaleDateString()}</span>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<!-- Create Modal -->
{#if showCreateModal}
	<div class="modal-overlay" on:click={closeModals}>
		<div class="modal" on:click|stopPropagation>
			<div class="modal-header">
				<h2>Create Group</h2>
				<button on:click={closeModals} class="close-btn">×</button>
			</div>

			<form on:submit|preventDefault={handleCreate}>
				<div class="form-group">
					<label for="name">Group Name *</label>
					<input id="name" type="text" bind:value={formName} required />
				</div>

				<div class="form-group">
					<label for="description">Description</label>
					<textarea id="description" bind:value={formDescription} rows="3"></textarea>
				</div>

				<div class="form-group">
					<label>Members</label>
					<div class="member-selector">
						{#each contacts as contact}
							<label class="member-option">
								<input
									type="checkbox"
									checked={formMembers.includes(contact.id)}
									on:change={() => toggleMember(contact.id)}
								/>
								<span>{contact.first_name} {contact.last_name || ''}</span>
								{#if contact.email}
									<span class="text-sm">({contact.email})</span>
								{/if}
							</label>
						{/each}
					</div>
				</div>

				<div class="modal-actions">
					<button type="button" on:click={closeModals} class="btn btn-secondary">
						Cancel
					</button>
					<button type="submit" class="btn btn-primary">Create Group</button>
				</div>
			</form>
		</div>
	</div>
{/if}

<!-- Edit Modal -->
{#if showEditModal && selectedGroup}
	<div class="modal-overlay" on:click={closeModals}>
		<div class="modal" on:click|stopPropagation>
			<div class="modal-header">
				<h2>Edit Group</h2>
				<button on:click={closeModals} class="close-btn">×</button>
			</div>

			<form on:submit|preventDefault={handleUpdate}>
				<div class="form-group">
					<label for="edit-name">Group Name *</label>
					<input id="edit-name" type="text" bind:value={formName} required />
				</div>

				<div class="form-group">
					<label for="edit-description">Description</label>
					<textarea id="edit-description" bind:value={formDescription} rows="3"></textarea>
				</div>

				<div class="form-group">
					<label>Members</label>
					<div class="member-selector">
						{#each contacts as contact}
							<label class="member-option">
								<input
									type="checkbox"
									checked={formMembers.includes(contact.id)}
									on:change={() => toggleMember(contact.id)}
								/>
								<span>{contact.first_name} {contact.last_name || ''}</span>
								{#if contact.email}
									<span class="text-sm">({contact.email})</span>
								{/if}
							</label>
						{/each}
					</div>
				</div>

				<div class="modal-actions">
					<button type="button" on:click={closeModals} class="btn btn-secondary">
						Cancel
					</button>
					<button type="submit" class="btn btn-primary">Update Group</button>
				</div>
			</form>
		</div>
	</div>
{/if}

<style>
	.page {
		padding: 2rem;
		max-width: 1200px;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 2rem;
	}

	.subtitle {
		color: var(--gray-600);
		margin-top: 0.25rem;
	}

	.group-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
		gap: 1.5rem;
	}

	.group-card {
		padding: 1.5rem;
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.group-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
	}

	.group-header h3 {
		margin: 0 0 0.5rem;
		font-size: 1.25rem;
	}

	.description {
		color: var(--gray-600);
		margin: 0;
		font-size: 0.875rem;
	}

	.group-actions {
		display: flex;
		gap: 0.5rem;
	}

	.group-members h4 {
		font-size: 0.875rem;
		margin-bottom: 0.75rem;
		color: var(--gray-700);
	}

	.member-list {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
	}

	.member-badge {
		padding: 0.375rem 0.75rem;
		background: var(--primary-light);
		color: var(--primary);
		border-radius: 12px;
		font-size: 0.75rem;
		font-weight: 500;
	}

	.member-badge.more {
		background: var(--gray-100);
		color: var(--gray-600);
	}

	.group-meta {
		padding-top: 0.75rem;
		border-top: 1px solid var(--gray-200);
	}

	.text-sm {
		font-size: 0.75rem;
		color: var(--gray-600);
	}

	.text-muted {
		color: var(--gray-500);
		font-size: 0.875rem;
	}

	.btn-sm {
		padding: 0.375rem 0.75rem;
		font-size: 0.875rem;
	}

	.loading, .empty {
		text-align: center;
		padding: 3rem;
	}

	/* Modal Styles */
	.modal-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		background: white;
		border-radius: 8px;
		padding: 2rem;
		width: 90%;
		max-width: 600px;
		max-height: 80vh;
		overflow-y: auto;
	}

	.modal-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1.5rem;
	}

	.modal-header h2 {
		margin: 0;
	}

	.close-btn {
		background: none;
		border: none;
		font-size: 2rem;
		line-height: 1;
		cursor: pointer;
		color: var(--gray-400);
		padding: 0;
		width: 2rem;
		height: 2rem;
	}

	.close-btn:hover {
		color: var(--gray-600);
	}

	.form-group {
		margin-bottom: 1.5rem;
	}

	.form-group label {
		display: block;
		margin-bottom: 0.5rem;
		font-weight: 500;
	}

	.form-group input,
	.form-group textarea {
		width: 100%;
		padding: 0.75rem;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		font-size: 1rem;
	}

	.member-selector {
		max-height: 300px;
		overflow-y: auto;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		padding: 0.5rem;
	}

	.member-option {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.75rem;
		cursor: pointer;
		border-radius: 4px;
	}

	.member-option:hover {
		background: var(--gray-50);
	}

	.member-option input[type="checkbox"] {
		width: auto;
	}

	.modal-actions {
		display: flex;
		justify-content: flex-end;
		gap: 1rem;
		margin-top: 2rem;
		padding-top: 1.5rem;
		border-top: 1px solid var(--gray-200);
	}
</style>