<template>
	<article
		class="project-card base-card"
		:aria-label="displayName"
		role="listitem"
		@click="onCardClick"
	>
		<router-link class="icon" tabindex="-1" :to="projectLink">
			<Avatar :src="iconUrl" :alt="displayName" size="md" no-shadow loading="lazy" />
		</router-link>
		<router-link
			class="gallery"
			:class="{ 'no-image': !displayFeaturedImage }"
			tabindex="-1"
			:to="projectLink"
			:style="color ? `background-color: ${toColor};` : ''"
		>
			<img
				v-if="displayFeaturedImage"
				:src="displayFeaturedImage"
				alt="gallery image"
				loading="lazy"
			/>
		</router-link>
		<div class="title">
			<router-link :to="projectLink">
				<h2 class="name">
					{{ displayName }}
				</h2>
			</router-link>
			<p v-if="displayAuthor" class="author">
				by
				<router-link
					v-if="authorHref && authorHref.startsWith('/')"
					class="title-link"
					:to="authorHref"
				>
					{{ displayAuthor }}
				</router-link>
				<a
					v-else-if="authorHref"
					class="title-link"
					:href="authorHref"
					target="_blank"
					rel="noopener noreferrer"
				>
					{{ displayAuthor }}
				</a>
			</p>
			<Badge v-if="status && status !== 'approved'" :type="status" class="status" />
		</div>
		<p class="description">
			{{ displayDescription }}
		</p>
		<Categories :categories="displayCategories" :type="type" class="tags">
			<slot name="tags-prefix" />
			<EnvironmentIndicator
				v-if="moderation || (displayClientSide && displayServerSide)"
				:type-only="moderation"
				:client-side="displayClientSide"
				:server-side="displayServerSide"
				:type="projectTypeDisplay ?? type ?? 'mod'"
				:search="search"
				:categories="displayCategories"
			/>
			<template #suffix>
				<slot name="tags-suffix" />
			</template>
		</Categories>
		<div class="stats">
			<div v-if="numericDownloads !== null" class="stat">
				<DownloadIcon aria-hidden="true" />
				<p>
					<strong>{{ formatNumber(numericDownloads) }}</strong
					><span class="stat-label"> download<span v-if="numericDownloads !== 1">s</span></span>
				</p>
			</div>
			<div v-if="numericFollows !== null" class="stat">
				<HeartIcon aria-hidden="true" />
				<p>
					<strong>{{ formatNumber(numericFollows) }}</strong
					><span class="stat-label"> follower<span v-if="numericFollows !== 1">s</span></span>
				</p>
			</div>
			<div class="buttons">
				<slot />
			</div>
			<div v-if="showUpdatedDate" v-tooltip="updatedDate" class="stat date">
				<EditIcon aria-hidden="true" />
				<span class="date-label">Updated </span> {{ sinceUpdated }}
			</div>
			<div v-else v-tooltip="createdDate" class="stat date">
				<CalendarIcon aria-hidden="true" />
				<span class="date-label">Published </span>{{ sinceCreation }}
			</div>
		</div>
	</article>
</template>

<script setup>
import { CalendarIcon, DownloadIcon, EditIcon, HeartIcon } from '@modrinth/assets'
import { formatNumber } from '@modrinth/utils'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime.js'
import { computed } from 'vue'
import { useRouter } from 'vue-router'

import { useRelativeTime } from '../../composables'
import Categories from '../search/Categories.vue'
import Avatar from './Avatar.vue'
import EnvironmentIndicator from './EnvironmentIndicator.vue'
import Badge from './SimpleBadge.vue'

dayjs.extend(relativeTime)
const router = useRouter()

