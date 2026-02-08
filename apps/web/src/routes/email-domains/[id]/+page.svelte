<script lang="ts">
	import { api } from '$lib/api/api';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import DomainTagInput from '$lib/components/DomainTagInput.svelte';
	import BulkDomainToolbar from '$lib/components/BulkDomainToolbar.svelte';

	type DomainStats = {
		concept_id: string;
		name: string;
		email_count: number;
		unique_senders: number;
		earliest_date: number | null;
		latest_date: number | null;
		top_senders: Array<{ address: string; name: string | null; count: number }>;
	};

	type DomainEmail = {
		id: string;
		from_address: string;
		from_name: string | null;
		to_address: string;
		subject: string;
		body_text: string | null;
		body_html: string | null;
		message_date: number;
		message_type: number;
		read_status: number;
		has_attachments: number;
		folder: string | null;
	};

	let domainId: string = '';
	let stats: DomainStats | null = null;
	let emails: DomainEmail[] = [];
	let selectedEmail: DomainEmail | null = null;
	let loading = true;
	let error = '';
	let selectedEmailIds: string[] = [];
	let emailDomains: Record<string, Array<{ concept_id: string; name: string }>> = {};

	$: domainId = $page.params.id || '';

	onMount(async () => {
		try {
			const [statsResult, emailsResult] = await Promise.all([
				api.getDomainStats(domainId),
				api.getDomainEmails(domainId),
			]);
			stats = statsResult;
			emails = emailsResult.emails || [];
			// Load domain assignments for visible emails
			loadEmailDomains();
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : 'Failed to load domain';
		} finally {
			loading = false;
		}
	});

	async function loadEmailDomains() {
		for (const email of emails.slice(0, 50)) {
			try {
				const result = await api.getEmailDomainAssignments(email.id);
				emailDomains[email.id] = result.domains.map((d: { concept_id: string; name: string }) => ({ concept_id: d.concept_id, name: d.name }));
			} catch {
				emailDomains[email.id] = [];
			}
		}
		emailDomains = { ...emailDomains };
	}

	function selectEmail(email: DomainEmail) {
		selectedEmail = email;
	}

	function toggleSelectEmail(emailId: string) {
		if (selectedEmailIds.includes(emailId)) {
			selectedEmailIds = selectedEmailIds.filter(id => id !== emailId);
		} else {
			selectedEmailIds = [...selectedEmailIds, emailId];
		}
	}

	function formatDate(ts: number) {
		if (!ts) return '';
		return new Date(ts).toLocaleDateString('en-US', {
			year: 'numeric', month: 'short', day: 'numeric'
		});
	}

	function formatDateTime(ts: number) {
		if (!ts) return '';
		return new Date(ts).toLocaleString();
	}

	// Group emails by sender for the left pane
	$: senderGroups = (() => {
		const groups: Record<string, { address: string; name: string | null; emails: DomainEmail[] }> = {};
		for (const email of emails) {
			const key = email.from_address;
			if (!groups[key]) {
				groups[key] = { address: key, name: email.from_name, emails: [] };
			}
			groups[key].emails.push(email);
		}
		return Object.values(groups).sort((a, b) => b.emails.length - a.emails.length);
	})();

	let selectedSender: string | null = null;

	$: filteredEmails = selectedSender
		? emails.filter(e => e.from_address === selectedSender)
		: emails;

	function handleBulkAssigned() {
		selectedEmailIds = [];
		loadEmailDomains();
	}
</script>

