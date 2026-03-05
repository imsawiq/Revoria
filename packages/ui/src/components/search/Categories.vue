<template>
	<div class="categories">
		<slot />
		<span
			v-for="(category, index) in normalizedCategories"
			:key="`${category.name}-${index}`"
			:class="[
				`tag-${category.name.toLowerCase().replace(/[^a-z0-9]+/g, '-')}`,
				{ 'is-loader': category.isLoader },
			]"
		>
			<span
				v-if="isHtmlIcon(category.icon)"
				class="tag-icon"
				v-html="category.icon"
			/>
			<img
				v-else-if="isImageIcon(category.icon)"
				class="tag-icon-image"
				:src="category.icon"
				alt=""
				loading="lazy"
			/>
			{{ formatCategory(category.name) }}
		</span>
		<slot name="suffix" />
	</div>
</template>
<script setup>
import { formatCategory } from '@modrinth/utils'
import { computed } from 'vue'

const props = defineProps({
	categories: {
		type: Array,
		default() {
			return []
		},
	},
})

const normalizedCategories = computed(() =>
	(props.categories ?? [])
		.map((category) => {
			if (typeof category === 'string') {
				return { name: category.trim(), icon: '' }
			}
			if (category && typeof category === 'object') {
				const name = String(category.name ?? '').trim()
				return {
					name,
					icon: typeof category.icon === 'string' ? category.icon : '',
					isLoader: Boolean(category.isLoader),
				}
			}
			return { name: '', icon: '', isLoader: false }
		})
		.filter((category) => category.name.length > 0),
)

const isHtmlIcon = (icon) => {
	if (typeof icon !== 'string') return false
	const value = icon.trim()
	return value.startsWith('<')
}

const isImageIcon = (icon) => {
	if (typeof icon !== 'string') return false
	const value = icon.trim().toLowerCase()
	return (
		value.startsWith('http://') ||
		value.startsWith('https://') ||
		value.startsWith('data:image/') ||
		value.startsWith('/')
	)
}
</script>

<style lang="scss" scoped>
.categories {
	display: flex;
	flex-direction: row;
	flex-wrap: wrap;
	gap: 0.4rem;

	> :deep(span:not(.server-ping)) {
		display: flex;
		flex-direction: row;
		align-items: center;
		gap: 0.3rem;
		min-height: 1.65rem;
		padding: 0.12rem 0.62rem;
		border: 1px solid var(--color-button-border);
		border-radius: 999px;
		background: color-mix(in srgb, var(--color-button-bg) 92%, transparent 8%);
		font-size: 0.98rem;
		line-height: 1.1;
		color: var(--color-secondary);
		transition:
			background-color 0.14s ease,
			border-color 0.14s ease,
			color 0.14s ease;
		cursor: pointer;

		&:not(.version-badge) {
			color: var(--color-secondary);
		}

		&:hover {
			background: color-mix(in srgb, var(--color-button-bg-hover) 78%, transparent 22%);
			border-color: color-mix(in srgb, var(--color-button-border) 40%, var(--color-brand) 60%);
			color: var(--color-contrast);
		}

		svg {
			width: 0.92rem;
			height: 0.92rem;
			margin-right: 0;
		}
	}

	:deep(.tag-icon) {
		display: inline-flex;
		align-items: center;
		line-height: 0;
		padding: 0;
		border: none;
		background: transparent;
		border-radius: 0;
	}

	:deep(.tag-icon-image) {
		width: 0.92rem;
		height: 0.92rem;
		object-fit: contain;
	}

	:deep(.tag-fabric),
	:deep(.tag-forge),
	:deep(.tag-neoforge),
	:deep(.tag-quilt),
	:deep(.tag-legacy-fabric),
	:deep(.tag-babric),
	:deep(.tag-liteloader),
	:deep(.tag-nilloader) {
		color: var(--color-platform);
	}

	:deep(.tag-fabric) {
		color: var(--color-platform-fabric);
	}

	:deep(.tag-forge) {
		color: var(--color-platform-forge);
	}

	:deep(.tag-neoforge) {
		color: var(--color-platform-neoforge);
	}

	:deep(.tag-quilt) {
		color: var(--color-platform-quilt);
	}
}
</style>
