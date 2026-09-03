/**
 * Weather system for visual effects (rain, snow, fog).
 * Renders weather particles as a canvas overlay.
 */

export type WeatherType = 'clear' | 'rain' | 'snow' | 'fog' | 'storm';

interface WeatherParticle {
	x: number;
	y: number;
	speed: number;
	size: number;
	opacity: number;
	drift: number;
}

export class WeatherSystem {
	private particles: WeatherParticle[] = [];
	private currentWeather: WeatherType = 'clear';
	private canvas: HTMLCanvasElement | null = null;
	private ctx: CanvasRenderingContext2D | null = null;
	private animFrame: number = 0;
	private running = false;
	private intensity = 1.0;

	attach(canvas: HTMLCanvasElement) {
		this.canvas = canvas;
		this.ctx = canvas.getContext('2d');
	}

	setWeather(type: WeatherType, intensity = 1.0) {
		this.currentWeather = type;
		this.intensity = Math.max(0, Math.min(1, intensity));
		this.particles = [];
		if (type !== 'clear') {
			this.spawnParticles();
		}
	}

	getWeather(): WeatherType {
		return this.currentWeather;
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
		this.particles = [];
	}

	private spawnParticles() {
		if (!this.canvas) return;
		const count = this.getParticleCount();
		for (let i = 0; i < count; i++) {
			this.particles.push(this.createParticle());
		}
	}

	private getParticleCount(): number {
		const base = {
			rain: 200,
			snow: 100,
			fog: 30,
			storm: 300,
			clear: 0,
		};
		return Math.floor((base[this.currentWeather] || 0) * this.intensity);
	}

	private createParticle(): WeatherParticle {
		const w = this.canvas?.width ?? 800;
		const h = this.canvas?.height ?? 600;
		switch (this.currentWeather) {
			case 'rain':
			case 'storm':
				return {
					x: Math.random() * w,
					y: Math.random() * h - h,
					speed: 8 + Math.random() * 6,
					size: 1 + Math.random() * 2,
					opacity: 0.3 + Math.random() * 0.4,
					drift: this.currentWeather === 'storm' ? -2 + Math.random() * 4 : 0,
				};
			case 'snow':
				return {
					x: Math.random() * w,
					y: Math.random() * h - h,
					speed: 0.5 + Math.random() * 1.5,
					size: 2 + Math.random() * 3,
					opacity: 0.5 + Math.random() * 0.5,
					drift: -0.5 + Math.random(),
				};
			case 'fog':
				return {
					x: Math.random() * w,
					y: Math.random() * h,
					speed: 0.2 + Math.random() * 0.3,
					size: 60 + Math.random() * 80,
					opacity: 0.05 + Math.random() * 0.1,
					drift: 0.1 + Math.random() * 0.2,
				};
			default:
				return { x: 0, y: 0, speed: 0, size: 0, opacity: 0, drift: 0 };
		}
	}

	private tick = () => {
		if (!this.running) return;
		this.update();
		this.render();
		this.animFrame = requestAnimationFrame(this.tick);
	};

	private update() {
		if (!this.canvas) return;
		const h = this.canvas.height;
		const w = this.canvas.width;

		for (const p of this.particles) {
			p.y += p.speed;
			p.x += p.drift;

			if (p.y > h + 10) {
				p.y = -10;
				p.x = Math.random() * w;
			}
			if (p.x < -10) p.x = w + 10;
			if (p.x > w + 10) p.x = -10;
		}
	}

	private render() {
		if (!this.ctx || !this.canvas) return;
		this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

		switch (this.currentWeather) {
			case 'rain':
			case 'storm':
				this.renderRain();
				break;
			case 'snow':
				this.renderSnow();
				break;
			case 'fog':
				this.renderFog();
				break;
		}
	}

	private renderRain() {
		if (!this.ctx) return;
		this.ctx.strokeStyle = 'rgba(174, 194, 224, 0.6)';
		this.ctx.lineWidth = 1;
		for (const p of this.particles) {
			this.ctx.globalAlpha = p.opacity;
			this.ctx.beginPath();
			this.ctx.moveTo(p.x, p.y);
			this.ctx.lineTo(p.x + p.drift, p.y + p.size * 5);
			this.ctx.stroke();
		}
		this.ctx.globalAlpha = 1;
	}

	private renderSnow() {
		if (!this.ctx) return;
		for (const p of this.particles) {
			this.ctx.globalAlpha = p.opacity;
			this.ctx.fillStyle = '#ffffff';
			this.ctx.beginPath();
			this.ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2);
			this.ctx.fill();
		}
		this.ctx.globalAlpha = 1;
	}

	private renderFog() {
		if (!this.ctx) return;
		for (const p of this.particles) {
			this.ctx.globalAlpha = p.opacity;
			const gradient = this.ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, p.size);
			gradient.addColorStop(0, 'rgba(200, 200, 200, 0.3)');
			gradient.addColorStop(1, 'rgba(200, 200, 200, 0)');
			this.ctx.fillStyle = gradient;
			this.ctx.fillRect(p.x - p.size, p.y - p.size, p.size * 2, p.size * 2);
		}
		this.ctx.globalAlpha = 1;
	}
}

/** Singleton weather system instance. */
export const weatherSystem = new WeatherSystem();
