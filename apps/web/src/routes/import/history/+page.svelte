<script lang="ts">
	import { api } from '$lib/api/api';
	import type { ImportLog, Connector, ImportHistoryFilters } from '$lib/api/types';
	import { toasts } from '$lib/stores/toast';
	import { onMount } from 'svelte';
	import LoadingSpinner from '$lib/components/ui/LoadingSpinner.svelte';

	let logs: ImportLog[] = [];
	let connectors: Connector[] = [];
	let loading = true;
	let showFilters = false;
	let filters: ImportHistoryFilters = {};
	let selectedLog: ImportLog | null = null;
	let showRollbackConfirm = false;

	onMount(async () => {
		await Promise.all([
			loadHistory(),
			loadConnectors()
		]);
	});

	async function loadHistory() {
		try {
			loading = true;
			logs = await api.getImportHistory(filters);
		} catch (error) {
			console.error('Failed to load import history:', error);
			toasts.error('Failed to load import history', 'Error');
		} finally {
			loading = false;
		}
	}

	async function loadConnectors() {
		try {
			connectors = await api.getConnectors();
		} catch (error) {
			console.error('Failed to load connectors:', error);
		}
	}

	async function applyFilters() {
		await loadHistory();
		showFilters = false;
	}

	function clearFilters() {
		filters = {};
		loadHistory();
	}

	function formatDate(dateString: string): string {
		return new Date(dateString).toLocaleString();
	}

	function formatDuration(seconds: number): string {
		if (seconds < 60) return `${seconds}s`;
		const minutes = Math.floor(seconds / 60);
		const secs = seconds % 60;
		return `${minutes}m ${secs}s`;
	}

	async function downloadErrorReport(logId: string) {
		try {
			await api.downloadErrorReport(logId);
			toasts.success('Error report downloaded', 'Success');
		} catch (error) {
			console.error('Failed to download error report:', error);
			toasts.error('Failed to download error report', 'Error');
		}
	}

	function confirmRollback(log: ImportLog) {
		selectedLog = log;
		showRollbackConfirm = true;
	}

	async function executeRollback() {
		if (!selectedLog) return;

		try {
			const result = await api.rollbackImport(selectedLog.id);
			if (result.success) {
				toasts.success(`Rolled back ${result.records_reverted} records`, 'Rollback Complete');
				await loadHistory();
			} else {
				toasts.error(`Rollback failed: ${result.errors.join(', ')}`, 'Rollback Failed');
			}
		} catch (error) {
			console.error('Failed to rollback import:', error);
			toasts.error('Failed to rollback import', 'Error');
		} finally {
			showRollbackConfirm = false;
			selectedLog = null;
		}
	}

	function getStatusColor(status: string): string {
		switch (status) {
			case 'Completed': return 'success';
			case 'Failed': return 'error';
			case 'Cancelled': return 'neutral';
			default: return 'info';
		}
	}

	function goToImport() {
		window.location.href = '/import';
	}

	function goToJobs() {
		window.location.href = '/import/jobs';
	}
</script>

