<script lang="ts">
	import { api } from '$lib/api/api';
	import { onMount } from 'svelte';
	import DomainTagInput from '$lib/components/DomainTagInput.svelte';
	import BulkDomainToolbar from '$lib/components/BulkDomainToolbar.svelte';

	type ExplorerDomain = {
		concept_id: string;
		name: string;
		email_count: number;
	};

	type ExplorerEmail = {
		id: string;
		from_address: string;
		from_name: string | null;
		subject: string;
		message_date: number;
		body_preview: string | null;
		message_type: number;
		folder: string | null;
		domains: Array<{ concept_id: string; concept_name: string }>;
	};

	type Overlap = {
		domain_ids: string[];
		count: number;
	};

	let domains: ExplorerDomain[] = [];
	let selectedDomainIds: string[] = [];
	let filterMode: 'union' | 'intersection' = 'union';
	let viewMode: 'list' | 'venn' = 'list';
	let emails: ExplorerEmail[] = [];
	let overlaps: Overlap[] = [];
	let loading = false;
	let domainsLoading = true;
	let error = '';
	let totalShowing = 0;
	let checkedEmailIds: string[] = [];
	let emailDomainMap: Record<string, Array<{ concept_id: string; name: string }>> = {};

	onMount(async () => {
		try {
			const result = await api.getEmailDomains();
			domains = result.domains || [];
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : 'Failed to load domains';
		} finally {
			domainsLoading = false;
		}
	});

	async function applyFilter() {
		if (selectedDomainIds.length === 0) {
			emails = [];
			overlaps = [];
			return;
		}

		loading = true;
		error = '';
		try {
			const result = await api.filterEmailsByDomains({
				domain_ids: selectedDomainIds,
				mode: filterMode,
				limit: 100,
				offset: 0,
			});
			emails = result.emails || [];
			overlaps = result.overlaps || [];
			totalShowing = emails.length;
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : 'Failed to filter emails';
		} finally {
			loading = false;
		}
	}

	function toggleDomain(id: string) {
		if (selectedDomainIds.includes(id)) {
			selectedDomainIds = selectedDomainIds.filter(d => d !== id);
		} else {
			selectedDomainIds = [...selectedDomainIds, id];
		}
		applyFilter();
	}

	function setFilterMode(mode: 'union' | 'intersection') {
		filterMode = mode;
		if (selectedDomainIds.length > 0) applyFilter();
	}

	function getDomainName(id: string): string {
		return domains.find(d => d.concept_id === id)?.name || id;
	}

	function getDomainColor(index: number): string {
		const colors = ['#3b82f6', '#ef4444', '#22c55e', '#f59e0b', '#8b5cf6', '#ec4899', '#06b6d4', '#f97316'];
		return colors[index % colors.length];
	}

	// Compute Venn circle data for 2-3 selected domains
	$: vennData = (() => {
		if (selectedDomainIds.length < 2 || selectedDomainIds.length > 3) return null;

		const getCount = (ids: string[]) => {
			const sorted = [...ids].sort();
			const match = overlaps.find((o: Overlap) => {
				const oSorted = [...o.domain_ids].sort();
				return oSorted.length === sorted.length && oSorted.every((v: string, i: number) => v === sorted[i]);
			});
			return match?.count || 0;
		};

		if (selectedDomainIds.length === 2) {
			const a = getCount([selectedDomainIds[0]]);
			const b = getCount([selectedDomainIds[1]]);
			const ab = getCount([selectedDomainIds[0], selectedDomainIds[1]]);
			return {
				circles: [
					{ id: selectedDomainIds[0], name: getDomainName(selectedDomainIds[0]), total: a, exclusive: a - ab, cx: 160, cy: 150, r: 100 },
					{ id: selectedDomainIds[1], name: getDomainName(selectedDomainIds[1]), total: b, exclusive: b - ab, cx: 260, cy: 150, r: 100 },
				],
				intersection: ab,
			};
		} else {
			const a = getCount([selectedDomainIds[0]]);
			const b = getCount([selectedDomainIds[1]]);
			const c = getCount([selectedDomainIds[2]]);
			const ab = getCount([selectedDomainIds[0], selectedDomainIds[1]]);
			const ac = getCount([selectedDomainIds[0], selectedDomainIds[2]]);
			const bc = getCount([selectedDomainIds[1], selectedDomainIds[2]]);
			const abc = getCount([selectedDomainIds[0], selectedDomainIds[1], selectedDomainIds[2]]);
			return {
				circles: [
					{ id: selectedDomainIds[0], name: getDomainName(selectedDomainIds[0]), total: a, exclusive: a - ab - ac + abc, cx: 190, cy: 120, r: 90 },
					{ id: selectedDomainIds[1], name: getDomainName(selectedDomainIds[1]), total: b, exclusive: b - ab - bc + abc, cx: 260, cy: 120, r: 90 },
					{ id: selectedDomainIds[2], name: getDomainName(selectedDomainIds[2]), total: c, exclusive: c - ac - bc + abc, cx: 225, cy: 195, r: 90 },
				],
				pairs: [
					{ ids: [selectedDomainIds[0], selectedDomainIds[1]], count: ab - abc },
					{ ids: [selectedDomainIds[0], selectedDomainIds[2]], count: ac - abc },
					{ ids: [selectedDomainIds[1], selectedDomainIds[2]], count: bc - abc },
				],
				center: abc,
			};
		}
	})();

	function formatDate(ts: number) {
		if (!ts) return '';
		return new Date(ts).toLocaleDateString();
	}

	let selectedEmail: ExplorerEmail | null = null;

	function toggleEmailCheck(id: string) {
		if (checkedEmailIds.includes(id)) {
			checkedEmailIds = checkedEmailIds.filter(x => x !== id);
		} else {
			checkedEmailIds = [...checkedEmailIds, id];
		}
	}

	// Build domain map from email.domains for DomainTagInput
	$: {
		const map: Record<string, Array<{ concept_id: string; name: string }>> = {};
		for (const email of emails) {
			map[email.id] = (email.domains || []).map((d: { concept_id: string; concept_name: string }) => ({
				concept_id: d.concept_id,
				name: d.concept_name
			}));
		}
		emailDomainMap = map;
	}

	function handleBulkAssigned() {
		checkedEmailIds = [];
		applyFilter();
	}

	function handleEmailDomainsChanged() {
		applyFilter();
	}
