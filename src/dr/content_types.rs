/// Content types available in the Diário da República search API.
///
/// Each variant maps to a specific `tipoConteudo` value that the `OutSystems`
/// backend expects. The `PascalCase` spelling is critical — camelCase returns
/// wrong results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrContentType {
    /// Individual acts, 1st series (Portarias, Decretos-Lei, etc.)
    AtosSerie1,
    /// Individual acts, 2nd series (Despachos, Avisos, etc.)
    AtosSerie2,
    /// Whole DR issues (PDFs)
    DiarioRepublica,
    /// Judicial decisions published in DR
    Jurisprudencia,
}

static ALL_CONTENT_TYPES: [DrContentType; 4] = [
    DrContentType::AtosSerie1,
    DrContentType::AtosSerie2,
    DrContentType::DiarioRepublica,
    DrContentType::Jurisprudencia,
];

impl DrContentType {
    /// Returns the `PascalCase` value expected by the DR API in `tipoConteudo`
    /// fields. Using any other casing (e.g. `"atosSerie1"`) returns wrong
    /// results.
    pub const fn tipo_conteudo(&self) -> &str {
        match self {
            Self::AtosSerie1 => "AtosSerie1",
            Self::AtosSerie2 => "AtosSerie2",
            Self::DiarioRepublica => "DiarioRepublica",
            Self::Jurisprudencia => "Jurisprudencia",
        }
    }

    /// Returns the key used in `PesquisaAvancadaBools` JSON.
    pub const fn bools_key(&self) -> &str {
        match self {
            Self::AtosSerie1 => "Atos1",
            Self::AtosSerie2 => "Atos2",
            Self::DiarioRepublica => "DiarioRepublica",
            Self::Jurisprudencia => "Jurisprudencia",
        }
    }

    /// Maps CLI-friendly aliases to content types.
    pub fn from_alias(alias: &str) -> Option<Self> {
        match alias.to_lowercase().as_str() {
            "atos-1" | "atos1" | "serie1" | "s1" => Some(Self::AtosSerie1),
            "atos-2" | "atos2" | "serie2" | "s2" => Some(Self::AtosSerie2),
            "dr" | "diario" | "diario-republica" => Some(Self::DiarioRepublica),
            "decisoes" | "jurisprudencia" | "juris" => Some(Self::Jurisprudencia),
            _ => None,
        }
    }

    /// Human-readable display name.
    pub const fn display_name(&self) -> &str {
        match self {
            Self::AtosSerie1 => "Atos da 1.ª Série",
            Self::AtosSerie2 => "Atos da 2.ª Série",
            Self::DiarioRepublica => "Diário da República",
            Self::Jurisprudencia => "Decisões Judiciais",
        }
    }

    /// All available content types.
    pub fn all() -> &'static [Self] {
        &ALL_CONTENT_TYPES
    }
}

/// Known act type aliases and their exact API values.
static ACT_TYPE_ALIASES: &[(&str, &str)] = &[
    ("portaria", "Portaria"),
    ("decreto-lei", "Decreto-Lei"),
    ("lei", "Lei"),
    ("resolucao", "Resolução do Conselho de Ministros"),
    ("despacho", "Despacho"),
    ("decreto", "Decreto"),
    ("aviso", "Aviso"),
    ("retificacao", "Declaração de Retificação"),
    ("decreto-regulamentar", "Decreto Regulamentar"),
    ("lei-organica", "Lei Orgânica"),
];

/// Resolve a CLI-friendly alias to the exact act type string the DR API
/// expects. Returns `None` if the alias is not recognised.
pub fn resolve_act_type(alias: &str) -> Option<String> {
    let lower = alias.to_lowercase();
    ACT_TYPE_ALIASES.iter().find(|(key, _)| *key == lower).map(|(_, val)| (*val).to_owned())
}

/// Returns all known `(alias, display_name)` pairs for act types.
pub fn act_type_aliases() -> &'static [(&'static str, &'static str)] {
    ACT_TYPE_ALIASES
}
