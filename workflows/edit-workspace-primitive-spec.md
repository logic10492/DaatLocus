---
id: edit-workspace-primitive-spec
---

## When To Use
- The user asks to change, clean up, replace, tighten, relax, or otherwise maintain an existing workspace SOP primitive spec.
- The user asks to modify the SOP primitive for a past, existing, or previously discussed task class.
- The requested change concerns the reusable SOP primitive behind that task class, even if the user still says "workflow".
- The user gives a follow-up instruction that refers to the previously discussed or recently used task process rather than only the current one-off result.
- The task is about editing a persisted SOP primitive specification, not executing the target primitive itself.

## Preconditions
- The user intent names, strongly implies, or is contextually attached to a target SOP primitive.
- The target SOP primitive corresponds to a past, existing, or previously discussed task class.
- The target SOP primitive is workspace-origin, not a read-only builtin primitive.
- The edit is intended to change reusable future behavior, not only to complete the current one-off task.

## Workflow
1. Identify the target SOP primitive from the `afterclaim_context` primitive routing catalog, user wording, recent conversation context, the currently discussed primitive, and primitive/workflow ids.
2. Activate this meta primitive as the current primitive binding; do not activate the target primitive being edited.
3. Call `read_primitive_spec` for the target primitive id before deciding edits.
4. Map the user's intent to the correct spec sections: `When To Use`, `Preconditions`, `Workflow`, `Done Criteria`, and `Recovery`.
5. Produce a complete replacement primitive spec, preserving still-valid existing items and removing duplicates, contradictions, stale task-specific details, and sleep-patch bloat.
6. Call `update_primitive_spec` with all sections of the cleaned replacement spec.
7. Review the tool result and send a concise final reply describing which primitive spec changed and what behavior changed.

## Done Criteria
- The target SOP primitive was identified by id.
- The complete current target primitive spec was read before editing.
- The edit changed only a workspace SOP primitive spec.
- The final primitive remains reusable for a task class rather than recording a one-off execution log.
- The replacement spec is concise, internally consistent, and free of obvious duplicate rules.
- The user receives a concise summary of the primitive spec change.

## Recovery
- If the target primitive is ambiguous after using conversation context and primitive summaries, ask the user to identify the task class or primitive id.
- If the target primitive is builtin, explain that builtin primitives are read-only and offer to create or edit a workspace SOP primitive instead.
- If the requested change conflicts with existing primitive behavior, replace the conflicting rule instead of appending a contradictory rule.
- If a requested edit would make the primitive too narrow for reuse, ask for confirmation or keep the reusable boundary explicit.
