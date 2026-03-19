# Task: mille ドキュメントサイト構築

## 概要

Starlight (Astro) + Pagefind + i18n (日本語デフォルト / 英語) による
GitHub Pages 対応のユーザードキュメントサイトを構築する。

## チェックリスト

- [ ] ブランチ作成: `feat/pr-docs-website`
- [ ] `website/` Starlight セットアップ
- [ ] `astro.config.mjs` i18n + デザイン設定
- [ ] `src/styles/custom.css` デザインシステム適用
- [ ] 日本語コンテンツ移行 (README.md → docs)
- [ ] 英語コンテンツ作成
- [ ] GitHub Actions ワークフロー追加 (`.github/workflows/docs.yml`)
- [ ] `npm run build` ビルド確認
- [ ] PR 作成

## スタック

- **Starlight (Astro)**: ドキュメントフレームワーク
- **Pagefind**: 静的全文検索（Starlight ビルトイン）
- **i18n**: 日本語（デフォルト）+ 英語

## デザインシステム

- Style: Minimalism & Swiss Style
- Heading: JetBrains Mono
- Body: IBM Plex Sans
- Primary: `#475569` / CTA: `#2563EB` / BG: `#F8FAFC` / Text: `#1E293B`
