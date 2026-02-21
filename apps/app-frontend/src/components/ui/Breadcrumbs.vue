<template>
	<div data-tauri-drag-region class="flex items-center gap-1 pl-3">
		<Button v-if="false" class="breadcrumbs__back transparent" icon-only @click="$router.back()">
			<ChevronLeftIcon />
		</Button>
		<Button
			v-if="false"
			class="breadcrumbs__forward transparent"
			icon-only
			@click="$router.forward()"
		>
			<ChevronRightIcon />
		</Button>
		{{ breadcrumbData.resetToNames(breadcrumbs) }}
		<template v-for="breadcrumb in breadcrumbs" :key="breadcrumb.name">
			<router-link
				v-if="getBreadcrumbTo(breadcrumb)"
				:to="getBreadcrumbTo(breadcrumb)"
				class="text-primary"
				>{{
					breadcrumb.name.charAt(0) === '?'
						? breadcrumbData.getName(breadcrumb.name.slice(1))
						: localizeBreadcrumbName(breadcrumb.name)
				}}
			</router-link>
			<span
				v-else
				data-tauri-drag-region
				class="text-contrast font-semibold cursor-default select-none"
				>{{
					breadcrumb.name.charAt(0) === '?'
						? breadcrumbData.getName(breadcrumb.name.slice(1))
						: localizeBreadcrumbName(breadcrumb.name)
				}}</span
			>
			<ChevronRightIcon v-if="getBreadcrumbTo(breadcrumb)" data-tauri-drag-region class="w-5 h-5" />
		</template>
	</div>
</template>

<script setup>
import { ChevronLeftIcon, ChevronRightIcon } from '@modrinth/assets'
import { Button } from '@modrinth/ui'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { computed } from 'vue'
import { useRoute } from 'vue-router'

import { useBreadcrumbs } from '@/store/breadcrumbs'

const route = useRoute()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	home: { id: 'app.nav.home', defaultMessage: 'Home' },
	worlds: { id: 'app.nav.worlds', defaultMessage: 'Worlds' },
	servers: { id: 'app.nav.servers', defaultMessage: 'Servers' },
	discoverContent: { id: 'app.nav.discover', defaultMessage: 'Discover content' },
	skins: { id: 'app.nav.skins', defaultMessage: 'Skins' },
	maintenance: { id: 'app.nav.maintenance', defaultMessage: 'Maintenance' },
	library: { id: 'app.nav.library', defaultMessage: 'Library' },
	versions: { id: 'project.tab.versions', defaultMessage: 'Versions' },
	gallery: { id: 'project.tab.gallery', defaultMessage: 'Gallery' },
	content: { id: 'instance.breadcrumb.content', defaultMessage: 'Content' },
	logs: { id: 'instance.breadcrumb.logs', defaultMessage: 'Logs' },
	screenshots: { id: 'instance.breadcrumb.screenshots', defaultMessage: 'Screenshots' },
})

const breadcrumbData = useBreadcrumbs()
const localizeBreadcrumbName = (name) => {
	const map = {
		Home: messages.home,
		Worlds: messages.worlds,
		Servers: messages.servers,
		'Discover content': messages.discoverContent,
		Skins: messages.skins,
		Maintenance: messages.maintenance,
		Library: messages.library,
		Versions: messages.versions,
		Gallery: messages.gallery,
		Content: messages.content,
		Logs: messages.logs,
		Screenshots: messages.screenshots,
	}
	return map[name] ? formatMessage(map[name]) : name
}

const breadcrumbs = computed(() => {
	const additionalContext =
		route.meta.useContext === true
			? breadcrumbData.context
			: route.meta.useRootContext === true
				? breadcrumbData.rootContext
				: null
	return additionalContext ? [additionalContext, ...route.meta.breadcrumb] : route.meta.breadcrumb
})

const getBreadcrumbTo = (breadcrumb) => {
	if (!breadcrumb?.link) return null
	if (!breadcrumb.link.includes('{id}')) {
		return { path: breadcrumb.link, query: breadcrumb.query }
	}
	const rawId = route.params?.id
	if (typeof rawId !== 'string' || rawId.length === 0) return null
	return {
		path: breadcrumb.link.replace('{id}', encodeURIComponent(rawId)),
		query: breadcrumb.query,
	}
}
</script>
