"""Z3 variable registry and NL→Z3 parsing for syllogism verification.

Sentence parsing uses spaCy dependency trees instead of surface-form regex,
eliminating all hardcoded vocabulary.  Requires ``pureason[nlp]``.
"""

from __future__ import annotations

from ._z3utils import _get_nlp, _lemma, _norm_id, _pred_key, _prop_key


def _pred_from_subtree(token) -> str:
    """Build a canonical predicate string from a token's content subtree.

    Collects non-stop, non-punctuation lemmas from the token's subtree in
    document order and joins them with ``_``.  Falls back to the token's own
    lemma when the subtree yields nothing.
    """
    parts = [
        t.lemma_
        for t in token.subtree
        if not t.is_stop and not t.is_punct and not t.is_space and t.lemma_
    ]
    return "_".join(parts) or token.lemma_


class _Z3Context:
    """Z3 variable registry for bounded-domain syllogism checking."""

    def __init__(self, entities: list[str]) -> None:
        self.entities = [_lemma(e) for e in entities]
        self._ep: dict[str, dict[str, object]] = {}
        self._prop: dict[str, object] = {}

    def _is_entity(self, name: str) -> bool:
        """Return True if *name* (after lemmatization) is a known entity."""
        return _lemma(name) in self.entities

    def ep(self, entity: str, pred: str) -> object:
        """Return a Z3 Bool for the (entity, normalised-predicate) pair."""
        from z3 import Bool

        ent_k = _norm_id(_lemma(entity))
        pred_k = _pred_key(pred) or _norm_id(pred)
        if ent_k not in self._ep:
            self._ep[ent_k] = {}
        if pred_k not in self._ep[ent_k]:
            self._ep[ent_k][pred_k] = Bool(f"ep_{ent_k}__{pred_k}")
        return self._ep[ent_k][pred_k]

    def prop(self, clause: str) -> object:
        """Return a Z3 Bool for a propositional clause (content-word key)."""
        from z3 import Bool

        k = _prop_key(clause)
        if k not in self._prop:
            self._prop[k] = Bool(f"prop__{k}")
        return self._prop[k]

    def parse_prop(self, text: str) -> tuple[object, bool]:
        """Parse a condition clause into ``(z3_var, is_negated)``.

        Uses the spaCy dependency tree to find the subject and predicate.
        Falls back to a propositional atom for clauses without a recognisable
        entity subject.
        """
        nlp = _get_nlp()
        text = text.strip().rstrip(".")
        doc = nlp(text)
        is_neg = any(t.dep_ == "neg" for t in doc)

        for token in doc:
            if token.dep_ not in {"nsubj", "nsubjpass"}:
                continue
            if not self._is_entity(token.lemma_):
                continue
            head = token.head
            # Collect predicate: skip subject, neg markers, aux, det tokens
            skip_deps = {"nsubj", "nsubjpass", "neg", "aux", "auxpass", "det", "punct"}
            parts = [
                t.lemma_
                for t in head.subtree
                if t.dep_ not in skip_deps
                and not t.is_stop
                and not t.is_punct
                and not t.is_space
                and t != token
                and t.lemma_
            ]
            pred = "_".join(parts) or head.lemma_
            return self.ep(token.text, pred), is_neg

        return self.prop(text), is_neg

    def universal_constraints(
        self, subj_class: str, pred: str, negated: bool = False
    ) -> list[object]:
        """Encode ``All subj_class are pred`` over every entity in the domain."""
        from z3 import Implies, Not

        return [
            Implies(self.ep(e, subj_class), Not(self.ep(e, pred)) if negated else self.ep(e, pred))
            for e in self.entities
        ]

    def parse_sentence(self, text: str) -> list[object] | None:
        """Parse one NL sentence into Z3 constraints using spaCy dep trees.

        Handles these grammatical patterns without any hardcoded vocabulary:

        - ``All/Every X are/have/V Y``  → universal constraint
        - ``No X are Y``               → negated universal constraint
        - ``Some X can/are Y``         → existential witness
        - ``Every X who V gets Y``     → conditional with relative clause
        - ``If P then Q``              → material implication
        - ``P implies Q``              → material implication
        - Entity-specific subject      → entity-predicate atom (with/without neg)
        - Propositional fallback       → bare propositional atom

        Returns ``None`` only when the text is empty.
        """
        from z3 import And, Implies, Not

        nlp = _get_nlp()
        t = text.strip().rstrip(".")
        if not t:
            return None

        doc = nlp(t)

        # Find the sentence root
        root = next((tok for tok in doc if tok.dep_ == "ROOT"), None)
        if root is None:
            return self._prop_fallback(t, doc)

        # ── Conditional: "If P, then Q" ──────────────────────────────────
        advcl = next(
            (c for c in root.children if c.dep_ == "advcl"),
            None,
        )
        if advcl is not None:
            mark = next((c for c in advcl.children if c.dep_ == "mark"), None)
            if mark and mark.lemma_.lower() == "if":
                p_text = " ".join(tok.text for tok in advcl.subtree if tok.dep_ != "mark")
                # Q = main clause tokens not in the advcl subtree
                advcl_ids = {tok.i for tok in advcl.subtree}
                q_text = " ".join(
                    tok.text
                    for tok in doc
                    if tok.i not in advcl_ids
                    and tok.dep_ not in {"advmod", "punct"}
                    and tok.lemma_.lower() not in {"then"}
                )
                p_var, p_neg = self.parse_prop(p_text.strip())
                q_var, q_neg = self.parse_prop(q_text.strip())
                return [Implies(Not(p_var) if p_neg else p_var, Not(q_var) if q_neg else q_var)]

        # ── "P implies Q" ─────────────────────────────────────────────────
        if root.lemma_ == "imply":
            ccomp = next((c for c in root.children if c.dep_ in {"ccomp", "xcomp"}), None)
            nsubj = next((c for c in root.children if c.dep_ == "nsubj"), None)
            if ccomp and nsubj:
                p_text = " ".join(tok.text for tok in nsubj.subtree)
                q_text = " ".join(tok.text for tok in ccomp.subtree)
                p_var, p_neg = self.parse_prop(p_text.strip())
                q_var, q_neg = self.parse_prop(q_text.strip())
                return [Implies(Not(p_var) if p_neg else p_var, Not(q_var) if q_neg else q_var)]

        # ── Universal / quantified patterns ──────────────────────────────
        nsubj = next((c for c in root.children if c.dep_ in {"nsubj", "nsubjpass"}), None)

        if nsubj is not None:
            det_tokens = [c for c in nsubj.children if c.dep_ == "det"]
            det_text = det_tokens[0].lemma_.lower() if det_tokens else ""
            is_neg_sent = any(c.dep_ == "neg" for c in root.children)

            # "All/Every X …"
            if det_text in {"all", "every"}:
                subj_class = nsubj.lemma_

                # Relative clause: "Every X who V gets Y"
                relcl = next((c for c in nsubj.children if c.dep_ == "relcl"), None)
                if relcl is not None:
                    action = "who__" + _norm_id(
                        " ".join(tok.lemma_ for tok in relcl.subtree if not tok.is_stop)
                    )
                    dobj = next((c for c in root.children if c.dep_ == "dobj"), None)
                    result_pred = _pred_from_subtree(dobj) if dobj else root.lemma_
                    return [
                        Implies(
                            And(self.ep(e, subj_class), self.ep(e, action)),
                            self.ep(e, result_pred),
                        )
                        for e in self.entities
                    ]

                # "All X are/is Y" (root=be, attr or acomp)
                if root.lemma_ == "be":
                    pred_tok = next(
                        (c for c in root.children if c.dep_ in {"attr", "acomp"}),
                        None,
                    )
                    if pred_tok:
                        pred = _pred_from_subtree(pred_tok)
                        return self.universal_constraints(subj_class, pred, negated=is_neg_sent)

                # "All X have Y"
                if root.lemma_ == "have":
                    dobj = next((c for c in root.children if c.dep_ == "dobj"), None)
                    if dobj:
                        pred = "have__" + _pred_from_subtree(dobj)
                        return self.universal_constraints(subj_class, pred, negated=is_neg_sent)

                # "All X V O" (any other content verb)
                dobj = next((c for c in root.children if c.dep_ == "dobj"), None)
                if dobj:
                    pred = f"{root.lemma_}__{_pred_from_subtree(dobj)}"
                    return self.universal_constraints(subj_class, pred, negated=is_neg_sent)

                # "All X V" (intransitive)
                return self.universal_constraints(subj_class, root.lemma_, negated=is_neg_sent)

            # "No X are Y" — det="no" or negation on root
            if det_text == "no":
                subj_class = nsubj.lemma_
                if root.lemma_ == "be":
                    pred_tok = next(
                        (c for c in root.children if c.dep_ in {"attr", "acomp"}),
                        None,
                    )
                    if pred_tok:
                        pred = _pred_from_subtree(pred_tok)
                        return self.universal_constraints(subj_class, pred, negated=True)

            # "Some X can/are Y" — existential, create witness
            if det_text == "some":
                subj_class = nsubj.lemma_
                # Predicate = xcomp, attr, acomp, or ROOT itself
                pred_tok = next(
                    (c for c in root.children if c.dep_ in {"attr", "acomp", "xcomp", "dobj"}),
                    None,
                )
                pred = _pred_from_subtree(pred_tok) if pred_tok else root.lemma_
                witness = f"witness_{_norm_id(subj_class)}"
                return [self.ep(witness, subj_class), self.ep(witness, pred)]

            # ── Entity-specific patterns ──────────────────────────────────
            if self._is_entity(nsubj.lemma_):
                is_neg_ent = any(c.dep_ == "neg" for c in root.children)

                # "X is/are (not) [a] Y"
                if root.lemma_ == "be":
                    pred_tok = next(
                        (c for c in root.children if c.dep_ in {"attr", "acomp"}),
                        None,
                    )
                    if pred_tok:
                        pred = _pred_from_subtree(pred_tok)
                        ep_var = self.ep(nsubj.text, pred)
                        return [Not(ep_var) if is_neg_ent else ep_var]

                # "X has/have (not) Y"
                if root.lemma_ == "have":
                    dobj = next((c for c in root.children if c.dep_ == "dobj"), None)
                    if dobj:
                        pred = "have__" + _pred_from_subtree(dobj)
                        ep_var = self.ep(nsubj.text, pred)
                        return [Not(ep_var) if is_neg_ent else ep_var]

                # "X V (not) O" or "X V" (content verb)
                dobj = next((c for c in root.children if c.dep_ == "dobj"), None)
                if dobj:
                    pred = f"{root.lemma_}__{_pred_from_subtree(dobj)}"
                else:
                    pred = root.lemma_
                ep_var = self.ep(nsubj.text, pred)
                return [Not(ep_var) if is_neg_ent else ep_var]

        # ── Propositional fallback ────────────────────────────────────────
        return self._prop_fallback(t, doc)

    def _prop_fallback(self, text: str, doc) -> list[object]:
        """Return a bare propositional Z3 atom for unrecognised sentence structures."""
        from z3 import Not

        is_neg = any(t.dep_ == "neg" for t in doc)
        var = self.prop(text)
        return [Not(var) if is_neg else var]
