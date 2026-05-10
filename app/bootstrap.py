"""First-run seed data so the drawer isn't empty."""
from __future__ import annotations

from app.db.repositories import folders, prompts, sites


SAMPLE_PROMPTS = [
    ("Code Review", "请帮我审查以下代码：\n\n```\n\n```\n\n关注:\n- 可读性\n- 性能\n- 安全"),
    ("Bug Triage", "复现步骤:\n1. \n2. \n\n期望: \n实际: \n环境: "),
    ("Daily Standup", "昨天: \n今天: \n阻塞: "),
]

SAMPLE_SITES = [
    ("Claude", "https://claude.ai"),
    ("ChatGPT", "https://chat.openai.com"),
    ("CNB", "https://cnb.cool"),
]


def seed_if_empty() -> None:
    if not folders.list_all() and not prompts.list_all():
        fid = folders.create("默认")
        for title, content in SAMPLE_PROMPTS:
            prompts.create(title=title, content=content, folder_id=fid)
    if not sites.list_all():
        for name, url in SAMPLE_SITES:
            sites.create(name, url)
