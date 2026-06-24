use std::{
    error::Error,
    fs,
    io::{self, BufWriter, IsTerminal, Write},
    path::{Component, Path, PathBuf},
    process::{Command, ExitCode},
};

use clap::{Args, CommandFactory, Parser, Subcommand};
use serde::Deserialize;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const DEFAULT_BINARY_PACKAGE_DIR_NAME: &str = "package";
const WINDOWS_MSI_TARGET: &str = "x86_64-pc-windows-msvc";
const WINDOWS_LAUNCHER_PACKAGE_NAME: &str = "daat-locus-launcher";
const WINDOWS_MSI_UTIL_EXTENSION: &str = "WixToolset.Util.wixext";
const WINDOWS_BOOTSTRAPPER_EXTENSION: &str = "WixToolset.BootstrapperApplications.wixext";
const WINDOWS_MSI_ICON_SIZES: &[u32] = &[16, 24, 32, 48, 64, 128, 256];
const WINDOWS_BOOTSTRAPPER_LOGO_SIZE: u32 = 128;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(XtaskCommand::Package(args)) => match args.command {
            PackageSubcommand::Binary(args) => package_release_binary(args)?,
            PackageSubcommand::Windows(args) => package_windows_msi(args)?,
        },
        None => {
            let mut command = Cli::command();
            command.print_help()?;
            println!();
        }
    }
    Ok(())
}

#[derive(Debug, Parser)]
#[command(name = "xtask", about = "Project automation commands.")]
struct Cli {
    #[command(subcommand)]
    command: Option<XtaskCommand>,
}

#[derive(Debug, Subcommand)]
enum XtaskCommand {
    /// Build project package artifacts.
    Package(PackageArgs),
}

#[derive(Debug, Args)]
struct PackageArgs {
    #[command(subcommand)]
    command: PackageSubcommand,
}

#[derive(Debug, Subcommand)]
enum PackageSubcommand {
    /// Package an already-built release binary for cargo-binstall.
    Binary(PackageReleaseArgs),

    /// Build the Windows x64 MSI and bootstrapper installers.
    Windows(PackageWindowsMsiArgs),
}

#[derive(Debug, Args)]
struct PackageWindowsMsiArgs {
    #[arg(long, hide = true)]
    skip_build: bool,

    #[arg(long, hide = true)]
    keep_work_dir: bool,
}

#[derive(Debug, Args)]
struct PackageReleaseArgs {
    #[arg(long, value_name = "TARGET")]
    target: Option<String>,

    #[arg(long, value_name = "PATH")]
    release_dir: Option<PathBuf>,

    #[arg(
        long,
        value_name = "PATH",
        help = "Output directory (defaults to target/<target>/release/package)"
    )]
    out_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct RootManifest {
    package: RootPackage,
}

#[derive(Debug, Deserialize)]
struct RootPackage {
    name: String,
    version: String,
    description: Option<String>,
    license: Option<String>,
    #[serde(default)]
    authors: Vec<String>,
    repository: Option<String>,
}

struct WindowsMsiPaths {
    work_dir: PathBuf,
    output_dir: PathBuf,
    binary_path: PathBuf,
    launcher_binary_path: PathBuf,
    icon_path: PathBuf,
    bootstrapper_logo_path: PathBuf,
    license_rtf_path: PathBuf,
    generated_wxs_path: PathBuf,
    generated_bundle_wxs_path: PathBuf,
    msi_path: PathBuf,
    bootstrapper_path: PathBuf,
}

struct WindowsMsiTemplateData {
    product_name: String,
    package_name: String,
    version: String,
    manufacturer: String,
    description: String,
    license: String,
    repository: String,
    binary_name: String,
    binary_path: String,
    launcher_binary_path: String,
    msi_path: String,
    icon_path: String,
    bootstrapper_logo_path: String,
    license_rtf_path: String,
    upgrade_code: String,
    bundle_upgrade_code: String,
    app_component_guid: String,
    launcher_component_guid: String,
    path_component_guid: String,
    shortcut_component_guid: String,
}

