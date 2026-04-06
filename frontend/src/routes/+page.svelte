<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { loadStoredUser, type UserDto } from '$lib/auth/auth.js';
	import { currentLanguage } from '$lib/i18n/i18n.js';
	import { t } from '$lib/i18n/translations.js';
	import AppTitle from '$lib/ui/AppTitle.svelte';
	import LandingActionButton from '$lib/ui/LandingActionButton.svelte';
	import LandingTextLink from '$lib/ui/LandingTextLink.svelte';

	let storedUser = $state<UserDto | null>(null);

	onMount(() => {
		storedUser = loadStoredUser();
	});
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

		{#if storedUser}
			<div class="mx-auto mb-8 max-w-[560px] rounded-[12px] border border-[#3f6f52] bg-[#0d1d16] px-4 py-3 text-[14px] text-[#b8f0c7]">
				{t('AUTH_SUCCESS', $currentLanguage)}: {storedUser.name}
			</div>
		{/if}

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
