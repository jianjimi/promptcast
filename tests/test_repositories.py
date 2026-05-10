"""Smoke tests for the data layer."""
from __future__ import annotations

import pytest

from app.db import connection
from app.db.repositories import folders, prompts, settings, sites, tags
from app.models import SortMode


@pytest.fixture(autouse=True)
def fresh_db():
    connection.reset_for_tests()
    yield


def test_prompts_crud_and_tags():
    folder_id = folders.create("Work")
    tag_id = tags.create("draft")

    pid = prompts.create(title="hello", content="world", folder_id=folder_id, tag_ids=[tag_id])
    assert pid > 0

    items = prompts.list_all()
    assert len(items) == 1
    assert items[0].title == "hello"
    assert items[0].tag_ids == [tag_id]

    prompts.update(pid, title="hi", tag_ids=[])
    again = prompts.get(pid)
    assert again is not None
    assert again.title == "hi"
    assert again.tag_ids == []


def test_prompt_sort_modes():
    a = prompts.create(title="b", content="")
    b = prompts.create(title="a", content="")
    prompts.toggle_pin(a)

    pinned_first = prompts.list_all(SortMode.TITLE)
    # `a` is pinned so it appears first regardless of title order
    assert pinned_first[0].id == a
    assert pinned_first[1].id == b


def test_prompt_record_use_increments():
    pid = prompts.create(title="x", content="")
    prompts.record_use(pid)
    prompts.record_use(pid)
    assert prompts.get(pid).use_count == 2


def test_folder_reorder():
    a = folders.create("A")
    b = folders.create("B")
    c = folders.create("C")
    folders.reorder([c, a, b])
    ids = [f.id for f in folders.list_all()]
    assert ids == [c, a, b]


def test_tag_unique_get_or_create():
    first = tags.get_or_create("x")
    second = tags.get_or_create("x")
    assert first == second


def test_sites_crud_and_favicon():
    sid = sites.create("Cnb", "https://cnb.cool")
    sites.set_favicon(sid, b"\x89PNGdata", "image/png")
    site = sites.get(sid)
    assert site is not None
    assert site.favicon_blob == b"\x89PNGdata"
    assert site.favicon_mime == "image/png"


def test_settings_roundtrip():
    settings.set_value("theme", "dark")
    settings.set_value("auto_start", True)
    snapshot = settings.get_all()
    assert snapshot["theme"] == "dark"
    assert snapshot["auto_start"] is True
    assert settings.get("missing", "fallback") == "fallback"
