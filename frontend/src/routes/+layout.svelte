<script lang="ts">
	import '../app.css';
	import { currentLanguage, setLanguage } from '$lib/i18n/i18n.js';
	import StarfieldBackground from '$lib/ui/StarfieldBackground.svelte';

	let { children } = $props();
</script>

<div class="app-shell">
	<StarfieldBackground />
	<div class="absolute right-6 top-6 z-20 flex items-center gap-2">
		<button
			class={`rounded-full border px-3 py-1 text-[12px] font-[700] uppercase tracking-[0.18em] transition ${$currentLanguage === 'nl-NL' ? 'border-[#d7d7d7] bg-[rgb(70_82_104/56%)] text-white' : 'border-[#6f7680] bg-[rgb(10_16_24/30%)] text-[#b0b7c2] hover:border-[#aeb6c4] hover:text-white'}`}
			onclick={() => setLanguage('nl-NL')}
			type="button"
		>
			NL
		</button>
		<button
			class={`rounded-full border px-3 py-1 text-[12px] font-[700] uppercase tracking-[0.18em] transition ${$currentLanguage === 'en-GB' ? 'border-[#d7d7d7] bg-[rgb(70_82_104/56%)] text-white' : 'border-[#6f7680] bg-[rgb(10_16_24/30%)] text-[#b0b7c2] hover:border-[#aeb6c4] hover:text-white'}`}
			onclick={() => setLanguage('en-GB')}
			type="button"
		>
			EN
		</button>
	</div>
	<div class="app-content">
		{@render children()}
	</div>
</div>