const props = defineProps({
	id: {
		type: String,
		default: 'modrinth-0',
	},
	type: {
		type: String,
		default: 'mod',
	},
	name: {
		type: String,
		default: 'Project Name',
	},
	title: {
		type: String,
		default: null,
	},
	author: {
		type: [String, Object],
		default: null,
	},
	authorLink: {
		type: Function,
		default: null,
	},
	description: {
		type: String,
		default: 'A _type description',
	},
	summary: {
		type: String,
		default: null,
	},
	iconUrl: {
		type: String,
		default: '#',
		required: false,
	},
	downloads: {
		type: [String, Number],
		default: null,
		required: false,
	},
	follows: {
		type: [String, Number],
		default: null,
		required: false,
	},
	followers: {
		type: [String, Number],
		default: null,
		required: false,
	},
	createdAt: {
		type: String,
		default: '0000-00-00',
	},
	updatedAt: {
		type: String,
		default: null,
	},
	dateUpdated: {
		type: String,
		default: null,
	},
	datePublished: {
		type: String,
		default: null,
	},
	categories: {
		type: Array,
		default() {
			return []
		},
	},
	tags: {
		type: Array,
		default() {
			return []
		},
	},
	allTags: {
		type: Array,
		default() {
			return []
		},
	},
	filteredCategories: {
		type: Array,
		default() {
			return []
		},
	},
	projectTypeDisplay: {
		type: String,
		default: null,
	},
	projectTypeUrl: {
		type: String,
		default: 'project',
	},
	status: {
		type: String,
		default: null,
	},
	serverSide: {
		type: String,
		required: false,
		default: '',
	},
	clientSide: {
		type: String,
		required: false,
		default: '',
	},
	moderation: {
		type: Boolean,
		required: false,
		default: false,
	},
	search: {
		type: Boolean,
		required: false,
		default: false,
	},
	featuredImage: {
		type: String,
		required: false,
		default: null,
	},
	banner: {
		type: String,
		required: false,
		default: null,
	},
	link: {
		type: [String, Function],
		default: null,
	},
	environment: {
		type: Object,
		default: null,
	},
	showUpdatedDate: {
		type: Boolean,
		required: false,
		default: true,
	},
	hideLoaders: {
		type: Boolean,
		required: false,
		default: false,
	},
	color: {
		type: Number,
		required: false,
		default: null,
	},
})

const formatRelativeTime = useRelativeTime()

const toColor = computed(() => {
	if (props.color == null) return ''
	const color = props.color >>> 0
	const b = color & 0xff
	const g = (color & 0xff00) >>> 8
	const r = (color & 0xff0000) >>> 16
	return `rgba(${[r, g, b, 1].join(',')})`
})
const displayName = computed(() => props.title ?? props.name)
const displayDescription = computed(() => props.summary ?? props.description)
const displayCategories = computed(() => {
	if (Array.isArray(props.tags) && props.tags.length > 0) return props.tags
	if (Array.isArray(props.categories) && props.categories.length > 0) return props.categories
	return props.allTags ?? []
})
const displayAuthor = computed(() =>
	typeof props.author === 'string' ? props.author : props.author?.name ?? null,
)
const displayServerSide = computed(
	() => props.environment?.serverSide ?? props.serverSide ?? '',
)
const displayClientSide = computed(
	() => props.environment?.clientSide ?? props.clientSide ?? '',
)
const displayFeaturedImage = computed(() => props.banner ?? props.featuredImage ?? null)
const displayUpdatedAt = computed(
	() => props.dateUpdated ?? props.updatedAt ?? props.datePublished ?? props.createdAt,
)
const displayCreatedAt = computed(
	() => props.datePublished ?? props.createdAt ?? props.dateUpdated ?? props.updatedAt,
)
const projectLink = computed(() => {
	if (typeof props.link === 'string' && props.link.length > 0) return props.link
	return `/${props.projectTypeUrl}/${props.id}`
})
function onCardClick(event) {
	const target = event.target
	if (target?.closest?.('a,button,input,textarea,select,label')) return
	if (typeof props.link === 'function') {
		props.link()
		return
	}
	if (typeof projectLink.value === 'string' && projectLink.value.length > 0) {
		router.push(projectLink.value)
	}
}
const authorHref = computed(() => {
	if (!displayAuthor.value) return null
	if (props.author && typeof props.author === 'object' && props.author.link) {
		return props.author.link
	}
	if (typeof props.authorLink === 'function') {
		return props.authorLink(displayAuthor.value)
	}
	return `https://modrinth.com/user/${displayAuthor.value}`
})
const numericDownloads = computed(() => {
	if (props.downloads == null) return null
	const value = Number(props.downloads)
	return Number.isFinite(value) ? value : null
})
const numericFollows = computed(() => {
	const follows = props.followers ?? props.follows
	if (follows == null) return null
	const value = Number(follows)
	return Number.isFinite(value) ? value : null
})

