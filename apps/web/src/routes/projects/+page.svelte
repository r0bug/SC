<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { Project, Contact, Tag } from '$lib/api/types';

	let projects: Project[] = [];
	let contacts: Contact[] = [];
	let tags: Tag[] = [];
	let loading = true;
	let showModal = false;
	let editingProject: Project | null = null;

	let formName = '';
	let formDescription = '';
	let formStatus: 'Active' | 'Completed' | 'Archived' | 'OnHold' = 'Active';
	let formContacts: string[] = [];
	let formTags: string[] = [];
	let filterStatus = 'all';

	onMount(async () => {
		await Promise.all([loadProjects(), loadContacts(), loadTags()]);
		setupWebSocket();
	});

	async function loadProjects() {
		try {
			loading = true;
			projects = await api.getProjects();
		} catch (error) {
			console.error('Failed to load projects:', error);
		} finally {
			loading = false;
		}
	}

	async function loadContacts() {
		try {
			contacts = await api.getContacts(500, 0);
		} catch (error) {
			console.error('Failed to load contacts:', error);
		}
	}

	async function loadTags() {
		try {
			tags = await api.getTags();
		} catch (error) {
			console.error('Failed to load tags:', error);
		}
	}

	function setupWebSocket() {
		api.onWebSocketMessage('project_updated', (data) => {
			const index = projects.findIndex(p => p.id === data.id);
			if (index >= 0) projects[index] = data;
			projects = projects;
		});
		api.onWebSocketMessage('project_created', (data) => {
			projects = [data, ...projects];
		});
		api.onWebSocketMessage('project_deleted', (data) => {
			projects = projects.filter(p => p.id !== data.id);
		});
	}

	function openCreate() {
		editingProject = null;
		formName = '';
		formDescription = '';
		formStatus = 'Active';
		formContacts = [];
		formTags = [];
		showModal = true;
	}

	function openEdit(project: Project) {
		editingProject = project;
		formName = project.name;
		formDescription = project.description || '';
		formStatus = project.status;
		formContacts = [...project.contacts];
		formTags = [...project.tags];
		showModal = true;
	}

	async function handleSubmit() {
		try {
			const projectData = {
				name: formName,
				description: formDescription,
				status: formStatus,
				contacts: formContacts,
				tags: formTags
			};

			if (editingProject) {
				const updated = await api.updateProject(editingProject.id, projectData);
				projects = projects.map(p => p.id === editingProject!.id ? updated : p);
			} else {
				const created = await api.createProject(projectData as any);
				projects = [created, ...projects];
			}
			showModal = false;
		} catch (error: any) {
			alert('Failed to save project: ' + error.message);
		}
	}

	async function handleDelete(id: string) {
		if (!confirm('Delete this project?')) return;
		try {
			await api.deleteProject(id);
			projects = projects.filter(p => p.id !== id);
		} catch (error: any) {
			alert('Failed to delete: ' + error.message);
		}
	}

	function toggleContact(id: string) {
		formContacts = formContacts.includes(id)
			? formContacts.filter(c => c !== id)
			: [...formContacts, id];
	}

	function toggleTag(id: string) {
		formTags = formTags.includes(id)
			? formTags.filter(t => t !== id)
			: [...formTags, id];
	}

	$: filteredProjects = projects.filter(p =>
		filterStatus === 'all' || p.status === filterStatus
	);
</script>

