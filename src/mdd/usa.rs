//! United States-level aggregation of MDD species distribution data.

use std::collections::{BTreeMap, HashMap, HashSet};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::mdd::species::SpeciesData;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsaStats {
    pub total_states: u32,
    pub state_data: BTreeMap<String, UsaStateData>,
}

impl UsaStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_country_data(&mut self, mdd_data: &[SpeciesData]) {
        // Filter species that are present in the US.
        let usa_species: Vec<&SpeciesData> = mdd_data
            .iter()
            .filter(|species| species.country_distribution.contains("USA"))
            .collect();

        let mut state_records: HashMap<String, StateRecord> = HashMap::new();
        for species in usa_species {
            let state_codes = self.parse_state_data(&species.subregion_distribution);
            for state_code in state_codes {
                let record = state_records
                    .entry(state_code.clone())
                    .or_insert_with(|| StateRecord::new(&state_code));
                record.update(species);
            }
        }

        // Convert state records to UsaStateData.
        self.state_data = state_records
            .into_iter()
            .map(|(state_code, record)| (state_code, record.to_usa_state_data()))
            .collect();
        self.total_states = self.state_data.len() as u32;
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    // MDD codes state data as:
    // USA(AL, AK, AZ, AR, CA, CO, CT, DE, DC, FL, GA, ID, IL, IN, IA, KS, KY, LA, ME, MD,
    // MA, MI, MN, MS, MO, MT, NE, NV, NH, NJ, NM, NY, NC, ND, OH, OK, OR, PA, RI, SC, SD,
    // TN, TX, UT, VT, VA, WA, WV, WI, WY)
    // It returns a vector of state codes that are present in the subregion_dist.
    fn parse_state_data(&self, subregion_dist: &str) -> Vec<String> {
        let caps = STATE_DIST_RE.captures(subregion_dist);
        if let Some(caps) = caps {
            let states_str = caps.get(1).unwrap().as_str();
            states_str.split(',').map(|s| s.to_string()).collect()
        } else {
            Vec::new()
        }
    }
}

// Lazy static to avoid recompiling regex.
lazy_static::lazy_static! {
    static ref STATE_DIST_RE: Regex = Regex::new(r"USA\((.*?)\)").expect("Failed to compile state distribution regex");
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsaStateData {
    pub state_code: String,
    pub total_order: u32,
    pub total_family: u32,
    pub total_genus: u32,
    pub total_living_species: u32,
    pub total_extinct_species: u32,
    pub species_list: Vec<String>,
}

impl UsaStateData {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Default)]
struct StateRecord {
    state_code: String,
    orders: HashSet<String>,
    families: HashSet<String>,
    genera: HashSet<String>,
    living_species: HashSet<String>,
    extinct_species: HashSet<String>,
}

impl StateRecord {
    pub fn new(state_code: &str) -> Self {
        Self {
            state_code: state_code.to_string(),
            orders: HashSet::new(),
            families: HashSet::new(),
            genera: HashSet::new(),
            living_species: HashSet::new(),
            extinct_species: HashSet::new(),
        }
    }

    fn update(&mut self, species: &SpeciesData) {
        self.orders.insert(species.taxon_order.clone());
        self.families.insert(species.family.clone());
        self.genera.insert(species.genus.clone());
        if species.extinct == 1 {
            self.extinct_species.insert(species.id.to_string());
        } else {
            self.living_species.insert(species.id.to_string());
        }
    }

    fn to_usa_state_data(&self) -> UsaStateData {
        UsaStateData {
            state_code: self.state_code.clone(),
            total_order: self.orders.len() as u32,
            total_family: self.families.len() as u32,
            total_genus: self.genera.len() as u32,
            total_living_species: self.living_species.len() as u32,
            total_extinct_species: self.extinct_species.len() as u32,
            species_list: self.living_species.iter().cloned().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_state_data() {
        let usa_stats = UsaStats::new();
        let state_data = usa_stats.parse_state_data("USA(AL,AK,AZ,AR,CA,CO,CT,DE,DC,FL,GA,ID,IL,IN,IA,KS,KY,LA,ME,MD,MA,MI,MN,MS,MO,MT,NE,NV,NH,NJ,NM,NY,NC,ND,OH,OK,OR,PA,RI,SC,SD,TN,TX,UT,VT,VA,WA,WV,WI,WY)");
        assert_eq!(state_data.len(), 50);
    }
}
