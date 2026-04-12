<script lang="ts">
	import { goto } from '$app/navigation';
	import { currentLanguage } from '$lib/i18n/i18n.js';
	import { t } from '$lib/i18n/translations.js';
	import AppTitle from '$lib/ui/AppTitle.svelte';
	import LandingTextLink from '$lib/ui/LandingTextLink.svelte';
</script>

<div class="relative flex h-full items-start justify-center px-6 pt-[12vh]">
	<div class="absolute left-6 top-6">
		<AppTitle className="origin-top-left scale-[0.25] uppercase" title={t('APP_NAME', $currentLanguage)} />
	</div>

	<div class="flex w-full flex-col items-center text-[#ecf1f7]">
		<div class="mb-10 mt-8 flex w-full justify-center text-center">
			<AppTitle className="origin-top scale-[0.66] uppercase" title={t('CREDITS_AND_LICENSES', $currentLanguage)} />
		</div>

		<div class="w-full max-w-[760px] space-y-6 rounded-[20px] border border-[#4b5f79] bg-[rgb(8_17_29/72%)] p-8 text-center text-[15px] leading-7 text-[#d7e4f2] shadow-[0_24px_80px_rgb(0_0_0/22%)] backdrop-blur-[10px]">
			<p>{t('CREDITS_INTRO', $currentLanguage)}</p>
			<p>{t('CREDITS_GRATITUDE', $currentLanguage)}</p>

			<div class="space-y-2">
				<h2 class="text-[13px] font-[800] uppercase tracking-[0.22em] text-[#a9bfd8]">
					{t('CREDITS_UPSTREAM', $currentLanguage)}
				</h2>
				<ul class="space-y-1 text-[#ecf1f7]">
					<li>The Ur-Quan Masters</li>
					<li>The Ur-Quan Masters HD MegaMod</li>
				</ul>
			</div>

			<div class="space-y-2">
				<h2 class="text-[13px] font-[800] uppercase tracking-[0.22em] text-[#a9bfd8]">
					{t('CREDITS_LICENSES_TITLE', $currentLanguage)}
				</h2>
				<p>{t('CREDITS_CODE_LICENSE', $currentLanguage)}</p>
				<p>{t('CREDITS_CONTENT_LICENSE', $currentLanguage)}</p>
				<p>{t('CREDITS_OPEN_SOURCE', $currentLanguage)}</p>
			</div>

			<div class="space-y-2">
				<h2 class="text-[13px] font-[800] uppercase tracking-[0.22em] text-[#a9bfd8]">
					{t('CODEBERG_DOCUMENTS', $currentLanguage)}
				</h2>
				<div class="flex flex-col items-center gap-2">
					<a
						class="text-[#d7e4f2] underline underline-offset-4 transition hover:text-[#f4f8fc]"
						href="https://codeberg.org/gsenden/battlecontrol/src/branch/main/CREDITS.md"
						rel="noreferrer"
						target="_blank"
					>
						{t('VIEW_CREDITS_DOCUMENT', $currentLanguage)}
					</a>
					<a
						class="text-[#d7e4f2] underline underline-offset-4 transition hover:text-[#f4f8fc]"
						href="https://codeberg.org/gsenden/battlecontrol/src/branch/main/LICENSE"
						rel="noreferrer"
						target="_blank"
					>
						{t('VIEW_LICENSE_DOCUMENT', $currentLanguage)}
					</a>
				</div>
			</div>
		</div>

		<div class="mt-8 text-center">
			<LandingTextLink
				className="text-[15px]"
				label={t('CLOSE', $currentLanguage)}
				onclick={() => goto('/')}
			/>
		</div>
	</div>
</div>
