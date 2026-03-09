<script setup>
import { defineMessages, useVIntl } from '@vintl/vintl'

import GridDisplay from '@/components/GridDisplay.vue'

const { formatMessage } = useVIntl()
const messages = defineMessages({
	tabDownloaded: {
		id: 'app.library.tab.downloaded',
		defaultMessage: 'Downloaded',
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
		:label="formatMessage(messages.tabDownloaded)"
		:instances="props.instances.filter((i) => i.linked_data)"
	/>
</template>
