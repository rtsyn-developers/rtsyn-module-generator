use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::names;

#[derive(Clone, Debug)]
pub struct ModuleSpec {
    pub name: String,
    pub kind: ModuleKind,
    pub description: String,
    pub author: String,
    pub ports: Vec<PortSpec>,
    pub params: Vec<ValueSpec>,
    pub states: Vec<ValueSpec>,
}

#[derive(Clone, Debug)]
pub struct PortSpec {
    pub name: String,
    pub number: u32,
    pub direction: PortDirection,
    pub value_type: ValueType,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct ValueSpec {
    pub name: String,
    pub number: u32,
    pub value_type: ValueType,
    pub default_value: String,
    pub description: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleKind {
    Plugin,
    Device,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueType {
    F32,
    F64,
    I64,
    U64,
    String,
}

impl ModuleSpec {
    pub fn read_from(path: &Path) -> Result<Self, String> {
        let contents = fs::read_to_string(path)
            .map_err(|error| format!("failed to read `{}`: {error}", path.display()))?;
        Self::parse(&contents)
    }

    fn parse(contents: &str) -> Result<Self, String> {
        let mut builder = ModuleSpecBuilder::default();
        let mut section = Section::Root;

        for (line_index, raw_line) in contents.lines().enumerate() {
            let line_number = line_index + 1;
            let line = strip_comment(raw_line).trim();
            if line.is_empty() {
                continue;
            }

            match line {
                "[[ports]]" => {
                    builder.ports.push(PortBuilder::default());
                    section = Section::Port(builder.ports.len() - 1);
                    continue;
                }
                "[[params]]" | "[[values]]" => {
                    builder.params.push(ValueBuilder::default());
                    section = Section::Param(builder.params.len() - 1);
                    continue;
                }
                "[[states]]" => {
                    builder.states.push(ValueBuilder::default());
                    section = Section::State(builder.states.len() - 1);
                    continue;
                }
                _ => {}
            }

            let (key, value) = parse_assignment(line, line_number)?;
            builder.set(section, key, value, line_number)?;
        }

        let spec = builder.build()?;
        spec.validate()?;
        Ok(spec)
    }

    fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("module name is required".to_string());
        }
        if names::slug(&self.name).is_empty() {
            return Err(format!("module name `{}` cannot form a valid slug", self.name));
        }

        validate_items("port", self.ports.iter().map(|port| (&port.name, port.number)))?;
        validate_items("param", self.params.iter().map(|param| (&param.name, param.number)))?;
        validate_items("state", self.states.iter().map(|state| (&state.name, state.number)))?;

        Ok(())
    }
}

impl ModuleKind {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "plugin" => Ok(Self::Plugin),
            "device" => Ok(Self::Device),
            _ => Err(format!("unsupported module kind `{value}`")),
        }
    }

    pub fn template_dir(self) -> &'static str {
        match self {
            Self::Plugin => "rtsyn-module-plugin-template",
            Self::Device => "rtsyn-module-device-template",
        }
    }
}

impl PortDirection {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "input" | "in" => Ok(Self::Input),
            "output" | "out" => Ok(Self::Output),
            _ => Err(format!("unsupported port direction `{value}`")),
        }
    }

    pub fn abi_name(self) -> &'static str {
        match self {
            Self::Input => "RTSYN_ABI_PORT_DIRECTION_IN",
            Self::Output => "RTSYN_ABI_PORT_DIRECTION_OUT",
        }
    }
}

