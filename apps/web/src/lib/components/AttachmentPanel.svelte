<script lang="ts">
	import { api } from '$lib/api/api';
	import type { Attachment, AttachmentEntityType } from '$lib/api/types';
	import { onMount } from 'svelte';
	import AttachmentUpload from './AttachmentUpload.svelte';
	import AttachmentList from './AttachmentList.svelte';

	export let entityType: AttachmentEntityType;
	export let entityId: string;
	export let uploadedBy: string;
	export let showUpload = true;

	let attachments: Attachment[] = [];
	let loading = true;
	let error: string | null = null;

	onMount(async () => {
		await loadAttachments();
	});

	async function loadAttachments() {
		try {
			loading = true;
			attachments = await api.getAttachments(entityType, entityId);
			loading = false;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load attachments';
			loading = false;
		}
	}

	function handleUploaded(event: CustomEvent<Attachment>) {
		attachments = [event.detail, ...attachments];
	}
</script>

<div class="attachment-panel">
	<div class="panel-header">
		<h3>📎 Attachments</h3>
		{#if attachments.length > 0}
			<span class="count">{attachments.length}</span>
		{/if}
	</div>

	{#if showUpload}
		<div class="upload-section">
			<AttachmentUpload
				{entityType}
				{entityId}
				{uploadedBy}
				on:uploaded={handleUploaded}
			/>
		</div>
	{/if}

	<div class="list-section">
		{#if loading}
			<div class="loading">Loading attachments...</div>
		{:else if error}
			<div class="error">{error}</div>
		{:else}
			<AttachmentList bind:attachments />
		{/if}
	</div>
</div>

<style>
	.attachment-panel {
		background: white;
		border: 1px solid #e0e0e0;
		border-radius: 8px;
		padding: 1rem;
	}

	.panel-header {
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

	.count {
		padding: 0.25rem 0.5rem;
		background: #e3f2fd;
		color: #1976d2;
		border-radius: 12px;
		font-size: 0.75rem;
		font-weight: 600;
	}

	.upload-section {
		margin-bottom: 1rem;
		padding-bottom: 1rem;
		border-bottom: 1px solid #e0e0e0;
	}

	.list-section {
		margin-top: 1rem;
	}

	.loading,
	.error {
		text-align: center;
		padding: 2rem;
		color: #757575;
		font-size: 0.875rem;
	}

	.error {
		color: #d32f2f;
	}
</style>
