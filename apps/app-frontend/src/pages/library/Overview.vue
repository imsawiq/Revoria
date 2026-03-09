<script setup>
import { defineMessages, useVIntl } from '@vintl/vintl'

import GridDisplay from '@/components/GridDisplay.vue'

const { formatMessage } = useVIntl()
const messages = defineMessages({
	tabAll: {
		id: 'app.library.tab.all',
		defaultMessage: 'All instances',
	},
})

const props = defineProps({
	instances: {
		type: Array,
		default: () => [],
	},
})
</script>
<template>
	<GridDisplay
		v-if="props.instances.length > 0"
		:label="formatMessage(messages.tabAll)"
		:instances="props.instances"
	/>
</template>
