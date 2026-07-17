use crate::content::biome::definition::BiomeDefinition;
use crate::shared::identifier::Identifier;
use bevy::prelude::Resource;
use std::collections::{HashMap, HashSet};

#[derive(Resource, Default, Clone)]
pub struct BiomeRegistry {
    biomes: Vec<BiomeDefinition>,
    identifier_to_index: HashMap<Identifier, u8>,
}

impl BiomeRegistry {
    pub fn from_definitions(definitions: Vec<BiomeDefinition>) -> Result<Self, String> {
        let mut registry = Self::default();
        registry.replace_definitions(definitions)?;
        Ok(registry)
    }

    pub fn replace_definitions(
        &mut self,
        mut definitions: Vec<BiomeDefinition>,
    ) -> Result<(), String> {
        if definitions.is_empty() {
            return Err("at least one biome definition is required".into());
        }
        if definitions.len() > u8::MAX as usize + 1 {
            return Err(format!(
                "too many biome definitions: {}, maximum is 256",
                definitions.len()
            ));
        }
        definitions.sort_by(|left, right| {
            left.generation_order
                .cmp(&right.generation_order)
                .then_with(|| left.identifier.cmp(&right.identifier))
        });

        let mut identifiers = HashSet::new();
        let mut orders = HashSet::new();
        for biome in &definitions {
            if !identifiers.insert(biome.identifier.clone()) {
                return Err(format!("duplicate biome identifier: {}", biome.identifier));
            }
            if !orders.insert(biome.generation_order) {
                return Err(format!(
                    "duplicate biome generation_order: {}",
                    biome.generation_order
                ));
            }
        }

        self.biomes = definitions;
        self.identifier_to_index.clear();
        for (index, biome) in self.biomes.iter().enumerate() {
            self.identifier_to_index
                .insert(biome.identifier.clone(), index as u8);
        }
        Ok(())
    }

    pub fn get(&self, index: u8) -> Option<&BiomeDefinition> {
        self.biomes.get(index as usize)
    }

    pub fn biomes_iter(&self) -> impl Iterator<Item = (usize, &BiomeDefinition)> {
        self.biomes.iter().enumerate()
    }

    pub fn len(&self) -> usize {
        self.biomes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.biomes.is_empty()
    }

    pub fn get_index(&self, identifier: &str) -> Option<u8> {
        let key = Identifier::parse(identifier).ok()?;
        self.identifier_to_index.get(&key).copied()
    }
}
