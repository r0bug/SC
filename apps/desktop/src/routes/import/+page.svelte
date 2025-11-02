<script lang="ts">
	import { tauriApi } from '$lib/api/tauri-api';
	import type { ImportAnalysis, SmartImportPreview } from '$lib/api/types';
	import { open } from '@tauri-apps/plugin-dialog';

	// Wizard state
	let currentStep = 1;
	let importing = false;
	let error = '';

	// Step 1: File selection & analysis
	let selectedFile = '';
	let analyzing = false;
	let analysis: ImportAnalysis | null = null;

	// Step 2: Field mapping & preview
	let fieldMappings: Record<string, string> = {};
	let preview: SmartImportPreview | null = null;
	let loadingPreview = false;

	// Step 3: Import results
	let importedCount = 0;
	let importComplete = false;

	// Available contact fields for mapping
	const contactFields = [
		{ value: '', label: '-- Skip field --' },
		{ value: 'first_name', label: 'First Name *' },
		{ value: 'last_name', label: 'Last Name' },
		{ value: 'email', label: 'Email' },
		{ value: 'phone', label: 'Phone' },
		{ value: 'organization', label: 'Organization' },
		{ value: 'title', label: 'Job Title' },
		{ value: 'notes', label: 'Notes' },
		{ value: 'birthday', label: 'Birthday' },
		{ value: 'address', label: 'Address' },
		{ value: 'city', label: 'City' },
		{ value: 'state', label: 'State' },
		{ value: 'postal_code', label: 'Postal Code' },
		{ value: 'country', label: 'Country' },
		{ value: 'website', label: 'Website' },
		{ value: 'linkedin_url', label: 'LinkedIn URL' },
		{ value: 'twitter_handle', label: 'Twitter Handle' }
	];

	async function handleSelectFile() {
		error = '';

		try {
			console.log('Opening file dialog...');
			const selected = await open({
				multiple: false,
				filters: [
					{
						name: 'All Supported',
						extensions: ['csv', 'json', 'xml', 'html', 'htm', 'txt']
					},
					{ name: 'CSV', extensions: ['csv'] },
					{ name: 'JSON', extensions: ['json'] },
					{ name: 'XML', extensions: ['xml'] },
					{ name: 'HTML', extensions: ['html', 'htm'] },
					{ name: 'Text', extensions: ['txt'] }
				]
			});

			console.log('File dialog result:', selected);

			if (selected && typeof selected === 'string') {
				selectedFile = selected;
				console.log('Selected file:', selectedFile);
				await analyzeFile();
			} else {
				console.log('No file selected or dialog cancelled');
			}
		} catch (err: any) {
			console.error('File dialog error:', err);
			error = `Failed to open file dialog: ${err.message || err}`;
		}
	}

	async function analyzeFile() {
		if (!selectedFile) return;

		try {
			analyzing = true;
			error = '';

			// Check if this is an SMS XML file
			if (selectedFile.toLowerCase().endsWith('.xml') && await isSmsFile(selectedFile)) {
				// Handle SMS file specially
				analysis = {
					detected_format: 'SMS_BACKUP',
					confidence: 1.0,
					detected_fields: [],
					suggested_mappings: {},
					structure_notes: 'Android SMS Backup XML detected. Will import messages and attach to existing contacts.',
					warnings: []
				};
				currentStep = 3; // Skip to import step
				await executeSmsImport();
				return;
			}

			analysis = await tauriApi.analyzeImportFile(selectedFile);
			fieldMappings = { ...analysis.suggested_mappings };

			// Auto-advance to step 2 if confidence is good
			if (analysis.confidence >= 0.7) {
				currentStep = 2;
				await loadPreview();
			} else {
				currentStep = 2; // Still advance but show warnings
			}
		} catch (err: any) {
			console.error('Analysis failed:', err);
			error = err.message || 'Failed to analyze file. Please try another file.';
		} finally {
			analyzing = false;
		}
	}

	async function isSmsFile(filePath: string): Promise<boolean> {
		// Use backend command to efficiently check if file is SMS backup
		try {
			const result = await tauriApi.checkIsSmsFile(filePath);
			return result;
		} catch (error) {
			console.error('Error checking if SMS file:', error);
			return false;
		}
	}

	async function executeSmsImport() {
		if (!selectedFile) return;

		try {
			importing = true;
			error = '';

			const result = await tauriApi.importSmsFile(selectedFile);

			importedCount = result.contacts_created + result.contacts_found;
			importComplete = true;

			const message = `✅ SMS Import Complete!\n\n` +
				`📱 Phone numbers processed: ${result.total_phone_numbers}\n` +
				`✨ Contacts created: ${result.contacts_created}\n` +
				`🔗 Existing contacts found: ${result.contacts_found}\n` +
				`💬 SMS messages imported: ${result.messages_imported}` +
				(result.failed > 0 ? `\n⚠️  Failed: ${result.failed}` : '');

			await tauriApi.showNotification('SMS Import Complete', message);
		} catch (err: any) {
			console.error('SMS import failed:', err);
			error = err.message || 'Failed to import SMS messages.';
			importComplete = false;
		} finally {
			importing = false;
		}
	}

	async function loadPreview() {
		if (!selectedFile) return;

		try {
			loadingPreview = true;
			error = '';

			preview = await tauriApi.previewImport(selectedFile, fieldMappings);
		} catch (err: any) {
			console.error('Preview failed:', err);
			error = err.message || 'Failed to load preview.';
		} finally {
			loadingPreview = false;
		}
	}

	async function handleFieldMappingChange() {
		// Reload preview when mappings change
		await loadPreview();
	}

	async function executeImport() {
		if (!selectedFile) return;

		// Validate that first_name is mapped
		const hasFirstName = Object.values(fieldMappings).includes('first_name');
		if (!hasFirstName) {
			error = 'You must map at least one field to "First Name" to import contacts.';
			return;
		}

		try {
			importing = true;
			error = '';
			currentStep = 3;

			importedCount = await tauriApi.executeImport(selectedFile, fieldMappings);
			importComplete = true;

			await tauriApi.showNotification(
				'Import Complete',
				`Successfully imported ${importedCount} contacts`
			);
		} catch (err: any) {
			console.error('Import failed:', err);
			error = err.message || 'Failed to import contacts.';
			importComplete = false;
		} finally {
			importing = false;
		}
	}

	function resetWizard() {
		currentStep = 1;
		selectedFile = '';
		analysis = null;
		fieldMappings = {};
		preview = null;
		importedCount = 0;
		importComplete = false;
		error = '';
	}

	function getFileName(path: string): string {
		return path.split('/').pop() || path.split('\\').pop() || path;
	}

	function getConfidenceColor(confidence: number): string {
		if (confidence >= 0.8) return 'var(--color-success-600)';
		if (confidence >= 0.5) return 'var(--color-warning-600)';
		return 'var(--color-error-600)';
	}
