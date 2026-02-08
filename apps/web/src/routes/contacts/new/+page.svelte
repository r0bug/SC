<script lang="ts">
	import { goto } from '$app/navigation';
	import { api } from '$lib/api/api';

	let formFirstName = '';
	let formLastName = '';
	let formEmail = '';
	let formPhone = '';
	let formOrganization = '';
	let formTitle = '';
	let formNotes = '';
	let formTags: string[] = [];
	let formSocialHandles: { platform: string; handle: string }[] = [];
	let saving = false;

	function addSocialHandle() {
		formSocialHandles = [...formSocialHandles, { platform: 'Twitter', handle: '' }];
	}

	function removeSocialHandle(index: number) {
		formSocialHandles = formSocialHandles.filter((_, i) => i !== index);
	}

	function addTag() {
		const tag = prompt('Enter tag:');
		if (tag && tag.trim()) {
			formTags = [...formTags, tag.trim()];
		}
	}

	function removeTag(tag: string) {
		formTags = formTags.filter(t => t !== tag);
	}

	async function handleSubmit() {
		if (!formFirstName.trim()) {
			alert('First name is required');
			return;
		}

		try {
			saving = true;
			const contact = await api.createContact({
				first_name: formFirstName,
				last_name: formLastName || undefined,
				email: formEmail || undefined,
				phone: formPhone || undefined,
				organization: formOrganization || undefined,
				title: formTitle || undefined,
				notes: formNotes || undefined,
				tags: formTags,
				social_handles: formSocialHandles.filter(sh => sh.handle.trim()).map(sh => ({
					platform: sh.platform,
					handle: sh.handle
				})),
				projects: [],
				groups: []
			});

			goto(`/contacts/${contact.id}`);
		} catch (error: unknown) {
			alert('Failed to create contact: ' + (error instanceof Error ? error.message : String(error)));
		} finally {
			saving = false;
		}
	}

	function handleCancel() {
		if (confirm('Discard changes?')) {
			goto('/contacts');
		}
	}
</script>

<div class="page">
	<div class="header">
		<h1>Create Contact</h1>
	</div>

	<div class="card">
		<form on:submit|preventDefault={handleSubmit}>
			<div class="form-group">
				<label for="new-first-name">First Name *</label>
				<input id="new-first-name" type="text" bind:value={formFirstName} required />
			</div>

			<div class="form-group">
				<label for="new-last-name">Last Name</label>
				<input id="new-last-name" type="text" bind:value={formLastName} />
			</div>

			<div class="form-group">
				<label for="new-email">Email</label>
				<input id="new-email" type="email" bind:value={formEmail} />
			</div>

			<div class="form-group">
				<label for="new-phone">Phone</label>
				<input id="new-phone" type="tel" bind:value={formPhone} />
			</div>

			<div class="form-group">
				<label for="new-organization">Organization</label>
				<input id="new-organization" type="text" bind:value={formOrganization} />
			</div>

			<div class="form-group">
				<label for="new-title">Title</label>
				<input id="new-title" type="text" bind:value={formTitle} />
			</div>

			<div class="form-group">
				<label for="new-notes">Notes</label>
				<textarea id="new-notes" bind:value={formNotes} rows="4"></textarea>
			</div>

			<div class="form-group">
				<span class="form-label-text">Social Handles</span>
				{#each formSocialHandles as handle, i}
					<div class="social-handle-row">
						<select bind:value={handle.platform}>
							<option value="Twitter">Twitter</option>
							<option value="LinkedIn">LinkedIn</option>
							<option value="Facebook">Facebook</option>
							<option value="Instagram">Instagram</option>
							<option value="Facebook">Facebook</option>
							<option value="Instagram">Instagram</option>
							<option value="GitHub">GitHub</option>
						</select>
						<input type="text" bind:value={handle.handle} placeholder="@username" />
						<button type="button" on:click={() => removeSocialHandle(i)} class="btn btn-sm btn-danger">Remove</button>
					</div>
				{/each}
				<button type="button" on:click={addSocialHandle} class="btn btn-sm btn-secondary">Add Social Handle</button>
			</div>

			<div class="form-group">
				<span class="form-label-text">Tags</span>
				<div class="tags">
					{#each formTags as tag}
						<span class="tag">
							{tag}
							<button type="button" on:click={() => removeTag(tag)} class="tag-remove">×</button>
						</span>
					{/each}
					<button type="button" on:click={addTag} class="btn btn-sm btn-secondary">Add Tag</button>
				</div>
			</div>

			<div class="form-actions">
				<button type="submit" class="btn btn-primary" disabled={saving}>
					{saving ? 'Creating...' : 'Create Contact'}
				</button>
				<button type="button" on:click={handleCancel} class="btn btn-secondary" disabled={saving}>
					Cancel
				</button>
			</div>
		</form>
	</div>
</div>

<style>
	.page { padding: 2rem; max-width: 800px; margin: 0 auto; }
	.header { margin-bottom: 2rem; }
	.card { padding: 2rem; background: white; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }

	.form-group { margin-bottom: 1.5rem; }
	.form-group label { display: block; margin-bottom: 0.5rem; font-weight: 500; }
	.form-label-text { display: block; margin-bottom: 0.5rem; font-weight: 500; }
	.form-group input, .form-group select, .form-group textarea {
		width: 100%;
		padding: 0.75rem;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		font-size: 1rem;
	}
	.form-group textarea { resize: vertical; font-family: inherit; }

	.social-handle-row { display: flex; gap: 0.5rem; margin-bottom: 0.5rem; align-items: center; }
	.social-handle-row select { width: 140px; }
	.social-handle-row input { flex: 1; }

	.tags { display: flex; flex-wrap: wrap; gap: 0.5rem; align-items: center; }
	.tag { display: inline-flex; align-items: center; gap: 0.25rem; padding: 0.25rem 0.75rem; background: var(--gray-100); border-radius: 12px; font-size: 0.875rem; }
	.tag-remove { background: none; border: none; cursor: pointer; font-size: 1.25rem; line-height: 1; padding: 0; color: var(--gray-600); }
	.tag-remove:hover { color: var(--gray-900); }

	.form-actions { display: flex; gap: 1rem; margin-top: 2rem; }

	.btn { padding: 0.75rem 1.5rem; border: none; border-radius: 6px; cursor: pointer; font-weight: 500; }
	.btn:disabled { opacity: 0.5; cursor: not-allowed; }
	.btn-primary { background: var(--primary); color: white; }
	.btn-secondary { background: var(--gray-200); color: var(--gray-800); }
	.btn-danger { background: #dc2626; color: white; }
	.btn-sm { padding: 0.375rem 0.75rem; font-size: 0.875rem; }
</style>
