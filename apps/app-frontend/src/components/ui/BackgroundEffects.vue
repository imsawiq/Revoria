<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'

import { useTheming } from '@/store/state'

type EffectType = 'off' | 'snow' | 'stars' | 'rain'

type Particle = {
	x: number
	y: number
	vx: number
	vy: number
	size: number
	alpha: number
	phase: number
}

const themeStore = useTheming()
const MAX_EFFECT_FPS = 36
const MAX_EFFECT_DPR = 1.25

const effect = computed(() => themeStore.backgroundEffect as EffectType)
const intensity = computed(() => Math.max(10, Math.min(400, themeStore.backgroundEffectIntensity)) / 100)

const canvasEl = ref<HTMLCanvasElement | null>(null)
let ctx: CanvasRenderingContext2D | null = null

let rafId: number | null = null
let particles: Particle[] = []
let lastTs = 0
let reinitRafId: number | null = null
let lastBlendModeKey = ''

const reducedMotion = ref(false)
let mediaQuery: MediaQueryList | null = null
let mediaQueryHandler: ((event: MediaQueryListEvent) => void) | null = null

function isLightTheme() {
	return document.documentElement.classList.contains('light-mode')
}

function getThemeColor(variable: string, fallback: string) {
	const value = getComputedStyle(document.documentElement).getPropertyValue(variable).trim()
	return value || fallback
}

function applyCanvasBlendMode() {
	const canvas = canvasEl.value
	if (!canvas) return
	const blendKey = `${isLightTheme() ? 'light' : 'dark'}:${effect.value}`
	if (blendKey === lastBlendModeKey) return
	lastBlendModeKey = blendKey
	if (isLightTheme()) {
		canvas.style.mixBlendMode = effect.value === 'rain' ? 'normal' : 'multiply'
		canvas.style.opacity = effect.value === 'rain' ? '1' : '0.92'
		return
	}
	canvas.style.mixBlendMode = 'normal'
	canvas.style.opacity = '1'
}

function getPalette() {
	return {
		snow: getThemeColor('--color-contrast', isLightTheme() ? '#204559' : '#FFFFFF'),
		rain: getThemeColor('--color-brand', isLightTheme() ? '#245C76' : '#99CAFF'),
		stars: getThemeColor('--color-text-primary', isLightTheme() ? '#2A5065' : '#E0ECFF'),
		starsGlow: getThemeColor('--color-brand', isLightTheme() ? '#75ABC4' : '#96C6FF'),
	}
}

function rand(min: number, max: number) {
	return min + Math.random() * (max - min)
}

function clamp(n: number, min: number, max: number) {
	return Math.max(min, Math.min(max, n))
}

function getBounds() {
	const canvas = canvasEl.value
	if (!canvas) return { w: 0, h: 0, dpr: 1 }
	const rect = canvas.parentElement?.getBoundingClientRect() ?? canvas.getBoundingClientRect()
	const dpr = Math.min(window.devicePixelRatio || 1, MAX_EFFECT_DPR)
	return { w: Math.max(1, rect.width), h: Math.max(1, rect.height), dpr }
}

function resizeCanvas() {
	const canvas = canvasEl.value
	if (!canvas) return
	const { w, h, dpr } = getBounds()
	canvas.width = Math.floor(w * dpr)
	canvas.height = Math.floor(h * dpr)
	canvas.style.width = `${w}px`
	canvas.style.height = `${h}px`
	if (ctx) ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
}

function initParticles(type: EffectType) {
	const { w, h } = getBounds()
	particles = []

	if (type === 'off') return

	const area = w * h
	const density = intensity.value
	const baseCount = clamp(Math.floor((area / 42000) * density), 10, 320)

	if (type === 'snow') {
		for (let i = 0; i < baseCount; i++) {
			particles.push({
				x: rand(0, w),
				y: rand(0, h),
				vx: rand(-10, 10),
				vy: rand(16, 46) * Math.max(0.85, density * 0.95),
				size: rand(1.0, 2.8),
				alpha: rand(0.18, 0.62),
				phase: rand(0, Math.PI * 2),
			})
		}
	} else if (type === 'rain') {
		for (let i = 0; i < clamp(baseCount + Math.round(15 * density), 24, 220); i++) {
			particles.push({
				x: rand(0, w),
				y: rand(0, h),
				vx: rand(-34, -12),
				vy: rand(280, 520) * Math.max(0.9, density),
				size: rand(8, 16),
				alpha: isLightTheme() ? rand(0.18, 0.34) : rand(0.08, 0.18),
				phase: rand(0, Math.PI * 2),
			})
		}
	} else if (type === 'stars') {
		for (let i = 0; i < clamp(Math.floor((baseCount + 72) * 2), 60, 420); i++) {
			particles.push({
				x: rand(0, w),
				y: rand(0, h),
				vx: rand(-0.9, 0.9) * density,
				vy: rand(-0.6, 0.6) * density,
				size: rand(0.45, 1.4),
				alpha: rand(0.34, 0.82),
				phase: rand(0, Math.PI * 2),
			})
		}
	}
}

