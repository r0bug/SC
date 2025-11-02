<script lang="ts">
	import { onMount } from 'svelte';
	import { tauriApi } from '$lib/api/tauri-api';
	import type { Project, ProjectStatus } from '$lib/api/types';

	let projects: Project[] = [];
	let filteredProjects: Project[] = [];
	let loading = true;
	let sortBy: 'name' | 'status' | 'size' | 'date' = 'name';
	let filterStatus: 'all' | ProjectStatus = 'all';
	let searchQuery = '';

	onMount(async () => {
		await loadProjects();
	});

	async function loadProjects() {
		try {
			loading = true;
			projects = await tauriApi.getProjects();
			applyFilters();
		} catch (error) {
			console.error('Failed to load projects:', error);
		} finally {
			loading = false;
		}
	}

	async function deleteProject(id: string, name: string) {
		if (!confirm(`Are you sure you want to delete the project "${name}"?`)) return;

		try {
			await tauriApi.deleteProject(id);
			projects = projects.filter(p => p.id !== id);
			applyFilters();
			await tauriApi.showNotification('Project Deleted', `${name} has been deleted successfully`);
		} catch (error) {
			console.error('Failed to delete project:', error);
			alert('Failed to delete project');
		}
	}

	function applyFilters() {
		// Filter by status
		let filtered = filterStatus === 'all'
			? projects
			: projects.filter(p => p.status === filterStatus);

		// Filter by search query
		if (searchQuery.trim()) {
			const query = searchQuery.toLowerCase();
			filtered = filtered.filter(p =>
				p.name.toLowerCase().includes(query) ||
				(p.description && p.description.toLowerCase().includes(query))
			);
		}

		// Sort
		switch (sortBy) {
			case 'name':
				filtered = filtered.sort((a, b) => a.name.localeCompare(b.name));
				break;
			case 'status':
				filtered = filtered.sort((a, b) => a.status.localeCompare(b.status));
				break;
			case 'size':
				filtered = filtered.sort((a, b) => b.contacts.length - a.contacts.length);
				break;
			case 'date':
				filtered = filtered.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime());
				break;
		}

		filteredProjects = filtered;
	}

	function handleFilterChange() {
		applyFilters();
	}

	function getStatusColor(status: string): string {
		switch (status) {
			case 'Active': return 'status-active';
			case 'Completed': return 'status-completed';
			case 'OnHold': return 'status-onhold';
			case 'Archived': return 'status-archived';
			default: return 'status-active';
		}
	}

	function getStatusCount(status: ProjectStatus): number {
		return projects.filter(p => p.status === status).length;
	}
</script>

