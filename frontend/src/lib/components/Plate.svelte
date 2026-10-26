<script lang="ts">
	let { plate }: { plate: string | null } = $props();
</script>

{#if plate}
	<span class="plate" aria-label="Targa {plate}">
		<span class="eu" aria-hidden="true">I</span>
		<span class="txt">{plate}</span>
	</span>
{/if}

<style>
	.plate {
		display: inline-flex;
		align-items: stretch;
		height: 26px;
		border-radius: 5px;
		overflow: hidden;
		border: 1.5px solid #1e293b;
		background: #fff;
		color: #0f172a;
		font-weight: 800;
		letter-spacing: 0.08em;
		font-size: 14px;
	}
	.eu {
		background: #1d4ed8;
		color: #fff;
		width: 16px;
		display: grid;
		place-items: end center;
		font-size: 8px;
		padding-bottom: 2px;
		letter-spacing: 0;
	}
	.txt {
		padding: 0 8px;
		display: grid;
		place-items: center;
		font-variant-numeric: tabular-nums;
	}
</style>
