//! Die Bibliothek: benannte Regelwerke und Tarife, aus denen man für einen Abend auswählt.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::model::{BockAtRamsch, BockMode, BockStacking, GameKind};
use crate::scoring::Tariff;

/// Welche Spielarten in einer Runde gespielt werden.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleSet {
    pub name: String,
    pub kinds: Vec<GameKind>,
    /// Ob und wie lange nach einem Auslöser (zum Beispiel Schwarz) Bockspiele folgen. Alte Dateien
    /// ohne diese Angabe spielen ohne Bock.
    #[serde(default)]
    pub bock: BockMode,
    /// Ob sich mehrere Bockrunden hintereinander reihen oder überlagern. Alte Dateien: hintereinander.
    #[serde(default)]
    pub stacking: BockStacking,
    /// Was beim Ramsch mit einer Bockrunde passiert. Alte Dateien: Der Ramsch zählt als Bockspiel.
    #[serde(default)]
    pub bock_at_ramsch: BockAtRamsch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamedTariff {
    pub name: String,
    pub tariff: Tariff,
}

/// Alle gespeicherten Regelwerke, Tarife und Spieler, Regelwerke und Tarife je mit einem Standard.
///
/// Es gibt immer mindestens ein Regelwerk und einen Tarif, und Namen sind eindeutig, ohne auf
/// Groß- und Kleinschreibung zu achten. Die Einträge ändern nie Abende, die schon laufen: Ein Tisch
/// kopiert Spielarten und Tarif beim Anlegen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Library {
    pub version: u32,
    rule_sets: Vec<RuleSet>,
    tariffs: Vec<NamedTariff>,
    default_rule_set: String,
    default_tariff: String,
    /// Die Spieler, die man bei der Einrichtung eines Abends auswählen kann. Alte Dateien ohne
    /// Spieler lesen sich als leere Liste.
    #[serde(default)]
    players: Vec<String>,
}

/// Spielarten in der üblichen Reihenfolge, ohne Doppelte.
fn canonical(kinds: &[GameKind]) -> Vec<GameKind> {
    GameKind::ALL
        .into_iter()
        .filter(|k| kinds.contains(k))
        .collect()
}

fn clean_name(name: &str) -> Result<String> {
    let name = name.trim();
    if name.is_empty() {
        Err(Error::NameEmpty)
    } else {
        Ok(name.to_owned())
    }
}

fn same(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// Was beim Zusammenführen zweier Bibliotheken neu dazukam.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MergeReport {
    pub players: usize,
    pub rule_sets: usize,
    pub tariffs: usize,
    /// Einträge, die es unter diesem Namen schon gab und die deshalb unverändert blieben.
    pub kept: usize,
}

impl Library {
    /// Nimmt Spieler, Regelwerke und Tarife aus `other` dazu, soweit gewünscht. Was es unter dem
    /// Namen schon gibt, bleibt, wie es ist, und die eigenen Standards bleiben auch.
    pub fn merge_from(
        &mut self,
        other: &Library,
        players: bool,
        rule_sets: bool,
        tariffs: bool,
    ) -> MergeReport {
        let mut report = MergeReport::default();
        if players {
            for name in &other.players {
                if self.players.iter().any(|p| same(p, name)) {
                    report.kept += 1;
                } else if self.add_player(name).is_ok() {
                    report.players += 1;
                }
            }
        }
        if rule_sets {
            for rule_set in &other.rule_sets {
                if self.rule_sets.iter().any(|r| same(&r.name, &rule_set.name)) {
                    report.kept += 1;
                } else {
                    self.rule_sets.push(rule_set.clone());
                    report.rule_sets += 1;
                }
            }
        }
        if tariffs {
            for tariff in &other.tariffs {
                if self.tariffs.iter().any(|t| same(&t.name, &tariff.name)) {
                    report.kept += 1;
                } else {
                    self.tariffs.push(tariff.clone());
                    report.tariffs += 1;
                }
            }
        }
        report
    }

