<script lang="ts">
	import { onMount } from 'svelte';
	import { tauriApi } from '$lib/api/tauri-api';
	import type { CalendarEvent } from '$lib/api/types';

	let events: CalendarEvent[] = [];
	let loading = true;
	let sortBy: 'date' | 'title' | 'upcoming' = 'date';

	onMount(async () => {
		await loadEvents();
	});

	async function loadEvents() {
		try {
			loading = true;
			events = await tauriApi.getCalendarEvents();
			sortEvents();
		} catch (error) {
			console.error('Failed to load events:', error);
		} finally {
			loading = false;
		}
	}

	async function deleteEvent(id: string, title: string) {
		if (!confirm(`Are you sure you want to delete the event "${title}"?`)) return;

		try {
			await tauriApi.deleteEvent(id);
			events = events.filter(e => e.id !== id);
			await tauriApi.showNotification('Event Deleted', `${title} has been deleted successfully`);
		} catch (error) {
			console.error('Failed to delete event:', error);
			alert('Failed to delete event');
		}
	}

	function sortEvents() {
		switch (sortBy) {
			case 'date':
				events = events.sort((a, b) => new Date(a.start_time).getTime() - new Date(b.start_time).getTime());
				break;
			case 'title':
				events = events.sort((a, b) => a.title.localeCompare(b.title));
				break;
			case 'upcoming':
				const now = new Date().getTime();
				events = events.sort((a, b) => {
					const aTime = new Date(a.start_time).getTime();
					const bTime = new Date(b.start_time).getTime();
					const aDiff = Math.abs(aTime - now);
					const bDiff = Math.abs(bTime - now);
					return aDiff - bDiff;
				});
				break;
		}
	}

	function handleSortChange() {
		sortEvents();
	}

	function formatDateTime(dateStr: string, allDay: boolean): string {
		const date = new Date(dateStr);
		if (allDay) {
			return date.toLocaleDateString();
		}
		return date.toLocaleString();
	}

	function formatTimeRange(start: string, end: string, allDay: boolean): string {
		if (allDay) {
			return 'All day';
		}
		const startTime = new Date(start).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
		const endTime = new Date(end).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
		return `${startTime} - ${endTime}`;
	}

	function isUpcoming(startTime: string): boolean {
		return new Date(startTime) > new Date();
	}

	function isPast(endTime: string): boolean {
		return new Date(endTime) < new Date();
	}
</script>

<div class="page">
	<div class="header">
		<div>
			<h1>Calendar</h1>
			<p class="subtitle">View and manage your events and meetings</p>
		</div>
		<div class="header-actions">
			<select bind:value={sortBy} on:change={handleSortChange} class="sort-select">
				<option value="date">Sort by Date</option>
				<option value="title">Sort by Title</option>
				<option value="upcoming">Sort by Upcoming</option>
			</select>
		</div>
	</div>

	{#if loading}
		<div class="loading">Loading events...</div>
	{:else if events.length === 0}
		<div class="empty card">
			<p>No events scheduled.</p>
			<p>Calendar events help you stay organized and connected with your contacts.</p>
		</div>
	{:else}
		<div class="events-list">
			{#each events as event}
				<div class="event-card card {isPast(event.end_time) ? 'event-past' : isUpcoming(event.start_time) ? 'event-upcoming' : 'event-current'}">
					<div class="event-main">
						<div class="event-header">
							<h3>{event.title}</h3>
							<div class="event-actions">
								<span class="time-badge">{formatTimeRange(event.start_time, event.end_time, event.all_day)}</span>
								<button
									on:click={() => deleteEvent(event.id, event.title)}
									class="btn-icon btn-danger"
									title="Delete event"
								>
									🗑️
								</button>
							</div>
						</div>

						<div class="event-date">
							<span class="date-icon">📅</span>
							<span>{formatDateTime(event.start_time, event.all_day)}</span>
						</div>

						{#if event.description}
							<p class="event-description">{event.description}</p>
						{/if}

						<div class="event-details">
							{#if event.location}
								<div class="detail-item">
									<span class="detail-icon">📍</span>
									<span>{event.location}</span>
								</div>
							{/if}
							{#if event.contacts && event.contacts.length > 0}
								<div class="detail-item">
									<span class="detail-icon">👥</span>
									<span>{event.contacts.length} attendee{event.contacts.length > 1 ? 's' : ''}</span>
								</div>
							{/if}
							{#if event.reminders && event.reminders.length > 0}
								<div class="detail-item">
									<span class="detail-icon">🔔</span>
									<span>{event.reminders.length} reminder{event.reminders.length > 1 ? 's' : ''}</span>
								</div>
							{/if}
							{#if event.attachment_ids && event.attachment_ids.length > 0}
								<div class="detail-item">
									<span class="detail-icon">📎</span>
									<span>{event.attachment_ids.length} attachment{event.attachment_ids.length > 1 ? 's' : ''}</span>
								</div>
							{/if}
						</div>

						{#if event.recurrence}
							<div class="recurrence-badge">
								🔁 Recurring ({event.recurrence.frequency})
							</div>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page {
		padding: 2rem;
		max-width: 1000px;
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

	.events-list {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.event-card {
		padding: 1.5rem;
		transition: transform 0.2s, box-shadow 0.2s;
		border-left: 4px solid var(--gray-300);
	}

	.event-upcoming {
		border-left-color: #3b82f6;
	}

	.event-current {
		border-left-color: #10b981;
	}

	.event-past {
		opacity: 0.6;
		border-left-color: var(--gray-400);
	}

	.event-card:hover {
		transform: translateX(4px);
		box-shadow: var(--shadow-md);
	}

	.event-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: 0.75rem;
	}

	.event-header h3 {
		margin: 0;
		font-size: 1.25rem;
		color: var(--gray-900);
		flex: 1;
	}

	.event-actions {
		display: flex;
		gap: 0.5rem;
		align-items: center;
	}

	.time-badge {
		background: var(--primary-light);
		color: var(--primary);
		padding: 0.25rem 0.75rem;
		border-radius: 999px;
		font-size: 0.75rem;
		font-weight: 600;
		white-space: nowrap;
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

	.event-date {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin-bottom: 0.75rem;
		color: var(--gray-700);
		font-weight: 500;
	}

	.date-icon {
		font-size: 1.125rem;
	}

	.event-description {
		margin: 0 0 1rem;
		color: var(--gray-700);
		font-size: 0.875rem;
		line-height: 1.5;
	}

	.event-details {
		display: flex;
		flex-wrap: wrap;
		gap: 1rem;
		margin-bottom: 0.5rem;
	}

	.detail-item {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.875rem;
		color: var(--gray-600);
	}

	.detail-icon {
		font-size: 1rem;
	}

	.recurrence-badge {
		display: inline-flex;
		align-items: center;
		gap: 0.5rem;
		margin-top: 0.75rem;
		padding: 0.375rem 0.75rem;
		background: #fef3c7;
		color: #92400e;
		border-radius: 6px;
		font-size: 0.75rem;
		font-weight: 600;
	}

	.card {
		background: white;
		border-radius: 8px;
		box-shadow: var(--shadow-sm);
	}
</style>
