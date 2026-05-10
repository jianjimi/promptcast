"""Fuzzy subsequence search with simple scoring."""
from __future__ import annotations

from collections.abc import Iterable, Mapping

from app.models import Prompt


def _score(haystack: str, needle: str) -> int | None:
    """Return a score (higher = better) if `needle` is a subsequence of `haystack`."""
    if not needle:
        return 0
    h = haystack.lower()
    n = needle.lower()

    if n in h:
        # Substring is by far the strongest signal.
        idx = h.index(n)
        return 1000 - idx

    # Subsequence match: penalize long gaps and reward matches at word boundaries.
    score = 0
    i = 0
    last_match = -1
    in_word_start = True
    for ch in h:
        if i >= len(n):
            break
        if ch == n[i]:
            gap = 0 if last_match < 0 else (h.index(ch, last_match + 1) - last_match - 1)
            score += 10 - min(gap, 8)
            if in_word_start:
                score += 5
            last_match += 1 + gap
            i += 1
            in_word_start = False
        else:
            if ch in (" ", "-", "_", "/", ".", "\n", "\t"):
                in_word_start = True
            else:
                in_word_start = False
    if i < len(n):
        return None
    return score


def fuzzy_filter(
    prompts: Iterable[Prompt],
    query: str,
    *,
    tag_names: Mapping[int, str] | None = None,
) -> list[Prompt]:
    """Filter + rank prompts by fuzzy match against title / content / tag names."""
    needle = query.strip()
    if not needle:
        return list(prompts)

    tag_names = tag_names or {}
    scored: list[tuple[int, Prompt]] = []
    for p in prompts:
        title_score = _score(p.title, needle) or 0
        content_score = _score(p.content[:400], needle) or 0
        tag_score = 0
        for tid in p.tag_ids:
            name = tag_names.get(tid)
            if name:
                tag_score = max(tag_score, _score(name, needle) or 0)

        best = max(title_score * 3, content_score, tag_score * 2)
        if best > 0:
            scored.append((best, p))

    scored.sort(key=lambda pair: (-pair[0], -(pair[1].is_pinned), pair[1].title.lower()))
    return [p for _s, p in scored]
