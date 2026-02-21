<template>
	<nav
		class="experimental-styles-within relative flex w-full max-w-full overflow-x-auto overflow-y-hidden rounded-full bg-[--color-glass-bg-strong] p-1 text-sm font-bold border border-[--glass-border] shadow-[--glass-shadow]"
	>
		<RouterLink
			v-for="(link, index) in filteredLinks"
			v-show="link.shown === undefined ? true : link.shown"
			:key="index"
			ref="tabLinkElements"
			:to="buildTo(link)"
			:class="`navtab-link button-animation shrink-0 z-[1] flex flex-row items-center gap-2 px-4 py-2 focus:rounded-full ${activeIndex === index && !subpageSelected ? 'text-button-textSelected is-active' : activeIndex === index && subpageSelected ? 'text-contrast is-subpage' : 'text-primary'}`"
		>
			<component :is="link.icon" v-if="link.icon" class="size-5" />
			<span class="text-nowrap">{{ link.label }}</span>
		</RouterLink>
		<div
			:class="`navtabs-transition pointer-events-none absolute left-0 top-0 overflow-hidden rounded-full ${subpageSelected ? 'bg-button-bg' : 'bg-button-bgSelected'}`"
			:style="{
				transform: `translate3d(${sliderX}px, ${sliderY}px, 0)`,
				width: sliderWidthPx,
				height: sliderHeightPx,
				opacity: activeIndex === -1 || sliderWidth === 0 ? 0 : 1,
			}"
			aria-hidden="true"
		></div>
	</nav>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import type { RouteLocationRaw } from 'vue-router'
import { RouterLink, useRoute } from 'vue-router'

const route = useRoute()

interface Tab {
	label: string
	href: string | RouteLocationRaw
	shown?: boolean
	icon?: unknown
	subpages?: string[]
}

const props = defineProps<{
	links: Tab[]
	query?: string
}>()

const sliderX = ref(0)
const sliderY = ref(0)
const sliderWidth = ref(0)
const sliderHeight = ref(0)
const activeIndex = ref(-1)
const oldIndex = ref(-1)
const subpageSelected = ref(false)

const filteredLinks = computed(() =>
	props.links.filter((x) => (x.shown === undefined ? true : x.shown)),
)
const sliderWidthPx = computed(() => `${sliderWidth.value}px`)
const sliderHeightPx = computed(() => `${sliderHeight.value}px`)

function buildTo(link: Tab): RouteLocationRaw {
	if (!props.query) return link.href

	const nextQuery: Record<string, any> = { ...route.query }
	if (typeof link.href === 'string' && link.href.length > 0) {
		nextQuery[props.query] = link.href
	} else {
		delete nextQuery[props.query]
	}

	return {
		path: route.path,
		query: nextQuery,
	}
}

function pickLink() {
	let index = -1
	subpageSelected.value = false
	if (props.query) {
		const current = route.query[props.query]
		for (let i = filteredLinks.value.length - 1; i >= 0; i--) {
			const link = filteredLinks.value[i]
			if (String(current ?? '') === String(link.href ?? '')) {
				index = i
				break
			}
		}
		activeIndex.value = index
		if (activeIndex.value !== -1) {
			startAnimation()
		} else {
			oldIndex.value = -1
			sliderX.value = 0
			sliderY.value = 0
			sliderWidth.value = 0
			sliderHeight.value = 0
		}
		return
	}
	for (let i = filteredLinks.value.length - 1; i >= 0; i--) {
		const link = filteredLinks.value[i]

		if (route.path === (typeof link.href === 'string' ? link.href : link.href.path)) {
			index = i
			break
		} else if (link.subpages && link.subpages.some((subpage) => route.path.includes(subpage))) {
			index = i
			subpageSelected.value = true
			break
		}
	}
	activeIndex.value = index

	if (activeIndex.value !== -1) {
		startAnimation()
	} else {
		oldIndex.value = -1
		sliderX.value = 0
		sliderY.value = 0
		sliderWidth.value = 0
		sliderHeight.value = 0
	}
}

const tabLinkElements = ref()

function startAnimation() {
	if (!tabLinkElements.value || !tabLinkElements.value[activeIndex.value]) return
	const el = tabLinkElements.value[activeIndex.value].$el

	if (!el || !el.offsetParent) return

	const parent = el.offsetParent
	const parentRect = parent.getBoundingClientRect()
	const elRect = el.getBoundingClientRect()
	const scrollLeft = parent.scrollLeft ?? 0
	const scrollTop = parent.scrollTop ?? 0

	sliderX.value = Math.round(elRect.left - parentRect.left + scrollLeft - (parent.clientLeft || 0))
	sliderY.value = Math.max(
		0,
		Math.round(elRect.top - parentRect.top + scrollTop - (parent.clientTop || 0)),
	)
	sliderWidth.value = Math.round(elRect.width)
	sliderHeight.value = Math.round(elRect.height)
}

onMounted(() => {
	window.addEventListener('resize', pickLink)
	pickLink()
})

onUnmounted(() => {
	window.removeEventListener('resize', pickLink)
})

watch(route, () => {
	pickLink()
})
</script>
<style scoped>
.navtabs-transition {
	/* Delay on opacity is to hide any jankiness as the page loads */
	transition:
		transform 420ms cubic-bezier(0.16, 1, 0.3, 1),
		width 340ms cubic-bezier(0.22, 1, 0.36, 1),
		height 260ms cubic-bezier(0.22, 1, 0.36, 1),
		opacity 250ms cubic-bezier(0.5, 0, 0.2, 1) 40ms,
		background-color 220ms ease;
	border: 1px solid color-mix(in srgb, var(--color-brand) 24%, transparent);
}

.navtab-link {
	position: relative;
	transition:
		transform 220ms cubic-bezier(0.2, 0.8, 0.2, 1),
		color 180ms ease,
		opacity 180ms ease;
}

.navtab-link:hover {
	transform: translateY(-1px);
}

.navtab-link:active {
	transform: translateY(0) scale(0.985);
}

.navtab-link::after {
	content: '';
	position: absolute;
	left: 20%;
	right: 20%;
	bottom: 0.3rem;
	height: 2px;
	border-radius: 999px;
	background: color-mix(in srgb, var(--color-brand) 68%, transparent);
	opacity: 0;
	transform: scaleX(0.65);
	transition: all 240ms cubic-bezier(0.2, 0.8, 0.2, 1);
}

.navtab-link.is-active::after,
.navtab-link.is-subpage::after {
	opacity: 1;
	transform: scaleX(1);
}
</style>
