<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { onDestroy, onMount } from 'svelte';
	import '../app.css';
	import { getCurrentUser, logoutUser, type UserDto } from '$lib/auth/auth.js';
	import { currentLanguage, languageOptions, setLanguage } from '$lib/i18n/i18n.js';
	import { APP_VERSION } from '$lib/game/version.js';
	import { t } from '$lib/i18n/translations.js';
	import StarfieldBackground from '$lib/ui/StarfieldBackground.svelte';

	let { children } = $props();
	const pageTitle = import.meta.env.DEV ? 'Battle Control DEV' : 'Battle Control';
	let currentUser = $state<UserDto | null>(null);
	let profileMenuOpen = $state(false);
	let profileMenuElement = $state<HTMLDivElement | null>(null);
	const showGlobalChrome = $derived(!page.url.pathname.startsWith('/battle'));

	function handleLanguageChange(event: Event) {
		const target = event.currentTarget as HTMLSelectElement;
		setLanguage(target.value as typeof $currentLanguage);
	}

	onMount(() => {
		void loadCurrentUser();
		const handleUserUpdated = (event: Event) => {
			const customEvent = event as CustomEvent<UserDto>;
			currentUser = customEvent.detail;
		};
		const handleDocumentClick = (event: MouseEvent) => {
			if (!profileMenuOpen || !profileMenuElement) {
				return;
			}

			const target = event.target;
			if (target instanceof Node && !profileMenuElement.contains(target)) {
				profileMenuOpen = false;
			}
		};

		document.addEventListener('mousedown', handleDocumentClick);
		window.addEventListener('battlecontrol:user-updated', handleUserUpdated as EventListener);

		return () => {
			document.removeEventListener('mousedown', handleDocumentClick);
			window.removeEventListener('battlecontrol:user-updated', handleUserUpdated as EventListener);
		};
	});
	
	onDestroy(() => {
		profileMenuOpen = false;
	});
	
	async function loadCurrentUser() {
		currentUser = await getCurrentUser();
	}

	async function signOut() {
		await logoutUser();
		profileMenuOpen = false;
		currentUser = null;
		await goto('/');
	}
</script>

<svelte:head>
	<title>{pageTitle}</title>
</svelte:head>

<div class="app-shell">
	<StarfieldBackground />
	{#if showGlobalChrome}
		<div class="absolute right-6 top-6 z-20 flex items-center gap-3">
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

			{#if currentUser}
				<div bind:this={profileMenuElement} class="relative">
					<button
						aria-label={t('PROFILE_MENU', $currentLanguage)}
						class="flex h-10 w-10 items-center justify-center rounded-full border border-[#6f7680] bg-[rgb(10_16_24/72%)] text-[#f4f8fc] transition hover:border-[#aeb6c4]"
						onclick={() => profileMenuOpen = !profileMenuOpen}
						type="button"
					>
						{#if currentUser.profile_image_url}
							<img alt={currentUser.name} class="h-10 w-10 rounded-full object-cover" src={currentUser.profile_image_url} />
						{:else}
							<svg viewBox="0 0 24 24" class="h-5 w-5 fill-current" aria-hidden="true">
								<path d="M12 12c2.76 0 5-2.69 5-6s-2.24-6-5-6-5 2.69-5 6 2.24 6 5 6Zm0 2c-4.42 0-8 2.69-8 6v2h16v-2c0-3.31-3.58-6-8-6Z" />
							</svg>
						{/if}
					</button>

					{#if profileMenuOpen}
						<div class="absolute right-0 mt-2 min-w-[180px] rounded-[14px] border border-[#4b5f79] bg-[rgb(8_17_29/94%)] p-2 shadow-[0_24px_80px_rgb(0_0_0/28%)]">
							<button
								class="w-full rounded-[10px] px-3 py-2 text-left text-[14px] font-[700] text-[#f4f8fc] transition hover:bg-[rgb(46_73_112/28%)]"
								onclick={() => {
									profileMenuOpen = false;
									void goto('/profile');
								}}
								type="button"
							>
								{t('PROFILE', $currentLanguage)}
							</button>
							<button
								class="w-full rounded-[10px] px-3 py-2 text-left text-[14px] font-[700] text-[#f4f8fc] transition hover:bg-[rgb(46_73_112/28%)]"
								onclick={() => void signOut()}
								type="button"
							>
								{t('LOGOUT', $currentLanguage)}
							</button>
						</div>
					{/if}
				</div>
			{/if}
		</div>
	{/if}
	<div class="app-content">
		{@render children()}
	</div>
	{#if showGlobalChrome}
		<div id="version-badge">v{APP_VERSION}</div>
	{/if}
</div>