<div class="page">
	<div class="header">
		<div>
			<h1>Import History</h1>
			<p class="subtitle">View and manage past imports</p>
		</div>
		<div class="header-actions">
			<button on:click={() => showFilters = !showFilters} class="btn btn-secondary">
				🔍 {showFilters ? 'Hide' : 'Show'} Filters
			</button>
			<button on:click={goToJobs} class="btn btn-secondary">
				📋 Active Jobs
			</button>
			<button on:click={goToImport} class="btn btn-primary">
				➕ New Import
			</button>
		</div>
	</div>

	{#if showFilters}
		<div class="filters-card">
			<h3>Filters</h3>
			<div class="filters-grid">
				<div class="filter-item">
					<label for="status-filter">Status</label>
					<select id="status-filter" bind:value={filters.status}>
						<option value="">All Statuses</option>
						<option value="Completed">Completed</option>
						<option value="Failed">Failed</option>
						<option value="Cancelled">Cancelled</option>
					</select>
				</div>

				<div class="filter-item">
					<label for="connector-filter">Connector</label>
					<select id="connector-filter" bind:value={filters.connector_id}>
						<option value="">All Connectors</option>
						{#each connectors as connector}
							<option value={connector.id}>{connector.name}</option>
						{/each}
					</select>
				</div>

				<div class="filter-item">
					<label for="start-date-filter">Start Date</label>
					<input
						type="date"
						id="start-date-filter"
						bind:value={filters.start_date}
					/>
				</div>

				<div class="filter-item">
					<label for="end-date-filter">End Date</label>
					<input
						type="date"
						id="end-date-filter"
						bind:value={filters.end_date}
					/>
				</div>
			</div>

			<div class="filter-actions">
				<button on:click={clearFilters} class="btn btn-secondary">
					Clear Filters
				</button>
				<button on:click={applyFilters} class="btn btn-primary">
					Apply Filters
				</button>
			</div>
		</div>
	{/if}

	{#if loading}
		<div class="card">
			<LoadingSpinner size="md" message="Loading import history..." />
		</div>
	{:else if logs.length === 0}
		<div class="empty-state">
			<div class="empty-icon">📊</div>
			<h2>No Import History</h2>
			<p>No imports found {Object.keys(filters).length > 0 ? 'matching your filters' : 'yet'}.</p>
			{#if Object.keys(filters).length > 0}
				<button on:click={clearFilters} class="btn btn-secondary">
					Clear Filters
				</button>
			{/if}
			<button on:click={goToImport} class="btn btn-primary">
				Start an Import
			</button>
		</div>
	{:else}
		<div class="history-table-container">
			<table class="history-table">
				<thead>
					<tr>
						<th>Date</th>
						<th>File</th>
						<th>Connector</th>
						<th>Status</th>
						<th>Total Rows</th>
						<th>Imported</th>
						<th>Skipped</th>
						<th>Failed</th>
						<th>Duration</th>
						<th>Actions</th>
					</tr>
				</thead>
				<tbody>
					{#each logs as log}
						<tr>
							<td>{formatDate(log.created_at)}</td>
							<td>
								<div class="file-cell">
									<strong>{log.file_name}</strong>
									{#if log.peak_memory_mb}
										<span class="memory-badge">{log.peak_memory_mb}MB</span>
									{/if}
								</div>
							</td>
							<td>{log.connector_id}</td>
							<td>
								<span class="status-badge status-{getStatusColor(log.status)}">
									{log.status}
								</span>
							</td>
							<td>{log.total_rows.toLocaleString()}</td>
							<td class="num-cell success">{log.imported.toLocaleString()}</td>
							<td class="num-cell warning">{log.skipped.toLocaleString()}</td>
							<td class="num-cell error">{log.failed.toLocaleString()}</td>
							<td>{formatDuration(log.elapsed_seconds)}</td>
							<td>
								<div class="action-buttons">
									{#if log.failed > 0}
										<button
											on:click={() => downloadErrorReport(log.id)}
											class="btn-icon"
											title="Download Error Report"
										>
											📥
										</button>
									{/if}
									{#if log.status === 'Completed' && log.imported > 0}
										<button
											on:click={() => confirmRollback(log)}
											class="btn-icon"
											title="Rollback Import"
										>
											↩️
										</button>
									{/if}
								</div>
							</td>
						</tr>
						<tr class="details-row">
							<td colspan="10">
								<div class="import-details">
									<div class="detail-group">
										<span class="detail-label">Deduplication:</span>
										<span class="detail-value">{log.dedupe_strategy}</span>
									</div>
									<div class="detail-group">
										<span class="detail-label">Match Criteria:</span>
										<span class="detail-value">{log.match_criteria}</span>
									</div>
									<div class="detail-group">
										<span class="detail-label">Duplicates Found:</span>
										<span class="detail-value">{log.duplicates_found}</span>
									</div>
									{#if log.completed_at}
										<div class="detail-group">
											<span class="detail-label">Completed:</span>
											<span class="detail-value">{formatDate(log.completed_at)}</span>
										</div>
									{/if}
								</div>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		<div class="summary-card">
			<h3>Summary</h3>
			<div class="summary-grid">
				<div class="summary-item">
					<span class="summary-label">Total Imports</span>
					<span class="summary-value">{logs.length}</span>
				</div>
				<div class="summary-item">
					<span class="summary-label">Total Contacts Imported</span>
					<span class="summary-value">{logs.reduce((sum, log) => sum + log.imported, 0).toLocaleString()}</span>
				</div>
				<div class="summary-item">
					<span class="summary-label">Success Rate</span>
					<span class="summary-value">
						{logs.filter(l => l.status === 'Completed').length > 0
							? Math.round((logs.filter(l => l.status === 'Completed').length / logs.length) * 100)
							: 0}%
					</span>
				</div>
			</div>
		</div>
	{/if}
</div>

{#if showRollbackConfirm && selectedLog}
	<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
	<div class="modal-overlay" role="button" tabindex="0" on:click={() => showRollbackConfirm = false} on:keydown={(e) => e.key === 'Enter' && (showRollbackConfirm = false)}>
		<div class="modal" role="dialog" aria-modal="true" on:click|stopPropagation on:keydown|stopPropagation>
			<h2>Confirm Rollback</h2>
			<p>Are you sure you want to rollback the import of <strong>{selectedLog.file_name}</strong>?</p>
			<p class="warning-text">This will remove {selectedLog.imported} imported contacts. This action cannot be undone.</p>

			<div class="modal-actions">
				<button on:click={() => showRollbackConfirm = false} class="btn btn-secondary">
					Cancel
				</button>
				<button on:click={executeRollback} class="btn btn-danger">
					Rollback Import
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.page {
		padding: 2rem;
		max-width: 1600px;
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

	.card, .filters-card, .summary-card {
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
		padding: 1.5rem;
		margin-bottom: 1.5rem;
	}

	.filters-card h3, .summary-card h3 {
		font-size: 1.125rem;
		font-weight: 600;
		margin: 0 0 1rem 0;
	}

	.filters-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
		gap: 1rem;
		margin-bottom: 1rem;
	}

	.filter-item {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.filter-item label {
		font-size: 0.875rem;
		font-weight: 500;
		color: #374151;
	}

	.filter-item select,
	.filter-item input {
		padding: 0.5rem;
		border: 1px solid #d1d5db;
		border-radius: 0.375rem;
		font-size: 0.875rem;
	}

	.filter-actions {
		display: flex;
		justify-content: flex-end;
		gap: 0.5rem;
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

	.empty-state button {
		margin: 0.5rem;
	}

	.history-table-container {
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
		overflow-x: auto;
		margin-bottom: 1.5rem;
	}

	.history-table {
		width: 100%;
		border-collapse: collapse;
		font-size: 0.875rem;
	}

	.history-table th,
	.history-table td {
		padding: 1rem;
		text-align: left;
		border-bottom: 1px solid #e5e7eb;
	}

	.history-table th {
		background: #f9fafb;
		font-weight: 600;
		position: sticky;
		top: 0;
		z-index: 10;
	}

	.history-table tbody tr:hover {
		background: #f9fafb;
	}

	.details-row {
		background: #f9fafb !important;
	}

	.details-row td {
		padding: 0.5rem 1rem;
	}

	.import-details {
		display: flex;
		gap: 2rem;
		flex-wrap: wrap;
		font-size: 0.8125rem;
	}

	.detail-group {
		display: flex;
		gap: 0.5rem;
	}

	.detail-label {
		color: #6b7280;
	}

	.detail-value {
		font-weight: 600;
	}

	.file-cell {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.memory-badge {
		font-size: 0.75rem;
		color: #6b7280;
		background: #f3f4f6;
		padding: 0.125rem 0.375rem;
		border-radius: 0.25rem;
		width: fit-content;
	}

	.status-badge {
		padding: 0.25rem 0.625rem;
		border-radius: 0.375rem;
		font-size: 0.75rem;
		font-weight: 600;
		white-space: nowrap;
	}

	.status-success { background: #d1fae5; color: #065f46; }
	.status-error { background: #fee2e2; color: #991b1b; }
	.status-info { background: #dbeafe; color: #1e40af; }
	.status-neutral { background: #f3f4f6; color: #374151; }

	.num-cell {
		font-weight: 600;
	}

	.num-cell.success { color: #065f46; }
	.num-cell.warning { color: #92400e; }
	.num-cell.error { color: #991b1b; }

	.action-buttons {
		display: flex;
		gap: 0.5rem;
	}

	.btn-icon {
		background: none;
		border: none;
		font-size: 1.25rem;
		cursor: pointer;
		padding: 0.25rem;
		transition: transform 0.2s;
	}

	.btn-icon:hover {
		transform: scale(1.2);
	}

	.summary-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 1.5rem;
	}

	.summary-item {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.summary-label {
		font-size: 0.875rem;
		color: #6b7280;
	}

	.summary-value {
		font-size: 2rem;
		font-weight: 700;
		color: #111827;
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

	.modal-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		background: white;
		border-radius: 0.5rem;
		padding: 2rem;
		max-width: 500px;
		width: 90%;
	}

	.modal h2 {
		margin: 0 0 1rem 0;
		font-size: 1.5rem;
		font-weight: 600;
	}

	.modal p {
		margin: 0 0 1rem 0;
		color: #374151;
	}

	.warning-text {
		color: #dc2626;
		font-weight: 500;
	}

	.modal-actions {
		display: flex;
		justify-content: flex-end;
		gap: 0.5rem;
		margin-top: 1.5rem;
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
			flex-wrap: wrap;
		}

		.header-actions button {
			flex: 1;
			min-width: 120px;
		}

		.history-table-container {
			overflow-x: scroll;
		}

		.history-table {
			min-width: 900px;
		}
	}
</style>
