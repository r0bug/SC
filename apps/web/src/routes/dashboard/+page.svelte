<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { DashboardSummary } from '$lib/api/types';

	let dashboard: DashboardSummary | null = null;
	let loading = true;
	let error = '';
	let updateInfo: {
		current_version: string;
		latest_version: string;
		update_available: boolean;
		release_url?: string;
		release_notes?: string;
		download_url?: string;
		published_at?: string;
	} | null = null;
	let checkingForUpdates = false;
	let updateDismissed = false;

	onMount(async () => {
		try {
			dashboard = await api.getDashboard();
			await checkForUpdates();
		} catch (e: unknown) {
			error = e instanceof Error ? e.message : String(e);
		} finally {
			loading = false;
		}
	});

	async function checkForUpdates() {
		try {
			checkingForUpdates = true;
			const response = await fetch('http://localhost:3002/api/system/updates/check');
			if (response.ok) {
				updateInfo = await response.json();
			}
		} catch (e) {
			console.error('Failed to check for updates:', e);
		} finally {
			checkingForUpdates = false;
		}
	}

	function dismissUpdate() {
		updateDismissed = true;
	}

	function formatDate(dateStr: string): string {
		return new Date(dateStr).toLocaleDateString('en-US', {
			month: 'short',
			day: 'numeric',
			hour: 'numeric',
			minute: '2-digit'
		});
	}

	function getStatusBadgeClass(status: string | { Failed: { reason: string } }): string {
		if (typeof status === 'string') {
			switch (status) {
				case 'Sent': return 'badge-success';
				case 'Pending': return 'badge-warning';
				case 'Retrying': return 'badge-info';
				default: return 'badge-default';
			}
		}
		return 'badge-error';
	}
</script>

