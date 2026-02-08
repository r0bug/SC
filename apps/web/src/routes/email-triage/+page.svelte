<script lang="ts">
	import { api } from '$lib/api/api';
	import { onMount } from 'svelte';
	import DomainTagInput from '$lib/components/DomainTagInput.svelte';

	type Step = 'config' | 'fetching' | 'analyzing' | 'review' | 'processing' | 'complete';

	let step: Step = 'config';
	let server = '';
	let folder = 'INBOX';
	let blockSize = 25;
	let sessionId = '';
	let totalFetched = 0;
	let totalBlocks = 0;
	let currentBlock = 0;
	let proposals: any[] = [];
	let approvedIds: string[] = [];
	let deniedIds: string[] = [];
	let autoApply = false;
	let error = '';
	let totalCategorized = 0;
	let loading = false;

	async function startTriage() {
		if (!server) { error = 'Server is required'; return; }
		error = '';
		loading = true;
		step = 'analyzing';

		try {
			const result = await api.startEmailTriage({ server, folder, block_size: blockSize });
			sessionId = result.session_id;
			totalFetched = result.total_fetched;
			totalBlocks = result.total_blocks;
			currentBlock = result.current_block;
			proposals = result.proposals || [];
			step = 'review';
		} catch (e: any) {
			error = e.message || 'Failed to start triage';
			step = 'config';
		} finally {
			loading = false;
		}
	}

	function toggleApprove(conceptId: string) {
		if (approvedIds.includes(conceptId)) {
			approvedIds = approvedIds.filter(id => id !== conceptId);
		} else {
			deniedIds = deniedIds.filter(id => id !== conceptId);
			approvedIds = [...approvedIds, conceptId];
		}
	}

	function toggleDeny(conceptId: string) {
		if (deniedIds.includes(conceptId)) {
			deniedIds = deniedIds.filter(id => id !== conceptId);
		} else {
			approvedIds = approvedIds.filter(id => id !== conceptId);
			deniedIds = [...deniedIds, conceptId];
		}
	}

	async function submitReview() {
		loading = true;
		error = '';
		try {
			await api.reviewTriageDomains(sessionId, { approved: approvedIds, denied: deniedIds });

			if (autoApply) {
				step = 'processing';
				const result = await api.continueTriage(sessionId, { auto_apply: true });
				totalCategorized = result.total_categorized || 0;
				step = 'complete';
			} else {
				// Continue to next block
				step = 'analyzing';
				const result = await api.continueTriage(sessionId, { auto_apply: false });
				if (result.status === 'completed') {
					step = 'complete';
				} else {
					currentBlock = result.current_block;
					proposals = result.proposals || [];
					approvedIds = [];
					deniedIds = [];
					step = 'review';
				}
			}
		} catch (e: any) {
			error = e.message || 'Failed to submit review';
		} finally {
			loading = false;
		}
	}

	function formatDate(ts: number) {
		if (!ts) return '';
		return new Date(ts).toLocaleDateString();
	}
</script>

