---
title: Go
description: Go プロジェクトでの mille 設定例
---

## mille.toml の設定例

```toml
[project]
name      = "my-go-app"
root      = "."
languages = ["go"]

[resolve.go]
module_name = "github.com/myorg/my-go-app"

[[layers]]
name            = "domain"
paths           = ["domain/**"]
dependency_mode = "opt-in"
allow           = []

[[layers]]
name            = "usecase"
paths           = ["usecase/**"]
dependency_mode = "opt-in"
allow           = ["domain"]

[[layers]]
name            = "infrastructure"
paths           = ["infrastructure/**"]
dependency_mode = "opt-out"
deny            = []

[[layers]]
name            = "cmd"
paths           = ["cmd/**"]
dependency_mode = "opt-in"
allow           = ["domain", "usecase", "infrastructure"]
```

## `mille init` の動作

`go.mod` を読み取り `[resolve.go] module_name` を自動生成します。外部パッケージは完全なインポートパス（例: `"github.com/cilium/ebpf"`, `"fmt"`, `"net/http"`）で `external_allow` に列挙されます。

## インポートの分類

| インポート | 分類 |
|---|---|
| `"github.com/myorg/my-go-app/domain"` | 内部（`module_name` で始まる） |
| `"fmt"`, `"net/http"` | 外部（標準ライブラリ） |
| `"github.com/gin-gonic/gin"` | 外部（サードパーティ） |
