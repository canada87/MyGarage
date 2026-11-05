<script lang="ts">
	import Icon from './Icon.svelte';
	import { fmtDate, fmtKm, fmtNum, todayIso } from '#lib/format.ts';
	import { isPending, openDialog } from '#lib/state.svelte.ts';
	import type { CarEvent, OdometerReading } from '#lib/types.ts';

	const { readings, events, vehicleId }: { readings: OdometerReading[]; events: CarEvent[]; vehicleId: number } = $props();

	const DAY = 86_400_000;
	const t = (iso: string) => Date.UTC(+iso.slice(0, 4), +iso.slice(5, 7) - 1, +iso.slice(8, 10));

	const sourceLabel: Record<string, string> = {
		manual: 'A mano',
		event: 'Da intervento',
		initial: 'Inserimento auto',
		ha: 'Home Assistant'
	};

	/** Readings oldest first; on the same day the last one entered wins. */
	const visible = $derived(readings.filter((r) => !isPending('reading', r.id)));
	const chrono = $derived([...visible].sort((a, b) => a.date.localeCompare(b.date) || a.id - b.id));
	const points = $derived.by(() => {
		const byDay = new Map<string, OdometerReading>();
		for (const r of chrono) byDay.set(r.date, r);
		return [...byDay.values()].map((r) => ({ r, x: t(r.date), km: r.km }));
	});

	/** Odometer at a time, interpolated between readings (never extrapolated). */
	function kmAt(x: number): number {
		const p = points;
		if (x <= p[0].x) return p[0].km;
		if (x >= p[p.length - 1].x) return p[p.length - 1].km;
		const i = p.findIndex((q) => q.x >= x);
		const a = p[i - 1];
		const b = p[i];
		return Math.round(a.km + ((b.km - a.km) * (x - a.x)) / (b.x - a.x));
	}

	/** Newest first, with the distance from the previous reading. */
	const rows = $derived.by(() => {
		const out = chrono.map((r, i) => ({ r, delta: i > 0 ? r.km - chrono[i - 1].km : null }));
		return out.reverse();
	});

	const years = $derived.by(() => {
		if (points.length < 2) return [];
		const first = points[0].x;
		const last = points[points.length - 1].x;
		const y0 = new Date(first).getUTCFullYear();
		const y1 = new Date(last).getUTCFullYear();
		const out: { year: number; km: number; partial: boolean }[] = [];
		for (let y = y0; y <= y1; y++) {
			const from = Math.max(first, Date.UTC(y, 0, 1));
			const to = Math.min(last, Date.UTC(y + 1, 0, 1));
			if (to <= from) continue;
			out.push({ year: y, km: Math.max(0, kmAt(to) - kmAt(from)), partial: from > Date.UTC(y, 0, 1) || to < Date.UTC(y + 1, 0, 1) });
		}
		return out;
	});

	const stats = $derived.by(() => {
		if (points.length < 2) return null;
		const first = points[0];
		const last = points[points.length - 1];
		const span = (last.x - first.x) / DAY;
		const yearAgo = last.x - 365 * DAY;
		const thisYear = Date.UTC(new Date(t(todayIso())).getUTCFullYear(), 0, 1);
		return {
			last12: yearAgo >= first.x ? kmAt(last.x) - kmAt(yearAgo) : null,
			perYear: span >= 30 ? Math.round(((last.km - first.km) / span) * 365) : null,
			thisYear: last.x >= thisYear ? kmAt(last.x) - kmAt(Math.max(thisYear, first.x)) : null,
			since: first.r.date
		};
	});

	/* ---------- Chart ---------- */

	let view = $state<'odo' | 'year'>('odo');
	let width = $state(600);
	const H = 220;
	const pad = { l: 64, r: 16, t: 16, b: 28 };
	const iw = $derived(Math.max(10, width - pad.l - pad.r));
	const ih = H - pad.t - pad.b;

	function niceTicks(min: number, max: number, count = 4) {
		if (max === min) max = min + 1;
		const raw = (max - min) / count;
		const mag = 10 ** Math.floor(Math.log10(raw));
		const step = [1, 2, 2.5, 5, 10].map((m) => m * mag).find((s) => s >= raw) ?? raw;
		const lo = Math.floor(min / step) * step;
		const hi = Math.ceil(max / step) * step;
		const ticks: number[] = [];
		for (let v = lo; v <= hi + step / 2; v += step) ticks.push(v);
		return ticks;
	}
	const shortKm = (v: number) => (v >= 10_000 ? `${fmtNum(Math.round(v / 1000))}k` : fmtNum(v));

	const odo = $derived.by(() => {
		if (points.length < 2) return null;
		const x0 = points[0].x;
		const x1 = points[points.length - 1].x;
		const kms = points.map((p) => p.km);
		const ticks = niceTicks(Math.min(...kms), Math.max(...kms));
		const lo = ticks[0];
		const hi = ticks[ticks.length - 1];
		const sx = (x: number) => pad.l + ((x - x0) / Math.max(1, x1 - x0)) * iw;
		const sy = (v: number) => pad.t + ih - ((v - lo) / (hi - lo)) * ih;
		const pts = points.map((p) => ({ ...p, px: sx(p.x), py: sy(p.km) }));
		const y0 = new Date(x0).getUTCFullYear();
		const y1 = new Date(x1).getUTCFullYear();
		const stepY = Math.max(1, Math.ceil((y1 - y0 + 1) / Math.max(1, Math.floor(iw / 70))));
		const xTicks: { label: string; px: number }[] = [];
		for (let y = y0 + 1; y <= y1; y += stepY) xTicks.push({ label: String(y), px: sx(Date.UTC(y, 0, 1)) });
		if (!xTicks.length) xTicks.push({ label: fmtDate(points[0].r.date), px: pts[0].px }, { label: fmtDate(points[points.length - 1].r.date), px: pts[pts.length - 1].px });
		return {
			pts,
			path: pts.map((p, i) => `${i ? 'L' : 'M'}${p.px.toFixed(1)},${p.py.toFixed(1)}`).join(''),
			yTicks: ticks.map((v) => ({ v, py: sy(v) })),
			xTicks
		};
	});

	const bars = $derived.by(() => {
		if (!years.length) return null;
		const ticks = niceTicks(0, Math.max(...years.map((y) => y.km)));
		const hi = ticks[ticks.length - 1];
		const slot = iw / years.length;
		const bw = Math.min(56, slot * 0.6);
		const sy = (v: number) => pad.t + ih - (v / hi) * ih;
		return {
			bw,
			showLabels: years.length <= 8,
			items: years.map((y, i) => ({ ...y, cx: pad.l + slot * (i + 0.5), top: sy(y.km) })),
			yTicks: ticks.map((v) => ({ v, py: sy(v) }))
		};
	});

	/** Rounded top, square bottom on the baseline. */
	function barPath(cx: number, top: number, w: number) {
		const base = pad.t + ih;
		const h = base - top;
		const r = Math.min(4, h, w / 2);
		const x = cx - w / 2;
		return `M${x},${base}V${top + r}Q${x},${top} ${x + r},${top}H${x + w - r}Q${x + w},${top} ${x + w},${top + r}V${base}Z`;
	}

	let hover = $state<number | null>(null);
	function onMove(e: PointerEvent) {
		const box = (e.currentTarget as SVGElement).getBoundingClientRect();
		const x = ((e.clientX - box.left) / box.width) * width;
		if (view === 'odo' && odo) {
			let best = 0;
			odo.pts.forEach((p, i) => Math.abs(p.px - x) < Math.abs(odo.pts[best].px - x) && (best = i));
			hover = best;
		} else if (view === 'year' && bars) {
			const slot = iw / bars.items.length;
			hover = Math.min(bars.items.length - 1, Math.max(0, Math.floor((x - pad.l) / slot)));
		}
	}

	const tip = $derived.by(() => {
		if (hover == null) return null;
		if (view === 'odo' && odo?.pts[hover]) {
			const p = odo.pts[hover];
			return { x: p.px, y: p.py, title: fmtDate(p.r.date), value: fmtKm(p.km) };
		}
		if (view === 'year' && bars?.items[hover]) {
			const b = bars.items[hover];
			return { x: b.cx, y: b.top, title: `${b.year}${b.partial ? ' (parziale)' : ''}`, value: fmtKm(b.km) };
		}
		return null;
	});

	function edit(r: OdometerReading) {
		if (r.event_id != null) {
			const ev = events.find((e) => e.id === r.event_id);
			if (ev) return openDialog({ kind: 'event', vehicleId, event: ev });
		}
		openDialog({ kind: 'km', vehicleId, reading: r, canDelete: visible.length > 1 });
	}
