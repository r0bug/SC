<script lang="ts">
	import { onMount } from 'svelte';
	import { tauriApi } from '$lib/api/tauri-api';

	let dashboard: any = null;
	let online = false;
	let syncing = false;
	let loading = true;

	onMount(async () => {
		await loadDashboard();
		await checkConnectivity();
	});

	async function loadDashboard() {
		try {
			dashboard = await tauriApi.getDashboard();
		} catch (error) {
			console.error('Failed to load dashboard:', error);
		} finally {
			loading = false;
		}
	}

	async function checkConnectivity() {
		try {
			online = await tauriApi.checkOnline();
		} catch (error) {
			console.error('Failed to check connectivity:', error);
			online = false;
		}
	}

	async function handleSync() {
		if (!online) {
			alert('Cannot sync: No internet connection');
			return;
		}

		try {
			syncing = true;
			const result = await tauriApi.syncWithServer();
			await loadDashboard();
			alert(`Synced: ${result.synced_contacts} contacts, ${result.synced_projects} projects`);
		} catch (error: any) {
			alert('Sync failed: ' + error);
		} finally {
			syncing = false;
		}
	}
</script>

<div class="page">
	<div class="header">
		<h1>SagensContact Desktop</h1>
		<div class="status-bar">
			<span class="status {online ? 'online' : 'offline'}">
				{online ? '● Online' : '● Offline'}
			</span>
			<button on:click={handleSync} class="btn btn-primary" disabled={!online || syncing}>
				{syncing ? 'Syncing...' : 'Sync Now'}
			</button>
		</div>
	</div>

	{#if loading}
		<div class="loading">Loading dashboard...</div>
	{:else if dashboard}
		<div class="dashboard-grid">
			<a href="/contacts" class="dashboard-card card">
				<h2>Contacts</h2>
				<div class="stat">{dashboard.total_contacts}</div>
				<p>Total contacts in your network</p>
			</a>

			<a href="/projects" class="dashboard-card card">
				<h2>Projects</h2>
				<div class="stat">{dashboard.total_projects}</div>
				<p>Total projects</p>
			</a>

			<a href="/groups" class="dashboard-card card">
				<h2>Groups</h2>
				<div class="stat">{dashboard.total_groups}</div>
				<p>Contact groups</p>
			</a>

			<a href="/calendar" class="dashboard-card card">
				<h2>Upcoming Events</h2>
				<div class="stat">{dashboard.upcoming_events}</div>
				<p>Events scheduled soon</p>
			</a>
		</div>

		<div class="quick-actions">
			<h2>Quick Actions</h2>
			<div class="actions-grid">
				<a href="/contacts/new" class="action-btn card">
					<span class="icon">+</span>
					<span>Add Contact</span>
				</a>
				<a href="/projects/new" class="action-btn card">
					<span class="icon">📋</span>
					<span>New Project</span>
				</a>
				<a href="/calendar/new" class="action-btn card">
					<span class="icon">📅</span>
					<span>Schedule Event</span>
				</a>
				<a href="/import" class="action-btn card">
					<span class="icon">📥</span>
					<span>Import Data</span>
				</a>
			</div>
		</div>

		{#if !online}
			<div class="offline-notice card">
				⚠️ You're currently offline. Changes will sync when you reconnect.
			</div>
		{/if}
	{/if}
</div>

<style>
	.page { padding: 2rem; max-width: 1200px; margin: 0 auto; }
	.header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 2rem; }
	.status-bar { display: flex; align-items: center; gap: 1rem; }
	.status { font-weight: 600; font-size: 0.875rem; }
	.status.online { color: #10b981; }
	.status.offline { color: #ef4444; }

	.dashboard-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
		gap: 1.5rem;
		margin-bottom: 2rem;
	}

	.dashboard-card {
		padding: 1.5rem;
		text-decoration: none;
		color: inherit;
		transition: transform 0.2s;
	}

	.dashboard-card:hover {
		transform: translateY(-2px);
	}

	.dashboard-card h2 {
		margin: 0 0 0.5rem;
		font-size: 1rem;
		color: var(--gray-600);
	}

	.stat {
		font-size: 2.5rem;
		font-weight: bold;
		color: var(--primary);
		margin: 0.5rem 0;
	}

	.dashboard-card p {
		margin: 0;
		font-size: 0.875rem;
		color: var(--gray-600);
	}

	.quick-actions { margin: 2rem 0; }
	.quick-actions h2 { margin-bottom: 1rem; }

	.actions-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
		gap: 1rem;
	}

	.action-btn {
		padding: 1.5rem;
		text-align: center;
		text-decoration: none;
		color: inherit;
		transition: transform 0.2s;
	}

	.action-btn:hover {
		transform: scale(1.05);
	}

	.icon {
		display: block;
		font-size: 2rem;
		margin-bottom: 0.5rem;
	}

	.offline-notice {
		margin-top: 2rem;
		padding: 1rem;
		background: #fef3c7;
		border: 1px solid #fde68a;
		text-align: center;
	}

	.card {
		background: white;
		border-radius: 8px;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
	}

	.btn {
		padding: 0.75rem 1.5rem;
		border: none;
		border-radius: 6px;
		cursor: pointer;
		font-weight: 500;
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-primary {
		background: var(--primary);
		color: white;
	}

	.loading {
		text-align: center;
		padding: 3rem;
	}
</style>
