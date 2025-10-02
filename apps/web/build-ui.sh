#!/bin/bash
# Comprehensive UI builder script for Phase 5

echo "Building comprehensive SvelteKit web app..."

# Create all route structures
mkdir -p src/routes/{auth/signup,dashboard,contacts/\[id\],groups/\[id\],concepts/\[id\],projects/\[id\],calendar,notes/\[id\],communications,shares,import,settings,search}
mkdir -p src/lib/components/{ui,layout,forms,calendar}
mkdir -p src/lib/stores

# Create navigation layout
cat > src/routes/+layout.svelte << 'EOF'
<script lang="ts">
	import '../app.css';
	import { page } from '$app/stores';
	import { auth, isAuthenticated } from '$lib/stores/auth';
	import { onMount } from 'svelte';

	const publicRoutes = ['/auth/login', '/auth/signup'];

	$: isPublicRoute = publicRoutes.includes($page.url.pathname);
	$: showNav = $isAuthenticated && !isPublicRoute;

	onMount(() => {
		auth.checkAuth();
	});
</script>

{#if showNav}
	<div class="app-layout">
		<nav class="sidebar">
			<div class="logo">
				<h2>📇 SagensContact</h2>
			</div>
			<div class="nav-section">
				<h3>Main</h3>
				<a href="/dashboard" class:active={$page.url.pathname === '/dashboard'}>
					🏠 Dashboard
				</a>
				<a href="/contacts" class:active={$page.url.pathname.startsWith('/contacts')}>
					👥 Contacts
				</a>
				<a href="/groups" class:active={$page.url.pathname.startsWith('/groups')}>
					👫 Groups
				</a>
				<a href="/projects" class:active={$page.url.pathname.startsWith('/projects')}>
					📁 Projects
				</a>
				<a href="/calendar" class:active={$page.url.pathname.startsWith('/calendar')}>
					📅 Calendar
				</a>
				<a href="/notes" class:active={$page.url.pathname.startsWith('/notes')}>
					📝 Notes
				</a>
			</div>
			<div class="nav-section">
				<h3>Engage</h3>
				<a href="/communications" class:active={$page.url.pathname.startsWith('/communications')}>
					📧 Communications
				</a>
				<a href="/shares" class:active={$page.url.pathname.startsWith('/shares')}>
					🤝 Shares
				</a>
				<a href="/import" class:active={$page.url.pathname.startsWith('/import')}>
					📥 Import
				</a>
			</div>
			<div class="nav-section">
				<h3>System</h3>
				<a href="/settings" class:active={$page.url.pathname.startsWith('/settings')}>
					⚙️ Settings
				</a>
				<a href="/search" class:active={$page.url.pathname.startsWith('/search')}>
					🔍 Search History
				</a>
			</div>
			<div class="nav-bottom">
				<button on:click={() => auth.logout()} class="btn btn-secondary btn-block">
					🚪 Logout
				</button>
			</div>
		</nav>
		<main class="main-content">
			<slot />
		</main>
	</div>
{:else}
	<slot />
{/if}

<style>
	:global(body) {
		margin: 0;
	}

	.app-layout {
		display: flex;
		min-height: 100vh;
	}

	.sidebar {
		width: 260px;
		background: white;
		border-right: 1px solid var(--gray-200);
		display: flex;
		flex-direction: column;
		position: fixed;
		height: 100vh;
		overflow-y: auto;
	}

	.logo {
		padding: 1.5rem;
		border-bottom: 1px solid var(--gray-200);
	}

	.logo h2 {
		margin: 0;
		font-size: 1.5rem;
	}

	.nav-section {
		padding: 1rem;
	}

	.nav-section h3 {
		font-size: 0.75rem;
		text-transform: uppercase;
		color: var(--gray-500);
		margin-bottom: 0.5rem;
		font-weight: 600;
	}

	.nav-section a {
		display: block;
		padding: 0.75rem 1rem;
		color: var(--gray-700);
		text-decoration: none;
		border-radius: 6px;
		margin-bottom: 0.25rem;
		transition: all 0.2s;
	}

	.nav-section a:hover {
		background: var(--gray-100);
		color: var(--primary);
	}

	.nav-section a.active {
		background: var(--primary-light);
		color: var(--primary);
		font-weight: 500;
	}

	.nav-bottom {
		margin-top: auto;
		padding: 1rem;
		border-top: 1px solid var(--gray-200);
	}

	.main-content {
		flex: 1;
		margin-left: 260px;
		background: var(--gray-50);
		min-height: 100vh;
	}

	@media (max-width: 768px) {
		.sidebar {
			transform: translateX(-100%);
		}

		.main-content {
			margin-left: 0;
		}
	}
</style>
EOF

# Create contact list page
cat > src/routes/contacts/+page.svelte << 'EOF'
<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { Contact } from '$lib/api/types';

	let contacts: Contact[] = [];
	let loading = true;
	let searchQuery = '';
	let selectedTags: string[] = [];

	onMount(async () => {
		await loadContacts();
	});

	async function loadContacts() {
		try {
			loading = true;
			contacts = await api.getContacts(100, 0);
		} catch (error) {
			console.error('Failed to load contacts:', error);
		} finally {
			loading = false;
		}
	}

	async function handleSearch() {
		if (!searchQuery.trim() && selectedTags.length === 0) {
			await loadContacts();
			return;
		}

		try {
			contacts = await api.searchContacts(searchQuery, { tags: selectedTags });
		} catch (error) {
			console.error('Search failed:', error);
		}
	}

	async function deleteContact(id: string) {
		if (!confirm('Are you sure you want to delete this contact?')) return;

		try {
			await api.deleteContact(id);
			contacts = contacts.filter(c => c.id !== id);
		} catch (error) {
			console.error('Failed to delete contact:', error);
		}
	}
</script>

<div class="page">
	<div class="header">
		<div>
			<h1>Contacts</h1>
			<p class="subtitle">Manage your contacts and relationships</p>
		</div>
		<a href="/contacts/new" class="btn btn-primary">+ Add Contact</a>
	</div>

	<div class="filters card">
		<input
			type="text"
			placeholder="Search contacts..."
			bind:value={searchQuery}
			on:input={handleSearch}
			class="search-input"
		/>
	</div>

	{#if loading}
		<div class="loading">Loading contacts...</div>
	{:else if contacts.length === 0}
		<div class="empty card">
			<p>No contacts found.</p>
			<a href="/contacts/new" class="btn btn-primary">Create your first contact</a>
		</div>
	{:else}
		<div class="contact-grid">
			{#each contacts as contact}
				<div class="contact-card card">
					<div class="contact-header">
						<h3>{contact.first_name} {contact.last_name || ''}</h3>
						<div class="contact-actions">
							<a href="/contacts/{contact.id}" class="btn btn-sm">Edit</a>
							<button on:click={() => deleteContact(contact.id)} class="btn btn-sm btn-secondary">
								Delete
							</button>
						</div>
					</div>
					{#if contact.organization}
						<p class="contact-field">🏢 {contact.organization}</p>
					{/if}
					{#if contact.email}
						<p class="contact-field">✉️ {contact.email}</p>
					{/if}
					{#if contact.phone}
						<p class="contact-field">📞 {contact.phone}</p>
					{/if}
					{#if contact.tags.length > 0}
						<div class="tags">
							{#each contact.tags as tag}
								<span class="tag">{tag}</span>
							{/each}
						</div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page {
		padding: 2rem;
		max-width: 1200px;
	}

	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 2rem;
	}

	.subtitle {
		color: var(--gray-600);
		margin-top: 0.25rem;
	}

	.filters {
		margin-bottom: 2rem;
		padding: 1.5rem;
	}

	.search-input {
		width: 100%;
		padding: 0.75rem;
		border: 1px solid var(--gray-300);
		border-radius: 6px;
		font-size: 1rem;
	}

	.contact-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
		gap: 1.5rem;
	}

	.contact-card {
		padding: 1.5rem;
	}

	.contact-header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		margin-bottom: 1rem;
	}

	.contact-header h3 {
		margin: 0;
		font-size: 1.25rem;
	}

	.contact-actions {
		display: flex;
		gap: 0.5rem;
	}

	.contact-field {
		margin: 0.5rem 0;
		color: var(--gray-700);
		font-size: 0.875rem;
	}

	.tags {
		display: flex;
		flex-wrap: wrap;
		gap: 0.5rem;
		margin-top: 1rem;
	}

	.tag {
		padding: 0.25rem 0.75rem;
		background: var(--primary-light);
		color: var(--primary);
		border-radius: 12px;
		font-size: 0.75rem;
	}

	.btn-sm {
		padding: 0.375rem 0.75rem;
		font-size: 0.875rem;
	}

	.loading, .empty {
		text-align: center;
		padding: 3rem;
	}
</style>
EOF

# Create calendar view
cat > src/routes/calendar/+page.svelte << 'EOF'
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
EOF

echo "✅ Core UI structure created"
echo "Run 'pnpm dev' to see the enhanced web app"