function stop() {
	if (rafId != null) {
		cancelAnimationFrame(rafId)
		rafId = null
	}
	if (reinitRafId != null) {
		cancelAnimationFrame(reinitRafId)
		reinitRafId = null
	}
	lastTs = 0
	lastBlendModeKey = ''
}

function clear() {
	if (!ctx) return
	const { w, h } = getBounds()
	ctx.clearRect(0, 0, w, h)
}

function shouldRun(type: EffectType) {
	if (type === 'off') return false
	if (reducedMotion.value) return false
	if (document.visibilityState !== 'visible') return false
	return true
}

function minimumVisibleParticles(type: EffectType) {
	if (type === 'stars') return 24
	if (type === 'rain') return 16
	if (type === 'snow') return 10
	return 0
}

function resetScene(force = false) {
	const type = effect.value
	if (!ctx) return

	resizeCanvas()
	const { w, h } = getBounds()
	if (!force && (!w || !h || w < 32 || h < 32)) return

	initParticles(type)
	clear()

	if (!shouldRun(type)) {
		stop()
		return
	}

	if (particles.length < minimumVisibleParticles(type)) {
		initParticles(type)
	}

	stop()
	lastTs = 0
	rafId = requestAnimationFrame(tick)
}

function scheduleReset(force = false) {
	if (reinitRafId != null) cancelAnimationFrame(reinitRafId)
	reinitRafId = requestAnimationFrame(() => {
		reinitRafId = requestAnimationFrame(() => {
			reinitRafId = null
			resetScene(force)
		})
	})
}

function tick(ts: number) {
	const type = effect.value
	if (!ctx || !shouldRun(type)) {
		stop()
		clear()
		return
	}

	const { w, h } = getBounds()
	if (!w || !h) {
		scheduleReset(true)
		return
	}

	if (particles.length < minimumVisibleParticles(type)) {
		scheduleReset(true)
		return
	}

	if (!lastTs) lastTs = ts
	const elapsed = ts - lastTs
	if (elapsed < 1000 / MAX_EFFECT_FPS) {
		rafId = requestAnimationFrame(tick)
		return
	}
	const dt = Math.min(0.033, elapsed / 1000)
	lastTs = ts

	ctx.clearRect(0, 0, w, h)
	const palette = getPalette()
	applyCanvasBlendMode()

	if (type === 'snow') {
		ctx.fillStyle = palette.snow
		for (const p of particles) {
			p.phase += dt * 0.9
			p.x += (p.vx + Math.sin(p.phase) * 10) * dt
			p.y += p.vy * dt
			if (p.y > h + 10) {
				p.y = -10
				p.x = rand(0, w)
			}
			if (p.x < -20) p.x = w + 20
			if (p.x > w + 20) p.x = -20
			ctx.globalAlpha = p.alpha
			ctx.beginPath()
			ctx.arc(p.x, p.y, p.size, 0, Math.PI * 2)
			ctx.fill()
		}
		ctx.globalAlpha = 1
	} else if (type === 'rain') {
		ctx.strokeStyle = palette.rain
		ctx.lineCap = 'round'
		for (const p of particles) {
			p.x += p.vx * dt
			p.y += p.vy * dt
			if (p.y > h + 40) {
				p.y = rand(-120, -20)
				p.x = rand(0, w)
			}
			if (p.x < -60) p.x = w + 60
			ctx.globalAlpha = p.alpha
			ctx.lineWidth = isLightTheme() ? 1.35 : 1
			ctx.beginPath()
			ctx.moveTo(p.x, p.y)
			ctx.lineTo(p.x - p.size * 0.8, p.y + p.size * 2.2)
			ctx.stroke()
		}
		ctx.globalAlpha = 1
	} else if (type === 'stars') {
		for (const p of particles) {
			p.phase += dt * 0.9
			p.x += p.vx * dt
			p.y += p.vy * dt
			if (p.x < -8) p.x = w + 8
			if (p.x > w + 8) p.x = -8
			if (p.y < -8) p.y = h + 8
			if (p.y > h + 8) p.y = -8
			const twinkle = (Math.sin(p.phase) + 1) / 2
			const glow = p.size * (0.8 + twinkle * 0.5)
			const sparkle = p.size > 1.0 && twinkle > 0.78
			ctx.globalAlpha = Math.min(0.62, p.alpha * 0.2 + twinkle * 0.08)
			ctx.fillStyle = palette.starsGlow
			ctx.beginPath()
			ctx.arc(p.x, p.y, glow * 1.5, 0, Math.PI * 2)
			ctx.fill()
			ctx.globalAlpha = Math.min(1, p.alpha + twinkle * 0.18)
			ctx.fillStyle = palette.stars
			ctx.beginPath()
			ctx.arc(p.x, p.y, Math.max(0.4, p.size * 0.55), 0, Math.PI * 2)
			ctx.fill()
			if (sparkle) {
				ctx.globalAlpha = Math.min(0.46, 0.08 + twinkle * 0.14)
				ctx.fillRect(p.x - glow * 1.1, p.y - 0.12, glow * 2.2, 0.24)
				ctx.fillRect(p.x - 0.12, p.y - glow * 1.1, 0.24, glow * 2.2)
			}
		}
		ctx.globalAlpha = 1
	}

	rafId = requestAnimationFrame(tick)
}

