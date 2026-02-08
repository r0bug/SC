<script lang="ts">
	import { api, ApiError } from '$lib/api/api';
	import { toasts } from '$lib/stores/toast';

	type FileResult = {
		name: string;
		size: number;
		file_type: string;
		total_records: number;
		imported: number;
		skipped: number;
		elapsed_seconds: number;
		status: 'pending' | 'uploading' | 'done' | 'error';
		error?: string;
	};

	type DirectoryResult = {
		path: string;
		total_files: number;
		files_imported: number;
		files_failed: number;
		total_records: number;
		total_imported: number;
		total_skipped: number;
		elapsed_seconds: number;
		results: Array<{
			file_name: string;
			file_type: string;
			total_records: number;
			imported: number;
			skipped: number;
			status: string;
			error: string | null;
		}>;
	};

	let files: File[] = [];
	let results: FileResult[] = [];
	let importing = false;
	let dragActive = false;
	let currentIndex = -1;

	// Directory import
	let dirPath = '';
	let dirImporting = false;
	let dirResult: DirectoryResult | null = null;

	// Totals across all files
	$: totals = results.reduce(
		(acc, r) => ({
			total_records: acc.total_records + r.total_records,
			imported: acc.imported + r.imported,
			skipped: acc.skipped + r.skipped,
			elapsed_seconds: acc.elapsed_seconds + r.elapsed_seconds,
			done: acc.done + (r.status === 'done' ? 1 : 0),
			errors: acc.errors + (r.status === 'error' ? 1 : 0)
		}),
		{ total_records: 0, imported: 0, skipped: 0, elapsed_seconds: 0, done: 0, errors: 0 }
	);

	$: allDone = results.length > 0 && results.every(r => r.status === 'done' || r.status === 'error');

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
		const dropped = e.dataTransfer?.files;
		if (dropped) {
			addFiles(dropped);
		}
	}

	function handleFileSelect(event: Event) {
		const target = event.target as HTMLInputElement;
		if (target.files && target.files.length > 0) {
			addFiles(target.files);
			target.value = ''; // Reset so same files can be re-selected
		}
	}

	function addFiles(fileList: FileList) {
		const newFiles: File[] = [];
		for (let i = 0; i < fileList.length; i++) {
			const f = fileList[i];
			// Only accept XML files, skip duplicates by name
			if (f.name.toLowerCase().endsWith('.xml') && !files.some(existing => existing.name === f.name)) {
				newFiles.push(f);
			}
		}
		if (newFiles.length === 0 && fileList.length > 0) {
			toasts.warning('No new XML files to add. Non-XML files are skipped.', 'Skipped');
			return;
		}
		files = [...files, ...newFiles];
	}

	function removeFile(index: number) {
		files = files.filter((_, i) => i !== index);
	}

	function clearFiles() {
		files = [];
		results = [];
		currentIndex = -1;
	}

	async function uploadAll() {
		if (files.length === 0) return;

		// Validate sizes
		const MAX_SIZE = 500 * 1024 * 1024;
		const oversized = files.filter(f => f.size > MAX_SIZE);
		if (oversized.length > 0) {
			toasts.error(
				`${oversized.length} file(s) exceed 500MB limit: ${oversized.map(f => f.name).join(', ')}`,
				'Files Too Large'
			);
			return;
		}

		importing = true;
		results = files.map(f => ({
			name: f.name,
			size: f.size,
			file_type: '',
			total_records: 0,
			imported: 0,
			skipped: 0,
			elapsed_seconds: 0,
			status: 'pending' as const
		}));

		for (let i = 0; i < files.length; i++) {
			currentIndex = i;
			results[i].status = 'uploading';
			results = results; // trigger reactivity

			try {
				const res = await api.importAndroidSms(files[i]);
				results[i] = {
					...results[i],
					file_type: res.file_type,
					total_records: res.total_records,
					imported: res.imported,
					skipped: res.skipped,
					elapsed_seconds: res.elapsed_seconds,
					status: 'done'
				};
			} catch (error) {
				const msg = error instanceof ApiError ? error.getUserMessage() : 'Import failed';
				results[i] = { ...results[i], status: 'error', error: msg };
			}
			results = results; // trigger reactivity
		}

		currentIndex = -1;
		importing = false;

		const doneCount = results.filter(r => r.status === 'done').length;
		const errorCount = results.filter(r => r.status === 'error').length;
		if (errorCount === 0) {
			toasts.success(
				`All ${doneCount} files imported: ${totals.imported.toLocaleString()} messages total`,
				'Import Complete'
			);
		} else {
			toasts.warning(
				`${doneCount} succeeded, ${errorCount} failed. ${totals.imported.toLocaleString()} messages imported.`,
				'Import Finished'
			);
		}
	}

	function formatFileSize(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	}

	async function importFromDirectory() {
		if (!dirPath.trim()) return;
		dirImporting = true;
		dirResult = null;

		try {
			const res = await api.importSmsDirectory(dirPath.trim());
			dirResult = res;

			if (res.files_failed === 0) {
				toasts.success(
					`Imported ${res.total_imported.toLocaleString()} messages from ${res.files_imported} files`,
					'Directory Import Complete'
				);
			} else {
				toasts.warning(
					`${res.files_imported} succeeded, ${res.files_failed} failed. ${res.total_imported.toLocaleString()} messages imported.`,
					'Directory Import Finished'
				);
			}
		} catch (error) {
			const msg = error instanceof ApiError ? error.getUserMessage() : 'Directory import failed';
			toasts.error(msg, 'Import Error');
		}

		dirImporting = false;
	}

	function clearDirResult() {
		dirResult = null;
		dirPath = '';
	}