    /// Wie viele Spieler, Regelwerke und Tarife die Bibliothek hat.
    pub fn counts(&self) -> (usize, usize, usize) {
        (self.players.len(), self.rule_sets.len(), self.tariffs.len())
    }
}

impl Library {
    pub const CURRENT_VERSION: u32 = 1;

    /// Die Bibliothek beim ersten Start: ein Regelwerk mit allen Spielarten, eines mit den
    /// Spielarten, die in unserer Runde üblich sind (Standard), und der Tarif "20/50" (Standard).
    pub fn with_defaults() -> Self {
        Library {
            version: Self::CURRENT_VERSION,
            rule_sets: vec![
                RuleSet {
                    name: "Alle Spielarten".into(),
                    kinds: GameKind::ALL.to_vec(),
                    bock: BockMode::Off,
                    stacking: BockStacking::Sequential,
                    bock_at_ramsch: BockAtRamsch::Counts,
                },
                RuleSet {
                    name: "Unsere Runde".into(),
                    kinds: vec![
                        GameKind::Sauspiel,
                        GameKind::Wenz,
                        GameKind::Geier,
                        GameKind::Farbsolo,
                        GameKind::Ramsch,
                    ],
                    bock: BockMode::NextGame,
                    stacking: BockStacking::Sequential,
                    bock_at_ramsch: BockAtRamsch::Counts,
                },
            ],
            tariffs: vec![NamedTariff {
                name: "20/50".into(),
                tariff: Tariff::default(),
            }],
            default_rule_set: "Unsere Runde".into(),
            default_tariff: "20/50".into(),
            players: Vec::new(),
        }
    }

    pub fn rule_sets(&self) -> &[RuleSet] {
        &self.rule_sets
    }

    pub fn tariffs(&self) -> &[NamedTariff] {
        &self.tariffs
    }

    pub fn rule_set(&self, name: &str) -> Option<&RuleSet> {
        self.rule_sets.iter().find(|r| r.name == name)
    }

    pub fn tariff(&self, name: &str) -> Option<&NamedTariff> {
        self.tariffs.iter().find(|t| t.name == name)
    }

    /// Das Standard-Regelwerk. Sollte der Name einmal nicht passen, das erste.
    pub fn default_rule_set(&self) -> &RuleSet {
        self.rule_set(&self.default_rule_set)
            .unwrap_or(&self.rule_sets[0])
    }

    pub fn default_tariff(&self) -> &NamedTariff {
        self.tariff(&self.default_tariff)
            .unwrap_or(&self.tariffs[0])
    }

    pub fn is_default_rule_set(&self, name: &str) -> bool {
        self.default_rule_set().name == name
    }

    pub fn is_default_tariff(&self, name: &str) -> bool {
        self.default_tariff().name == name
    }

    fn check_name<T>(
        entries: &[T],
        name_of: impl Fn(&T) -> &str,
        name: &str,
        editing: Option<&str>,
    ) -> Result<()> {
        let taken = entries
            .iter()
            .any(|e| same(name_of(e), name) && editing.is_none_or(|own| name_of(e) != own));
        if taken {
            Err(Error::NameTaken(name.to_owned()))
        } else {
            Ok(())
        }
    }

    /// Legt ein Regelwerk an (`original` ist `None`) oder ändert ein bestehendes samt Namen.
    /// Der Bock-Modus bleibt, wie er war (bei einem neuen Regelwerk: kein Bock). Gibt den
    /// gespeicherten Namen zurück.
    pub fn save_rule_set(
        &mut self,
        original: Option<&str>,
        name: &str,
        kinds: &[GameKind],
    ) -> Result<String> {
        let bock = original
            .and_then(|old| self.rule_set(old))
            .map_or(BockMode::Off, |r| r.bock);
        self.save_rule_set_with(original, name, kinds, bock)
    }

