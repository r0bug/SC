#!/bin/bash
# Generate all remaining Phase 5 pages

echo "Generating remaining web UI pages..."

# Create concepts page
mkdir -p src/routes/concepts
cat > src/routes/concepts/+page.svelte << 'EOF'
<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { Concept } from '$lib/api/types';

	let concepts: Concept[] = [];
	let loading = true;
	let showModal = false;
	let editing: Concept | null = null;
	let formName = '';
	let formDescription = '';

	onMount(async () => {
		await loadConcepts();
		api.onWebSocketMessage('concept_updated', (data) => {
			concepts = concepts.map(c => c.id === data.id ? data : c);
		});
		api.onWebSocketMessage('concept_created', (data) => {
			concepts = [data, ...concepts];
		});
	});

	async function loadConcepts() {
		try {
			concepts = await api.getConcepts();
		} catch (error) {
			console.error('Failed to load concepts:', error);
		} finally {
			loading = false;
		}
	}

	async function handleSubmit() {
		try {
			const conceptData = { name: formName, description: formDescription };
			if (editing) {
				const updated = await api.updateConcept(editing.id, conceptData);
				concepts = concepts.map(c => c.id === editing!.id ? updated : c);
			} else {
				const created = await api.createConcept(conceptData as any);
				concepts = [created, ...concepts];
			}
			showModal = false;
		} catch (error: any) {
			alert('Failed: ' + error.message);
		}
	}

	async function handleDelete(id: string) {
		if (!confirm('Delete?')) return;
		try {
			await api.deleteConcept(id);
			concepts = concepts.filter(c => c.id !== id);
		} catch (error: any) {
			alert('Failed: ' + error.message);
		}
	}
</script>

