<script lang="ts">
	import { api, ApiError } from '$lib/api/api';
	import { toasts } from '$lib/stores/toast';
	import { onMount } from 'svelte';
	import SenderDomainTags from '$lib/components/SenderDomainTags.svelte';
	import LabelBadges from '$lib/components/LabelBadges.svelte';
	import { goto } from '$app/navigation';

	function handleLabelClick(e: CustomEvent<{ conceptId: string }>) {
		goto(`/labels/${e.detail.conceptId}`);
	}

	type Conversation = {
		phone_number: string;
		contact_name: string | null;
		contact_id: string | null;
		message_count: number;
		last_message_date: number;
		last_message_preview: string;
	};

	type Message = {
		id: string;
		contact_id: string | null;
		phone_number: string;
		contact_name: string | null;
		message_date: number;
		message_type: number;
		subject: string | null;
		body: string;
		readable_date: string;
		thread_id: number | null;
		read_status: number;
		imported_at: string;
		source_file: string | null;
	};

	type Stats = {
		total_messages: number;
		unique_conversations: number;
		earliest_message_date: number | null;
		latest_message_date: number | null;
		messages_with_contacts: number;
		messages_without_contacts: number;
	};

	let conversations: Conversation[] = [];
	let messages: Message[] = [];
	let stats: Stats | null = null;
	let selectedPhone: string | null = null;
	let selectedName: string | null = null;
	let loading = true;
	let loadingMessages = false;
	let searchQuery = '';

	$: filteredConversations = conversations.filter(c => {
		if (!searchQuery) return true;
		const q = searchQuery.toLowerCase();
		return (
			c.phone_number.toLowerCase().includes(q) ||
			(c.contact_name && c.contact_name.toLowerCase().includes(q)) ||
			c.last_message_preview.toLowerCase().includes(q)
		);
	});

	onMount(async () => {
		await loadData();
	});

	async function loadData() {
		try {
			loading = true;
			const [convoResult, statsResult] = await Promise.all([
				api.getSmsConversations(),
				api.getSmsStats()
			]);
			conversations = convoResult.conversations;
			stats = statsResult;
		} catch (error) {
			console.error('Failed to load SMS data:', error);
			if (error instanceof ApiError) {
				toasts.error(error.getUserMessage(), 'Failed to Load');
			} else {
				toasts.error('Failed to load SMS history', 'Error');
			}
		} finally {
			loading = false;
		}
	}

	async function selectConversation(phone: string, name: string | null) {
		selectedPhone = phone;
		selectedName = name;
		loadingMessages = true;
		try {
			const result = await api.getSmsConversation(phone);
			// Sort oldest first for chat view
			messages = result.messages.sort((a, b) => a.message_date - b.message_date);
		} catch (error) {
			console.error('Failed to load conversation:', error);
			toasts.error('Failed to load conversation', 'Error');
			messages = [];
		} finally {
			loadingMessages = false;
			// Scroll to bottom after render
			setTimeout(scrollToBottom, 100);
		}
	}

	function scrollToBottom() {
		const container = document.querySelector('.messages-container');
		if (container) {
			container.scrollTop = container.scrollHeight;
		}
	}

	function formatDate(ms: number): string {
		if (!ms) return '';
		const d = new Date(ms);
		return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });
	}

	function formatTime(ms: number): string {
		if (!ms) return '';
		const d = new Date(ms);
		return d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
	}

	function formatDateFull(ms: number): string {
		if (!ms) return '';
		const d = new Date(ms);
		return d.toLocaleString();
	}

	function isSent(type: number): boolean {
		return type === 2; // 2=sent in Android SMS format
	}

	function getDisplayName(convo: Conversation): string {
		return convo.contact_name || convo.phone_number;
	}

	function shouldShowDateHeader(messages: Message[], index: number): boolean {
		if (index === 0) return true;
		const curr = new Date(messages[index].message_date).toDateString();
		const prev = new Date(messages[index - 1].message_date).toDateString();
		return curr !== prev;
	}
</script>