<div class="dashboard">
	<div class="header">
		<div>
			<h1>Dashboard</h1>
			<p class="subtitle">Welcome back to SagensContact</p>
		</div>
		<div class="header-actions">
			<a href="/contacts/new" class="btn btn-primary">+ New Contact</a>
			<a href="/import" class="btn btn-secondary">📥 Import</a>
		</div>
	</div>

	<!-- Update Notification Banner -->
	{#if updateInfo && updateInfo.update_available && !updateDismissed}
		<div class="update-banner">
			<div class="update-content">
				<div class="update-icon">🎉</div>
				<div class="update-details">
					<h3>Update Available: v{updateInfo.latest_version}</h3>
					<p>A new version of SagensContact is available. Current: v{updateInfo.current_version}</p>
					{#if updateInfo.published_at}
						<p class="text-sm">Released: {formatDate(updateInfo.published_at)}</p>
					{/if}
				</div>
			</div>
			<div class="update-actions">
				{#if updateInfo.release_url}
					<a href={updateInfo.release_url} target="_blank" class="btn btn-primary btn-sm">
						View Release
					</a>
				{/if}
				<button class="btn btn-sm btn-secondary" on:click={dismissUpdate}>
					Dismiss
				</button>
			</div>
		</div>
	{/if}

	{#if loading}
		<div class="loading-container">
			<div class="spinner"></div>
			<p>Loading dashboard...</p>
		</div>
	{:else if error}
		<div class="card error-card">
			<h3>Error Loading Dashboard</h3>
			<p>{error}</p>
		</div>
	{:else if dashboard}
		<!-- Summary Cards -->
		<div class="summary-grid">
			<div class="summary-card card">
				<div class="summary-icon">👥</div>
				<div class="summary-content">
					<h3>{dashboard.contacts_count}</h3>
					<p>Total Contacts</p>
				</div>
			</div>
			<div class="summary-card card">
				<div class="summary-icon">📁</div>
				<div class="summary-content">
					<h3>{dashboard.projects_active}</h3>
					<p>Active Projects</p>
				</div>
			</div>
			<div class="summary-card card">
				<div class="summary-icon">📅</div>
				<div class="summary-content">
					<h3>{dashboard.upcoming_events?.length || 0}</h3>
					<p>Upcoming Events</p>
				</div>
			</div>
			<div class="summary-card card">
				<div class="summary-icon">📧</div>
				<div class="summary-content">
					<h3>{dashboard.recent_communications?.length || 0}</h3>
					<p>Communications</p>
				</div>
			</div>
		</div>

		<div class="dashboard-grid">
			<!-- Upcoming Events -->
			<div class="card">
				<div class="card-header">
					<h2>📅 Upcoming Events</h2>
					<a href="/calendar" class="link">View all</a>
				</div>
				{#if dashboard.upcoming_events && dashboard.upcoming_events.length > 0}
					<div class="event-list">
						{#each dashboard.upcoming_events.slice(0, 5) as event}
							<div class="event-item">
								<div class="event-time">
									{formatDate(event.start_time)}
								</div>
								<div class="event-details">
									<h4>{event.title}</h4>
									{#if event.location}
										<p class="text-sm">📍 {event.location}</p>
									{/if}
									{#if event.contacts.length > 0}
										<p class="text-sm">👥 {event.contacts.length} attendees</p>
									{/if}
								</div>
							</div>
						{/each}
					</div>
				{:else}
					<p class="empty-state">No upcoming events</p>
				{/if}
			</div>

			<!-- Recent Communications -->
			<div class="card">
				<div class="card-header">
					<h2>📧 Recent Communications</h2>
					<a href="/communications" class="link">View all</a>
				</div>
				{#if dashboard.recent_communications && dashboard.recent_communications.length > 0}
					<div class="comm-list">
						{#each dashboard.recent_communications.slice(0, 5) as comm}
							<div class="comm-item">
								<div class="comm-method">
									{#if comm.method === 'Email'}
										✉️
									{:else if comm.method === 'SMS'}
										💬
									{:else}
										🌐
									{/if}
								</div>
								<div class="comm-details">
									<p class="comm-message">{comm.message.substring(0, 100)}...</p>
									<div class="comm-meta">
										<span class="badge {getStatusBadgeClass(comm.status)}">
											{typeof comm.status === 'string' ? comm.status : 'Failed'}
										</span>
										{#if comm.scheduled_at}
											<span class="text-sm">Scheduled: {formatDate(comm.scheduled_at)}</span>
										{/if}
									</div>
								</div>
							</div>
						{/each}
					</div>
				{:else}
					<p class="empty-state">No recent communications</p>
				{/if}
			</div>

			<!-- AI Suggestions -->
			<div class="card">
				<div class="card-header">
					<h2>🤖 AI Suggestions</h2>
					<a href="/insights" class="link">View all</a>
				</div>
				{#if dashboard.ai_suggestions && dashboard.ai_suggestions.length > 0}
					<div class="suggestion-list">
						{#each dashboard.ai_suggestions.slice(0, 5) as suggestion}
							<div class="suggestion-item">
								<div class="suggestion-type">
									<span class="badge badge-ai">{suggestion.insight_type}</span>
								</div>
								<p class="suggestion-content">{suggestion.content}</p>
								<div class="suggestion-actions">
									<button class="btn btn-sm">Apply</button>
									<button class="btn btn-sm btn-secondary">Dismiss</button>
								</div>
							</div>
						{/each}
					</div>
				{:else}
					<p class="empty-state">No suggestions available</p>
				{/if}
			</div>

			<!-- Nag Reminders -->
			<div class="card">
				<div class="card-header">
					<h2>🔔 Nag Reminders</h2>
					<a href="/communications" class="link">Take action</a>
				</div>
				{#if dashboard.nag_reminders.length > 0}
					<div class="reminder-list">
						{#each dashboard.nag_reminders.slice(0, 5) as reminder}
							<div class="reminder-item">
								<div class="reminder-icon">⏰</div>
								<div class="reminder-details">
									<h4>{reminder.contact_name}</h4>
									<p class="text-sm">No contact for {reminder.days_since_contact} days</p>
								</div>
								<a href="/communications?contact={reminder.contact_id}" class="btn btn-sm">
									Reach out
								</a>
							</div>
						{/each}
					</div>
				{:else}
					<p class="empty-state">No reminders</p>
				{/if}
			</div>

			<!-- Pending Invites -->
			{#if dashboard.pending_invites.length > 0}
				<div class="card span-2">
					<div class="card-header">
						<h2>👥 Pending Invites</h2>
						<a href="/shares" class="link">Manage</a>
					</div>
					<div class="invite-list">
						{#each dashboard.pending_invites as invite}
							<div class="invite-item">
								<div class="invite-details">
									<p>
										<strong>{invite.shared_by}</strong> shared a {invite.entity_type}
									</p>
									<p class="text-sm">
										Permissions: {invite.permissions.join(', ')}
									</p>
								</div>
								<div class="invite-actions">
									<button class="btn btn-sm btn-primary">Accept</button>
									<button class="btn btn-sm btn-secondary">Decline</button>
								</div>
							</div>
						{/each}
					</div>
				</div>
			{/if}
		</div>

		<!-- Mock Notice -->
		<div class="mock-banner">
			<strong>⚠️ Alpha Release:</strong> Email, SMS, and social messaging are currently mocked.
			All communications are simulated for testing purposes.
		</div>
	{/if}
</div>

<style>
	.dashboard {
		padding: 2rem;
		max-width: 1400px;
		margin: 0 auto;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 2rem;
	}

	.header h1 {
		font-size: 2.5rem;
		margin-bottom: 0.25rem;
	}

	.subtitle {
		color: var(--gray-600);
		font-size: 1.125rem;
	}

	.header-actions {
		display: flex;
		gap: 1rem;
	}

	.summary-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
		gap: 1.5rem;
		margin-bottom: 2rem;
	}

	.summary-card {
		display: flex;
		align-items: center;
		gap: 1.5rem;
		padding: 1.5rem;
	}

	.summary-icon {
		font-size: 2.5rem;
		background: var(--primary-light);
		width: 60px;
		height: 60px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 12px;
	}

	.summary-content h3 {
		font-size: 2rem;
		margin-bottom: 0.25rem;
	}

	.summary-content p {
		color: var(--gray-600);
	}

	.dashboard-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
		gap: 1.5rem;
	}

	.card-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1.5rem;
		padding-bottom: 1rem;
		border-bottom: 1px solid var(--gray-200);
	}

	.card-header h2 {
		font-size: 1.25rem;
		margin: 0;
	}

	.link {
		color: var(--primary);
		text-decoration: none;
		font-size: 0.875rem;
		font-weight: 500;
	}

	.link:hover {
		text-decoration: underline;
	}

	.empty-state {
		text-align: center;
		color: var(--gray-500);
		padding: 2rem;
	}

	.event-list,
	.comm-list,
	.suggestion-list,
	.reminder-list,
	.invite-list {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.event-item,
	.comm-item,
	.suggestion-item,
	.reminder-item,
	.invite-item {
		display: flex;
		gap: 1rem;
		padding: 1rem;
		background: var(--gray-50);
		border-radius: 8px;
		align-items: flex-start;
	}

	.event-time {
		font-size: 0.875rem;
		color: var(--gray-600);
		min-width: 120px;
	}

	.event-details h4 {
		margin-bottom: 0.25rem;
	}

	.text-sm {
		font-size: 0.875rem;
		color: var(--gray-600);
	}

	.comm-method {
		font-size: 1.5rem;
	}

	.comm-details {
		flex: 1;
	}

	.comm-message {
		margin-bottom: 0.5rem;
	}

	.comm-meta {
		display: flex;
		gap: 1rem;
		align-items: center;
	}

	.badge {
		padding: 0.25rem 0.5rem;
		border-radius: 4px;
		font-size: 0.75rem;
		font-weight: 500;
	}

	.badge-success {
		background: #d1fae5;
		color: #065f46;
	}

	.badge-warning {
		background: #fed7aa;
		color: #92400e;
	}

	.badge-info {
		background: #dbeafe;
		color: #1e40af;
	}

	.badge-error {
		background: #fee2e2;
		color: #991b1b;
	}

	.badge-ai {
		background: var(--primary-light);
		color: var(--primary);
	}

	.suggestion-content {
		flex: 1;
		margin: 0.5rem 0;
	}

	.suggestion-actions {
		display: flex;
		gap: 0.5rem;
		margin-top: 0.5rem;
	}

	.reminder-icon {
		font-size: 2rem;
	}

	.reminder-details {
		flex: 1;
	}

	.reminder-details h4 {
		margin-bottom: 0.25rem;
	}

	.invite-details {
		flex: 1;
	}

	.invite-actions {
		display: flex;
		gap: 0.5rem;
	}

	.btn-sm {
		padding: 0.375rem 0.75rem;
		font-size: 0.875rem;
	}

	.span-2 {
		grid-column: span 2;
	}

	.update-banner {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1.5rem;
		background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
		color: white;
		border-radius: 12px;
		margin-bottom: 2rem;
		box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
	}

	.update-content {
		display: flex;
		align-items: center;
		gap: 1.5rem;
		flex: 1;
	}

	.update-icon {
		font-size: 2.5rem;
		background: rgba(255, 255, 255, 0.2);
		width: 60px;
		height: 60px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 12px;
	}

	.update-details h3 {
		margin: 0 0 0.5rem 0;
		font-size: 1.25rem;
		font-weight: 600;
	}

	.update-details p {
		margin: 0.25rem 0;
		opacity: 0.95;
	}

	.update-actions {
		display: flex;
		gap: 0.75rem;
		align-items: center;
	}

	.update-banner .btn-primary {
		background: white;
		color: #667eea;
	}

	.update-banner .btn-primary:hover {
		background: rgba(255, 255, 255, 0.9);
	}

	.update-banner .btn-secondary {
		background: rgba(255, 255, 255, 0.2);
		color: white;
		border: 1px solid rgba(255, 255, 255, 0.3);
	}

	.update-banner .btn-secondary:hover {
		background: rgba(255, 255, 255, 0.3);
	}

	.mock-banner {
		margin-top: 2rem;
		padding: 1rem;
		background: #fef3c7;
		border: 1px solid #fde68a;
		border-radius: 8px;
		text-align: center;
		color: #92400e;
	}

	.loading-container {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		min-height: 400px;
		gap: 1rem;
	}

	.spinner {
		width: 40px;
		height: 40px;
		border: 3px solid var(--gray-300);
		border-top-color: var(--primary);
		border-radius: 50%;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.error-card {
		border-left: 4px solid var(--error);
		padding: 1.5rem;
	}

	.error-card h3 {
		color: var(--error);
		margin-bottom: 0.5rem;
	}

	@media (max-width: 768px) {
		.dashboard {
			padding: 1rem;
		}

		.header {
			flex-direction: column;
			align-items: flex-start;
			gap: 1rem;
		}

		.dashboard-grid {
			grid-template-columns: 1fr;
		}

		.span-2 {
			grid-column: span 1;
		}

		.update-banner {
			flex-direction: column;
			gap: 1rem;
		}

		.update-content {
			flex-direction: column;
			text-align: center;
		}

		.update-actions {
			width: 100%;
			justify-content: center;
		}
	}
</style>