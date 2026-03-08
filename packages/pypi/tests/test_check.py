"""
mille CLI の統合テスト (subprocess 経由)。

テスト実行前に maturin develop でビルドが必要:
    cd packages/pypi && maturin develop
"""

import os
import sys
import subprocess

import pytest


REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../.."))
FIXTURE_RUST = os.path.join(REPO_ROOT, "tests/fixtures/rust_sample")

# Prefer the mille binary installed in the same venv as this Python interpreter.
_VENV_MILLE = os.path.join(os.path.dirname(sys.executable), "mille")
_MILLE = _VENV_MILLE if os.path.isfile(_VENV_MILLE) else None


def run_mille(*args):
    if _MILLE is None:
        pytest.skip("mille not installed — run: cd packages/pypi && maturin develop")
    return subprocess.run([_MILLE] + list(args), capture_output=True, text=True)


def test_check_self_exits_zero():
    """mille 自身の mille.toml は違反 0 件で exit 0"""
    result = run_mille("check", "--config", os.path.join(REPO_ROOT, "mille.toml"))
    assert result.returncode == 0


def test_check_nonexistent_config_exits_nonzero():
    """存在しない config ファイルは exit 非 0"""
    result = run_mille("check", "--config", "/nonexistent/path/mille.toml")
    assert result.returncode != 0


def test_check_broken_fixture_exits_nonzero():
    """違反がある fixture は exit 非 0"""
    toml = os.path.join(FIXTURE_RUST, "mille_broken.toml")
    if not os.path.exists(toml):
        pytest.skip("broken fixture not found")
    result = run_mille("check", "--config", toml)
    assert result.returncode != 0


def test_help_exits_zero():
    """mille --help は exit 0"""
    result = run_mille("--help")
    assert result.returncode == 0
