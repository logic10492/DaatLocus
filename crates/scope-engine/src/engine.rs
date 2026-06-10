use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::analyzer::Analyzer;
use crate::api::*;
use crate::lsp::TsJsConfig;
use crate::lsp::{
    GoplsConfig, JdtlsConfig, LspAnalyzer, LspServerConfig, PyrightConfig, RustAnalyzerConfig,
};
use crate::patch;
use crate::selector::{self, SelectorTarget};
use crate::state::PropagationState;
use crate::treesitter::TreeSitterAnalyzer;
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::{DirEntry, WalkBuilder};
use regex::Regex;
use std::sync::Mutex;

const DEFAULT_SEARCH_LIMIT: usize = 100;
const MAX_SEARCH_LIMIT: usize = 1000;
const DEFAULT_REVIEW_LIMIT: usize = 1;
const MAX_REVIEW_LIMIT: usize = 100;
const DEFAULT_FILE_SEARCH_LIMIT: usize = 100;
const MAX_LSP_DID_OPEN_FILES: usize = 500;

fn lsp_config_for_language(lsp_lang: &str) -> Option<Box<dyn LspServerConfig>> {
    match lsp_lang {
        "rust" => Some(Box::new(RustAnalyzerConfig)),
        "python" => Some(Box::new(PyrightConfig)),
        "typescript" | "javascript" => Some(Box::new(TsJsConfig)),
        "go" => Some(Box::new(GoplsConfig)),
        "java" => Some(Box::new(JdtlsConfig)),
        _ => None,
    }
}

fn lsp_extensions_for_language(lsp_lang: &str) -> &'static [&'static str] {
    match lsp_lang {
        "rust" => &["rs"],
        "python" => &["py"],
        "typescript" | "javascript" => &["ts", "tsx", "js", "jsx"],
        "go" => &["go"],
        "java" => &["java"],
        _ => &[],
    }
}

fn lsp_language_for_extension(ext: &str) -> Option<&'static str> {
    match ext {
        "rs" => Some("rust"),
        "py" => Some("python"),
        "ts" | "tsx" => Some("typescript"),
        "js" | "jsx" => Some("javascript"),
        "go" => Some("go"),
        "java" => Some("java"),
        _ => None,
    }
}

fn detect_project_lsp_language(root: &Path) -> Option<&'static str> {
    if root.join("Cargo.toml").is_file() {
        return Some("rust");
    }
    if root.join("pyproject.toml").is_file()
        || root.join("requirements.txt").is_file()
        || root.join("setup.py").is_file()
    {
        return Some("python");
    }
    if root.join("go.mod").is_file() {
        return Some("go");
    }
    if root.join("pom.xml").is_file()
        || root.join("build.gradle").is_file()
        || root.join("build.gradle.kts").is_file()
    {
        return Some("java");
    }
    if root.join("tsconfig.json").is_file() {
        return Some("typescript");
    }
    if root.join("package.json").is_file() {
        return Some("typescript");
    }

    let mut counts: HashMap<&'static str, usize> = HashMap::new();
    for entry in WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .ignore(true)
        .parents(true)
        .build()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .is_some_and(|file_type| file_type.is_file())
        })
        .take(MAX_LSP_DID_OPEN_FILES)
    {
        let Some(ext) = entry.path().extension().and_then(|ext| ext.to_str()) else {
            continue;
        };
        if let Some(language) = lsp_language_for_extension(ext) {
            *counts.entry(language).or_default() += 1;
        }
    }

    ["rust", "typescript", "javascript", "python", "go", "java"]
        .into_iter()
        .max_by_key(|language| counts.get(language).copied().unwrap_or(0))
        .filter(|language| counts.get(language).copied().unwrap_or(0) > 0)
}

fn open_existing_source_files_for_lsp(lsp: &dyn Analyzer, root: &Path, lsp_lang: &str) {
    let exts = lsp_extensions_for_language(lsp_lang);
    if exts.is_empty() {
        return;
    }
    let scan_root = {
        let src_dir = root.join("src");
        if src_dir.exists() {
            src_dir
        } else {
            root.to_path_buf()
        }
    };
    for entry in WalkBuilder::new(scan_root)
        .hidden(false)
        .git_ignore(true)
        .ignore(true)
        .parents(true)
        .build()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_type()
                .is_some_and(|file_type| file_type.is_file())
        })
        .take(MAX_LSP_DID_OPEN_FILES)
    {
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|ext| ext.to_str())
            && exts.contains(&ext)
            && let Ok(content) = std::fs::read_to_string(path)
        {
            lsp.notify_did_open(path, &content);
        }
    }
}