    /// Wie [`Library::save_rule_set`], mit dem Bock-Modus des Regelwerks. Die Art, mehrere
    /// Bockrunden zu verbinden, bleibt, wie sie war.
    pub fn save_rule_set_with(
        &mut self,
        original: Option<&str>,
        name: &str,
        kinds: &[GameKind],
        bock: BockMode,
    ) -> Result<String> {
        let stacking = original
            .and_then(|old| self.rule_set(old))
            .map_or(BockStacking::Sequential, |r| r.stacking);
        self.save_rule_set_full(original, name, kinds, bock, stacking)
    }

    /// Wie [`Library::save_rule_set_with`], auch mit der Art, mehrere Bockrunden zu verbinden. Was
    /// beim Ramsch mit dem Bock passiert, bleibt, wie es war.
    pub fn save_rule_set_full(
        &mut self,
        original: Option<&str>,
        name: &str,
        kinds: &[GameKind],
        bock: BockMode,
        stacking: BockStacking,
    ) -> Result<String> {
        let at_ramsch = original
            .and_then(|old| self.rule_set(old))
            .map_or(BockAtRamsch::Counts, |r| r.bock_at_ramsch);
        self.save_rule_set_all(original, name, kinds, bock, stacking, at_ramsch)
    }

    /// Wie [`Library::save_rule_set_full`], auch mit dem Verhalten der Bockrunde beim Ramsch.
    pub fn save_rule_set_all(
        &mut self,
        original: Option<&str>,
        name: &str,
        kinds: &[GameKind],
        bock: BockMode,
        stacking: BockStacking,
        bock_at_ramsch: BockAtRamsch,
    ) -> Result<String> {
        let name = clean_name(name)?;
        let kinds = canonical(kinds);
        if kinds.is_empty() {
            return Err(Error::NoKinds);
        }
        Self::check_name(&self.rule_sets, |r| &r.name, &name, original)?;
        let rule_set = RuleSet {
            name: name.clone(),
            kinds,
            bock,
            stacking,
            bock_at_ramsch,
        };
        match original {
            None => self.rule_sets.push(rule_set),
            Some(old) => {
                let entry = self
                    .rule_sets
                    .iter_mut()
                    .find(|r| r.name == old)
                    .ok_or_else(|| Error::UnknownEntry(old.to_owned()))?;
                if self.default_rule_set == old {
                    self.default_rule_set = name.clone();
                }
                *entry = rule_set;
            }
        }
        Ok(name)
    }

    /// Löscht ein Regelwerk. War es der Standard, wird das erste übrige zum Standard.
    pub fn delete_rule_set(&mut self, name: &str) -> Result<()> {
        let index = self
            .rule_sets
            .iter()
            .position(|r| r.name == name)
            .ok_or_else(|| Error::UnknownEntry(name.to_owned()))?;
        if self.rule_sets.len() == 1 {
            return Err(Error::LastEntry);
        }
        self.rule_sets.remove(index);
        if self.default_rule_set == name {
            self.default_rule_set = self.rule_sets[0].name.clone();
        }
        Ok(())
    }

    pub fn set_default_rule_set(&mut self, name: &str) -> Result<()> {
        let rule_set = self
            .rule_set(name)
            .ok_or_else(|| Error::UnknownEntry(name.to_owned()))?;
        self.default_rule_set = rule_set.name.clone();
        Ok(())
    }

    /// Legt einen Tarif an (`original` ist `None`) oder ändert einen bestehenden samt Namen.
    pub fn save_tariff(
        &mut self,
        original: Option<&str>,
        name: &str,
        tariff: Tariff,
    ) -> Result<String> {
        let name = clean_name(name)?;
        Self::check_name(&self.tariffs, |t| &t.name, &name, original)?;
        match original {
            None => self.tariffs.push(NamedTariff {
                name: name.clone(),
                tariff,
            }),
            Some(old) => {
                let entry = self
                    .tariffs
                    .iter_mut()
                    .find(|t| t.name == old)
                    .ok_or_else(|| Error::UnknownEntry(old.to_owned()))?;
                if self.default_tariff == old {
                    self.default_tariff = name.clone();
                }
                *entry = NamedTariff {
                    name: name.clone(),
                    tariff,
                };
            }
        }
        Ok(name)
    }

