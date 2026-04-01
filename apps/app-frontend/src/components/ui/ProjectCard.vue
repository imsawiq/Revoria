<script setup>
import { DownloadIcon, HeartIcon, TagIcon } from '@modrinth/assets'
import { Avatar } from '@modrinth/ui'
import { formatCategory, formatNumber } from '@modrinth/utils'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { computed } from 'vue'
import { useRouter } from 'vue-router'

dayjs.extend(relativeTime)

const router = useRouter()

const props = defineProps({
	project: {
		type: Object,
		default() {
			return {}
		},
	},
})

const featuredCategory = computed(() => {
	if (props.project.display_categories.includes('optimization')) {
		return 'optimization'
	}

	return props.project.display_categories[0] ?? props.project.categories[0]
})

const accentColor = computed(() => {
	let color = props.project.color

	color >>>= 0
	const b = color & 0xff
	const g = (color >>> 8) & 0xff
	const r = (color >>> 16) & 0xff
	return `rgba(${[r, g, b].join(',')}, 1)`
})

const accentGlow = computed(() => {
	let color = props.project.color

	color >>>= 0
	const b = color & 0xff
	const g = (color >>> 8) & 0xff
	const r = (color >>> 16) & 0xff
	return `linear-gradient(180deg, rgba(${[r, g, b].join(',')}, 0.08) 0%, rgba(${[r, g, b].join(',')}, 0.26) 100%)`
})
</script>

<template>
	<article
		class="home-project-card group relative overflow-hidden rounded-[1.35rem] cursor-pointer"
		@click="router.push(`/project/${project.slug}`)"
	>
		<div class="home-project-card__media">
			<div
				class="home-project-card__cover"
				:style="{
					'background-color': (project.featured_gallery ?? project.gallery[0]) ? null : accentColor,
					'background-image': `url(${
						project.featured_gallery ??
						project.gallery[0] ??
						'https://launcher-files.modrinth.com/assets/maze-bg.png'
					})`,
				}"
			></div>
			<div class="home-project-card__cover-overlay"></div>
			<div
				v-if="!project.featured_gallery && !project.gallery[0]"
				class="home-project-card__accent"
				:style="{ background: accentGlow }"
			></div>
		</div>

		<div class="home-project-card__body">
			<div class="home-project-card__header">
				<Avatar size="3.25rem" :src="project.icon_url" class="home-project-card__avatar" />
				<div class="min-w-0">
					<h3 class="m-0 line-clamp-2 text-[1.05rem] font-extrabold leading-tight text-contrast">
						{{ project.title }}
					</h3>
					<p class="m-0 mt-1 text-sm font-medium text-secondary line-clamp-2 leading-snug">
						{{ project.description }}
					</p>
				</div>
			</div>

			<div class="home-project-card__meta">
				<div class="home-project-card__metric">
					<DownloadIcon class="size-4" />
					<span>{{ formatNumber(project.downloads) }}</span>
				</div>
				<div class="home-project-card__metric">
					<HeartIcon class="size-4" />
					<span>{{ formatNumber(project.follows) }}</span>
				</div>
				<div class="home-project-card__metric home-project-card__metric--tag">
					<TagIcon class="size-4" />
					<span class="home-project-card__tag-label">{{ formatCategory(featuredCategory) }}</span>
				</div>
			</div>
		</div>
	</article>
</template>

<style scoped lang="scss">
.home-project-card {
	display: flex;
	flex-direction: column;
	border: 1px solid color-mix(in srgb, var(--glass-border) 84%, transparent);
	background: color-mix(in srgb, var(--color-glass-bg-strong) 92%, transparent);
	box-shadow: var(--shadow-card);
	transition:
		transform 180ms ease,
		border-color 180ms ease,
		box-shadow 180ms ease,
		background 180ms ease;
}

.home-project-card:hover {
	transform: translateY(-2px);
	border-color: color-mix(in srgb, var(--color-brand) 24%, var(--glass-border));
	box-shadow: var(--shadow-floating);
	background: color-mix(in srgb, var(--color-glass-bg-strong) 96%, var(--color-brand) 3%);
}

.home-project-card__media {
	position: relative;
	aspect-ratio: 2.1 / 1;
	overflow: hidden;
	border-bottom: 1px solid color-mix(in srgb, var(--glass-border) 72%, transparent);
}

.home-project-card__cover {
	position: absolute;
	inset: 0;
	background-size: cover;
	background-position: center;
	transform: scale(1.01);
}

.home-project-card__cover-overlay {
	position: absolute;
	inset: 0;
	background:
		linear-gradient(180deg, rgba(0, 0, 0, 0.02) 0%, rgba(0, 0, 0, 0.22) 100%),
		linear-gradient(0deg, color-mix(in srgb, var(--color-bg) 12%, transparent), transparent);
}

.home-project-card__accent {
	position: absolute;
	inset: 0;
	mix-blend-mode: screen;
}

.home-project-card__body {
	display: flex;
	flex-direction: column;
	flex: 1 1 auto;
	gap: 0.9rem;
	padding: 1rem;
}

.home-project-card__header {
	display: grid;
	grid-template-columns: auto 1fr;
	gap: 0.85rem;
	align-items: center;
	min-height: 4.75rem;
}

.home-project-card__avatar {
	border-radius: 1rem;
	border: 1px solid color-mix(in srgb, var(--glass-border) 78%, transparent);
	box-shadow: 0 8px 18px rgba(0, 0, 0, 0.12);
	background: color-mix(in srgb, var(--color-raised-bg) 88%, transparent);
}

.home-project-card__meta {
	display: flex;
	flex-wrap: wrap;
	gap: 0.5rem;
	margin-top: auto;
}

.home-project-card__metric {
	display: inline-flex;
	align-items: center;
	gap: 0.4rem;
	min-height: 2rem;
	padding: 0.35rem 0.7rem;
	border-radius: 999px;
	border: 1px solid color-mix(in srgb, var(--glass-border) 82%, transparent);
	background: color-mix(in srgb, var(--color-glass-bg) 92%, transparent);
	color: var(--color-secondary);
	font-size: 0.85rem;
	font-weight: 700;
}

.home-project-card__metric--tag {
	padding: 0.35rem 0.7rem;
}

.home-project-card__tag-label {
	display: inline-flex;
	align-items: center;
	color: var(--color-secondary);
	font-size: 0.85rem;
	font-weight: 700;
	line-height: 1;
}
</style>