impl ValueType {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "f32" | "float32" | "float" => Ok(Self::F32),
            "f64" | "float64" | "double" => Ok(Self::F64),
            "i64" | "int64" | "int" => Ok(Self::I64),
            "u64" | "uint64" | "uint" => Ok(Self::U64),
            "string" | "str" => Ok(Self::String),
            _ => Err(format!("unsupported value type `{value}`")),
        }
    }

    pub fn abi_name(self) -> &'static str {
        match self {
            Self::F32 => "RTSYN_ABI_VALUE_F32",
            Self::F64 => "RTSYN_ABI_VALUE_F64",
            Self::I64 => "RTSYN_ABI_VALUE_I64",
            Self::U64 => "RTSYN_ABI_VALUE_U64",
            Self::String => "RTSYN_ABI_VALUE_STRING",
        }
    }

    pub fn c_name(self) -> &'static str {
        match self {
            Self::F32 => "float",
            Self::F64 => "double",
            Self::I64 => "int",
            Self::U64 => "uint64_t",
            Self::String => "char",
        }
    }

    pub fn zero_literal(self) -> &'static str {
        match self {
            Self::F32 => "0.0F",
            Self::F64 => "0.0",
            Self::I64 => "0",
            Self::U64 => "0U",
            Self::String => "\"\"",
        }
    }

    pub fn c_size(self) -> &'static str {
        match self {
            Self::F32 => "sizeof(float)",
            Self::F64 => "sizeof(double)",
            Self::I64 => "sizeof(int)",
            Self::U64 => "sizeof(uint64_t)",
            Self::String => "16",
        }
    }

    pub fn is_string(self) -> bool {
        self == Self::String
    }
}

#[derive(Clone, Copy)]
enum Section {
    Root,
    Port(usize),
    Param(usize),
    State(usize),
}

#[derive(Default)]
struct ModuleSpecBuilder {
    name: Option<String>,
    kind: Option<ModuleKind>,
    description: Option<String>,
    author: Option<String>,
    ports: Vec<PortBuilder>,
    params: Vec<ValueBuilder>,
    states: Vec<ValueBuilder>,
}

#[derive(Default)]
struct PortBuilder {
    name: Option<String>,
    number: Option<u32>,
    direction: Option<PortDirection>,
    value_type: Option<ValueType>,
    description: Option<String>,
}

#[derive(Default)]
struct ValueBuilder {
    name: Option<String>,
    number: Option<u32>,
    value_type: Option<ValueType>,
    default_value: Option<String>,
    description: Option<String>,
}

impl ModuleSpecBuilder {
    fn set(
        &mut self,
        section: Section,
        key: &str,
        value: String,
        line_number: usize,
    ) -> Result<(), String> {
        match section {
            Section::Root => self.set_root(key, value, line_number),
            Section::Port(index) => self.ports[index].set(key, value, line_number),
            Section::Param(index) => self.params[index].set(key, value, line_number),
            Section::State(index) => self.states[index].set(key, value, line_number),
        }
    }

    fn set_root(&mut self, key: &str, value: String, line_number: usize) -> Result<(), String> {
        match key {
            "name" => self.name = Some(value),
            "kind" | "type" => self.kind = Some(ModuleKind::parse(&value)?),
            "description" => self.description = Some(value),
            "author" => self.author = Some(value),
            _ => return Err(format!("line {line_number}: unknown root key `{key}`")),
        }
        Ok(())
    }

