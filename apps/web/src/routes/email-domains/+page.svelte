<script lang="ts">
	import { api } from '$lib/api/api';
	import { onMount } from 'svelte';
	import { domainStore } from '$lib/stores/domains';

	let domains: any[] = [];
	let loading = true;
	let error = '';
	let showNewDomain = false;
	let newDomainName = '';
	let newDomainDescription = '';
	let creating = false;

	onMount(async () => {
		try {
			const result = await api.getEmailDomains();
			domains = result.domains || [];
		} catch (e: any) {
			error = e.message || 'Failed to load domains';
		} finally {
			loading = false;
		}
	});

	async function createNewDomain() {
		if (!newDomainName.trim()) return;
		creating = true;
		try {
			await api.createEmailDomain(newDomainName.trim(), newDomainDescription.trim() || undefined);
			domainStore.invalidate();
			// Refresh the list
			const result = await api.getEmailDomains();
			domains = result.domains || [];
			newDomainName = '';
			newDomainDescription = '';
			showNewDomain = false;
		} catch (e: any) {
			error = e.message || 'Failed to create domain';
		} finally {
			creating = false;
		}
	}

	function formatDate(ts: number | null) {
		if (!ts) return 'N/A';
		return new Date(ts).toLocaleDateString();
	}
</script>

<div class="page">
	<div class="page-header">
		<div>
			<h1>Email Domains</h1>
			<p class="subtitle">Communication domains discovered from your emails</p>
		</div>
		<div class="header-actions">
			<button class="btn btn-success" on:click={() => showNewDomain = !showNewDomain}>
				{showNewDomain ? 'Cancel' : '+ New Domain'}
			</button>
			<a href="/email-triage" class="btn btn-primary">Run AI Triage</a>
			<a href="/email-explorer" class="btn btn-secondary">Open Explorer</a>
		</div>
	</div>

	{#if showNewDomain}
		<div class="new-domain-form">
			<input
				type="text"
				bind:value={newDomainName}
				placeholder="Domain name (e.g., Sales, Personal)"
				class="new-domain-input"
			/>
			<input
				type="text"
				bind:value={newDomainDescription}
				placeholder="Description (optional)"
				class="new-domain-input"
			/>
			<button class="btn btn-primary" on:click={createNewDomain} disabled={creating || !newDomainName.trim()}>
				{creating ? 'Creating...' : 'Create'}
			</button>
		</div>
	{/if}

	{#if error}
		<div class="error-banner">{error}</div>
	{/if}

	{#if loading}
		<div class="loading">Loading domains...</div>
	{:else if domains.length === 0}
		<div class="empty-state">
			<h2>No Communication Domains Yet</h2>
			<p>Run AI Email Triage to discover domains from your imported emails.</p>
			<a href="/email-triage" class="btn btn-primary">Start AI Triage</a>
		</div>
	{:else}
		<div class="domains-grid">
			{#each domains as domain}
				<a href="/email-domains/{domain.concept_id}" class="domain-card">
					<div class="domain-header">
						<h3>{domain.name}</h3>
						<span class="email-count">{domain.email_count} emails</span>
					</div>
					{#if domain.description}
						<p class="domain-description">{domain.description}</p>
					{/if}
					<div class="domain-meta">
						<span>{domain.unique_senders} senders</span>
						{#if domain.earliest_date && domain.latest_date}
							<span>{formatDate(domain.earliest_date)} - {formatDate(domain.latest_date)}</span>
						{/if}
					</div>
					{#if domain.keywords && domain.keywords.length > 0}
						<div class="keywords">
							{#each domain.keywords.slice(0, 5) as kw}
								<span class="keyword-tag">{kw}</span>
							{/each}
							{#if domain.keywords.length > 5}
								<span class="keyword-tag more">+{domain.keywords.length - 5}</span>
							{/if}
						</div>
					{/if}
				</a>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page { padding: 2rem; max-width: 1200px; margin: 0 auto; }
	.page-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 2rem; }
	.subtitle { color: var(--gray-500); margin-top: 0.25rem; }
	.header-actions { display: flex; gap: 0.5rem; }

	.error-banner {
		background: #fee; color: #c00; padding: 1rem; border-radius: 8px;
		margin-bottom: 1rem; border: 1px solid #fcc;
	}
	.loading { text-align: center; padding: 3rem; color: var(--gray-500); }

	.empty-state {
		text-align: center; padding: 4rem 2rem; background: white;
		border-radius: 12px; box-shadow: 0 1px 3px rgba(0,0,0,0.1);
	}
	.empty-state h2 { color: var(--gray-700); }
	.empty-state p { color: var(--gray-500); margin-bottom: 1.5rem; }

	.domains-grid {
		display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
		gap: 1rem;
	}

	.domain-card {
		background: white; border-radius: 12px; padding: 1.25rem;
		box-shadow: 0 1px 3px rgba(0,0,0,0.1); text-decoration: none; color: inherit;
		transition: transform 0.15s, box-shadow 0.15s; display: block;
		border: 2px solid transparent;
	}
	.domain-card:hover {
		transform: translateY(-2px);
		box-shadow: 0 4px 12px rgba(0,0,0,0.1);
		border-color: var(--primary-light);
	}

	.domain-header { display: flex; justify-content: space-between; align-items: center; }
	.domain-header h3 { margin: 0; color: var(--gray-800); }
	.email-count {
		background: var(--primary-light); color: var(--primary);
		padding: 0.2rem 0.6rem; border-radius: 12px; font-size: 0.8rem; font-weight: 600;
	}

	.domain-description { color: var(--gray-600); font-size: 0.9rem; margin: 0.5rem 0; }
	.domain-meta {
		display: flex; gap: 1rem; font-size: 0.8rem; color: var(--gray-500);
		margin-bottom: 0.75rem;
	}

	.keywords { display: flex; flex-wrap: wrap; gap: 0.25rem; }
	.keyword-tag {
		background: var(--gray-100); color: var(--gray-600); padding: 0.15rem 0.5rem;
		border-radius: 12px; font-size: 0.75rem;
	}
	.keyword-tag.more { background: var(--primary-light); color: var(--primary); }

	.btn { padding: 0.5rem 1.25rem; border-radius: 6px; font-size: 0.95rem; cursor: pointer; text-decoration: none; border: none; display: inline-block; }
	.btn-primary { background: var(--primary); color: white; }
	.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
	.btn-secondary { background: var(--gray-100); color: var(--gray-700); border: 1px solid var(--gray-300); }
	.btn-success { background: #22c55e; color: white; }

	.new-domain-form {
		display: flex; gap: 0.75rem; align-items: center; flex-wrap: wrap;
		background: white; padding: 1rem 1.25rem; border-radius: 10px;
		margin-bottom: 1rem; box-shadow: 0 1px 3px rgba(0,0,0,0.1);
		border: 2px solid #22c55e;
	}
	.new-domain-input {
		flex: 1; min-width: 150px; padding: 0.5rem 0.75rem;
		border: 1px solid var(--gray-300); border-radius: 6px; font-size: 0.9rem;
	}
	.new-domain-input:focus { border-color: var(--primary); outline: none; box-shadow: 0 0 0 2px rgba(99,102,241,0.1); }
</style>