pub fn open_project(
    project_root: &Path,
    current_project_root: Option<&Path>,
    lsp_analyzer: &Mutex<Option<Box<dyn Analyzer + Send>>>,
) -> Result<OpenProjectResponse, String> {
    if current_project_root == Some(project_root) {
        return Ok(OpenProjectResponse {
            status: "already_open".to_string(),
            project_root: project_root.to_string_lossy().into_owned(),
            detected_lsp_language: None,
            lsp: None,
        });
    }

    let detected_lsp_language = detect_project_lsp_language(project_root);
    let Some(config) = detected_lsp_language.and_then(lsp_config_for_language) else {
        let mut lsp_guard = lsp_analyzer
            .lock()
            .map_err(|_| "lock poisoned".to_string())?;
        *lsp_guard = None;
        return Ok(OpenProjectResponse {
            status: "opened".to_string(),
            project_root: project_root.to_string_lossy().into_owned(),
            detected_lsp_language: detected_lsp_language.map(str::to_string),
            lsp: Some("unsupported".to_string()),
        });
    };

    {
        let mut lsp_guard = lsp_analyzer
            .lock()
            .map_err(|_| "lock poisoned".to_string())?;
        *lsp_guard = None;
        let new_lsp = LspAnalyzer::new(project_root, config.as_ref());
        *lsp_guard = Some(Box::new(new_lsp));
    }

    {
        let lsp_guard = lsp_analyzer
            .lock()
            .map_err(|_| "lock poisoned".to_string())?;
        if let (Some(lsp), Some(lsp_lang)) = (&*lsp_guard, detected_lsp_language) {
            open_existing_source_files_for_lsp(lsp.as_ref(), project_root, lsp_lang);
        }
    }

    Ok(OpenProjectResponse {
        status: "opened".to_string(),
        project_root: project_root.to_string_lossy().into_owned(),
        detected_lsp_language: detected_lsp_language.map(str::to_string),
        lsp: None,
    })
}

pub fn search_code(
    project_root: &Path,
    params: &SearchCodeRequest,
) -> Result<SearchCodeResponse, String> {
    let limit = normalize_search_limit(params.limit);
    let matches = search_project(project_root, &params.query, None, None, limit)?;
    Ok(SearchCodeResponse { matches })
}

pub fn grep_code(
    project_root: &Path,
    params: &GrepCodeRequest,
) -> Result<GrepCodeResponse, String> {
    if params.pattern.is_empty() {
        return Err("pattern is required".to_string());
    }

    let target = project_relative_arg(project_root, params.path.as_deref())?;
    let matches = search_project(
        project_root,
        &params.pattern,
        target.as_deref(),
        params.include.as_deref(),
        DEFAULT_FILE_SEARCH_LIMIT,
    )?;
    Ok(GrepCodeResponse { matches })
}

pub fn glob_files(
    project_root: &Path,
    params: &GlobFilesRequest,
) -> Result<GlobFilesResponse, String> {
    if params.pattern.is_empty() {
        return Err("pattern is required".to_string());
    }

    let target = project_relative_arg(project_root, params.path.as_deref())?;
    let mut files = find_glob_files(project_root, &params.pattern, target.as_deref())?;
    files.sort_by(|(left_path, left_mtime), (right_path, right_mtime)| {
        right_mtime
            .cmp(left_mtime)
            .then_with(|| left_path.cmp(right_path))
    });

    let truncated = files.len() > DEFAULT_FILE_SEARCH_LIMIT;
    files.truncate(DEFAULT_FILE_SEARCH_LIMIT);
    let files = files.into_iter().map(|(path, _)| path).collect::<Vec<_>>();
    Ok(GlobFilesResponse { files, truncated })
}

