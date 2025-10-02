<script lang="ts">
	import { onMount } from 'svelte';
	import { api, type Contact } from '$lib/api/client';

	let contacts: Contact[] = [];
	let loading = true;
	let error = '';
	let searchQuery = '';
	let searching = false;

	onMount(async () => {
		await loadContacts();
	});

	async function loadContacts() {
		try {
			loading = true;
			contacts = await api.getContacts(50, 0);
		} catch (e: any) {
			error = e.message;
		} finally {
			loading = false;
		}
	}

	async function handleSearch() {
		if (!searchQuery.trim()) {
			await loadContacts();
			return;
		}

		try {
			searching = true;
			contacts = await api.searchContacts(searchQuery);
		} catch (e: any) {
			error = e.message;
		} finally {
			searching = false;
		}
	}
</script>

<div>
	<div class="header">
		<h1>Contacts</h1>
		<button class="btn btn-primary">+ Add Contact</button>
	</div>

	<div class="search-bar">
		<input
			type="text"
			placeholder="Search contacts..."
			bind:value={searchQuery}
			on:input={handleSearch}
			class="search-input"
		/>
		<button class="btn btn-secondary" on:click={handleSearch} disabled={searching}>
			{searching ? 'Searching...' : '🔍 Search'}
		</button>
	</div>

	{#if loading}
		<div class="card">
			<p>Loading contacts...</p>
		</div>
	{:else if error}
		<div class="card" style="border-left: 4px solid var(--error);">
			<h3 style="color: var(--error); margin-bottom: 0.5rem;">Error</h3>
			<p>{error}</p>
			<p style="margin-top: 1rem; font-size: 0.875rem; color: var(--gray-600);">
				Make sure the sync service is running on port 3002:<br/>
				<code>cargo run --release --bin sync_service</code>
			</p>
		</div>
	{:else if contacts.length === 0}
		<div class="card">
			<p>No contacts found. Import sample data:</p>
			<pre style="background: var(--gray-100); padding: 1rem; border-radius: 4px; margin-top: 1rem;">
./target/release/sagenscontact import --csv sample_data/contacts.csv</pre>
		</div>
	{:else}
		<div class="contact-list">
			{#each contacts as contact}
				<div class="card contact-card">
					<div class="contact-info">
						<a href="/contacts/{contact.id}" class="contact-name">
							<h3>{contact.first_name} {contact.last_name || ''}</h3>
						</a>
						{#if contact.email}
							<p class="contact-detail">✉️ {contact.email}</p>
						{/if}
						{#if contact.phone}
							<p class="contact-detail">📞 {contact.phone}</p>
						{/if}
						{#if contact.organization}
							<p class="contact-detail">🏢 {contact.organization}</p>
						{/if}
					</div>
					<div class="contact-actions">
						<a href="/contacts/{contact.id}" class="btn btn-secondary">
							👁️ View
						</a>
						<a href="/communications?contact={contact.id}" class="btn btn-secondary">
							📧 Communicate
						</a>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1.5rem;
	}

	h1 {
		font-size: 2rem;
		font-weight: 700;
	}

	.search-bar {
		display: flex;
		gap: 0.5rem;
		margin-bottom: 1.5rem;
	}

	.search-input {
		flex: 1;
		padding: 0.75rem;
		border: 1px solid var(--gray-300);
		border-radius: 4px;
		font-size: 1rem;
	}

	.contact-name {
		text-decoration: none;
		color: inherit;
	}

	.contact-name:hover h3 {
		color: var(--primary);
	}

	.contact-list {
		display: grid;
		gap: 1rem;
	}

	.contact-card {
		display: flex;
		justify-content: space-between;
		align-items: center;
		transition: box-shadow 0.2s;
	}

	.contact-card:hover {
		box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
	}

	.contact-info h3 {
		font-size: 1.25rem;
		margin-bottom: 0.5rem;
	}

	.contact-detail {
		font-size: 0.875rem;
		color: var(--gray-600);
		margin: 0.25rem 0;
	}

	.contact-actions {
		display: flex;
		gap: 0.5rem;
	}

	@media (max-width: 768px) {
		.contact-card {
			flex-direction: column;
			align-items: flex-start;
		}

		.contact-actions {
			margin-top: 1rem;
			width: 100%;
		}

		.contact-actions .btn {
			flex: 1;
		}
	}
</style>