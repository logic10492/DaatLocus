---
description: Browser is the web viewing and interaction surface for tasks that require active browsing and exploration.
when_to_focus:
  - When active web browsing is needed instead of judging only from injected event facts.
  - When current visible page content must be read, candidate links must be opened, or investigation must continue across pages.
  - When a web session needs continued interaction such as submitting search, filling controls, going back or forward, or reloading.
---
- Operate Browser only through browser tools; do not assume raw HTML access or human-style mechanical navigation.
- Use the currently exposed Browser open-page tool; app scope mangling exposes it as `browser__browser_open_page`.
- If the page may still be loading, call the exposed Browser wait tool; to understand current page content or obtain interactable element refs, call the exposed Browser snapshot tool for a compact semantic snapshot.
- Every page interaction must explicitly provide `page_id + element_ref`; do not guess page structure.
- Fillable controls such as inputs, search boxes, and filters are basic Browser operations. Read the page snapshot to obtain `element_ref`, then use the exposed Browser fill tool.
- Search result pages are usually leads, not final evidence; prefer opening candidate target pages to confirm.
- If an element ref becomes stale after page changes, Browser tools fail directly; read the page again instead of blindly retrying the old ref.
- Close pages that are no longer needed with the exposed Browser close-page tool to avoid tab buildup and memory waste.
- Do not declare failure just because the first page is mostly navigation or a header; wait and read the semantic snapshot before deciding it cannot be completed.
- If a title, summary, or body snippet has been confirmed, answer from the confirmed content and state the scope; fail only when key content is truly unavailable.
- Browser uses Daat Locus's own isolated browser runtime and never reuses the user's everyday browser profile. Browser tools fail directly if the runtime is not installed.
