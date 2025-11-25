<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { api } from '$lib/api/api';
	import type { User } from '$lib/api/types';

	let user: User | null = null;
	let loading = true;
	let settings: Record<string, any> = {};

	// Profile editing
	let editingProfile = false;
	let profileName = '';
	let profileEmail = '';

	// Password change
	let showPasswordChange = false;
	let currentPassword = '';
	let newPassword = '';
	let confirmPassword = '';
	let passwordChangeError = '';
	let passwordChangeSuccess = '';

	// Preferences
	let theme: 'light' | 'dark' = 'light';
	let notifications = true;
	let syncEnabled = true;

	onMount(async () => {
		await loadUserAndSettings();
	});

	async function loadUserAndSettings() {
		try {
			loading = true;
			user = await api.getCurrentUser();
			profileName = user.name;
			profileEmail = user.email;

			// Load preferences
			if (user.preferences) {
				theme = user.preferences.theme || 'light';
				notifications = user.preferences.notifications !== false;
			}

			// Load settings
			try {
				settings = await api.getSettings();
				syncEnabled = settings.sync_enabled !== false;
			} catch (err) {
				console.error('Failed to load settings:', err);
			}
		} catch (err: any) {
			console.error('Failed to load user:', err);
			alert('Failed to load user profile. Please log in again.');
			goto('/auth/login');
		} finally {
			loading = false;
		}
	}

	async function updateProfile() {
		if (!profileName.trim()) {
			alert('Name cannot be empty');
			return;
		}

		try {
			user = await api.updateUserProfile({
				name: profileName,
				email: profileEmail
			});
			editingProfile = false;
			alert('Profile updated successfully');
		} catch (err: any) {
			alert('Failed to update profile: ' + err.message);
		}
	}

	async function changePassword() {
		passwordChangeError = '';
		passwordChangeSuccess = '';

		if (!currentPassword || !newPassword || !confirmPassword) {
			passwordChangeError = 'All fields are required';
			return;
		}

		if (newPassword !== confirmPassword) {
			passwordChangeError = 'New passwords do not match';
			return;
		}

		if (newPassword.length < 8) {
			passwordChangeError = 'Password must be at least 8 characters';
			return;
		}

		try {
			await api.changePassword(currentPassword, newPassword);
			passwordChangeSuccess = 'Password changed successfully';
			currentPassword = '';
			newPassword = '';
			confirmPassword = '';
			setTimeout(() => {
				showPasswordChange = false;
				passwordChangeSuccess = '';
			}, 2000);
		} catch (err: any) {
			passwordChangeError = err.message || 'Failed to change password';
		}
	}

	async function handleLogoutAll() {
		if (!confirm('This will log you out of all devices. Continue?')) return;

		try {
			await api.logoutAll();
			alert('Logged out of all sessions successfully');
			goto('/auth/login');
		} catch (err: any) {
			alert('Failed to logout: ' + err.message);
		}
	}

	async function updatePreferences() {
		try {
			await api.updateUserProfile({
				preferences: {
					theme,
					notifications
				}
			});

			await api.updateSettings({
				...settings,
				sync_enabled: syncEnabled
			});

			alert('Preferences saved successfully');
		} catch (err: any) {
			alert('Failed to save preferences: ' + err.message);
		}
	}
</script>

