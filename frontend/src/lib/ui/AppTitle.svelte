<script lang="ts">
	let { title, className = '' }: { title: string; className?: string } = $props();

	const words = $derived(title.split(/\s+/).filter((word) => word.length > 0));
</script>

<h1
	aria-label={title}
	class={`mx-auto flex max-w-full items-start justify-center whitespace-nowrap font-[900] font-sans tracking-wider [filter:drop-shadow(0_8px_0_#7d7d7d)] text-[#f4f8fc] ${className}`.trim()}
>
	{#each words as word, index}
		<span class={`flex items-start ${index > 0 ? 'ml-[1.28em]' : ''}`.trim()}>
			<span class="text-[114px] leading-none -mt-[4px]">{word[0] ?? ''}</span>
			<span class="text-[84px] leading-none">{word.slice(1)}</span>
		</span>
	{/each}
</h1>
