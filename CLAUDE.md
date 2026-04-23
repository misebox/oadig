# oadig

OpenAPI spec から必要な情報だけを抽出する CLI。AI からの non-interactive 利用が主想定。

## バージョニング

セマンティックバージョニング。1.0.0 未達のあいだは「Phase N = `0.N.0`」で運用：

- Phase 1 → `v0.1.0`
- Phase 2 → `v0.2.0`
- Phase 3 → `v0.3.0`
- 以降、API が安定したら `v1.0.0`

Phase 内の追加・修正はパッチ（`v0.N.x`）。

## リリース手順

1. main が最新であることを確認（`git switch main && git pull`）
2. `scripts/release.sh <version>`（例: `0.1.0`）
   - `Cargo.toml` の version を書き換え、コミットし、タグ `v<version>` を付ける
3. スクリプトが表示する push コマンドを実行（タグ push で CI がバイナリをビルドして GitHub Release に添付）

## プラン

作業前にプランを書く場所: `tmp/plans/`（git ignore 済み）。
