/**
 * Day/night cycle system.
 * Applies a tinted overlay to the game canvas based on time of day.
 * A full cycle takes `CYCLE_DURATION_MS` (default 20 min).
 */

export type TimeOfDay = 'dawn' | 'day' | 'dusk' | 'night';

interface DayNightState {
	phase: TimeOfDay;
	/** 0-1 progress within current phase. */
	progress: number;
	/** Overall cycle progress 0-1. */
	cycle: number;
}

const CYCLE_DURATION_MS = 20 * 60 * 1000;

const PHASE_RANGES: { phase: TimeOfDay; start: number; end: number }[] = [
	{ phase: 'dawn', start: 0.0, end: 0.1 },
	{ phase: 'day', start: 0.1, end: 0.5 },
	{ phase: 'dusk', start: 0.5, end: 0.6 },
	{ phase: 'night', start: 0.6, end: 1.0 },
];

function getState(cycleProgress: number): DayNightState {
	const c = cycleProgress % 1;
	for (const { phase, start, end } of PHASE_RANGES) {
		if (c >= start && c < end) {
			return {
				phase,
				progress: (c - start) / (end - start),
				cycle: c,
			};
		}
	}
	return { phase: 'night', progress: 1, cycle: c };
}

function getTint(state: DayNightState): { r: number; g: number; b: number; a: number } {
	switch (state.phase) {
		case 'dawn': {
			const t = state.progress;
			return { r: lerp(20, 255, t), g: lerp(20, 200, t), b: lerp(60, 120, t), a: lerp(0.35, 0.05, t) };
		}
		case 'day':
			return { r: 255, g: 255, b: 200, a: 0 };
		case 'dusk': {
			const t = state.progress;
			return { r: lerp(255, 40, t), g: lerp(150, 30, t), b: lerp(80, 80, t), a: lerp(0.05, 0.4, t) };
		}
		case 'night':
			return { r: 20, g: 20, b: 60, a: 0.4 };
	}
}

function lerp(a: number, b: number, t: number): number {
	return a + (b - a) * t;
}

export class DayNightCycle {
	private startTime = Date.now();
	private canvas: HTMLCanvasElement | null = null;
	private ctx: CanvasRenderingContext2D | null = null;
	private animFrame = 0;
	private running = false;
	private enabled = true;
	private cycleDurationMs = CYCLE_DURATION_MS;

	attach(canvas: HTMLCanvasElement) {
		this.canvas = canvas;
		this.ctx = canvas.getContext('2d');
	}

	setEnabled(enabled: boolean) {
		this.enabled = enabled;
		if (!enabled && this.ctx && this.canvas) {
			this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
		}
	}

	setCycleDuration(ms: number) {
		this.cycleDurationMs = Math.max(1000, ms);
	}

	getTimeOfDay(): TimeOfDay {
		return this.getCurrentState().phase;
	}

	getCurrentState(): DayNightState {
		const elapsed = Date.now() - this.startTime;
		return getState(elapsed / this.cycleDurationMs);
	}

	start() {
		if (this.running) return;
		this.running = true;
		this.tick();
	}

	stop() {
		this.running = false;
		if (this.animFrame) {
			cancelAnimationFrame(this.animFrame);
			this.animFrame = 0;
		}
	}

	destroy() {
		this.stop();
		this.canvas = null;
		this.ctx = null;
	}

	private tick = () => {
		if (!this.running) return;
		this.render();
		this.animFrame = requestAnimationFrame(this.tick);
	};

	private render() {
		if (!this.ctx || !this.canvas) return;
		this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

		if (!this.enabled) return;

		const state = this.getCurrentState();
		const tint = getTint(state);

		if (tint.a > 0.001) {
			this.ctx.globalAlpha = tint.a;
			this.ctx.fillStyle = `rgb(${Math.round(tint.r)}, ${Math.round(tint.g)}, ${Math.round(tint.b)})`;
			this.ctx.fillRect(0, 0, this.canvas.width, this.canvas.height);
			this.ctx.globalAlpha = 1;
		}
	}
}

/** Singleton day/night cycle instance. */
export const dayNightCycle = new DayNightCycle();
