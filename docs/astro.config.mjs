// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import { nuschtSearch } from './integrations/nuscht-search';
import catppuccin from '@catppuccin/starlight';

// https://astro.build/config
export default defineConfig({
	integrations: [
		starlight({
			title: 'Nixifest',
			social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/bitbloxhub/nixifest' }],
			plugins: [
				catppuccin({
					dark: { flavor: 'mocha', accent: 'mauve' },
					light: { flavor: 'latte', accent: 'mauve' },
				}),
			],
			customCss: ['./src/styles/custom.css'],
			sidebar: [
				{ label: 'Options', link: '/options/' },
				{
					label: 'Guides',
					items: [
						{ label: 'Getting started', slug: 'guides/getting-started' },
						{ label: 'Resources', slug: 'guides/resources' },
						{ label: 'Custom resources', slug: 'guides/crds' },
					],
				},
				{
					label: 'Reference',
					items: [{ autogenerate: { directory: 'reference' } }],
				},
			],
		}),
		nuschtSearch(),
	],
});
