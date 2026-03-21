use std::fmt;

use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Court {
    Stj,
    Sta,
    Conflitos,
    RelPorto,
    RelLisboa,
    RelCoimbra,
    RelGuimaraes,
    RelEvora,
    TcaSul,
    TcaNorte,
}

impl Court {
    pub const fn db(&self) -> &str {
        match self {
            Self::Stj => "jstj.nsf",
            Self::Sta => "jsta.nsf",
            Self::Conflitos => "jcon.nsf",
            Self::RelPorto => "jtrp.nsf",
            Self::RelLisboa => "jtrl.nsf",
            Self::RelCoimbra => "jtrc.nsf",
            Self::RelGuimaraes => "jtrg.nsf",
            Self::RelEvora => "jtre.nsf",
            Self::TcaSul => "jtca.nsf",
            Self::TcaNorte => "jtcn.nsf",
        }
    }

    pub const fn view_unid(&self) -> &str {
        match self {
            Self::Stj => "954f0ce6ad9dd8b980256b5f003fa814",
            Self::Sta | Self::Conflitos => "35fbbbf22e1bb1e680256f8e003ea931",
            Self::RelPorto => "56a6e7121657f91e80257cda00381fdf",
            Self::RelLisboa => "33182fc732316039802565fa00497eec",
            Self::RelCoimbra => "8fe0e606d8f56b22802576c0005637dc",
            Self::RelGuimaraes => "86c25a698e4e7cb7802579ec004d3832",
            Self::RelEvora => "134973db04f39bf2802579bf005f080b",
            Self::TcaSul => "170589492546a7fb802575c3004c6d7d",
            Self::TcaNorte => "89d1c0288c2dd49c802575c8003279c7",
        }
    }

    pub const fn alias(&self) -> &str {
        match self {
            Self::Stj => "stj",
            Self::Sta => "sta",
            Self::Conflitos => "conflitos",
            Self::RelPorto => "rel-porto",
            Self::RelLisboa => "rel-lisboa",
            Self::RelCoimbra => "rel-coimbra",
            Self::RelGuimaraes => "rel-guimaraes",
            Self::RelEvora => "rel-evora",
            Self::TcaSul => "tca-sul",
            Self::TcaNorte => "tca-norte",
        }
    }

    pub fn from_alias(alias: &str) -> Option<Self> {
        let lower = alias.to_lowercase();
        match lower.as_str() {
            "stj" => Some(Self::Stj),
            "sta" => Some(Self::Sta),
            "conflitos" => Some(Self::Conflitos),
            "rel-porto" => Some(Self::RelPorto),
            "rel-lisboa" => Some(Self::RelLisboa),
            "rel-coimbra" => Some(Self::RelCoimbra),
            "rel-guimaraes" => Some(Self::RelGuimaraes),
            "rel-evora" => Some(Self::RelEvora),
            "tca-sul" => Some(Self::TcaSul),
            "tca-norte" => Some(Self::TcaNorte),
            _ => None,
        }
    }

    pub const fn display_name(&self) -> &str {
        match self {
            Self::Stj => "Supremo Tribunal de Justiça",
            Self::Sta => "Supremo Tribunal Administrativo",
            Self::Conflitos => "Tribunal de Conflitos",
            Self::RelPorto => "Tribunal da Relação do Porto",
            Self::RelLisboa => "Tribunal da Relação de Lisboa",
            Self::RelCoimbra => "Tribunal da Relação de Coimbra",
            Self::RelGuimaraes => "Tribunal da Relação de Guimarães",
            Self::RelEvora => "Tribunal da Relação de Évora",
            Self::TcaSul => "Tribunal Central Administrativo Sul",
            Self::TcaNorte => "Tribunal Central Administrativo Norte",
        }
    }

    pub const fn all() -> &'static [Self] {
        &[
            Self::Stj,
            Self::Sta,
            Self::Conflitos,
            Self::RelPorto,
            Self::RelLisboa,
            Self::RelCoimbra,
            Self::RelGuimaraes,
            Self::RelEvora,
            Self::TcaSul,
            Self::TcaNorte,
        ]
    }

    pub fn search_url(&self, query: &str, count: u32, start: u32, sort_by_date: bool) -> String {
        let encoded_query = utf8_percent_encode(query, NON_ALPHANUMERIC).to_string();
        let mut url = format!(
            "https://www.dgsi.pt/{db}/{view}?SearchView&Query={encoded_query}&SearchMax=0&Count={count}&Start={start}",
            db = self.db(),
            view = self.view_unid(),
        );
        if sort_by_date {
            url.push_str("&SearchOrder=1");
        }
        url
    }
}

impl fmt::Display for Court {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}