<div class="page">
	<h1>Settings</h1>

	{#if loading}
		<div class="loading">
			<div class="spinner"></div>
			<p>Loading settings...</p>
		</div>
	{:else}
		<div class="settings-grid">
			<!-- User Profile Card -->
			<div class="card">
				<h2>User Profile</h2>
				{#if user}
					{#if !editingProfile}
						<div class="profile-view">
							<div class="profile-avatar">
								{user.name[0].toUpperCase()}
							</div>
							<div class="profile-info">
								<h3>{user.name}</h3>
								<p>{user.email}</p>
								<div class="profile-meta">
									<span class:verified={user.email_verified}>
										{user.email_verified ? '✓ Verified' : '⚠ Not Verified'}
									</span>
									<span>
										Last login: {user.last_login_at ? new Date(user.last_login_at).toLocaleString() : 'Never'}
									</span>
								</div>
							</div>
							<button on:click={() => editingProfile = true} class="btn btn-sm">
								Edit Profile
							</button>
						</div>
					{:else}
						<form on:submit|preventDefault={updateProfile}>
							<div class="form-group">
								<label>Name</label>
								<input type="text" bind:value={profileName} required />
							</div>
							<div class="form-group">
								<label>Email</label>
								<input type="email" bind:value={profileEmail} required />
							</div>
							<div class="form-actions">
								<button type="button" on:click={() => editingProfile = false} class="btn">
									Cancel
								</button>
								<button type="submit" class="btn btn-primary">Save</button>
							</div>
						</form>
					{/if}
				{/if}
			</div>

			<!-- Password Change Card -->
			<div class="card">
				<h2>Security</h2>
				{#if !showPasswordChange}
					<p class="card-description">Manage your password and security settings</p>
					<button on:click={() => showPasswordChange = true} class="btn">
						Change Password
					</button>
				{:else}
					<form on:submit|preventDefault={changePassword}>
						{#if passwordChangeError}
							<div class="alert alert-error">
								{passwordChangeError}
							</div>
						{/if}
						{#if passwordChangeSuccess}
							<div class="alert alert-success">
								{passwordChangeSuccess}
							</div>
						{/if}
						<div class="form-group">
							<label>Current Password</label>
							<input type="password" bind:value={currentPassword} required />
						</div>
						<div class="form-group">
							<label>New Password</label>
							<input type="password" bind:value={newPassword} required minlength="8" />
						</div>
						<div class="form-group">
							<label>Confirm New Password</label>
							<input type="password" bind:value={confirmPassword} required minlength="8" />
						</div>
						<div class="form-actions">
							<button type="button" on:click={() => showPasswordChange = false} class="btn">
								Cancel
							</button>
							<button type="submit" class="btn btn-primary">Change Password</button>
						</div>
					</form>
				{/if}
			</div>

			<!-- Sessions Card -->
			<div class="card">
				<h2>Sessions</h2>
				<p class="card-description">Manage your active sessions across all devices</p>
				<button on:click={handleLogoutAll} class="btn btn-danger">
					Logout All Sessions
				</button>
			</div>

			<!-- Preferences Card -->
			<div class="card">
				<h2>Preferences</h2>
				<form on:submit|preventDefault={updatePreferences}>
					<div class="form-group">
						<label>Theme</label>
						<select bind:value={theme}>
							<option value="light">Light</option>
							<option value="dark">Dark</option>
						</select>
					</div>
					<div class="form-group">
						<label class="checkbox-label">
							<input type="checkbox" bind:checked={notifications} />
							<span>Enable Notifications</span>
						</label>
					</div>
					<div class="form-group">
						<label class="checkbox-label">
							<input type="checkbox" bind:checked={syncEnabled} />
							<span>Enable Auto-Sync</span>
						</label>
					</div>
					<button type="submit" class="btn btn-primary">Save Preferences</button>
				</form>
			</div>

			<!-- Communication Config Card -->
			<div class="card">
				<h2>Communication</h2>
				<p class="card-description">Configure email and SMS settings</p>
				<div class="config-status">
					<div class="status-item">
						<span class="status-label">Email:</span>
						<span class="status-badge status-mock">Mock Mode</span>
					</div>
					<div class="status-item">
						<span class="status-label">SMS:</span>
						<span class="status-badge status-mock">Mock Mode</span>
					</div>
					<div class="status-item">
						<span class="status-label">Social:</span>
						<span class="status-badge status-mock">Mock Mode</span>
					</div>
				</div>
				<p class="config-note">
					Configure real providers in production via environment variables
				</p>
			</div>

			<!-- API Tokens Card (Placeholder) -->
			<div class="card">
				<h2>API Tokens</h2>
				<p class="card-description">Generate and manage API tokens for integrations</p>
				<p class="coming-soon">Coming soon in beta release</p>
			</div>
		</div>
	{/if}
</div>

<style>
	.page {
		padding: 2rem;
		max-width: 1200px;
		margin: 0 auto;
	}

	h1 {
		margin-bottom: 2rem;
	}

	.loading {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		min-height: 300px;
		gap: 1rem;
	}

	.spinner {
		border: 3px solid #f3f3f3;
		border-top: 3px solid var(--primary, #3b82f6);
		border-radius: 50%;
		width: 40px;
		height: 40px;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		0% { transform: rotate(0deg); }
		100% { transform: rotate(360deg); }
	}

	.settings-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
		gap: 1.5rem;
	}

	.card {
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 8px;
		padding: 1.5rem;
	}

	.card h2 {
		margin: 0 0 1.5rem;
		font-size: 1.25rem;
	}

	.card-description {
		color: #6b7280;
		margin-bottom: 1.5rem;
	}

	.profile-view {
		display: flex;
		align-items: center;
		gap: 1.5rem;
	}

	.profile-avatar {
		width: 80px;
		height: 80px;
		border-radius: 50%;
		background: var(--primary, #3b82f6);
		color: white;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 2rem;
		font-weight: 600;
		flex-shrink: 0;
	}

	.profile-info {
		flex: 1;
	}

	.profile-info h3 {
		margin: 0 0 0.5rem;
	}

	.profile-info p {
		margin: 0 0 0.5rem;
		color: #6b7280;
	}

	.profile-meta {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		font-size: 0.875rem;
		color: #9ca3af;
	}

	.profile-meta span.verified {
		color: #059669;
	}

	.form-group {
		margin-bottom: 1.5rem;
	}

	.form-group label {
		display: block;
		margin-bottom: 0.5rem;
		font-weight: 500;
		color: #374151;
	}

	.form-group input,
	.form-group select {
		width: 100%;
		padding: 0.75rem;
		border: 1px solid #d1d5db;
		border-radius: 6px;
		font-size: 1rem;
	}

	.checkbox-label {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		cursor: pointer;
	}

	.checkbox-label input[type="checkbox"] {
		width: auto;
		margin: 0;
	}

	.form-actions {
		display: flex;
		justify-content: flex-end;
		gap: 1rem;
		margin-top: 1.5rem;
	}

	.alert {
		padding: 0.75rem 1rem;
		border-radius: 6px;
		margin-bottom: 1rem;
	}

	.alert-error {
		background: #fee2e2;
		color: #dc2626;
		border: 1px solid #fca5a5;
	}

	.alert-success {
		background: #d1fae5;
		color: #065f46;
		border: 1px solid #6ee7b7;
	}

	.config-status {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		margin-bottom: 1rem;
	}

	.status-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.status-label {
		font-weight: 500;
	}

	.status-badge {
		padding: 0.25rem 0.75rem;
		border-radius: 12px;
		font-size: 0.75rem;
		font-weight: 500;
	}

	.status-mock {
		background: #fef3c7;
		color: #92400e;
	}

	.config-note {
		font-size: 0.875rem;
		color: #6b7280;
		font-style: italic;
	}

	.coming-soon {
		padding: 2rem;
		text-align: center;
		color: #9ca3af;
		font-style: italic;
	}

	.btn {
		padding: 0.75rem 1.5rem;
		border: 1px solid #d1d5db;
		background: white;
		border-radius: 6px;
		cursor: pointer;
		font-size: 1rem;
		font-weight: 500;
		transition: all 0.2s;
	}

	.btn:hover {
		background: #f9fafb;
	}

	.btn-primary {
		background: var(--primary, #3b82f6);
		color: white;
		border-color: var(--primary, #3b82f6);
	}

	.btn-primary:hover {
		background: #2563eb;
	}

	.btn-danger {
		background: #dc2626;
		color: white;
		border-color: #dc2626;
	}

	.btn-danger:hover {
		background: #b91c1c;
	}

	.btn-sm {
		padding: 0.5rem 1rem;
		font-size: 0.875rem;
	}

	/* Mobile responsiveness */
	@media (max-width: 768px) {
		.page {
			padding: 1rem;
		}

		.settings-grid {
			grid-template-columns: 1fr;
		}

		.profile-view {
			flex-direction: column;
			align-items: flex-start;
		}

		.form-actions {
			flex-direction: column;
		}

		.form-actions button {
			width: 100%;
		}
	}
</style>
