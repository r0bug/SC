<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { api, ApiError } from '$lib/api/api';
	import type { LabelDetail, MatcherGroup, CommunicationSummaryItem } from '$lib/api/api';
	import { toasts } from '$lib/stores/toast';
	import { onMount } from 'svelte';
	import CriteriaBuilder from '$lib/components/CriteriaBuilder.svelte';

	let labelId = '';
	let detail: LabelDetail | null = null;
	let loading = true;
	let scanning = false;
	let activeTab = 'all';
	let commTypeFilter = 'all';
	let deleting = false;

	$: labelId = $page.params.id || '';
	$: if (labelId) loadDetail();

	async function loadDetail() {
		loading = true;
		try {
			detail = await api.getLabelDetail(labelId, {
				status: activeTab === 'all' ? undefined : activeTab,
				comm_type: commTypeFilter === 'all' ? undefined : commTypeFilter,
				limit: 50
			});
		} catch (e: any) {
			if (e instanceof ApiError) {
				toasts.error(e.getUserMessage());
			}
			detail = null;
		} finally {
			loading = false;
		}
	}

	async function runScan() {
		scanning = true;
		try {
			const result = await api.scanConcept(labelId);
			toasts.success(`Scan complete: ${result.new_suggestions} new suggestions`);
			await loadDetail();
		} catch (e: any) {
			toasts.error(e.message || 'Scan failed');
		} finally {
			scanning = false;
		}
	}

	async function confirmSuggestion(cc: CommunicationSummaryItem) {
		try {
			await api.updateConceptSuggestion(cc.communication_concept_id, 'confirmed');
			toasts.success('Confirmed');
			await loadDetail();
		} catch (e: any) {
			toasts.error(e.message || 'Failed to confirm');
		}
	}

	async function denySuggestion(cc: CommunicationSummaryItem) {
		try {
			await api.updateConceptSuggestion(cc.communication_concept_id, 'denied');
			await loadDetail();
		} catch (e: any) {
			toasts.error(e.message || 'Failed to deny');
		}
	}

	async function confirmAll() {
		if (!detail) return;
		const suggested = detail.communications.filter(c => c.status === 'suggested');
		for (const cc of suggested) {
			try {
				await api.updateConceptSuggestion(cc.communication_concept_id, 'confirmed');
			} catch { /* continue */ }
		}
		toasts.success(`Confirmed ${suggested.length} suggestions`);
		await loadDetail();
	}

	async function deleteLabel() {
		if (!confirm(`Delete label "${detail?.name}"? This will remove the label and all its criteria.`)) return;
		deleting = true;
		try {
			await api.deleteConcept(labelId);
			toasts.success('Label deleted');
			goto('/labels');
		} catch (e: any) {
			toasts.error(e.message || 'Failed to delete');
		} finally {
			deleting = false;
		}
	}

	function navigateToComm(cc: CommunicationSummaryItem) {
		if (cc.communication_type === 'email') {
			goto('/email-history');
		} else {
			goto('/sms-history');
		}
	}

	function setTab(tab: string) {
		activeTab = tab;
		loadDetail();
	}

	function formatDate(d: string): string {
		if (!d) return '';
		try {
			return new Date(d).toLocaleDateString('en-US', {
				year: 'numeric', month: 'short', day: 'numeric',
				hour: '2-digit', minute: '2-digit'
			});
		} catch { return d; }
	}
</script>

