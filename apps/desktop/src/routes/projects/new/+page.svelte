<script lang="ts">
	import { goto } from '$app/navigation';
	import { tauriApi } from '$lib/api/tauri-api';

	let name = '';
	let description = '';
	let status = 'Planning';

	let creating = false;
	let error = '';

	async function handleSubmit() {
		if (!name.trim()) {
			error = 'Project name is required';
			return;
		}

		try {
			creating = true;
			error = '';

			await tauriApi.createProject({
				name,
				description: description || null,
				status
			});

			// Redirect to projects list
			goto('/projects');
		} catch (err: any) {
			error = err.message || 'Failed to create project';
		} finally {
			creating = false;
		}
	}
</script>

<div class="page">
	<div class="header">
		<h1>Create New Project</h1>
		<a href="/projects" class="btn">Cancel</a>
	</div>

	{#if error}
		<div class="error">{error}</div>
	{/if}

	<form on:submit|preventDefault={handleSubmit} class="card">
		<div class="form-group">
			<label for="name">Project Name *</label>
			<input
				id="name"
				type="text"
				bind:value={name}
				placeholder="My Project"
				required
			/>
		</div>

		<div class="form-group">
			<label for="status">Status</label>
			<select id="status" bind:value={status}>
				<option value="Planning">Planning</option>
				<option value="Active">Active</option>
				<option value="On Hold">On Hold</option>
				<option value="Completed">Completed</option>
			</select>
		</div>

		<div class="form-group">
			<label for="description">Description</label>
			<textarea
				id="description"
				bind:value={description}
				placeholder="Project description..."
				rows="6"
			></textarea>
		</div>

		<div class="form-actions">
			<button type="submit" class="btn btn-primary" disabled={creating}>
				{creating ? 'Creating...' : 'Create Project'}
			</button>
		</div>
	</form>
</div>

<style>
	.page { padding: 2rem; max-width: 700px; margin: 0 auto; }
	.header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 2rem; }
	.header h1 { margin: 0; }

	.card {
		background: white;
		border-radius: 8px;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
		padding: 2rem;
	}

	.form-group {
		margin-bottom: 1.5rem;
		display: flex;
		flex-direction: column;
	}

	.form-group label {
		font-weight: 500;
		margin-bottom: 0.5rem;
		color: var(--gray-700);
	}

	.form-group input,
	.form-group select,
	.form-group textarea {
		padding: 0.75rem;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		font-size: 1rem;
	}

	.form-group input:focus,
	.form-group select:focus,
	.form-group textarea:focus {
		outline: none;
		border-color: var(--primary);
	}

	.form-actions {
		display: flex;
		justify-content: flex-end;
		gap: 1rem;
		margin-top: 2rem;
	}

	.btn {
		padding: 0.75rem 1.5rem;
		border: none;
		border-radius: 6px;
		cursor: pointer;
		font-weight: 500;
		text-decoration: none;
		display: inline-block;
	}

	.btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.btn-primary {
		background: var(--primary);
		color: white;
	}

	.error {
		background: #fee2e2;
		border: 1px solid #fecaca;
		color: #991b1b;
		padding: 1rem;
		border-radius: 6px;
		margin-bottom: 1rem;
	}
</style>
