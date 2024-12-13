use crate::models::{MitreMitigation, MitreMitigations};
use crate::state::{MITRE_MITIGATIONS, MITRE_MITIGATIONS_JSON};
use serde::de::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub enum Keywords {
    Website,
    Web,
    Database,
    Backend,
    Credentials,
    Security,
    Network,
    Authentication,
    Permissions,
    Encryption,
}

impl Keywords {
    pub fn mitigation_ids(&self) -> Vec<String> {
        match self {
            Keywords::Website => vec!["M1036".into(), "M1048".into(), "M1057".into()],
            Keywords::Web => vec!["M1036".into(), "M1047".into(), "M1050".into()],
            Keywords::Database => vec!["M1049".into(), "M1028".into(), "M1032".into()],
            Keywords::Backend => vec!["M1015".into(), "M1042".into(), "M1025".into()],
            Keywords::Credentials => vec!["M1043".into(), "M1034".into(), "M1033".into()],
            Keywords::Security => vec!["M1050".into(), "M1041".into(), "M1038".into()],
            Keywords::Network => vec!["M1037".into(), "M1035".into(), "M1030".into()],
            Keywords::Authentication => vec!["M1032".into(), "M1018".into(), "M1026".into()],
            Keywords::Permissions => vec!["M1032".into(), "M1018".into(), "M1026".into()],
            Keywords::Encryption => vec!["M1041".into(), "M1051".into(), "M1029".into()],
        }
    }
}

pub trait ToMitigations {
    fn to_mitigations(self) -> MitreMitigations;
}

impl ToMitigations for Vec<Keywords> {
    fn to_mitigations(self) -> MitreMitigations {
        if self.is_empty() {
            return MitreMitigations(vec![]);
        }
        let mut ids = self
            .iter()
            .map(Keywords::mitigation_ids)
            .collect::<Vec<Vec<_>>>();

        ids.sort();
        ids.dedup();

        let ids = ids.iter().flatten().collect::<Vec<&String>>();

        let mitigations = MITRE_MITIGATIONS
            .iter()
            .filter(|mitre_mitigation| {
                let id = &mitre_mitigation.id;
                ids.contains(&id)
            })
            .cloned()
            .collect();

        MitreMitigations(mitigations)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    Structured {
        prompt: String,
        #[serde(default)]
        keywords: Vec<Keywords>,
    },
    Chat {
        prompt: String,
    },
}
