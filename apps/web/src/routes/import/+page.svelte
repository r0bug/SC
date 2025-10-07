<script lang="ts">
	import { api, ApiError } from '$lib/api/api';
	import type { ImportPreviewResponse, ImportConfig, ImportJob, Connector } from '$lib/api/types';
	import { toasts } from '$lib/stores/toast';
	import { onMount, onDestroy } from 'svelte';
	import LoadingSpinner from '$lib/components/ui/LoadingSpinner.svelte';
	import ProgressBar from '$lib/components/ui/ProgressBar.svelte';

	let file: File | null = null;
	let connectors: Connector[] = [];
	let selectedConnector: Connector | null = null;
	let preview: ImportPreviewResponse | null = null;
	let loading = false;
	let importing = false;
	let currentJob: ImportJob | null = null;
	let dragActive = false;
	let pollInterval: number | null = null;

	// Import configuration
	let config: ImportConfig = {
		dedupe_strategy: 'Skip',
		match_criteria: 'EmailOrPhone',
		dry_run: false,
		field_mappings: {}
	};

	onMount(async () => {
		try {
			connectors = await api.getConnectors();
		} catch (error) {
			console.error('Failed to load connectors:', error);
			toasts.error('Failed to load import connectors', 'Initialization Error');
		}
	});

	onDestroy(() => {
		if (pollInterval) {
			clearInterval(pollInterval);
		}
	});

	// Drag and drop handlers
	function handleDragOver(e: DragEvent) {
		e.preventDefault();
		dragActive = true;
	}

	function handleDragLeave(e: DragEvent) {
		e.preventDefault();
		dragActive = false;
	}

	function handleDrop(e: DragEvent) {
		e.preventDefault();
		dragActive = false;
		const files = e.dataTransfer?.files;
		if (files?.[0]) {
			file = files[0];
			handleFile(file);
		}
	}

	async function handleFileSelect(event: Event) {
		const target = event.target as HTMLInputElement;
		if (target.files && target.files.length > 0) {
			file = target.files[0];
			await handleFile(file);
		}
	}

	async function handleFile(selectedFile: File) {
		// Validate file size (500MB max for imports)
		const MAX_SIZE = 500 * 1024 * 1024;
		if (selectedFile.size > MAX_SIZE) {
			toasts.error(
				`File is too large. Maximum size is 500MB. Your file is ${(selectedFile.size / 1024 / 1024).toFixed(1)}MB.`,
				'File Too Large'
			);
			file = null;
			return;
		}

		await previewImport();
	}

	async function previewImport() {
		if (!file) return;

		try {
			loading = true;
			preview = await api.previewImport(file, 10);

			// Auto-select connector if detected
			selectedConnector = connectors.find(c => c.id === preview?.connector_id) || null;

			// Auto-populate field mappings from detected fields
			if (preview.detected_fields) {
				preview.detected_fields.forEach(field => {
					if (field.confidence > 0.7) {
						config.field_mappings![field.field_name] = field.suggested_mapping;
					}
				});
			}

			if (preview.validation_errors.length > 0) {
				toasts.warning(
					`Found ${preview.validation_errors.length} validation issue(s) in your file.`,
					'Validation Warnings'
				);
			}

			if (preview.warnings.length > 0) {
				toasts.info(
					`${preview.warnings.length} warning(s) found. Review before importing.`,
					'Data Warnings'
				);
			}
		} catch (error) {
			console.error('Failed to preview import:', error);
			if (error instanceof ApiError) {
				toasts.error(error.getUserMessage(), 'Preview Failed');
			} else {
				toasts.error('Failed to preview file. Please check the format and try again.', 'Preview Failed');
			}
			file = null;
			preview = null;
		} finally {
			loading = false;
		}
	}

	async function executeImport() {
		if (!file || !preview) return;

		try {
			importing = true;
			config.connector_id = preview.connector_id;

			const jobResponse = await api.executeImport(file, config);
			currentJob = {
				id: jobResponse.job_id,
				file_name: file.name,
				connector_id: preview.connector_id,
				status: jobResponse.status,
				progress: { current: 0, total: preview.total_rows, phase: 'Starting...' },
				created_at: jobResponse.created_at,
				updated_at: jobResponse.created_at
			};

			// Start polling for job status
			startJobPolling();

		} catch (error) {
			console.error('Import failed:', error);
			if (error instanceof ApiError) {
				toasts.error(error.getUserMessage(), 'Import Failed');
			} else {
				toasts.error('Import failed. Please try again.', 'Import Failed');
			}
			importing = false;
		}
	}

	function startJobPolling() {
		if (pollInterval) clearInterval(pollInterval);

		pollInterval = window.setInterval(async () => {
			if (!currentJob) {
				clearInterval(pollInterval!);
				return;
			}

			try {
				const updatedJob = await api.getJobStatus(currentJob.id);
				currentJob = updatedJob;

				if (updatedJob.status === 'Completed') {
					clearInterval(pollInterval!);
					importing = false;
					toasts.success(
						`Successfully imported ${updatedJob.result?.imported || 0} contact(s)`,
						'Import Complete'
					);
				} else if (updatedJob.status === 'Failed') {
					clearInterval(pollInterval!);
					importing = false;
					toasts.error(
						`Import failed: ${updatedJob.result?.failed || 0} errors`,
						'Import Failed'
					);
				} else if (updatedJob.status === 'Cancelled') {
					clearInterval(pollInterval!);
					importing = false;
					toasts.info('Import cancelled', 'Cancelled');
				}
			} catch (error) {
				console.error('Failed to check job status:', error);
			}
		}, 2000); // Poll every 2 seconds
	}

	async function cancelImport() {
		if (!currentJob) return;

		try {
			await api.cancelJob(currentJob.id);
			toasts.info('Import cancelled', 'Cancelled');
		} catch (error) {
			console.error('Failed to cancel import:', error);
			toasts.error('Failed to cancel import', 'Error');
		}
	}

	function updateMapping(field: string, value: string) {
		config.field_mappings![field] = value;
	}

	function reset() {
		file = null;
		preview = null;
		currentJob = null;
		config.field_mappings = {};
		if (pollInterval) {
			clearInterval(pollInterval);
			pollInterval = null;
		}
	}

	function viewJobs() {
		window.location.href = '/import/jobs';
	}

	function viewHistory() {
		window.location.href = '/import/history';
	}
