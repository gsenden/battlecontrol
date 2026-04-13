<script lang="ts">
	import { onMount } from 'svelte';
	import { getCurrentUser, loginUser, toReadableErrorMessage } from '$lib/auth/auth.js';
	import { currentLanguage } from '$lib/i18n/i18n.js';
	import { t } from '$lib/i18n/translations.js';
	import AppTitle from '$lib/ui/AppTitle.svelte';
	import LandingActionButton from '$lib/ui/LandingActionButton.svelte';
	import LandingTextLink from '$lib/ui/LandingTextLink.svelte';

	let name = $state('');
	let errorMessage = $state('');
	let isSubmitting = $state(false);

	onMount(() => {
		void redirectLoggedInUserToLobby();
	});

	async function redirectLoggedInUserToLobby() {
		try {
			const currentUser = await getCurrentUser();
			if (currentUser) {
				window.location.assign('/menu');
			}
		} catch {
			// The landing page should remain usable when the session probe fails.
		}
	}

	async function submitLogin() {
		errorMessage = '';
		isSubmitting = true;

		try {
			await loginUser(name.trim());
			window.location.assign('/menu');
		} catch (error) {
			errorMessage = toReadableErrorMessage(error);
		} finally {
			isSubmitting = false;
		}
	}
</script>

<div class="relative flex h-full items-start justify-center px-6 pt-[18vh]">
	<div class="w-full max-w-[760px] text-[#ecf1f7]">
		<div class="mb-12 text-center">
			<div class="mx-auto inline-flex flex-col items-stretch">
				<p class="mt-4 whitespace-nowrap text-[16px] leading-7 text-[#a7bacf]">
					{t('HOME_SUBTITLE', $currentLanguage)}
				</p>
				<div class="mt-8">
					<AppTitle className="uppercase" title={t('APP_NAME', $currentLanguage)} />
				</div>
			</div>
		</div>

		<form
			class="mt-40 flex flex-col items-center"
			onsubmit={(event) => {
				event.preventDefault();
				void submitLogin();
			}}
		>
			<div class="w-full max-w-[560px]">
				<input
					aria-label={t('PLAYER_NAME', $currentLanguage)}
					bind:value={name}
					class="w-full rounded-[12px] border border-[#3d5570] bg-[#08111d] px-4 py-3 text-[16px] text-[#f3f7fb] outline-none transition focus:border-[#83c5ff]"
					maxlength="32"
					placeholder={t('PLAYER_NAME', $currentLanguage)}
					required
				/>
			</div>

			{#if errorMessage}
				<div class="mt-5 w-full max-w-[560px] rounded-[12px] border border-[#8f3e45] bg-[#2a1115] px-4 py-3 text-[14px] text-[#ffbcc2]">
					{errorMessage}
				</div>
			{/if}

			<div class="mt-8">
				<LandingActionButton
					label={t('LOGIN', $currentLanguage)}
					disabled={isSubmitting}
					type="submit"
				/>
			</div>

			<div class="mt-4">
				<LandingTextLink
					label={t('RECOVER_PLAYER', $currentLanguage)}
					onclick={() => {
						window.location.assign('/recover');
					}}
				/>
			</div>
		</form>
	</div>

	<div class="absolute bottom-8 left-1/2 -translate-x-1/2">
		<div class="flex flex-col items-center gap-3 text-center">
			<p class="max-w-[980px] text-[12px] leading-6 text-[#8ea3ba]">
				{t('CREDITS_THANKS_PREFIX', $currentLanguage)}
				<a
					class="underline underline-offset-4 transition hover:text-[#d6e7ff]"
					href="https://www.kickstarter.com/projects/pistolshrimp/free-stars-children-of-infinity"
					rel="noreferrer"
					target="_blank"
				>
					{t('FREE_STARS_PROJECT', $currentLanguage)}
				</a>
				{t('CREDITS_THANKS_SUFFIX', $currentLanguage)}
			</p>
			<LandingTextLink
				className="text-[13px] uppercase tracking-[0.16em] text-[#9cb2c9] no-underline hover:text-[#d6e7ff]"
				label={t('CREDITS_AND_LICENSES', $currentLanguage)}
				onclick={() => goto('/credits')}
			/>
		</div>
	</div>
</div>