const createdDate = computed(() => dayjs(displayCreatedAt.value).format('MMMM D, YYYY [at] h:mm:ss A'))
const sinceCreation = computed(() => formatRelativeTime(displayCreatedAt.value))
const updatedDate = computed(() => dayjs(displayUpdatedAt.value).format('MMMM D, YYYY [at] h:mm:ss A'))
const sinceUpdated = computed(() => formatRelativeTime(displayUpdatedAt.value))
</script>

<style lang="scss" scoped>
.project-card {
	display: inline-grid;
	box-sizing: border-box;
	overflow: hidden;
	margin: 0;
	line-height: 1;
	cursor: pointer;
	transition:
		background-color 0.15s ease,
		border-color 0.15s ease,
		box-shadow 0.15s ease,
		transform 0.15s ease;
}

.project-card:hover {
	background-color: color-mix(in srgb, var(--color-raised-bg) 70%, var(--color-button-bg-hover) 30%);
	border-color: color-mix(in srgb, var(--color-button-border) 55%, var(--color-brand) 45%);
	box-shadow: var(--shadow-raised);
	transform: translateY(-1px);
}

.display-mode--list .project-card {
	grid-template:
		'icon title stats'
		'icon description stats'
		'icon tags stats';
	grid-template-columns: min-content 1fr auto;
	grid-template-rows: min-content 1fr min-content;
	column-gap: var(--gap-lg);
	row-gap: var(--gap-sm);
	width: 100%;

	@media screen and (max-width: 750px) {
		grid-template:
			'icon title'
			'icon description'
			'icon tags'
			'stats stats';
		grid-template-columns: min-content auto;
		grid-template-rows: min-content 1fr min-content min-content;
	}

	@media screen and (max-width: 550px) {
		grid-template:
			'icon title'
			'icon description'
			'tags tags'
			'stats stats';
		grid-template-columns: min-content auto;
		grid-template-rows: min-content 1fr min-content min-content;
	}

	h2 {
		margin: 0;
		font-size: 1.14rem;
	}
}

.display-mode--gallery .project-card,
.display-mode--grid .project-card {
	padding: 0 0 1rem 0;
	grid-template: 'gallery gallery' 'icon title' 'description  description' 'tags tags' 'stats stats';
	grid-template-columns: min-content 1fr;
	grid-template-rows: min-content min-content 1fr min-content min-content;
	row-gap: var(--gap-sm);

	.gallery {
		display: inline-block;
		width: 100%;
		height: 10rem;
		background-color: var(--color-button-bg);

		&.no-image {
			background-color: var(--color-bg);
			border-bottom: 1px solid var(--color-divider);
		}

		img {
			box-shadow: none;
			width: 100%;
			height: 10rem;
			object-fit: cover;
		}
	}

	.icon {
		margin-left: var(--gap-lg);
		margin-top: -3rem;
		z-index: 1;

		img,
		svg {
			border-radius: var(--radius-lg);
			box-shadow:
				-2px -2px 0 2px var(--color-raised-bg),
				2px -2px 0 2px var(--color-raised-bg);
		}
	}

	.title {
		margin-left: var(--gap-md);
		margin-right: var(--gap-md);
		flex-direction: column;

		.name {
			font-size: 1.25rem;
		}

		.status {
			margin-top: var(--gap-xs);
		}
	}

	.description {
		margin-inline: var(--gap-lg);
	}

	.tags {
		margin-inline: var(--gap-lg);
	}

	.stats {
		margin-inline: var(--gap-lg);
		flex-direction: row;
		align-items: center;

		.stat-label {
			display: none;
		}

		.buttons {
			flex-direction: row;
			gap: var(--gap-sm);
			align-items: center;

			> :first-child {
				margin-left: auto;
			}

			&:first-child > :last-child {
				margin-right: auto;
			}
		}

		.buttons:not(:empty) + .date {
			flex-basis: 100%;
		}
	}
}

.display-mode--grid .project-card {
	.gallery {
		display: none;
	}

	.icon {
		margin-top: calc(var(--gap-lg) - var(--gap-sm));

		img,
		svg {
			border: none;
		}
	}

	.title {
		margin-top: calc(var(--gap-lg) - var(--gap-sm));
	}
}

