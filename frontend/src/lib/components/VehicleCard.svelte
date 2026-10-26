<script lang="ts">
	import Icon from './Icon.svelte';
	import Plate from './Plate.svelte';
	import CarSilhouette from './CarSilhouette.svelte';
	import { daysAgo, dueBadge, fmtKm, progressKm, progressTime } from '#lib/format.ts';
	import type { VehicleSummary } from '#lib/types.ts';

	let { v }: { v: VehicleSummary } = $props();
	const subtitle = $derived([v.make, v.model, v.year].filter(Boolean).join(' · '));
	const next = $derived(v.next);
	const progress = $derived(next ? (progressTime(next) ?? progressKm(next)) : null);
</script>

<a class="card car" href="/auto/{v.id}" style="--car:{v.color}">
	<div class="hero">
		<span class="sil"><CarSilhouette /></span>
		<div class="name">{v.name}</div>
		{#if subtitle}<div class="model">{subtitle}</div>{/if}
		<Plate plate={v.plate} />
	</div>
	<div class="body">
		<div class="km">
			<span class="km-val num">{fmtKm(v.current_km)}</span>
			<span class="km-sub">aggiornati {daysAgo(v.current_km_date)}</span>
		</div>
		<div class="row">
			{#if v.counts.overdue}<span class="badge overdue"><Icon name="alert" size={16} />{v.counts.overdue} scadut{v.counts.overdue === 1 ? 'a' : 'e'}</span>{/if}
			{#if v.counts.soon}<span class="badge soon"><Icon name="clock" size={16} />{v.counts.soon} in arrivo</span>{/if}
			{#if v.counts.setup}<span class="badge setup"><Icon name="help" size={16} />{v.counts.setup} da completare</span>{/if}
			{#if v.counts.ok}<span class="badge ok"><Icon name="ok" size={16} />{v.counts.ok} ok</span>{/if}
		</div>
		{#if next}
			<div class="next">
				<div class="prow"><span>{next.title}</span><span class="num">{dueBadge(next)}</span></div>
				{#if progress != null}<div class="bar {next.status}"><span style="width:{Math.max(4, progress * 100)}%"></span></div>{/if}
			</div>
		{/if}
	</div>
</a>

<style>
	.car {
		display: block;
		overflow: hidden;
		transition:
			transform 0.2s var(--ease),
			box-shadow 0.2s var(--ease);
	}
	.car:hover {
		transform: translateY(-2px);
	}
	.hero {
		position: relative;
		padding: 18px 18px 14px;
		background: linear-gradient(135deg, color-mix(in srgb, var(--car) 18%, var(--surface)) 0%, var(--surface) 70%);
	}
	.sil {
		position: absolute;
		right: -8px;
		bottom: -6px;
		width: 150px;
		color: var(--car);
		opacity: 0.22;
	}
	.name {
		font-size: 20px;
		font-weight: 800;
		letter-spacing: -0.01em;
		position: relative;
	}
	.model {
		color: var(--fg-muted);
		font-size: 14px;
		margin-bottom: 10px;
		position: relative;
	}
	.body {
		padding: 14px 18px 18px;
		display: grid;
		gap: 14px;
	}
	.km {
		display: flex;
		align-items: baseline;
		gap: 8px;
		flex-wrap: wrap;
	}
	.km-val {
		font-size: 24px;
		font-weight: 800;
		letter-spacing: -0.02em;
	}
	.km-sub {
		font-size: 13px;
		color: var(--fg-subtle);
	}
	.next {
		display: grid;
		gap: 6px;
	}
	.prow {
		display: flex;
		justify-content: space-between;
		gap: 8px;
		font-size: 13px;
		color: var(--fg-muted);
		font-weight: 500;
	}
</style>
