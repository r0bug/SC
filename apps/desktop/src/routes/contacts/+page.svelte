<script lang="ts">
	import { onMount } from 'svelte';
	import { tauriApi } from '$lib/api/tauri-api';
	import type { Contact } from '$lib/api/types';

	let contacts: Contact[] = [];
	let loading = true;
	let searchQuery = '';
	let searchError = '';

	onMount(async () => {
		await loadContacts();
	});

	async function loadContacts() {
		try {
			loading = true;
			contacts = await tauriApi.getContacts(100, 0);
		} catch (error) {
			console.error('Failed to load contacts:', error);
		} finally {
			loading = false;
		}
	}

	async function handleSearch() {
		searchError = '';

		if (!searchQuery.trim()) {
			await loadContacts();
			return;
		}

		try {
			loading = true;
			contacts = await tauriApi.searchContacts(searchQuery);
			searchError = '';
		} catch (error: any) {
			console.error('Search failed:', error);
			searchError = error.message || 'Failed to search contacts. Please try again.';
		} finally {
			loading = false;
		}
	}

	async function deleteContact(id: string) {
		if (!confirm('Are you sure you want to delete this contact?')) return;

		try {
			await tauriApi.deleteContact(id);
			contacts = contacts.filter(c => c.id !== id);
			await tauriApi.showNotification('Contact Deleted', 'Contact has been deleted successfully');
		} catch (error) {
			console.error('Failed to delete contact:', error);
			alert('Failed to delete contact');
		}
	}

	function getDisplayName(contact: Contact): string {
		if (contact.first_name || contact.last_name) {
			return `${contact.first_name} ${contact.last_name || ''}`.trim();
		}
		if (contact.phone) return contact.phone;
		if (contact.email) return contact.email;
		return 'Unknown Contact';
	}
</script>

<div class="page">
	<div class="header">
		<div>
			<h1>Contacts</h1>
			<p class="subtitle">Manage your contacts and relationships</p>
		</div>
		<a href="/contacts/new" class="btn btn-primary">+ Add Contact</a>
	</div>

	<div class="filters card">
		<input
			type="text"
			placeholder="Search contacts..."
			bind:value={searchQuery}
			on:input={handleSearch}
			class="search-input"
		/>
		{#if searchError}
			<div class="error-message">{searchError}</div>
		{/if}
	</div>

	{#if loading}
		<div class="loading">Loading contacts...</div>
	{:else if contacts.length === 0}
		<div class="empty card">
			<p>No contacts found.</p>
			<a href="/contacts/new" class="btn btn-primary">Create your first contact</a>
		</div>
	{:else}
		<div class="contact-grid">
			{#each contacts as contact}
				<div class="contact-card card">
					<div class="contact-header">
						<h3>{getDisplayName(contact)}</h3>
						<div class="contact-actions">
							<a href="/contacts/{contact.id}" class="btn-icon" title="View">
								👁️
							</a>
							<button
								on:click={() => deleteContact(contact.id)}
								class="btn-icon btn-danger"
								title="Delete"
							>
								🗑️
							</button>
						</div>
					</div>

					<div class="contact-details">
						{#if contact.organization}
							<div class="detail">
								<span class="label">🏢</span>
								<span>{contact.organization}</span>
							</div>
						{/if}
						{#if contact.title}
							<div class="detail">
								<span class="label">💼</span>
								<span>{contact.title}</span>
							</div>
						{/if}
						{#if contact.email}
							<div class="detail">
								<span class="label">📧</span>
								<span>{contact.email}</span>
							</div>
						{/if}
						{#if contact.phone}
							<div class="detail">
								<span class="label">📱</span>
								<span>{contact.phone}</span>
							</div>
						{/if}
					</div>

					{#if contact.tags && contact.tags.length > 0}
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
		max-width: 1400px;
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

	.filters {
		margin-bottom: 2rem;
		padding: 1.5rem;
	}

	.search-input {
		width: 100%;
		padding: 0.75rem 1rem;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		font-size: 1rem;
	}

	.search-input:focus {
		outline: none;
		border-color: var(--primary);
		box-shadow: 0 0 0 3px var(--primary-light);
	}

	.error-message {
		margin-top: 1rem;
		padding: 0.75rem;
		background: var(--color-error-50);
		color: var(--color-error-700);
		border-radius: 6px;
		font-size: 0.875rem;
	}

	.loading {
		text-align: center;
		padding: 3rem;
		color: var(--gray-600);
		font-size: 1.125rem;
	}

	.empty {
		text-align: center;
		padding: 3rem;
	}

	.empty p {
		margin: 0 0 1.5rem;
		color: var(--gray-600);
		font-size: 1.125rem;
	}

	.contact-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
		gap: 1.5rem;
	}

	.contact-card {
		padding: 1.5rem;
		transition: transform 0.2s, box-shadow 0.2s;
	}

	.contact-card:hover {
		transform: translateY(-2px);
		box-shadow: var(--shadow-md);
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
		color: var(--gray-900);
		flex: 1;
	}

	.contact-actions {
		display: flex;
		gap: 0.5rem;
	}

	.btn-icon {
		padding: 0.5rem;
		background: none;
		border: none;
		cursor: pointer;
		font-size: 1.25rem;
		border-radius: 4px;
		transition: background-color 0.2s;
		text-decoration: none;
	}

	.btn-icon:hover {
		background: var(--gray-100);
	}

	.btn-danger:hover {
		background: var(--color-error-50);
	}

	.contact-details {
		margin-bottom: 1rem;
	}

	.detail {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		margin-bottom: 0.5rem;
		font-size: 0.875rem;
		color: var(--gray-700);
	}

	.label {
		font-size: 1rem;
	}

	.tags {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
		margin-top: 1rem;
	}

	.tag {
		display: inline-block;
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

	.btn {
		padding: 0.75rem 1.5rem;
		border: none;
		border-radius: 6px;
		cursor: pointer;
		font-weight: 500;
		text-decoration: none;
		display: inline-block;
		transition: all 0.2s;
	}

	.btn-primary {
		background: var(--primary);
		color: white;
	}

	.btn-primary:hover {
		background: var(--primary-dark);
	}
</style>