</script>

<div class="page">
	<div class="header">
		<div>
			<h1>SMS Import</h1>
			<p class="subtitle">Import SMS and MMS messages from Android Backup &amp; Restore XML files</p>
		</div>
		<div class="header-actions">
			<a href="/sms-history" class="btn btn-secondary">
				📱 View SMS History
			</a>
		</div>
	</div>

	{#if allDone}
		<!-- Final results summary -->
		<div class="card">
			<h2>Import Complete - {results.length} Files</h2>
			<div class="result-grid">
				<div class="result-item">
					<span class="result-label">Files Processed</span>
					<span class="result-value">{totals.done} / {results.length}</span>
				</div>
				<div class="result-item">
					<span class="result-label">Total Records</span>
					<span class="result-value">{totals.total_records.toLocaleString()}</span>
				</div>
				<div class="result-item">
					<span class="result-label">Imported</span>
					<span class="result-value success">{totals.imported.toLocaleString()}</span>
				</div>
				<div class="result-item">
					<span class="result-label">Skipped</span>
					<span class="result-value warning">{totals.skipped.toLocaleString()}</span>
				</div>
				{#if totals.errors > 0}
					<div class="result-item">
						<span class="result-label">Failed Files</span>
						<span class="result-value error">{totals.errors}</span>
					</div>
				{/if}
				<div class="result-item">
					<span class="result-label">Total Time</span>
					<span class="result-value">{totals.elapsed_seconds.toFixed(1)}s</span>
				</div>
			</div>

			<!-- Per-file breakdown -->
			<h3>File Details</h3>
			<div class="file-results">
				{#each results as r}
					<div class="file-result-row" class:error={r.status === 'error'}>
						<span class="file-result-name">
							{r.status === 'done' ? '✅' : '❌'} {r.name}
						</span>
						{#if r.status === 'done'}
							<span class="file-result-stats">
								{r.imported.toLocaleString()} imported, {r.skipped.toLocaleString()} skipped ({r.elapsed_seconds.toFixed(1)}s)
							</span>
						{:else}
							<span class="file-result-error">{r.error}</span>
						{/if}
					</div>
				{/each}
			</div>

			<div class="actions">
				<button on:click={clearFiles} class="btn btn-secondary">Import More Files</button>
				<a href="/sms-history" class="btn btn-primary">Browse Messages</a>
			</div>
		</div>
	{:else}
		<!-- Upload area -->
		<div class="card">
			<h2>Upload SMS Backup Files</h2>
			<div
				class="upload-area"
				class:drag-active={dragActive}
				role="region"
				aria-label="File upload area"
				on:dragover={handleDragOver}
				on:dragleave={handleDragLeave}
				on:drop={handleDrop}
			>
				<input
					type="file"
					id="sms-file-input"
					accept=".xml"
					multiple
					on:change={handleFileSelect}
					style="display: none;"
				/>
				<label for="sms-file-input" class="upload-label">
					<div class="upload-prompt">
						<div class="upload-icon">💬</div>
						<p><strong>Drag and drop XML files here</strong></p>
						<p class="text-sm">or click to browse - select multiple files at once</p>
						<p class="text-xs">Supports Android SMS Backup &amp; Restore XML files (SMS and MMS)</p>
					</div>
				</label>
			</div>

			<!-- File list -->
			{#if files.length > 0}
				<div class="file-list">
					<div class="file-list-header">
						<span>{files.length} file{files.length === 1 ? '' : 's'} selected ({formatFileSize(files.reduce((s, f) => s + f.size, 0))} total)</span>
						{#if !importing}
							<button class="btn-text" on:click={clearFiles}>Clear all</button>
						{/if}
					</div>
					{#each files as f, i}
						<div class="file-row" class:uploading={importing && currentIndex === i} class:done={results[i]?.status === 'done'} class:error={results[i]?.status === 'error'}>
							<span class="file-status">
								{#if results[i]?.status === 'done'}
									✅
								{:else if results[i]?.status === 'error'}
									❌
								{:else if results[i]?.status === 'uploading'}
									<span class="spinner-sm"></span>
								{:else}
									📄
								{/if}
							</span>
							<span class="file-name">{f.name}</span>
							<span class="file-size">{formatFileSize(f.size)}</span>
							{#if results[i]?.status === 'done'}
								<span class="file-imported">{results[i].imported.toLocaleString()} msgs</span>
							{:else if results[i]?.status === 'error'}
								<span class="file-error">{results[i].error}</span>
							{:else if !importing}
								<button class="btn-icon-sm" on:click={() => removeFile(i)} title="Remove">✕</button>
							{/if}
						</div>
					{/each}
				</div>

				{#if importing}
					<div class="progress-bar">
						<div class="progress-fill" style="width: {((results.filter(r => r.status === 'done' || r.status === 'error').length) / results.length * 100)}%"></div>
					</div>
					<p class="progress-text">
						Importing file {currentIndex + 1} of {files.length}...
					</p>
				{/if}

				{#if !importing && !allDone}
					<div class="actions">
						<button on:click={clearFiles} class="btn btn-secondary">Cancel</button>
						<button on:click={uploadAll} class="btn btn-primary">
							Upload &amp; Import {files.length} File{files.length === 1 ? '' : 's'}
						</button>
					</div>
				{/if}
			{/if}
		</div>

		<!-- Server directory import -->
		<div class="card">
			<h2>Import from Server Directory</h2>
			<p class="text-sm" style="margin-bottom: 1rem;">
				Enter a directory path on the server containing XML files. All .xml files in the directory will be imported.
			</p>

			{#if dirResult}
				<!-- Directory import results -->
				<div class="result-grid">
					<div class="result-item">
						<span class="result-label">Files Found</span>
						<span class="result-value">{dirResult.total_files}</span>
					</div>
					<div class="result-item">
						<span class="result-label">Imported</span>
						<span class="result-value success">{dirResult.files_imported}</span>
					</div>
					{#if dirResult.files_failed > 0}
						<div class="result-item">
							<span class="result-label">Failed</span>
							<span class="result-value error">{dirResult.files_failed}</span>
						</div>
					{/if}
					<div class="result-item">
						<span class="result-label">Messages</span>
						<span class="result-value success">{dirResult.total_imported.toLocaleString()}</span>
					</div>
					<div class="result-item">
						<span class="result-label">Time</span>
						<span class="result-value">{dirResult.elapsed_seconds.toFixed(1)}s</span>
					</div>
				</div>

				<h3>File Details</h3>
				<div class="file-results">
					{#each dirResult.results as r}
						<div class="file-result-row" class:error={r.status === 'error'}>
							<span class="file-result-name">
								{r.status === 'done' ? '✅' : '❌'} {r.file_name}
							</span>
							{#if r.status === 'done'}
								<span class="file-result-stats">
									{r.file_type} - {r.imported.toLocaleString()} imported, {r.skipped.toLocaleString()} skipped
								</span>
							{:else}
								<span class="file-result-error">{r.error}</span>
							{/if}
						</div>
					{/each}
				</div>

				<div class="actions">
					<button on:click={clearDirResult} class="btn btn-secondary">Import Another Directory</button>
					<a href="/sms-history" class="btn btn-primary">Browse Messages</a>
				</div>
			{:else}
				<div class="dir-input-row">
					<input
						type="text"
						bind:value={dirPath}
						placeholder="/path/to/xml/files"
						class="dir-input"
						disabled={dirImporting}
						on:keydown={(e) => e.key === 'Enter' && importFromDirectory()}
					/>
					<button
						on:click={importFromDirectory}
						class="btn btn-primary"
						disabled={dirImporting || !dirPath.trim()}
					>
						{#if dirImporting}
							<span class="spinner-sm"></span> Importing...
						{:else}
							Import Directory
						{/if}
					</button>
				</div>
			{/if}
		</div>

		<div class="card info-card">
			<h3>About Android SMS Backup</h3>
			<p>This importer supports XML files from the <strong>SMS Backup &amp; Restore</strong> app by SyncTech.</p>
			<ul>
				<li>Select or drag multiple XML files at once for batch import</li>
				<li>Enter a server directory path to import all XML files at once</li>
				<li>Both SMS and MMS messages are supported</li>
				<li>Messages are stored even if there's no matching contact</li>
				<li>Phone numbers are automatically matched to existing contacts when possible</li>
			</ul>
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
		padding: 1.5rem;
		margin-bottom: 1.5rem;
	}

	.card h2 {
		font-size: 1.25rem;
		font-weight: 600;
		margin: 0 0 1rem 0;
	}

	.card h3 {
		font-size: 1rem;
		font-weight: 600;
		margin: 1.5rem 0 0.75rem 0;
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

	/* File list */
	.file-list {
		margin-top: 1rem;
	}

	.file-list-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.5rem 0;
		font-size: 0.875rem;
		color: #6b7280;
		border-bottom: 1px solid #e5e7eb;
		margin-bottom: 0.25rem;
	}

	.file-row {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.5rem 0.25rem;
		border-bottom: 1px solid #f3f4f6;
		font-size: 0.875rem;
	}

	.file-row.uploading {
		background: #eff6ff;
	}

	.file-row.done {
		opacity: 0.7;
	}

	.file-row.error {
		background: #fef2f2;
	}

	.file-status {
		width: 1.5rem;
		text-align: center;
		flex-shrink: 0;
	}

	.file-name {
		flex: 1;
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.file-size {
		color: #6b7280;
		flex-shrink: 0;
	}

	.file-imported {
		color: #059669;
		font-weight: 500;
		flex-shrink: 0;
	}

	.file-error {
		color: #dc2626;
		font-size: 0.8rem;
		flex-shrink: 0;
		max-width: 200px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.btn-icon-sm {
		background: none;
		border: none;
		cursor: pointer;
		color: #9ca3af;
		font-size: 1rem;
		padding: 0.25rem;
		line-height: 1;
	}

	.btn-icon-sm:hover {
		color: #ef4444;
	}

	.btn-text {
		background: none;
		border: none;
		cursor: pointer;
		color: #3b82f6;
		font-size: 0.875rem;
		padding: 0;
	}

	.btn-text:hover {
		text-decoration: underline;
	}

	/* Progress bar */
	.progress-bar {
		height: 6px;
		background: #e5e7eb;
		border-radius: 3px;
		margin-top: 1rem;
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		background: #3b82f6;
		border-radius: 3px;
		transition: width 0.3s ease;
	}

	.progress-text {
		text-align: center;
		font-size: 0.875rem;
		color: #6b7280;
		margin-top: 0.5rem;
	}

	/* Spinner */
	.spinner-sm {
		display: inline-block;
		width: 1rem;
		height: 1rem;
		border: 2px solid #e5e7eb;
		border-top-color: #3b82f6;
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	/* Results */
	.result-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
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

	.file-results {
		margin-top: 0.5rem;
	}

	.file-result-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.5rem 0.25rem;
		border-bottom: 1px solid #f3f4f6;
		font-size: 0.875rem;
	}

	.file-result-row.error {
		background: #fef2f2;
	}

	.file-result-name {
		font-weight: 500;
	}

	.file-result-stats {
		color: #6b7280;
	}

	.file-result-error {
		color: #dc2626;
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
		text-decoration: none;
		display: inline-block;
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

	.info-card p {
		color: #4b5563;
		line-height: 1.6;
		margin: 0 0 0.75rem 0;
	}

	.info-card ul {
		margin: 0;
		padding-left: 1.25rem;
		color: #4b5563;
	}

	.info-card li {
		margin-bottom: 0.5rem;
		line-height: 1.5;
	}

	/* Directory import */
	.dir-input-row {
		display: flex;
		gap: 0.75rem;
		align-items: center;
	}

	.dir-input {
		flex: 1;
		padding: 0.625rem 0.75rem;
		font-size: 0.875rem;
		font-family: monospace;
		border: 1px solid #d1d5db;
		border-radius: 0.375rem;
		outline: none;
	}

	.dir-input:focus {
		border-color: #3b82f6;
		box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.15);
	}

	.dir-input:disabled {
		background: #f3f4f6;
	}

	.text-sm { font-size: 0.875rem; color: #6b7280; }
	.text-xs { font-size: 0.75rem; color: #6b7280; }

	@media (max-width: 768px) {
		.page { padding: 1rem; }
		.header { flex-direction: column; gap: 1rem; }
		.result-grid { grid-template-columns: repeat(2, 1fr); }
	}
</style>
