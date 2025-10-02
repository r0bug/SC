<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';
	import { open, save } from '@tauri-apps/api/dialog';
	import type { Attachment } from '$lib/api/types';
	import { onMount } from 'svelte';

	export let entityType: string;
	export let entityId: string;
	export let uploadedBy: string;

	let attachments: Attachment[] = [];
	let loading = true;
	let error: string | null = null;
	let uploading = false;

	onMount(async () => {
		await loadAttachments();
	});

	async function loadAttachments() {
		try {
			loading = true;
			attachments = await invoke('get_attachments', { entityType, entityId });
			loading = false;
		} catch (e) {
			error = String(e);
			loading = false;
		}
	}

	async function handleUpload() {
		try {
			const selected = await open({
				multiple: false,
				filters: [
					{
						name: 'All Files',
						extensions: ['*']
					}
				]
			});

			if (!selected || Array.isArray(selected)) return;

			uploading = true;
			const attachment: Attachment = await invoke('upload_attachment', {
				filePath: selected,
				entityType,
				entityId,
				uploadedBy
			});

			attachments = [attachment, ...attachments];
			uploading = false;
		} catch (e) {
			error = `Upload failed: ${e}`;
			uploading = false;
		}
	}

	async function handleDownload(attachment: Attachment) {
		try {
			const savePath = await save({
				defaultPath: attachment.filename
			});

			if (!savePath) return;

			await invoke('download_attachment', {
				id: attachment.id,
				savePath
			});

			alert('File downloaded successfully');
		} catch (e) {
			alert(`Download failed: ${e}`);
		}
	}

	async function handleDelete(attachment: Attachment) {
		if (!confirm(`Delete ${attachment.filename}?`)) return;

		try {
			await invoke('delete_attachment', { id: attachment.id });
			attachments = attachments.filter((a) => a.id !== attachment.id);
		} catch (e) {
			alert(`Delete failed: ${e}`);
		}
	}

	function formatFileSize(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
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
</script>

<div class="attachment-manager">
	<div class="header">
		<h3>📎 Attachments ({attachments.length})</h3>
		<button class="upload-button" on:click={handleUpload} disabled={uploading}>
			{uploading ? 'Uploading...' : '+ Add File'}
		</button>
	</div>

	{#if loading}
		<div class="loading">Loading attachments...</div>
	{:else if error}
		<div class="error">{error}</div>
	{:else if attachments.length === 0}
		<div class="empty-state">No attachments yet. Click "Add File" to upload.</div>
	{:else}
		<div class="attachment-list">
			{#each attachments as attachment (attachment.id)}
				<div class="attachment-item">
					<div class="attachment-icon">📄</div>
					<div class="attachment-details">
						<div class="attachment-name">{attachment.filename}</div>
						<div class="attachment-meta">
							<span>{formatFileSize(attachment.size_bytes)}</span>
							{#if attachment.scan_status}
								<span class="separator">•</span>
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
							<button class="action-btn download" on:click={() => handleDownload(attachment)}>
								⬇
							</button>
						{/if}
						<button class="action-btn delete" on:click={() => handleDelete(attachment)}>
							🗑
						</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.attachment-manager {
		padding: 1rem;
		background: #f9f9f9;
		border-radius: 8px;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	h3 {
		margin: 0;
		font-size: 1rem;
		font-weight: 600;
	}

	.upload-button {
		padding: 0.5rem 1rem;
		background: #0066cc;
		color: white;
		border: none;
		border-radius: 4px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.2s;
	}

	.upload-button:hover:not(:disabled) {
		background: #0052a3;
	}

	.upload-button:disabled {
		background: #ccc;
		cursor: not-allowed;
	}

	.loading,
	.error,
	.empty-state {
		text-align: center;
		padding: 2rem;
		color: #757575;
		font-size: 0.875rem;
	}

	.error {
		color: #d32f2f;
	}

	.attachment-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.attachment-item {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.75rem;
		background: white;
		border: 1px solid #e0e0e0;
		border-radius: 4px;
	}

	.attachment-icon {
		font-size: 1.5rem;
	}

	.attachment-details {
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

	.separator {
		color: #bdbdbd;
	}

	.scan-badge {
		font-weight: 500;
	}

	.attachment-actions {
		display: flex;
		gap: 0.5rem;
	}

	.action-btn {
		width: 32px;
		height: 32px;
		border: 1px solid #e0e0e0;
		background: white;
		border-radius: 4px;
		cursor: pointer;
		font-size: 1rem;
		transition: all 0.2s;
	}

	.action-btn:hover {
		background: #f5f5f5;
	}

	.action-btn.delete:hover {
		background: #ffebee;
		border-color: #d32f2f;
	}
</style>
