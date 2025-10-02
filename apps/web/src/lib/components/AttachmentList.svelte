<script lang="ts">
	import { api } from '$lib/api/api';
	import type { Attachment } from '$lib/api/types';
	import { onMount } from 'svelte';

	export let attachments: Attachment[] = [];

	function formatFileSize(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
	}

	function formatDate(dateStr: string): string {
		const date = new Date(dateStr);
		return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
	}

	function getScanStatusBadge(status: string): { text: string; color: string } {
		switch (status) {
			case 'Clean':
				return { text: '✓ Clean', color: '#4caf50' };
			case 'Pending':
				return { text: '⏳ Scanning', color: '#ff9800' };
			case 'Infected':
				return { text: '⚠ Infected', color: '#f44336' };
			case 'Error':
				return { text: '⚠ Scan Error', color: '#9e9e9e' };
			default:
				return { text: status, color: '#9e9e9e' };
		}
	}

	async function handleDownload(attachment: Attachment) {
		try {
			await api.downloadAttachment(attachment.id, attachment.filename);
		} catch (e) {
			alert('Download failed: ' + (e instanceof Error ? e.message : 'Unknown error'));
		}
	}

	async function handleDelete(attachment: Attachment) {
		if (!confirm(`Delete ${attachment.filename}?`)) return;

		try {
			await api.deleteAttachment(attachment.id);
			attachments = attachments.filter((a) => a.id !== attachment.id);
		} catch (e) {
			alert('Delete failed: ' + (e instanceof Error ? e.message : 'Unknown error'));
		}
	}
</script>

<div class="attachment-list">
	{#if attachments.length === 0}
		<div class="empty-state">No attachments</div>
	{:else}
		<div class="attachments">
			{#each attachments as attachment (attachment.id)}
				<div class="attachment-item">
					<div class="attachment-icon">📄</div>
					<div class="attachment-info">
						<div class="attachment-name">{attachment.filename}</div>
						<div class="attachment-meta">
							<span>{formatFileSize(attachment.size_bytes)}</span>
							<span>•</span>
							<span>{formatDate(attachment.created_at)}</span>
							{#if attachment.scan_status}
								<span>•</span>
								<span
									class="scan-badge"
									style="color: {getScanStatusBadge(attachment.scan_status).color}"
								>
									{getScanStatusBadge(attachment.scan_status).text}
								</span>
							{/if}
						</div>
					</div>
					<div class="attachment-actions">
						{#if attachment.scan_status !== 'Infected'}
							<button on:click={() => handleDownload(attachment)} class="action-button">
								⬇ Download
							</button>
						{/if}
						<button on:click={() => handleDelete(attachment)} class="action-button delete">
							🗑 Delete
						</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.attachment-list {
		width: 100%;
	}

	.empty-state {
		text-align: center;
		padding: 2rem;
		color: #757575;
		font-style: italic;
	}

	.attachments {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.attachment-item {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding: 0.75rem;
		background: #f5f5f5;
		border-radius: 4px;
		transition: background 0.2s;
	}

	.attachment-item:hover {
		background: #eeeeee;
	}

	.attachment-icon {
		font-size: 1.5rem;
		flex-shrink: 0;
	}

	.attachment-info {
		flex: 1;
		min-width: 0;
	}

	.attachment-name {
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.attachment-meta {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.75rem;
		color: #757575;
		margin-top: 0.25rem;
	}

	.scan-badge {
		font-weight: 500;
	}

	.attachment-actions {
		display: flex;
		gap: 0.5rem;
	}

	.action-button {
		padding: 0.375rem 0.75rem;
		font-size: 0.75rem;
		border: none;
		border-radius: 4px;
		cursor: pointer;
		background: white;
		color: #424242;
		transition: all 0.2s;
		white-space: nowrap;
	}

	.action-button:hover {
		background: #e0e0e0;
	}

	.action-button.delete {
		color: #d32f2f;
	}

	.action-button.delete:hover {
		background: #ffebee;
	}
</style>
