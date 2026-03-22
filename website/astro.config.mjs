// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
  site: 'https://makinzm.github.io',
  base: '/mille',
  integrations: [
    starlight({
      title: 'mille',
      logo: {
        src: './src/assets/mille.svg',
        replacesTitle: false,
      },
      defaultLocale: 'root',
      locales: {
        root: {
          label: '日本語',
          lang: 'ja',
        },
        en: {
          label: 'English',
          lang: 'en',
        },
      },
      social: {
        github: 'https://github.com/makinzm/mille',
      },
      sidebar: [
        {
          label: 'はじめに',
          translations: { en: 'Getting Started' },
          items: [
            { slug: 'getting-started/install', label: 'インストール', translations: { en: 'Installation' } },
            { slug: 'getting-started/quickstart', label: 'クイックスタート', translations: { en: 'Quick Start' } },
          ],
        },
        {
          label: 'コマンドリファレンス',
          translations: { en: 'Commands' },
          items: [
            { slug: 'commands/init', label: 'mille init' },
            { slug: 'commands/check', label: 'mille check' },
            { slug: 'commands/analyze', label: 'mille analyze' },
            { slug: 'commands/report', label: 'mille report external' },
          ],
        },
        {
          label: '設定リファレンス',
          translations: { en: 'Configuration' },
          items: [
            { slug: 'configuration/overview', label: '概要', translations: { en: 'Overview' } },
            { slug: 'configuration/layers', label: 'レイヤー設定', translations: { en: 'Layers' } },
            { slug: 'configuration/naming', label: 'ネーミング規則', translations: { en: 'Naming Rules' } },
            { slug: 'configuration/resolve', label: 'インポート解決', translations: { en: 'Import Resolution' } },
            { slug: 'configuration/severity', label: '重大度設定', translations: { en: 'Severity' } },
          ],
        },
        {
          label: 'ガイド',
          translations: { en: 'Guides' },
          items: [
            { slug: 'guides/ci-integration', label: 'CI インテグレーション', translations: { en: 'CI Integration' } },
            {
              label: '言語別ガイド',
              translations: { en: 'Languages' },
              items: [
                { slug: 'guides/languages/rust', label: 'Rust' },
                { slug: 'guides/languages/go', label: 'Go' },
                { slug: 'guides/languages/typescript', label: 'TypeScript / JavaScript' },
                { slug: 'guides/languages/python', label: 'Python' },
                { slug: 'guides/languages/java', label: 'Java' },
                { slug: 'guides/languages/kotlin', label: 'Kotlin' },
              ],
            },
          ],
        },
      ],
      customCss: ['./src/styles/custom.css'],
    }),
  ],
});
