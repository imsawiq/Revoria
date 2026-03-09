<template>
	<RouterLink
		v-if="typeof to === 'string'"
		:to="to"
		v-bind="$attrs"
		:class="{
			'router-link-active': isPrimary && isPrimary(route),
			'subpage-active': isSubpage && isSubpage(route),
			disabled: disabled,
			'nav-button-attention': highlightOverride,
		}"
		class="w-10 h-10 text-primary rounded-2xl grid place-items-center text-2xl transition-all bg-transparent hover:bg-button-bg hover:text-contrast focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[--color-brand] focus-visible:ring-offset-2 focus-visible:ring-offset-[--color-bg]"
	>
		<slot />
	</RouterLink>
	<button
		v-else
		v-bind="$attrs"
		:class="[
			'button-animation border-none text-primary cursor-pointer w-10 h-10 rounded-2xl grid place-items-center text-2xl transition-all bg-transparent hover:bg-button-bg hover:text-contrast focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[--color-brand] focus-visible:ring-offset-2 focus-visible:ring-offset-[--color-bg]',
			{ 'nav-button-attention': highlightOverride },
		]"
		:disabled="disabled"
		@click="to"
	>
		<slot />
	</button>
</template>

<script setup lang="ts">
import type { RouteLocationNormalizedLoaded } from 'vue-router'
import { RouterLink, useRoute } from 'vue-router'

const route = useRoute()

type RouteFunction = (route: RouteLocationNormalizedLoaded) => boolean

withDefaults(
	defineProps<{
		to: (() => void) | string
		isPrimary?: RouteFunction
		isSubpage?: RouteFunction
		highlightOverride?: boolean
		disabled?: boolean
	}>(),
	{
		disabled: false,
	},
)

defineOptions({
	inheritAttrs: false,
})
</script>

<style lang="scss" scoped>
.router-link-active,
.subpage-active,
button,
a {
	line-height: 0;

	:deep(svg) {
		display: block;
		width: 1.25rem;
		height: 1.25rem;
	}
}

.router-link-active,
.subpage-active {
	svg {
		filter: drop-shadow(0 0 0.5rem black);
	}
}

.router-link-active {
	color: var(--color-button-text-selected);
	background: var(--color-button-bg-selected);
}

.subpage-active {
	color: var(--color-contrast);
	background: var(--color-button-bg);
}

.nav-button-attention {
	color: var(--color-button-text-selected);
	background: var(--color-button-bg-selected);
	box-shadow: 0 0 0 1px var(--color-brand-highlight);
	animation: none;

	svg {
		filter: none;
	}
}
</style>
