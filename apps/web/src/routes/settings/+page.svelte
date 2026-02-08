<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { WorkerMetrics } from '$lib/api/types';

	let metrics: WorkerMetrics[] = [];
	let settings: { theme?: string; notifications?: boolean; sync_enabled?: boolean; [key: string]: string | boolean | number | undefined } = {};
	let loading = true;

	// AI settings
	let aiStatus: { configured: boolean; provider: string; model: string } | null = null;
	let aiApiKey = '';
	let aiSaving = false;
	let aiMessage = '';

	onMount(async () => {
		await Promise.all([loadMetrics(), loadSettings(), loadAiStatus()]);
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
			settings = await api.getSettings() as typeof settings;
		} catch (error) {
			console.error('Failed to load settings:', error);
		} finally {
			loading = false;
		}
	}

	async function loadAiStatus() {
		try {
			aiStatus = await api.getAiStatus();
		} catch (error) {
			console.error('Failed to load AI status:', error);
		}
	}

	async function handleSaveSettings() {
		try {
			await api.updateSettings(settings);
			alert('Settings saved');
		} catch (error: unknown) {
			alert('Failed: ' + (error instanceof Error ? error.message : String(error)));
		}
	}

	async function handleSaveAiKey() {
		aiSaving = true;
		aiMessage = '';
		try {
			const result = await api.updateAiKey(aiApiKey);
			aiMessage = result.message;
			aiApiKey = '';
			await loadAiStatus();
		} catch (error: unknown) {
			aiMessage = 'Failed: ' + (error instanceof Error ? error.message : String(error));
		} finally {
			aiSaving = false;
		}
	}

	async function handleClearAiKey() {
		aiSaving = true;
		aiMessage = '';
		try {
			const result = await api.updateAiKey('');
			aiMessage = result.message;
			await loadAiStatus();
		} catch (error: unknown) {
			aiMessage = 'Failed: ' + (error instanceof Error ? error.message : String(error));
		} finally {
			aiSaving = false;
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
					<label for="settings-theme">Theme</label>
					<select id="settings-theme" bind:value={settings.theme}>
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

			<div class="card">
				<h2>AI Configuration</h2>
				<div class="ai-status">
					{#if aiStatus}
						<div class="status-row">
							<span class="status-label">Status:</span>
							<span class="status-badge" class:active={aiStatus.configured} class:inactive={!aiStatus.configured}>
								{aiStatus.configured ? 'Active' : 'Not configured'}
							</span>
						</div>
						<div class="status-row">
							<span class="status-label">Provider:</span>
							<span>{aiStatus.provider}</span>
						</div>
						<div class="status-row">
							<span class="status-label">Model:</span>
							<span>{aiStatus.model}</span>
						</div>
					{:else}
						<p class="muted">Loading AI status...</p>
					{/if}
				</div>

				<div class="form-group">
					<label for="ai-key">Segmind API Key</label>
					<input
						id="ai-key"
						type="password"
						bind:value={aiApiKey}
						placeholder={aiStatus?.configured ? '••••••••  (key is set)' : 'Enter your Segmind API key'}
						class="key-input"
					/>
					<p class="hint">Get a key at <a href="https://www.segmind.com" target="_blank" rel="noopener">segmind.com</a>. Used for AI email triage and contact suggestions.</p>
				</div>

				{#if aiMessage}
					<div class="ai-message" class:success={aiMessage.includes('success') || aiMessage.includes('updated')} class:info={aiMessage.includes('cleared')}>
						{aiMessage}
					</div>
				{/if}

				<div class="btn-row">
					<button on:click={handleSaveAiKey} class="btn btn-primary" disabled={aiSaving || !aiApiKey.trim()}>
						{aiSaving ? 'Saving...' : 'Save API Key'}
					</button>
					{#if aiStatus?.configured}
						<button on:click={handleClearAiKey} class="btn btn-secondary" disabled={aiSaving}>
							Clear Key
						</button>
					{/if}
				</div>
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
	.form-group select, .key-input { width: 100%; padding: 0.75rem; border: 1px solid var(--gray-300); border-radius: 6px; font-size: 0.9rem; }
	.key-input { font-family: monospace; }
	.hint { font-size: 0.8rem; color: var(--gray-500); margin-top: 0.4rem; }
	.hint a { color: var(--primary); }
	.ai-status { padding: 1rem; background: var(--gray-50); border-radius: 6px; margin-bottom: 1.5rem; }
	.status-row { display: flex; align-items: center; gap: 0.5rem; margin-bottom: 0.4rem; font-size: 0.9rem; }
	.status-label { font-weight: 500; min-width: 70px; }
	.status-badge { padding: 0.15rem 0.6rem; border-radius: 12px; font-size: 0.8rem; font-weight: 600; }
	.status-badge.active { background: #d4edda; color: #155724; }
	.status-badge.inactive { background: #fff3cd; color: #856404; }
	.ai-message { padding: 0.75rem; border-radius: 6px; margin-bottom: 1rem; font-size: 0.875rem; }
	.ai-message.success { background: #d4edda; color: #155724; }
	.ai-message.info { background: #d1ecf1; color: #0c5460; }
	.btn-row { display: flex; gap: 0.75rem; }
	.muted { color: var(--gray-500); }
</style>
