<script setup lang="ts">
import { injectPageContext } from '../../providers'

const { hierarchicalSidebarAvailable } = injectPageContext()

defineProps<{
	title: string
}>()
</script>
<template>
	<div
		class="flex flex-col gap-2 p-3"
		:class="{
			'mb-3 last:mb-0 rounded-xl border border-solid border-divider bg-bg-raised':
				!hierarchicalSidebarAvailable,
		}"
	>
		<span class="font-semibold">{{ title }}</span>
		<slot />
	</div>
</template>
