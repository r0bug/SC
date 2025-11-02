<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { tauriApi } from '$lib/api/tauri-api';
	import type { Group, Contact } from '$lib/api/types';

	let groupId: string;
	let group: Group | null = null;
	let allContacts: Contact[] = [];
	let groupMembers: Contact[] = [];
	let availableContacts: Contact[] = [];
	let loading = true;
	let error = '';
	let searchQuery = '';
	let filteredAvailableContacts: Contact[] = [];

	$: groupId = $page.params.id;

	onMount(async () => {
		await loadData();
	});

	async function loadData() {
		try {
			loading = true;
			error = '';

			// Load group details
			group = await tauriApi.getGroup(groupId);

			// Load all contacts
			allContacts = await tauriApi.getContacts(100000, 0);

			// Separate members from available contacts
			const memberIds = new Set(group.contact_ids);
			groupMembers = allContacts.filter(c => memberIds.has(c.id));
			availableContacts = allContacts.filter(c => !memberIds.has(c.id));
			filteredAvailableContacts = availableContacts;
		} catch (err) {
			console.error('Failed to load group data:', err);
			error = 'Failed to load group details';
		} finally {
			loading = false;
		}
	}

	function getDisplayName(contact: Contact): string {
		if (contact.first_name || contact.last_name) {
			return `${contact.first_name || ''} ${contact.last_name || ''}`.trim();
		}
		if (contact.phone) return contact.phone;
		if (contact.email) return contact.email;
		return 'Unnamed Contact';
	}

	function handleSearchInput() {
		if (!searchQuery.trim()) {
			filteredAvailableContacts = availableContacts;
		} else {
			const query = searchQuery.toLowerCase().trim();
			filteredAvailableContacts = availableContacts.filter(c => {
				const name = getDisplayName(c).toLowerCase();
				const email = (c.email || '').toLowerCase();
				const phone = (c.phone || '').toLowerCase();
				return name.includes(query) || email.includes(query) || phone.includes(query);
			});
		}
	}

	async function addMember(contactId: string) {
		try {
			await tauriApi.addGroupMember(groupId, contactId);
			await tauriApi.showNotification('Member Added', 'Contact has been added to the group');
			await loadData();
			searchQuery = '';
		} catch (err) {
			console.error('Failed to add member:', err);
			alert('Failed to add member to group');
		}
	}

	async function removeMember(contactId: string, contactName: string) {
		if (!confirm(`Remove ${contactName} from this group?`)) return;

		try {
			await tauriApi.removeGroupMember(groupId, contactId);
			await tauriApi.showNotification('Member Removed', 'Contact has been removed from the group');
			await loadData();
		} catch (err) {
			console.error('Failed to remove member:', err);
			alert('Failed to remove member from group');
		}
	}

	async function deleteGroup() {
		if (!group) return;
		if (!confirm(`Are you sure you want to delete the group "${group.name}"?`)) return;

		try {
			await tauriApi.deleteGroup(groupId);
			await tauriApi.showNotification('Group Deleted', `${group.name} has been deleted`);
			goto('/groups');
		} catch (err) {
			console.error('Failed to delete group:', err);
			alert('Failed to delete group');
		}
	}

	function goBack() {
		goto('/groups');
	}
</script>

