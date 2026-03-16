<script setup lang="ts">
import { computed } from 'vue'
import { useVIntl } from '@vintl/vintl'

import type { MessageDescriptor } from '../../composables/i18n'
import { injectI18nDebug } from '../../composables/i18n-debug'

const props = defineProps<{
	messageId: MessageDescriptor
	values?: Record<string, unknown>
}>()

const { formatMessage } = useVIntl()
const debugContext = injectI18nDebug()

const debugEnabled = computed(() => debugContext?.enabled.value ?? false)
const debugKeyReveal = computed(() => debugContext?.keyReveal.value ?? false)

const formattedParts = computed(() => {
	const key = props.messageId.id
	const msg = formatMessage(props.messageId, props.values as Record<string, string | number>)

	if (debugEnabled.value) {
		debugContext!.registry.set(key, {
			key,
			value: msg,
			defaultMessage: props.messageId.defaultMessage,
			timestamp: Date.now(),
		})
		if (debugKeyReveal.value) {
			return [`\u300C${key}\u300D`]
		}
	}
	return [msg]
})
</script>

<template>
	<span
		v-if="debugEnabled && !debugKeyReveal"
		:data-i18n-key="messageId.id"
		style="display: contents"
	>
		<template v-for="(part, index) in formattedParts" :key="index">
			<component :is="() => part" v-if="typeof part === 'object'" />
			<template v-else>{{ part }}</template>
		</template>
	</span>
	<template v-for="(part, index) in formattedParts" v-else :key="index">
		<component :is="() => part" v-if="typeof part === 'object'" />
		<template v-else>{{ part }}</template>
	</template>
</template>
