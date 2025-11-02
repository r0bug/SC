<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { api } from '$lib/api/api';
	import type { CalendarEvent } from '$lib/api/types';

	let eventId: string;
	let event: CalendarEvent | null = null;
	let loading = true;
	let error = '';

	$: eventId = $page.params.id;

	onMount(async () => {
		await loadEvent();
	});

	async function loadEvent() {
		try {
			loading = true;
			error = '';
			event = await api.getEvent(eventId);
		} catch (err: any) {
			console.error('Failed to load event:', err);
			error = 'Failed to load event details';
		} finally {
			loading = false;
		}
	}

	async function deleteEvent() {
		if (!event) return;
		if (!confirm(`Are you sure you want to delete "${event.title}"?`)) return;

		try {
			await api.deleteEvent(eventId);
			goto('/calendar');
		} catch (err: any) {
			console.error('Failed to delete event:', err);
			alert('Failed to delete event');
		}
	}

	function formatDateTime(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleString('en-US', {
			weekday: 'long',
			year: 'numeric',
			month: 'long',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}
</script>

<div class="page">
	{#if loading}
		<div class="loading">Loading event details...</div>
	{:else if error}
		<div class="error card">
			<p>{error}</p>
			<a href="/calendar" class="btn btn-primary">Back to Calendar</a>
		</div>
	{:else if event}
		<div class="header">
			<h1>{event.title}</h1>
			<div class="header-actions">
				<a href="/calendar" class="btn">Back</a>
				<button on:click={deleteEvent} class="btn btn-danger">Delete Event</button>
			</div>
		</div>

		<div class="event-details card">
			<div class="detail-section">
				<h3>When</h3>
				<p class="detail-value">
					<strong>Start:</strong> {formatDateTime(event.start_time)}
				</p>
				{#if event.end_time}
					<p class="detail-value">
						<strong>End:</strong> {formatDateTime(event.end_time)}
					</p>
				{/if}
				{#if event.all_day}
					<p class="badge">All Day Event</p>
				{/if}
			</div>

			{#if event.location}
				<div class="detail-section">
					<h3>Location</h3>
					<p class="detail-value">{event.location}</p>
				</div>
			{/if}

			{#if event.description}
				<div class="detail-section">
					<h3>Description</h3>
					<p class="detail-value description">{event.description}</p>
				</div>
			{/if}

			<div class="detail-section">
				<h3>Event Details</h3>
				<p class="detail-value meta">
					Created: {new Date(event.created_at).toLocaleDateString()}
				</p>
			</div>
		</div>
	{/if}
</div>

<style>
	.page {
		padding: 2rem;
		max-width: 900px;
		margin: 0 auto;
	}

	.loading, .error {
		text-align: center;
		padding: 3rem;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 2rem;
		gap: 2rem;
	}

	.header h1 {
		margin: 0;
		font-size: 2rem;
		color: var(--gray-900);
	}

	.header-actions {
		display: flex;
		gap: 1rem;
	}

	.card {
		background: white;
		border-radius: 8px;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
		padding: 2rem;
	}

	.event-details {
		display: flex;
		flex-direction: column;
		gap: 2rem;
	}

	.detail-section {
		border-bottom: 1px solid var(--gray-200);
		padding-bottom: 1.5rem;
	}

	.detail-section:last-child {
		border-bottom: none;
		padding-bottom: 0;
	}

	.detail-section h3 {
		margin: 0 0 1rem;
		font-size: 1.125rem;
		color: var(--gray-700);
		text-transform: uppercase;
		letter-spacing: 0.05em;
		font-size: 0.875rem;
		font-weight: 600;
	}

	.detail-value {
		margin: 0.5rem 0;
		color: var(--gray-900);
		font-size: 1rem;
		line-height: 1.6;
	}

	.detail-value.description {
		white-space: pre-wrap;
	}

	.detail-value.meta {
		color: var(--gray-600);
		font-size: 0.875rem;
	}

	.badge {
		display: inline-block;
		padding: 0.375rem 0.75rem;
		background: var(--primary-light);
		color: var(--primary);
		border-radius: 999px;
		font-size: 0.875rem;
		font-weight: 500;
		margin-top: 0.5rem;
	}

	.btn {
		padding: 0.75rem 1.5rem;
		border: none;
		border-radius: 6px;
		cursor: pointer;
		font-weight: 500;
		text-decoration: none;
		display: inline-block;
		background: var(--gray-200);
		color: var(--gray-900);
		transition: background-color 0.2s;
	}

	.btn:hover {
		background: var(--gray-300);
	}

	.btn-primary {
		background: var(--primary);
		color: white;
	}

	.btn-primary:hover {
		background: var(--primary-dark);
	}

	.btn-danger {
		background: var(--color-error);
		color: white;
	}

	.btn-danger:hover {
		background: var(--color-error-dark);
	}
</style>
