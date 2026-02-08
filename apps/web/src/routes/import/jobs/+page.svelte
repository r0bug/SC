<script lang="ts">
	import { api } from '$lib/api/api';
	import type { ImportJob } from '$lib/api/types';
	import { toasts } from '$lib/stores/toast';
	import { onMount, onDestroy } from 'svelte';
	import LoadingSpinner from '$lib/components/ui/LoadingSpinner.svelte';
	import ProgressBar from '$lib/components/ui/ProgressBar.svelte';

	let jobs: ImportJob[] = [];
	let loading = true;
	let pollInterval: number | null = null;

	onMount(async () => {
		await loadJobs();
		startPolling();
	});

	onDestroy(() => {
		if (pollInterval) {
			clearInterval(pollInterval);
		}
	});

	async function loadJobs() {
		try {
			jobs = await api.listJobs();
			// Sort by created_at descending (newest first)
			jobs.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime());
		} catch (error) {
			console.error('Failed to load jobs:', error);
			toasts.error('Failed to load import jobs', 'Error');
		} finally {
			loading = false;
		}
	}

	function startPolling() {
		// Only poll if there are active jobs
		const hasActiveJobs = jobs.some(j =>
			j.status === 'Pending' ||
			j.status === 'Validating' ||
			j.status === 'Parsing' ||
			j.status === 'Deduplicating' ||
			j.status === 'Importing'
		);

		if (hasActiveJobs) {
			pollInterval = window.setInterval(async () => {
				await loadJobs();

				// Stop polling if no more active jobs
				const stillHasActiveJobs = jobs.some(j =>
					j.status === 'Pending' ||
					j.status === 'Validating' ||
					j.status === 'Parsing' ||
					j.status === 'Deduplicating' ||
					j.status === 'Importing'
				);

				if (!stillHasActiveJobs && pollInterval) {
					clearInterval(pollInterval);
					pollInterval = null;
				}
			}, 2000); // Poll every 2 seconds
		}
	}

	async function cancelJob(jobId: string) {
		try {
			await api.cancelJob(jobId);
			toasts.success('Import job cancelled', 'Cancelled');
			await loadJobs();
		} catch (error) {
			console.error('Failed to cancel job:', error);
			toasts.error('Failed to cancel job', 'Error');
		}
	}

	function formatDate(dateString: string): string {
		const date = new Date(dateString);
		const now = new Date();
		const diff = now.getTime() - date.getTime();
		const minutes = Math.floor(diff / 60000);
		const hours = Math.floor(diff / 3600000);
		const days = Math.floor(diff / 86400000);

		if (minutes < 1) return 'Just now';
		if (minutes < 60) return `${minutes}m ago`;
		if (hours < 24) return `${hours}h ago`;
		if (days < 7) return `${days}d ago`;

		return date.toLocaleDateString();
	}

	function getProgressPercentage(job: ImportJob): number {
		if (job.progress.total === 0) return 0;
		return Math.round((job.progress.current / job.progress.total) * 100);
	}

	function goToImport() {
		window.location.href = '/import';
	}

	function goToHistory() {
		window.location.href = '/import/history';
	}
</script>

