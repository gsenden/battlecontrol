<script lang="ts">

	import starBig from '$lib/game/assets/stars/stars-000.png';
	import starMed from '$lib/game/assets/stars/stars-001.png';
	import starSml from '$lib/game/assets/stars/stars-002.png';
	import starSml2 from '$lib/game/assets/stars/stars-003.png';
	import starSml3 from '$lib/game/assets/stars/stars-004.png';
	import starMiscBig0 from '$lib/game/assets/stars/stars-misc-big-000.png';
	import starMiscBig1 from '$lib/game/assets/stars/stars-misc-big-001.png';
	import starMiscMed0 from '$lib/game/assets/stars/stars-misc-med-000.png';
	import starMiscMed1 from '$lib/game/assets/stars/stars-misc-med-001.png';
	import starMiscSml0 from '$lib/game/assets/stars/stars-misc-sml-000.png';
	import starMiscSml1 from '$lib/game/assets/stars/stars-misc-sml-001.png';
	import yehatRight from '$lib/game/assets/ships/yehat/terminator-med-004.png';

	type Star = {
		texture: string;
		x: number;
		y: number;
		scale: number;
		opacity: number;
	};

	type Flyby = {
		texture: string;
		top: string;
		scale: number;
		duration: string;
		delay: string;
		className: string;
		opacity: number;
	};

	const BIG_TEXTURES = [starBig, starMiscBig0, starMiscBig1];
	const MED_TEXTURES = [starMed, starMiscMed0, starMiscMed1];
	const SMALL_TEXTURES = [starSml, starSml2, starSml3, starMiscSml0, starMiscSml1];

	const BIG_STAR_COUNT = 34;
	const MED_STAR_COUNT = 58;
	const SMALL_STAR_COUNT = 96;

	const bigStars = createStars(BIG_TEXTURES, BIG_STAR_COUNT, 11, 0.72, 1.15, 0.42, 0.9);
	const medStars = createStars(MED_TEXTURES, MED_STAR_COUNT, 29, 0.55, 0.95, 0.28, 0.72);
	const smallStars = createStars(SMALL_TEXTURES, SMALL_STAR_COUNT, 47, 0.4, 0.72, 0.18, 0.55);
	const flybys: Flyby[] = [
		{
			texture: yehatRight,
			top: '22%',
			scale: 1.1,
			duration: '34s',
			delay: '6s',
			className: 'starfield-flyby-right',
			opacity: 0.24,
		},
		{
			texture: yehatRight,
			top: '68%',
			scale: 0.9,
			duration: '41s',
			delay: '19s',
			className: 'starfield-flyby-left',
			opacity: 0.18,
		},
	];

	function createStars(
		textures: string[],
		count: number,
		seed: number,
		minScale: number,
		maxScale: number,
		minOpacity: number,
		maxOpacity: number,
	): Star[] {
		const rand = mulberry32(seed);
		return Array.from({ length: count }, () => ({
			texture: textures[Math.floor(rand() * textures.length)],
			x: rand() * 100,
			y: rand() * 100,
			scale: minScale + (rand() * (maxScale - minScale)),
			opacity: minOpacity + (rand() * (maxOpacity - minOpacity)),
		}));
	}

	function mulberry32(seed: number) {
		let t = seed;
		return () => {
			t += 0x6d2b79f5;
			let r = Math.imul(t ^ (t >>> 15), t | 1);
			r ^= r + Math.imul(r ^ (r >>> 7), r | 61);
			return ((r ^ (r >>> 14)) >>> 0) / 4294967296;
		};
	}
</script>

<div class="starfield-shell" aria-hidden="true">

	<div class="starfield-layer">
		<div class="starfield-track starfield-layer-big">
			{#each [0, 1, 2, 3] as tile}
				<div class="starfield-tile" class:starfield-tile-right={tile % 2 === 1} class:starfield-tile-bottom={tile >= 2}>
					{#each bigStars as star}
						<img
							alt=""
							class="starfield-star"
							draggable="false"
							src={star.texture}
							style:left={`${star.x}%`};
							style:top={`${star.y}%`};
							style:opacity={star.opacity};
							style:transform={`translate(-50%, -50%) scale(${star.scale})`}
						/>
					{/each}
				</div>
			{/each}
		</div>
	</div>

	<div class="starfield-layer">
		<div class="starfield-track starfield-layer-medium">
			{#each [0, 1, 2, 3] as tile}
				<div class="starfield-tile" class:starfield-tile-right={tile % 2 === 1} class:starfield-tile-bottom={tile >= 2}>
					{#each medStars as star}
						<img
							alt=""
							class="starfield-star"
							draggable="false"
							src={star.texture}
							style:left={`${star.x}%`};
							style:top={`${star.y}%`};
							style:opacity={star.opacity};
							style:transform={`translate(-50%, -50%) scale(${star.scale})`}
						/>
					{/each}
				</div>
			{/each}
		</div>
	</div>

	<div class="starfield-layer">
		<div class="starfield-track starfield-layer-small">
			{#each [0, 1, 2, 3] as tile}
				<div class="starfield-tile" class:starfield-tile-right={tile % 2 === 1} class:starfield-tile-bottom={tile >= 2}>
					{#each smallStars as star}
						<img
							alt=""
							class="starfield-star"
							draggable="false"
							src={star.texture}
							style:left={`${star.x}%`};
							style:top={`${star.y}%`};
							style:opacity={star.opacity};
							style:transform={`translate(-50%, -50%) scale(${star.scale})`}
						/>
					{/each}
				</div>
			{/each}
		</div>
	</div>

	{#each flybys as flyby}
		<img
			alt=""
			aria-hidden="true"
			class={`starfield-flyby ${flyby.className}`.trim()}
			draggable="false"
			src={flyby.texture}
			style:top={flyby.top}
			style:opacity={flyby.opacity}
			style:animation-duration={flyby.duration}
			style:animation-delay={flyby.delay}
			style={`--flyby-scale:${flyby.scale};`}
		/>
	{/each}
</div>
