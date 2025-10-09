<script lang="ts">
	export let mainId: string = 'main-content';
</script>

<a href="#{mainId}" class="skip-nav">
	Skip to main content
</a>

<style>
	.skip-nav {
		position: absolute;
		left: -9999px;
		z-index: 10000;
		padding: 1rem 1.5rem;
		background: var(--primary, #3b82f6);
		color: white;
		text-decoration: none;
		border-radius: 0 0 4px 0;
		font-weight: 600;
	}

	.skip-nav:focus {
		left: 0;
		top: 0;
		outline: 3px solid var(--primary-dark, #2563eb);
		outline-offset: 2px;
	}
</style>
