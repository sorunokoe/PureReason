#!/usr/bin/env python3
"""Train and export the arithmetic operation classifier.

Generates ~600 labeled word-problem examples (150 per operator), trains a
TF-IDF + LogisticRegression pipeline on spaCy-lemmatized text, and serializes
the weights to ``data/op_classifier.npz`` as pure numpy arrays so inference
requires no sklearn at runtime.

Usage::

    python scripts/train_op_classifier.py

Outputs:
    data/op_train.jsonl   — training corpus (readable, inspectable)
    data/op_classifier.npz — serialized weights (vocab, idf, coef, intercept)

Accuracy target: ≥ 90 % on a held-out 20 % split.
"""

from __future__ import annotations

import json
import random
from pathlib import Path

import numpy as np
import spacy
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import classification_report
from sklearn.model_selection import train_test_split
from sklearn.pipeline import Pipeline

# ---------------------------------------------------------------------------
# Training data templates
# ---------------------------------------------------------------------------

_NAMES = ["Alice", "Bob", "Maria", "Tom", "Sara", "Leo", "Aisha", "Jake", "Priya", "Kim"]
_ITEMS = ["apples", "books", "coins", "cookies", "pencils", "stamps", "cards", "oranges"]
_CONTAINERS = ["boxes", "bags", "crates", "baskets", "shelves", "trays", "rows", "groups"]
_UNITS = ["dollars", "euros", "tokens", "points", "liters", "meters", "kilograms"]
_FINANCIAL_UNITS = ["dollars", "euros", "tokens", "points"]  # for addition context
_TIME = ["day", "hour", "week", "shift"]

# Verbs that strongly signal each operator (base forms, each inflected to past)
_ADD_VERBS_PAST = [
    "earned",
    "received",
    "bought",
    "collected",
    "gained",
    "found",
    "got",
    "won",
    "acquired",
    "obtained",
    "saved",
    "accumulated",
    "picked up",
    "secured",
    "added",
    "deposited",
    "loaded",
    "grabbed",
    "fetched",
    "gathered",
]
_SUB_VERBS_PAST = [
    "sold",
    "spent",
    "used",
    "removed",
    "ate",
    "lost",
    "decreased",
    "cut",
    "took",
    "subtracted",
    "reduced",
    "leaked",
    "moved",
    "donated",
    "gave away",
    "traded",
    "discarded",
    "broke",
    "consumed",
    "dropped",
    "wasted",
    "donated",
    "threw away",
    "distributed",
    "handed out",
    "lent",
    "sent away",
    "delivered",
]
_MUL_VERBS = [
    "contains",
    "holds",
    "packs",
    "fills",
    "produces",
    "makes",
    "manufactures",
    "yields",
    "stores",
    "arranges",
    "stacks",
    "plants",
    "organizes",
    "sets up",
    "groups",
    "bundles",
]
_DIV_VERBS_PAST = [
    "divided",
    "split",
    "shared",
    "distributed",
    "allocated",
    "portioned",
    "rationed",
    "spread",
    "apportioned",
    "separated",
    "parcelled",
    "assigned equally",
    "doled out",
]


def _name() -> str:
    return random.choice(_NAMES)


def _item() -> str:
    return random.choice(_ITEMS)


def _container() -> str:
    return random.choice(_CONTAINERS)


def _unit() -> str:
    return random.choice(_UNITS)


def _n(lo: int = 2, hi: int = 50) -> int:
    return random.randint(lo, hi)


def _gen_add() -> list[str]:
    examples = []
    for v in _ADD_VERBS_PAST:
        n = _name()
        it = _item()
        amt = _n()
        # Bare statement (no question suffix — crucial for generalization)
        examples.append(f"{n} {v} {amt} {it}.")
        examples.append(f"{n} {v} {amt} more {it}.")
        examples.append(f"How many {it} does {n} have after {v.rstrip('d')}ing {amt}?")
        # Also with financial unit words so "earn dollars" etc. appear in addition class
        u = random.choice(_FINANCIAL_UNITS)
        examples.append(f"{n} {v} {amt} {u}.")

    # Keyword signals
    for _ in range(25):
        examples.append(f"How many {_item()} in total if they had {_n()} and got {_n()} more?")
        examples.append(f"Find the sum of {_n()} and {_n()}.")
        examples.append(f"What is the combined total of {_n()} {_item()} and {_n()} {_item()}?")
        examples.append(f"Altogether they have {_n()} {_item()} and {_n()} {_item()}.")
        examples.append(f"How many {_item()} are there altogether?")
        examples.append("How many items are there in total?")

    return examples