</script>

<div class="page">
	<div class="header">
		<h1>Smart Import - AI-Powered</h1>
		<p class="subtitle">
			Import contacts from CSV, JSON, XML, HTML, or text files with automatic field detection
		</p>
	</div>

	<!-- Progress indicator -->
	<div class="progress-steps">
		<div class="step" class:active={currentStep === 1} class:completed={currentStep > 1}>
			<div class="step-number">1</div>
			<div class="step-label">Select & Analyze</div>
		</div>
		<div class="step-line" class:completed={currentStep > 1}></div>
		<div class="step" class:active={currentStep === 2} class:completed={currentStep > 2}>
			<div class="step-number">2</div>
			<div class="step-label">Review Mappings</div>
		</div>
		<div class="step-line" class:completed={currentStep > 2}></div>
		<div class="step" class:active={currentStep === 3} class:completed={importComplete}>
			<div class="step-number">3</div>
			<div class="step-label">Import</div>
		</div>
	</div>

	{#if error}
		<div class="error-banner card">⚠️ {error}</div>
	{/if}

	<!-- Step 1: File Selection & Analysis -->
	{#if currentStep === 1}
		<div class="step-content card">
			<h2>Step 1: Select File</h2>
			<p>Choose a file to import. We'll automatically detect the format and suggest field mappings.</p>

			{#if !selectedFile}
				<div class="file-drop-zone">
					<div class="icon">📁</div>
					<p>Click below to select a file</p>
					<button on:click={handleSelectFile} class="btn btn-primary btn-large">
						📥 Select Import File
					</button>
					<div class="supported-formats">
						<strong>Supported formats:</strong> CSV, JSON, XML, HTML, TXT
					</div>
				</div>
			{:else if analyzing}
				<div class="analyzing">
					<div class="spinner"></div>
					<p>Analyzing {getFileName(selectedFile)}...</p>
					<p class="hint">Using AI to detect format and fields</p>
				</div>
			{/if}
		</div>
	{/if}

	<!-- Step 2: Field Mapping & Preview -->
	{#if currentStep === 2 && analysis}
		<div class="step-content card">
			<h2>Step 2: Review Field Mappings</h2>

			<!-- Analysis summary -->
			<div class="analysis-summary">
				<div class="summary-row">
					<span class="label">File:</span>
					<span class="value">{getFileName(selectedFile)}</span>
				</div>
				<div class="summary-row">
					<span class="label">Detected Format:</span>
					<span class="value format-badge">{analysis.detected_format.toUpperCase()}</span>
				</div>
				<div class="summary-row">
					<span class="label">Confidence:</span>
					<span class="value" style="color: {getConfidenceColor(analysis.confidence)}">
						{Math.round(analysis.confidence * 100)}%
					</span>
				</div>
				{#if preview}
					<div class="summary-row">
						<span class="label">Total Records:</span>
						<span class="value">{preview.total_records}</span>
					</div>
				{/if}
			</div>

			{#if analysis.warnings && analysis.warnings.length > 0}
				<div class="warnings">
					<strong>⚠️ Warnings:</strong>
					<ul>
						{#each analysis.warnings as warning}
							<li>{warning}</li>
						{/each}
					</ul>
				</div>
			{/if}

			<!-- Field mapping table -->
			<div class="mapping-section">
				<h3>Field Mappings</h3>
				<p class="hint">Map detected fields to contact fields. Fields marked with * are recommended.</p>

				<div class="mapping-table">
					<div class="mapping-header">
						<div>Source Field</div>
						<div>→</div>
						<div>Contact Field</div>
					</div>

					{#each analysis.detected_fields as sourceField}
						<div class="mapping-row">
							<div class="source-field">
								<code>{sourceField}</code>
							</div>
							<div class="arrow">→</div>
							<div class="target-field">
								<select
									bind:value={fieldMappings[sourceField]}
									on:change={handleFieldMappingChange}
									class="field-select"
								>
									{#each contactFields as option}
										<option value={option.value}>{option.label}</option>
									{/each}
								</select>
							</div>
						</div>
					{/each}
				</div>
			</div>

			<!-- Preview -->
			{#if loadingPreview}
				<div class="preview-loading">
					<div class="spinner"></div>
					<p>Loading preview...</p>
				</div>
			{:else if preview && preview.sample_records.length > 0}
				<div class="preview-section">
					<h3>Preview (first {preview.sample_records.length} records)</h3>

					<div class="preview-table-container">
						<table class="preview-table">
							<thead>
								<tr>
									{#each Object.values(fieldMappings).filter((v) => v) as field}
										<th>{field.replace('_', ' ')}</th>
									{/each}
								</tr>
							</thead>
							<tbody>
								{#each preview.sample_records.slice(0, 5) as record}
									<tr>
										{#each Object.entries(fieldMappings).filter(([_, v]) => v) as [sourceField, targetField]}
											<td>{record[sourceField] || '-'}</td>
										{/each}
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				</div>
			{/if}

			<div class="step-actions">
				<button on:click={resetWizard} class="btn btn-secondary">← Start Over</button>
				<button on:click={executeImport} class="btn btn-primary" disabled={loadingPreview}>
					Import {preview?.total_records || 0} Contacts →
				</button>
			</div>
		</div>
	{/if}

	<!-- Step 3: Import Progress & Results -->
	{#if currentStep === 3}
		<div class="step-content card">
			{#if importing}
				<div class="importing">
					<div class="spinner large"></div>
					<h2>Importing contacts...</h2>
					<p>Please wait while we import your contacts.</p>
				</div>
			{:else if importComplete}
				<div class="import-complete">
					<div class="success-icon">✅</div>
					<h2>Import Complete!</h2>
					<p class="success-message">
						Successfully imported <strong>{importedCount}</strong>
						{importedCount === 1 ? 'contact' : 'contacts'}
					</p>

					<div class="complete-actions">
						<a href="/contacts" class="btn btn-primary">View Contacts</a>
						<button on:click={resetWizard} class="btn btn-secondary">Import More</button>
					</div>
				</div>
			{/if}
		</div>
	{/if}

	<!-- Help section -->
	<div class="help-section card">
		<h3>💡 Smart Import Features</h3>
		<ul>
			<li>
				<strong>AI-Powered Detection:</strong> Automatically detects file format (CSV, JSON, XML, HTML,
				text)
			</li>
			<li>
				<strong>Smart Field Mapping:</strong> Suggests optimal mappings from your data to contact fields
			</li>
			<li><strong>Preview Before Import:</strong> Review sample records before committing</li>
			<li><strong>Multiple Formats:</strong> Support for nested JSON, XML attributes, HTML tables</li>
			<li>
				<strong>Flexible Mapping:</strong> Easily adjust field mappings to match your data structure
			</li>
		</ul>
	</div>
</div>

<style>
	.page {
		padding: 2rem;
		max-width: 1200px;
		margin: 0 auto;
	}

	.header {
		margin-bottom: 2rem;
		text-align: center;
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

	/* Progress Steps */
	.progress-steps {
		display: flex;
		align-items: center;
		justify-content: center;
		margin-bottom: 2rem;
		padding: 2rem 0;
	}

	.step {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
	}

	.step-number {
		width: 3rem;
		height: 3rem;
		border-radius: 50%;
		background: var(--gray-200);
		color: var(--gray-600);
		display: flex;
		align-items: center;
		justify-content: center;
		font-weight: 600;
		font-size: 1.25rem;
		transition: all 0.3s;
	}

	.step.active .step-number {
		background: var(--primary);
		color: white;
		transform: scale(1.1);
	}

	.step.completed .step-number {
		background: var(--color-success-500);
		color: white;
	}

	.step-label {
		font-size: 0.875rem;
		color: var(--gray-600);
		font-weight: 500;
	}

	.step.active .step-label {
		color: var(--primary);
		font-weight: 600;
	}

	.step-line {
		width: 6rem;
		height: 2px;
		background: var(--gray-200);
		margin: 0 1rem;
		transition: all 0.3s;
	}

	.step-line.completed {
		background: var(--color-success-500);
	}

	/* Error Banner */
	.error-banner {
		padding: 1rem 1.5rem;
		margin-bottom: 1.5rem;
		background: var(--color-error-50);
		color: var(--color-error-700);
		border-left: 4px solid var(--color-error-500);
	}

	/* Step Content */
	.step-content {
		padding: 2rem;
		margin-bottom: 2rem;
	}

	.step-content h2 {
		margin: 0 0 0.75rem;
		font-size: 1.5rem;
		color: var(--gray-900);
	}

	.step-content > p {
		margin: 0 0 2rem;
		color: var(--gray-600);
	}

	/* File Drop Zone */
	.file-drop-zone {
		text-align: center;
		padding: 3rem;
		border: 2px dashed var(--gray-300);
		border-radius: 8px;
		background: var(--gray-50);
	}

	.file-drop-zone .icon {
		font-size: 4rem;
		margin-bottom: 1rem;
	}

	.file-drop-zone p {
		margin: 0 0 1.5rem;
		color: var(--gray-600);
		font-size: 1.125rem;
	}

	.supported-formats {
		margin-top: 1.5rem;
		padding: 1rem;
		background: white;
		border-radius: 6px;
		font-size: 0.875rem;
		color: var(--gray-700);
	}

	/* Analyzing */
	.analyzing {
		text-align: center;
		padding: 3rem;
	}

	.analyzing p {
		margin: 1rem 0 0;
		font-size: 1.125rem;
		color: var(--gray-700);
	}

	.analyzing .hint {
		font-size: 0.875rem;
		color: var(--gray-500);
	}

	/* Analysis Summary */
	.analysis-summary {
		background: var(--gray-50);
		padding: 1.5rem;
		border-radius: 6px;
		margin-bottom: 2rem;
	}

	.summary-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.5rem 0;
		border-bottom: 1px solid var(--gray-200);
	}

	.summary-row:last-child {
		border-bottom: none;
	}

	.summary-row .label {
		font-weight: 500;
		color: var(--gray-700);
	}

	.summary-row .value {
		font-weight: 600;
		color: var(--gray-900);
	}

	.format-badge {
		padding: 0.25rem 0.75rem;
		background: var(--primary-light);
		color: var(--primary);
		border-radius: 4px;
		font-size: 0.875rem;
	}

	/* Warnings */
	.warnings {
		padding: 1rem 1.5rem;
		margin-bottom: 2rem;
		background: var(--color-warning-50);
		border-left: 4px solid var(--color-warning-500);
		border-radius: 6px;
	}

	.warnings strong {
		color: var(--color-warning-700);
	}

	.warnings ul {
		margin: 0.5rem 0 0;
		padding-left: 1.5rem;
		color: var(--color-warning-700);
	}

	/* Mapping Section */
	.mapping-section {
		margin-bottom: 2rem;
	}

	.mapping-section h3 {
		margin: 0 0 0.5rem;
		font-size: 1.25rem;
		color: var(--gray-900);
	}

	.mapping-section .hint {
		margin: 0 0 1.5rem;
		font-size: 0.875rem;
		color: var(--gray-600);
	}

	.mapping-table {
		border: 1px solid var(--gray-200);
		border-radius: 6px;
		overflow: hidden;
	}

	.mapping-header,
	.mapping-row {
		display: grid;
		grid-template-columns: 1fr auto 1fr;
		gap: 1rem;
		align-items: center;
		padding: 1rem;
	}

	.mapping-header {
		background: var(--gray-100);
		font-weight: 600;
		color: var(--gray-700);
		text-align: center;
	}

	.mapping-row {
		border-top: 1px solid var(--gray-200);
	}

	.mapping-row:hover {
		background: var(--gray-50);
	}

	.source-field code {
		padding: 0.25rem 0.5rem;
		background: var(--gray-100);
		border-radius: 4px;
		font-family: var(--font-mono);
		font-size: 0.875rem;
		color: var(--gray-800);
	}

	.arrow {
		text-align: center;
		color: var(--gray-400);
		font-size: 1.25rem;
	}

	.field-select {
		width: 100%;
		padding: 0.5rem;
		border: 1px solid var(--gray-300);
		border-radius: 4px;
		font-size: 0.875rem;
		background: white;
	}

	.field-select:focus {
		outline: none;
		border-color: var(--primary);
		box-shadow: 0 0 0 3px var(--primary-light);
	}

	/* Preview Section */
	.preview-section {
		margin-top: 2rem;
	}

	.preview-section h3 {
		margin: 0 0 1rem;
		font-size: 1.25rem;
		color: var(--gray-900);
	}

	.preview-table-container {
		overflow-x: auto;
		border: 1px solid var(--gray-200);
		border-radius: 6px;
	}

	.preview-table {
		width: 100%;
		border-collapse: collapse;
		font-size: 0.875rem;
	}

	.preview-table th {
		background: var(--gray-100);
		padding: 0.75rem;
		text-align: left;
		font-weight: 600;
		color: var(--gray-700);
		border-bottom: 2px solid var(--gray-300);
		text-transform: capitalize;
	}

	.preview-table td {
		padding: 0.75rem;
		border-bottom: 1px solid var(--gray-200);
		color: var(--gray-700);
	}

	.preview-table tbody tr:hover {
		background: var(--gray-50);
	}

	.preview-loading {
		text-align: center;
		padding: 2rem;
	}

	.preview-loading p {
		margin: 1rem 0 0;
		color: var(--gray-600);
	}

	/* Import Progress */
	.importing {
		text-align: center;
		padding: 4rem 2rem;
	}

	.importing h2 {
		margin: 2rem 0 0.5rem;
		color: var(--gray-900);
	}

	.importing p {
		margin: 0;
		color: var(--gray-600);
	}

	/* Import Complete */
	.import-complete {
		text-align: center;
		padding: 3rem 2rem;
	}

	.success-icon {
		font-size: 5rem;
		margin-bottom: 1rem;
	}

	.import-complete h2 {
		margin: 0 0 1rem;
		color: var(--gray-900);
	}

	.success-message {
		font-size: 1.25rem;
		color: var(--gray-700);
		margin: 0 0 2rem;
	}

	.success-message strong {
		color: var(--color-success-700);
		font-size: 1.5rem;
	}

	.complete-actions {
		display: flex;
		gap: 1rem;
		justify-content: center;
	}

	/* Step Actions */
	.step-actions {
		display: flex;
		justify-content: space-between;
		margin-top: 2rem;
		padding-top: 2rem;
		border-top: 1px solid var(--gray-200);
	}

	/* Help Section */
	.help-section {
		padding: 1.5rem;
		margin-top: 2rem;
	}

	.help-section h3 {
		margin: 0 0 1rem;
		font-size: 1.125rem;
		color: var(--gray-900);
	}

	.help-section ul {
		margin: 0;
		padding-left: 1.5rem;
		color: var(--gray-700);
	}

	.help-section li {
		margin-bottom: 0.75rem;
		line-height: 1.6;
	}

	/* Common Elements */
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
		text-decoration: none;
		display: inline-block;
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-large {
		padding: 1rem 2rem;
		font-size: 1.125rem;
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

	/* Spinner */
	.spinner {
		width: 3rem;
		height: 3rem;
		border: 4px solid var(--gray-200);
		border-top-color: var(--primary);
		border-radius: 50%;
		animation: spin 1s linear infinite;
		margin: 0 auto;
	}

	.spinner.large {
		width: 4rem;
		height: 4rem;
		border-width: 5px;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>
