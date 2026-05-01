"""TF-IDF+LogReg classifier for syllogism validity (no hardcoded vocabulary)."""

from __future__ import annotations

import numpy as np
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.linear_model import LogisticRegression


def _train_syllogism_classifier(
    premises_list: list[list[str]], conclusions: list[str], labels: list[int]
) -> tuple[TfidfVectorizer, LogisticRegression]:
    """Train a TF-IDF+LogReg classifier on syllogism validity.

    Parameters
    ----------
    premises_list : list[list[str]]
        List of premise groups (each group is a list of strings).
    conclusions : list[str]
        List of conclusion strings.
    labels : list[int]
        Binary labels: 1 = valid, 0 = invalid.

    Returns
    -------
    tuple[TfidfVectorizer, LogisticRegression]
        Trained vectorizer and classifier.
    """
    vectorizer = TfidfVectorizer(
        max_features=200,
        lowercase=True,
        ngram_range=(1, 2),
        stop_words="english",
        min_df=1,
    )

    features = [
        "\n".join(prems) + " | " + concl for prems, concl in zip(premises_list, conclusions)
    ]
    X = vectorizer.fit_transform(features)

    clf = LogisticRegression(max_iter=100, random_state=42)
    clf.fit(X, np.array(labels))
    return vectorizer, clf


def syllogism_clf_predict(
    vectorizer: TfidfVectorizer, clf: LogisticRegression, premises: list[str], conclusion: str
) -> bool | None:
    """Predict syllogism validity using trained classifier.

    Returns True (valid), False (invalid), or None (prediction not confident).
    """
    text = "\n".join(premises) + " | " + conclusion
    X = vectorizer.transform([text])
    proba = clf.predict_proba(X)[0]

    if max(proba) < 0.6:
        return None

    return bool(clf.predict(X)[0] == 1)