<div class="page">
	{#if loading}
		<div class="loading">Loading domain...</div>
	{:else if error}
		<div class="error-banner">{error}</div>
	{:else}
		<div class="domain-header">
			<div>
				<a href="/email-domains" class="back-link">Back to Domains</a>
				<h1>{stats?.name || 'Domain'}</h1>
			</div>
			<div class="header-stats">
				<div class="stat">
					<span class="stat-value">{stats?.email_count || 0}</span>
					<span class="stat-label">Emails</span>
				</div>
				<div class="stat">
					<span class="stat-value">{stats?.unique_senders || 0}</span>
					<span class="stat-label">Senders</span>
				</div>
				{#if stats?.earliest_date}
					<div class="stat">
						<span class="stat-value">{formatDate(stats.earliest_date)}</span>
						<span class="stat-label">Earliest</span>
					</div>
				{/if}
			</div>
		</div>

		<!-- Top senders bar -->
		{#if stats?.top_senders && stats.top_senders.length > 0}
			<div class="top-senders">
				<strong>Top senders:</strong>
				{#each stats.top_senders.slice(0, 5) as sender}
					<span class="sender-chip" class:active={selectedSender === sender.address}
						role="button"
						tabindex="0"
						on:click={() => selectedSender = selectedSender === sender.address ? null : sender.address}
						on:keydown={(e) => e.key === 'Enter' && (selectedSender = selectedSender === sender.address ? null : sender.address)}>
						{sender.name || sender.address} ({sender.count})
					</span>
				{/each}
				{#if selectedSender}
					<button class="clear-filter" on:click={() => selectedSender = null}>Clear filter</button>
				{/if}
			</div>
		{/if}

		<!-- Three-pane layout -->
		<div class="three-pane">
			<!-- Left: Senders -->
			<div class="pane senders-pane">
				<h3>Senders ({senderGroups.length})</h3>
				{#each senderGroups as group}
					<button class="sender-item" class:active={selectedSender === group.address}
						on:click={() => selectedSender = selectedSender === group.address ? null : group.address}>
						<span class="sender-name">{group.name || group.address}</span>
						<span class="sender-count">{group.emails.length}</span>
					</button>
				{/each}
			</div>

			<!-- Middle: Email list -->
			<div class="pane email-list-pane">
				<h3>Emails ({filteredEmails.length})</h3>
				{#each filteredEmails as email}
					<div class="email-item-row" class:active={selectedEmail?.id === email.id}>
						<input
							type="checkbox"
							checked={selectedEmailIds.includes(email.id)}
							on:change={() => toggleSelectEmail(email.id)}
							on:click|stopPropagation
							class="email-checkbox"
						/>
						<button class="email-item" on:click={() => selectEmail(email)}>
							<div class="email-from">{email.from_name || email.from_address}</div>
							<div class="email-subject">{email.subject}</div>
							<div class="email-date">{formatDate(email.message_date)}</div>
						</button>
						{#if emailDomains[email.id]}
							<div class="email-inline-domains">
								<DomainTagInput
									emailId={email.id}
									currentDomains={emailDomains[email.id]}
									compact={true}
									on:domainsChanged={() => loadEmailDomains()}
								/>
							</div>
						{/if}
					</div>
				{/each}
				{#if filteredEmails.length === 0}
					<p class="empty-text">No emails found</p>
				{/if}
			</div>

			<!-- Right: Reading pane -->
			<div class="pane reading-pane">
				{#if selectedEmail}
					<div class="email-detail">
						<h2>{selectedEmail.subject}</h2>
						<div class="reading-pane-domains">
							<DomainTagInput
								emailId={selectedEmail.id}
								currentDomains={emailDomains[selectedEmail.id] || []}
								on:domainsChanged={() => loadEmailDomains()}
							/>
						</div>
						<div class="email-meta">
							<div><strong>From:</strong> {selectedEmail.from_name || ''} &lt;{selectedEmail.from_address}&gt;</div>
							<div><strong>To:</strong> {selectedEmail.to_address}</div>
							<div><strong>Date:</strong> {formatDateTime(selectedEmail.message_date)}</div>
							{#if selectedEmail.folder}
								<div><strong>Folder:</strong> {selectedEmail.folder}</div>
							{/if}
						</div>
						<div class="email-body">
							{#if selectedEmail.body_html}
								{@html selectedEmail.body_html}
							{:else}
								<pre>{selectedEmail.body_text || '(No content)'}</pre>
							{/if}
						</div>
					</div>
				{:else}
					<div class="no-selection">
						<p>Select an email to read</p>
					</div>
				{/if}
			</div>
		</div>

		<BulkDomainToolbar
			{selectedEmailIds}
			on:clear={() => selectedEmailIds = []}
			on:assigned={handleBulkAssigned}
		/>
	{/if}
</div>

<style>
	.page { padding: 1.5rem; height: calc(100vh - 3rem); display: flex; flex-direction: column; }
	.back-link { color: var(--primary); text-decoration: none; font-size: 0.85rem; }
	.back-link:hover { text-decoration: underline; }

	.domain-header { display: flex; justify-content: space-between; align-items: flex-end; margin-bottom: 1rem; }
	.domain-header h1 { margin: 0.25rem 0 0 0; }
	.header-stats { display: flex; gap: 2rem; }
	.stat { display: flex; flex-direction: column; align-items: center; }
	.stat-value { font-size: 1.2rem; font-weight: 700; color: var(--primary); }
	.stat-label { font-size: 0.75rem; color: var(--gray-500); }

	.error-banner { background: #fee; color: #c00; padding: 1rem; border-radius: 8px; border: 1px solid #fcc; }
	.loading { text-align: center; padding: 3rem; color: var(--gray-500); }

	.top-senders {
		display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap;
		padding: 0.75rem 1rem; background: white; border-radius: 8px;
		margin-bottom: 1rem; font-size: 0.85rem;
	}
	.sender-chip {
		background: var(--gray-100); padding: 0.2rem 0.6rem; border-radius: 12px;
		cursor: pointer; font-size: 0.8rem;
	}
	.sender-chip:hover { background: var(--gray-200); }
	.sender-chip.active { background: var(--primary); color: white; }
	.clear-filter {
		background: none; border: none; color: var(--primary); cursor: pointer;
		font-size: 0.8rem; text-decoration: underline;
	}

	.three-pane {
		display: grid; grid-template-columns: 220px 1fr 1fr;
		gap: 1rem; flex: 1; min-height: 0;
	}
	.pane {
		background: white; border-radius: 8px; overflow-y: auto;
		box-shadow: 0 1px 3px rgba(0,0,0,0.08);
	}
	.pane h3 {
		padding: 0.75rem 1rem; margin: 0; border-bottom: 1px solid var(--gray-200);
		font-size: 0.85rem; color: var(--gray-600); position: sticky; top: 0;
		background: white; z-index: 1;
	}

	.sender-item {
		display: flex; justify-content: space-between; align-items: center;
		padding: 0.6rem 1rem; width: 100%; border: none; background: none;
		cursor: pointer; text-align: left; border-bottom: 1px solid var(--gray-100);
	}
	.sender-item:hover { background: var(--gray-50); }
	.sender-item.active { background: var(--primary-light); }
	.sender-name { font-size: 0.85rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; }
	.sender-count { font-size: 0.75rem; color: var(--gray-500); background: var(--gray-100); padding: 0.1rem 0.4rem; border-radius: 8px; }

	.email-item-row {
		display: flex; align-items: flex-start; gap: 0.5rem;
		padding: 0.4rem 0.5rem; border-bottom: 1px solid var(--gray-100);
	}
	.email-item-row:hover { background: var(--gray-50); }
	.email-item-row.active { background: var(--primary-light); }
	.email-checkbox { margin-top: 0.5rem; flex-shrink: 0; cursor: pointer; }
	.email-item {
		display: block; padding: 0.2rem 0; flex: 1; border: none; background: none;
		cursor: pointer; text-align: left; min-width: 0;
	}
	.email-from { font-size: 0.8rem; color: var(--gray-500); }
	.email-subject { font-size: 0.9rem; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
	.email-date { font-size: 0.75rem; color: var(--gray-400); }
	.email-inline-domains { padding: 0.125rem 0 0.25rem 1.75rem; }

	.reading-pane-domains { margin-bottom: 0.75rem; }

	.reading-pane { padding: 1rem; }
	.email-detail h2 { margin-top: 0; font-size: 1.2rem; }
	.email-meta { font-size: 0.85rem; color: var(--gray-600); margin-bottom: 1rem; padding-bottom: 1rem; border-bottom: 1px solid var(--gray-200); }
	.email-meta div { margin-bottom: 0.2rem; }
	.email-body { font-size: 0.9rem; line-height: 1.6; overflow-wrap: break-word; }
	.email-body pre { white-space: pre-wrap; font-family: inherit; }

	.no-selection { display: flex; align-items: center; justify-content: center; height: 100%; color: var(--gray-400); }
	.empty-text { padding: 2rem; text-align: center; color: var(--gray-400); }

	@media (max-width: 900px) {
		.three-pane { grid-template-columns: 1fr; }
	}
</style>
