<script lang="ts">
	import { onMount } from 'svelte';
	import { api } from '$lib/api/api';

	let updateInfo: {
		current_version: string;
		latest_version: string;
		update_available: boolean;
		release_url?: string;
	} | null = null;
	let dismissed = false;
	let loading = true;

	onMount(async () => {
		// Check if notification was dismissed in this session
		const dismissedKey = 'update_notification_dismissed';
		const dismissedSession = sessionStorage.getItem(dismissedKey);
		if (dismissedSession) {
			dismissed = true;
			loading = false;
			return;
		}

		// Check for updates
		try {
			const result = await api.checkUpdates();
			if (result.update_available) {
				updateInfo = result;
			}
		} catch (error) {
			// Quietly log error and continue
			console.log('Update check failed:', error);
		} finally {
			loading = false;
		}
	});

	function dismiss() {
		dismissed = true;
		sessionStorage.setItem('update_notification_dismissed', 'true');
	}
</script>

{#if !loading && !dismissed && updateInfo?.update_available}
	<div class="update-banner">
		<div class="update-content">
			<div class="update-icon">🔄</div>
			<div class="update-text">
				<strong>Update Available</strong>
				<span>
					You're on v{updateInfo.current_version} • v{updateInfo.latest_version} available
				</span>
			</div>
			{#if updateInfo.release_url}
				<a
					href={updateInfo.release_url}
					target="_blank"
					rel="noopener noreferrer"
					class="update-link"
				>
					View Release
				</a>
			{/if}
			<button class="dismiss-btn" on:click={dismiss} aria-label="Dismiss notification">
				✕
			</button>
		</div>
	</div>
{/if}

<style>
	.update-banner {
		position: fixed;
		top: 0;
		left: 260px;
		right: 0;
		background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
		color: white;
		padding: 0.75rem 1.5rem;
		z-index: 1000;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
		animation: slideDown 0.3s ease-out;
	}

	@keyframes slideDown {
		from {
			transform: translateY(-100%);
			opacity: 0;
		}
		to {
			transform: translateY(0);
			opacity: 1;
		}
	}

	.update-content {
		display: flex;
		align-items: center;
		gap: 1rem;
		max-width: 1400px;
		margin: 0 auto;
	}

	.update-icon {
		font-size: 1.5rem;
		animation: rotate 2s linear infinite;
	}

	@keyframes rotate {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}

	.update-text {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		flex: 1;
	}

	.update-text strong {
		font-weight: 600;
		font-size: 0.95rem;
	}

	.update-text span {
		font-size: 0.85rem;
		opacity: 0.9;
	}

	.update-link {
		color: white;
		text-decoration: none;
		padding: 0.5rem 1rem;
		background: rgba(255, 255, 255, 0.2);
		border-radius: 6px;
		font-size: 0.875rem;
		font-weight: 500;
		transition: all 0.2s;
		border: 1px solid rgba(255, 255, 255, 0.3);
	}

	.update-link:hover {
		background: rgba(255, 255, 255, 0.3);
		transform: translateY(-1px);
	}

	.dismiss-btn {
		background: none;
		border: none;
		color: white;
		font-size: 1.25rem;
		cursor: pointer;
		padding: 0.25rem 0.5rem;
		opacity: 0.8;
		transition: opacity 0.2s;
		line-height: 1;
	}

	.dismiss-btn:hover {
		opacity: 1;
	}

	@media (max-width: 768px) {
		.update-banner {
			left: 0;
			padding: 0.5rem 1rem;
		}

		.update-content {
			gap: 0.5rem;
		}

		.update-text {
			font-size: 0.85rem;
		}

		.update-text span {
			font-size: 0.75rem;
		}

		.update-link {
			padding: 0.4rem 0.8rem;
			font-size: 0.8rem;
		}
	}
</style>
