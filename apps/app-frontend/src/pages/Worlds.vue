<script setup lang="ts">
import { defineMessages, useVIntl } from '@vintl/vintl'

const { formatMessage } = useVIntl()
const messages = defineMessages({
	pageTitle: {
		id: 'app.worlds.page-title',
		defaultMessage: 'Worlds',
	},
})
</script>
<template>
	<div class="p-6 flex flex-col gap-2">{{ formatMessage(messages.pageTitle) }}</div>
</template>
