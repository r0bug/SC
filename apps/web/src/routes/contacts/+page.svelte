<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { Contact } from '$lib/api/types';

	let allContacts: Contact[] = [];
	let contacts: Contact[] = [];
	let loading = true;
	let searchQuery = '';
	let selectedTags: string[] = [];
	let searchError = '';
	let onlyWithNames = false;
	let searchTimeout: number | undefined;

	// Reconciliation state
	let unlinkedCount = 0;
	let reconciling = false;
	let reconcileMessage = '';

	onMount(async () => {
		await Promise.all([loadContacts(), loadUnlinkedStats()]);
	});

	async function loadContacts() {
		try {
			loading = true;
			allContacts = await api.getContacts(100000, 0); // Load all contacts
			applyFilters();
		} catch (error) {
			console.error('Failed to load contacts:', error);
		} finally {
			loading = false;
		}
	}

	async function loadUnlinkedStats() {
		try {
			const stats = await api.getUnlinkedStats();
			unlinkedCount = stats.total_unlinked_senders;
		} catch (error) {
			console.error('Failed to load unlinked stats:', error);
		}
	}

	async function discoverContacts() {
		reconciling = true;
		reconcileMessage = '';
		try {
			const result = await api.reconcileContacts();
			const parts: string[] = [];
			if (result.contacts_created > 0) parts.push(`${result.contacts_created} contacts created`);
			if (result.contacts_matched > 0) parts.push(`${result.contacts_matched} existing contacts matched`);
			if (result.email_messages_linked > 0) parts.push(`${result.email_messages_linked} emails linked`);
			if (result.sms_messages_linked > 0) parts.push(`${result.sms_messages_linked} SMS linked`);
			reconcileMessage = parts.length > 0 ? parts.join(', ') : 'No new contacts to discover';
			// Refresh contacts and stats
			await Promise.all([loadContacts(), loadUnlinkedStats()]);
		} catch (error) {
			console.error('Failed to reconcile contacts:', error);
			reconcileMessage = 'Failed to discover contacts. Please try again.';
		} finally {
			reconciling = false;
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

	function applyFilters() {
		let filtered = [...allContacts];

		// Filter by search query
		if (searchQuery.trim()) {
			const query = searchQuery.toLowerCase().trim();
			filtered = filtered.filter(c => {
				const name = getDisplayName(c).toLowerCase();
				const email = (c.email || '').toLowerCase();
				const phone = (c.phone || '').toLowerCase();
				const org = (c.organization || '').toLowerCase();
				return name.includes(query) || email.includes(query) ||
				       phone.includes(query) || org.includes(query);
			});
		}

		// Filter by tags
		if (selectedTags.length > 0) {
			filtered = filtered.filter(c =>
				selectedTags.some(tag => c.tags.includes(tag))
			);
		}

		// Filter by has name
		if (onlyWithNames) {
			filtered = filtered.filter(c =>
				(c.first_name && c.first_name.trim()) ||
				(c.last_name && c.last_name.trim())
			);
		}

		contacts = filtered;
	}

	function handleSearchInput() {
		// Debounce search to prevent race conditions
		if (searchTimeout) {
			clearTimeout(searchTimeout);
		}
		searchTimeout = window.setTimeout(() => {
			applyFilters();
		}, 300);
	}

	function toggleNameFilter() {
		onlyWithNames = !onlyWithNames;
		applyFilters();
	}

	async function deleteContact(id: string) {
		if (!confirm('Are you sure you want to delete this contact?')) return;

		try {
			await api.deleteContact(id);
			contacts = contacts.filter(c => c.id !== id);
		} catch (error) {
			console.error('Failed to delete contact:', error);
		}
	}
</script>

<div class="page">
	<div class="header">
		<div>
			<h1>Contacts</h1>
			<p class="subtitle">Manage your contacts and relationships</p>
		</div>
		<div class="header-actions">
			{#if unlinkedCount > 0}
				<button
					class="btn btn-discover"
					on:click={discoverContacts}
					disabled={reconciling}
				>
					{#if reconciling}
						Discovering...
					{:else}
						Discover Contacts ({unlinkedCount} unlinked)
					{/if}
				</button>
			{/if}
			<a href="/contacts/new" class="btn btn-primary">+ Add Contact</a>
		</div>
	</div>

	{#if reconcileMessage}
		<div class="reconcile-message card">
			<p>{reconcileMessage}</p>
			<button class="dismiss-btn" on:click={() => reconcileMessage = ''}>&times;</button>
		</div>
	{/if}

	<div class="filters card">
		<div class="search-row">
			<input
				type="text"
				placeholder="Search contacts..."
				bind:value={searchQuery}
				on:input={handleSearchInput}
				class="search-input"
			/>
			<label class="filter-checkbox">
				<input
					type="checkbox"
					bind:checked={onlyWithNames}
					on:change={toggleNameFilter}
				/>
				<span>Only show contacts with names</span>
			</label>
		</div>
		{#if searchError}
			<div class="error-message">{searchError}</div>
		{/if}
	</div>

	{#if loading}
		<div class="loading">Loading contacts...</div>
	{:else if contacts.length === 0}
		<div class="empty card">
			<p>No contacts found.</p>
			{#if unlinkedCount > 0}
				<p class="empty-hint">You have {unlinkedCount} unlinked senders from your emails and messages.</p>
				<button class="btn btn-primary" on:click={discoverContacts} disabled={reconciling}>
					{reconciling ? 'Discovering...' : 'Discover Contacts'}
				</button>
			{:else}
				<a href="/contacts/new" class="btn btn-primary">Create your first contact</a>
			{/if}
		</div>
	{:else}
		<div class="contact-grid">
			{#each contacts as contact}
				<div class="contact-card card">
					<div class="contact-header">
						<h3>
							{#if contact.first_name || contact.last_name}
								{contact.first_name} {contact.last_name || ''}
							{:else if contact.phone}
								{contact.phone}
							{:else if contact.email}
								{contact.email}
							{:else}
								Unnamed Contact
							{/if}
						</h3>
						<div class="contact-actions">
							<a href="/contacts/{contact.id}" class="btn btn-sm">Edit</a>
							<button on:click={() => deleteContact(contact.id)} class="btn btn-sm btn-secondary">
								Delete
							</button>
						</div>
					</div>
					{#if contact.organization}
						<p class="contact-field">{contact.organization}</p>
					{/if}
					{#if contact.email}
						<p class="contact-field">{contact.email}</p>
					{/if}
					{#if contact.phone}
						<p class="contact-field">{contact.phone}</p>
					{/if}
					{#if contact.tags.length > 0}
						<div class="tags">
							{#each contact.tags as tag}
								<span class="tag">{tag}</span>
							{/each}
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
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 2rem;
	}

	.header-actions {
		display: flex;
		gap: 0.75rem;
		align-items: center;
	}

	.subtitle {
		color: var(--gray-600);
		margin-top: 0.25rem;
	}

	.btn-discover {
		padding: 0.625rem 1.25rem;
		background: var(--gray-100, #f3f4f6);
		color: var(--gray-800, #1f2937);
		border: 1px solid var(--gray-300, #d1d5db);
		border-radius: 6px;
		font-size: 0.875rem;
		cursor: pointer;
		white-space: nowrap;
	}

	.btn-discover:hover:not(:disabled) {
		background: var(--gray-200, #e5e7eb);
	}

	.btn-discover:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.reconcile-message {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1rem 1.5rem;
		margin-bottom: 1.5rem;
		background: #f0fdf4;
		border: 1px solid #bbf7d0;
		border-radius: 6px;
	}

	.reconcile-message p {
		margin: 0;
		color: #166534;
		font-size: 0.875rem;
	}

	.dismiss-btn {
		background: none;
		border: none;
		font-size: 1.25rem;
		color: #166534;
		cursor: pointer;
		padding: 0 0.25rem;
	}

	.empty-hint {
		color: var(--gray-500);
		font-size: 0.875rem;
		margin-bottom: 1rem;
	}

	.filters {
		margin-bottom: 2rem;
		padding: 1.5rem;
	}

	.search-row {
		display: flex;
		gap: 1rem;
		align-items: center;
		flex-wrap: wrap;
	}

	.search-input {
		flex: 1;
		min-width: 300px;
		padding: 0.75rem;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		font-size: 1rem;
	}

	.filter-checkbox {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		cursor: pointer;
		user-select: none;
		white-space: nowrap;
	}

	.filter-checkbox input[type="checkbox"] {
		cursor: pointer;
		width: 1.125rem;
		height: 1.125rem;
	}

	.filter-checkbox span {
		font-size: 0.9rem;
		color: var(--gray-700);
	}

	.error-message {
		margin-top: 0.75rem;
		padding: 0.75rem;
		background: #fee;
		color: #c33;
		border-radius: 6px;
		font-size: 0.875rem;
	}

	.contact-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
		gap: 1.5rem;
	}

	.contact-card {
		padding: 1.5rem;
	}

	.contact-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: 1rem;
	}

	.contact-header h3 {
		margin: 0;
		font-size: 1.25rem;
	}

	.contact-actions {
		display: flex;
		gap: 0.5rem;
	}

	.contact-field {
		margin: 0.5rem 0;
		color: var(--gray-700);
		font-size: 0.875rem;
	}

	.tags {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
		margin-top: 1rem;
	}

	.tag {
		padding: 0.25rem 0.75rem;
		background: var(--primary-light);
		color: var(--primary);
		border-radius: 12px;
		font-size: 0.75rem;
	}

	.btn-sm {
		padding: 0.375rem 0.75rem;
		font-size: 0.875rem;
	}

	.loading, .empty {
		text-align: center;
		padding: 3rem;
	}
</style>