<div class="page">
	<div class="header">
		<div>
			<h1>Projects</h1>
			<p class="subtitle">Manage your projects and collaborations</p>
		</div>
		<div class="header-actions">
			<select bind:value={sortBy} on:change={handleFilterChange} class="sort-select">
				<option value="name">Sort by Name</option>
				<option value="status">Sort by Status</option>
				<option value="size">Sort by Size</option>
				<option value="date">Sort by Date</option>
			</select>
		</div>
	</div>

	<div class="filters card">
		<input
			type="text"
			placeholder="Search projects..."
			bind:value={searchQuery}
			on:input={handleFilterChange}
			class="search-input"
		/>

		<div class="status-filters">
			<button
				class="filter-btn {filterStatus === 'all' ? 'active' : ''}"
				on:click={() => { filterStatus = 'all'; handleFilterChange(); }}
			>
				All ({projects.length})
			</button>
			<button
				class="filter-btn status-active {filterStatus === 'Active' ? 'active' : ''}"
				on:click={() => { filterStatus = 'Active'; handleFilterChange(); }}
			>
				Active ({getStatusCount('Active')})
			</button>
			<button
				class="filter-btn status-completed {filterStatus === 'Completed' ? 'active' : ''}"
				on:click={() => { filterStatus = 'Completed'; handleFilterChange(); }}
			>
				Completed ({getStatusCount('Completed')})
			</button>
			<button
				class="filter-btn status-onhold {filterStatus === 'OnHold' ? 'active' : ''}"
				on:click={() => { filterStatus = 'OnHold'; handleFilterChange(); }}
			>
				On Hold ({getStatusCount('OnHold')})
			</button>
			<button
				class="filter-btn status-archived {filterStatus === 'Archived' ? 'active' : ''}"
				on:click={() => { filterStatus = 'Archived'; handleFilterChange(); }}
			>
				Archived ({getStatusCount('Archived')})
			</button>
		</div>
	</div>

	{#if loading}
		<div class="loading">Loading projects...</div>
	{:else if filteredProjects.length === 0}
		<div class="empty card">
			{#if searchQuery || filterStatus !== 'all'}
				<p>No projects match your filters.</p>
				<button on:click={() => { searchQuery = ''; filterStatus = 'all'; handleFilterChange(); }} class="btn btn-primary">
					Clear Filters
				</button>
			{:else}
				<p>No projects found.</p>
				<p>Projects help you track work and collaborate with your contacts.</p>
			{/if}
		</div>
	{:else}
		<div class="project-grid">
			{#each filteredProjects as project}
				<div class="project-card card">
					<div class="project-header">
						<div class="project-title">
							<h3>{project.name}</h3>
							<span class="status-badge {getStatusColor(project.status)}">{project.status}</span>
						</div>
						<button
							on:click={() => deleteProject(project.id, project.name)}
							class="btn-icon btn-danger"
							title="Delete project"
						>
							🗑️
						</button>
					</div>

					{#if project.description}
						<p class="project-description">{project.description}</p>
					{/if}

					<div class="project-stats">
						<div class="stat-item">
							<span class="stat-icon">👥</span>
							<span>{project.contacts.length} contacts</span>
						</div>
						{#if project.tags && project.tags.length > 0}
							<div class="stat-item">
								<span class="stat-icon">🏷️</span>
								<span>{project.tags.length} tags</span>
							</div>
						{/if}
						{#if project.attachment_ids && project.attachment_ids.length > 0}
							<div class="stat-item">
								<span class="stat-icon">📎</span>
								<span>{project.attachment_ids.length} files</span>
							</div>
						{/if}
					</div>

					{#if project.tags && project.tags.length > 0}
						<div class="tags">
							{#each project.tags.slice(0, 3) as tag}
								<span class="tag">Tag {tag.substring(0, 8)}</span>
							{/each}
							{#if project.tags.length > 3}
								<span class="tag">+{project.tags.length - 3} more</span>
							{/if}
						</div>
					{/if}

					<div class="project-meta">
						<span class="meta-item">Created {new Date(project.created_at).toLocaleDateString()}</span>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page {
		padding: 2rem;
		max-width: 1400px;
		margin: 0 auto;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
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

	.header-actions {
		display: flex;
		gap: 1rem;
		align-items: center;
	}

	.sort-select {
		padding: 0.5rem 1rem;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		font-size: 0.875rem;
		background: white;
		cursor: pointer;
	}

	.sort-select:focus {
		outline: none;
		border-color: var(--primary);
		box-shadow: 0 0 0 3px var(--primary-light);
	}

	.filters {
		margin-bottom: 2rem;
		padding: 1.5rem;
	}

	.search-input {
		width: 100%;
		padding: 0.75rem 1rem;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		font-size: 1rem;
		margin-bottom: 1rem;
	}

	.search-input:focus {
		outline: none;
		border-color: var(--primary);
		box-shadow: 0 0 0 3px var(--primary-light);
	}

	.status-filters {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
	}

	.filter-btn {
		padding: 0.5rem 1rem;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		background: white;
		cursor: pointer;
		font-size: 0.875rem;
		font-weight: 500;
		transition: all 0.2s;
	}

	.filter-btn:hover {
		background: var(--gray-50);
	}

	.filter-btn.active {
		background: var(--primary);
		color: white;
		border-color: var(--primary);
	}

	.filter-btn.status-active.active {
		background: #1e40af;
		border-color: #1e40af;
	}

	.filter-btn.status-completed.active {
		background: #065f46;
		border-color: #065f46;
	}

	.filter-btn.status-onhold.active {
		background: #92400e;
		border-color: #92400e;
	}

	.filter-btn.status-archived.active {
		background: #374151;
		border-color: #374151;
	}

	.loading {
		text-align: center;
		padding: 3rem;
		color: var(--gray-600);
		font-size: 1.125rem;
	}

	.empty {
		text-align: center;
		padding: 3rem;
	}

	.empty p {
		margin: 0 0 1rem;
		color: var(--gray-600);
		font-size: 1.125rem;
	}

	.project-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
		gap: 1.5rem;
	}

	.project-card {
		padding: 1.5rem;
		transition: transform 0.2s, box-shadow 0.2s;
	}

	.project-card:hover {
		transform: translateY(-2px);
		box-shadow: var(--shadow-md);
	}

	.project-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: 1rem;
	}

	.project-title {
		flex: 1;
	}

	.project-header h3 {
		margin: 0 0 0.5rem;
		font-size: 1.25rem;
		color: var(--gray-900);
	}

	.status-badge {
		padding: 0.25rem 0.75rem;
		border-radius: 999px;
		font-size: 0.75rem;
		font-weight: 600;
	}

	.status-active {
		background: #dbeafe;
		color: #1e40af;
	}

	.status-completed {
		background: #d1fae5;
		color: #065f46;
	}

	.status-onhold {
		background: #fef3c7;
		color: #92400e;
	}

	.status-archived {
		background: #e5e7eb;
		color: #374151;
	}

	.btn-icon {
		padding: 0.5rem;
		background: none;
		border: none;
		cursor: pointer;
		font-size: 1.25rem;
		border-radius: 4px;
		transition: background-color 0.2s;
	}

	.btn-icon:hover {
		background: var(--gray-100);
	}

	.btn-danger:hover {
		background: var(--color-error-50);
	}

	.project-description {
		margin: 0 0 1rem;
		color: var(--gray-700);
		font-size: 0.875rem;
		line-height: 1.5;
	}

	.project-stats {
		display: flex;
		flex-wrap: wrap;
		gap: 1rem;
		margin-bottom: 1rem;
	}

	.stat-item {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.875rem;
		color: var(--gray-700);
	}

	.stat-icon {
		font-size: 1rem;
	}

	.tags {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
		margin-bottom: 1rem;
	}

	.tag {
		display: inline-block;
		padding: 0.25rem 0.75rem;
		background: var(--primary-light);
		color: var(--primary);
		border-radius: 999px;
		font-size: 0.75rem;
		font-weight: 500;
	}

	.project-meta {
		display: flex;
		gap: 1rem;
		font-size: 0.75rem;
		color: var(--gray-500);
		border-top: 1px solid var(--gray-200);
		padding-top: 1rem;
	}

	.meta-item {
		display: flex;
		align-items: center;
		gap: 0.25rem;
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
		text-decoration: none;
		display: inline-block;
		transition: all 0.2s;
	}

	.btn-primary {
		background: var(--primary);
		color: white;
	}

	.btn-primary:hover {
		background: var(--primary-dark);
	}
</style>