<div class="page">
	<h1>AI Email Triage</h1>
	<p class="subtitle">AI analyzes your imported emails and discovers Communication Domains</p>

	{#if error}
		<div class="error-banner">{error}</div>
	{/if}

	<!-- Step 1: Config -->
	{#if step === 'config'}
		<div class="card config-card">
			<h2>Configure Triage</h2>
			<p>This wizard analyzes your already-imported emails and groups them into Communication Domains (categories like Personal, Bills, Work, etc.).</p>

			<div class="form-group">
				<label for="server">IMAP Server (for reference)</label>
				<input id="server" type="text" bind:value={server} placeholder="imap.gmail.com" />
			</div>
			<div class="form-group">
				<label for="folder">Folder</label>
				<input id="folder" type="text" bind:value={folder} />
			</div>
			<div class="form-group">
				<label for="blockSize">Block Size (emails per analysis batch)</label>
				<select id="blockSize" bind:value={blockSize}>
					<option value={10}>10 (Quick)</option>
					<option value={25}>25 (Balanced)</option>
					<option value={50}>50 (Thorough)</option>
				</select>
			</div>

			<button class="btn btn-primary" on:click={startTriage} disabled={loading}>
				{loading ? 'Starting...' : 'Start AI Triage'}
			</button>
		</div>
	{/if}

	<!-- Step 2/3: Analyzing -->
	{#if step === 'analyzing'}
		<div class="card">
			<h2>Analyzing Emails...</h2>
			<div class="progress-section">
				<div class="progress-bar">
					<div class="progress-fill" style="width: {totalBlocks > 0 ? ((currentBlock + 1) / totalBlocks * 100) : 0}%"></div>
				</div>
				<p>AI analyzing block {currentBlock + 1} of {totalBlocks} ({totalFetched} total emails)</p>
			</div>
		</div>
	{/if}

	<!-- Step 4: Review -->
	{#if step === 'review'}
		<div class="review-section">
			<div class="review-header">
				<h2>Review Proposed Domains (Block {currentBlock + 1} of {totalBlocks})</h2>
				<p>{proposals.length} domains discovered. Approve or deny each one.</p>
			</div>

			<div class="proposals-grid">
				{#each proposals as proposal}
					<div class="proposal-card"
						class:approved={approvedIds.includes(proposal.concept_id)}
						class:denied={deniedIds.includes(proposal.concept_id)}>

						<div class="proposal-header">
							<h3>{proposal.name}</h3>
							<div class="confidence-badge" title="AI Confidence">
								{Math.round((proposal.confidence || 0) * 100)}%
							</div>
						</div>

						<p class="proposal-description">{proposal.description}</p>

						<div class="proposal-meta">
							<span class="email-count">{proposal.email_count} emails</span>
						</div>

						{#if proposal.keywords && proposal.keywords.length > 0}
							<div class="keywords">
								{#each proposal.keywords as kw}
									<span class="keyword-tag">{kw}</span>
								{/each}
							</div>
						{/if}

						{#if proposal.sample_subjects && proposal.sample_subjects.length > 0}
							<div class="samples">
								<strong>Sample subjects:</strong>
								<ul>
									{#each proposal.sample_subjects as subj}
										<li>{subj}</li>
									{/each}
								</ul>
							</div>
						{/if}

						{#if proposal.sample_email_ids && proposal.sample_email_ids.length > 0}
							<div class="manual-domains">
								<span class="manual-label">Add domains manually:</span>
								<DomainTagInput
									emailIds={proposal.sample_email_ids}
									currentDomains={[]}
									compact={true}
								/>
							</div>
						{/if}

						<div class="proposal-actions">
							<button class="btn btn-approve" class:active={approvedIds.includes(proposal.concept_id)}
								on:click={() => toggleApprove(proposal.concept_id)}>
								Approve
							</button>
							<button class="btn btn-deny" class:active={deniedIds.includes(proposal.concept_id)}
								on:click={() => toggleDeny(proposal.concept_id)}>
								Deny
							</button>
						</div>
					</div>
				{/each}
			</div>

			<div class="review-footer">
				<label class="auto-apply-label">
					<input type="checkbox" bind:checked={autoApply} />
					Auto-apply approved domains to all remaining emails (skip further review)
				</label>
				<button class="btn btn-primary" on:click={submitReview} disabled={loading}>
					{loading ? 'Processing...' : autoApply ? 'Apply & Complete' : 'Continue to Next Block'}
				</button>
			</div>
		</div>
	{/if}

	<!-- Step 5: Processing -->
	{#if step === 'processing'}
		<div class="card">
			<h2>Applying Approved Domains...</h2>
			<p>Auto-categorizing remaining emails using approved domain keywords.</p>
			<div class="progress-bar">
				<div class="progress-fill progress-indeterminate"></div>
			</div>
		</div>
	{/if}

	<!-- Step 6: Complete -->
	{#if step === 'complete'}
		<div class="card complete-card">
			<h2>Triage Complete!</h2>
			<div class="complete-stats">
				<div class="stat">
					<span class="stat-value">{totalFetched}</span>
					<span class="stat-label">Emails Processed</span>
				</div>
				<div class="stat">
					<span class="stat-value">{approvedIds.length}</span>
					<span class="stat-label">Domains Approved</span>
				</div>
				{#if totalCategorized > 0}
					<div class="stat">
						<span class="stat-value">{totalCategorized}</span>
						<span class="stat-label">Auto-Categorized</span>
					</div>
				{/if}
			</div>

			<div class="complete-actions">
				<a href="/email-domains" class="btn btn-primary">View Email Domains</a>
				<a href="/email-explorer" class="btn btn-secondary">Open Email Explorer</a>
				<button class="btn btn-secondary" on:click={() => { step = 'config'; approvedIds = []; deniedIds = []; }}>
					Start New Triage
				</button>
			</div>
		</div>
	{/if}
</div>

<style>
	.page { padding: 2rem; max-width: 1200px; margin: 0 auto; }
	.subtitle { color: var(--gray-500); margin-bottom: 2rem; }

	.error-banner {
		background: #fee; color: #c00; padding: 1rem; border-radius: 8px;
		margin-bottom: 1rem; border: 1px solid #fcc;
	}

	.card {
		background: white; border-radius: 12px; padding: 2rem;
		box-shadow: 0 1px 3px rgba(0,0,0,0.1); margin-bottom: 1.5rem;
	}
	.card h2 { margin-top: 0; }

	.config-card { max-width: 500px; }
	.form-group { margin-bottom: 1rem; }
	.form-group label { display: block; font-weight: 500; margin-bottom: 0.25rem; }
	.form-group input, .form-group select {
		width: 100%; padding: 0.5rem; border: 1px solid var(--gray-300);
		border-radius: 6px; font-size: 1rem;
	}

	.progress-section { text-align: center; }
	.progress-bar {
		height: 8px; background: var(--gray-200); border-radius: 4px;
		overflow: hidden; margin: 1rem 0;
	}
	.progress-fill { height: 100%; background: var(--primary); transition: width 0.3s; }
	.progress-indeterminate {
		width: 30%; animation: indeterminate 1.5s infinite;
	}
	@keyframes indeterminate {
		0% { transform: translateX(-100%); }
		100% { transform: translateX(400%); }
	}

	.review-header { margin-bottom: 1.5rem; }
	.proposals-grid {
		display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
		gap: 1rem; margin-bottom: 1.5rem;
	}
	.proposal-card {
		background: white; border: 2px solid var(--gray-200); border-radius: 12px;
		padding: 1.25rem; transition: border-color 0.2s;
	}
	.proposal-card.approved { border-color: #22c55e; background: #f0fdf4; }
	.proposal-card.denied { border-color: #ef4444; background: #fef2f2; opacity: 0.7; }

	.proposal-header { display: flex; justify-content: space-between; align-items: center; }
	.proposal-header h3 { margin: 0; }
	.confidence-badge {
		background: var(--primary-light); color: var(--primary);
		padding: 0.25rem 0.5rem; border-radius: 12px; font-size: 0.8rem; font-weight: 600;
	}
	.proposal-description { color: var(--gray-600); font-size: 0.9rem; }
	.proposal-meta { display: flex; gap: 1rem; font-size: 0.85rem; color: var(--gray-500); margin-bottom: 0.5rem; }

	.keywords { display: flex; flex-wrap: wrap; gap: 0.25rem; margin-bottom: 0.75rem; }
	.keyword-tag {
		background: var(--gray-100); color: var(--gray-700); padding: 0.15rem 0.5rem;
		border-radius: 12px; font-size: 0.8rem;
	}

	.samples { font-size: 0.85rem; color: var(--gray-600); margin-bottom: 0.75rem; }
	.samples ul { margin: 0.25rem 0 0 1.25rem; padding: 0; }
	.samples li { margin-bottom: 0.15rem; }

	.manual-domains { margin-bottom: 0.75rem; }
	.manual-label { font-size: 0.75rem; color: var(--gray-500); display: block; margin-bottom: 0.25rem; }

	.proposal-actions { display: flex; gap: 0.5rem; }
	.btn-approve {
		background: #dcfce7; color: #166534; border: 1px solid #86efac;
		padding: 0.4rem 1rem; border-radius: 6px; cursor: pointer;
	}
	.btn-approve.active { background: #22c55e; color: white; }
	.btn-deny {
		background: #fef2f2; color: #991b1b; border: 1px solid #fca5a5;
		padding: 0.4rem 1rem; border-radius: 6px; cursor: pointer;
	}
	.btn-deny.active { background: #ef4444; color: white; }

	.review-footer {
		display: flex; justify-content: space-between; align-items: center;
		padding-top: 1rem; border-top: 1px solid var(--gray-200);
	}
	.auto-apply-label { display: flex; align-items: center; gap: 0.5rem; font-size: 0.9rem; }

	.complete-card { text-align: center; }
	.complete-stats { display: flex; justify-content: center; gap: 3rem; margin: 2rem 0; }
	.stat { display: flex; flex-direction: column; align-items: center; }
	.stat-value { font-size: 2rem; font-weight: 700; color: var(--primary); }
	.stat-label { font-size: 0.85rem; color: var(--gray-500); }
	.complete-actions { display: flex; gap: 1rem; justify-content: center; flex-wrap: wrap; }

	.btn { padding: 0.5rem 1.25rem; border-radius: 6px; font-size: 0.95rem; cursor: pointer; text-decoration: none; border: none; }
	.btn-primary { background: var(--primary); color: white; }
	.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
	.btn-secondary { background: var(--gray-100); color: var(--gray-700); border: 1px solid var(--gray-300); }
</style>
