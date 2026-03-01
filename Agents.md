# Agents.md

## プロジェクト概要

Aoi Journal は Tauri v2 を使ったデスクトップアプリです。
フロントエンドは React + TypeScript (Vite)、バックエンドは Rust で構成されています。

## ビルドチェック方法

### フロントエンド（React/TypeScript）

```bash
npm ci
npm run build
```

### バックエンド（Rust / Tauri）

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

`cargo check` はコンパイルの検証のみを行い、バイナリを生成しません。
フル Tauri ビルドには `npm run tauri build` を使用してください。

### テスト

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

## Windows 向けビルドチェック

Windows 向けのビルド互換性は GitHub Actions で自動的に確認されます。

- ワークフロー: `.github/workflows/build-windows.yml`
- ランナー: `windows-latest`
- トリガー: `main` ブランチへの push および pull request

ローカルで Windows 向けにクロスコンパイルする場合は、次のターゲットを追加してください。

```bash
rustup target add x86_64-pc-windows-msvc
cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-msvc
```

> [!NOTE]
> ローカルでのクロスコンパイルには Windows SDK が必要です。
> CI でのビルドチェックには `windows-latest` ランナーを使用することを推奨します。
