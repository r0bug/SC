<script lang="ts">
	import { api, ApiError } from '$lib/api/api';
	import { toasts } from '$lib/stores/toast';
	import { onMount } from 'svelte';

	type SavedAccount = {
		id: string;
		name: string;
		server: string;
		port: number;
		username: string;
		default_folder: string;
		max_emails: number;
		last_fetched_at: string | null;
		created_at: string;
		updated_at: string;
	};

	let server = 'imap.gmail.com';
	let port = 993;
	let username = '';
	let password = '';
	let folder = 'INBOX';
	let maxEmails = 100;
	let fetching = false;
	let saveAccount = false;
	let accountName = '';

	let savedAccounts: SavedAccount[] = [];
	let loadingAccounts = true;
	let fetchingAccountId: string | null = null;
	let editingAccount: SavedAccount | null = null;
	let editPassword = '';
	let deletingId: string | null = null;

	let result: {
		fetched: number;
		stored: number;
		skipped: number;
		server: string;
		folder: string;
	} | null = null;

	const folders = [
		{ value: 'INBOX', label: 'Inbox' },
		{ value: '[Gmail]/Sent Mail', label: 'Sent Mail' },
		{ value: '[Gmail]/All Mail', label: 'All Mail' },
		{ value: '[Gmail]/Drafts', label: 'Drafts' },
		{ value: '[Gmail]/Spam', label: 'Spam' },
		{ value: '[Gmail]/Trash', label: 'Trash' },
	];

	onMount(async () => {
		await loadAccounts();
	});

	async function loadAccounts() {
		try {
			loadingAccounts = true;
			const res = await api.getImapAccounts();
			savedAccounts = res.accounts;
		} catch {
			// Silently fail, user can still use manual form
		} finally {
			loadingAccounts = false;
		}
	}

	async function handleFetch() {
		if (!username || !password) {
			toasts.error('Username and App Password are required', 'Validation');
			return;
		}

		fetching = true;
		result = null;

		try {
			result = await api.fetchEmails({
				server,
				port,
				username,
				password,
				folder,
				max_emails: maxEmails,
			});
			toasts.success(
				`Fetched ${result.fetched} emails, stored ${result.stored} new`,
				'Email Import'
			);

			// Save account if requested
			if (saveAccount && accountName.trim()) {
				try {
					await api.createImapAccount({
						name: accountName.trim(),
						server,
						port,
						username,
						password,
						default_folder: folder,
						max_emails: maxEmails,
					});
					toasts.success('Account saved for future use', 'Saved');
					saveAccount = false;
					accountName = '';
					await loadAccounts();
				} catch {
					toasts.error('Failed to save account', 'Error');
				}
			}
		} catch (error) {
			console.error('Email fetch failed:', error);
			if (error instanceof ApiError) {
				toasts.error(error.getUserMessage(), 'Fetch Failed');
			} else {
				toasts.error('Failed to fetch emails from IMAP server', 'Error');
			}
		} finally {
			fetching = false;
		}
	}

	async function fetchFromSaved(account: SavedAccount) {
		fetchingAccountId = account.id;
		result = null;
		try {
			result = await api.fetchFromImapAccount(account.id);
			toasts.success(
				`Fetched ${result.fetched} emails, stored ${result.stored} new`,
				account.name
			);
			await loadAccounts(); // Refresh last_fetched_at
		} catch (error) {
			if (error instanceof ApiError) {
				toasts.error(error.getUserMessage(), 'Fetch Failed');
			} else {
				toasts.error('Failed to fetch emails', 'Error');
			}
		} finally {
			fetchingAccountId = null;
		}
	}

	async function deleteAccount(id: string) {
		deletingId = id;
		try {
			await api.deleteImapAccount(id);
			toasts.success('Account removed', 'Deleted');
			savedAccounts = savedAccounts.filter(a => a.id !== id);
		} catch {
			toasts.error('Failed to delete account', 'Error');
		} finally {
			deletingId = null;
		}
	}

	function startEdit(account: SavedAccount) {
		editingAccount = { ...account };
		editPassword = '';
	}

	async function saveEdit() {
		if (!editingAccount) return;
		try {
			const updates: any = {
				name: editingAccount.name,
				server: editingAccount.server,
				port: editingAccount.port,
				username: editingAccount.username,
				default_folder: editingAccount.default_folder,
				max_emails: editingAccount.max_emails,
			};
			if (editPassword) {
				updates.password = editPassword;
			}
			await api.updateImapAccount(editingAccount.id, updates);
			toasts.success('Account updated', 'Saved');
			editingAccount = null;
			editPassword = '';
			await loadAccounts();
		} catch {
			toasts.error('Failed to update account', 'Error');
		}
	}

	function formatLastFetched(iso: string | null): string {
		if (!iso) return 'Never';
		const d = new Date(iso);
		return d.toLocaleString();
	}