<div class="page">
	{#if loading && !detail}
		<div class="loading-state">
			<div class="spinner"></div>
			<p>Loading...</p>
		</div>
	{:else if detail}
		<div class="header">
			<div class="header-left">
				<a href="/labels" class="back-link">&larr; Labels</a>
				<h1>{detail.name}</h1>
				{#if detail.description}
					<p class="description">{detail.description}</p>
				{/if}
			</div>
			<div class="header-actions">
				<button class="btn btn-secondary" on:click={runScan} disabled={scanning}>
					{scanning ? 'Scanning...' : 'Run Scan'}
				</button>
				<button class="btn btn-danger" on:click={deleteLabel} disabled={deleting}>
					Delete
				</button>
			</div>
		</div>

		<div class="content-grid">
			<section class="criteria-section">
				<h2>Criteria</h2>
				<CriteriaBuilder
					conceptId={labelId}
					groups={detail.matcher_groups}
					readonly={false}
					on:save={() => loadDetail()}
					on:scan={() => loadDetail()}
				/>
			</section>

			<section class="matches-section">
				<div class="matches-header">
					<h2>Matches</h2>
					<div class="stats">
						<span class="stat confirmed">{detail.confirmed_count} confirmed</span>
						<span class="stat suggested">{detail.suggested_count} suggested</span>
					</div>
				</div>

				<div class="tabs">
					<button class="tab" class:active={activeTab === 'all'} on:click={() => setTab('all')}>All</button>
					<button class="tab" class:active={activeTab === 'confirmed'} on:click={() => setTab('confirmed')}>Confirmed</button>
					<button class="tab" class:active={activeTab === 'suggested'} on:click={() => setTab('suggested')}>Suggested</button>

					<select class="type-filter" bind:value={commTypeFilter} on:change={() => loadDetail()}>
						<option value="all">All Types</option>
						<option value="email">Email</option>
						<option value="sms">SMS</option>
					</select>

					{#if activeTab === 'suggested' && detail.communications.some(c => c.status === 'suggested')}
						<button class="btn btn-sm btn-confirm-all" on:click={confirmAll}>
							Confirm All
						</button>
					{/if}
				</div>

				{#if detail.communications.length === 0}
					<div class="empty-matches">
						<p>No matches found. Try running a scan.</p>
					</div>
				{:else}
					<div class="match-list">
						{#each detail.communications as comm}
							<div class="match-item">
								<div class="match-icon" class:email={comm.communication_type === 'email'} class:sms={comm.communication_type === 'sms'}>
									{comm.communication_type === 'email' ? '&#9993;' : '&#128172;'}
								</div>
								<div class="match-info" on:click={() => navigateToComm(comm)}>
									<div class="match-sender">{comm.sender}</div>
									{#if comm.subject}
										<div class="match-subject">{comm.subject}</div>
									{/if}
									{#if comm.preview}
										<div class="match-preview">{comm.preview}</div>
									{/if}
									<div class="match-date">{formatDate(comm.date)}</div>
								</div>
								<div class="match-actions">
									{#if comm.status === 'suggested'}
										<span class="confidence-badge" title="Confidence: {((comm.confidence || 0) * 100).toFixed(0)}%">
											{((comm.confidence || 0) * 100).toFixed(0)}%
										</span>
										<button class="btn-icon confirm" on:click={() => confirmSuggestion(comm)} title="Confirm">&#10003;</button>
										<button class="btn-icon deny" on:click={() => denySuggestion(comm)} title="Deny">&#10005;</button>
									{:else}
										<span class="status-badge confirmed">confirmed</span>
									{/if}
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</section>
		</div>
	{:else}
		<div class="error-state">
			<p>Label not found.</p>
			<a href="/labels">Back to Labels</a>
		</div>
	{/if}
</div>

<style>
	.page {
		padding: 2rem;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: 1.5rem;
	}

	.header-left {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.back-link {
		font-size: 0.85rem;
		color: #6b7280;
		text-decoration: none;
	}

	.back-link:hover {
		color: #3b82f6;
	}

	.header h1 {
		margin: 0;
		font-size: 1.5rem;
	}

	.description {
		color: #6b7280;
		font-size: 0.9rem;
		margin: 0;
	}

	.header-actions {
		display: flex;
		gap: 0.5rem;
	}

	.content-grid {
		display: grid;
		grid-template-columns: 1fr 1.5fr;
		gap: 1.5rem;
		align-items: start;
	}

	@media (max-width: 900px) {
		.content-grid {
			grid-template-columns: 1fr;
		}
	}

	.criteria-section, .matches-section {
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
		padding: 1rem;
	}

	.criteria-section h2, .matches-section h2 {
		margin: 0 0 0.75rem;
		font-size: 1.1rem;
	}

	.matches-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.5rem;
	}

	.stats {
		display: flex;
		gap: 0.75rem;
	}

	.stat {
		font-size: 0.8rem;
		font-weight: 500;
	}

	.stat.confirmed { color: #16a34a; }
	.stat.suggested { color: #d97706; }

	.tabs {
		display: flex;
		gap: 0.25rem;
		align-items: center;
		margin-bottom: 0.75rem;
		flex-wrap: wrap;
	}

	.tab {
		padding: 0.35rem 0.75rem;
		border: 1px solid #d1d5db;
		border-radius: 0.25rem;
		background: white;
		cursor: pointer;
		font-size: 0.8rem;
	}

	.tab.active {
		background: #3b82f6;
		color: white;
		border-color: #3b82f6;
	}

	.type-filter {
		margin-left: auto;
		padding: 0.3rem 0.5rem;
		border: 1px solid #d1d5db;
		border-radius: 0.25rem;
		font-size: 0.8rem;
	}

	.btn-confirm-all {
		padding: 0.3rem 0.6rem;
		font-size: 0.75rem;
		background: #dcfce7;
		color: #16a34a;
		border: 1px solid #86efac;
		border-radius: 0.25rem;
		cursor: pointer;
	}

	.match-list {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.match-item {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem;
		border: 1px solid #f3f4f6;
		border-radius: 0.375rem;
		transition: background 0.1s;
	}

	.match-item:hover {
		background: #f9fafb;
	}

	.match-icon {
		font-size: 1.2rem;
		width: 2rem;
		text-align: center;
		flex-shrink: 0;
	}

	.match-info {
		flex: 1;
		min-width: 0;
		cursor: pointer;
	}

	.match-sender {
		font-weight: 500;
		font-size: 0.85rem;
		color: #111827;
	}

	.match-subject {
		font-size: 0.8rem;
		color: #374151;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.match-preview {
		font-size: 0.75rem;
		color: #9ca3af;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.match-date {
		font-size: 0.7rem;
		color: #9ca3af;
	}

	.match-actions {
		display: flex;
		align-items: center;
		gap: 0.3rem;
		flex-shrink: 0;
	}

	.confidence-badge {
		font-size: 0.7rem;
		background: #fef3c7;
		color: #92400e;
		padding: 0.1rem 0.3rem;
		border-radius: 0.25rem;
	}

	.status-badge {
		font-size: 0.7rem;
		padding: 0.1rem 0.3rem;
		border-radius: 0.25rem;
	}

	.status-badge.confirmed {
		background: #dcfce7;
		color: #16a34a;
	}

	.btn-icon {
		border: none;
		background: none;
		cursor: pointer;
		font-size: 1rem;
		padding: 0.2rem;
		border-radius: 0.25rem;
	}

	.btn-icon.confirm { color: #16a34a; }
	.btn-icon.confirm:hover { background: #dcfce7; }
	.btn-icon.deny { color: #dc2626; }
	.btn-icon.deny:hover { background: #fee2e2; }

	.empty-matches {
		text-align: center;
		padding: 2rem;
		color: #9ca3af;
	}

	.loading-state, .error-state {
		text-align: center;
		padding: 3rem;
		color: #6b7280;
	}

	.spinner {
		width: 24px;
		height: 24px;
		border: 3px solid #e5e7eb;
		border-top-color: #3b82f6;
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
		margin: 0 auto 0.5rem;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	.btn {
		padding: 0.5rem 1rem;
		border-radius: 0.375rem;
		border: 1px solid #d1d5db;
		background: white;
		cursor: pointer;
		font-size: 0.85rem;
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-primary {
		background: #3b82f6;
		color: white;
		border-color: #3b82f6;
	}

	.btn-secondary {
		background: #f3f4f6;
		color: #374151;
	}

	.btn-danger {
		background: #fee2e2;
		color: #dc2626;
		border-color: #fca5a5;
	}

	.btn-danger:hover:not(:disabled) {
		background: #fecaca;
	}

	.btn-sm {
		padding: 0.3rem 0.6rem;
		font-size: 0.8rem;
	}
</style>