pub fn is_responsible_source(
    project_root: &Path,
    params: &IsResponsibleSourceRequest,
) -> Result<IsResponsibleSourceResponse, String> {
    let relative = project_relative_arg(project_root, Some(&params.path))?
        .ok_or_else(|| "path is required".to_string())?;
    let path = project_root.join(&relative);
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_string);
    let analyzer = TreeSitterAnalyzer::new();
    let language = extension
        .as_deref()
        .and_then(|ext| analyzer.responsible_language_for_extension(ext))
        .map(str::to_string);
    let is_responsible = language.is_some();
    let reason = match (&extension, &language) {
        (Some(extension), Some(language)) => {
            format!("SCOPE recognizes .{extension} as {language} source")
        }
        (Some(extension), None) => {
            format!("SCOPE has no source adapter for .{extension}")
        }
        (None, _) => "path has no file extension for SCOPE source ownership".to_string(),
    };

    Ok(IsResponsibleSourceResponse {
        is_responsible,
        path: relative,
        extension,
        language,
        reason,
    })
}

fn normalize_search_limit(limit: Option<usize>) -> usize {
    limit
        .unwrap_or(DEFAULT_SEARCH_LIMIT)
        .clamp(1, MAX_SEARCH_LIMIT)
}

fn search_project(
    project_root: &Path,
    pattern: &str,
    target: Option<&str>,
    include: Option<&str>,
    limit: usize,
) -> Result<Vec<SearchMatch>, String> {
    let regex = Regex::new(pattern).map_err(|e| format!("regex error: {e}"))?;
    let include = build_optional_glob_set(include)?;
    let analyzer = TreeSitterAnalyzer::new();
    let mut matches = Vec::new();

    for entry in project_files(project_root, target)? {
        let path = entry.path();
        let relative = relative_file_path(project_root, path);
        if let Some(include) = include.as_ref()
            && !include.is_match(&relative)
        {
            continue;
        }

        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        for (line_index, text) in content.lines().enumerate() {
            if !regex.is_match(text) {
                continue;
            }
            let line = line_index + 1;
            let symbol_match = analyzer.find_containing_symbol_match(path, line);
            let selector = symbol_match
                .as_ref()
                .map(|symbol| symbol.canonical_selector(path, project_root));
            matches.push(SearchMatch {
                file: relative.clone(),
                line,
                text: text.to_string(),
                selector,
            });
            if matches.len() >= limit {
                return Ok(matches);
            }
        }
    }

    Ok(matches)
}