fn package_release_binary(args: PackageReleaseArgs) -> Result<()> {
    let target = match args.target {
        Some(target) => target,
        None => rustc_host_target()?,
    };
    ensure_safe_relative_path("target triple", Path::new(&target))?;
    let manifest = read_root_manifest()?;
    let repo = repo_root();
    let release_dir = args
        .release_dir
        .unwrap_or_else(|| repo.join("target").join(&target).join("release"));
    let binary_name = binary_name(&manifest.package.name);
    let binary_path = release_dir.join(&binary_name);
    if !binary_path.is_file() {
        return Err(format!(
            "release binary missing at {}; build it first with `cargo build --release --target {}`",
            binary_path.display(),
            target
        )
        .into());
    }

    let package_dir_name = format!(
        "{}-{}-{}",
        manifest.package.name, manifest.package.version, target
    );
    let stage_root = repo.join("target").join("xtask").join("package-release");
    if stage_root.exists() {
        fs::remove_dir_all(&stage_root)?;
    }
    let package_dir = stage_root.join(&package_dir_name);
    fs::create_dir_all(&package_dir)?;
    fs::copy(&binary_path, package_dir.join(&binary_name))?;

    let out_dir = match &args.out_dir {
        Some(out_dir) if out_dir.is_absolute() => out_dir.clone(),
        Some(out_dir) => repo.join(out_dir),
        None => release_dir.join(DEFAULT_BINARY_PACKAGE_DIR_NAME),
    };
    fs::create_dir_all(&out_dir)?;
    let archive_path = out_dir.join(format!(
        "{}-{}-{}.tar.zst",
        manifest.package.name, manifest.package.version, target
    ));
    if archive_path.exists() {
        fs::remove_file(&archive_path)?;
    }
    archive_tar_zst(&stage_root, &archive_path)?;

    let archive_entry = format!("{package_dir_name}/{binary_name}");
    if !tar_zst_contains_entry(&archive_path, &archive_entry)? {
        return Err(format!(
            "release archive {} does not contain required entry `{archive_entry}`",
            archive_path.display()
        )
        .into());
    }

    print_packaged_artifact(&format!("release binary for {target}"), &archive_path);
    Ok(())
}
fn package_windows_msi(args: PackageWindowsMsiArgs) -> Result<()> {
    if !cfg!(windows) {
        return Err("Windows installer packaging requires Windows".into());
    }

    let manifest = read_root_manifest()?;
    let repo = repo_root();
    let binary_name = binary_name(&manifest.package.name);
    let paths = windows_msi_paths(&repo, &manifest.package, &binary_name)?;

    if !args.skip_build {
        run_command(
            Command::new("cargo")
                .arg("build")
                .arg("-p")
                .arg(&manifest.package.name)
                .arg("-p")
                .arg(WINDOWS_LAUNCHER_PACKAGE_NAME)
                .arg("--release")
                .arg("--locked")
                .arg("--target")
                .arg(WINDOWS_MSI_TARGET),
            "build Windows release binaries",
        )?;
    }

    if !paths.binary_path.is_file() {
        return Err(format!(
            "release binary missing at {}; run `cargo xtask package windows` without --skip-build to build it",
            paths.binary_path.display()
        )
        .into());
    }
    if !paths.launcher_binary_path.is_file() {
        return Err(format!(
            "launcher binary missing at {}; run `cargo xtask package windows` without --skip-build to build it",
            paths.launcher_binary_path.display()
        )
        .into());
    }

    if paths.work_dir.exists() && !args.keep_work_dir {
        fs::remove_dir_all(&paths.work_dir)?;
    }
    fs::create_dir_all(&paths.work_dir)?;
    fs::create_dir_all(&paths.output_dir)?;

    render_svg_icon_to_ico(&repo.join("assets").join("logo.svg"), &paths.icon_path)?;
    render_svg_to_png(
        &repo.join("assets").join("logo.svg"),
        &paths.bootstrapper_logo_path,
        WINDOWS_BOOTSTRAPPER_LOGO_SIZE,
        WINDOWS_BOOTSTRAPPER_LOGO_SIZE,
    )?;
    render_text_file_to_rtf(&repo.join("LICENSE"), &paths.license_rtf_path)?;
    let template_data = windows_msi_template_data(&manifest.package, &paths, &binary_name)?;
    render_windows_msi_template(
        &repo
            .join("packaging")
            .join("windows")
            .join("daat-locus.wxs"),
        &paths.generated_wxs_path,
        &template_data,
    )?;

    if paths.msi_path.exists() {
        fs::remove_file(&paths.msi_path)?;
    }

    run_command(
        Command::new("wix")
            .arg("build")
            .arg(&paths.generated_wxs_path)
            .arg("-ext")
            .arg(WINDOWS_MSI_UTIL_EXTENSION)
            .arg("-o")
            .arg(&paths.msi_path),
        "build Windows MSI",
    )?;

    if !paths.msi_path.is_file() {
        return Err(format!(
            "WiX did not create expected MSI at {}",
            paths.msi_path.display()
        )
        .into());
    }

    render_windows_msi_template(
        &repo
            .join("packaging")
            .join("windows")
            .join("daat-locus-bootstrapper.wxs"),
        &paths.generated_bundle_wxs_path,
        &template_data,
    )?;

    if paths.bootstrapper_path.exists() {
        fs::remove_file(&paths.bootstrapper_path)?;
    }

    run_command(
        Command::new("wix")
            .arg("build")
            .arg(&paths.generated_bundle_wxs_path)
            .arg("-ext")
            .arg(WINDOWS_BOOTSTRAPPER_EXTENSION)
            .arg("-o")
            .arg(&paths.bootstrapper_path),
        "build Windows bootstrapper",
    )?;

    if !paths.bootstrapper_path.is_file() {
        return Err(format!(
            "WiX did not create expected bootstrapper at {}",
            paths.bootstrapper_path.display()
        )
        .into());
    }

    print_packaged_artifact("Windows MSI", &paths.msi_path);
    print_packaged_artifact("Windows bootstrapper", &paths.bootstrapper_path);
    Ok(())
}

