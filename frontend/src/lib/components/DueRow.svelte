<script lang="ts">
	import Icon from './Icon.svelte';
	import StatusBadge from './StatusBadge.svelte';
	import { dueBadge, dueDetail } from '#lib/format.ts';
	import type { ScheduleView } from '#lib/types.ts';

	let { s, showVehicle = true }: { s: ScheduleView; showVehicle?: boolean } = $props();
</script>

<li>
	<a class="due" href="/auto/{s.vehicle_id}?tab=scadenze#s{s.id}">
		<span class="tile-ico lg"><Icon name={s.icon} /></span>
		<div class="main">
			<div class="title">
				{s.title}
				{#if showVehicle}
					<span class="car-chip"><span class="dot" style="background:{s.vehicle_color};width:8px;height:8px"></span>{s.vehicle_name}</span>
				{/if}
			</div>
			<div class="meta num">
				{dueDetail(s)}
				{#if s.km_stale && s.next_due_km != null}<span class="stale"> · km da aggiornare</span>{/if}
			</div>
		</div>
		<div class="right"><StatusBadge status={s.status} text={dueBadge(s)} /></div>
	</a>
</li>

<style>
	.due {
		display: grid;
		grid-template-columns: 44px minmax(0, 1fr) auto;
		gap: 12px;
		align-items: center;
		padding: 12px 10px;
		border-radius: 12px;
		transition: background 0.15s var(--ease);
	}
	.due:hover {
		background: var(--surface-2);
	}
	.title {
		font-weight: 700;
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}
	.car-chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		font-weight: 600;
		color: var(--fg-muted);
	}
	.meta {
		font-size: 14px;
		color: var(--fg-subtle);
		margin-top: 2px;
	}
	.stale {
		color: var(--warn);
		font-weight: 600;
	}
	@media (max-width: 480px) {
		.due {
			grid-template-columns: 40px minmax(0, 1fr);
		}
		.due :global(.tile-ico.lg) {
			width: 40px;
			height: 40px;
		}
		.right {
			grid-column: 2;
		}
	}
</style>
