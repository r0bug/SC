<script lang="ts">
	import { api, ApiError } from '$lib/api/api';
	import type { ImportPreview } from '$lib/api/types';
	import { toasts } from '$lib/stores/toast';
	import LoadingSpinner from '$lib/components/ui/LoadingSpinner.svelte';
	import ProgressBar from '$lib/components/ui/ProgressBar.svelte';

	let file: File | null = null;
	let format = 'csv';
	let preview: ImportPreview | null = null;
	let loading = false;
	let importing = false;
	let importResult: { imported: number; errors: any[] } | null = null;
	let importProgress = 0;
	let currentRow = 0;
	let totalRows = 0;
	let estimatedTimeRemaining = '';
	let errorMessage = '';
	let startTime = 0;

	async function handleFileSelect(event: Event) {
		const target = event.target as HTMLInputElement;
		if (target.files && target.files.length > 0) {
			file = target.files[0];

			// Validate file size (10MB max for imports)
			const MAX_SIZE = 10 * 1024 * 1024;
			if (file.size > MAX_SIZE) {
				toasts.error(
					`File is too large. Maximum size is 10MB. Your file is ${(file.size / 1024 / 1024).toFixed(1)}MB.`,
					'File Too Large'
				);
				file = null;
				target.value = '';
				return;
			}

			await previewImport();
		}
	}

	async function previewImport() {
		if (!file) return;

		try {
			loading = true;
			errorMessage = '';
			preview = await api.previewImport(file, format);

			if (preview.validation_errors.length > 0) {
				toasts.warning(
					`Found ${preview.validation_errors.length} validation issue(s) in your file. Please review them below.`,
					'Validation Warnings'
				);
			}
		} catch (error) {
			console.error('Failed to preview import:', error);
			if (error instanceof ApiError) {
				errorMessage = error.getUserMessage();
				toasts.error(errorMessage, 'Preview Failed');
			} else {
				errorMessage = 'Failed to preview file. Please check the format and try again.';
				toasts.error(errorMessage, 'Preview Failed');
			}
		} finally {
			loading = false;
		}
	}

	async function confirmImport() {
		if (!file || !preview) return;

		try {
			importing = true;
			importProgress = 0;
			currentRow = 0;
			totalRows = preview.rows.length;
			errorMessage = '';
			startTime = Date.now();

			// Simulate progress updates (in real implementation, this would come from the server)
			const progressInterval = setInterval(() => {
				if (importProgress < 90) {
					importProgress += 2;
					currentRow = Math.floor((importProgress / 100) * totalRows);

					// Calculate estimated time remaining
					const elapsed = Date.now() - startTime;
					const rate = importProgress / elapsed;
					const remaining = (100 - importProgress) / rate;

					if (remaining > 60000) {
						estimatedTimeRemaining = `${Math.ceil(remaining / 60000)} min`;
					} else {
						estimatedTimeRemaining = `${Math.ceil(remaining / 1000)} sec`;
					}
				}
			}, 200);

			importResult = await api.confirmImport(file, format, preview.field_mappings);

			clearInterval(progressInterval);
			importProgress = 100;
			currentRow = totalRows;

			if (importResult.errors.length === 0) {
				toasts.success(
					`Successfully imported ${importResult.imported} contact(s)`,
					'Import Complete'
				);
			} else {
				toasts.warning(
					`Imported ${importResult.imported} contact(s) with ${importResult.errors.length} error(s)`,
					'Import Complete with Warnings'
				);
			}
		} catch (error) {
			console.error('Import failed:', error);
			if (error instanceof ApiError) {
				errorMessage = error.getUserMessage();
				toasts.error(errorMessage, 'Import Failed', {
					duration: 8000,
					action: error.isRetryable() ? {
						label: 'Retry',
						callback: confirmImport
					} : undefined
				});
			} else {
				errorMessage = 'Import failed. Please try again.';
				toasts.error(errorMessage, 'Import Failed');
			}
		} finally {
			importing = false;
			importProgress = 0;
		}
	}

	function updateMapping(field: string, value: string) {
		if (preview) {
			preview.field_mappings[field] = value;
		}
	}

	function reset() {
		file = null;
		preview = null;
		importResult = null;
		errorMessage = '';
		importProgress = 0;
		currentRow = 0;
		totalRows = 0;
		estimatedTimeRemaining = '';
	}
</script>