fn print_packaged_artifact(label: &str, path: &Path) {
    let url = file_url(path);
    let link = terminal_hyperlink(&url, &url);
    println!("{} {label} at {link}", cargo_status("Packaged"));
}

fn cargo_status(status: &str) -> String {
    if io::stdout().is_terminal() {
        format!("\x1b[1m\x1b[92m{status:>12}\x1b[0m")
    } else {
        format!("{status:>12}")
    }
}

fn terminal_hyperlink(url: &str, text: &str) -> String {
    if io::stdout().is_terminal() {
        format!("\x1b]8;;{url}\x1b\\{text}\x1b]8;;\x1b\\")
    } else {
        text.to_string()
    }
}

fn file_url(path: &Path) -> String {
    let absolute = path.canonicalize().unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            repo_root().join(path)
        }
    });
    let mut path_text = absolute.display().to_string();
    if let Some(stripped) = path_text.strip_prefix(r"\\?\UNC\") {
        path_text = format!(r"\\{stripped}");
    } else if let Some(stripped) = path_text.strip_prefix(r"\\?\") {
        path_text = stripped.to_string();
    }

    let normalized = path_text.replace('\\', "/");
    if let Some(unc_path) = normalized.strip_prefix("//") {
        format!("file://{}", percent_encode_file_url_path(unc_path))
    } else if normalized.starts_with('/') {
        format!("file://{}", percent_encode_file_url_path(&normalized))
    } else {
        format!("file:///{}", percent_encode_file_url_path(&normalized))
    }
}

