<script lang="ts">
	import { tauriApi } from '$lib/api/tauri-api';

	let importing = false;
	let importedCount = 0;
	let error = '';

	async function handleImportCsv() {
		error = '';
		importing = true;

		try {
			importedCount = await tauriApi.importCsvDialog();

			if (importedCount > 0) {
				await tauriApi.showNotification(
					'Import Complete',
					`Successfully imported ${importedCount} contacts`
				);
			}
		} catch (err: any) {
			console.error('Import failed:', err);
			error = err.message || 'Failed to import contacts';
		} finally {
			importing = false;
		}
	}

	async function handleImportVCard() {
		error = 'vCard import is coming soon in beta release';
	}

	async function handleImportSms() {
		error = 'SMS import is coming soon in beta release';
	}
</script>

<div class="page">
	<div class="header">
		<h1>Import Contacts</h1>
		<p class="subtitle">Import contacts from CSV, vCard, or SMS backups</p>
	</div>

	{#if error}
		<div class="error-banner card">
			⚠️ {error}
		</div>
	{/if}

	{#if importedCount > 0}
		<div class="success-banner card">
			✅ Successfully imported {importedCount} contacts!
			<a href="/contacts" class="btn btn-primary">View Contacts</a>
		</div>
	{/if}

	<div class="import-grid">
		<div class="import-card card">
			<div class="icon">📄</div>
			<h2>CSV File</h2>
			<p>Import contacts from a CSV file with headers for first_name, last_name, email, phone, etc.</p>

			<button
				on:click={handleImportCsv}
				class="btn btn-primary btn-block"
				disabled={importing}
			>
				{importing ? 'Importing...' : '📥 Select CSV File'}
			</button>

			<div class="format-hint">
				<strong>Expected format:</strong>
				<code>first_name,last_name,email,phone,organization,title</code>
			</div>
		</div>

		<div class="import-card card disabled">
			<div class="icon">📇</div>
			<h2>vCard (.vcf)</h2>
			<p>Import contacts from vCard files exported from other contact managers.</p>

			<button
				on:click={handleImportVCard}
				class="btn btn-secondary btn-block"
				disabled
			>
				📥 Select vCard File (Coming Soon)
			</button>

			<div class="format-hint">
				<strong>Supports:</strong> vCard 3.0 and 4.0 formats
			</div>
		</div>

		<div class="import-card card disabled">
			<div class="icon">💬</div>
			<h2>SMS Backup</h2>
			<p>Import contacts from Android SMS backup files (XML format).</p>

			<button
				on:click={handleImportSms}
				class="btn btn-secondary btn-block"
				disabled
			>
				📥 Select SMS Backup (Coming Soon)
			</button>

			<div class="format-hint">
				<strong>Supports:</strong> SMS Backup & Restore XML format
			</div>
		</div>
	</div>

	<div class="help-section card">
		<h3>💡 Import Tips</h3>
		<ul>
			<li>CSV files should have headers in the first row</li>
			<li>Supported fields: first_name, last_name, email, phone, organization, title, notes</li>
			<li>Make sure your file is UTF-8 encoded to avoid character issues</li>
			<li>Duplicate contacts (by email/phone) will be skipped</li>
			<li>After import, you can find your contacts in the Contacts page</li>
		</ul>
	</div>

	<div class="sample-data-section card">
		<h3>📊 Need Sample Data?</h3>
		<p>Sample CSV files are available in the <code>alpha/sample_data/</code> directory:</p>
		<ul>
			<li><code>contacts.csv</code> - Basic contact information</li>
			<li><code>contacts_full.csv</code> - Complete contact details</li>
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

	.error-banner {
		padding: 1rem 1.5rem;
		margin-bottom: 1.5rem;
		background: var(--color-error-50);
		color: var(--color-error-700);
		border-left: 4px solid var(--color-error-500);
	}

	.success-banner {
		padding: 1.5rem;
		margin-bottom: 1.5rem;
		background: var(--color-success-50);
		color: var(--color-success-700);
		border-left: 4px solid var(--color-success-500);
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.import-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
		gap: 1.5rem;
		margin-bottom: 2rem;
	}

	.import-card {
		padding: 2rem;
		text-align: center;
		transition: transform 0.2s;
	}

	.import-card:not(.disabled):hover {
		transform: translateY(-2px);
		box-shadow: var(--shadow-md);
	}

	.import-card.disabled {
		opacity: 0.6;
	}

	.icon {
		font-size: 3rem;
		margin-bottom: 1rem;
	}

	.import-card h2 {
		margin: 0 0 0.75rem;
		font-size: 1.5rem;
		color: var(--gray-900);
	}

	.import-card p {
		margin: 0 0 1.5rem;
		color: var(--gray-600);
		font-size: 0.875rem;
		line-height: 1.5;
	}

	.format-hint {
		margin-top: 1rem;
		padding: 0.75rem;
		background: var(--gray-50);
		border-radius: 6px;
		font-size: 0.75rem;
		text-align: left;
	}

	.format-hint strong {
		display: block;
		margin-bottom: 0.25rem;
		color: var(--gray-700);
	}

	.format-hint code {
		display: block;
		font-family: var(--font-mono);
		color: var(--gray-600);
		word-break: break-all;
	}

	.help-section,
	.sample-data-section {
		padding: 1.5rem;
		margin-bottom: 1.5rem;
	}

	.help-section h3,
	.sample-data-section h3 {
		margin: 0 0 1rem;
		font-size: 1.125rem;
		color: var(--gray-900);
	}

	.help-section ul,
	.sample-data-section ul {
		margin: 0;
		padding-left: 1.5rem;
		color: var(--gray-700);
	}

	.help-section li,
	.sample-data-section li {
		margin-bottom: 0.5rem;
		line-height: 1.6;
	}

	.sample-data-section p {
		margin: 0 0 0.75rem;
		color: var(--gray-700);
	}

	.sample-data-section code {
		padding: 0.125rem 0.375rem;
		background: var(--gray-100);
		border-radius: 3px;
		font-family: var(--font-mono);
		font-size: 0.875rem;
		color: var(--gray-800);
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

	.btn-block {
		width: 100%;
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
</style>
