export const APRIL_FOOLS_ENABLED = true
export const APRIL_FOOLS_THEME_CLASS = 'april-fools-mode'

const DODGE_CLASS = 'april-fools-dodging'
const ACTIVE_SELECTOR = '[data-april-fools-dodge]'

export function isAprilFoolsActive(date = new Date()) {
	return APRIL_FOOLS_ENABLED && date.getMonth() === 3 && date.getDate() === 1
}

export function applyAprilFoolsTheme(active) {
	const root = document.documentElement
	root.classList.toggle(APRIL_FOOLS_THEME_CLASS, !!active)
	document.body?.classList.toggle('april-fools-active', !!active)
}

export function startAprilFoolsMode({ active, addNotification, formatMessage }) {
	if (!active) {
		applyAprilFoolsTheme(false)
		return () => {}
	}

	applyAprilFoolsTheme(true)

	const timeouts = []
	const dodging = new Set()
	const randomMessages = [
		{
			title: formatMessage({ id: 'app.april-fools.random.launcher.title' }),
			text: formatMessage({ id: 'app.april-fools.random.launcher.body' }),
		},
		{
			title: formatMessage({ id: 'app.april-fools.random.theme.title' }),
			text: formatMessage({ id: 'app.april-fools.random.theme.body' }),
		},
		{
			title: formatMessage({ id: 'app.april-fools.random.buttons.title' }),
			text: formatMessage({ id: 'app.april-fools.random.buttons.body' }),
		},
	]

	const notify = (title, text, type = 'info') => addNotification({ title, text, type })

	timeouts.push(
		setTimeout(() => {
			notify(
				formatMessage({ id: 'app.april-fools.theme.title' }),
				formatMessage({ id: 'app.april-fools.theme.body' }),
				'warning',
			)
		}, 1200),
	)

	timeouts.push(
		setTimeout(() => {
			notify(
				formatMessage({ id: 'app.april-fools.launcher.title' }),
				formatMessage({ id: 'app.april-fools.launcher.body' }),
				'info',
			)
		}, 4200),
	)

	const interval = setInterval(() => {
		const next = randomMessages[Math.floor(Math.random() * randomMessages.length)]
		notify(next.title, next.text, 'info')
	}, 26000)

	const onPointerEnter = (event) => {
		const target = event.target instanceof Element ? event.target.closest(ACTIVE_SELECTOR) : null
		if (!target || dodging.has(target)) return
		if (Math.random() > 0.55) return

		dodging.add(target)
		const x = Math.round((Math.random() - 0.5) * 220)
		const y = Math.round((Math.random() - 0.5) * 120)
		target.style.setProperty('--april-dodge-x', `${x}px`)
		target.style.setProperty('--april-dodge-y', `${y}px`)
		target.classList.add(DODGE_CLASS)

		const reset = setTimeout(() => {
			target.classList.remove(DODGE_CLASS)
			target.style.removeProperty('--april-dodge-x')
			target.style.removeProperty('--april-dodge-y')
			dodging.delete(target)
		}, 1200)

		timeouts.push(reset)
	}

	document.addEventListener('pointerenter', onPointerEnter, true)

	return () => {
		clearInterval(interval)
		for (const timeout of timeouts) clearTimeout(timeout)
		document.removeEventListener('pointerenter', onPointerEnter, true)
		applyAprilFoolsTheme(false)
		for (const target of dodging) {
			target.classList.remove(DODGE_CLASS)
			target.style.removeProperty('--april-dodge-x')
			target.style.removeProperty('--april-dodge-y')
		}
		dodging.clear()
	}
}
