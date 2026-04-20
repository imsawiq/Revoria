<template>
	<div class="gallery-view">
		<div class="gallery">
			<Card v-for="(image, index) in project.gallery" :key="image.url" class="gallery-item">
				<a @click="expandImage(image, index)">
					<img :src="image.url" :alt="image.title" class="gallery-image" />
				</a>
				<div class="gallery-body">
					<h3>{{ image.title }}</h3>
					{{ image.description }}
				</div>
				<span class="gallery-time">
					<CalendarIcon />
					{{
						new Date(image.created).toLocaleDateString('en-US', {
							year: 'numeric',
							month: 'long',
							day: 'numeric',
						})
					}}
				</span>
			</Card>
		</div>
		<Teleport to="body">
			<Transition name="gallery-modal">
				<div v-if="expandedGalleryItem" class="expanded-image-modal" @click="hideImage">
					<div class="content" @click.stop>
						<div class="image-stage">
						<img
							class="image"
							:class="{ 'zoomed-in': zoomedIn }"
							:src="
								expandedGalleryItem.raw_url
									? expandedGalleryItem.raw_url
									: 'https://cdn.modrinth.com/placeholder-banner.svg'
							"
							:alt="expandedGalleryItem.title ? expandedGalleryItem.title : 'gallery-image'"
						/>
					</div>

						<div class="floating">
						<div class="text">
							<h2 v-if="expandedGalleryItem.title">
								{{ expandedGalleryItem.title }}
							</h2>
							<p v-if="expandedGalleryItem.description">
								{{ expandedGalleryItem.description }}
							</p>
						</div>
						<div class="controls">
							<div class="buttons">
								<Button class="close" icon-only @click="hideImage">
									<XIcon aria-hidden="true" />
								</Button>
								<a
									class="open btn icon-only"
									target="_blank"
									:href="
										expandedGalleryItem.raw_url
											? expandedGalleryItem.raw_url
											: 'https://cdn.modrinth.com/placeholder-banner.svg'
									"
								>
									<ExternalIcon aria-hidden="true" />
								</a>
								<Button icon-only @click="zoomedIn = !zoomedIn">
									<ExpandIcon v-if="!zoomedIn" aria-hidden="true" />
									<ContractIcon v-else aria-hidden="true" />
								</Button>
								<Button
									v-if="project.gallery.length > 1"
									class="previous"
									icon-only
									@click="previousImage()"
								>
									<LeftArrowIcon aria-hidden="true" />
								</Button>
								<Button
									v-if="project.gallery.length > 1"
									class="next"
									icon-only
									@click="nextImage()"
								>
									<RightArrowIcon aria-hidden="true" />
								</Button>
							</div>
						</div>
					</div>
					</div>
				</div>
			</Transition>
		</Teleport>
	</div>
</template>

<script setup>
import {
	CalendarIcon,
	ContractIcon,
	ExpandIcon,
	ExternalIcon,
	LeftArrowIcon,
	RightArrowIcon,
	XIcon,
} from '@modrinth/assets'
import { Button, Card } from '@modrinth/ui'
import { Teleport, onMounted, onUnmounted, ref } from 'vue'

// import { hide_ads_window, show_ads_window } from '@/helpers/ads.js'
import { trackEvent } from '@/helpers/analytics'

const props = defineProps({
	project: {
		type: Object,
		default: () => ({}),
	},
})

const expandedGalleryItem = ref(null)
const expandedGalleryIndex = ref(0)
const zoomedIn = ref(false)

const hideImage = () => {
	expandedGalleryItem.value = null
	// show_ads_window()
}

const nextImage = () => {
	expandedGalleryIndex.value++
	if (expandedGalleryIndex.value >= props.project.gallery.length) {
		expandedGalleryIndex.value = 0
	}
	expandedGalleryItem.value = props.project.gallery[expandedGalleryIndex.value]
	trackEvent('GalleryImageNext', {
		project_id: props.project.id,
		url: expandedGalleryItem.value.url,
	})
}

const previousImage = () => {
	expandedGalleryIndex.value--
	if (expandedGalleryIndex.value < 0) {
		expandedGalleryIndex.value = props.project.gallery.length - 1
	}
	expandedGalleryItem.value = props.project.gallery[expandedGalleryIndex.value]
	trackEvent('GalleryImagePrevious', {
		project_id: props.project.id,
		url: expandedGalleryItem.value,
	})
}

const expandImage = (item, index) => {
	// hide_ads_window()
	expandedGalleryItem.value = item
	expandedGalleryIndex.value = index
	zoomedIn.value = false

	trackEvent('GalleryImageExpand', {
		project_id: props.project.id,
		url: item.url,
	})
}

function keyListener(e) {
	if (expandedGalleryItem.value) {
		if (e.key === 'Escape') {
			e.preventDefault()
			hideImage()
		} else if (e.key === 'ArrowLeft') {
			e.preventDefault()
			previousImage()
		} else if (e.key === 'ArrowRight') {
			e.preventDefault()
			nextImage()
		}
	}
}

onMounted(() => {
	document.addEventListener('keydown', keyListener)
})

onUnmounted(() => {
	document.removeEventListener('keydown', keyListener)
})
</script>

