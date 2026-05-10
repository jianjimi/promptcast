"""Fuzzy search behavior."""
from __future__ import annotations

import pytest

from app.db import connection
from app.db.repositories import prompts as prompts_repo, tags as tags_repo
from app.services.search import fuzzy_filter


@pytest.fixture(autouse=True)
def fresh_db():
    connection.reset_for_tests()
    yield


def _create(title: str, content: str = "", tags: list[int] = ()) -> int:
    return prompts_repo.create(title=title, content=content, tag_ids=list(tags))


def test_substring_beats_subsequence():
    a = _create("Daily Standup")
    _create("DSP Notes")
    items = [prompts_repo.get(a), prompts_repo.get(2)]
    out = fuzzy_filter(items, "stand")
    assert out[0].id == a


def test_subsequence_match():
    pid = _create("Code Review Checklist")
    out = fuzzy_filter([prompts_repo.get(pid)], "crc")
    assert out and out[0].id == pid


def test_no_match_filtered_out():
    _create("Hello world")
    out = fuzzy_filter(prompts_repo.list_all(), "xyz123")
    assert out == []


def test_tag_match():
    tid = tags_repo.create("AI")
    pid = _create("Random title", tags=[tid])
    out = fuzzy_filter([prompts_repo.get(pid)], "ai", tag_names={tid: "AI"})
    assert out and out[0].id == pid


def test_empty_query_returns_all():
    a = _create("a")
    b = _create("b")
    out = fuzzy_filter(prompts_repo.list_all(), "")
    assert {p.id for p in out} == {a, b}