    /// Löscht einen Tarif. War er der Standard, wird der erste übrige zum Standard.
    pub fn delete_tariff(&mut self, name: &str) -> Result<()> {
        let index = self
            .tariffs
            .iter()
            .position(|t| t.name == name)
            .ok_or_else(|| Error::UnknownEntry(name.to_owned()))?;
        if self.tariffs.len() == 1 {
            return Err(Error::LastEntry);
        }
        self.tariffs.remove(index);
        if self.default_tariff == name {
            self.default_tariff = self.tariffs[0].name.clone();
        }
        Ok(())
    }

    pub fn set_default_tariff(&mut self, name: &str) -> Result<()> {
        let tariff = self
            .tariff(name)
            .ok_or_else(|| Error::UnknownEntry(name.to_owned()))?;
        self.default_tariff = tariff.name.clone();
        Ok(())
    }

    /// Die gespeicherten Spielernamen, in der Reihenfolge, in der sie angelegt wurden.
    pub fn players(&self) -> &[String] {
        &self.players
    }

    /// Die Spielernamen alphabetisch (ohne Beachtung der Schreibweise), für Auswahllisten.
    pub fn players_sorted(&self) -> Vec<String> {
        let mut sorted = self.players.clone();
        sorted.sort_by_key(|p| p.to_lowercase());
        sorted
    }

    /// Speichert einen neuen Spielernamen. Namen sind eindeutig, ohne auf Groß- und
    /// Kleinschreibung zu achten. Gibt den gespeicherten Namen zurück.
    pub fn add_player(&mut self, name: &str) -> Result<String> {
        let name = clean_name(name)?;
        Self::check_name(&self.players, |p| p.as_str(), &name, None)?;
        self.players.push(name.clone());
        Ok(name)
    }

    /// Nimmt alle Namen auf, die noch fehlen, und überspringt leere und schon bekannte. Gibt die
    /// Zahl der neu aufgenommenen Namen zurück. So merkt sich die App Namen, die beim Einrichten
    /// eines Abends einmal getippt wurden.
    pub fn ensure_players(&mut self, names: &[String]) -> usize {
        let before = self.players.len();
        for name in names {
            let _ = self.add_player(name);
        }
        self.players.len() - before
    }

    /// Benennt einen Spieler um. Schon gespielte Abende behalten den alten Namen.
    pub fn rename_player(&mut self, old: &str, new: &str) -> Result<String> {
        let new = clean_name(new)?;
        Self::check_name(&self.players, |p| p.as_str(), &new, Some(old))?;
        let entry = self
            .players
            .iter_mut()
            .find(|p| p.as_str() == old)
            .ok_or_else(|| Error::UnknownEntry(old.to_owned()))?;
        *entry = new.clone();
        Ok(new)
    }

    pub fn remove_player(&mut self, name: &str) -> Result<()> {
        let index = self
            .players
            .iter()
            .position(|p| p == name)
            .ok_or_else(|| Error::UnknownEntry(name.to_owned()))?;
        self.players.remove(index);
        Ok(())
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| Error::Json(e.to_string()))
    }

    /// Liest eine gespeicherte Bibliothek. Leere Listen oder fehlende Standards werden nicht
    /// geduldet, damit die App sich auf die Zusagen der Typen verlassen kann.
    pub fn from_json(json: &str) -> Result<Self> {
        let library: Library =
            serde_json::from_str(json).map_err(|e| Error::Json(e.to_string()))?;
        if library.version != Self::CURRENT_VERSION {
            return Err(Error::UnsupportedVersion(library.version));
        }
        if library.rule_sets.is_empty()
            || library.tariffs.is_empty()
            || library.rule_sets.iter().any(|r| r.kinds.is_empty())
        {
            return Err(Error::Json("Die Bibliothek ist unvollständig".into()));
        }
        Ok(library)
    }
}
