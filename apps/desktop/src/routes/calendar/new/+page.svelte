<script lang="ts">
	import { goto } from '$app/navigation';
	import { tauriApi } from '$lib/api/tauri-api';

	let title = '';
	let description = '';
	let location = '';
	let startDate = '';
	let startTime = '';
	let endDate = '';
	let endTime = '';

	let creating = false;
	let error = '';

	// Set default to today
	const today = new Date().toISOString().split('T')[0];
	startDate = today;
	endDate = today;

	async function handleSubmit() {
		if (!title.trim()) {
			error = 'Event title is required';
			return;
		}

		if (!startDate || !startTime) {
			error = 'Start date and time are required';
			return;
		}

		try {
			creating = true;
			error = '';

			const startDateTime = new Date(`${startDate}T${startTime}`).toISOString();
			const endDateTime = endDate && endTime ? new Date(`${endDate}T${endTime}`).toISOString() : null;

			await tauriApi.createEvent({
				title,
				description: description || null,
				location: location || null,
				start_time: startDateTime,
				end_time: endDateTime,
				all_day: false
			});

			// Redirect to calendar
			goto('/calendar');
		} catch (err: any) {
			error = err.message || 'Failed to create event';
		} finally {
			creating = false;
		}
	}
</script>

<div class="page">
	<div class="header">
		<h1>Schedule New Event</h1>
		<a href="/calendar" class="btn">Cancel</a>
	</div>

	{#if error}
		<div class="error">{error}</div>
	{/if}

	<form on:submit|preventDefault={handleSubmit} class="card">
		<div class="form-group">
			<label for="title">Event Title *</label>
			<input
				id="title"
				type="text"
				bind:value={title}
				placeholder="Meeting with..."
				required
			/>
		</div>

		<div class="form-grid">
			<div class="form-group">
				<label for="startDate">Start Date *</label>
				<input
					id="startDate"
					type="date"
					bind:value={startDate}
					required
				/>
			</div>

			<div class="form-group">
				<label for="startTime">Start Time *</label>
				<input
					id="startTime"
					type="time"
					bind:value={startTime}
					required
				/>
			</div>

			<div class="form-group">
				<label for="endDate">End Date</label>
				<input
					id="endDate"
					type="date"
					bind:value={endDate}
				/>
			</div>

			<div class="form-group">
				<label for="endTime">End Time</label>
				<input
					id="endTime"
					type="time"
					bind:value={endTime}
				/>
			</div>
		</div>

		<div class="form-group">
			<label for="location">Location</label>
			<input
				id="location"
				type="text"
				bind:value={location}
				placeholder="Office, Zoom, etc."
			/>
		</div>

		<div class="form-group">
			<label for="description">Description</label>
			<textarea
				id="description"
				bind:value={description}
				placeholder="Event details..."
				rows="4"
			></textarea>
		</div>

		<div class="form-actions">
			<button type="submit" class="btn btn-primary" disabled={creating}>
				{creating ? 'Creating...' : 'Create Event'}
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

	.form-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 1rem;
		margin-bottom: 1.5rem;
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
