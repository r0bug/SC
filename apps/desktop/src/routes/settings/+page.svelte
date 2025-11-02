<script lang="ts">
	import { onMount } from 'svelte';
	import { tauriApi } from '$lib/api/tauri-api';
	import { invoke } from '@tauri-apps/api/core';
	import { appDataDir } from '@tauri-apps/api/path';
	import { open as openPath } from '@tauri-apps/plugin-shell';

	let settings: any = {};
	let loading = true;
	let saving = false;
	let dataDir = '';
	let online = false;

	// Form fields
	let syncUrl = 'http://localhost:3000/api';
	let syncEnabled = true;
	let notificationsEnabled = true;
	let theme = 'light';

	onMount(async () => {
		await loadSettings();
		await checkOnline();
		dataDir = await appDataDir();
	});

	async function loadSettings() {
		try {
			loading = true;
			settings = await tauriApi.getSettings();

			// Populate form fields
			syncUrl = settings.sync_url || 'http://localhost:3000/api';
			syncEnabled = settings.sync_enabled !== false;
			notificationsEnabled = settings.notifications !== false;
			theme = settings.theme || 'light';
		} catch (error) {
			console.error('Failed to load settings:', error);
		} finally {
			loading = false;
		}
	}

	async function checkOnline() {
		try {
			online = await tauriApi.checkOnline();
		} catch (error) {
			console.error('Failed to check online status:', error);
			online = false;
		}
	}

	async function saveSettings() {
		try {
			saving = true;
			const newSettings = {
				sync_url: syncUrl,
				sync_enabled: syncEnabled,
				notifications: notificationsEnabled,
				theme
			};

			await tauriApi.updateSettings(newSettings);
			await tauriApi.showNotification('Settings Saved', 'Your settings have been updated successfully');
		} catch (error: any) {
			console.error('Failed to save settings:', error);
			alert(`Failed to save settings: ${error.message || error}`);
		} finally {
			saving = false;
		}
	}

	async function openDataDirectory() {
		try {
			await openPath(dataDir);
		} catch (error) {
			console.error('Failed to open data directory:', error);
			alert('Failed to open data directory');
		}
	}

	async function testSync() {
		try {
			const result = await tauriApi.syncWithServer();
			alert(`Sync successful!\nContacts: ${result.synced_contacts}\nProjects: ${result.synced_projects}`);
		} catch (error: any) {
			alert(`Sync failed: ${error.message || error}`);
		}
	}
</script>

