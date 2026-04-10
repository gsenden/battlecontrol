<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { getCurrentUser } from '$lib/auth/auth.js';
	import { currentLanguage } from '$lib/i18n/i18n.js';
	import { t } from '$lib/i18n/translations.js';
	import AppTitle from '$lib/ui/AppTitle.svelte';
	import LandingActionButton from '$lib/ui/LandingActionButton.svelte';
	import LandingTextLink from '$lib/ui/LandingTextLink.svelte';

	onMount(() => {
		void redirectLoggedInUserToLobby();
	});

	async function redirectLoggedInUserToLobby() {
		try {
			const currentUser = await getCurrentUser();
			if (currentUser) {
				window.location.assign('/lobby');
			}
		} catch {
			// The landing page should remain usable when the session probe fails.
		}
	}
</script>

<div class="flex h-full items-start justify-center px-6 pt-[18vh]">
	<div class="w-full max-w-[760px] text-[#ecf1f7]">
		<div class="mb-12 text-center">
			<div class="mx-auto inline-flex flex-col items-stretch">

				<p class="mt-4 text-[17px] leading-7 text-[#a7bacf]">
					{t('HOME_SUBTITLE', $currentLanguage)}
				</p>
				<div class="mt-8">
					<AppTitle className="uppercase" title={t('APP_NAME', $currentLanguage)} />
				</div>

			</div>
		</div>

		<div class="mt-50 flex flex-col items-center">
			<div class="flex flex-wrap items-center justify-center gap-8">
				<LandingActionButton
					label={t('LOGIN', $currentLanguage)}
					onclick={() => goto('/login')}
				/>

				<LandingActionButton
					label={t('REGISTER', $currentLanguage)}
					onclick={() => goto('/register')}
				/>
			</div>

			<LandingTextLink
				label={t('LOGIN_WITH_ONE_TIME_CODE', $currentLanguage)}
			/>
		</div>
	</div>
</div>