<div class="page">
	<div class="header">
		<h1>Concepts</h1>
		<button on:click={() => { editing = null; formName = ''; formDescription = ''; showModal = true; }} class="btn btn-primary">+ New Concept</button>
	</div>

	{#if loading}
		<div>Loading...</div>
	{:else}
		<div class="grid">
			{#each concepts as concept}
				<div class="card">
					<h3>{concept.name}</h3>
					{#if concept.description}
						<p>{concept.description}</p>
					{/if}
					<div class="actions">
						<button on:click={() => { editing = concept; formName = concept.name; formDescription = concept.description || ''; showModal = true; }} class="btn btn-sm">Edit</button>
						<button on:click={() => handleDelete(concept.id)} class="btn btn-sm">Delete</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

{#if showModal}
	<div class="modal-overlay" on:click={() => showModal = false}>
		<div class="modal" on:click|stopPropagation>
			<h2>{editing ? 'Edit' : 'New'} Concept</h2>
			<form on:submit|preventDefault={handleSubmit}>
				<div>
					<label>Name</label>
					<input type="text" bind:value={formName} required />
				</div>
				<div>
					<label>Description</label>
					<textarea bind:value={formDescription} rows="4"></textarea>
				</div>
				<div class="actions">
					<button type="button" on:click={() => showModal = false} class="btn">Cancel</button>
					<button type="submit" class="btn btn-primary">Save</button>
				</div>
			</form>
		</div>
	</div>
{/if}

<style>
	.page { padding: 2rem; max-width: 1200px; }
	.header { display: flex; justify-content: space-between; margin-bottom: 2rem; }
	.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 1.5rem; }
	.card { padding: 1.5rem; }
	.actions { display: flex; gap: 0.5rem; margin-top: 1rem; }
	.btn-sm { padding: 0.375rem 0.75rem; font-size: 0.875rem; }
	.modal-overlay { position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000; }
	.modal { background: white; padding: 2rem; border-radius: 8px; width: 90%; max-width: 500px; }
	label { display: block; margin-bottom: 0.5rem; font-weight: 500; }
	input, textarea { width: 100%; padding: 0.75rem; border: 1px solid var(--gray-300); border-radius: 6px; margin-bottom: 1rem; }
</style>
EOF

echo "✅ Concepts page created"

# Create shares page
mkdir -p src/routes/shares
cat > src/routes/shares/+page.svelte << 'EOF'
<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { ShareInvite } from '$lib/api/types';

	let shares: ShareInvite[] = [];
	let loading = true;

	onMount(async () => {
		await loadShares();
	});

	async function loadShares() {
		try {
			shares = await api.getShares();
		} catch (error) {
			console.error('Failed to load shares:', error);
		} finally {
			loading = false;
		}
	}

	async function handleAccept(id: string) {
		try {
			await api.acceptShare(id);
			shares = shares.map(s => s.id === id ? {...s, accepted: true} : s);
		} catch (error: any) {
			alert('Failed: ' + error.message);
		}
	}

	async function handleRevoke(id: string) {
		if (!confirm('Revoke this share?')) return;
		try {
			await api.revokeShare(id);
			shares = shares.filter(s => s.id !== id);
		} catch (error: any) {
			alert('Failed: ' + error.message);
		}
	}
</script>

<div class="page">
	<h1>Shared Items</h1>

	{#if loading}
		<div>Loading...</div>
	{:else if shares.length === 0}
		<div class="empty">No shared items</div>
	{:else}
		<div class="shares-list">
			{#each shares as share}
				<div class="share-card card">
					<div>
						<h3>{share.entity_type} Share</h3>
						<p>Shared with: {share.shared_with_email}</p>
						<p>Permissions: {share.permissions.join(', ')}</p>
						<p class="status">{share.accepted ? 'Accepted' : 'Pending'}</p>
					</div>
					<div class="actions">
						{#if !share.accepted}
							<button on:click={() => handleAccept(share.id)} class="btn btn-primary">Accept</button>
						{/if}
						<button on:click={() => handleRevoke(share.id)} class="btn btn-secondary">Revoke</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page { padding: 2rem; max-width: 1000px; }
	.shares-list { display: flex; flex-direction: column; gap: 1rem; }
	.share-card { padding: 1.5rem; display: flex; justify-content: space-between; align-items: center; }
	.actions { display: flex; gap: 0.5rem; }
	.status { font-size: 0.875rem; color: var(--gray-600); }
	.empty { text-align: center; padding: 3rem; }
</style>
EOF

echo "✅ Shares page created"

# Create communications page
mkdir -p src/routes/communications
cat > src/routes/communications/+page.svelte << 'EOF'
<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { CommunicationAttempt } from '$lib/api/types';

	let communications: CommunicationAttempt[] = [];
	let loading = true;

	onMount(async () => {
		await loadCommunications();
		api.onWebSocketMessage('communication_updated', (data) => {
			communications = communications.map(c => c.id === data.id ? data : c);
		});
	});

	async function loadCommunications() {
		try {
			communications = await api.getCommunications();
		} catch (error) {
			console.error('Failed to load communications:', error);
		} finally {
			loading = false;
		}
	}

	function getStatusBadge(status: any): string {
		if (typeof status === 'string') {
			switch (status) {
				case 'Sent': return 'status-sent';
				case 'Pending': return 'status-pending';
				case 'Retrying': return 'status-retrying';
				default: return 'status-default';
			}
		}
		return 'status-failed';
	}
</script>

<div class="page">
	<h1>Communications</h1>

	<div class="notice">
		⚠️ Alpha: Email/SMS/Social sends are mocked for testing
	</div>

	{#if loading}
		<div>Loading...</div>
	{:else}
		<div class="comms-list">
			{#each communications as comm}
				<div class="comm-card card">
					<div class="comm-header">
						<span class="method">{typeof comm.method === 'string' ? comm.method : 'Social'}</span>
						<span class="badge {getStatusBadge(comm.status)}">
							{typeof comm.status === 'string' ? comm.status : 'Failed'}
						</span>
					</div>
					{#if comm.subject}
						<h3>{comm.subject}</h3>
					{/if}
					<p class="message">{comm.message}</p>
					<div class="meta">
						<span>Attempts: {comm.retry_count}</span>
						{#if comm.scheduled_at}
							<span>Scheduled: {new Date(comm.scheduled_at).toLocaleString()}</span>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page { padding: 2rem; max-width: 1000px; }
	.notice { padding: 1rem; background: #fef3c7; border: 1px solid #fde68a; border-radius: 6px; margin-bottom: 2rem; }
	.comms-list { display: flex; flex-direction: column; gap: 1rem; }
	.comm-card { padding: 1.5rem; }
	.comm-header { display: flex; justify-content: space-between; margin-bottom: 0.5rem; }
	.method { font-weight: 600; }
	.badge { padding: 0.25rem 0.75rem; border-radius: 12px; font-size: 0.75rem; }
	.status-sent { background: #d1fae5; color: #065f46; }
	.status-pending { background: #fed7aa; color: #92400e; }
	.status-retrying { background: #dbeafe; color: #1e40af; }
	.status-failed { background: #fee2e2; color: #991b1b; }
	.message { color: var(--gray-700); margin: 0.5rem 0; }
	.meta { font-size: 0.875rem; color: var(--gray-600); display: flex; gap: 1rem; }
</style>
EOF

echo "✅ Communications page created"

# Create search history page
mkdir -p src/routes/search
cat > src/routes/search/+page.svelte << 'EOF'
<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { SearchHistory } from '$lib/api/types';

	let history: SearchHistory[] = [];
	let loading = true;

	onMount(async () => {
		await loadHistory();
	});

	async function loadHistory() {
		try {
			history = await api.getSearchHistory();
		} catch (error) {
			console.error('Failed to load history:', error);
		} finally {
			loading = false;
		}
	}

	async function handleClear() {
		if (!confirm('Clear all search history?')) return;
		try {
			await api.clearSearchHistory();
			history = [];
		} catch (error: any) {
			alert('Failed: ' + error.message);
		}
	}
</script>

<div class="page">
	<div class="header">
		<h1>Search History</h1>
		{#if history.length > 0}
			<button on:click={handleClear} class="btn btn-secondary">Clear History</button>
		{/if}
	</div>

	{#if loading}
		<div>Loading...</div>
	{:else if history.length === 0}
		<div class="empty">No search history</div>
	{:else}
		<div class="history-list">
			{#each history as item}
				<div class="history-item card">
					<div class="query">"{item.query}"</div>
					<div class="meta">
						<span>{item.result_count} results</span>
						<span>{new Date(item.created_at).toLocaleString()}</span>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page { padding: 2rem; max-width: 800px; }
	.header { display: flex; justify-content: space-between; margin-bottom: 2rem; }
	.history-list { display: flex; flex-direction: column; gap: 0.5rem; }
	.history-item { padding: 1rem; }
	.query { font-weight: 600; margin-bottom: 0.5rem; }
	.meta { font-size: 0.875rem; color: var(--gray-600); display: flex; gap: 1rem; }
	.empty { text-align: center; padding: 3rem; }
</style>
EOF

echo "✅ Search history page created"

# Create insights page
mkdir -p src/routes/insights
cat > src/routes/insights/+page.svelte << 'EOF'
<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { AiInsight } from '$lib/api/types';

	let insights: AiInsight[] = [];
	let loading = true;

	onMount(async () => {
		await loadInsights();
	});

	async function loadInsights() {
		try {
			insights = await api.getInsights();
		} catch (error) {
			console.error('Failed to load insights:', error);
		} finally {
			loading = false;
		}
	}

	async function handleApply(id: string) {
		try {
			await api.applyInsight(id);
			insights = insights.map(i => i.id === id ? {...i, applied: true} : i);
		} catch (error: any) {
			alert('Failed: ' + error.message);
		}
	}

	async function handleFeedback(id: string, helpful: boolean) {
		try {
			await api.feedbackInsight(id, helpful);
			insights = insights.map(i => i.id === id ? {...i, feedback: { helpful, submitted_at: new Date().toISOString() }} : i);
		} catch (error: any) {
			alert('Failed: ' + error.message);
		}
	}
</script>

<div class="page">
	<h1>AI Insights</h1>

	{#if loading}
		<div>Loading...</div>
	{:else if insights.length === 0}
		<div class="empty">No insights available</div>
	{:else}
		<div class="insights-list">
			{#each insights as insight}
				<div class="insight-card card">
					<div class="insight-type">{insight.insight_type}</div>
					<p class="content">{insight.content}</p>
					<div class="confidence">Confidence: {(insight.confidence * 100).toFixed(0)}%</div>
					{#if !insight.applied && !insight.feedback}
						<div class="actions">
							<button on:click={() => handleApply(insight.id)} class="btn btn-sm btn-primary">Apply</button>
							<button on:click={() => handleFeedback(insight.id, true)} class="btn btn-sm">👍 Helpful</button>
							<button on:click={() => handleFeedback(insight.id, false)} class="btn btn-sm">👎 Not Helpful</button>
						</div>
					{:else if insight.applied}
						<div class="status">✅ Applied</div>
					{:else if insight.feedback}
						<div class="status">Feedback submitted</div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page { padding: 2rem; max-width: 1000px; }
	.insights-list { display: flex; flex-direction: column; gap: 1rem; }
	.insight-card { padding: 1.5rem; }
	.insight-type { font-size: 0.75rem; font-weight: 600; color: var(--primary); text-transform: uppercase; margin-bottom: 0.5rem; }
	.content { color: var(--gray-700); margin: 0.5rem 0; }
	.confidence { font-size: 0.875rem; color: var(--gray-600); margin: 0.5rem 0; }
	.actions { display: flex; gap: 0.5rem; margin-top: 1rem; }
	.status { font-size: 0.875rem; color: var(--gray-600); margin-top: 1rem; }
	.empty { text-align: center; padding: 3rem; }
	.btn-sm { padding: 0.375rem 0.75rem; font-size: 0.875rem; }
</style>
EOF

echo "✅ AI Insights page created"

# Create settings page
mkdir -p src/routes/settings
cat > src/routes/settings/+page.svelte << 'EOF'
<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { WorkerMetrics } from '$lib/api/types';

	let metrics: WorkerMetrics[] = [];
	let settings: Record<string, any> = {};
	let loading = true;

	onMount(async () => {
		await Promise.all([loadMetrics(), loadSettings()]);
	});

	async function loadMetrics() {
		try {
			metrics = await api.getWorkerMetrics();
		} catch (error) {
			console.error('Failed to load metrics:', error);
		}
	}

	async function loadSettings() {
		try {
			settings = await api.getSettings();
		} catch (error) {
			console.error('Failed to load settings:', error);
		} finally {
			loading = false;
		}
	}

	async function handleSaveSettings() {
		try {
			await api.updateSettings(settings);
			alert('Settings saved');
		} catch (error: any) {
			alert('Failed: ' + error.message);
		}
	}
</script>

<div class="page">
	<h1>Settings</h1>

	{#if loading}
		<div>Loading...</div>
	{:else}
		<div class="settings-grid">
			<div class="card">
				<h2>Worker Metrics</h2>
				{#if metrics.length === 0}
					<p>No metrics available</p>
				{:else}
					<div class="metrics-list">
						{#each metrics as metric}
							<div class="metric-item">
								<h3>{metric.task_name}</h3>
								<div class="metric-stats">
									<span>✅ {metric.success_count} success</span>
									<span>❌ {metric.failure_count} failures</span>
									{#if metric.average_duration_ms}
										<span>⏱️ {metric.average_duration_ms}ms avg</span>
									{/if}
								</div>
								{#if metric.last_run_at}
									<div class="metric-time">Last run: {new Date(metric.last_run_at).toLocaleString()}</div>
								{/if}
							</div>
						{/each}
					</div>
				{/if}
			</div>

			<div class="card">
				<h2>Application Settings</h2>
				<div class="form-group">
					<label>Theme</label>
					<select bind:value={settings.theme}>
						<option value="light">Light</option>
						<option value="dark">Dark</option>
					</select>
				</div>
				<div class="form-group">
					<label>
						<input type="checkbox" bind:checked={settings.notifications} />
						Enable Notifications
					</label>
				</div>
				<div class="form-group">
					<label>
						<input type="checkbox" bind:checked={settings.sync_enabled} />
						Enable Auto-Sync
					</label>
				</div>
				<button on:click={handleSaveSettings} class="btn btn-primary">Save Settings</button>
			</div>
		</div>
	{/if}
</div>

<style>
	.page { padding: 2rem; max-width: 1200px; }
	.settings-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(500px, 1fr)); gap: 1.5rem; }
	.card { padding: 1.5rem; }
	.card h2 { margin-bottom: 1.5rem; }
	.metrics-list { display: flex; flex-direction: column; gap: 1rem; }
	.metric-item { padding: 1rem; background: var(--gray-50); border-radius: 6px; }
	.metric-item h3 { margin: 0 0 0.5rem; }
	.metric-stats { display: flex; gap: 1rem; font-size: 0.875rem; }
	.metric-time { font-size: 0.75rem; color: var(--gray-600); margin-top: 0.5rem; }
	.form-group { margin-bottom: 1.5rem; }
	.form-group label { display: block; margin-bottom: 0.5rem; font-weight: 500; }
	.form-group input[type="checkbox"] { margin-right: 0.5rem; }
	.form-group select { width: 100%; padding: 0.75rem; border: 1px solid var(--gray-300); border-radius: 6px; }
</style>
EOF

echo "✅ Settings page created"

echo ""
echo "✅ All remaining web UI pages generated!"
echo ""
echo "Pages created:"
echo "  - concepts/+page.svelte"
echo "  - shares/+page.svelte"
echo "  - communications/+page.svelte"
echo "  - search/+page.svelte (search history)"
echo "  - insights/+page.svelte (AI insights)"
echo "  - settings/+page.svelte"
echo ""