<div class="page">
	<div class="header">
		<h1>Projects</h1>
		<button on:click={openCreate} class="btn btn-primary">+ New Project</button>
	</div>

	<div class="filters">
		<button class:active={filterStatus === 'all'} on:click={() => filterStatus = 'all'}>All</button>
		<button class:active={filterStatus === 'Active'} on:click={() => filterStatus = 'Active'}>Active</button>
		<button class:active={filterStatus === 'Completed'} on:click={() => filterStatus = 'Completed'}>Completed</button>
		<button class:active={filterStatus === 'OnHold'} on:click={() => filterStatus = 'OnHold'}>On Hold</button>
		<button class:active={filterStatus === 'Archived'} on:click={() => filterStatus = 'Archived'}>Archived</button>
	</div>

	{#if loading}
		<div class="loading">Loading...</div>
	{:else if filteredProjects.length === 0}
		<div class="empty">
			<p>No projects</p>
			<button on:click={openCreate} class="btn btn-primary">Create first project</button>
		</div>
	{:else}
		<div class="projects-grid">
			{#each filteredProjects as project}
				<div class="project-card card">
					<div class="project-header">
						<div>
							<h3>{project.name}</h3>
							<span class="status-badge status-{project.status.toLowerCase()}">{project.status}</span>
						</div>
						<div class="actions">
							<button on:click={() => openEdit(project)} class="btn btn-sm">Edit</button>
							<button on:click={() => handleDelete(project.id)} class="btn btn-sm">Delete</button>
						</div>
					</div>
					{#if project.description}
						<p class="description">{project.description}</p>
					{/if}
					<div class="project-meta">
						<span>Team: {project.contacts.length}</span>
						<span>Tags: {project.tags.length}</span>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

{#if showModal}
	<div class="modal-overlay" on:click={() => showModal = false}>
		<div class="modal" on:click|stopPropagation>
			<h2>{editingProject ? 'Edit' : 'New'} Project</h2>
			<form on:submit|preventDefault={handleSubmit}>
				<div class="form-group">
					<label>Name</label>
					<input type="text" bind:value={formName} required />
				</div>
				<div class="form-group">
					<label>Description</label>
					<textarea bind:value={formDescription} rows="4"></textarea>
				</div>
				<div class="form-group">
					<label>Status</label>
					<select bind:value={formStatus}>
						<option value="Active">Active</option>
						<option value="OnHold">On Hold</option>
						<option value="Completed">Completed</option>
						<option value="Archived">Archived</option>
					</select>
				</div>
				<div class="form-group">
					<label>Team Members</label>
					<div class="checkbox-list">
						{#each contacts as contact}
							<label>
								<input type="checkbox" checked={formContacts.includes(contact.id)} on:change={() => toggleContact(contact.id)} />
								{contact.first_name} {contact.last_name || ''}
							</label>
						{/each}
					</div>
				</div>
				<div class="form-group">
					<label>Tags</label>
					<div class="checkbox-list">
						{#each tags as tag}
							<label>
								<input type="checkbox" checked={formTags.includes(tag.id)} on:change={() => toggleTag(tag.id)} />
								{tag.name}
							</label>
						{/each}
					</div>
				</div>
				<div class="form-actions">
					<button type="button" on:click={() => showModal = false} class="btn">Cancel</button>
					<button type="submit" class="btn btn-primary">Save</button>
				</div>
			</form>
		</div>
	</div>
{/if}

<style>
	.page { padding: 2rem; max-width: 1200px; }
	.header { display: flex; justify-content: space-between; margin-bottom: 2rem; }
	.filters { display: flex; gap: 0.5rem; margin-bottom: 2rem; flex-wrap: wrap; }
	.filters button { padding: 0.5rem 1rem; border: 1px solid var(--gray-300); background: white; border-radius: 6px; cursor: pointer; }
	.filters button.active { background: var(--primary); color: white; }
	.projects-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(400px, 1fr)); gap: 1.5rem; }
	.project-card { padding: 1.5rem; }
	.project-header { display: flex; justify-content: space-between; margin-bottom: 1rem; }
	.project-header h3 { margin: 0 0 0.5rem; }
	.status-badge { padding: 0.25rem 0.75rem; border-radius: 12px; font-size: 0.75rem; }
	.status-active { background: #d1fae5; color: #065f46; }
	.status-completed { background: #dbeafe; color: #1e40af; }
	.status-archived { background: #f3f4f6; color: #4b5563; }
	.status-onhold { background: #fed7aa; color: #92400e; }
	.actions { display: flex; gap: 0.5rem; }
	.description { color: var(--gray-700); margin-bottom: 1rem; }
	.project-meta { display: flex; gap: 1rem; font-size: 0.875rem; color: var(--gray-600); }
	.loading, .empty { text-align: center; padding: 3rem; }
	.modal-overlay { position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000; }
	.modal { background: white; padding: 2rem; border-radius: 8px; width: 90%; max-width: 600px; max-height: 80vh; overflow-y: auto; }
	.form-group { margin-bottom: 1.5rem; }
	.form-group label { display: block; margin-bottom: 0.5rem; font-weight: 500; }
	.form-group input, .form-group textarea, .form-group select { width: 100%; padding: 0.75rem; border: 1px solid var(--gray-300); border-radius: 6px; }
	.checkbox-list { max-height: 200px; overflow-y: auto; border: 1px solid var(--gray-300); border-radius: 6px; padding: 0.5rem; }
	.checkbox-list label { display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem; cursor: pointer; }
	.checkbox-list label:hover { background: var(--gray-50); }
	.form-actions { display: flex; justify-content: flex-end; gap: 1rem; margin-top: 2rem; }
	.btn-sm { padding: 0.375rem 0.75rem; font-size: 0.875rem; }
</style>
