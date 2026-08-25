//! 本地化运行时数据：语言标识、注册表与翻译查询回退链。

use std::collections::BTreeMap;

use bevy::prelude::Resource;

/// 缺失条目时的回退语言；回退表也没有该键时返回键本身，便于定位漏翻。
pub const FALLBACK_LANGUAGE: &str = "zh-CN";

/// BCP 47 风格的语言标识，如 `zh-CN`、`en-US`。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LanguageId(pub String);

impl LanguageId {
    /// 用任意字符串构造语言标识；调用方负责保证取值来自已注册语言。
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// 返回标识原文。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 语言的展示元数据，由语言文件自描述。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageInfo {
    /// 语言标识。
    pub id: LanguageId,
    /// 用该语言自身书写的名称，例如「简体中文」「English」。
    pub native_name: String,
}

/// 本地化查询资源：持有全部已加载语言的翻译表与当前激活语言。
///
/// 查询回退链固定为「激活语言 -> 回退语言 -> 键本身」；切换语言只改变
/// 激活项，不重新读取文件。资源在 LocalizationPlugin 构建期注入
/// （早于全部调度阶段），保证任何界面构建都能取到译文。
#[derive(Resource, Debug, Clone)]
pub struct Localization {
    /// 已加载语言，按标识排序保证设置界面的遍历顺序稳定。
    languages: Vec<LanguageInfo>,
    /// 各语言的翻译表：点号分隔的扁平键到译文的映射。
    tables: BTreeMap<LanguageId, BTreeMap<String, String>>,
    active: LanguageId,
}

impl Localization {
    /// 用加载结果组装查询资源；激活语言初始为回退语言，目录为空时为空资源。
    pub fn new(
        languages: Vec<LanguageInfo>,
        tables: BTreeMap<LanguageId, BTreeMap<String, String>>,
    ) -> Self {
        let fallback = LanguageId::new(FALLBACK_LANGUAGE);
        let active = if tables.contains_key(&fallback) {
            fallback
        } else {
            // 目录为空或不含回退语言时退化为首个可用语言，保持资源可用。
            languages
                .first()
                .map(|info| info.id.clone())
                .unwrap_or(fallback)
        };
        Self {
            languages,
            tables,
            active,
        }
    }

    /// 已加载语言列表，按标识排序。
    pub fn languages(&self) -> &[LanguageInfo] {
        &self.languages
    }

    /// 当前激活语言。
    pub fn active(&self) -> &LanguageId {
        &self.active
    }

    /// 切换激活语言；未注册的语言保持原状并返回 `false`。
    pub fn set_active(&mut self, id: &LanguageId) -> bool {
        if self.tables.contains_key(id) {
            self.active = id.clone();
            true
        } else {
            false
        }
    }

    /// 按回退链查询译文；两级都缺失时返回键本身，便于在界面上定位漏翻条目。
    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        self.get_in(&self.active, key)
    }

    /// 查询指定语言的译文，回退链与 [`Localization::get`] 一致。
    ///
    /// 返回值可能直接引用 `key`（两级表都缺失时），因此返回生命周期
    /// 同时绑定 `self` 与 `key`。
    pub fn get_in<'a>(&'a self, id: &LanguageId, key: &'a str) -> &'a str {
        if let Some(text) = self.tables.get(id).and_then(|table| table.get(key)) {
            return text.as_str();
        }
        let fallback = LanguageId::new(FALLBACK_LANGUAGE);
        if let Some(text) = self.tables.get(&fallback).and_then(|table| table.get(key)) {
            return text.as_str();
        }
        key
    }

    /// 按回退链查询译文；两级都缺失时返回调用方提供的兜底值。
    ///
    /// 供内容数据（如 JSON `display_name`）驱动的文案使用：键缺失时退回
    /// 内容自带的文本而不是键名，避免界面直接暴露内部键。
    pub fn get_or<'a>(&'a self, key: &'a str, fallback: &'a str) -> &'a str {
        let fallback_language = LanguageId::new(FALLBACK_LANGUAGE);
        if let Some(text) = self
            .tables
            .get(&self.active)
            .and_then(|table| table.get(key))
        {
            return text.as_str();
        }
        if let Some(text) = self
            .tables
            .get(&fallback_language)
            .and_then(|table| table.get(key))
        {
            return text.as_str();
        }
        fallback
    }

    /// 查询译文并完成 `{name}` 占位符插值；未知占位符原样保留。
    pub fn format(&self, key: &str, args: &[(&str, &str)]) -> String {
        substitute(self.get(key), args)
    }

    /// 指定语言的全部键，供语言文件一致性校验使用。
    pub fn keys_of(&self, id: &LanguageId) -> impl Iterator<Item = &str> {
        self.tables
            .get(id)
            .map(|table| table.keys().map(String::as_str).collect::<Vec<_>>())
            .unwrap_or_default()
            .into_iter()
    }

    /// 指定语言的展示名；未知语言返回 `None`。
    pub fn native_name_of(&self, id: &LanguageId) -> Option<&str> {
        self.languages
            .iter()
            .find(|info| &info.id == id)
            .map(|info| info.native_name.as_str())
    }
}

/// 把 `{name}` 占位符替换为参数值。
fn substitute(text: &str, args: &[(&str, &str)]) -> String {
    let mut result = text.to_string();
    for (name, value) in args {
        result = result.replace(&format!("{{{name}}}"), value);
    }
    result
}

#[cfg(test)]
#[path = "../../../tests/unit/engine/localization/store.rs"]
mod tests;
