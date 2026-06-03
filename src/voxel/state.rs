use std::collections::HashMap;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 定义方块属性状态
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct BlockStateProperty{
    /// 属性名
    pub name: String,
    /// 允许的值
    pub values: Vec<String>,
    /// 默认值索引
    pub default_index: usize,
}

/// 运行时方块状态
/// 仅对有状态的方块分配
pub struct BlockStateData {
    /// 方块状态索引
    pub state_index: u16,
}

/// 方块完整状态
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlockStateDefinition{
    /// 所有状态属性
    pub properties: Vec<BlockStateProperty>,
}

impl BlockStateDefinition{
    /// 空状态
    pub fn empty()-> Self{
        Self {
            properties:Vec::new(),
        }
    }

    /// 创建单属性状态
     pub fn single(name: &str, values: &[&str], default: usize) -> Self {
        Self {
            properties: vec![BlockStateProperty {
                name: name.to_string(),
                values: values.iter().map(|v| v.to_string()).collect(),
                default_index: default,
            }],
        }
    }

    /// 计算所有可能的状态组合数
    pub fn total_state_count(&self) -> usize {
        if self.properties.is_empty() { return 1; }
        self.properties.iter().map(|p| p.values.len()).product()
    }

    /// 从状态索引获取各属性值
    pub fn get_state_values(&self, state_index: usize) -> HashMap<String, String> {
        let mut result = HashMap::new();
        let mut remaining = state_index;

        // 从最后一个属性开始分解
        for prop in self.properties.iter().rev() {
            let value_idx = remaining % prop.values.len();
            remaining /= prop.values.len();
            result.insert(prop.name.clone(), prop.values[value_idx].clone());
        }
        result
    }

    /// 从属性值映射计算状态索引
    pub fn get_state_index(&self, values: &HashMap<String, String>) -> usize {
        let mut index = 0;
        let mut multiplier = 1;

        for prop in &self.properties {
            let value_idx = values.get(&prop.name)
                .and_then(|v| prop.values.iter().position(|pv| pv == v))
                .unwrap_or(prop.default_index);
            index += value_idx * multiplier;
            multiplier *= prop.values.len();
        }
        index
    }

    /// 获取默认状态索引
    pub fn default_state_index(&self) -> usize {
        self.get_state_index(&self.properties.iter().map(|p| {
            (p.name.clone(), p.values[p.default_index].clone())
        }).collect())
    }
}