<div class="page">
	<div class="header">
		<h1>Settings</h1>
		<p class="subtitle">Configure your desktop application</p>
	</div>

	{#if loading}
		<div class="loading">Loading settings...</div>
	{:else}
		<div class="content">
			<div class="settings-section card">
				<h2>Application</h2>

				<div class="setting-item">
					<div class="setting-info">
						<label for="theme">Theme</label>
						<p class="description">Choose your preferred color theme</p>
					</div>
					<select id="theme" bind:value={theme}>
						<option value="light">Light</option>
						<option value="dark">Dark (Coming Soon)</option>
						<option value="auto">Auto (Coming Soon)</option>
					</select>
				</div>

				<div class="setting-item">
					<div class="setting-info">
						<label for="notifications">Desktop Notifications</label>
						<p class="description">Show notifications for important events</p>
					</div>
					<label class="toggle">
						<input type="checkbox" bind:checked={notificationsEnabled} />
						<span class="toggle-slider"></span>
					</label>
				</div>

				<div class="setting-item">
					<div class="setting-info">
						<label>Data Directory</label>
						<p class="description">Location where your contacts database is stored</p>
					</div>
					<button on:click={openDataDirectory} class="btn btn-secondary">
						📁 Open Folder
					</button>
				</div>
			</div>

			<div class="settings-section card">
				<h2>Sync & Network</h2>

				<div class="setting-item">
					<div class="setting-info">
						<label>Connection Status</label>
						<p class="description">Current network connectivity</p>
					</div>
					<span class="status {online ? 'online' : 'offline'}">
						{online ? '● Online' : '● Offline'}
					</span>
				</div>

				<div class="setting-item">
					<div class="setting-info">
						<label for="sync">Enable Sync</label>
						<p class="description">Synchronize with remote server</p>
					</div>
					<label class="toggle">
						<input type="checkbox" bind:checked={syncEnabled} />
						<span class="toggle-slider"></span>
					</label>
				</div>

				{#if syncEnabled}
					<div class="setting-item">
						<div class="setting-info">
							<label for="syncUrl">Sync Server URL</label>
							<p class="description">API endpoint for synchronization</p>
						</div>
						<input
							type="url"
							id="syncUrl"
							bind:value={syncUrl}
							placeholder="http://localhost:3000/api"
							class="input-text"
						/>
					</div>

					<div class="setting-item">
						<div class="setting-info">
							<label>Test Sync</label>
							<p class="description">Verify connection to sync server</p>
						</div>
						<button on:click={testSync} class="btn btn-secondary" disabled={!online}>
							🔄 Test Connection
						</button>
					</div>
				{/if}
			</div>

			<div class="settings-section card">
				<h2>About</h2>

				<div class="about-info">
					<div class="about-item">
						<span class="about-label">Version</span>
						<span class="about-value">0.1.0-alpha.3</span>
					</div>
					<div class="about-item">
						<span class="about-label">Platform</span>
						<span class="about-value">Desktop (Tauri)</span>
					</div>
					<div class="about-item">
						<span class="about-label">Database</span>
						<span class="about-value">SQLite (Local)</span>
					</div>
				</div>
			</div>

			<div class="save-section">
				<button on:click={saveSettings} class="btn btn-primary btn-lg" disabled={saving}>
					{saving ? '💾 Saving...' : '💾 Save Settings'}
				</button>
			</div>
		</div>
	{/if}
</div>

<style>
	.page {
		padding: 2rem;
		max-width: 900px;
		margin: 0 auto;
	}

	.header {
		margin-bottom: 2rem;
	}

	.header h1 {
		margin: 0 0 0.5rem;
		font-size: 2rem;
		color: var(--gray-900);
	}

	.subtitle {
		margin: 0;
		color: var(--gray-600);
		font-size: 1rem;
	}

	.loading {
		text-align: center;
		padding: 3rem;
		color: var(--gray-600);
		font-size: 1.125rem;
	}

	.content {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.settings-section {
		padding: 2rem;
	}

	.settings-section h2 {
		margin: 0 0 1.5rem;
		font-size: 1.25rem;
		color: var(--gray-900);
		padding-bottom: 0.75rem;
		border-bottom: 1px solid var(--gray-200);
	}

	.setting-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1.5rem 0;
		border-bottom: 1px solid var(--gray-100);
	}

	.setting-item:last-child {
		border-bottom: none;
		padding-bottom: 0;
	}

	.setting-info {
		flex: 1;
	}

	.setting-info label {
		display: block;
		font-weight: 500;
		color: var(--gray-900);
		margin-bottom: 0.25rem;
	}

	.setting-info .description {
		margin: 0;
		font-size: 0.875rem;
		color: var(--gray-600);
	}

	.input-text {
		width: 300px;
		padding: 0.75rem;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		font-size: 0.875rem;
	}

	.input-text:focus {
		outline: none;
		border-color: var(--primary);
		box-shadow: 0 0 0 3px var(--primary-light);
	}

	select {
		padding: 0.75rem 1rem;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		font-size: 0.875rem;
		background: white;
		cursor: pointer;
	}

	select:focus {
		outline: none;
		border-color: var(--primary);
		box-shadow: 0 0 0 3px var(--primary-light);
	}

	.toggle {
		position: relative;
		display: inline-block;
		width: 50px;
		height: 28px;
	}

	.toggle input {
		opacity: 0;
		width: 0;
		height: 0;
	}

	.toggle-slider {
		position: absolute;
		cursor: pointer;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background-color: var(--gray-300);
		border-radius: 999px;
		transition: 0.3s;
	}

	.toggle-slider:before {
		position: absolute;
		content: "";
		height: 20px;
		width: 20px;
		left: 4px;
		bottom: 4px;
		background-color: white;
		border-radius: 50%;
		transition: 0.3s;
	}

	.toggle input:checked + .toggle-slider {
		background-color: var(--primary);
	}

	.toggle input:checked + .toggle-slider:before {
		transform: translateX(22px);
	}

	.status {
		font-weight: 600;
		font-size: 0.875rem;
	}

	.status.online {
		color: var(--color-success-600);
	}

	.status.offline {
		color: var(--color-error-600);
	}

	.about-info {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.about-item {
		display: flex;
		justify-content: space-between;
		padding: 0.75rem 0;
		border-bottom: 1px solid var(--gray-100);
	}

	.about-item:last-child {
		border-bottom: none;
	}

	.about-label {
		font-weight: 500;
		color: var(--gray-700);
	}

	.about-value {
		color: var(--gray-600);
	}

	.save-section {
		display: flex;
		justify-content: center;
		padding: 2rem 0;
	}

	.card {
		background: white;
		border-radius: 8px;
		box-shadow: var(--shadow-sm);
	}

	.btn {
		padding: 0.75rem 1.5rem;
		border: none;
		border-radius: 6px;
		cursor: pointer;
		font-weight: 500;
		transition: all 0.2s;
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-primary {
		background: var(--primary);
		color: white;
	}

	.btn-primary:hover:not(:disabled) {
		background: var(--primary-dark);
	}

	.btn-secondary {
		background: var(--gray-200);
		color: var(--gray-900);
	}

	.btn-secondary:hover:not(:disabled) {
		background: var(--gray-300);
	}

	.btn-lg {
		padding: 1rem 2.5rem;
		font-size: 1.125rem;
	}
</style>