<div class="page">
	<div class="header">
		<div>
			<h1>Import Data</h1>
			<p class="subtitle">Import contacts from CSV, vCard, or other formats</p>
		</div>
	</div>

	{#if !importResult}
		<div class="import-container">
			<div class="card">
				<h2>Select Import Format</h2>
				<div class="format-options">
					<label class="format-option">
						<input type="radio" bind:group={format} value="csv" />
						<div class="format-content">
							<strong>CSV</strong>
							<p>Comma-separated values file</p>
						</div>
					</label>
					<label class="format-option">
						<input type="radio" bind:group={format} value="vcard" />
						<div class="format-content">
							<strong>vCard</strong>
							<p>Contact card format (.vcf)</p>
						</div>
					</label>
					<label class="format-option">
						<input type="radio" bind:group={format} value="json" />
						<div class="format-content">
							<strong>JSON</strong>
							<p>JavaScript Object Notation</p>
						</div>
					</label>
				</div>
			</div>

			<div class="card">
				<h2>Upload File</h2>
				<div class="upload-area">
					<input
						type="file"
						id="file-input"
						accept={format === 'csv' ? '.csv' : format === 'vcard' ? '.vcf' : '.json'}
						on:change={handleFileSelect}
					/>
					<label for="file-input" class="upload-label">
						{#if file}
							<div class="file-info">
								<strong>📄 {file.name}</strong>
								<p>{(file.size / 1024).toFixed(2)} KB</p>
							</div>
						{:else}
							<div class="upload-prompt">
								<div class="upload-icon">📁</div>
								<p>Click to select a file or drag and drop</p>
								<p class="text-sm">Supports {format.toUpperCase()} format</p>
							</div>
						{/if}
					</label>
				</div>
			</div>

			{#if errorMessage && !loading && !preview}
				<div class="card error-card">
					<div class="error-icon">⚠</div>
					<h3>Unable to Process File</h3>
					<p class="error-text">{errorMessage}</p>
					<button on:click={reset} class="btn btn-secondary">Try Again</button>
				</div>
			{/if}

			{#if loading}
				<div class="card">
					<LoadingSpinner size="md" message="Analyzing file structure and validating data..." />
				</div>
			{:else if preview}
				<div class="card">
					<h2>Preview & Field Mapping</h2>

					{#if preview.validation_errors.length > 0}
						<div class="validation-errors">
							<h3>⚠️ Validation Issues</h3>
							{#each preview.validation_errors as error}
								<div class="error-item">
									Row {error.row}: {error.field} - {error.error}
								</div>
							{/each}
						</div>
					{/if}

					<div class="mappings">
						<h3>Field Mappings</h3>
						<div class="mapping-grid">
							{#each Object.entries(preview.field_mappings) as [source, target]}
								<div class="mapping-item">
									<span class="source-field">{source}</span>
									<span>→</span>
									<select value={target} on:change={e => updateMapping(source, e.target.value)}>
										<option value="">Skip</option>
										<option value="first_name">First Name</option>
										<option value="last_name">Last Name</option>
										<option value="email">Email</option>
										<option value="phone">Phone</option>
										<option value="organization">Organization</option>
										<option value="title">Title</option>
										<option value="notes">Notes</option>
									</select>
								</div>
							{/each}
						</div>
					</div>

					<div class="preview-table">
						<h3>Data Preview (First 5 Rows)</h3>
						<div class="table-container">
							<table>
								<thead>
									<tr>
										{#each preview.headers as header}
											<th>{header}</th>
										{/each}
									</tr>
								</thead>
								<tbody>
									{#each preview.rows.slice(0, 5) as row}
										<tr>
											{#each row as cell}
												<td>{cell || '-'}</td>
											{/each}
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					</div>

					{#if importing}
						<div class="import-progress">
							<h3>Importing Contacts</h3>
							<ProgressBar
								progress={importProgress}
								label={`Processing row ${currentRow} of ${totalRows}`}
								estimatedTimeRemaining={estimatedTimeRemaining}
								variant="default"
								size="md"
							/>
							<p class="progress-note">Please don't close this window while importing...</p>
						</div>
					{/if}

					<div class="import-actions">
						<button on:click={reset} class="btn btn-secondary" disabled={importing}>Cancel</button>
						<button
							on:click={confirmImport}
							disabled={importing}
							class="btn btn-primary"
						>
							{importing ? 'Importing...' : `Import ${preview.rows.length} Contacts`}
						</button>
					</div>
				</div>
			{/if}
		</div>
	{:else}
		<div class="card success-card">
			<div class="success-icon">✅</div>
			<h2>Import Complete!</h2>
			<p class="success-stats">
				Successfully imported {importResult.imported} contacts
			</p>
			{#if importResult.errors.length > 0}
				<div class="import-errors">
					<h3>Some rows had issues:</h3>
					{#each importResult.errors as error}
						<div class="error-item">{error}</div>
					{/each}
				</div>
			{/if}
			<div class="success-actions">
				<a href="/contacts" class="btn btn-primary">View Contacts</a>
				<button on:click={reset} class="btn btn-secondary">Import More</button>
			</div>
		</div>
	{/if}

	<div class="mock-notice">
		<strong>⚠️ Alpha Notice:</strong> Import service is mocked. Sample data will be generated for testing.
	</div>
</div>

<style>
	.page {
		padding: 2rem;
		max-width: 1000px;
		margin: 0 auto;
	}

	.header {
		margin-bottom: 2rem;
	}

	.subtitle {
		color: var(--gray-600);
		margin-top: 0.25rem;
	}

	.import-container {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.format-options {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 1rem;
		margin-top: 1rem;
	}

	.format-option {
		display: flex;
		align-items: flex-start;
		padding: 1rem;
		border: 2px solid var(--gray-200);
		border-radius: 8px;
		cursor: pointer;
		transition: all 0.2s;
	}

	.format-option:hover {
		border-color: var(--primary);
	}

	.format-option input[type="radio"] {
		margin-right: 1rem;
		margin-top: 0.25rem;
	}

	.format-option input[type="radio"]:checked + .format-content {
		color: var(--primary);
	}

	.format-content p {
		margin: 0.25rem 0 0;
		font-size: 0.875rem;
		color: var(--gray-600);
	}

	.upload-area {
		margin-top: 1rem;
	}

	#file-input {
		display: none;
	}

	.upload-label {
		display: block;
		padding: 3rem;
		border: 2px dashed var(--gray-300);
		border-radius: 8px;
		text-align: center;
		cursor: pointer;
		transition: all 0.2s;
	}

	.upload-label:hover {
		border-color: var(--primary);
		background: var(--gray-50);
	}

	.upload-icon {
		font-size: 3rem;
		margin-bottom: 1rem;
	}

	.file-info {
		text-align: center;
	}

	.validation-errors {
		background: #fef2f2;
		border: 1px solid #fecaca;
		padding: 1rem;
		border-radius: 6px;
		margin-bottom: 1.5rem;
	}

	.validation-errors h3 {
		color: #dc2626;
		margin-bottom: 0.5rem;
	}

	.error-item {
		color: #991b1b;
		font-size: 0.875rem;
		margin: 0.25rem 0;
	}

	.mappings {
		margin: 1.5rem 0;
	}

	.mapping-grid {
		display: grid;
		gap: 1rem;
		margin-top: 1rem;
	}

	.mapping-item {
		display: grid;
		grid-template-columns: 1fr auto 1fr;
		align-items: center;
		gap: 1rem;
		padding: 0.75rem;
		background: var(--gray-50);
		border-radius: 6px;
	}

	.source-field {
		font-weight: 500;
	}

	.mapping-item select {
		padding: 0.5rem;
		border: 1px solid var(--gray-300);
		border-radius: 4px;
	}

	.table-container {
		overflow-x: auto;
		margin-top: 1rem;
	}

	table {
		width: 100%;
		border-collapse: collapse;
	}

	th, td {
		padding: 0.75rem;
		text-align: left;
		border-bottom: 1px solid var(--gray-200);
	}

	th {
		background: var(--gray-50);
		font-weight: 600;
		position: sticky;
		top: 0;
	}

	.import-actions {
		display: flex;
		justify-content: flex-end;
		gap: 1rem;
		margin-top: 2rem;
		padding-top: 1.5rem;
		border-top: 1px solid var(--gray-200);
	}

	.success-card {
		text-align: center;
		padding: 3rem;
	}

	.success-icon {
		font-size: 4rem;
		margin-bottom: 1rem;
	}

	.success-stats {
		font-size: 1.25rem;
		color: var(--gray-700);
		margin: 1rem 0;
	}

	.import-errors {
		background: #fef3c7;
		border: 1px solid #fde68a;
		padding: 1rem;
		border-radius: 6px;
		margin: 1.5rem 0;
		text-align: left;
	}

	.success-actions {
		display: flex;
		gap: 1rem;
		justify-content: center;
		margin-top: 2rem;
	}

	.mock-notice {
		margin-top: 2rem;
		padding: 1rem;
		background: #fef3c7;
		border: 1px solid #fde68a;
		border-radius: 6px;
		text-align: center;
		color: #92400e;
	}

	.loading {
		text-align: center;
		padding: 2rem;
	}

	.text-sm {
		font-size: 0.875rem;
		color: var(--gray-600);
	}

	.error-card {
		text-align: center;
		padding: 2rem;
	}

	.error-icon {
		font-size: 3rem;
		color: #ef4444;
		margin-bottom: 1rem;
	}

	.error-text {
		color: var(--gray-700);
		margin: 1rem 0;
		line-height: 1.6;
	}

	.import-progress {
		margin: 2rem 0;
		padding: 1.5rem;
		background: var(--gray-50);
		border-radius: 8px;
	}

	.import-progress h3 {
		margin: 0 0 1rem;
		font-size: 1rem;
		color: var(--gray-900);
	}

	.progress-note {
		margin: 1rem 0 0;
		font-size: 0.875rem;
		color: var(--gray-600);
		text-align: center;
	}
</style>