.icon {
	grid-area: icon;
	display: flex;
	align-items: center;
}

.gallery {
	display: none;
	height: 10rem;
	grid-area: gallery;
}

.title {
	grid-area: title;
	display: flex;
	flex-direction: row;
	flex-wrap: wrap;
	align-items: baseline;
	column-gap: var(--gap-sm);
	row-gap: 0;
	word-wrap: anywhere;

	h2 {
		font-weight: bolder;
		color: var(--color-contrast);
	}

	h2,
	p {
		margin: 0;
		overflow-wrap: anywhere;
	}

	svg {
		width: auto;
		color: var(--color-special-orange);
		height: 1.5rem;
		margin-bottom: -0.25rem;
	}

	.title-link {
		text-decoration: none;
		color: var(--color-secondary);
		font-weight: 600;

		&:focus-visible,
		&:hover {
			color: var(--color-brand);
		}

		&:active {
			color: var(--color-text-dark);
		}
	}
}

.stats {
	grid-area: stats;
	display: flex;
	flex-direction: column;
	flex-wrap: wrap;
	align-items: flex-end;
	gap: 0.625rem;

	.stat {
		display: flex;
		flex-direction: row;
		align-items: center;
		width: fit-content;
		gap: var(--gap-xs);
		--stat-strong-size: 1rem;

		strong {
			font-size: var(--stat-strong-size);
		}

		p {
			margin: 0;
			display: flex;
			align-items: baseline;
			gap: 0.4rem;
			font-size: 0.9rem;
		}

		svg {
			height: var(--stat-strong-size);
			width: var(--stat-strong-size);
		}
	}

	.date {
		margin-top: auto;
		opacity: 0.9;
	}

	@media screen and (max-width: 750px) {
		flex-direction: row;
		column-gap: var(--gap-md);
		margin-top: var(--gap-xs);
	}

	@media screen and (max-width: 600px) {
		margin-top: 0;

		.stat-label {
			display: none;
		}
	}
}

.description {
	grid-area: description;
	margin-block: 0;
	display: flex;
	justify-content: flex-start;
	font-size: 1.02rem;
	line-height: 1.3;
	color: var(--color-secondary);
}

.tags {
	grid-area: tags;
	display: flex;
	flex-direction: row;

	@media screen and (max-width: 550px) {
		margin-top: var(--gap-xs);
	}
}

.buttons {
	display: flex;
	flex-direction: column;
	gap: var(--gap-sm);
	align-items: flex-end;
	flex-grow: 1;
}

.small-mode {
	@media screen and (min-width: 750px) {
		grid-template:
			'icon title'
			'icon description'
			'icon tags'
			'stats stats' !important;
		grid-template-columns: min-content auto !important;
		grid-template-rows: min-content 1fr min-content min-content !important;

		.tags {
			margin-top: var(--gap-xs) !important;
		}

		.stats {
			flex-direction: row;
			column-gap: var(--gap-md) !important;
			margin-top: var(--gap-xs) !important;

			.stat-label {
				display: none !important;
			}
		}
	}
}

.base-card {
	padding: 1rem;

	position: relative;
	min-height: 2rem;

	background: linear-gradient(
		90deg,
		color-mix(in srgb, var(--color-raised-bg) 86%, black 14%) 0%,
		color-mix(in srgb, var(--color-raised-bg) 94%, black 6%) 100%
	);
	border-radius: var(--radius-lg);
	border: 1px solid var(--color-button-border);

	outline: 2px solid transparent;

	box-shadow: var(--shadow-card);

	.card__overlay {
		position: absolute;
		top: 1rem;
		right: 1rem;
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		grid-gap: 0.5rem;
		z-index: 2;
	}

	&.warning {
		border-left: 0.5rem solid var(--color-banner-side);
		padding: 1.5rem;
		line-height: 1.5;
		background-color: var(--color-banner-bg);
		color: var(--color-banner-text);
		min-height: 0;

		a {
			/* Uses active color to increase contrast */
			color: var(--color-blue);
			text-decoration: underline;
		}
	}

	&.moderation-card {
		background-color: var(--color-banner-bg);
	}
}

.icon :deep(.avatar) {
	background: transparent;
	border: none;
	box-shadow: none;
}
</style>