<div class="page">
	{#if loading}
		<div class="loading">Loading group details...</div>
	{:else if error}
		<div class="error card">
			<p>{error}</p>
			<button on:click={goBack} class="btn btn-primary">Back to Groups</button>
		</div>
	{:else if group}
		<div class="header">
			<div class="header-left">
				<button on:click={goBack} class="btn-back" title="Back to Groups">
					← Back
				</button>
				<div>
					<h1>{group.name}</h1>
					{#if group.description}
						<p class="subtitle">{group.description}</p>
					{/if}
					<p class="meta">
						{groupMembers.length} member{groupMembers.length !== 1 ? 's' : ''} •
						Created {new Date(group.created_at).toLocaleDateString()}
					</p>
				</div>
			</div>
			<button on:click={deleteGroup} class="btn btn-danger">
				Delete Group
			</button>
		</div>

		<div class="content-grid">
			<!-- Current Members -->
			<div class="section">
				<div class="section-header">
					<h2>Members</h2>
				</div>

				{#if groupMembers.length === 0}
					<div class="empty card">
						<p>No members in this group yet.</p>
						<p>Add contacts from the list on the right.</p>
					</div>
				{:else}
					<div class="member-list">
						{#each groupMembers as member}
							<div class="member-card card">
								<div class="member-info">
									<h3>{getDisplayName(member)}</h3>
									{#if member.email}
										<p class="contact-detail">✉️ {member.email}</p>
									{/if}
									{#if member.phone}
										<p class="contact-detail">📞 {member.phone}</p>
									{/if}
									{#if member.organization}
										<p class="contact-detail">🏢 {member.organization}</p>
									{/if}
								</div>
								<button
									on:click={() => removeMember(member.id, getDisplayName(member))}
									class="btn btn-sm btn-secondary"
									title="Remove from group"
								>
									Remove
								</button>
							</div>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Add Members -->
			<div class="section">
				<div class="section-header">
					<h2>Add Members</h2>
				</div>

				<div class="search-box card">
					<input
						type="text"
						placeholder="Search contacts to add..."
						bind:value={searchQuery}
						on:input={handleSearchInput}
						class="search-input"
					/>
				</div>

				{#if filteredAvailableContacts.length === 0}
					<div class="empty card">
						{#if availableContacts.length === 0}
							<p>All contacts are already in this group.</p>
						{:else}
							<p>No contacts found matching "{searchQuery}".</p>
						{/if}
					</div>
				{:else}
					<div class="available-list">
						{#each filteredAvailableContacts.slice(0, 50) as contact}
							<div class="available-card card">
								<div class="member-info">
									<h3>{getDisplayName(contact)}</h3>
									{#if contact.email}
										<p class="contact-detail">✉️ {contact.email}</p>
									{/if}
									{#if contact.phone}
										<p class="contact-detail">📞 {contact.phone}</p>
									{/if}
								</div>
								<button
									on:click={() => addMember(contact.id)}
									class="btn btn-sm btn-primary"
									title="Add to group"
								>
									Add
								</button>
							</div>
						{/each}
						{#if filteredAvailableContacts.length > 50}
							<div class="more-notice card">
								<p>Showing first 50 of {filteredAvailableContacts.length} contacts. Use search to narrow results.</p>
							</div>
						{/if}
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>

<style>
	.page {
		padding: 2rem;
		max-width: 1400px;
		margin: 0 auto;
	}

	.loading, .error {
		text-align: center;
		padding: 3rem;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: 2rem;
		gap: 2rem;
	}

	.header-left {
		display: flex;
		align-items: flex-start;
		gap: 1rem;
		flex: 1;
	}

	.btn-back {
		padding: 0.5rem 1rem;
		background: var(--gray-200);
		border: none;
		border-radius: 6px;
		cursor: pointer;
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--gray-900);
		transition: background-color 0.2s;
	}

	.btn-back:hover {
		background: var(--gray-300);
	}

	.header h1 {
		margin: 0 0 0.5rem;
		font-size: 2rem;
		color: var(--gray-900);
	}

	.subtitle {
		margin: 0 0 0.5rem;
		color: var(--gray-700);
		font-size: 1rem;
	}

	.meta {
		margin: 0;
		color: var(--gray-500);
		font-size: 0.875rem;
	}

	.content-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 2rem;
	}

	@media (max-width: 1024px) {
		.content-grid {
			grid-template-columns: 1fr;
		}
	}

	.section {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.section-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.section-header h2 {
		margin: 0;
		font-size: 1.5rem;
		color: var(--gray-900);
	}

	.card {
		background: white;
		border-radius: 8px;
		box-shadow: var(--shadow-sm);
		padding: 1.5rem;
	}

	.empty {
		text-align: center;
		padding: 2rem;
	}

	.empty p {
		margin: 0 0 0.5rem;
		color: var(--gray-600);
	}

	.search-box {
		padding: 1rem;
	}

	.search-input {
		width: 100%;
		padding: 0.75rem;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		font-size: 1rem;
	}

	.search-input:focus {
		outline: none;
		border-color: var(--primary);
		box-shadow: 0 0 0 3px var(--primary-light);
	}

	.member-list,
	.available-list {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		max-height: 600px;
		overflow-y: auto;
	}

	.member-card,
	.available-card {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 1rem;
		padding: 1rem;
		transition: transform 0.2s;
	}

	.member-card:hover,
	.available-card:hover {
		transform: translateX(2px);
	}

	.member-info {
		flex: 1;
		min-width: 0;
	}

	.member-info h3 {
		margin: 0 0 0.25rem;
		font-size: 1rem;
		color: var(--gray-900);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.contact-detail {
		margin: 0.125rem 0;
		font-size: 0.75rem;
		color: var(--gray-600);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.more-notice {
		text-align: center;
		padding: 1rem;
		background: var(--primary-light);
	}

	.more-notice p {
		margin: 0;
		color: var(--primary);
		font-size: 0.875rem;
	}

	.btn {
		padding: 0.75rem 1.5rem;
		border: none;
		border-radius: 6px;
		font-size: 1rem;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.2s;
		white-space: nowrap;
	}

	.btn-sm {
		padding: 0.5rem 1rem;
		font-size: 0.875rem;
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
		background: var(--color-error);
		color: white;
	}

	.btn-danger:hover {
		background: var(--color-error-dark);
	}
</style>
