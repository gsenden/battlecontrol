<script lang="ts">
	import '../app.css';
	import { currentLanguage, languageOptions, setLanguage } from '$lib/i18n/i18n.js';
	import { APP_VERSION } from '$lib/game/version.js';
	import StarfieldBackground from '$lib/ui/StarfieldBackground.svelte';

	let { children } = $props();
	const pageTitle = import.meta.env.DEV ? 'Battle Control DEV' : 'Battle Control';

	function handleLanguageChange(event: Event) {
		const target = event.currentTarget as HTMLSelectElement;
		setLanguage(target.value as typeof $currentLanguage);
	}
</script>

<svelte:head>
	<title>{pageTitle}</title>
</svelte:head>

<div class="app-shell">
	<StarfieldBackground />
	<div class="absolute right-6 top-6 z-20">
		<select
			aria-label="Language"
			class="rounded-full border border-[#6f7680] bg-[rgb(10_16_24/72%)] px-4 py-2 text-[12px] font-[700] uppercase tracking-[0.18em] text-[#f4f8fc] outline-none transition hover:border-[#aeb6c4]"
			onchange={handleLanguageChange}
			value={$currentLanguage}
		>
			{#each languageOptions as language}
				<option value={language.code}>{language.label}</option>
			{/each}
		</select>
	</div>
	<div class="app-content">
		{@render children()}
	</div>
	<div id="version-badge">v{APP_VERSION}</div>
</div>
