function stripMarkdownEmphasis(value: string) {
  const trimmed = value.trim();
  const strongMatch = trimmed.match(/^(\*\*|__)([\s\S]+)\1$/);
  if (strongMatch) {
    return strongMatch[2].trim();
  }
  const emphasisMatch = trimmed.match(/^(\*|_)([\s\S]+)\1$/);
  if (emphasisMatch) {
    return emphasisMatch[2].trim();
  }
  return trimmed.replace(/^#{1,6}\s+/, "").trim();
}

function isMarkdownBlockSyntax(value: string) {
  return /^(#{1,6}\s+|[-*+]\s+|\d+[.)]\s+|>\s*|```|~~~|\|)/.test(value);
}

function isStandaloneThinkingHeading(value: string) {
  const trimmed = value.trim();
  if (!trimmed || isMarkdownBlockSyntax(trimmed)) {
    return false;
  }

  const text = stripMarkdownEmphasis(trimmed);
  if (!text || text.length > 88 || /[.!?。！？]$/.test(text)) {
    return false;
  }

  if (/^(\*\*|__)[\s\S]+(\*\*|__)$/.test(trimmed)) {
    return true;
  }

  const words = text.split(/\s+/).filter(Boolean);
  if (words.length > 9) {
    return false;
  }

  return /^[A-Z]/.test(text) || /^[\p{Script=Han}]/u.test(text);
}

function lastOutputLineIsBlank(lines: string[]) {
  return lines.length === 0 || lines[lines.length - 1].trim() === "";
}

function nextSourceLineIsBlank(lines: string[], index: number) {
  return index + 1 >= lines.length || lines[index + 1].trim() === "";
}

export function normalizeThinkingMarkdown(text: string) {
  const normalized = splitEmbeddedThinkingHeadings(
    text.replace(/\r\n/g, "\n").replace(/\r/g, "\n"),
  );
  const lines = normalized.split("\n");
  const output: string[] = [];
  let inFence = false;

  lines.forEach((line, index) => {
    const trimmed = line.trim();
    const fenceLine = /^(```|~~~)/.test(trimmed);
    const heading = !inFence && isStandaloneThinkingHeading(trimmed);

    if (heading && !lastOutputLineIsBlank(output)) {
      output.push("");
    }

    output.push(line);

    if (heading && !nextSourceLineIsBlank(lines, index)) {
      output.push("");
    }

    if (fenceLine) {
      inFence = !inFence;
    }
  });

  return output.join("\n");
}

function splitEmbeddedThinkingHeadings(text: string) {
  let output = "";
  let cursor = 0;

  while (true) {
    const start = text.indexOf("**", cursor);
    if (start < 0) {
      break;
    }
    const endMarker = text.indexOf("**", start + 2);
    if (endMarker < 0) {
      break;
    }
    const end = endMarker + 2;
    const candidate = text.slice(start, end);

    output += text.slice(cursor, start);
    if (embeddedThinkingHeadingNeedsBreak(text, start, end, candidate)) {
      output = appendParagraphBreak(output);
    }
    output += candidate;
    cursor = end;
  }

  return output + text.slice(cursor);
}

function embeddedThinkingHeadingNeedsBreak(
  text: string,
  start: number,
  end: number,
  candidate: string,
) {
  if (start === 0 || text.slice(0, start).endsWith("\n")) {
    return false;
  }
  if (!text.slice(end).startsWith("\n\n")) {
    return false;
  }
  const previous = text.slice(start - 1, start);
  return previous.trim() !== "" && isStandaloneThinkingHeading(candidate);
}

function appendParagraphBreak(text: string) {
  if (text.endsWith("\n\n")) {
    return text;
  }
  if (text.endsWith("\n")) {
    return `${text}\n`;
  }
  return `${text}\n\n`;
}
