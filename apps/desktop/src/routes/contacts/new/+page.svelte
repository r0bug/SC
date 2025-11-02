<script lang="ts">
	import { goto } from '$app/navigation';
	import { tauriApi } from '$lib/api/tauri-api';

	let firstName = '';
	let lastName = '';
	let email = '';
	let phone = '';
	let organization = '';
	let title = '';
	let notes = '';

	let creating = false;
	let error = '';

	async function handleSubmit() {
		if (!firstName.trim()) {
			error = 'First name is required';
			return;
		}

		try {
			creating = true;
			error = '';

			await tauriApi.createContact({
				first_name: firstName,
				last_name: lastName || null,
				email: email || null,
				phone: phone || null,
				organization: organization || null,
				title: title || null,
				notes: notes || null
			});

			// Redirect to contacts list
			goto('/contacts');
		} catch (err: any) {
			error = err.message || 'Failed to create contact';
		} finally {
			creating = false;
		}
	}
</script>

<div class="page">
	<div class="header">
		<h1>Add New Contact</h1>
		<a href="/contacts" class="btn">Cancel</a>
	</div>

	{#if error}
		<div class="error">{error}</div>
	{/if}

	<form on:submit|preventDefault={handleSubmit} class="card">
		<div class="form-grid">
			<div class="form-group">
				<label for="firstName">First Name *</label>
				<input
					id="firstName"
					type="text"
					bind:value={firstName}
					placeholder="John"
					required
				/>
			</div>

			<div class="form-group">
				<label for="lastName">Last Name</label>
				<input
					id="lastName"
					type="text"
					bind:value={lastName}
					placeholder="Doe"
				/>
			</div>

			<div class="form-group">
				<label for="email">Email</label>
				<input
					id="email"
					type="email"
					bind:value={email}
					placeholder="john.doe@example.com"
				/>
			</div>

			<div class="form-group">
				<label for="phone">Phone</label>
				<input
					id="phone"
					type="tel"
					bind:value={phone}
					placeholder="+1234567890"
				/>
			</div>

			<div class="form-group">
				<label for="organization">Organization</label>
				<input
					id="organization"
					type="text"
					bind:value={organization}
					placeholder="Company Name"
				/>
			</div>

			<div class="form-group">
				<label for="title">Title</label>
				<input
					id="title"
					type="text"
					bind:value={title}
					placeholder="Job Title"
				/>
			</div>
		</div>

		<div class="form-group full-width">
			<label for="notes">Notes</label>
			<textarea
				id="notes"
				bind:value={notes}
				placeholder="Additional notes..."
				rows="4"
			></textarea>
		</div>

		<div class="form-actions">
			<button type="submit" class="btn btn-primary" disabled={creating}>
				{creating ? 'Creating...' : 'Create Contact'}
			</button>
		</div>
	</form>
</div>

<style>
	.page { padding: 2rem; max-width: 900px; margin: 0 auto; }
	.header { display: flex; justify-items: space-between; align-items: center; margin-bottom: 2rem; }
	.header h1 { flex: 1; margin: 0; }

	.card {
		background: white;
		border-radius: 8px;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
		padding: 2rem;
	}

	.form-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
		gap: 1.5rem;
		margin-bottom: 1.5rem;
	}

	.form-group { display: flex; flex-direction: column; }
	.form-group.full-width { grid-column: 1 / -1; }

	.form-group label {
		font-weight: 500;
		margin-bottom: 0.5rem;
		color: var(--gray-700);
	}

	.form-group input,
	.form-group textarea {
		padding: 0.75rem;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		font-size: 1rem;
	}

	.form-group input:focus,
	.form-group textarea:focus {
		outline: none;
		border-color: var(--primary);
		ring: 2px solid var(--primary-light);
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
