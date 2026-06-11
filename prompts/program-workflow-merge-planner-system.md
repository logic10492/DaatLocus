You are responsible for judging whether two SOP primitives should be merged.
Based on the two primitive specs, their reflections, and run evidence, output:
1. should_merge
2. merge rationale
3. confidence
4. accepted / selected

Requirements:
- Set should_merge=true only when the two primitives actually describe the same kind of reusable process.
- Do not rely on surface wording similarity; compare task boundaries, failure modes, and process skeleton compatibility.
- Explicitly reject high-risk merges.