</script>

<div class="page">
	<div class="header">
		<div>
			<h1>Email Import</h1>
			<p class="subtitle">Import emails from your IMAP email account (Gmail, Outlook, etc.)</p>
		</div>
		<div class="header-actions">
			<a href="/email-history" class="btn btn-secondary">View Email History</a>
		</div>
	</div>

	<div class="content-grid">
		<!-- Saved Accounts -->
		{#if !loadingAccounts && savedAccounts.length > 0}
			<div class="config-card">
				<h2>Saved Accounts</h2>
				<p class="card-description">Click "Fetch Now" to import new emails with one click.</p>
				<div class="accounts-list">
					{#each savedAccounts as account (account.id)}
						{#if editingAccount && editingAccount.id === account.id}
							<div class="account-card editing">
								<div class="edit-form">
									<div class="form-row">
										<div class="form-group flex-2">
											<label>Name</label>
											<input type="text" bind:value={editingAccount.name} />
										</div>
										<div class="form-group flex-1">
											<label>Port</label>
											<input type="number" bind:value={editingAccount.port} />
										</div>
									</div>
									<div class="form-group">
										<label>Server</label>
										<input type="text" bind:value={editingAccount.server} />
									</div>
									<div class="form-group">
										<label>Username</label>
										<input type="text" bind:value={editingAccount.username} />
									</div>
									<div class="form-group">
										<label>New Password (leave blank to keep)</label>
										<input type="password" bind:value={editPassword} placeholder="unchanged" />
									</div>
									<div class="form-row">
										<div class="form-group flex-2">
											<label>Default Folder</label>
											<select bind:value={editingAccount.default_folder}>
												{#each folders as f}
													<option value={f.value}>{f.label}</option>
												{/each}
											</select>
										</div>
										<div class="form-group flex-1">
											<label>Max Emails</label>
											<input type="number" bind:value={editingAccount.max_emails} min="1" max="5000" />
										</div>
									</div>
									<div class="edit-actions">
										<button class="btn btn-primary btn-sm" on:click={saveEdit}>Save</button>
										<button class="btn btn-secondary btn-sm" on:click={() => editingAccount = null}>Cancel</button>
									</div>
								</div>
							</div>
						{:else}
							<div class="account-card">
								<div class="account-info">
									<div class="account-name">{account.name}</div>
									<div class="account-details">
										{account.username} @ {account.server}
										<span class="account-folder">{account.default_folder}</span>
									</div>
									<div class="account-fetched">
										Last fetched: {formatLastFetched(account.last_fetched_at)}
									</div>
								</div>
								<div class="account-actions">
									<button
										class="btn btn-primary btn-sm"
										on:click={() => fetchFromSaved(account)}
										disabled={fetchingAccountId === account.id}
									>
										{#if fetchingAccountId === account.id}
											<span class="spinner-inline"></span>
											Fetching...
										{:else}
											Fetch Now
										{/if}
									</button>
									<button class="btn btn-secondary btn-sm" on:click={() => startEdit(account)}>
										Edit
									</button>
									<button
										class="btn btn-danger btn-sm"
										on:click={() => deleteAccount(account.id)}
										disabled={deletingId === account.id}
									>
										Delete
									</button>
								</div>
							</div>
						{/if}
					{/each}
				</div>
			</div>
		{/if}

		<!-- Manual Connection Form -->
		<div class="config-card">
			<h2>IMAP Connection</h2>
			<p class="card-description">
				For Gmail, you need an <strong>App Password</strong> (not your regular password).
				Go to myaccount.google.com &rarr; Security &rarr; 2-Step Verification &rarr; App passwords.
			</p>

			<form on:submit|preventDefault={handleFetch}>
				<div class="form-row">
					<div class="form-group flex-2">
						<label for="server">IMAP Server</label>
						<input id="server" type="text" bind:value={server} placeholder="imap.gmail.com" />
					</div>
					<div class="form-group flex-1">
						<label for="port">Port</label>
						<input id="port" type="number" bind:value={port} />
					</div>
				</div>

				<div class="form-group">
					<label for="username">Email / Username</label>
					<input id="username" type="email" bind:value={username} placeholder="john@robug.com" />
				</div>

				<div class="form-group">
					<label for="password">App Password</label>
					<input id="password" type="password" bind:value={password} placeholder="xxxx xxxx xxxx xxxx" />
				</div>

				<div class="form-row">
					<div class="form-group flex-2">
						<label for="folder">Folder</label>
						<select id="folder" bind:value={folder}>
							{#each folders as f}
								<option value={f.value}>{f.label}</option>
							{/each}
						</select>
					</div>
					<div class="form-group flex-1">
						<label for="max">Max Emails</label>
						<input id="max" type="number" bind:value={maxEmails} min="1" max="5000" />
					</div>
				</div>

				<div class="save-option">
					<label class="checkbox-label">
						<input type="checkbox" bind:checked={saveAccount} />
						Save this account for future imports
					</label>
					{#if saveAccount}
						<div class="form-group" style="margin-top: 0.5rem;">
							<input
								type="text"
								bind:value={accountName}
								placeholder="Account name (e.g., Work Gmail)"
							/>
						</div>
					{/if}
				</div>

				<button type="submit" class="btn btn-primary btn-block" disabled={fetching}>
					{#if fetching}
						<span class="spinner-inline"></span>
						Fetching emails...
					{:else}
						Fetch Emails
					{/if}
				</button>
			</form>
		</div>

		{#if result}
			<div class="result-card">
				<h2>Import Results</h2>
				<div class="result-grid">
					<div class="result-stat">
						<span class="stat-value">{result.fetched}</span>
						<span class="stat-label">Fetched</span>
					</div>
					<div class="result-stat success">
						<span class="stat-value">{result.stored}</span>
						<span class="stat-label">New Stored</span>
					</div>
					<div class="result-stat muted">
						<span class="stat-value">{result.skipped}</span>
						<span class="stat-label">Duplicates Skipped</span>
					</div>
				</div>
				<div class="result-details">
					<p><strong>Server:</strong> {result.server}</p>
					<p><strong>Folder:</strong> {result.folder}</p>
				</div>
				<a href="/email-history" class="btn btn-primary btn-block" style="margin-top: 1rem;">
					View Email History
				</a>
			</div>
		{/if}
	</div>
</div>

<style>
	.page {
		padding: 2rem;
		max-width: 900px;
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
		margin: 0 0 0.25rem 0;
	}

	.subtitle {
		color: #666;
		margin: 0;
		font-size: 0.875rem;
	}

	.header-actions {
		display: flex;
		gap: 0.5rem;
	}

	.content-grid {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.config-card, .result-card {
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
		padding: 1.5rem;
	}

	.config-card h2, .result-card h2 {
		margin: 0 0 0.5rem 0;
		font-size: 1.25rem;
	}

	.card-description {
		color: #6b7280;
		font-size: 0.875rem;
		margin: 0 0 1.5rem 0;
		line-height: 1.5;
	}

	/* Saved Accounts */
	.accounts-list {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.account-card {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1rem;
		background: #f9fafb;
		border: 1px solid #e5e7eb;
		border-radius: 0.375rem;
		gap: 1rem;
	}

	.account-card.editing {
		display: block;
		background: #eff6ff;
		border-color: #93c5fd;
	}

	.account-info {
		flex: 1;
		min-width: 0;
	}

	.account-name {
		font-weight: 600;
		font-size: 0.9375rem;
		color: #111827;
	}

	.account-details {
		font-size: 0.8125rem;
		color: #6b7280;
		margin-top: 0.125rem;
	}

	.account-folder {
		background: #e5e7eb;
		padding: 0.0625rem 0.375rem;
		border-radius: 0.25rem;
		font-size: 0.75rem;
		margin-left: 0.375rem;
	}

	.account-fetched {
		font-size: 0.75rem;
		color: #9ca3af;
		margin-top: 0.25rem;
	}

	.account-actions {
		display: flex;
		gap: 0.375rem;
		flex-shrink: 0;
	}

	.edit-form {
		padding: 0.5rem 0;
	}

	.edit-actions {
		display: flex;
		gap: 0.5rem;
		margin-top: 0.75rem;
	}

	/* Save Account Option */
	.save-option {
		margin-bottom: 1rem;
		padding: 0.75rem;
		background: #f9fafb;
		border-radius: 0.375rem;
		border: 1px solid #e5e7eb;
	}

	.checkbox-label {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.875rem;
		color: #374151;
		cursor: pointer;
	}

	.checkbox-label input[type="checkbox"] {
		width: auto;
	}

	/* Form */
	.form-row {
		display: flex;
		gap: 1rem;
	}

	.flex-1 { flex: 1; }
	.flex-2 { flex: 2; }

	.form-group {
		margin-bottom: 1rem;
	}

	.form-group label {
		display: block;
		font-size: 0.875rem;
		font-weight: 500;
		margin-bottom: 0.375rem;
		color: #374151;
	}

	.form-group input,
	.form-group select {
		width: 100%;
		padding: 0.5rem 0.75rem;
		border: 1px solid #d1d5db;
		border-radius: 0.375rem;
		font-size: 0.875rem;
		box-sizing: border-box;
	}

	.form-group input:focus,
	.form-group select:focus {
		outline: none;
		border-color: #3b82f6;
		box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.15);
	}

	/* Buttons */
	.btn {
		padding: 0.625rem 1.25rem;
		font-size: 0.875rem;
		font-weight: 500;
		border-radius: 0.375rem;
		border: none;
		cursor: pointer;
		transition: all 0.2s;
		text-decoration: none;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 0.5rem;
	}

	.btn-sm {
		padding: 0.375rem 0.75rem;
		font-size: 0.8125rem;
	}

	.btn-primary {
		background: #3b82f6;
		color: white;
	}

	.btn-primary:hover:not(:disabled) {
		background: #2563eb;
	}

	.btn-primary:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.btn-secondary {
		background: #f3f4f6;
		color: #374151;
		border: 1px solid #d1d5db;
	}

	.btn-secondary:hover {
		background: #e5e7eb;
	}

	.btn-danger {
		background: #fef2f2;
		color: #dc2626;
		border: 1px solid #fecaca;
	}

	.btn-danger:hover:not(:disabled) {
		background: #fee2e2;
	}

	.btn-danger:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.btn-block {
		width: 100%;
		margin-top: 0.5rem;
	}

	/* Results */
	.result-grid {
		display: flex;
		gap: 1rem;
		margin: 1rem 0;
	}

	.result-stat {
		flex: 1;
		text-align: center;
		padding: 1rem;
		background: #f9fafb;
		border-radius: 0.375rem;
		border: 1px solid #e5e7eb;
	}

	.result-stat.success {
		background: #f0fdf4;
		border-color: #bbf7d0;
	}

	.result-stat.muted {
		background: #f9fafb;
		border-color: #e5e7eb;
	}

	.stat-value {
		display: block;
		font-size: 1.75rem;
		font-weight: 700;
		color: #111827;
	}

	.result-stat.success .stat-value { color: #16a34a; }
	.result-stat.muted .stat-value { color: #6b7280; }

	.stat-label {
		display: block;
		font-size: 0.75rem;
		color: #6b7280;
		margin-top: 0.25rem;
		text-transform: uppercase;
		font-weight: 500;
	}

	.result-details {
		font-size: 0.875rem;
		color: #374151;
	}

	.result-details p {
		margin: 0.375rem 0;
	}

	.spinner-inline {
		width: 1rem;
		height: 1rem;
		border: 2px solid rgba(255,255,255,0.3);
		border-top-color: white;
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
		display: inline-block;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	@media (max-width: 768px) {
		.page { padding: 1rem; }
		.header { flex-direction: column; gap: 1rem; }
		.form-row { flex-direction: column; gap: 0; }
		.result-grid { flex-direction: column; }
		.account-card { flex-direction: column; align-items: stretch; }
		.account-actions { justify-content: flex-end; }
	}
</style>