fn find_glob_files(
    project_root: &Path,
    pattern: &str,
    target: Option<&str>,
) -> Result<Vec<(String, SystemTime)>, String> {
    let glob_set = build_glob_set(pattern)?;
    let mut files = Vec::new();
    for entry in project_files(project_root, target)? {
        let path = entry.path();
        let relative = relative_file_path(project_root, path);
        if glob_set.is_match(&relative) {
            let mtime = entry
                .metadata()
                .or_else(|_| fs::metadata(path))
                .and_then(|metadata| metadata.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            files.push((relative, mtime));
        }
    }
    Ok(files)
}

fn project_files(project_root: &Path, target: Option<&str>) -> Result<Vec<DirEntry>, String> {
    let root = project_root.to_path_buf();
    let walk_root = match target {
        Some(target) => root.join(target),
        None => root.clone(),
    };

    let mut entries = Vec::new();
    for entry in WalkBuilder::new(walk_root)
        .standard_filters(false)
        .hidden(false)
        .parents(false)
        .build()
    {
        let entry = entry.map_err(|e| format!("failed to walk project files: {e}"))?;
        if !entry
            .file_type()
            .is_some_and(|file_type| file_type.is_file())
        {
            continue;
        }
        entries.push(entry);
    }

    entries.sort_by(|left, right| {
        relative_file_path(&root, left.path()).cmp(&relative_file_path(&root, right.path()))
    });
    Ok(entries)
}

fn build_optional_glob_set(pattern: Option<&str>) -> Result<Option<GlobSet>, String> {
    let Some(pattern) = pattern.map(str::trim).filter(|pattern| !pattern.is_empty()) else {
        return Ok(None);
    };
    build_glob_set(pattern).map(Some)
}

fn build_glob_set(pattern: &str) -> Result<GlobSet, String> {
    let mut builder = GlobSetBuilder::new();
    builder.add(Glob::new(pattern).map_err(|e| format!("glob error: {e}"))?);
    if !pattern.contains('/') && !pattern.contains('\\') {
        builder.add(Glob::new(&format!("**/{pattern}")).map_err(|e| format!("glob error: {e}"))?);
    }
    builder.build().map_err(|e| format!("glob error: {e}"))
}

fn relative_file_path(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .ok()
        .map(|path| normalize_relative_path(&path.to_string_lossy()))
        .unwrap_or_else(|| normalize_relative_path(&path.to_string_lossy()))
}

pub fn format_grep_output(matches: &[SearchMatch]) -> String {
    if matches.is_empty() {
        return "No files found".to_string();
    }

    let mut lines = vec![format!("Found {} matches", matches.len()), String::new()];
    let mut current_file: Option<&str> = None;
    let mut current_selector: Option<Option<&str>> = None;

    for item in matches {
        if current_file != Some(item.file.as_str()) {
            if current_file.is_some() {
                lines.push(String::new());
            }
            lines.push(format!("{}:", item.file));
            current_file = Some(item.file.as_str());
            current_selector = None;
        }

        let selector = item.selector.as_deref();
        if current_selector != Some(selector) {
            match selector {
                Some(selector) => lines.push(format!("  {selector}:")),
                None => lines.push("  Unclassified matches:".to_string()),
            }
            current_selector = Some(selector);
        }

        lines.push(format!("    Line {}: {}", item.line, item.text));
    }

    lines.join("\n")
}

pub fn format_glob_output(files: &[String], truncated: bool) -> String {
    let mut lines = Vec::new();
    if files.is_empty() {
        lines.push("No files found".to_string());
    } else {
        lines.extend(files.iter().cloned());
    }

    if truncated {
        lines.push(String::new());
        lines.push(
            "(Results are truncated: showing first 100 results. Consider using a more specific path or pattern.)"
                .to_string(),
        );
    }

    lines.join("\n")
}

fn project_relative_arg(root: &Path, path: Option<&str>) -> Result<Option<String>, String> {
    let Some(path) = path.map(str::trim).filter(|path| !path.is_empty()) else {
        return Ok(None);
    };
    let path = PathBuf::from(path);
    if path.is_absolute() {
        let relative = path.strip_prefix(root).map_err(|_| {
            format!(
                "path {} is outside project root {}",
                path.display(),
                root.display()
            )
        })?;
        Ok(Some(normalize_relative_path(&relative.to_string_lossy())))
    } else {
        Ok(Some(normalize_relative_path(&path.to_string_lossy())))
    }
}

fn normalize_relative_path(path: &str) -> String {
    path.replace('\\', "/")
}

struct ResolvedReadSelection {
    start_line: usize,
    end_line: usize,
    content_override: Option<String>,
}

fn resolve_read_selection(
    analyzer: &TreeSitterAnalyzer,
    full_path: &Path,
    _project_root: &Path,
    file_content: &str,
    parsed: &selector::ParsedSelector,
) -> Result<ResolvedReadSelection, String> {
    let line_count = file_content.lines().count().max(1);
    match &parsed.target {
        SelectorTarget::Symbol(_) => {
            let symbol = analyzer.resolve_selector(full_path, parsed)?;
            Ok(ResolvedReadSelection {
                start_line: symbol.start_line,
                end_line: symbol.end_line,
                content_override: None,
            })
        }
        SelectorTarget::LineRange {
            start_line,
            end_line,
        } => {
            let (start_line, end_line) = clamp_range(*start_line, *end_line, line_count)?;
            Ok(ResolvedReadSelection {
                start_line,
                end_line,
                content_override: None,
            })
        }
        SelectorTarget::AroundLine { line, context } => {
            let start_line = line.saturating_sub(*context).max(1);
            let end_line = (*line + *context).min(line_count);
            Ok(ResolvedReadSelection {
                start_line,
                end_line,
                content_override: None,
            })
        }
        SelectorTarget::Match { pattern, around } => {
            let regex = Regex::new(pattern).map_err(|e| format!("regex error: {e}"))?;
            let mut hits = file_content
                .lines()
                .enumerate()
                .filter_map(|(idx, line)| regex.is_match(line).then_some(idx + 1))
                .collect::<Vec<_>>();
            match hits.len() {
                0 => Err(format!("match selector found no matches for /{pattern}/")),
                1 => {
                    let line = hits.remove(0);
                    let (start_line, end_line, _kind) = if let Some(context) = around {
                        (
                            line.saturating_sub(*context).max(1),
                            (line + *context).min(line_count),
                            "match_around",
                        )
                    } else {
                        (line, line, "match")
                    };
                    Ok(ResolvedReadSelection {
                        start_line,
                        end_line,
                        content_override: None,
                    })
                }
                _ => Err(format!(
                    "match selector is ambiguous for /{pattern}/; candidate lines: {}",
                    hits.iter()
                        .map(|line| line.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
            }
        }
        SelectorTarget::BeforeLine { line } | SelectorTarget::AfterLine { line } => {
            let (start_line, end_line) = clamp_range(*line, *line, line_count)?;
            Ok(ResolvedReadSelection {
                start_line,
                end_line,
                content_override: None,
            })
        }
        SelectorTarget::Enclosing { line } => {
            let symbol = analyzer
                .find_containing_symbol_match(full_path, *line)
                .ok_or_else(|| {
                    format!(
                        "no enclosing symbol found at {} line {}",
                        full_path.display(),
                        line
                    )
                })?;
            Ok(ResolvedReadSelection {
                start_line: symbol.start_line,
                end_line: symbol.end_line,
                content_override: None,
            })
        }
        SelectorTarget::Outline => {
            let symbols = analyzer.symbols_in_file(full_path)?;
            let content = symbols
                .iter()
                .map(|symbol| {
                    format!(
                        "{}{} #L{}-L{}",
                        symbol.kind_prefix, symbol.name, symbol.start_line, symbol.end_line
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            let end_line = content.lines().count().max(1);
            Ok(ResolvedReadSelection {
                start_line: 1,
                end_line,
                content_override: Some(if content.is_empty() {
                    String::new()
                } else {
                    format!("{content}\n")
                }),
            })
        }
    }
}

fn clamp_range(
    start_line: usize,
    end_line: usize,
    line_count: usize,
) -> Result<(usize, usize), String> {
    if start_line == 0 || end_line == 0 || start_line > end_line {
        return Err(format!("invalid line range {start_line}-{end_line}"));
    }
    if start_line > line_count {
        return Err(format!(
            "line range starts after end of file: {start_line} > {line_count}"
        ));
    }
    Ok((start_line, end_line.min(line_count)))
}

fn read_line_range(content: &str, start_line: usize, end_line: usize) -> String {
    let lines = content.lines().collect::<Vec<_>>();
    if start_line == 0 || end_line < start_line || start_line > lines.len() {
        return String::new();
    }
    let mut snippet = lines[(start_line - 1)..end_line.min(lines.len())].join("\n");
    if content.ends_with('\n') || end_line < lines.len() {
        snippet.push('\n');
    }
    snippet
}

pub fn read_code(
    project_root: &Path,
    params: &ReadCodeRequest,
) -> Result<ReadCodeResponse, String> {
    let parsed =
        selector::parse_selector(&params.selector).map_err(|e| format!("Bad selector: {e}"))?;
    let (full_path, _ext) = selector::resolve_file(&parsed, project_root)?;
    let file_content = std::fs::read_to_string(&full_path)
        .map_err(|e| format!("Failed to read {}: {e}", full_path.display()))?;
    let analyzer = TreeSitterAnalyzer::new();
    let resolved =
        resolve_read_selection(&analyzer, &full_path, project_root, &file_content, &parsed)?;
    let raw_content = resolved
        .content_override
        .clone()
        .unwrap_or_else(|| read_line_range(&file_content, resolved.start_line, resolved.end_line));

    let path = relative_file_path(project_root, &full_path);
    let prefixed_content = prefix_lines_with_hash(&raw_content, resolved.start_line);

    Ok(ReadCodeResponse {
        path,
        content: prefixed_content,
    })
}

fn line_hash(line_content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(line_content.as_bytes());
    let result = hasher.finalize();
    format!("{:02x}", result[0])
}

fn prefix_lines_with_hash(content: &str, start_line: usize) -> String {
    content
        .lines()
        .enumerate()
        .map(|(i, line)| {
            let line_num = start_line + i;
            let hash = line_hash(line);
            format!("{line_num}#{hash}|{line}")
        })
        .collect::<Vec<_>>()
        .join("\n")
        + if content.ends_with('\n') || content.is_empty() {
            "\n"
        } else {
            ""
        }
}

pub fn edit_code(
    project_root: &Path,
    params: &EditCodeRequest,
    propagation_state: &Mutex<PropagationState>,
    lsp_analyzer: &Mutex<Option<Box<dyn Analyzer + Send>>>,
) -> Result<PropagationResponse, String> {
    match patch::edit_code_apply(&params.edits, project_root, lsp_analyzer) {
        Ok(results) => {
            if !results.is_empty()
                && let Ok(mut state) = propagation_state.lock()
            {
                state.accumulate(results.clone());
            }
            Ok(PropagationResponse {
                propagation_results: results,
            })
        }
        Err(e) => Err(e),
    }
}

pub fn config_hints() -> serde_json::Value {
    use crate::language::LanguageRegistry;
    use crate::lsp::{
        GoplsConfig, JdtlsConfig, LspServerConfig, PyrightConfig, RustAnalyzerConfig, TsJsConfig,
    };

    let registry = LanguageRegistry::new();
    let configs: Vec<Box<dyn LspServerConfig>> = vec![
        Box::new(RustAnalyzerConfig),
        Box::new(PyrightConfig),
        Box::new(TsJsConfig), // covers both TS and JS
        Box::new(GoplsConfig),
        Box::new(JdtlsConfig),
    ];

    let mut languages = Vec::new();

    // Tree-sitter languages from registry
    let ts_langs: Vec<serde_json::Value> = registry
        .list_languages()
        .into_iter()
        .map(|(name, exts)| {
            serde_json::json!({
                "name": name,
                "extensions": exts,
            })
        })
        .collect();

    // LSP configs
    for cfg in &configs {
        let binary_found = std::process::Command::new("which")
            .arg(cfg.binary_name())
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let mut lang_entry = serde_json::json!({
            "language": cfg.language_id(),
            "lsp_server": cfg.server_name(),
            "lsp_binary": cfg.binary_name(),
            "lsp_available": binary_found,
            "setup_hints": cfg.setup_hints(),
        });

        if let Some((cmd, args)) = cfg.install_command() {
            lang_entry["install_command"] = serde_json::json!({
                "command": cmd,
                "args": args,
            });
        }

        if let Some(url) = cfg.download_url() {
            lang_entry["download_url"] = serde_json::json!(url);
        }

        languages.push(lang_entry);
    }

    serde_json::json!({
        "tree_sitter_languages": ts_langs,
        "lsp_languages": languages,
    })
}

pub fn ack_next_events(
    propagation_state: &Mutex<PropagationState>,
    limit: Option<usize>,
) -> Result<NextReviewResponse, String> {
    let limit = limit
        .unwrap_or(DEFAULT_REVIEW_LIMIT)
        .clamp(1, MAX_REVIEW_LIMIT);
    let mut state = propagation_state
        .lock()
        .map_err(|_| "lock poisoned".to_string())?;
    let reviews = state.next_reviews(limit);
    let review = reviews.first().cloned();
    let returned = reviews.len();
    let remaining = state.pending_count();
    Ok(NextReviewResponse {
        review,
        reviews,
        returned,
        remaining,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::PropagationState;
    use std::sync::Mutex;

    #[test]
    fn open_project_detects_lsp_language_from_project_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"tmp\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
        )
        .unwrap();
        let lsp_analyzer: Mutex<Option<Box<dyn Analyzer + Send>>> = Mutex::new(None);

        let response = open_project(dir.path(), None, &lsp_analyzer).unwrap();

        assert_eq!(response.detected_lsp_language.as_deref(), Some("rust"));
    }

    #[test]
    fn open_project_is_idempotent_for_current_project_root() {
        let dir = tempfile::tempdir().unwrap();
        let lsp_analyzer: Mutex<Option<Box<dyn Analyzer + Send>>> = Mutex::new(None);

        let response = open_project(dir.path(), Some(dir.path()), &lsp_analyzer).unwrap();

        assert_eq!(response.status, "already_open");
        assert_eq!(response.detected_lsp_language, None);
    }

    #[test]
    fn grep_output_groups_matches_by_file_and_selector() {
        let matches = vec![
            SearchMatch {
                file: "src/coding_app.rs".to_string(),
                line: 10,
                text: "fn build_usage() {".to_string(),
                selector: Some("src/coding_app.rs::fn build_usage #L10-L20".to_string()),
            },
            SearchMatch {
                file: "src/coding_app.rs".to_string(),
                line: 12,
                text: "usage.push_str(\"grep\");".to_string(),
                selector: Some("src/coding_app.rs::fn build_usage #L10-L20".to_string()),
            },
            SearchMatch {
                file: "src/coding_app.rs".to_string(),
                line: 3,
                text: "use std::path::PathBuf;".to_string(),
                selector: None,
            },
        ];

        assert_eq!(
            format_grep_output(&matches),
            "Found 3 matches\n\nsrc/coding_app.rs:\n  src/coding_app.rs::fn build_usage #L10-L20:\n    Line 10: fn build_usage() {\n    Line 12: usage.push_str(\"grep\");\n  Unclassified matches:\n    Line 3: use std::path::PathBuf;"
        );
    }

    #[test]
    fn glob_output_matches_opencode_style() {
        let files = vec!["src/coding_app.rs".to_string(), "src/app.rs".to_string()];

        assert_eq!(
            format_glob_output(&files, true),
            "src/coding_app.rs\nsrc/app.rs\n\n(Results are truncated: showing first 100 results. Consider using a more specific path or pattern.)"
        );
    }

    #[test]
    fn grep_code_filters_by_path_and_include() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src/nested")).unwrap();
        std::fs::create_dir_all(dir.path().join("tests")).unwrap();
        std::fs::write(
            dir.path().join("src/lib.rs"),
            "pub fn lib() { let needle = true; }\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("src/nested/mod.rs"),
            "pub fn nested() { let needle = true; }\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("tests/lib_test.rs"),
            "pub fn test() { let needle = true; }\n",
        )
        .unwrap();

        let response = grep_code(
            dir.path(),
            &GrepCodeRequest {
                pattern: "needle".to_string(),
                path: Some("src".to_string()),
                include: Some("*.rs".to_string()),
            },
        )
        .unwrap();

        let files = response
            .matches
            .iter()
            .map(|item| item.file.as_str())
            .collect::<Vec<_>>();
        assert_eq!(files, vec!["src/lib.rs", "src/nested/mod.rs"]);
        assert_eq!(response.matches.len(), 2);
    }

    #[test]
    fn glob_files_sorts_by_mtime_and_filters_path() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::create_dir_all(dir.path().join("tests")).unwrap();
        std::fs::write(dir.path().join("src/old.rs"), "old").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        std::fs::write(dir.path().join("src/new.rs"), "new").unwrap();
        std::fs::write(dir.path().join("tests/newer.rs"), "newer").unwrap();

        let response = glob_files(
            dir.path(),
            &GlobFilesRequest {
                pattern: "*.rs".to_string(),
                path: Some("src".to_string()),
            },
        )
        .unwrap();

        assert_eq!(response.files, vec!["src/new.rs", "src/old.rs"]);
        assert!(!response.truncated);
    }

    #[test]
    fn search_code_honors_explicit_limit() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("lib.rs"),
            "pub fn one() { let needle = 1; }\npub fn two() { let needle = 2; }\npub fn three() { let needle = 3; }\n",
        )
        .unwrap();

        let response = search_code(
            dir.path(),
            &SearchCodeRequest {
                query: "needle".to_string(),
                limit: Some(2),
            },
        )
        .unwrap();
        assert_eq!(response.matches.len(), 2);
    }

    #[test]
    fn search_code_clamps_zero_limit_to_one() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("lib.rs"),
            "pub fn one() { let needle = 1; }\npub fn two() { let needle = 2; }\n",
        )
        .unwrap();

        let response = search_code(
            dir.path(),
            &SearchCodeRequest {
                query: "needle".to_string(),
                limit: Some(0),
            },
        )
        .unwrap();
        assert_eq!(response.matches.len(), 1);
    }

    #[test]
    fn read_code_returns_only_resolved_symbol_range() {
        let dir = tempfile::tempdir().unwrap();
        let source = "pub fn first() {\n    println!(\"first\");\n}\n\npub fn second() {\n    println!(\"second\");\n}\n";
        std::fs::write(dir.path().join("lib.rs"), source).unwrap();

        let result = read_code(
            dir.path(),
            &ReadCodeRequest {
                selector: "lib.rs::fn second".to_string(),
            },
        )
        .expect("read_code should return a result");

        assert_eq!(result.path, "lib.rs");
        let content = result.content.as_str();
        assert!(
            content.contains("pub fn second()"),
            "should contain fn second, got: {content}"
        );
        assert!(!content.contains("first"));
        // Check line-hash format: line_number#hash|content
        assert!(
            content.lines().all(|line| {
                let parts: Vec<&str> = line.splitn(2, '|').collect();
                parts.len() == 2 && parts[0].contains('#')
            }),
            "each line should have line#hash| prefix, got: {content}"
        );
    }
    #[test]
    fn read_code_supports_line_range_enclosing_outline() {
        let dir = tempfile::tempdir().unwrap();
        let source = "pub fn first() {\n    println!(\"first\");\n}\n\npub fn second() {\n    println!(\"second\");\n}\n";
        std::fs::write(dir.path().join("lib.rs"), source).unwrap();

        let result = read_code(
            dir.path(),
            &ReadCodeRequest {
                selector: "lib.rs#L5-L7".to_string(),
            },
        )
        .unwrap();
        assert_eq!(result.path, "lib.rs");
        let content = result.content.as_str();
        assert!(
            content.contains("pub fn second()"),
            "should contain fn second, got: {content}"
        );
        assert!(
            !content.contains("pub fn first()"),
            "should not contain fn first"
        );

        let result = read_code(
            dir.path(),
            &ReadCodeRequest {
                selector: "lib.rs#enclosing:L6".to_string(),
            },
        )
        .unwrap();
        assert_eq!(result.path, "lib.rs");
        let content = result.content.as_str();
        assert!(
            content.contains("pub fn second()"),
            "enclosing should contain fn second, got: {content}"
        );

        let result = read_code(
            dir.path(),
            &ReadCodeRequest {
                selector: "lib.rs#outline".to_string(),
            },
        )
        .unwrap();
        assert_eq!(result.path, "lib.rs");
        assert!(result.content.contains("fn second #L5-L7"));
    }

    #[test]
    fn ack_next_event_can_return_a_limited_batch() {
        let propagation_state = Mutex::new(PropagationState::new());
        propagation_state.lock().unwrap().accumulate(vec![
            PropagationResult {
                selector: "src/a.rs::fn foo".to_string(),
                reason: "first".to_string(),
                source: PropagationSource::Lsp,
                lsp_references: Some(vec![]),
                diff_summary: None,
                file_snippet: None,
                project_files: None,
            },
            PropagationResult {
                selector: "src/b.rs::fn bar".to_string(),
                reason: "second".to_string(),
                source: PropagationSource::Lsp,
                lsp_references: Some(vec![]),
                diff_summary: None,
                file_snippet: None,
                project_files: None,
            },
            PropagationResult {
                selector: "src/c.rs::fn baz".to_string(),
                reason: "third".to_string(),
                source: PropagationSource::Lsp,
                lsp_references: Some(vec![]),
                diff_summary: None,
                file_snippet: None,
                project_files: None,
            },
        ]);

        let result = ack_next_events(&propagation_state, Some(2)).unwrap();

        assert_eq!(result.returned, 2);
        assert_eq!(result.remaining, 1);
        assert_eq!(result.reviews.len(), 2);
        match result.review.unwrap() {
            ReviewEvent::KnownReferences {
                modified_symbol, ..
            } => assert_eq!(modified_symbol, "src/c.rs::fn baz"),
            _ => panic!("expected KnownReferences review"),
        }
    }

    #[test]
    fn scope_usage_is_available() {
        let result = serde_json::to_value(crate::usage::usage_response()).unwrap();
        assert!(
            result["usage_markdown"]
                .as_str()
                .unwrap()
                .contains("positioning DSL")
        );
        assert!(
            result["selector_kinds"]
                .as_array()
                .unwrap()
                .iter()
                .any(|kind| kind["kind"] == "outline" && kind["edit"] == false)
        );
    }

    #[test]
    fn is_responsible_source_reports_scope_owned_source() {
        let dir = tempfile::tempdir().unwrap();

        let result = is_responsible_source(
            dir.path(),
            &IsResponsibleSourceRequest {
                path: "src/lib.rs".to_string(),
            },
        )
        .unwrap();
        assert!(result.is_responsible);
        assert_eq!(result.path, "src/lib.rs");
        assert_eq!(result.extension.as_deref(), Some("rs"));
        assert_eq!(result.language.as_deref(), Some("rust"));
    }

    #[test]
    fn is_responsible_source_reports_non_source_file() {
        let dir = tempfile::tempdir().unwrap();

        let result = is_responsible_source(
            dir.path(),
            &IsResponsibleSourceRequest {
                path: "README.md".to_string(),
            },
        )
        .unwrap();
        assert!(!result.is_responsible);
        assert_eq!(result.path, "README.md");
        assert_eq!(result.extension.as_deref(), Some("md"));
        assert_eq!(result.language, None);
    }
}
