export type ParticleEffectType = "spell_hit" | "heal" | "levelup" | "death";

interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  life: number;
  maxLife: number;
  size: number;
  color: string;
  alpha: number;
}

interface ActiveEffect {
  type: ParticleEffectType;
  particles: Particle[];
  startedAt: number;
  originX: number;
  originY: number;
}

const MAX_EFFECTS = 20;

export class ParticleEngine {
  private effects: ActiveEffect[] = [];
  private ctx: CanvasRenderingContext2D | null = null;
  private canvas: HTMLCanvasElement | null = null;
  private rafId: number | undefined;
  private lastTime = 0;

  attach(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.ctx = canvas.getContext("2d");
    this.lastTime = performance.now();
    this.loop();
  }

  detach() {
    if (this.rafId !== undefined) cancelAnimationFrame(this.rafId);
    this.rafId = undefined;
    this.effects = [];
    this.ctx = null;
    this.canvas = null;
  }

  emit(type: ParticleEffectType, screenX: number, screenY: number) {
    if (this.effects.length >= MAX_EFFECTS) this.effects.shift();
    const particles = createParticles(type, screenX, screenY);
    this.effects.push({
      type,
      particles,
      startedAt: performance.now(),
      originX: screenX,
      originY: screenY,
    });
  }

  private loop = () => {
    const now = performance.now();
    const dt = Math.min((now - this.lastTime) / 1000, 0.05);
    this.lastTime = now;

    this.update(dt);
    this.draw();

    this.rafId = requestAnimationFrame(this.loop);
  };

  private update(dt: number) {
    for (let i = this.effects.length - 1; i >= 0; i--) {
      const effect = this.effects[i]!;
      let alive = false;
      for (const p of effect.particles) {
        p.life -= dt;
        if (p.life <= 0) continue;
        alive = true;
        p.x += p.vx * dt;
        p.y += p.vy * dt;
        p.vy += 40 * dt;
        p.alpha = Math.max(0, p.life / p.maxLife);
      }
      if (!alive) this.effects.splice(i, 1);
    }
  }

  private draw() {
    if (!this.ctx || !this.canvas) return;
    this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

    for (const effect of this.effects) {
      for (const p of effect.particles) {
        if (p.life <= 0) continue;
        this.ctx.globalAlpha = p.alpha;
        this.ctx.fillStyle = p.color;
        this.ctx.beginPath();
        this.ctx.arc(p.x, p.y, p.size * p.alpha, 0, Math.PI * 2);
        this.ctx.fill();
      }
    }
    this.ctx.globalAlpha = 1;
  }
}

function createParticles(type: ParticleEffectType, cx: number, cy: number): Particle[] {
  switch (type) {
    case "spell_hit":
      return spawnBurst(cx, cy, 15, "#ef4444", "#ff6b35", 0.6);
    case "heal":
      return spawnRise(cx, cy, 12, "#4ade80", "#22d3ee", 1.0);
    case "levelup":
      return spawnStarburst(cx, cy, 25, "#fbbf24", "#fcd34d", 1.2);
    case "death":
      return spawnFade(cx, cy, 10, "#991b1b", "#6b2020", 1.5);
  }
}

function randRange(min: number, max: number): number {
  return min + Math.random() * (max - min);
}

function spawnBurst(cx: number, cy: number, count: number, c1: string, c2: string, life: number): Particle[] {
  const particles: Particle[] = [];
  for (let i = 0; i < count; i++) {
    const angle = Math.random() * Math.PI * 2;
    const speed = randRange(40, 120);
    particles.push({
      x: cx,
      y: cy,
      vx: Math.cos(angle) * speed,
      vy: Math.sin(angle) * speed - 30,
      life,
      maxLife: life,
      size: randRange(2, 5),
      color: Math.random() > 0.5 ? c1 : c2,
      alpha: 1,
    });
  }
  return particles;
}

function spawnRise(cx: number, cy: number, count: number, c1: string, c2: string, life: number): Particle[] {
  const particles: Particle[] = [];
  for (let i = 0; i < count; i++) {
    particles.push({
      x: cx + randRange(-15, 15),
      y: cy + randRange(-5, 5),
      vx: randRange(-10, 10),
      vy: randRange(-80, -40),
      life: life + randRange(0, 0.4),
      maxLife: life,
      size: randRange(2, 4),
      color: Math.random() > 0.5 ? c1 : c2,
      alpha: 1,
    });
  }
  return particles;
}

function spawnStarburst(cx: number, cy: number, count: number, c1: string, c2: string, life: number): Particle[] {
  const particles: Particle[] = [];
  for (let i = 0; i < count; i++) {
    const angle = (i / count) * Math.PI * 2 + randRange(-0.2, 0.2);
    const speed = randRange(30, 100);
    particles.push({
      x: cx,
      y: cy,
      vx: Math.cos(angle) * speed,
      vy: Math.sin(angle) * speed - 50,
      life: life + randRange(0, 0.5),
      maxLife: life,
      size: randRange(2, 6),
      color: Math.random() > 0.5 ? c1 : c2,
      alpha: 1,
    });
  }
  return particles;
}

function spawnFade(cx: number, cy: number, count: number, c1: string, c2: string, life: number): Particle[] {
  const particles: Particle[] = [];
  for (let i = 0; i < count; i++) {
    particles.push({
      x: cx + randRange(-20, 20),
      y: cy + randRange(-10, 10),
      vx: randRange(-5, 5),
      vy: randRange(-20, -5),
      life: life + randRange(0, 0.6),
      maxLife: life,
      size: randRange(3, 7),
      color: Math.random() > 0.5 ? c1 : c2,
      alpha: 1,
    });
  }
  return particles;
}

export const particleEngine = new ParticleEngine();