def _gen_sub() -> list[str]:
    examples = []
    for v in _SUB_VERBS_PAST:
        n = _name()
        it = _item()
        amt = _n()
        # Bare statements (key: does NOT end in "left?" — classifier must learn verb semantics)
        examples.append(f"{n} {v} {amt} {it}.")
        examples.append(f"{n} {v} {_n()} {it} yesterday.")
        # With remainder question
        examples.append(f"{n} had {_n(30, 80)} {it} but {v} {amt}. How many are left?")
        examples.append(f"After {n} {v} {amt} {it}, how many remain?")

    # Remainder signals
    for _ in range(20):
        n = _name()
        examples.append(f"How many {_item()} remain after removing {_n()}?")
        examples.append(f"What is the difference between {_n(20, 50)} and {_n(1, 19)}?")
        examples.append(f"How many are left if {_n()} {_item()} were taken away?")
        examples.append(f"{n} lost {_n()} {_item()}.")
        examples.append(f"{n} removed {_n()} items.")

    return examples


def _gen_mul() -> list[str]:
    examples = []
    for v in _MUL_VERBS:
        cont = _container()
        it = _item()
        n1, n2 = _n(2, 12), _n(2, 12)
        examples.append(f"Each {cont[:-1]} {v} {n1} {it}. There are {n2} {cont}.")
        examples.append(f"There are {n2} {cont} and each {v} {n1} {it}.")
        examples.append(f"If {n2} {cont} each {v} {n1} {it}, how many {it} are there?")
        # Bare statement with just the verb
        examples.append(f"The factory {v} {n1} units.")
        examples.append(f"A machine {v} {n1} {it} per cycle with {n2} machines.")

    # Per/rate signals — do NOT use "earn" here (conflicts with addition class)
    for _ in range(20):
        n = _name()
        examples.append(
            f"{n} works {_n(2, 10)} {random.choice(_TIME)}s at a rate of {_n()} {_unit()} per {random.choice(_TIME)}."
        )
        examples.append(f"There are {_n()} rows with {_n()} {_item()} in each row.")
        examples.append(
            f"A machine produces {_n()} {_item()} per {random.choice(_TIME)}. How many in {_n(2, 8)} {random.choice(_TIME)}s?"
        )
        examples.append(f"Each box contains {_n()} apples and there are {_n(2, 12)} boxes.")

    return examples


def _gen_div() -> list[str]:
    examples = []
    for v in _DIV_VERBS_PAST:
        n = _name()
        it = _item()
        total, parts = _n(10, 80), _n(2, 10)
        examples.append(f"{n} {v} {total} {it} equally among {parts} people. How many each?")
        examples.append(f"{total} {it} were {v} among {parts} groups equally.")
        examples.append(f"If {total} {it} are {v} into {parts} equal parts, how many per part?")
        # Bare statement
        examples.append(f"{n} {v} {total} {it} among {parts} friends.")

    # Average / equal-share / per signals
    for _ in range(25):
        examples.append(f"What is the average of {_n()} and {_n()}?")
        examples.append(
            f"Find the average score if the total is {_n(50, 200)} over {_n(2, 8)} tests."
        )
        examples.append(
            f"Each person gets an equal share of {_n(10, 100)} {_item()} split among {_n(2, 10)} people."
        )
        examples.append(
            f"How many {_item()} per person if {_n()} are shared equally among {_n(2, 8)}?"
        )
        examples.append("Costs are calculated per kilometer.")
        examples.append(f"The price is {_n()} {_unit()} per {_item()[:-1]}.")

    return examples


