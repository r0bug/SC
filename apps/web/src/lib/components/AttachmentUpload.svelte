<script lang="ts">
	import { api, ApiError } from '$lib/api/api';
	import type { Attachment, AttachmentEntityType } from '$lib/api/types';
	import { createEventDispatcher } from 'svelte';
	import { toasts } from '$lib/stores/toast';
	import ProgressBar from '$lib/components/ui/ProgressBar.svelte';

	export let entityType: AttachmentEntityType;
	export let entityId: string;
	export let multiple: boolean = false;
	// NOTE: uploadedBy is now derived from authenticated session on server

	const dispatch = createEventDispatcher<{ uploaded: Attachment }>();

	let uploading = false;
	let error: string | null = null;
	let progress = 0;
	let fileName = '';
	let fileSize = '';
	let uploadSpeed = '';

	async function handleFileSelect(event: Event) {
		const target = event.target as HTMLInputElement;
		const files = target.files;
		if (!files || files.length === 0) return;

		const file = files[0];
		fileName = file.name;
		fileSize = formatFileSize(file.size);

		uploading = true;
		error = null;
		progress = 0;

		const startTime = Date.now();
		let lastProgress = 0;

		try {
			// Simulate progress for better UX (real implementation would use XMLHttpRequest with progress events)
			const progressInterval = setInterval(() => {
				if (progress < 90) {
					progress += 5;

					// Calculate upload speed
					const elapsed = (Date.now() - startTime) / 1000; // seconds
					const bytesUploaded = (progress / 100) * file.size;
					const speed = bytesUploaded / elapsed;
					uploadSpeed = formatSpeed(speed);

					lastProgress = progress;
				}
			}, 200);

			const attachment = await api.uploadAttachment(file, entityType, entityId);

			clearInterval(progressInterval);
			progress = 100;

			dispatch('uploaded', attachment);

			toasts.success(`File "${file.name}" uploaded successfully`, 'Upload Complete');

			// Reset
			setTimeout(() => {
				uploading = false;
				progress = 0;
				fileName = '';
				fileSize = '';
				uploadSpeed = '';
				target.value = '';
			}, 1000);
		} catch (e) {
			if (e instanceof ApiError) {
				error = e.getUserMessage();
				toasts.error(error, 'Upload Failed', {
					duration: 7000,
					action: e.isRetryable() ? {
						label: 'Retry',
						callback: () => {
							error = null;
							handleFileSelect(event);
						}
					} : undefined
				});
			} else {
				error = e instanceof Error ? e.message : 'Upload failed';
				toasts.error(error, 'Upload Failed');
			}
			uploading = false;
			progress = 0;
			fileName = '';
			fileSize = '';
			uploadSpeed = '';
		}
	}

	function formatFileSize(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
	}

	function formatSpeed(bytesPerSecond: number): string {
		if (bytesPerSecond < 1024) return `${bytesPerSecond.toFixed(0)} B/s`;
		if (bytesPerSecond < 1024 * 1024) return `${(bytesPerSecond / 1024).toFixed(1)} KB/s`;
		return `${(bytesPerSecond / 1024 / 1024).toFixed(1)} MB/s`;
	}
</script>

<div class="attachment-upload">
	<label for="file-input" class="upload-button" class:disabled={uploading}>
		{#if uploading}
			<span>Uploading...</span>
		{:else}
			<span>📎 Attach File</span>
		{/if}
	</label>
	<input
		id="file-input"
		type="file"
		{multiple}
		on:change={handleFileSelect}
		disabled={uploading}
		class="file-input"
		accept="image/*,.pdf,.doc,.docx,.xls,.xlsx,.txt,.csv"
	/>

	{#if uploading && fileName}
		<div class="upload-info">
			<div class="file-details">
				<strong>{fileName}</strong>
				<span class="file-size">{fileSize}</span>
			</div>
			<ProgressBar
				progress={progress}
				showPercentage={true}
				size="sm"
			/>
			{#if uploadSpeed}
				<div class="upload-speed">{uploadSpeed}</div>
			{/if}
		</div>
	{/if}

	{#if error}
		<div class="error">
			<span class="error-icon">⚠</span>
			<span class="error-message">{error}</span>
		</div>
	{/if}
</div>

<style>
	.attachment-upload {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.upload-button {
		display: inline-flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 1rem;
		background: #0066cc;
		color: white;
		border-radius: 4px;
		cursor: pointer;
		font-size: 0.875rem;
		font-weight: 500;
		transition: background 0.2s;
	}

	.upload-button:hover:not(.disabled) {
		background: #0052a3;
	}

	.upload-button.disabled {
		background: #ccc;
		cursor: not-allowed;
	}

	.file-input {
		display: none;
	}

	.upload-info {
		padding: 1rem;
		background: var(--gray-50, #f9fafb);
		border-radius: 6px;
		border: 1px solid var(--gray-200, #e5e7eb);
	}

	.file-details {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.75rem;
	}

	.file-details strong {
		font-size: 0.875rem;
		color: var(--gray-900, #111827);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 70%;
	}

	.file-size {
		font-size: 0.75rem;
		color: var(--gray-600, #6b7280);
	}

	.upload-speed {
		margin-top: 0.5rem;
		font-size: 0.75rem;
		color: var(--gray-600, #6b7280);
		text-align: center;
	}

	.error {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.75rem;
		background: #fef2f2;
		border: 1px solid #fecaca;
		border-radius: 6px;
		color: #991b1b;
		font-size: 0.875rem;
	}

	.error-icon {
		flex-shrink: 0;
		font-size: 1rem;
	}

	.error-message {
		flex: 1;
		line-height: 1.4;
	}
</style>