<div class="page">
	<div class="header">
		<div>
			<h1>SMS History</h1>
			{#if stats}
				<p class="subtitle">
					{stats.total_messages.toLocaleString()} messages across {stats.unique_conversations.toLocaleString()} conversations
				</p>
			{/if}
		</div>
		<div class="header-actions">
			<a href="/sms-import" class="btn btn-primary">
				📥 Import SMS
			</a>
		</div>
	</div>

	{#if loading}
		<div class="loading-state">
			<div class="spinner"></div>
			<p>Loading SMS history...</p>
		</div>
	{:else if conversations.length === 0}
		<div class="empty-state">
			<div class="empty-icon">💬</div>
			<h2>No SMS Messages</h2>
			<p>Import your Android SMS backup to see your message history here.</p>
			<a href="/sms-import" class="btn btn-primary">Import SMS Backup</a>
		</div>
	{:else}
		<div class="chat-layout">
			<!-- Conversation List -->
			<div class="conversation-list">
				<div class="search-box">
					<input
						type="text"
						bind:value={searchQuery}
						placeholder="Search conversations..."
					/>
				</div>
				<div class="conversations">
					{#each filteredConversations as convo}
						<button
							class="conversation-item"
							class:active={selectedPhone === convo.phone_number}
							on:click={() => selectConversation(convo.phone_number, convo.contact_name)}
						>
							<div class="convo-avatar">
								{#if convo.contact_name}
									{convo.contact_name.charAt(0).toUpperCase()}
								{:else}
									#
								{/if}
							</div>
							<div class="convo-details">
								<div class="convo-header">
									<span class="convo-name">{getDisplayName(convo)}</span>
									<span class="convo-date">{formatDate(convo.last_message_date)}</span>
								</div>
								<div class="convo-preview">
									<span class="preview-text">{convo.last_message_preview || '...'}</span>
									<span class="convo-count">{convo.message_count}</span>
								</div>
							</div>
						</button>
					{/each}
				</div>
			</div>

			<!-- Message Thread -->
			<div class="message-thread">
				{#if selectedPhone}
					<div class="thread-header">
						<div class="thread-info">
							<h2>{selectedName || selectedPhone}</h2>
							{#if selectedName}
								<p class="thread-phone">{selectedPhone}</p>
							{/if}
						</div>
						<span class="thread-count">{messages.length} messages</span>
					</div>
					<div class="thread-domains">
						<SenderDomainTags
							senderIdentifier={selectedPhone}
							mode="sms"
						/>
					</div>
					{#if messages.length > 0}
						<div class="thread-labels">
							<LabelBadges
								communicationType="sms"
								communicationId={messages[0].id}
								showSuggested={true}
								on:labelClick={handleLabelClick}
							/>
						</div>
					{/if}

					{#if loadingMessages}
						<div class="loading-state">
							<div class="spinner"></div>
							<p>Loading messages...</p>
						</div>
					{:else}
						<div class="messages-container">
							{#each messages as msg, i}
								{#if shouldShowDateHeader(messages, i)}
									<div class="date-header">
										<span>{formatDate(msg.message_date)}</span>
									</div>
								{/if}
								<div class="message-bubble" class:sent={isSent(msg.message_type)} class:received={!isSent(msg.message_type)}>
									{#if msg.subject}
										<div class="message-subject">{msg.subject}</div>
									{/if}
									<div class="message-body">{msg.body}</div>
									<div class="message-meta">
										<span class="message-time">{formatTime(msg.message_date)}</span>
										{#if msg.read_status === 0}
											<span class="unread-badge">unread</span>
										{/if}
									</div>
								</div>
							{/each}
						</div>
					{/if}
				{:else}
					<div class="no-selection">
						<div class="no-selection-icon">💬</div>
						<p>Select a conversation to view messages</p>
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

	.chat-layout {
		display: flex;
		flex: 1;
		min-height: 0;
		border: 1px solid #e5e7eb;
		border-radius: 0.5rem;
		overflow: hidden;
		background: white;
	}

	/* Conversation List */
	.conversation-list {
		width: 360px;
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

	.conversations {
		flex: 1;
		overflow-y: auto;
	}

	.conversation-item {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		width: 100%;
		padding: 0.75rem;
		border: none;
		background: none;
		cursor: pointer;
		text-align: left;
		border-bottom: 1px solid #f3f4f6;
		transition: background 0.15s;
	}

	.conversation-item:hover {
		background: #f9fafb;
	}

	.conversation-item.active {
		background: #eff6ff;
	}

	.convo-avatar {
		width: 40px;
		height: 40px;
		border-radius: 50%;
		background: #e5e7eb;
		display: flex;
		align-items: center;
		justify-content: center;
		font-weight: 600;
		color: #374151;
		flex-shrink: 0;
	}

	.convo-details {
		flex: 1;
		min-width: 0;
	}

	.convo-header {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
		gap: 0.5rem;
	}

	.convo-name {
		font-weight: 500;
		font-size: 0.875rem;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.convo-date {
		font-size: 0.75rem;
		color: #9ca3af;
		white-space: nowrap;
		flex-shrink: 0;
	}

	.convo-preview {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 0.5rem;
		margin-top: 0.125rem;
	}

	.preview-text {
		font-size: 0.8rem;
		color: #6b7280;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.convo-count {
		font-size: 0.7rem;
		background: #e5e7eb;
		color: #374151;
		padding: 0.125rem 0.4rem;
		border-radius: 999px;
		flex-shrink: 0;
	}

	/* Message Thread */
	.message-thread {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-width: 0;
	}

	.thread-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1rem 1.5rem;
		border-bottom: 1px solid #e5e7eb;
		background: #f9fafb;
	}

	.thread-header h2 {
		margin: 0;
		font-size: 1.1rem;
		font-weight: 600;
	}

	.thread-phone {
		margin: 0.125rem 0 0 0;
		font-size: 0.8rem;
		color: #6b7280;
	}

	.thread-count {
		font-size: 0.8rem;
		color: #6b7280;
	}

	.thread-domains {
		padding: 0 1.5rem 0.5rem;
		border-bottom: 1px solid #e5e7eb;
	}

	.messages-container {
		flex: 1;
		overflow-y: auto;
		padding: 1rem 1.5rem;
		display: flex;
		flex-direction: column;
		gap: 0.375rem;
	}

	.date-header {
		text-align: center;
		padding: 0.75rem 0;
	}

	.date-header span {
		background: #f3f4f6;
		color: #6b7280;
		padding: 0.25rem 0.75rem;
		border-radius: 999px;
		font-size: 0.75rem;
		font-weight: 500;
	}

	.message-bubble {
		max-width: 75%;
		padding: 0.625rem 0.875rem;
		border-radius: 1rem;
		line-height: 1.45;
		font-size: 0.875rem;
		word-wrap: break-word;
	}

	.message-bubble.sent {
		align-self: flex-end;
		background: #3b82f6;
		color: white;
		border-bottom-right-radius: 0.25rem;
	}

	.message-bubble.received {
		align-self: flex-start;
		background: #f3f4f6;
		color: #1f2937;
		border-bottom-left-radius: 0.25rem;
	}

	.message-subject {
		font-weight: 600;
		margin-bottom: 0.25rem;
	}

	.message-body {
		white-space: pre-wrap;
	}

	.message-meta {
		display: flex;
		align-items: center;
		gap: 0.375rem;
		margin-top: 0.25rem;
	}

	.message-time {
		font-size: 0.7rem;
		opacity: 0.7;
	}

	.unread-badge {
		font-size: 0.65rem;
		background: rgba(255,255,255,0.2);
		padding: 0.0625rem 0.3rem;
		border-radius: 0.25rem;
	}

	.received .unread-badge {
		background: #e5e7eb;
		color: #6b7280;
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

		.chat-layout {
			flex-direction: column;
		}

		.conversation-list {
			width: 100%;
			max-height: 40vh;
			border-right: none;
			border-bottom: 1px solid #e5e7eb;
		}

		.message-bubble {
			max-width: 85%;
		}
	}
</style>
