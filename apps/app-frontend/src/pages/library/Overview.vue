<script setup>
import GridDisplay from '@/components/GridDisplay.vue'

const props = defineProps({
	instances: {
		type: Array,
		default: () => [],
	},
})
</script>
<template>
	<GridDisplay v-if="props.instances.length > 0" label="Instances" :instances="props.instances" />
</template>
