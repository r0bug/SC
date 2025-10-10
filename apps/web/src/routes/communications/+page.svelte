<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';
	import type { CommunicationAttempt } from '$lib/api/types';

	let communications: CommunicationAttempt[] = [];
	let loading = true;

	onMount(async () => {
		await loadCommunications();
		api.onWebSocketMessage('communication_updated', (data) => {
			communications = communications.map(c => c.id === data.id ? data : c);
		});
	});

	async function loadCommunications() {
		try {
			communications = await api.getCommunications();
		} catch (error) {
			console.error('Failed to load communications:', error);
		} finally {
			loading = false;
		}
	}

	function getStatusBadge(status: any): string {
		if (typeof status === 'string') {
			switch (status) {
				case 'Sent': return 'status-sent';
				case 'Pending': return 'status-pending';
				case 'Retrying': return 'status-retrying';
				default: return 'status-default';
			}
		}
		return 'status-failed';
	}
</script>

<div class="page">
	<h1>Communications</h1>

	<div class="notice">
		⚠️ Alpha: Email/SMS/Social sends are mocked for testing
	</div>

	{#if loading}
		<div>Loading...</div>
	{:else}
		<div class="comms-list">
			{#each communications as comm}
				<div class="comm-card card">
					<div class="comm-header">
						<span class="method">{typeof comm.method === 'string' ? comm.method : 'Social'}</span>
						<span class="badge {getStatusBadge(comm.status)}">
							{typeof comm.status === 'string' ? comm.status : 'Failed'}
						</span>
					</div>
					{#if comm.subject}
						<h3>{comm.subject}</h3>
					{/if}
					<p class="message">{comm.message}</p>
					<div class="meta">
						<span>Attempts: {comm.retry_count}</span>
						{#if comm.scheduled_at}
							<span>Scheduled: {new Date(comm.scheduled_at).toLocaleString()}</span>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page { padding: 2rem; max-width: 1000px; }
	.notice { padding: 1rem; background: #fef3c7; border: 1px solid #fde68a; border-radius: 6px; margin-bottom: 2rem; }
	.comms-list { display: flex; flex-direction: column; gap: 1rem; }
	.comm-card { padding: 1.5rem; }
	.comm-header { display: flex; justify-content: space-between; margin-bottom: 0.5rem; }
	.method { font-weight: 600; }
	.badge { padding: 0.25rem 0.75rem; border-radius: 12px; font-size: 0.75rem; }
	.status-sent { background: #d1fae5; color: #065f46; }
	.status-pending { background: #fed7aa; color: #92400e; }
	.status-retrying { background: #dbeafe; color: #1e40af; }
	.status-failed { background: #fee2e2; color: #991b1b; }
	.message { color: var(--gray-700); margin: 0.5rem 0; }
	.meta { font-size: 0.875rem; color: var(--gray-600); display: flex; gap: 1rem; }
</style>
