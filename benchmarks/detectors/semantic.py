"""Semantic similarity, grounding novelty, and entity-novelty utilities."""

import re

# ─── Grounding novelty (TRIZ P25 Self-service) ───────────────────────────────

_GROUNDING_STOPWORDS = frozenset(
    {
        "a",
        "an",
        "the",
        "is",
        "are",
        "was",
        "were",
        "be",
        "been",
        "being",
        "have",
        "has",
        "had",
        "do",
        "does",
        "did",
        "will",
        "would",
        "could",
        "should",
        "may",
        "might",
        "shall",
        "can",
        "not",
        "and",
        "or",
        "but",
        "in",
        "on",
        "at",
        "to",
        "for",
        "of",
        "with",
        "by",
        "from",
        "that",
        "this",
        "which",
        "who",
        "whom",
        "where",
        "when",
        "how",
        "what",
        "if",
        "it",
        "its",
        "he",
        "she",
        "they",
        "we",
        "i",
        "you",
        "his",
        "her",
        "their",
        "our",
        "my",
        "your",
        "also",
        "as",
        "so",
        "than",
        "into",
        "about",
        "after",
        "before",
        "then",
        "these",
        "those",
        "said",
        "says",
        "just",
        "more",
        "some",
        "such",
        "there",
        "here",
        "very",
    }
)


def _content_words(text: str) -> set[str]:
    return {w for w in re.findall(r"\b[a-z]{4,}\b", text.lower()) if w not in _GROUNDING_STOPWORDS}


def python_grounding_novelty(text: str, threshold: float = 0.50) -> bool:
    """Python-level grounding check: word-novelty between Knowledge and Answer.

    Parses 'Knowledge: ...' / 'Context: ...' and 'A: ...' / 'Answer: ...'
    from the formatted benchmark input, then flags if answer introduces
    ≥threshold fraction of content words not in the reference.

    Works especially well for cross-domain swap tests (HalluLens) where
    swapped answers have entirely different vocabulary than the reference.
    Less reliable for HalluMix (hallucinated answers use context vocabulary)
    so it is ONLY applied to HalluLens.
    """
    ref_match = re.search(r"(?:Knowledge|Context):\s*(.+?)(?=\n\n|\nQuestion:)", text, re.DOTALL)
    ans_match = re.search(r"\nA:\s*(.+?)(?:\n\n|$)", text, re.DOTALL)
    if not ans_match:
        ans_match = re.search(r"\nAnswer:\s*(.+?)(?:\n\n|$)", text, re.DOTALL)

    if not ref_match or not ans_match:
        return False

    ref_words = _content_words(ref_match.group(1))
    ans_words = _content_words(ans_match.group(1))

    if len(ans_words) < 1:
        return False

    novel = ans_words - ref_words
    novelty_ratio = len(novel) / len(ans_words)
    return novelty_ratio >= threshold


# ─── Unigram faithfulness (TRIZ P22 Feedback) ────────────────────────────────


def _unigram_faithfulness(text: str) -> float:
    """Fraction of answer tokens that appear in the reference (Knowledge/Context) block."""
    ref_m = re.search(
        r"(?:Knowledge|Context):\s*(.+?)(?=\n\n|\nQuestion:|\nPrompt:|\nDialogue:|\nResponse:)",
        text,
        re.DOTALL,
    )
    ans_m = re.search(r"\n(?:Answer|Response|Conclusion):\s*(.+?)(?:\n\n|$)", text, re.DOTALL)
    if not ref_m or not ans_m:
        return 0.0
    ref_tokens = set(re.sub(r"[^\w\s]", " ", ref_m.group(1).lower()).split())
    ans_tokens = re.sub(r"[^\w\s]", " ", ans_m.group(1).lower()).split()
    if not ans_tokens:
        return 0.0
    overlap = sum(1 for t in ans_tokens if t in ref_tokens)
    return overlap / len(ans_tokens)


# ─── Sentence-transformer semantic scoring (S44/S45) ─────────────────────────

_ST_MODEL = None  # Lazy-loaded singleton


def _get_st_model():
    """Lazy-load all-MiniLM-L6-v2. Returns None if sentence-transformers not installed."""
    global _ST_MODEL
    if _ST_MODEL is None:
        try:
            import warnings

            from sentence_transformers import SentenceTransformer  # type: ignore

            with warnings.catch_warnings():
                warnings.simplefilter("ignore")
                _ST_MODEL = SentenceTransformer("all-MiniLM-L6-v2", device="cpu")
        except (ImportError, Exception):
            _ST_MODEL = False  # sentinel: tried and failed
    return _ST_MODEL if _ST_MODEL else None


def _batch_semantic_scores(
    pairs: list[tuple[str, bool, str]], model, threshold: float = 0.50
) -> dict[str, bool]:
    """Pre-compute cosine similarity scores for all (context, answer) pairs.

    Returns a dict mapping text[:100] → bool (True = below threshold = ISSUE).
    Processes all pairs in one batched encode call for efficiency.
    """
    if model is None:
        return {}

    ctx_texts: list[str] = []
    ans_texts: list[str] = []
    keys: list[str] = []

    for text, _, _ in pairs:
        ctx_m = re.search(
            r"Context:\s*(.+?)(?=\n\nQuestion:|\nQuestion:|\n\nAnswer:|\nAnswer:)",
            text,
            re.DOTALL,
        )
        ans_m = re.search(r"\nAnswer:\s*(.+?)(?:\n\n|$)", text, re.DOTALL)
        ctx = ctx_m.group(1)[:1500] if ctx_m else text[:1500]
        ans = ans_m.group(1)[:400] if ans_m else text[-400:]
        ctx_texts.append(ctx)
        ans_texts.append(ans)
        keys.append(text[:100])

    all_texts = ctx_texts + ans_texts
    try:
        embs = model.encode(
            all_texts,
            batch_size=32,
            normalize_embeddings=True,
            show_progress_bar=False,
        )
    except Exception:
        return {}

    n = len(ctx_texts)
    scores: dict[str, bool] = {}
    for i, key in enumerate(keys):
        cos = float(embs[i] @ embs[n + i])  # ctx_emb · ans_emb
        scores[key] = cos < threshold

    return scores


