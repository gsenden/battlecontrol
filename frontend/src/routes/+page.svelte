<script lang="ts">
	import { goto } from '$app/navigation';
</script>

<div class="flex h-full items-center justify-center">
	<button
		class="min-w-[220px] min-h-[64px] px-6 py-3.5 border border-[#8c8c8c]
			bg-gradient-to-b from-[#2f3f56] to-[#162031] text-[#f1f1f1]
			font-mono font-bold text-[22px] uppercase tracking-wider cursor-pointer
			shadow-[0_0_0_1px_rgb(255_255_255/10%)_inset,0_12px_32px_rgb(0_0_0/45%)]
			hover:bg-gradient-to-b hover:from-[#3b4e69] hover:to-[#1b283d]"
		onclick={() => goto('/battle')}
	>
		Start Battle
	</button>
</div>
