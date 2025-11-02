<script lang="ts">
	import '../app.css';
	import { page } from '$app/stores';

	let mobileMenuOpen = false;

	function toggleMobileMenu() {
		mobileMenuOpen = !mobileMenuOpen;
	}

	function closeMobileMenu() {
		mobileMenuOpen = false;
	}
</script>

<div class="app-layout">
	<nav class="sidebar">
		<div class="logo">
			<h2>📇 SagensContact</h2>
			<p class="version">Desktop v0.1.0-alpha.3</p>
		</div>
		<div class="nav-section">
			<h3>Main</h3>
			<a href="/" class:active={$page.url.pathname === '/'}>
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
			<a href="/import" class:active={$page.url.pathname.startsWith('/import')}>
				📥 Import
			</a>
		</div>
		<div class="nav-section">
			<h3>System</h3>
			<a href="/settings" class:active={$page.url.pathname.startsWith('/settings')}>
				⚙️ Settings
			</a>
		</div>
	</nav>
	<main class="main-content">
		<slot />
	</main>
</div>

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

	.version {
		font-size: 0.75rem;
		color: var(--gray-500);
		margin-top: 0.25rem;
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
