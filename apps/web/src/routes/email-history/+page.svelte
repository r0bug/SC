<script lang="ts">
	import { api, ApiError } from '$lib/api/api';
	import { toasts } from '$lib/stores/toast';
	import { onMount } from 'svelte';
	import SenderDomainTags from '$lib/components/SenderDomainTags.svelte';
	import LabelBadges from '$lib/components/LabelBadges.svelte';
	import ElementSelector from '$lib/components/ElementSelector.svelte';
	import LabelPicker from '$lib/components/LabelPicker.svelte';
	import type { MatcherInput } from '$lib/api/api';
	import { goto } from '$app/navigation';

	let pendingCriteria: MatcherInput[] = [];
	let showLabelPicker = false;

	function handleAddCriterion(e: CustomEvent<{ element_type: string; match_value: string; match_mode: string }>) {
		pendingCriteria = [e.detail];
		showLabelPicker = true;
	}

	function handleLabelSelected() {
		showLabelPicker = false;
		pendingCriteria = [];
	}

	function handleLabelClick(e: CustomEvent<{ conceptId: string }>) {
		goto(`/labels/${e.detail.conceptId}`);
	}

	type Conversation = {
		from_address: string;
		from_name: string | null;
		contact_id: string | null;
		message_count: number;
		last_message_date: number;
		last_subject: string;
		last_body_preview: string;
	};

	type EmailMsg = {
		id: string;
		contact_id: string | null;
		from_address: string;
		from_name: string | null;
		to_address: string;
		cc: string | null;
		subject: string;
		body_text: string | null;
		body_html: string | null;
		message_date: number;
		message_type: number;
		message_id: string | null;
		read_status: number;
		has_attachments: number;
		folder: string | null;
		imported_at: string;
		source_server: string | null;
	};

	type Stats = {
		total_emails: number;
		unique_senders: number;
		earliest_message_date: number | null;
		latest_message_date: number | null;
		received_count: number;
		sent_count: number;
	};

	let conversations: Conversation[] = [];
	let messages: EmailMsg[] = [];
	let stats: Stats | null = null;
	let selectedAddress: string | null = null;
	let selectedName: string | null = null;
	let selectedEmail: EmailMsg | null = null;
	let loading = true;
	let loadingMessages = false;
	let searchQuery = '';

	$: filteredConversations = conversations.filter(c => {
		if (!searchQuery) return true;
		const q = searchQuery.toLowerCase();
		return (
			c.from_address.toLowerCase().includes(q) ||
			(c.from_name && c.from_name.toLowerCase().includes(q)) ||
			c.last_subject.toLowerCase().includes(q)
		);
	});

	onMount(async () => {
		await loadData();
	});

	async function loadData() {
		try {
			loading = true;
			const [convoResult, statsResult] = await Promise.all([
				api.getEmailConversations(),
				api.getEmailStats()
			]);
			conversations = convoResult.conversations;
			stats = statsResult;
		} catch (error) {
			console.error('Failed to load email data:', error);
			if (error instanceof ApiError) {
				toasts.error(error.getUserMessage(), 'Failed to Load');
			} else {
				toasts.error('Failed to load email history', 'Error');
			}
		} finally {
			loading = false;
		}
	}

	async function selectConversation(address: string, name: string | null) {
		selectedAddress = address;
		selectedName = name;
		selectedEmail = null;
		loadingMessages = true;
		try {
			const result = await api.getEmailConversation(address);
			messages = result.messages.sort((a, b) => b.message_date - a.message_date);
			if (messages.length > 0) {
				selectedEmail = messages[0];
			}
		} catch (error) {
			console.error('Failed to load emails:', error);
			toasts.error('Failed to load emails', 'Error');
			messages = [];
		} finally {
			loadingMessages = false;
		}
	}

	function selectEmail(email: EmailMsg) {
		selectedEmail = email;
	}

	function formatDate(ms: number): string {
		if (!ms) return '';
		const d = new Date(ms);
		return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });
	}

	function formatDateFull(ms: number): string {
		if (!ms) return '';
		const d = new Date(ms);
		return d.toLocaleString();
	}

	function getDisplayName(convo: Conversation): string {
		return convo.from_name || convo.from_address;
	}

	function getInitial(convo: Conversation): string {
		if (convo.from_name) return convo.from_name.charAt(0).toUpperCase();
		return convo.from_address.charAt(0).toUpperCase();
	}

	function stripHtml(html: string): string {
		const div = document.createElement('div');
		div.innerHTML = html;
		return div.textContent || div.innerText || '';
	}