</script>

{#if stats}
	<div class="kstats">
		<div class="card kstat">
			<div class="lbl">Ultimi 12 mesi</div>
			<div class="val num">{stats.last12 != null ? fmtKm(stats.last12) : '—'}</div>
			{#if stats.last12 == null}<div class="sub">dati da meno di un anno</div>{/if}
		</div>
		<div class="card kstat">
			<div class="lbl">Media all’anno</div>
			<div class="val num">{stats.perYear != null ? fmtKm(stats.perYear) : '—'}</div>
			<div class="sub">dal {fmtDate(stats.since)}</div>
		</div>
		<div class="card kstat">
			<div class="lbl">Quest’anno</div>
			<div class="val num">{stats.thisYear != null ? fmtKm(stats.thisYear) : '—'}</div>
			<div class="sub">fino all’ultima lettura</div>
		</div>
	</div>

	<section class="card card-pad chart-card">
		<div class="chart-head">
			<h3>{view === 'odo' ? 'Contachilometri nel tempo' : 'Km percorsi per anno'}</h3>
			<div class="seg" role="group" aria-label="Tipo di grafico">
				<button class="chip" type="button" aria-pressed={view === 'odo'} onclick={() => ((view = 'odo'), (hover = null))}>Totale</button>
				<button class="chip" type="button" aria-pressed={view === 'year'} onclick={() => ((view = 'year'), (hover = null))}>Per anno</button>
			</div>
		</div>
		<div class="chart" bind:clientWidth={width}>
			<svg
				viewBox="0 0 {width} {H}"
				width={width}
				height={H}
				role="img"
				aria-label={view === 'odo' ? 'Andamento del contachilometri' : 'Km percorsi per anno'}
				onpointermove={onMove}
				onpointerdown={onMove}
				onpointerleave={(e) => e.pointerType === 'mouse' && (hover = null)}
			>
				{#if view === 'odo' && odo}
					{#each odo.yTicks as tk (tk.v)}
						<line class="grid" x1={pad.l} x2={width - pad.r} y1={tk.py} y2={tk.py} />
						<text class="axis" x={pad.l - 8} y={tk.py} text-anchor="end" dominant-baseline="middle">{shortKm(tk.v)}</text>
					{/each}
					{#each odo.xTicks as tk (tk.label)}
						<text class="axis" x={tk.px} y={H - 8} text-anchor="middle">{tk.label}</text>
					{/each}
					{#if tip}<line class="cross" x1={tip.x} x2={tip.x} y1={pad.t} y2={pad.t + ih} />{/if}
					<path class="line" d={odo.path} />
					{#each odo.pts as p, i (p.r.id)}
						<circle class="dot" class:active={hover === i} cx={p.px} cy={p.py} r={hover === i ? 6 : 4} />
					{/each}
				{:else if view === 'year' && bars}
					{#each bars.yTicks as tk (tk.v)}
						<line class="grid" x1={pad.l} x2={width - pad.r} y1={tk.py} y2={tk.py} />
						<text class="axis" x={pad.l - 8} y={tk.py} text-anchor="end" dominant-baseline="middle">{shortKm(tk.v)}</text>
					{/each}
					{#each bars.items as b, i (b.year)}
						<path class="bar" class:partial={b.partial} class:dim={hover != null && hover !== i} d={barPath(b.cx, b.top, bars.bw)} />
						{#if bars.showLabels && b.km > 0}
							<text class="val-lbl" x={b.cx} y={b.top - 6} text-anchor="middle">{shortKm(b.km)}</text>
						{/if}
						<text class="axis" x={b.cx} y={H - 8} text-anchor="middle">{b.year}</text>
					{/each}
				{/if}
			</svg>
			{#if tip}
				<div class="tip" style="left:{Math.min(Math.max(tip.x, 70), width - 70)}px; top:{tip.y}px">
					<div class="tip-t">{tip.title}</div>
					<div class="tip-v num">{tip.value}</div>
				</div>
			{/if}
		</div>
		{#if view === 'year'}
			<p class="note subtle">
				I km di ogni anno sono stimati dalle letture più vicine al 1° gennaio. Il primo e l’ultimo anno sono parziali (barre più chiare).
			</p>
		{/if}
	</section>
{:else}
	<div class="card card-pad subtle">Con almeno due letture qui vedrai il grafico dei km e quanti ne fai all’anno.</div>
{/if}

<div class="section-head section">
	<h2>Letture</h2>
	<button class="btn btn-primary btn-sm" type="button" onclick={() => openDialog({ kind: 'km', vehicleId })}><Icon name="plus" size={18} />Aggiungi</button>
</div>
<p class="subtle hint">Tocca una lettura per correggerla o eliminarla.</p>
<ul class="card list">
	{#each rows as { r, delta } (r.id)}
		<li>
			<button class="rd" type="button" onclick={() => edit(r)} aria-label="Modifica lettura del {fmtDate(r.date)}, {fmtKm(r.km)}">
				<span class="tile-ico"><Icon name={r.source === 'event' ? 'wrench' : 'gauge'} size={18} /></span>
				<span class="grow">
					<span class="t num">{fmtKm(r.km)}</span>
					<span class="s">{fmtDate(r.date)} · {sourceLabel[r.source] ?? r.source}{r.note && r.source === 'manual' ? ` · ${r.note}` : ''}</span>
				</span>
				{#if delta != null && delta < 0}
					<span class="badge soon"><Icon name="alert" size={16} />Più bassa della precedente</span>
				{:else if delta != null}
					<span class="delta num">+{fmtNum(delta)}</span>
				{/if}
				<Icon name="pencil" size={16} />
			</button>
		</li>
	{/each}
</ul>

<style>
	.kstats {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 12px;
		margin-bottom: 12px;
	}
	.kstat {
		padding: 14px 16px;
		display: grid;
		gap: 2px;
		align-content: start;
	}
	.lbl {
		font-size: 13px;
		color: var(--fg-muted);
		font-weight: 600;
	}
	.val {
		font-size: 22px;
		font-weight: 800;
		letter-spacing: -0.02em;
	}
	.sub {
		font-size: 12px;
		color: var(--fg-subtle);
	}
	@media (max-width: 560px) {
		.kstats {
			grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
		}
		.kstat:first-child {
			grid-column: 1 / -1;
		}
		.val {
			font-size: 19px;
		}
	}
	.chart-card {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: 8px;
	}
	.chart-head {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}
	.chart-head h3 {
		margin: 0;
		font-size: 16px;
	}
	.seg {
		display: flex;
		gap: 6px;
	}
	.chart {
		position: relative;
		width: 100%;
		touch-action: pan-y;
	}
	svg {
		display: block;
		overflow: visible;
		/* The container sets the width; the SVG must never push it wider. */
		width: 100%;
		height: auto;
	}
	.grid {
		stroke: var(--border);
		stroke-width: 1;
	}
	.axis {
		fill: var(--fg-subtle);
		font-size: 12px;
		font-variant-numeric: tabular-nums;
	}
	.val-lbl {
		fill: var(--fg-muted);
		font-size: 12px;
		font-weight: 700;
		font-variant-numeric: tabular-nums;
	}
	.line {
		fill: none;
		stroke: var(--primary);
		stroke-width: 2;
		stroke-linejoin: round;
		stroke-linecap: round;
	}
	.dot {
		fill: var(--primary);
		stroke: var(--surface);
		stroke-width: 2;
	}
	.cross {
		stroke: var(--border-strong);
		stroke-width: 1;
		stroke-dasharray: 3 3;
	}
	.bar {
		fill: var(--primary);
		transition: opacity 0.15s;
	}
	.bar.partial {
		fill: color-mix(in srgb, var(--primary) 45%, var(--surface));
	}
	.bar.dim {
		opacity: 0.55;
	}
	.tip {
		position: absolute;
		transform: translate(-50%, calc(-100% - 12px));
		background: var(--surface);
		border: 1px solid var(--border-strong);
		border-radius: 10px;
		box-shadow: var(--shadow-lg);
		padding: 6px 10px;
		pointer-events: none;
		white-space: nowrap;
	}
	.tip-t {
		font-size: 12px;
		color: var(--fg-subtle);
	}
	.tip-v {
		font-weight: 800;
	}
	.note {
		margin: 0;
		font-size: 13px;
	}
	.hint {
		margin: -4px 0 8px;
		font-size: 13px;
	}
	.list li:has(.rd) {
		padding: 0;
	}
	.rd {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px;
		border: 0;
		background: transparent;
		border-radius: 12px;
		cursor: pointer;
		text-align: left;
		color: var(--fg-subtle);
		min-height: 56px;
	}
	.rd:hover {
		background: var(--surface-2);
	}
	.rd .grow {
		display: grid;
		color: var(--fg);
	}
	.t {
		font-weight: 700;
	}
	.s {
		font-size: 13px;
		color: var(--fg-subtle);
	}
	.delta {
		font-weight: 700;
		color: var(--fg-muted);
		font-size: 14px;
	}
	@media (prefers-reduced-motion: reduce) {
		.bar {
			transition: none;
		}
	}
</style>