fn percent_encode_file_url_path(path: &str) -> String {
    let mut encoded = String::with_capacity(path.len());
    for byte in path.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' | b':' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

fn windows_msi_paths(
    repo: &Path,
    package: &RootPackage,
    main_binary_name: &str,
) -> Result<WindowsMsiPaths> {
    ensure_safe_relative_path("target triple", Path::new(WINDOWS_MSI_TARGET))?;
    let release_dir = repo.join("target").join(WINDOWS_MSI_TARGET).join("release");
    let work_dir = release_dir.join("msi-work");
    let output_dir = release_dir.join("msi");
    let msi_path = output_dir.join(format!(
        "{}-{}-{}.msi",
        package.name, package.version, WINDOWS_MSI_TARGET
    ));
    let bootstrapper_path = output_dir.join(format!(
        "{}-{}-{}-setup.exe",
        package.name, package.version, WINDOWS_MSI_TARGET
    ));

    Ok(WindowsMsiPaths {
        binary_path: release_dir.join(main_binary_name),
        launcher_binary_path: release_dir.join(binary_name(WINDOWS_LAUNCHER_PACKAGE_NAME)),
        icon_path: work_dir.join(format!("{}.ico", package.name)),
        bootstrapper_logo_path: work_dir.join(format!("{}-bootstrapper-logo.png", package.name)),
        license_rtf_path: work_dir.join(format!("{}-license.rtf", package.name)),
        generated_wxs_path: work_dir.join(format!("{}.wxs", package.name)),
        generated_bundle_wxs_path: work_dir.join(format!("{}-bootstrapper.wxs", package.name)),
        work_dir,
        output_dir,
        msi_path,
        bootstrapper_path,
    })
}

fn windows_msi_template_data(
    package: &RootPackage,
    paths: &WindowsMsiPaths,
    binary_name: &str,
) -> Result<WindowsMsiTemplateData> {
    Ok(WindowsMsiTemplateData {
        product_name: product_name(&package.name),
        package_name: package.name.clone(),
        version: msi_version(&package.version)?,
        manufacturer: package_manufacturer(package),
        description: package.description.clone().unwrap_or_default(),
        license: package.license.clone().unwrap_or_default(),
        repository: package.repository.clone().unwrap_or_default(),
        binary_name: binary_name.to_string(),
        binary_path: wix_path(&paths.binary_path),
        launcher_binary_path: wix_path(&paths.launcher_binary_path),
        msi_path: wix_path(&paths.msi_path),
        icon_path: wix_path(&paths.icon_path),
        bootstrapper_logo_path: wix_path(&paths.bootstrapper_logo_path),
        license_rtf_path: wix_path(&paths.license_rtf_path),
        upgrade_code: "ce78b6f8-ed5d-4ea4-823e-25ef51910924".to_string(),
        bundle_upgrade_code: "dc1d21bb-0a31-4c37-a8b4-045875b1e202".to_string(),
        app_component_guid: "1c3dbd45-2997-4aa7-8906-d7bf8e169cba".to_string(),
        launcher_component_guid: "89344c45-7d12-409b-b44e-eeb10ad70212".to_string(),
        path_component_guid: "7dff73dd-d542-4793-afb5-f93d1e2d921f".to_string(),
        shortcut_component_guid: "3da8345e-d097-4875-a50a-2f5132209088".to_string(),
    })
}

fn run_command(command: &mut Command, label: &str) -> Result<()> {
    let status = command.status()?;
    if !status.success() {
        return Err(format!("{label} failed with status {status}").into());
    }
    Ok(())
}

fn render_svg_icon_to_ico(svg_path: &Path, ico_path: &Path) -> Result<()> {
    let tree = parse_svg(svg_path)?;
    let original_size = tree.size();
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);

    for &size in WINDOWS_MSI_ICON_SIZES {
        let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size)
            .ok_or_else(|| format!("failed to allocate {size}x{size} icon pixmap"))?;
        let sx = size as f32 / original_size.width();
        let sy = size as f32 / original_size.height();
        let transform = resvg::tiny_skia::Transform::from_scale(sx, sy);
        resvg::render(&tree, transform, &mut pixmap.as_mut());
        let image = ico::IconImage::from_rgba_data(size, size, pixmap.take_demultiplied());
        icon_dir.add_entry(ico::IconDirEntry::encode(&image)?);
    }

    let file = fs::File::create(ico_path)?;
    icon_dir.write(BufWriter::new(file))?;
    Ok(())
}

fn render_svg_to_png(svg_path: &Path, png_path: &Path, width: u32, height: u32) -> Result<()> {
    let tree = parse_svg(svg_path)?;
    let original_size = tree.size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| format!("failed to allocate {width}x{height} logo pixmap"))?;
    let sx = width as f32 / original_size.width();
    let sy = height as f32 / original_size.height();
    let transform = resvg::tiny_skia::Transform::from_scale(sx, sy);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let file = fs::File::create(png_path)?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&pixmap.take_demultiplied())?;
    Ok(())
}

