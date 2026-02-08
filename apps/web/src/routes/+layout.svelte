<script lang="ts">
	import '../app.css';
	import { page } from '$app/stores';
	import { auth, isAuthenticated } from '$lib/stores/auth';
	import { onMount } from 'svelte';
	import Toast from '$lib/components/ui/Toast.svelte';
	import UpdateNotification from '$lib/components/ui/UpdateNotification.svelte';

	const publicRoutes = ['/auth/login', '/auth/signup'];

	$: isPublicRoute = publicRoutes.includes($page.url.pathname);
	$: showNav = $isAuthenticated && !isPublicRoute;

	let mobileMenuOpen = false;

	function toggleMobileMenu() {
		mobileMenuOpen = !mobileMenuOpen;
	}

	function closeMobileMenu() {
		mobileMenuOpen = false;
	}

	onMount(() => {
		auth.checkAuth();

		// Listen for auth logout event from API client to avoid navigation freeze
		const handleAuthLogout = () => {
			// Use goto instead of window.location to avoid breaking SvelteKit navigation
			import('$app/navigation').then(({ goto }) => {
				goto('/auth/login', { replaceState: true });
			});
		};

		window.addEventListener('auth:logout', handleAuthLogout);

		return () => {
			window.removeEventListener('auth:logout', handleAuthLogout);
		};
	});
</script>

<Toast />
<UpdateNotification />

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
				<h3>Concept Graph</h3>
				<a href="/concepts" class:active={$page.url.pathname.startsWith('/concepts')}>
					🧠 Concepts
				</a>
				<a href="/picks" class:active={$page.url.pathname.startsWith('/picks')}>
					🛒 Picks
				</a>
				<a href="/locations" class:active={$page.url.pathname.startsWith('/locations')}>
					📍 Locations
				</a>
				<a href="/matches" class:active={$page.url.pathname.startsWith('/matches')}>
					🔗 Matches
				</a>
				<a href="/labels" class:active={$page.url.pathname.startsWith('/labels')}>
					🏷 Labels
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
				<a href="/import" class:active={$page.url.pathname === '/import'}>
					📥 Import
				</a>
				<a href="/sms-import" class:active={$page.url.pathname.startsWith('/sms-import')}>
					💬 SMS Import
				</a>
				<a href="/sms-history" class:active={$page.url.pathname.startsWith('/sms-history')}>
					📱 SMS History
				</a>
				<a href="/email-import" class:active={$page.url.pathname.startsWith('/email-import')}>
					📨 Email Import
				</a>
				<a href="/email-history" class:active={$page.url.pathname.startsWith('/email-history')}>
					📧 Email History
				</a>
				<a href="/email-triage" class:active={$page.url.pathname.startsWith('/email-triage')}>
					🤖 AI Email Triage
				</a>
				<a href="/email-domains" class:active={$page.url.pathname.startsWith('/email-domains')}>
					🏷 Email Domains
				</a>
				<a href="/email-explorer" class:active={$page.url.pathname.startsWith('/email-explorer')}>
					🔬 Email Explorer
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
