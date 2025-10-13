<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { Contact } from '$lib/api/types';

	let contacts: Contact[] = [];
	let loading = true;
	let searchQuery = '';
	let selectedTags: string[] = [];
	let searchError = '';

	onMount(async () => {
		await loadContacts();
	});

	async function loadContacts() {
		try {
			loading = true;
			contacts = await api.getContacts(100, 0);
		} catch (error) {
			console.error('Failed to load contacts:', error);
		} finally {
			loading = false;
		}
	}

	async function handleSearch() {
		searchError = ''; // Clear previous errors

		if (!searchQuery.trim() && selectedTags.length === 0) {
			await loadContacts();
			return;
		}

		try {
			loading = true;
			contacts = await api.searchContacts(searchQuery, { tags: selectedTags });
			searchError = ''; // Clear error on success
		} catch (error: any) {
			console.error('Search failed:', error);
			searchError = error.message || 'Failed to search contacts. Please try again.';
			// Keep showing current contacts even if search fails
		} finally {
			loading = false;
		}
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
						<p class="contact-field">🏢 {contact.organization}</p>
					{/if}
					{#if contact.email}
						<p class="contact-field">✉️ {contact.email}</p>
					{/if}
					{#if contact.phone}
						<p class="contact-field">📞 {contact.phone}</p>
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

	.subtitle {
		color: var(--gray-600);
		margin-top: 0.25rem;
	}

	.filters {
		margin-bottom: 2rem;
		padding: 1.5rem;
	}

	.search-input {
		width: 100%;
		padding: 0.75rem;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		font-size: 1rem;
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
