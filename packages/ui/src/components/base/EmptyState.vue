<template>
	<div class="mx-auto flex flex-col items-center p-6 text-center">
		<component :is="illustration" v-if="illustration" class="h-[200px] w-auto" />
		<div class="flex flex-col items-center gap-1.5">
			<span class="text-2xl font-semibold text-contrast">
				<slot name="heading">{{ heading }}</slot>
			</span>
			<span v-if="$slots.description || description" class="text-secondary">
				<slot name="description">{{ description }}</slot>
			</span>
		</div>
		<div v-if="$slots.actions" class="mt-8 flex gap-2">
			<slot name="actions" />
		</div>
	</div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
	type?: string
	heading?: string
	description?: string
}>()

const illustration = computed(() => undefined)
</script>