</script>

<div class="page">
	<div class="header">
		<div>
			<h1>Email History</h1>
			{#if stats}
				<p class="subtitle">
					{stats.total_emails.toLocaleString()} emails from {stats.unique_senders.toLocaleString()} senders
				</p>
			{/if}
		</div>
		<div class="header-actions">
			<a href="/email-import" class="btn btn-primary">
				Fetch Emails
			</a>
		</div>
	</div>

	{#if loading}
		<div class="loading-state">
			<div class="spinner"></div>
			<p>Loading email history...</p>
		</div>
	{:else if conversations.length === 0}
		<div class="empty-state">
			<div class="empty-icon">@</div>
			<h2>No Emails</h2>
			<p>Import emails from your IMAP account to browse them here.</p>
			<a href="/email-import" class="btn btn-primary">Import Emails</a>
		</div>
	{:else}
		<div class="email-layout">
			<!-- Sender List -->
			<div class="sender-list">
				<div class="search-box">
					<input
						type="text"
						bind:value={searchQuery}
						placeholder="Search senders..."
					/>
				</div>
				<div class="senders">
					{#each filteredConversations as convo}
						<button
							class="sender-item"
							class:active={selectedAddress === convo.from_address}
							on:click={() => selectConversation(convo.from_address, convo.from_name)}
						>
							<div class="sender-avatar">
								{getInitial(convo)}
							</div>
							<div class="sender-details">
								<div class="sender-header">
									<span class="sender-name">{getDisplayName(convo)}</span>
									<span class="sender-date">{formatDate(convo.last_message_date)}</span>
								</div>
								<div class="sender-subject">{convo.last_subject}</div>
								<div class="sender-preview">
									<span class="preview-text">{convo.last_body_preview || '...'}</span>
									<span class="sender-count">{convo.message_count}</span>
								</div>
							</div>
						</button>
					{/each}
				</div>
			</div>

			<!-- Email List + Reading Pane -->
			<div class="email-content">
				{#if selectedAddress}
					{#if loadingMessages}
						<div class="loading-state">
							<div class="spinner"></div>
							<p>Loading emails...</p>
						</div>
					{:else}
						<div class="email-list-pane">
							<div class="email-list-header">
								<h3>{selectedName || selectedAddress}</h3>
								<span class="email-count">{messages.length} emails</span>
							</div>
							<div class="email-list">
								{#each messages as msg}
									<button
										class="email-item"
										class:active={selectedEmail?.id === msg.id}
										on:click={() => selectEmail(msg)}
									>
										<div class="email-item-subject">{msg.subject}</div>
										<div class="email-item-meta">
											<span>{formatDate(msg.message_date)}</span>
											{#if msg.has_attachments}
												<span class="attachment-icon" title="Has attachments">clip</span>
											{/if}
										</div>
									</button>
								{/each}
							</div>
						</div>

						{#if selectedEmail}
							<div class="reading-pane">
								<div class="email-header-detail">
									<h2>{selectedEmail.subject}</h2>
									<div class="email-meta-detail">
										<div class="meta-row">
											<span class="meta-label">From</span>
											<span>{selectedEmail.from_name ? `${selectedEmail.from_name} <${selectedEmail.from_address}>` : selectedEmail.from_address}</span>
										</div>
										<div class="meta-row">
											<span class="meta-label">To</span>
											<span>{selectedEmail.to_address}</span>
										</div>
										{#if selectedEmail.cc}
											<div class="meta-row">
												<span class="meta-label">CC</span>
												<span>{selectedEmail.cc}</span>
											</div>
										{/if}
										<div class="meta-row">
											<span class="meta-label">Date</span>
											<span>{formatDateFull(selectedEmail.message_date)}</span>
										</div>
										{#if selectedEmail.folder}
											<div class="meta-row">
												<span class="meta-label">Folder</span>
												<span>{selectedEmail.folder}</span>
											</div>
										{/if}
									</div>
									<SenderDomainTags
										senderIdentifier={selectedEmail.from_address}
										mode="email"
									/>
									<LabelBadges
										communicationType="email"
										communicationId={selectedEmail.id}
										showSuggested={true}
										on:labelClick={handleLabelClick}
									/>
									<ElementSelector
										fromAddress={selectedEmail.from_address}
										toAddress={selectedEmail.to_address}
										subject={selectedEmail.subject}
										bodyText={selectedEmail.body_text || ''}
										on:addCriterion={handleAddCriterion}
									/>
									{#if showLabelPicker}
										<LabelPicker
											criteria={pendingCriteria}
											on:labelSelected={handleLabelSelected}
											on:close={() => showLabelPicker = false}
										/>
									{/if}
								</div>
								<div class="email-body">
									{#if selectedEmail.body_html}
										{@html selectedEmail.body_html}
									{:else if selectedEmail.body_text}
										<pre class="body-text">{selectedEmail.body_text}</pre>
									{:else}
										<p class="no-body">(No body content)</p>
									{/if}
								</div>
							</div>
						{:else}
							<div class="no-selection">
								<p>Select an email to read</p>
							</div>
						{/if}
					{/if}
				{:else}
					<div class="no-selection">
						<div class="no-selection-icon">@</div>
						<p>Select a sender to view their emails</p>
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>

<style>
	.page {
		padding: 2rem;
		height: calc(100vh - 4rem);
		display: flex;
		flex-direction: column;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: 1.5rem;
		flex-shrink: 0;
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

	.email-layout {
		display: flex;
		flex: 1;
		min-height: 0;
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
		overflow: hidden;
		background: white;
	}

	/* Sender List */
	.sender-list {
		width: 300px;
		border-right: 1px solid #e5e7eb;
		display: flex;
		flex-direction: column;
		flex-shrink: 0;
	}

	.search-box {
		padding: 0.75rem;
		border-bottom: 1px solid #e5e7eb;
	}

	.search-box input {
		width: 100%;
		padding: 0.5rem 0.75rem;
		border: 1px solid #d1d5db;
		border-radius: 0.375rem;
		font-size: 0.875rem;
		box-sizing: border-box;
	}

	.search-box input:focus {
		outline: none;
		border-color: #3b82f6;
		box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.15);
	}

	.senders {
		flex: 1;
		overflow-y: auto;
	}

	.sender-item {
		display: flex;
		align-items: flex-start;
		gap: 0.625rem;
		width: 100%;
		padding: 0.75rem;
		border: none;
		background: none;
		cursor: pointer;
		text-align: left;
		border-bottom: 1px solid #f3f4f6;
		transition: background 0.15s;
	}

	.sender-item:hover { background: #f9fafb; }
	.sender-item.active { background: #eff6ff; }

	.sender-avatar {
		width: 36px;
		height: 36px;
		border-radius: 50%;
		background: #dbeafe;
		color: #2563eb;
		display: flex;
		align-items: center;
		justify-content: center;
		font-weight: 600;
		font-size: 0.875rem;
		flex-shrink: 0;
	}

	.sender-details {
		flex: 1;
		min-width: 0;
	}

	.sender-header {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
		gap: 0.5rem;
	}

	.sender-name {
		font-weight: 500;
		font-size: 0.8125rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.sender-date {
		font-size: 0.7rem;
		color: #9ca3af;
		white-space: nowrap;
		flex-shrink: 0;
	}

	.sender-subject {
		font-size: 0.75rem;
		color: #374151;
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		margin-top: 0.125rem;
	}

	.sender-preview {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 0.5rem;
		margin-top: 0.125rem;
	}

	.preview-text {
		font-size: 0.7rem;
		color: #9ca3af;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.sender-count {
		font-size: 0.65rem;
		background: #e5e7eb;
		color: #374151;
		padding: 0.0625rem 0.375rem;
		border-radius: 999px;
		flex-shrink: 0;
	}

	/* Email Content Area */
	.email-content {
		flex: 1;
		display: flex;
		min-width: 0;
	}

	.email-list-pane {
		width: 280px;
		border-right: 1px solid #e5e7eb;
		display: flex;
		flex-direction: column;
		flex-shrink: 0;
	}

	.email-list-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.75rem 1rem;
		border-bottom: 1px solid #e5e7eb;
		background: #f9fafb;
	}

	.email-list-header h3 {
		margin: 0;
		font-size: 0.875rem;
		font-weight: 600;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.email-count {
		font-size: 0.75rem;
		color: #6b7280;
		flex-shrink: 0;
	}

	.email-list {
		flex: 1;
		overflow-y: auto;
	}

	.email-item {
		display: block;
		width: 100%;
		padding: 0.625rem 1rem;
		border: none;
		background: none;
		cursor: pointer;
		text-align: left;
		border-bottom: 1px solid #f3f4f6;
		transition: background 0.15s;
	}

	.email-item:hover { background: #f9fafb; }
	.email-item.active { background: #eff6ff; }

	.email-item-subject {
		font-size: 0.8125rem;
		font-weight: 500;
		color: #111827;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.email-item-meta {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-top: 0.125rem;
		font-size: 0.7rem;
		color: #9ca3af;
	}

	.attachment-icon {
		font-size: 0.65rem;
		background: #e5e7eb;
		padding: 0.0625rem 0.25rem;
		border-radius: 0.25rem;
	}

	/* Reading Pane */
	.reading-pane {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-width: 0;
		overflow-y: auto;
	}

	.email-header-detail {
		padding: 1.25rem 1.5rem;
		border-bottom: 1px solid #e5e7eb;
	}

	.email-header-detail h2 {
		margin: 0 0 0.75rem 0;
		font-size: 1.25rem;
		font-weight: 600;
		color: #111827;
	}

	.email-meta-detail {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.meta-row {
		display: flex;
		gap: 0.75rem;
		font-size: 0.8125rem;
	}

	.meta-label {
		color: #6b7280;
		font-weight: 500;
		min-width: 3rem;
	}

	.email-body {
		flex: 1;
		padding: 1.25rem 1.5rem;
		font-size: 0.875rem;
		line-height: 1.6;
		overflow-y: auto;
		overflow-x: hidden;
	}

	.email-body :global(img) {
		max-width: 100%;
		height: auto;
	}

	.email-body :global(a) {
		color: #2563eb;
	}

	.body-text {
		white-space: pre-wrap;
		font-family: inherit;
		font-size: 0.875rem;
		margin: 0;
		line-height: 1.6;
	}

	.no-body {
		color: #9ca3af;
		font-style: italic;
	}

	.no-selection {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		color: #9ca3af;
		gap: 0.5rem;
	}

	.no-selection-icon {
		font-size: 3rem;
		font-weight: 700;
		color: #d1d5db;
	}

	.no-selection p {
		font-size: 1rem;
	}

	/* Empty / Loading states */
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 4rem 2rem;
		text-align: center;
		background: white;
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
	}

	.empty-icon {
		font-size: 4rem;
		margin-bottom: 1rem;
		color: #d1d5db;
		font-weight: 700;
	}

	.empty-state h2 {
		margin: 0 0 0.5rem 0;
		color: #374151;
	}

	.empty-state p {
		margin: 0 0 1.5rem 0;
		color: #6b7280;
	}

	.loading-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 3rem;
		color: #6b7280;
		gap: 1rem;
		flex: 1;
	}

	.spinner {
		width: 2rem;
		height: 2rem;
		border: 3px solid #e5e7eb;
		border-top-color: #3b82f6;
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
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

	@media (max-width: 768px) {
		.page { padding: 1rem; }
		.header { flex-direction: column; gap: 1rem; }
		.email-layout { flex-direction: column; }
		.sender-list {
			width: 100%;
			max-height: 35vh;
			border-right: none;
			border-bottom: 1px solid #e5e7eb;
		}
		.email-list-pane { display: none; }
	}
</style>
