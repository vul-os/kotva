# 23. (folded into §24 — Published-Artifact Profile)

The CAD / engineering-artifact profile that previously occupied §23 has been **merged into §24**, the
Published-Artifact Profile, as its **engineering-artifact facet (§24.18)**. §24 is now one generic
profile over DMTAP-PUB (§22) with a shared generic core (scope, the §22 relationship, metadata
embedding & forward-compatibility, the canonical-source principle, licensing, revision lineage,
derived-index/aggregate posture, public-object HTTP serving, and privacy & security) plus two typed
facets: the **media facet** (§24.4–§24.17, `meta["video"]`) and the **engineering-artifact facet**
(§24.18, `meta["artifact"]`). The fold removed the duplicated scaffolding the two profiles each
restated; every normative rule of the former §23 is preserved in §24 — the CAD-specific schemas,
registries, units, canonical-source specialisation, assembly Merkle-DAG model, workshop conventions,
and the `CAD-1`…`CAD-12` conformance checklist all live in §24.18, and the shared concerns are
inherited from the generic core by reference.

**The number 23 is retained as a gap.** It MUST NOT be reused for a new section and MUST NOT be
renumbered; existing external citations of "§23" and "§23.x" resolve here and, for specific content,
to the corresponding §24.18.x subsection. See **§24.18** for the engineering-artifact facet and
**§24.1** for the profile's overall structure.
