<script setup>
import { computed, onMounted, onUnmounted, ref } from 'vue'

import { isAprilFoolsActive } from '@/helpers/april-fools.js'

const active = computed(() => isAprilFoolsActive())
const visible = ref(false)
const mouseX = ref(window.innerWidth / 2)
const mouseY = ref(window.innerHeight / 2)
const poops = ref([])

let spawnTimeout
let autoHideTimeout
let nextId = 0

function clearTimers() {
	if (spawnTimeout) {
		clearTimeout(spawnTimeout)
		spawnTimeout = undefined
	}
	if (autoHideTimeout) {
		clearTimeout(autoHideTimeout)
		autoHideTimeout = undefined
	}
}

function scheduleNext() {
	if (!active.value || visible.value) return
	const delay = Math.round(18000 + Math.random() * 26000)
	spawnTimeout = setTimeout(spawnCleanupEvent, delay)
}

function randomBetween(min, max) {
	return min + Math.random() * (max - min)
}

function spawnCleanupEvent() {
	if (!active.value) return

	const count = Math.round(randomBetween(11, 18))
	poops.value = Array.from({ length: count }, () => ({
		id: ++nextId,
		x: Math.round(randomBetween(40, Math.max(60, window.innerWidth - 90))),
		y: Math.round(randomBetween(80, Math.max(120, window.innerHeight - 140))),
		rotation: Math.round(randomBetween(-24, 24)),
		scale: Number(randomBetween(0.92, 1.32).toFixed(2)),
	}))

	visible.value = true

	autoHideTimeout = setTimeout(() => {
		poops.value = []
		visible.value = false
		scheduleNext()
	}, 15000)
}

function finishCleanup() {
	poops.value = []
	visible.value = false
	clearTimers()
	scheduleNext()
}

function tryClean(clientX, clientY) {
	if (!visible.value) return
	poops.value = poops.value.filter((poop) => {
		const dx = poop.x - clientX
		const dy = poop.y - clientY
		return Math.sqrt(dx * dx + dy * dy) > 58
	})

	if (poops.value.length === 0) {
		finishCleanup()
	}
}

function onMouseMove(event) {
	mouseX.value = event.clientX
	mouseY.value = event.clientY
	tryClean(event.clientX, event.clientY)
}

onMounted(() => {
	window.addEventListener('mousemove', onMouseMove)
	scheduleNext()
})

onUnmounted(() => {
	window.removeEventListener('mousemove', onMouseMove)
	clearTimers()
})
</script>

<template>
	<div v-if="visible" class="april-fools-cleanup-layer" aria-hidden="true">
		<div class="april-fools-cleanup-banner">
			{{ $t('app.april-fools.cleanup.message') }}
		</div>

		<div
			v-for="poop in poops"
			:key="poop.id"
			class="april-fools-poop"
			:style="{
				left: `${poop.x}px`,
				top: `${poop.y}px`,
				transform: `translate(-50%, -50%) rotate(${poop.rotation}deg) scale(${poop.scale})`,
			}"
		>
			💩
		</div>

		<div
			class="april-fools-mop"
			:style="{
				left: `${mouseX}px`,
				top: `${mouseY}px`,
			}"
		>
			🧹
		</div>
	</div>
</template>

<style scoped>
.april-fools-cleanup-layer {
	position: fixed;
	inset: 0;
	z-index: 90;
	pointer-events: none;
}

.april-fools-cleanup-banner {
	position: fixed;
	top: 1rem;
	left: 50%;
	transform: translateX(-50%);
	padding: 1rem 1.6rem;
	border-radius: 1.2rem;
	background:
		linear-gradient(
			135deg,
			color-mix(in srgb, var(--color-glass-bg-strong) 78%, rgba(255, 56, 126, 0.38)),
			color-mix(in srgb, var(--color-glass-bg-strong) 84%, rgba(255, 196, 72, 0.22))
		);
	border: 1px solid color-mix(in srgb, var(--color-brand) 42%, var(--glass-border));
	box-shadow:
		0 18px 48px rgba(0, 0, 0, 0.34),
		0 0 0 1px color-mix(in srgb, rgba(255, 255, 255, 0.08) 60%, transparent);
	color: var(--color-contrast);
	font-size: 1.15rem;
	font-weight: 900;
	letter-spacing: 0.01em;
	text-align: center;
	min-width: min(42rem, calc(100vw - 2rem));
	backdrop-filter: blur(18px) saturate(1.18);
	-webkit-backdrop-filter: blur(18px) saturate(1.18);
}

.april-fools-poop {
	position: fixed;
	font-size: 2.45rem;
	line-height: 1;
	filter: drop-shadow(0 10px 14px rgba(0, 0, 0, 0.35));
	animation: april-fools-poop-bob 1.8s ease-in-out infinite;
}

.april-fools-mop {
	position: fixed;
	font-size: 2rem;
	line-height: 1;
	transform: translate(-18%, -82%) rotate(28deg);
	filter: drop-shadow(0 10px 18px rgba(0, 0, 0, 0.3));
	transition: left 55ms linear, top 55ms linear;
}

@keyframes april-fools-poop-bob {
	0%,
	100% {
		filter: drop-shadow(0 10px 14px rgba(0, 0, 0, 0.35));
	}

	50% {
		filter: drop-shadow(0 14px 18px rgba(0, 0, 0, 0.42));
	}
}
</style>