def _batch_felm_semantic_scores(
    pairs: list[tuple[str, bool, str]], model, threshold: float = 0.82
) -> dict[str, bool]:
    """Compute cosine(prompt, response) for FELM-format pairs.

    FELM format: "Prompt: {prompt}\\nAnswer: {response}"
    Lower cosine = response diverges from prompt topic = hallucination signal.
    """
    if model is None:
        return {}

    prompt_texts: list[str] = []
    response_texts: list[str] = []
    keys: list[str] = []

    for text, _, _ in pairs:
        p_m = re.search(r"^Prompt:\s*(.+?)(?=\nAnswer:)", text, re.DOTALL)
        a_m = re.search(r"\nAnswer:\s*(.+?)$", text, re.DOTALL)
        prompt_t = p_m.group(1)[:512] if p_m else text[:256]
        response_t = a_m.group(1)[:512] if a_m else text[-256:]
        prompt_texts.append(prompt_t)
        response_texts.append(response_t)
        keys.append(text[:100])

    all_texts = prompt_texts + response_texts
    try:
        embs = model.encode(
            all_texts,
            batch_size=32,
            normalize_embeddings=True,
            show_progress_bar=False,
        )
    except Exception:
        return {}

    n = len(prompt_texts)
    return {keys[i]: float(embs[i] @ embs[n + i]) < threshold for i in range(n)}


# ─── Entity novelty grounding (S41 TRIZ P13 + P3) ────────────────────────────

_ENT_SKIP_WORDS = frozenset(
    {
        "The",
        "A",
        "An",
        "He",
        "She",
        "They",
        "It",
        "In",
        "But",
        "And",
        "Or",
        "Not",
        "This",
        "That",
        "His",
        "Her",
        "Their",
        "Its",
        "Also",
        "Then",
        "When",
        "For",
        "With",
        "After",
        "Before",
        "During",
        "While",
        "About",
        "Some",
        "Many",
        "Most",
        "Each",
        "Both",
        "All",
        "Any",
        "One",
        "Two",
        "Three",
        "Four",
        "Five",
        "Six",
        "Seven",
        "Eight",
        "Nine",
        "Ten",
        "Into",
        "From",
        "Over",
        "Under",
        "Between",
        "Through",
        "Upon",
        "Has",
    }
)


def _entity_novelty_grounded(text: str) -> bool:
    """Detect novel specific facts (years, numbers, entities) absent from reference.

    Signals (any one triggers ISSUE):
    1. Novel 4-digit year (19xx/20xx) in answer not in reference
       → high-precision: years are rare, precise, and factual claims.
    2. 2+ novel multi-digit numbers (2+ digits) in answer not in reference
       → catches date fragments, quantities, statistics.
    3. 3+ novel named entities (title-case) in answer not in reference
       → catches entity substitution in summarization/dialogue hallucinations.

    Works for all grounded benchmarks:
      RAGTruth:          Knowledge: ...\\nPrompt: ...\\nAnswer: ...
      FaithBench:        Knowledge: ...\\nAnswer: ...
      HaluEval Dialogue: Knowledge: ...\\nDialogue: ...\\nResponse: ...
    """
    ref_m = re.search(
        r"(?:Knowledge|Context):\s*(.+?)(?=\n\n|\nQuestion:|\nPrompt:|\nDialogue:|\nResponse:|\nAnswer:)",
        text,
        re.DOTALL,
    )
    ans_m = re.search(r"\n(?:Answer|Response):\s*(.+?)(?:\n\n|$)", text, re.DOTALL)
    if not ref_m or not ans_m:
        return False

    ref_text = ref_m.group(1)
    ans_text = ans_m.group(1)

    # Signal 1: novel 4-digit year
    ref_years = set(re.findall(r"\b(?:19|20)\d{2}\b", ref_text))
    ans_years = set(re.findall(r"\b(?:19|20)\d{2}\b", ans_text))
    if ans_years - ref_years:
        return True

    # Signal 2: 2+ novel multi-digit numbers (not years — those caught above)
    ref_nums = set(re.findall(r"\b\d{2,}\b", ref_text)) - ref_years
    ans_nums = set(re.findall(r"\b\d{2,}\b", ans_text)) - ans_years
    if len(ans_nums - ref_nums) >= 2:
        return True

    # Signal 3: 3+ novel named entities (title-case, not in skip-list, not in reference)
    ref_named = set(re.findall(r"\b[A-Z][a-z]{2,}\b", ref_text))
    novel_named = [
        w
        for w in re.findall(r"\b[A-Z][a-z]{2,}\b", ans_text)
        if w not in ref_named and w not in _ENT_SKIP_WORDS
    ]
    return len(novel_named) >= 3