</script>

<div class="explorer-layout">
	<!-- Left sidebar: Domain filters -->
	<div class="filter-sidebar">
		<h2>Domains</h2>

		{#if domainsLoading}
			<p class="loading-text">Loading...</p>
		{:else}
			{#each domains as domain, i}
				<label class="domain-checkbox" style="--domain-color: {getDomainColor(i)}">
					<input type="checkbox"
						checked={selectedDomainIds.includes(domain.concept_id)}
						on:change={() => toggleDomain(domain.concept_id)} />
					<span class="domain-indicator"></span>
					<span class="domain-label">
						{domain.name}
						<span class="domain-count">({domain.email_count})</span>
					</span>
				</label>
			{/each}
		{/if}

		{#if domains.length > 0}
			<div class="filter-controls">
				<h3>Mode</h3>
				<div class="mode-toggle">
					<button class:active={filterMode === 'union'} on:click={() => setFilterMode('union')}>
						Union (OR)
					</button>
					<button class:active={filterMode === 'intersection'} on:click={() => setFilterMode('intersection')}>
						Intersect (AND)
					</button>
				</div>

				<h3>View</h3>
				<div class="mode-toggle">
					<button class:active={viewMode === 'list'} on:click={() => viewMode = 'list'}>
						List
					</button>
					<button class:active={viewMode === 'venn'} on:click={() => viewMode = 'venn'}
						disabled={selectedDomainIds.length < 2 || selectedDomainIds.length > 3}>
						Venn
					</button>
				</div>
			</div>
		{/if}
	</div>

	<!-- Main area -->
	<div class="main-area">
		<!-- Stats bar -->
		<div class="stats-bar">
			<span>Showing <strong>{totalShowing}</strong> emails across <strong>{selectedDomainIds.length}</strong> domains</span>
			<span class="mode-label">{filterMode === 'union' ? 'Union (OR)' : 'Intersection (AND)'}</span>
		</div>

		{#if error}
			<div class="error-banner">{error}</div>
		{/if}

		{#if loading}
			<div class="loading-area">Filtering emails...</div>
		{:else if viewMode === 'venn' && vennData}
			<!-- Venn Diagram -->
			<div class="venn-container">
				<svg viewBox="0 0 420 300" class="venn-svg">
					{#each vennData.circles as circle, i}
						<circle cx={circle.cx} cy={circle.cy} r={circle.r}
							fill={getDomainColor(i)} fill-opacity="0.2"
							stroke={getDomainColor(i)} stroke-width="2" />
						<text x={circle.cx + (i === 0 ? -45 : i === 1 ? 45 : 0)}
							y={circle.cy + (i === 2 ? 45 : -30)}
							text-anchor="middle" font-size="12" font-weight="bold" fill={getDomainColor(i)}>
							{circle.name}
						</text>
						<text x={circle.cx + (i === 0 ? -45 : i === 1 ? 45 : 0)}
							y={circle.cy + (i === 2 ? 58 : -18)}
							text-anchor="middle" font-size="11" fill={getDomainColor(i)}>
							{circle.exclusive} exclusive
						</text>
					{/each}

					<!-- Intersection label -->
					{#if 'intersection' in vennData}
						<text x="210" y="155" text-anchor="middle" font-size="14" font-weight="bold" fill="#333">
							{vennData.intersection}
						</text>
						<text x="210" y="168" text-anchor="middle" font-size="10" fill="#666">overlap</text>
					{/if}

					{#if 'center' in vennData}
						<text x="225" y="155" text-anchor="middle" font-size="14" font-weight="bold" fill="#333">
							{vennData.center}
						</text>
						<text x="225" y="168" text-anchor="middle" font-size="10" fill="#666">all three</text>
					{/if}
				</svg>
			</div>
		{:else}
			<!-- Email list -->
			<div class="email-list">
				{#each emails as email}
					<div class="email-row" class:selected={selectedEmail?.id === email.id}>
						<div class="email-row-header">
							<input
								type="checkbox"
								checked={checkedEmailIds.includes(email.id)}
								on:change={() => toggleEmailCheck(email.id)}
								on:click|stopPropagation
								class="email-checkbox"
							/>
							<div class="email-row-main" role="button" tabindex="0" on:click={() => selectedEmail = selectedEmail?.id === email.id ? null : email} on:keydown={(e) => e.key === 'Enter' && (selectedEmail = selectedEmail?.id === email.id ? null : email)}>
								<span class="email-from">{email.from_name || email.from_address}</span>
								<span class="email-subject">{email.subject}</span>
								<span class="email-date">{formatDate(email.message_date)}</span>
							</div>
						</div>
						<div class="email-row-domains">
							<DomainTagInput
								emailId={email.id}
								currentDomains={emailDomainMap[email.id] || []}
								compact={true}
								on:domainsChanged={handleEmailDomainsChanged}
							/>
						</div>

						{#if selectedEmail?.id === email.id}
							<div class="email-preview">
								<div class="preview-meta">
									<div><strong>From:</strong> {email.from_name || ''} &lt;{email.from_address}&gt;</div>
									{#if email.folder}
										<div><strong>Folder:</strong> {email.folder}</div>
									{/if}
								</div>
								<div class="preview-body">{email.body_preview || '(No preview)'}</div>
							</div>
						{/if}
					</div>
				{/each}

				{#if emails.length === 0 && selectedDomainIds.length > 0}
					<div class="empty-text">No emails match the selected filters</div>
				{:else if selectedDomainIds.length === 0}
					<div class="empty-text">Select one or more domains from the sidebar to filter emails</div>
				{/if}
			</div>
		{/if}

		<BulkDomainToolbar
			selectedEmailIds={checkedEmailIds}
			on:clear={() => checkedEmailIds = []}
			on:assigned={handleBulkAssigned}
		/>
	</div>
</div>

<style>
	.explorer-layout {
		display: grid; grid-template-columns: 260px 1fr;
		height: 100vh; overflow: hidden;
	}

	.filter-sidebar {
		background: white; border-right: 1px solid var(--gray-200);
		padding: 1.25rem; overflow-y: auto;
	}
	.filter-sidebar h2 { margin-top: 0; font-size: 1.1rem; }
	.filter-sidebar h3 { font-size: 0.75rem; text-transform: uppercase; color: var(--gray-500); margin: 1rem 0 0.5rem; }

	.domain-checkbox {
		display: flex; align-items: center; gap: 0.5rem;
		padding: 0.4rem 0; cursor: pointer; font-size: 0.9rem;
	}
	.domain-checkbox input { display: none; }
	.domain-indicator {
		width: 16px; height: 16px; border-radius: 4px; flex-shrink: 0;
		border: 2px solid var(--domain-color); background: white;
	}
	.domain-checkbox input:checked + .domain-indicator {
		background: var(--domain-color);
	}
	.domain-label { flex: 1; }
	.domain-count { color: var(--gray-400); font-size: 0.8rem; }

	.filter-controls { margin-top: 1rem; }
	.mode-toggle { display: flex; gap: 0; }
	.mode-toggle button {
		flex: 1; padding: 0.4rem 0.5rem; font-size: 0.8rem; border: 1px solid var(--gray-300);
		background: white; cursor: pointer;
	}
	.mode-toggle button:first-child { border-radius: 6px 0 0 6px; }
	.mode-toggle button:last-child { border-radius: 0 6px 6px 0; border-left: none; }
	.mode-toggle button.active { background: var(--primary); color: white; border-color: var(--primary); }
	.mode-toggle button:disabled { opacity: 0.4; cursor: not-allowed; }

	.main-area { overflow-y: auto; background: var(--gray-50); }

	.stats-bar {
		display: flex; justify-content: space-between; align-items: center;
		padding: 0.75rem 1.5rem; background: white; border-bottom: 1px solid var(--gray-200);
		font-size: 0.85rem; color: var(--gray-600); position: sticky; top: 0; z-index: 1;
	}
	.mode-label {
		background: var(--gray-100); padding: 0.2rem 0.6rem; border-radius: 12px;
		font-size: 0.8rem;
	}

	.error-banner { background: #fee; color: #c00; padding: 1rem; margin: 1rem 1.5rem; border-radius: 8px; border: 1px solid #fcc; }
	.loading-area { text-align: center; padding: 3rem; color: var(--gray-500); }
	.loading-text { color: var(--gray-400); font-size: 0.85rem; }

	.venn-container {
		display: flex; justify-content: center; padding: 2rem;
	}
	.venn-svg { max-width: 500px; width: 100%; }

	.email-list { padding: 0.5rem 1rem; }
	.email-row {
		background: white; border-radius: 8px; padding: 0.75rem 1rem;
		margin-bottom: 0.5rem; cursor: pointer; border: 1px solid transparent;
		transition: border-color 0.15s;
	}
	.email-row:hover { border-color: var(--gray-300); }
	.email-row.selected { border-color: var(--primary); }

	.email-row-header { display: flex; align-items: flex-start; gap: 0.5rem; }
	.email-checkbox { margin-top: 0.25rem; flex-shrink: 0; cursor: pointer; }
	.email-row-main { display: flex; gap: 1rem; align-items: baseline; flex: 1; cursor: pointer; }
	.email-from { font-weight: 500; font-size: 0.85rem; min-width: 150px; color: var(--gray-700); }
	.email-subject { flex: 1; font-size: 0.9rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
	.email-date { font-size: 0.8rem; color: var(--gray-400); white-space: nowrap; }

	.email-row-domains { margin-top: 0.25rem; padding-left: 1.75rem; }

	.email-preview {
		margin-top: 0.75rem; padding-top: 0.75rem; border-top: 1px solid var(--gray-200);
		font-size: 0.85rem;
	}
	.preview-meta { color: var(--gray-600); margin-bottom: 0.5rem; }
	.preview-meta div { margin-bottom: 0.15rem; }
	.preview-body { color: var(--gray-700); line-height: 1.5; white-space: pre-wrap; }

	.empty-text { text-align: center; padding: 3rem; color: var(--gray-400); }

	@media (max-width: 768px) {
		.explorer-layout { grid-template-columns: 1fr; }
		.filter-sidebar { border-right: none; border-bottom: 1px solid var(--gray-200); }
	}
</style>