    fn build(self) -> Result<ModuleSpec, String> {
        Ok(ModuleSpec {
            name: required(self.name, "name")?,
            kind: self.kind.unwrap_or(ModuleKind::Plugin),
            description: self.description.unwrap_or_default(),
            author: self.author.unwrap_or_default(),
            ports: self
                .ports
                .into_iter()
                .enumerate()
                .map(|(index, port)| port.build(index))
                .collect::<Result<Vec<_>, _>>()?,
            params: self
                .params
                .into_iter()
                .enumerate()
                .map(|(index, param)| param.build("param", index))
                .collect::<Result<Vec<_>, _>>()?,
            states: self
                .states
                .into_iter()
                .enumerate()
                .map(|(index, state)| state.build("state", index))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl PortBuilder {
    fn set(&mut self, key: &str, value: String, line_number: usize) -> Result<(), String> {
        match key {
            "name" => self.name = Some(value),
            "number" | "index" => self.number = Some(parse_u32(&value, line_number, key)?),
            "direction" => self.direction = Some(PortDirection::parse(&value)?),
            "type" | "value_type" => self.value_type = Some(ValueType::parse(&value)?),
            "description" => self.description = Some(value),
            _ => return Err(format!("line {line_number}: unknown port key `{key}`")),
        }
        Ok(())
    }

    fn build(self, index: usize) -> Result<PortSpec, String> {
        Ok(PortSpec {
            name: required(self.name, &format!("ports[{index}].name"))?,
            number: required(self.number, &format!("ports[{index}].number"))?,
            direction: required(self.direction, &format!("ports[{index}].direction"))?,
            value_type: self.value_type.unwrap_or(ValueType::F64),
            description: self.description.unwrap_or_default(),
        })
    }
}

impl ValueBuilder {
    fn set(&mut self, key: &str, value: String, line_number: usize) -> Result<(), String> {
        match key {
            "name" => self.name = Some(value),
            "number" | "index" => self.number = Some(parse_u32(&value, line_number, key)?),
            "type" | "value_type" => self.value_type = Some(ValueType::parse(&value)?),
            "default" | "default_value" => self.default_value = Some(value),
            "description" => self.description = Some(value),
            _ => return Err(format!("line {line_number}: unknown value key `{key}`")),
        }
        Ok(())
    }

    fn build(self, kind: &str, index: usize) -> Result<ValueSpec, String> {
        let value_type = self.value_type.unwrap_or(ValueType::F64);
        Ok(ValueSpec {
            name: required(self.name, &format!("{kind}s[{index}].name"))?,
            number: required(self.number, &format!("{kind}s[{index}].number"))?,
            value_type,
            default_value: self.default_value.unwrap_or_else(|| value_type.zero_literal().to_string()),
            description: self.description.unwrap_or_default(),
        })
    }
}

fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    for (index, ch) in line.char_indices() {
        match ch {
            '"' => in_string = !in_string,
            '#' if !in_string => return &line[..index],
            _ => {}
        }
    }
    line
}

fn parse_assignment(line: &str, line_number: usize) -> Result<(&str, String), String> {
    let Some((key, value)) = line.split_once('=') else {
        return Err(format!("line {line_number}: expected `key = value`"));
    };
    let key = key.trim();
    let value = parse_value(value.trim(), line_number)?;
    if key.is_empty() {
        return Err(format!("line {line_number}: key cannot be empty"));
    }
    Ok((key, value))
}

fn parse_value(value: &str, line_number: usize) -> Result<String, String> {
    if value.starts_with('"') {
        if !value.ends_with('"') || value.len() == 1 {
            return Err(format!("line {line_number}: unterminated string"));
        }
        Ok(value[1..value.len() - 1].replace("\\\"", "\""))
    } else {
        Ok(value.to_string())
    }
}

fn parse_u32(value: &str, line_number: usize, key: &str) -> Result<u32, String> {
    value
        .parse::<u32>()
        .map_err(|error| format!("line {line_number}: `{key}` must be a u32: {error}"))
}

fn required<T>(value: Option<T>, name: &str) -> Result<T, String> {
    value.ok_or_else(|| format!("missing required field `{name}`"))
}

fn validate_items<'a>(
    kind: &str,
    items: impl Iterator<Item = (&'a String, u32)>,
) -> Result<(), String> {
    let mut names_seen = HashSet::new();
    let mut numbers_seen = HashSet::new();

    for (name, number) in items {
        if number == u32::MAX {
            return Err(format!("{kind} number `{number}` is reserved"));
        }
        if !names::is_c_identifier(name) {
            return Err(format!("invalid {kind} name `{name}`: expected a C identifier"));
        }
        if !names_seen.insert(name.clone()) {
            return Err(format!("duplicate {kind} name `{name}`"));
        }
        if !numbers_seen.insert(number) {
            return Err(format!("duplicate {kind} number `{number}`"));
        }
    }

    for number in 0..numbers_seen.len() as u32 {
        if !numbers_seen.contains(&number) {
            return Err(format!(
                "{kind} numbers must be dense from 0 to {}",
                numbers_seen.len().saturating_sub(1)
            ));
        }
    }

    Ok(())
}
