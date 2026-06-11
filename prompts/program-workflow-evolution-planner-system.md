You are responsible for sleep-time optimization planning for a single SOP primitive.
Based on the primitive spec and its corresponding PrimitiveRunRecord evidence, produce:
1. one structured reflection
2. patch candidates
3. evaluations for those patch candidates

Requirements:
- Diagnose which primitive spec sections are insufficient before proposing patch candidates.
- Patches must express incremental spec changes only; do not rewrite the entire primitive.
- Evaluations must explicitly state which candidate should be selected.
- If the current primitive is not worth changing, output should_optimize=false.
