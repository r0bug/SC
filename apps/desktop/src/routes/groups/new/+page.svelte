<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { tauriApi } from '$lib/api/tauri-api';

	let name = '';
	let description = '';
	let saving = false;
	let error = '';

	async function handleSubmit() {
		if (!name.trim()) {
			error = 'Group name is required';
			return;
		}

		try {
			saving = true;
			error = '';

			const group = await tauriApi.createGroup({
				name: name.trim(),
				description: description.trim() || undefined,
				contact_ids: []
			});

			// Show success notification
			await tauriApi.showNotification('Group Created', `${name} has been created successfully`);

			// Navigate to the group detail page
			goto(`/groups/${group.id}`);
		} catch (err) {
			console.error('Failed to create group:', err);
			error = 'Failed to create group. Please try again.';
			saving = false;
		}
	}

	function handleCancel() {
		goto('/groups');
	}
</script>

<div class="page">
	<div class="header">
		<div>
			<h1>Create New Group</h1>
			<p class="subtitle">Organize your contacts into a group</p>
		</div>
	</div>

	<div class="card form-card">
		<form on:submit|preventDefault={handleSubmit}>
			<div class="form-group">
				<label for="name">
					Group Name <span class="required">*</span>
				</label>
				<input
					id="name"
					type="text"
					bind:value={name}
					placeholder="e.g., Project Team, Family, Work Contacts"
					required
					disabled={saving}
					class="form-input"
				/>
			</div>

			<div class="form-group">
				<label for="description">
					Description
				</label>
				<textarea
					id="description"
					bind:value={description}
					placeholder="Optional description for this group"
					rows="4"
					disabled={saving}
					class="form-input"
				/>
			</div>

			{#if error}
				<div class="error-message">
					{error}
				</div>
			{/if}

			<div class="form-actions">
				<button
					type="button"
					on:click={handleCancel}
					disabled={saving}
					class="btn btn-secondary"
				>
					Cancel
				</button>
				<button
					type="submit"
					disabled={saving || !name.trim()}
					class="btn btn-primary"
				>
					{saving ? 'Creating...' : 'Create Group'}
				</button>
			</div>
		</form>
	</div>
</div>

<style>
	.page {
		padding: 2rem;
		max-width: 800px;
		margin: 0 auto;
	}

	.header {
		margin-bottom: 2rem;
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

	.form-card {
		background: white;
		border-radius: 8px;
		box-shadow: var(--shadow-sm);
		padding: 2rem;
	}

	.form-group {
		margin-bottom: 1.5rem;
	}

	.form-group label {
		display: block;
		margin-bottom: 0.5rem;
		font-weight: 600;
		color: var(--gray-900);
		font-size: 0.875rem;
	}

	.required {
		color: var(--color-error);
	}

	.form-input {
		width: 100%;
		padding: 0.75rem;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		font-size: 1rem;
		font-family: inherit;
		transition: border-color 0.2s, box-shadow 0.2s;
	}

	.form-input:focus {
		outline: none;
		border-color: var(--primary);
		box-shadow: 0 0 0 3px var(--primary-light);
	}

	.form-input:disabled {
		background: var(--gray-100);
		cursor: not-allowed;
	}

	textarea.form-input {
		resize: vertical;
		min-height: 100px;
	}

	.error-message {
		padding: 0.75rem;
		background: var(--color-error-50);
		color: var(--color-error);
		border-radius: 6px;
		font-size: 0.875rem;
		margin-bottom: 1rem;
	}

	.form-actions {
		display: flex;
		gap: 1rem;
		justify-content: flex-end;
		margin-top: 2rem;
		padding-top: 1.5rem;
		border-top: 1px solid var(--gray-200);
	}

	.btn {
		padding: 0.75rem 1.5rem;
		border: none;
		border-radius: 6px;
		font-size: 1rem;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.2s;
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
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

	.card {
		background: white;
		border-radius: 8px;
		box-shadow: var(--shadow-sm);
	}
</style>
