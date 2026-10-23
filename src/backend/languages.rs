//! Language kinds and families: which backend languages are data or documentation, and which variants fold into a family.
use super::*;

/// Languages counted as data or configuration rather than source.
///
/// Tokei recognizes mostly programming languages, so source is the default
/// kind; these lists name the reviewed exceptions for Tokei 15.0.0.
pub const DATA_LANGUAGES: &[LanguageType] = &[
    Bean,
    DotNetResource,
    Edn,
    FEN,
    ForgeConfig,
    Hex,
    Hledger,
    Ini,
    IntelHex,
    Json,
    MsBuild,
    NuGetConfig,
    Po,
    RON,
    Svg,
    Toml,
    UnrealPlugin,
    UnrealProject,
    UrWebProject,
    VerilogArgsFile,
    VisualStudioProject,
    VisualStudioSolution,
    XcodeConfig,
    Xml,
    Yaml,
];

pub const DOCUMENTATION_LANGUAGES: &[LanguageType] = &[
    AsciiDoc,
    Djot,
    Markdown,
    Mdx,
    Org,
    ReStructuredText,
    Tex,
    Text,
    Typst,
    UnrealDeveloperMarkdown,
];

/// Variants that human views show within their family's row: the same
/// language with embedded markup, its header and module units, or BASH,
/// which Tokei assigns by `.bash` or a bash shebang while every `.sh` is
/// Shell, so one project's scripts split by naming rather than dialect. C
/// Header stays apart because Tokei assigns every `.h` to it, including C++
/// and Objective-C headers; Fortran has no family name Tokei knows.
pub(super) const LANGUAGE_FAMILIES: &[(LanguageType, LanguageType)] = &[
    (Tsx, TypeScript),
    (GlimmerTs, TypeScript),
    (Jsx, JavaScript),
    (GlimmerJs, JavaScript),
    (CppHeader, Cpp),
    (CppModule, Cpp),
    (UnrealShaderHeader, UnrealShader),
    (Bash, Sh),
];

/// The family a variant's display name joins, such as TypeScript for TSX;
/// `None` for every other name.
pub fn language_family(name: &str) -> Option<&'static str> {
    LANGUAGE_FAMILIES
        .iter()
        .find(|(variant, _)| variant.name() == name)
        .map(|(_, family)| family.name())
}

/// The display name of the source language `name` denotes, in any case.
/// Data and documentation names are refused: tables count only source, so
/// selecting one would leave them empty.
pub fn source_language(name: &str) -> Result<&'static str, String> {
    let language = LanguageType::from_name(name)
        .or_else(|| {
            LanguageType::list()
                .iter()
                .map(|(language, _)| *language)
                .find(|language| language.name().eq_ignore_ascii_case(name))
        })
        .ok_or_else(|| format!("Unknown language '{name}'. Use a name from the Language table."))?;
    match language_kind(language.name()) {
        Kind::Source => Ok(language.name()),
        kind => Err(format!("{} is {kind}, not source", language.name())),
    }
}

/// Content kind of a backend language name; names unknown to this Tokei
/// version, for example from saved payloads, stay visible as `Unknown`.
pub fn language_kind(name: &str) -> Kind {
    match LanguageType::from_name(name) {
        None => Kind::Unknown,
        Some(language) if DATA_LANGUAGES.contains(&language) => Kind::Data,
        Some(language) if DOCUMENTATION_LANGUAGES.contains(&language) => Kind::Documentation,
        Some(_) => Kind::Source,
    }
}