</script>

<div class="page">
	<div class="header">
		<div>
			<h1>Import Data</h1>
			<p class="subtitle">Import contacts from various sources with intelligent deduplication</p>
		</div>
		<div class="header-actions">
			<button on:click={viewJobs} class="btn btn-secondary">
				📋 View Jobs
			</button>
			<button on:click={viewHistory} class="btn btn-secondary">
				📊 Import History
			</button>
		</div>
	</div>

	{#if !currentJob}
		<div class="import-container">
			<!-- Upload Area -->
			<div class="card">
				<h2>Upload File</h2>
				<div
					class="upload-area"
					class:drag-active={dragActive}
					on:dragover={handleDragOver}
					on:dragleave={handleDragLeave}
					on:drop={handleDrop}
				>
					<input
						type="file"
						id="file-input"
						accept=".csv,.vcf,.xml,.json,.mbox"
						on:change={handleFileSelect}
						style="display: none;"
					/>
					<label for="file-input" class="upload-label">
						{#if file}
							<div class="file-info">
								<div class="file-icon">📄</div>
								<div class="file-details">
									<strong>{file.name}</strong>
									<p>{(file.size / 1024).toFixed(2)} KB</p>
									{#if selectedConnector}
										<p class="connector-badge">{selectedConnector.name}</p>
									{/if}
								</div>
								<button class="btn-icon" on:click|preventDefault={reset} title="Remove file">
									✕
								</button>
							</div>
						{:else}
							<div class="upload-prompt">
								<div class="upload-icon">📁</div>
								<p><strong>Drag and drop a file here</strong></p>
								<p class="text-sm">or click to browse</p>
								<p class="text-xs">Supports CSV, vCard, XML, JSON, MBOX (max 50MB)</p>
							</div>
						{/if}
					</label>
				</div>

				{#if connectors.length > 0}
					<div class="connectors-info">
						<h3>Supported Sources</h3>
						<div class="connectors-grid">
							{#each connectors.filter(c => c.supports_preview) as connector}
								<div class="connector-item">
									<strong>{connector.name}</strong>
									<p class="text-xs">{connector.file_extensions.join(', ')}</p>
								</div>
							{/each}
						</div>
					</div>
				{/if}
			</div>

			{#if loading}
				<div class="card">
					<LoadingSpinner size="md" message="Analyzing file structure and validating data..." />
				</div>
			{:else if preview}
				<!-- Preview & Configuration -->
				<div class="card">
					<h2>Preview & Configuration</h2>

					<div class="preview-summary">
						<div class="summary-item">
							<span class="label">File:</span>
							<span class="value">{preview.file_name}</span>
						</div>
						<div class="summary-item">
							<span class="label">Total Rows:</span>
							<span class="value">{preview.total_rows}</span>
						</div>
						<div class="summary-item">
							<span class="label">Connector:</span>
							<span class="value">{selectedConnector?.name || preview.connector_id}</span>
						</div>
					</div>

					{#if preview.validation_errors.length > 0}
						<div class="validation-errors">
							<h3>⚠️ Validation Errors ({preview.validation_errors.length})</h3>
							<div class="error-list">
								{#each preview.validation_errors.slice(0, 10) as error}
									<div class="error-item">
										<span class="error-row">Row {error.row}</span>
										<span class="error-field">{error.field}</span>
										<span class="error-message">{error.error}</span>
									</div>
								{/each}
								{#if preview.validation_errors.length > 10}
									<p class="text-sm">... and {preview.validation_errors.length - 10} more errors</p>
								{/if}
							</div>
						</div>
					{/if}

					{#if preview.warnings.length > 0}
						<div class="validation-warnings">
							<h3>ℹ️ Warnings ({preview.warnings.length})</h3>
							<div class="warning-list">
								{#each preview.warnings.slice(0, 5) as warning}
									<div class="warning-item {warning.severity ? `severity-${warning.severity.toLowerCase()}` : ''}">
										<span class="warning-row">Row {warning.row}</span>
										<span class="warning-field">{warning.field}</span>
										<span class="warning-message">{warning.warning}</span>
									</div>
								{/each}
								{#if preview.warnings.length > 5}
									<p class="text-sm">... and {preview.warnings.length - 5} more warnings</p>
								{/if}
							</div>
						</div>
					{/if}

					<!-- Field Mappings -->
					<div class="field-mappings">
						<h3>Field Mappings</h3>
						<div class="mappings-grid">
							{#each preview.detected_fields as field}
								<div class="mapping-item">
									<label>{field.field_name}</label>
									<select
										value={config.field_mappings?.[field.field_name] || field.suggested_mapping}
										on:change={(e) => updateMapping(field.field_name, e.currentTarget.value)}
									>
										<option value="">-- Skip --</option>
										<option value="first_name">First Name</option>
										<option value="last_name">Last Name</option>
										<option value="email">Email</option>
										<option value="phone">Phone</option>
										<option value="organization">Organization</option>
										<option value="title">Title</option>
										<option value="notes">Notes</option>
									</select>
									<span class="confidence" class:high={field.confidence > 0.8}>
										{Math.round(field.confidence * 100)}% confident
									</span>
								</div>
							{/each}
						</div>
					</div>

					<!-- Import Configuration -->
					<div class="import-config">
						<h3>Import Settings</h3>

						<div class="config-row">
							<label for="dedupe-strategy">Deduplication Strategy</label>
							<select id="dedupe-strategy" bind:value={config.dedupe_strategy}>
								<option value="Skip">Skip duplicates</option>
								<option value="Update">Update existing</option>
								<option value="Merge">Merge data</option>
								<option value="KeepBoth">Keep both</option>
							</select>
						</div>

						<div class="config-row">
							<label for="match-criteria">Match Criteria</label>
							<select id="match-criteria" bind:value={config.match_criteria}>
								<option value="Email">Email address</option>
								<option value="Phone">Phone number</option>
								<option value="FullName">Full name (fuzzy)</option>
								<option value="EmailOrPhone">Email or Phone</option>
							</select>
						</div>

						<div class="config-row">
							<label>
								<input type="checkbox" bind:checked={config.dry_run} />
								Dry run (preview only, don't save)
							</label>
						</div>
					</div>

					<!-- Preview Data -->
					<div class="preview-data">
						<h3>Preview Data (first 10 rows)</h3>
						<div class="preview-table">
							<table>
								<thead>
									<tr>
										<th>#</th>
										{#each Object.keys(preview.preview_rows[0]?.data || {}) as header}
											<th>{header}</th>
										{/each}
									</tr>
								</thead>
								<tbody>
									{#each preview.preview_rows as row}
										<tr class:has-errors={row.has_errors}>
											<td>{row.row_number}</td>
											{#each Object.values(row.data || {}) as value}
												<td>{value || '-'}</td>
											{/each}
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					</div>

					<div class="actions">
						<button on:click={reset} class="btn btn-secondary">
							Cancel
						</button>
						<button
							on:click={executeImport}
							class="btn btn-primary"
							disabled={preview.validation_errors.length > 0}
						>
							{config.dry_run ? 'Preview Import' : 'Start Import'}
						</button>
					</div>
				</div>
			{/if}
		</div>
	{:else}
		<!-- Import Progress -->
		<div class="card">
			<h2>Import in Progress</h2>

			<div class="job-info">
				<div class="job-header">
					<h3>{currentJob.file_name}</h3>
					<span class="status-badge status-{currentJob.status.toLowerCase()}">
						{currentJob.status}
					</span>
				</div>

				<div class="job-progress">
					<ProgressBar
						value={currentJob.progress.current}
						max={currentJob.progress.total}
					/>
					<p class="progress-phase">{currentJob.progress.phase}</p>
					<p class="progress-text">
						{currentJob.progress.current} / {currentJob.progress.total} rows
					</p>
				</div>

				{#if currentJob.result}
					<div class="job-results">
						<div class="result-item">
							<span class="result-label">Imported:</span>
							<span class="result-value success">{currentJob.result.imported}</span>
						</div>
						<div class="result-item">
							<span class="result-label">Updated:</span>
							<span class="result-value">{currentJob.result.updated || 0}</span>
						</div>
						<div class="result-item">
							<span class="result-label">Skipped:</span>
							<span class="result-value warning">{currentJob.result.skipped}</span>
						</div>
						<div class="result-item">
							<span class="result-label">Failed:</span>
							<span class="result-value error">{currentJob.result.failed}</span>
						</div>
						<div class="result-item">
							<span class="result-label">Duplicates:</span>
							<span class="result-value">{currentJob.result.duplicates_found}</span>
						</div>
						<div class="result-item">
							<span class="result-label">Time:</span>
							<span class="result-value">{currentJob.result.elapsed_seconds}s</span>
						</div>
					</div>
				{/if}

				<div class="actions">
					{#if currentJob.status === 'Pending' || currentJob.status === 'Importing'}
						<button on:click={cancelImport} class="btn btn-danger">
							Cancel Import
						</button>
					{/if}
					{#if currentJob.status === 'Completed' || currentJob.status === 'Failed'}
						<button on:click={reset} class="btn btn-secondary">
							Import Another File
						</button>
						<button on:click={viewHistory} class="btn btn-primary">
							View Details
						</button>
					{/if}
				</div>
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

	.import-container {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.card {
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
		padding: 1.5rem;
	}

	.card h2 {
		font-size: 1.25rem;
		font-weight: 600;
		margin: 0 0 1rem 0;
	}

	.card h3 {
		font-size: 1rem;
		font-weight: 600;
		margin: 1.5rem 0 1rem 0;
	}

	.upload-area {
		border: 2px dashed #d1d5db;
		border-radius: 0.5rem;
		transition: all 0.2s;
	}

	.upload-area.drag-active {
		border-color: #3b82f6;
		background-color: #eff6ff;
	}

	.upload-label {
		display: block;
		padding: 3rem 2rem;
		cursor: pointer;
		text-align: center;
	}

	.upload-prompt {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
	}

	.upload-icon {
		font-size: 3rem;
		margin-bottom: 0.5rem;
	}

	.file-info {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding: 1rem;
		background: #f9fafb;
		border-radius: 0.5rem;
	}

	.file-icon {
		font-size: 2rem;
	}

	.file-details {
		flex: 1;
		text-align: left;
	}

	.connector-badge {
		display: inline-block;
		padding: 0.25rem 0.5rem;
		background: #3b82f6;
		color: white;
		border-radius: 0.25rem;
		font-size: 0.75rem;
		margin-top: 0.25rem;
	}

	.btn-icon {
		background: none;
		border: none;
		font-size: 1.5rem;
		cursor: pointer;
		color: #6b7280;
		padding: 0.5rem;
	}

	.btn-icon:hover {
		color: #ef4444;
	}

	.connectors-info {
		margin-top: 1.5rem;
	}

	.connectors-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
		gap: 1rem;
		margin-top: 0.5rem;
	}

	.connector-item {
		padding: 0.75rem;
		background: #f9fafb;
		border-radius: 0.375rem;
	}

	.preview-summary {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 1rem;
		padding: 1rem;
		background: #f9fafb;
		border-radius: 0.5rem;
		margin-bottom: 1.5rem;
	}

	.summary-item {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.summary-item .label {
		font-size: 0.875rem;
		color: #6b7280;
	}

	.summary-item .value {
		font-weight: 600;
	}

	.validation-errors, .validation-warnings {
		margin: 1.5rem 0;
		padding: 1rem;
		border-radius: 0.5rem;
	}

	.validation-errors {
		background: #fef2f2;
		border: 1px solid #fecaca;
	}

	.validation-warnings {
		background: #fffbeb;
		border: 1px solid #fed7aa;
	}

	.error-list, .warning-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		margin-top: 0.5rem;
		max-height: 300px;
		overflow-y: auto;
	}

	.error-item, .warning-item {
		display: grid;
		grid-template-columns: auto 150px 1fr;
		gap: 1rem;
		padding: 0.5rem;
		background: white;
		border-radius: 0.25rem;
		font-size: 0.875rem;
	}

	.error-row, .warning-row {
		color: #6b7280;
		font-weight: 500;
	}

	.error-field, .warning-field {
		font-weight: 600;
	}

	.field-mappings {
		margin-top: 1.5rem;
	}

	.mappings-grid {
		display: grid;
		gap: 1rem;
		margin-top: 0.5rem;
	}

	.mapping-item {
		display: grid;
		grid-template-columns: 200px 1fr auto;
		gap: 1rem;
		align-items: center;
	}

	.mapping-item label {
		font-weight: 500;
	}

	.mapping-item select {
		padding: 0.5rem;
		border: 1px solid #d1d5db;
		border-radius: 0.375rem;
	}

	.confidence {
		font-size: 0.75rem;
		color: #6b7280;
	}

	.confidence.high {
		color: #059669;
		font-weight: 600;
	}

	.import-config {
		margin-top: 1.5rem;
		padding: 1rem;
		background: #f9fafb;
		border-radius: 0.5rem;
	}

	.config-row {
		display: flex;
		align-items: center;
		gap: 1rem;
		margin-bottom: 1rem;
	}

	.config-row label {
		min-width: 200px;
		font-weight: 500;
	}

	.config-row select {
		flex: 1;
		padding: 0.5rem;
		border: 1px solid #d1d5db;
		border-radius: 0.375rem;
	}

	.preview-data {
		margin-top: 1.5rem;
	}

	.preview-table {
		overflow-x: auto;
		margin-top: 0.5rem;
	}

	.preview-table table {
		width: 100%;
		border-collapse: collapse;
		font-size: 0.875rem;
	}

	.preview-table th,
	.preview-table td {
		padding: 0.75rem;
		text-align: left;
		border-bottom: 1px solid #e5e7eb;
	}

	.preview-table th {
		background: #f9fafb;
		font-weight: 600;
	}

	.preview-table tr.has-errors {
		background: #fef2f2;
	}

	.actions {
		display: flex;
		justify-content: flex-end;
		gap: 1rem;
		margin-top: 1.5rem;
		padding-top: 1.5rem;
		border-top: 1px solid #e5e7eb;
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

	.btn-primary:hover:not(:disabled) {
		background: #2563eb;
	}

	.btn-primary:disabled {
		background: #9ca3af;
		cursor: not-allowed;
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

	.job-info {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.job-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.status-badge {
		padding: 0.375rem 0.75rem;
		border-radius: 0.375rem;
		font-size: 0.875rem;
		font-weight: 600;
	}

	.status-pending { background: #fef3c7; color: #92400e; }
	.status-validating, .status-parsing { background: #dbeafe; color: #1e40af; }
	.status-importing { background: #bfdbfe; color: #1e3a8a; }
	.status-completed { background: #d1fae5; color: #065f46; }
	.status-failed { background: #fee2e2; color: #991b1b; }
	.status-cancelled { background: #f3f4f6; color: #374151; }

	.job-progress {
		text-align: center;
	}

	.progress-phase {
		margin-top: 1rem;
		font-weight: 600;
		color: #374151;
	}

	.progress-text {
		margin-top: 0.5rem;
		color: #6b7280;
		font-size: 0.875rem;
	}

	.job-results {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
		gap: 1rem;
		padding: 1rem;
		background: #f9fafb;
		border-radius: 0.5rem;
	}

	.result-item {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.result-label {
		font-size: 0.875rem;
		color: #6b7280;
	}

	.result-value {
		font-size: 1.5rem;
		font-weight: 700;
	}

	.result-value.success { color: #059669; }
	.result-value.warning { color: #d97706; }
	.result-value.error { color: #dc2626; }

	.text-xs { font-size: 0.75rem; color: #6b7280; }
	.text-sm { font-size: 0.875rem; color: #6b7280; }

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

		.mapping-item,
		.config-row {
			grid-template-columns: 1fr;
		}

		.job-results {
			grid-template-columns: repeat(2, 1fr);
		}
	}
</style>
