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