function startIfNeeded() {
	const type = effect.value
	if (!ctx) return
	if (!shouldRun(type)) {
		stop()
		clear()
		return
	}
	if (particles.length < minimumVisibleParticles(type)) {
		scheduleReset(true)
		return
	}
	if (rafId == null) {
		lastTs = 0
		rafId = requestAnimationFrame(tick)
	}
}

function handleVisibility() {
	if (document.visibilityState === 'visible') {
		scheduleReset(true)
	} else {
		startIfNeeded()
	}
}

let resizeObserver: ResizeObserver | null = null
let resizeTarget: Element | null = null
let windowResizeHandler: (() => void) | null = null
let windowFocusHandler: (() => void) | null = null

onMounted(() => {
	const canvas = canvasEl.value
	if (!canvas) return
	ctx = canvas.getContext('2d', { alpha: true, desynchronized: true })
	if (!ctx) return
	applyCanvasBlendMode()

	mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)')
	reducedMotion.value = mediaQuery.matches
	mediaQueryHandler = (e) => {
		reducedMotion.value = e.matches
		scheduleReset(true)
	}
	mediaQuery.addEventListener('change', mediaQueryHandler)

	document.addEventListener('visibilitychange', handleVisibility)
	windowResizeHandler = () => scheduleReset(true)
	windowFocusHandler = () => scheduleReset(true)
	window.addEventListener('resize', windowResizeHandler)
	window.addEventListener('focus', windowFocusHandler)
	resizeObserver = new ResizeObserver(() => {
		scheduleReset(true)
	})
	resizeTarget = canvas.parentElement ?? canvas
	resizeObserver.observe(resizeTarget)
	scheduleReset(true)
})

onUnmounted(() => {
	stop()
	document.removeEventListener('visibilitychange', handleVisibility)
	if (windowResizeHandler) window.removeEventListener('resize', windowResizeHandler)
	if (windowFocusHandler) window.removeEventListener('focus', windowFocusHandler)
	resizeObserver?.disconnect()
	reduceMotionCleanup()
})

function reduceMotionCleanup() {
	if (!mediaQuery) return
	try {
		if (mediaQueryHandler) mediaQuery.removeEventListener('change', mediaQueryHandler)
	} catch {
		// ignore
	}
	mediaQueryHandler = null
	mediaQuery = null
}

watch(
	() => [effect.value, intensity.value],
	([next]) => {
		if (next === 'off') {
			stop()
			clear()
			return
		}
		scheduleReset(true)
	},
	{ immediate: false },
)
</script>

<template>
	<canvas ref="canvasEl" class="effects-canvas" aria-hidden="true" />
</template>

<style scoped>
.effects-canvas {
	position: absolute;
	inset: 0;
	width: 100%;
	height: 100%;
	pointer-events: none;
	z-index: 0;
	mix-blend-mode: screen;
}
</style>
