<script setup>
import { computed, onMounted, onUnmounted, ref } from 'vue'

import { isAprilFoolsActive } from '@/helpers/april-fools.js'

const imageModules = import.meta.glob('../../assets/april-fools-popups/*.{png,jpg,jpeg,webp,avif,gif}', {
	eager: true,
	import: 'default',
})

const imageUrls = Object.values(imageModules)
const isActive = computed(() => isAprilFoolsActive() && imageUrls.length > 0)

const visible = ref(false)
const currentImage = ref('')
const popupStyle = ref({})

let showTimeout
let hideTimeout

function clearTimers() {
	if (showTimeout) {
		clearTimeout(showTimeout)
		showTimeout = undefined
	}
	if (hideTimeout) {
		clearTimeout(hideTimeout)
		hideTimeout = undefined
	}
}

function scheduleNext() {
	if (!isActive.value) return
	const delay = Math.round(6000 + Math.random() * 18000)
	showTimeout = setTimeout(showRandomImage, delay)
}

function showRandomImage() {
	if (!isActive.value || imageUrls.length === 0) return

	const width = window.innerWidth
	const height = window.innerHeight
	const size = Math.round(140 + Math.random() * 180)
	const left = Math.max(12, Math.round(Math.random() * Math.max(12, width - size - 24)))
	const top = Math.max(12, Math.round(Math.random() * Math.max(12, height - size - 24)))
	const rotate = Math.round(-18 + Math.random() * 36)
	const scale = (0.85 + Math.random() * 0.35).toFixed(2)

	currentImage.value = imageUrls[Math.floor(Math.random() * imageUrls.length)]
	popupStyle.value = {
		left: `${left}px`,
		top: `${top}px`,
		width: `${size}px`,
		transform: `rotate(${rotate}deg) scale(${scale})`,
	}
	visible.value = true

	hideTimeout = setTimeout(() => {
		visible.value = false
		currentImage.value = ''
		scheduleNext()
	}, 4200)
}

onMounted(() => {
	scheduleNext()
})

onUnmounted(() => {
	clearTimers()
})
</script>

<template>
	<transition name="april-popup">
		<img
			v-if="visible && currentImage"
			:key="currentImage"
			:src="currentImage"
			alt=""
			class="april-popup-image"
			:style="popupStyle"
		/>
	</transition>
</template>

<style scoped>
.april-popup-image {
	position: fixed;
	z-index: 70;
	pointer-events: none;
	user-select: none;
	filter: drop-shadow(0 22px 36px rgba(0, 0, 0, 0.38));
}

.april-popup-enter-active,
.april-popup-leave-active {
	transition:
		opacity 180ms ease,
		transform 180ms ease;
}

.april-popup-enter-from,
.april-popup-leave-to {
	opacity: 0;
	transform: scale(0.78) rotate(-8deg);
}
</style>