fn parse_svg(svg_path: &Path) -> Result<resvg::usvg::Tree> {
    let svg_data = fs::read(svg_path)?;
    let options = resvg::usvg::Options {
        resources_dir: svg_path.parent().map(Path::to_path_buf),
        ..resvg::usvg::Options::default()
    };
    Ok(resvg::usvg::Tree::from_data(&svg_data, &options)?)
}

fn render_text_file_to_rtf(text_path: &Path, rtf_path: &Path) -> Result<()> {
    let text = fs::read_to_string(text_path)?;
    let mut rtf = String::from(r"{\rtf1\ansi\deff0{\fonttbl{\f0 Consolas;}}\fs18 ");
    for line in text.lines() {
        rtf.push_str(&escape_rtf(line));
        rtf.push_str(r"\par ");
    }
    rtf.push('}');
    fs::write(rtf_path, rtf)?;
    Ok(())
}

fn escape_rtf(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\\' => escaped.push_str(r"\\"),
            '{' => escaped.push_str(r"\{"),
            '}' => escaped.push_str(r"\}"),
            '\t' => escaped.push_str(r"\tab "),
            '\u{00}'..='\u{7f}' => escaped.push(ch),
            _ => escaped.push_str(&format!(r"\u{}?", ch as i32)),
        }
    }
    escaped
}

fn render_windows_msi_template(
    template_path: &Path,
    output_path: &Path,
    data: &WindowsMsiTemplateData,
) -> Result<()> {
    let mut text = fs::read_to_string(template_path)?;
    let replacements = [
        ("{{product_name}}", data.product_name.as_str()),
        ("{{package_name}}", data.package_name.as_str()),
        ("{{version}}", data.version.as_str()),
        ("{{manufacturer}}", data.manufacturer.as_str()),
        ("{{description}}", data.description.as_str()),
        ("{{license}}", data.license.as_str()),
        ("{{repository}}", data.repository.as_str()),
        ("{{binary_name}}", data.binary_name.as_str()),
        ("{{binary_path}}", data.binary_path.as_str()),
        (
            "{{launcher_binary_path}}",
            data.launcher_binary_path.as_str(),
        ),
        ("{{msi_path}}", data.msi_path.as_str()),
        ("{{icon_path}}", data.icon_path.as_str()),
        (
            "{{bootstrapper_logo_path}}",
            data.bootstrapper_logo_path.as_str(),
        ),
        ("{{license_rtf_path}}", data.license_rtf_path.as_str()),
        ("{{upgrade_code}}", data.upgrade_code.as_str()),
        ("{{bundle_upgrade_code}}", data.bundle_upgrade_code.as_str()),
        ("{{app_component_guid}}", data.app_component_guid.as_str()),
        (
            "{{launcher_component_guid}}",
            data.launcher_component_guid.as_str(),
        ),
        ("{{path_component_guid}}", data.path_component_guid.as_str()),
        (
            "{{shortcut_component_guid}}",
            data.shortcut_component_guid.as_str(),
        ),
    ];

    for (placeholder, value) in replacements {
        text = text.replace(placeholder, &escape_xml(value));
    }

    let mut file = fs::File::create(output_path)?;
    file.write_all(text.as_bytes())?;
    Ok(())
}

fn msi_version(version: &str) -> Result<String> {
    let core = version
        .split_once('-')
        .map(|(core, _)| core)
        .unwrap_or(version);
    let parts = core.split('.').collect::<Vec<_>>();
    if parts.len() != 3 || parts.iter().any(|part| part.is_empty()) {
        return Err(format!(
            "package version `{version}` must be three numeric components for MSI packaging"
        )
        .into());
    }
    for part in &parts {
        part.parse::<u16>().map_err(|_| {
            format!("package version `{version}` contains non-numeric MSI component `{part}`")
        })?;
    }
    Ok(parts.join("."))
}

