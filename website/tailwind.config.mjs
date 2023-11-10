/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{astro,html,js,jsx,md,mdx,svelte,ts,tsx,vue}'],
	theme: {
		extend: {
			backgroundColor: {
				"custom-yellow": "rgb(233, 196, 106)",
				"custom-blue": "rgb(73, 151, 222)",
				"custom-dark-gray": "#131315"
			}
		},
	},
	plugins: [],
}