<div class="page">
	<div class="header">
		<div>
			<h1>Import Jobs</h1>
			<p class="subtitle">Monitor active and recent import jobs</p>
		</div>
		<div class="header-actions">
			<button on:click={goToImport} class="btn btn-primary">
				➕ New Import
			</button>
			<button on:click={goToHistory} class="btn btn-secondary">
				📊 View History
			</button>
		</div>
	</div>

	{#if loading}
		<div class="card">
			<LoadingSpinner size="md" message="Loading import jobs..." />
		</div>
	{:else if jobs.length === 0}
		<div class="empty-state">
			<div class="empty-icon">📋</div>
			<h2>No Import Jobs</h2>
			<p>You haven't started any imports yet.</p>
			<button on:click={goToImport} class="btn btn-primary">
				Start Your First Import
			</button>
		</div>
	{:else}
		<div class="jobs-grid">
			{#each jobs as job}
				<div class="job-card" class:active={job.status === 'Importing' || job.status === 'Pending'}>
					<div class="job-header">
						<div class="job-title">
							<h3>{job.file_name}</h3>
							<span class="job-time">{formatDate(job.created_at)}</span>
						</div>
						<span class="status-badge status-{job.status.toLowerCase()}">
							{job.status}
						</span>
					</div>

					<div class="job-details">
						<div class="detail-item">
							<span class="label">Connector:</span>
							<span class="value">{job.connector_id}</span>
						</div>
						<div class="detail-item">
							<span class="label">Progress:</span>
							<span class="value">{getProgressPercentage(job)}%</span>
						</div>
					</div>

					{#if job.status === 'Pending' || job.status === 'Validating' || job.status === 'Parsing' || job.status === 'Deduplicating' || job.status === 'Importing'}
						<div class="job-progress">
							<ProgressBar
								progress={job.progress.total > 0 ? (job.progress.current / job.progress.total) * 100 : 0}
							/>
							<p class="progress-phase">{job.progress.phase}</p>
							<p class="progress-text">
								{job.progress.current.toLocaleString()} / {job.progress.total.toLocaleString()} rows
							</p>
						</div>

						<div class="job-actions">
							<button on:click={() => cancelJob(job.id)} class="btn btn-danger btn-sm">
								Cancel
							</button>
						</div>
					{/if}

					{#if job.result}
						<div class="job-results">
							<div class="result-grid">
								<div class="result-item success">
									<span class="result-value">{job.result.imported}</span>
									<span class="result-label">Imported</span>
								</div>
								<div class="result-item info">
									<span class="result-value">{job.result.updated}</span>
									<span class="result-label">Updated</span>
								</div>
								<div class="result-item warning">
									<span class="result-value">{job.result.skipped}</span>
									<span class="result-label">Skipped</span>
								</div>
								<div class="result-item error">
									<span class="result-value">{job.result.failed}</span>
									<span class="result-label">Failed</span>
								</div>
							</div>
							<div class="result-summary">
								<span>{job.result.duplicates_found} duplicates found</span>
								<span>•</span>
								<span>{job.result.elapsed_seconds}s elapsed</span>
							</div>
						</div>
					{/if}

					{#if job.status === 'Completed' || job.status === 'Failed'}
						<div class="job-actions">
							<button on:click={goToHistory} class="btn btn-secondary btn-sm">
								View Details
							</button>
						</div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page {
		padding: 2rem;
		max-width: 1400px;
		margin: 0 auto;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: 2rem;
	}

	.header h1 {
		font-size: 2rem;
		font-weight: 700;
		margin: 0 0 0.5rem 0;
	}

	.subtitle {
		color: #666;
		margin: 0;
	}

	.header-actions {
		display: flex;
		gap: 0.5rem;
	}

	.card {
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
		padding: 2rem;
	}

	.empty-state {
		text-align: center;
		padding: 4rem 2rem;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
	}

	.empty-icon {
		font-size: 4rem;
		margin-bottom: 1rem;
	}

	.empty-state h2 {
		font-size: 1.5rem;
		font-weight: 600;
		margin: 0 0 0.5rem 0;
	}

	.empty-state p {
		color: #666;
		margin: 0 0 1.5rem 0;
	}

	.jobs-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
		gap: 1.5rem;
	}

	.job-card {
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
		padding: 1.5rem;
		transition: all 0.2s;
	}

	.job-card.active {
		border-color: #3b82f6;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

	.job-card:hover {
		box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
	}

	.job-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: 1rem;
		gap: 1rem;
	}

	.job-title {
		flex: 1;
		min-width: 0;
	}

	.job-title h3 {
		font-size: 1.125rem;
		font-weight: 600;
		margin: 0 0 0.25rem 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.job-time {
		font-size: 0.875rem;
		color: #6b7280;
	}

	.status-badge {
		padding: 0.375rem 0.75rem;
		border-radius: 0.375rem;
		font-size: 0.75rem;
		font-weight: 600;
		white-space: nowrap;
	}

	.status-pending { background: #fef3c7; color: #92400e; }
	.status-validating, .status-parsing { background: #dbeafe; color: #1e40af; }
	.status-deduplicating { background: #e0e7ff; color: #3730a3; }
	.status-importing { background: #bfdbfe; color: #1e3a8a; }
	.status-completed { background: #d1fae5; color: #065f46; }
	.status-failed { background: #fee2e2; color: #991b1b; }
	.status-cancelled { background: #f3f4f6; color: #374151; }

	.job-details {
		display: flex;
		gap: 2rem;
		margin-bottom: 1rem;
	}

	.detail-item {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.detail-item .label {
		font-size: 0.75rem;
		color: #6b7280;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.detail-item .value {
		font-weight: 600;
	}

	.job-progress {
		margin: 1.5rem 0;
	}

	.progress-phase {
		margin-top: 0.75rem;
		font-weight: 600;
		font-size: 0.875rem;
		text-align: center;
		color: #374151;
	}

	.progress-text {
		margin-top: 0.5rem;
		color: #6b7280;
		font-size: 0.875rem;
		text-align: center;
	}

	.job-results {
		margin-top: 1.5rem;
		padding-top: 1.5rem;
		border-top: 1px solid #e5e7eb;
	}

	.result-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 1rem;
		margin-bottom: 1rem;
	}

	.result-item {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.25rem;
		padding: 0.75rem;
		border-radius: 0.375rem;
	}

	.result-item.success {
		background: #d1fae5;
	}

	.result-item.info {
		background: #dbeafe;
	}

	.result-item.warning {
		background: #fef3c7;
	}

	.result-item.error {
		background: #fee2e2;
	}

	.result-value {
		font-size: 1.5rem;
		font-weight: 700;
	}

	.result-item.success .result-value { color: #065f46; }
	.result-item.info .result-value { color: #1e40af; }
	.result-item.warning .result-value { color: #92400e; }
	.result-item.error .result-value { color: #991b1b; }

	.result-label {
		font-size: 0.75rem;
		font-weight: 500;
		color: #6b7280;
	}

	.result-summary {
		display: flex;
		justify-content: center;
		gap: 0.5rem;
		font-size: 0.875rem;
		color: #6b7280;
	}

	.job-actions {
		margin-top: 1rem;
		display: flex;
		justify-content: flex-end;
		gap: 0.5rem;
	}

	.btn {
		padding: 0.625rem 1.25rem;
		font-size: 0.875rem;
		font-weight: 500;
		border-radius: 0.375rem;
		border: none;
		cursor: pointer;
		transition: all 0.2s;
	}

	.btn-sm {
		padding: 0.5rem 1rem;
		font-size: 0.8125rem;
	}

	.btn-primary {
		background: #3b82f6;
		color: white;
	}

	.btn-primary:hover {
		background: #2563eb;
	}

	.btn-secondary {
		background: #f3f4f6;
		color: #374151;
	}

	.btn-secondary:hover {
		background: #e5e7eb;
	}

	.btn-danger {
		background: #ef4444;
		color: white;
	}

	.btn-danger:hover {
		background: #dc2626;
	}

	/* Mobile responsive */
	@media (max-width: 768px) {
		.page {
			padding: 1rem;
		}

		.header {
			flex-direction: column;
			gap: 1rem;
		}

		.header-actions {
			width: 100%;
		}

		.header-actions button {
			flex: 1;
		}

		.jobs-grid {
			grid-template-columns: 1fr;
		}

		.result-grid {
			grid-template-columns: repeat(2, 1fr);
		}

		.job-details {
			flex-direction: column;
			gap: 0.5rem;
		}
	}
</style>