fn product_name(package_name: &str) -> String {
    package_name
        .split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn package_manufacturer(package: &RootPackage) -> String {
    package
        .authors
        .first()
        .map(|author| {
            author
                .split('<')
                .next()
                .unwrap_or(author)
                .trim()
                .to_string()
        })
        .filter(|author| !author.is_empty())
        .unwrap_or_else(|| product_name(&package.name))
}

fn wix_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn escape_xml(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn archive_tar_zst(stage_root: &Path, archive_path: &Path) -> Result<()> {
    if let Some(parent) = archive_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = fs::File::create(archive_path)?;
    let mut encoder = zstd::stream::write::Encoder::new(file, 19)?;
    encoder.multithread(zstd_worker_count())?;
    let mut builder = tar::Builder::new(encoder);
    builder.append_dir_all(".", stage_root)?;
    let encoder = builder.into_inner()?;
    encoder.finish()?;
    Ok(())
}

fn tar_zst_contains_entry(archive_path: &Path, expected_entry: &str) -> Result<bool> {
    let file = fs::File::open(archive_path)?;
    let decoder = zstd::stream::read::Decoder::new(file)?;
    let mut archive = tar::Archive::new(decoder);
    for entry in archive.entries()? {
        let entry = entry?;
        if archive_entry_path(&entry.path()?) == expected_entry {
            return Ok(true);
        }
    }
    Ok(false)
}

fn archive_entry_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn zstd_worker_count() -> u32 {
    std::thread::available_parallelism()
        .map(|count| count.get().clamp(1, 8) as u32)
        .unwrap_or(1)
}

fn read_root_manifest() -> Result<RootManifest> {
    let text = fs::read_to_string(repo_root().join("Cargo.toml"))?;
    Ok(toml::from_str(&text)?)
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask manifest has parent")
        .to_path_buf()
}

fn rustc_host_target() -> Result<String> {
    let output = Command::new("rustc").arg("-vV").output()?;
    if !output.status.success() {
        return Err("rustc -vV failed".into());
    }
    let stdout = String::from_utf8(output.stdout)?;
    stdout
        .lines()
        .find_map(|line| line.strip_prefix("host: ").map(str::to_string))
        .ok_or_else(|| "rustc -vV did not report host target".into())
}

fn binary_name(package_name: &str) -> String {
    if cfg!(windows) {
        format!("{package_name}.exe")
    } else {
        package_name.to_string()
    }
}

fn ensure_safe_relative_path(label: &str, path: &Path) -> Result<()> {
    if path.is_absolute() {
        return Err(format!("{label} must be relative, got {}", path.display()).into());
    }
    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            _ => {
                return Err(
                    format!("{label} contains unsafe component: {}", path.display()).into(),
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::escape_rtf;

    const BOOTSTRAPPER_TEMPLATE: &str =
        include_str!("../../packaging/windows/daat-locus-bootstrapper.wxs");

    #[test]
    fn bootstrapper_uses_real_standard_ba_theme() {
        assert!(BOOTSTRAPPER_TEMPLATE.contains("WixStandardBootstrapperApplication"));
        assert!(BOOTSTRAPPER_TEMPLATE.contains("Theme=\"rtfLargeLicense\""));
        assert!(BOOTSTRAPPER_TEMPLATE.contains("LicenseFile=\"{{license_rtf_path}}\""));
        assert!(BOOTSTRAPPER_TEMPLATE.contains("LogoFile=\"{{bootstrapper_logo_path}}\""));
        assert!(!BOOTSTRAPPER_TEMPLATE.contains("LicenseUrl="));
        assert!(!BOOTSTRAPPER_TEMPLATE.contains("Theme=\"none\""));
    }

    #[test]
    fn bootstrapper_does_not_show_nested_msi_ui() {
        assert!(!BOOTSTRAPPER_TEMPLATE.contains("DisplayInternalUICondition"));
    }

    #[test]
    fn bootstrapper_bundle_name_is_product_name() {
        assert!(BOOTSTRAPPER_TEMPLATE.contains("Name=\"{{product_name}}\""));
        assert!(!BOOTSTRAPPER_TEMPLATE.contains("Name=\"{{product_name}} Setup\""));
    }

    #[test]
    fn rtf_escaping_preserves_rtf_syntax() {
        assert_eq!(escape_rtf(r"a\b{c}"), r"a\\b\{c\}");
        assert_eq!(escape_rtf("x\ty"), r"x\tab y");
    }
}
