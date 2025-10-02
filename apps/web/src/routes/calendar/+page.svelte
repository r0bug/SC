<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { CalendarEvent } from '$lib/api/types';

	let events: CalendarEvent[] = [];
	let loading = true;
	let currentDate = new Date();
	let view: 'month' | 'week' = 'month';

	onMount(async () => {
		await loadEvents();
	});

	async function loadEvents() {
		try {
			loading = true;
			const start = new Date(currentDate.getFullYear(), currentDate.getMonth(), 1);
			const end = new Date(currentDate.getFullYear(), currentDate.getMonth() + 1, 0);
			events = await api.getEvents(start.toISOString(), end.toISOString());
		} catch (error) {
			console.error('Failed to load events:', error);
		} finally {
			loading = false;
		}
	}

	function getMonthDays() {
		const year = currentDate.getFullYear();
		const month = currentDate.getMonth();
		const firstDay = new Date(year, month, 1);
		const lastDay = new Date(year, month + 1, 0);
		const days = [];

		// Add padding for days before month starts
		const startPadding = firstDay.getDay();
		for (let i = 0; i < startPadding; i++) {
			days.push(null);
		}

		// Add all days of month
		for (let i = 1; i <= lastDay.getDate(); i++) {
			days.push(new Date(year, month, i));
		}

		return days;
	}

	function getEventsForDay(date: Date): CalendarEvent[] {
		if (!date) return [];
		return events.filter(event => {
			const eventDate = new Date(event.start_time);
			return eventDate.getDate() === date.getDate() &&
				   eventDate.getMonth() === date.getMonth() &&
				   eventDate.getFullYear() === date.getFullYear();
		});
	}

	function previousMonth() {
		currentDate = new Date(currentDate.getFullYear(), currentDate.getMonth() - 1);
		loadEvents();
	}

	function nextMonth() {
		currentDate = new Date(currentDate.getFullYear(), currentDate.getMonth() + 1);
		loadEvents();
	}

	function formatTime(dateStr: string): string {
		return new Date(dateStr).toLocaleTimeString('en-US', {
			hour: 'numeric',
			minute: '2-digit'
		});
	}
</script>

<div class="page">
	<div class="header">
		<div>
			<h1>Calendar</h1>
			<p class="subtitle">{currentDate.toLocaleDateString('en-US', { month: 'long', year: 'numeric' })}</p>
		</div>
		<div class="header-actions">
			<div class="view-toggle">
				<button class:active={view === 'month'} on:click={() => view = 'month'}>
					Month
				</button>
				<button class:active={view === 'week'} on:click={() => view = 'week'}>
					Week
				</button>
			</div>
			<a href="/calendar/new" class="btn btn-primary">+ New Event</a>
		</div>
	</div>

	<div class="calendar card">
		<div class="calendar-nav">
			<button on:click={previousMonth} class="btn btn-secondary">← Previous</button>
			<button on:click={() => currentDate = new Date()} class="btn btn-secondary">Today</button>
			<button on:click={nextMonth} class="btn btn-secondary">Next →</button>
		</div>

		{#if loading}
			<div class="loading">Loading calendar...</div>
		{:else if view === 'month'}
			<div class="month-view">
				<div class="weekdays">
					{#each ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'] as day}
						<div class="weekday">{day}</div>
					{/each}
				</div>
				<div class="days-grid">
					{#each getMonthDays() as day}
						<div class="day-cell" class:empty={!day}>
							{#if day}
								<div class="day-number">{day.getDate()}</div>
								<div class="day-events">
									{#each getEventsForDay(day).slice(0, 3) as event}
										<a href="/calendar/{event.id}" class="event-item">
											<span class="event-time">{formatTime(event.start_time)}</span>
											<span class="event-title">{event.title}</span>
										</a>
									{/each}
									{#if getEventsForDay(day).length > 3}
										<div class="more-events">
											+{getEventsForDay(day).length - 3} more
										</div>
									{/if}
								</div>
							{/if}
						</div>
					{/each}
				</div>
			</div>
		{:else}
			<div class="week-view">
				<!-- Week view implementation would go here -->
				<p class="text-center">Week view coming soon...</p>
			</div>
		{/if}
	</div>
</div>

<style>
	.page {
		padding: 2rem;
		max-width: 1400px;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 2rem;
	}

	.header-actions {
		display: flex;
		gap: 1rem;
		align-items: center;
	}

	.view-toggle {
		display: flex;
		background: white;
		border-radius: 6px;
		border: 1px solid var(--gray-300);
	}

	.view-toggle button {
		padding: 0.5rem 1rem;
		background: transparent;
		border: none;
		cursor: pointer;
		transition: all 0.2s;
	}

	.view-toggle button.active {
		background: var(--primary);
		color: white;
	}

	.calendar {
		padding: 1.5rem;
		min-height: 600px;
	}

	.calendar-nav {
		display: flex;
		justify-content: center;
		gap: 1rem;
		margin-bottom: 2rem;
	}

	.weekdays {
		display: grid;
		grid-template-columns: repeat(7, 1fr);
		gap: 1px;
		background: var(--gray-200);
		margin-bottom: 1px;
	}

	.weekday {
		background: white;
		padding: 1rem;
		text-align: center;
		font-weight: 600;
		color: var(--gray-700);
		font-size: 0.875rem;
	}

	.days-grid {
		display: grid;
		grid-template-columns: repeat(7, 1fr);
		gap: 1px;
		background: var(--gray-200);
	}

	.day-cell {
		background: white;
		min-height: 120px;
		padding: 0.5rem;
		position: relative;
	}

	.day-cell.empty {
		background: var(--gray-50);
	}

	.day-number {
		font-weight: 600;
		margin-bottom: 0.5rem;
	}

	.day-events {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.event-item {
		display: flex;
		gap: 0.5rem;
		padding: 0.25rem;
		background: var(--primary-light);
		border-radius: 4px;
		text-decoration: none;
		color: var(--primary);
		font-size: 0.75rem;
		overflow: hidden;
	}

	.event-item:hover {
		background: var(--primary);
		color: white;
	}

	.event-time {
		font-weight: 600;
		white-space: nowrap;
	}

	.event-title {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.more-events {
		font-size: 0.75rem;
		color: var(--gray-600);
		padding: 0.25rem;
	}

	.loading {
		text-align: center;
		padding: 3rem;
	}

	.text-center {
		text-align: center;
	}

	.subtitle {
		color: var(--gray-600);
		margin-top: 0.25rem;
	}
</style>