# ---------------------------------------------------------------------------
# spaCy lemmatization helper
# ---------------------------------------------------------------------------

_NLP_CACHE = None


def _lemmatize(text: str) -> str:
    global _NLP_CACHE
    if _NLP_CACHE is None:
        _NLP_CACHE = spacy.load("en_core_web_sm", disable=["ner"])
    doc = _NLP_CACHE(text.lower())
    # Exclude punctuation, whitespace, and pronouns — pronouns cause classifier bias
    return " ".join(t.lemma_ for t in doc if not t.is_punct and not t.is_space and t.pos_ != "PRON")


# ---------------------------------------------------------------------------
# Main training pipeline
# ---------------------------------------------------------------------------


def main() -> None:
    random.seed(42)

    # Build corpus
    corpus = (
        [(_t, "+") for _t in _gen_add()]
        + [(_t, "-") for _t in _gen_sub()]
        + [(_t, "*") for _t in _gen_mul()]
        + [(_t, "/") for _t in _gen_div()]
    )
    random.shuffle(corpus)
    texts, labels = zip(*corpus, strict=True)

    print(
        f"Total examples: {len(texts)}  ({labels.count('+')}/+ {labels.count('-')}/- {labels.count('*')}/* {labels.count('/')}/÷)"
    )

    # Lemmatize
    print("Lemmatizing with spaCy …")
    lemmatized = [_lemmatize(t) for t in texts]

    # Save training corpus
    data_dir = Path(__file__).parent.parent / "data"
    data_dir.mkdir(exist_ok=True)
    jsonl_path = data_dir / "op_train.jsonl"
    with jsonl_path.open("w", encoding="utf-8") as f:
        for text, label in zip(texts, labels, strict=True):
            f.write(json.dumps({"text": text, "label": label}) + "\n")
    print(f"Training corpus saved → {jsonl_path}")

    # Train / evaluate
    X_train, X_test, y_train, y_test = train_test_split(
        lemmatized, labels, test_size=0.2, random_state=42, stratify=labels
    )

    pipeline = Pipeline(
        [
            (
                "tfidf",
                TfidfVectorizer(
                    ngram_range=(1, 2),
                    sublinear_tf=True,
                    min_df=1,
                    token_pattern=r"(?u)\b[a-zA-Z][a-zA-Z]+\b",  # letters only — exclude digits
                    stop_words="english",  # remove function words ("of", "the", "in"…)
                ),
            ),
            ("clf", LogisticRegression(max_iter=1000, C=1.0, random_state=42)),
        ]
    )
    pipeline.fit(X_train, y_train)

    y_pred = pipeline.predict(X_test)
    print("\nClassification report (held-out 20%):")
    print(classification_report(y_test, y_pred, target_names=["+", "-", "*", "/"]))

    acc = (np.array(y_pred) == np.array(y_test)).mean()
    if acc < 0.90:
        print(f"WARNING: accuracy {acc:.1%} is below 90% target.")
    else:
        print(f"Accuracy: {acc:.1%} ✓")

    # Serialize to numpy for dependency-free inference
    tfidf = pipeline.named_steps["tfidf"]
    clf = pipeline.named_steps["clf"]

    vocab_terms = [None] * len(tfidf.vocabulary_)
    for term, idx in tfidf.vocabulary_.items():
        vocab_terms[idx] = term

    clf_path = data_dir / "op_classifier.npz"
    np.savez(
        clf_path,
        vocab=np.array(vocab_terms, dtype=object),
        idf=tfidf.idf_.astype(np.float32),
        coef=clf.coef_.astype(np.float32),
        intercept=clf.intercept_.astype(np.float32),
        classes=np.array(clf.classes_, dtype=object),
        stop_words=np.array(sorted(tfidf.get_stop_words()), dtype=object),
    )
    print(f"Classifier weights saved → {clf_path}")
    print(f"  Vocabulary size: {len(vocab_terms)}")
    print(f"  Stop words excluded: {len(tfidf.get_stop_words())}")
    print(f"  Classes: {clf.classes_}")


if __name__ == "__main__":
    main()
