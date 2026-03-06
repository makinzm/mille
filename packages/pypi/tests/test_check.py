"""
mille Python extension (PyO3) の統合テスト。

テスト実行前に maturin develop でビルドが必要:
    cd packages/pypi && maturin develop
"""

import os
import sys
import pytest

import mille


REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
FIXTURE_RUST = os.path.join(REPO_ROOT, "tests/fixtures/rust_sample")
FIXTURE_GO = os.path.join(REPO_ROOT, "tests/fixtures/go_sample")


# ---------------------------------------------------------------------------
# mille.check() — ライブラリ API
# ---------------------------------------------------------------------------


def test_check_returns_check_result():
    """mille.check() は CheckResult を返す"""
    result = mille.check(os.path.join(REPO_ROOT, "mille.toml"))
    assert hasattr(result, "violations")
    assert hasattr(result, "layer_stats")


def test_check_self_has_no_violations():
    """mille 自身の mille.toml は違反 0 件"""
    result = mille.check(os.path.join(REPO_ROOT, "mille.toml"))
    assert len(result.violations) == 0


def test_check_nonexistent_config_raises():
    """存在しない config ファイルは例外を送出する"""
    with pytest.raises(Exception):
        mille.check("/nonexistent/path/mille.toml")


def test_check_violation_has_expected_fields():
    """Violation オブジェクトに期待するフィールドがある"""
    toml = os.path.join(FIXTURE_RUST, "mille_broken.toml")
    if not os.path.exists(toml):
        pytest.skip("broken fixture not found")
    result = mille.check(toml)
    assert len(result.violations) > 0
    v = result.violations[0]
    assert hasattr(v, "file")
    assert hasattr(v, "line")
    assert hasattr(v, "from_layer")
    assert hasattr(v, "to_layer")
    assert hasattr(v, "import_path")
    assert hasattr(v, "kind")


def test_check_layer_stats_populated():
    """layer_stats に各レイヤーの情報が入っている"""
    result = mille.check(os.path.join(REPO_ROOT, "mille.toml"))
    assert len(result.layer_stats) > 0
    stat = result.layer_stats[0]
    assert hasattr(stat, "name")
    assert hasattr(stat, "file_count")
    assert hasattr(stat, "violation_count")
    assert isinstance(stat.file_count, int)