<style scoped lang="scss">
.gallery {
	display: grid;
	grid-template-columns: repeat(auto-fill, minmax(20rem, 1fr));
	width: 100%;
	gap: 1rem;
}

.gallery-item {
	padding: 0;
	overflow: hidden;
	margin: 0;
	display: flex;
	flex-direction: column;

	.gallery-image {
		width: 100%;
		aspect-ratio: 2/1;
		object-fit: cover;
		object-position: center;
	}

	.gallery-body {
		flex-grow: 1;
		padding: 1rem;
	}

	.gallery-time {
		padding: 0 1rem 1rem;
		vertical-align: center;
	}
}

.expanded-image-modal {
	position: fixed;
	z-index: 40;
	overflow: hidden;
	top: 0;
	left: 0;
	width: 100%;
	height: 100%;
	background:
		radial-gradient(
			1200px 700px at 50% 10%,
			color-mix(in srgb, var(--color-brand-highlight) 10%, transparent),
			transparent 74%
		),
		rgba(8, 10, 14, 0.92);
	backdrop-filter: blur(8px) saturate(112%);
	display: flex;
	justify-content: center;
	align-items: center;

	.content {
		position: relative;
		width: calc(100vw - 2 * var(--gap-lg));
		height: calc(100vh - 2 * var(--gap-lg));

		.image-stage {
			position: absolute;
			inset: 0;
			display: flex;
			align-items: center;
			justify-content: center;
			padding: 2.5rem;
			border-radius: 1.5rem;
		}

		.circle-button {
			padding: 0.5rem;
			line-height: 1;
			display: flex;
			max-width: 2rem;
			color: var(--color-button-text);
			background-color: var(--color-button-bg);
			border-radius: var(--size-rounded-max);
			margin: 0;
			box-shadow: inset 0px -1px 1px rgb(17 24 39 / 10%);

			&:not(:last-child) {
				margin-right: 0.5rem;
			}

			&:hover {
				background-color: var(--color-button-bg-hover) !important;

				svg {
					color: var(--color-button-text-hover) !important;
				}
			}

			&:active {
				background-color: var(--color-button-bg-active) !important;

				svg {
					color: var(--color-button-text-active) !important;
				}
			}

			svg {
				height: 1rem;
				width: 1rem;
			}
		}

		.image {
			display: block;
			max-width: min(calc(100vw - 10rem), 1400px);
			max-height: calc(100vh - 12rem);
			border-radius: 1.25rem;
			border: 1px solid color-mix(in srgb, var(--glass-border) 86%, transparent);
			box-shadow:
				0 40px 120px -48px rgba(0, 0, 0, 0.72),
				0 0 0 1px color-mix(in srgb, var(--glass-border) 46%, transparent);
			background: color-mix(in srgb, var(--color-raised-bg) 90%, black 10%);
			object-fit: contain;

			&.zoomed-in {
				object-fit: contain;
				width: auto;
				height: calc(100vh - 12rem);
				max-width: min(calc(100vw - 10rem), 1400px);
			}
		}
		.floating {
			position: absolute;
			left: 50%;
			transform: translateX(-50%);
			bottom: 1.5rem;
			display: flex;
			flex-direction: column;
			align-items: center;
			gap: var(--gap-md);
			transition: opacity 0.25s ease-in-out;
			opacity: 1;
			padding: 0;

			&:not(&:hover) {
				opacity: 0.94;
				.text {
					transform: translateY(0);
					opacity: 1;
				}
				.controls {
					transform: translateY(0);
				}
			}

			.text {
				display: flex;
				flex-direction: column;
				max-width: min(42rem, calc(100vw - 8rem));
				transition:
					opacity 0.25s ease-in-out,
					transform 0.25s ease-in-out;
				text-shadow: 0 1px 10px rgba(9, 18, 14, 0.42);
				margin-bottom: 0.25rem;
				gap: 0.5rem;
				padding: 1rem 1.25rem;
				border-radius: 1rem;
				background: color-mix(in srgb, var(--color-glass-bg-strong) 92%, transparent);
				border: 1px solid color-mix(in srgb, var(--glass-border) 80%, transparent);
				box-shadow: var(--glass-shadow);

				h2 {
					color: var(--color-contrast);
					font-size: 1.25rem;
					text-align: center;
					margin: 0;
				}

				p {
					color: var(--color-base);
					margin: 0;
				}
			}
			.controls {
				background-color: color-mix(in srgb, var(--color-glass-bg-strong) 96%, transparent);
				border: 1px solid color-mix(in srgb, var(--glass-border) 86%, transparent);
				box-shadow: var(--glass-shadow);
				padding: 0.75rem;
				border-radius: 999px;
				transition:
					opacity 0.25s ease-in-out,
					transform 0.25s ease-in-out;
			}
		}
	}
}

.gallery-modal-enter-active,
.gallery-modal-leave-active {
	transition:
		opacity 0.2s ease,
		transform 0.24s ease;
}

.gallery-modal-enter-from,
.gallery-modal-leave-to {
	opacity: 0;
}

.gallery-modal-enter-from .content,
.gallery-modal-leave-to .content {
	transform: scale(0.97) translateY(8px);
}

.gallery-modal-enter-active .content,
.gallery-modal-leave-active .content {
	transition: transform 0.24s ease;
}

.buttons {
	display: flex;
	gap: 0.5rem;
}
</style>
