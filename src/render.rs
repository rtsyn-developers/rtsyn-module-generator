use std::collections::BTreeMap;

use crate::names;
use crate::spec::{ModuleKind, ModuleSpec, PortDirection, PortSpec, ValueSpec};

pub struct RenderContext {
    spec: ModuleSpec,
    module: ModuleNames,
    replacements: BTreeMap<String, String>,
}

struct ModuleNames {
    slug: String,
    kebab: String,
    upper: String,
    title: String,
    pascal: String,
}

impl RenderContext {
    pub fn new(spec: ModuleSpec) -> Result<Self, String> {
        let module = ModuleNames {
            slug: names::slug(&spec.name),
            kebab: names::kebab(&spec.name),
            upper: names::upper(&spec.name),
            title: names::title(&spec.name),
            pascal: names::pascal(&spec.name),
        };

        if module.slug.is_empty() || !names::is_c_identifier(&module.slug) {
            return Err("module name cannot produce a valid C identifier".to_string());
        }

        let mut context = Self {
            spec,
            module,
            replacements: BTreeMap::new(),
        };
        context.build_replacements();
        Ok(context)
    }

    pub fn render_template(&self, template: &str) -> Result<String, String> {
        let mut rendered = template.to_string();
        for (key, value) in &self.replacements {
            rendered = rendered.replace(key, value);
        }

        if let Some(index) = rendered.find("_rtsyn_") {
            let end = rendered[index..]
                .find(|ch: char| ch.is_whitespace())
                .map(|offset| index + offset)
                .unwrap_or(rendered.len());
            return Err(format!("unresolved template token `{}`", &rendered[index..end]));
        }

        Ok(rendered)
    }

    pub fn render_path_component(&self, component: &str) -> String {
        component
            .replace("_rtsyn_module_slug_", &self.module.slug)
            .replace("_rtsyn_module_kebab_", &self.module.kebab)
            .replace("_rtsyn_module_upper_", &self.module.upper)
    }

    pub fn template_dir(&self) -> &'static str {
        self.spec.kind.template_dir()
    }

    fn build_replacements(&mut self) {
        self.insert("_rtsyn_module_name_", &self.spec.name.clone());
        self.insert("_rtsyn_module_slug_", &self.module.slug.clone());
        self.insert("_rtsyn_module_kebab_", &self.module.kebab.clone());
        self.insert("_rtsyn_module_upper_", &self.module.upper.clone());
        self.insert("_rtsyn_module_title_", &self.module.title.clone());
        self.insert("_rtsyn_module_pascal_", &self.module.pascal.clone());
        self.insert("_rtsyn_module_description_", &self.spec.description.clone());
        self.insert("_rtsyn_module_author_", &self.spec.author.clone());
        self.insert("_rtsyn_year_", "2026");

        self.insert_block(
            "    /* RTSYN_TEMPLATE:_rtsyn_instance_fields_ */\n    uint32_t reserved;",
            &self.instance_fields(),
        );
        self.insert_block(
            "    /* RTSYN_TEMPLATE:_rtsyn_port_enum_entries_ */\n    RTSYN__rtsyn_module_upper__PORT_COUNT,",
            &self.port_enum_entries(),
        );
        self.insert_block(
            "    /* RTSYN_TEMPLATE:_rtsyn_param_enum_entries_ */\n    RTSYN__rtsyn_module_upper__PARAM_COUNT,",
            &self.value_enum_entries("PARAM", &self.spec.params),
        );
        self.insert_block(
            "    /* RTSYN_TEMPLATE:_rtsyn_state_enum_entries_ */\n    RTSYN__rtsyn_module_upper__STATE_COUNT,",
            &self.value_enum_entries("STATE", &self.spec.states),
        );
        self.insert_block("/* RTSYN_TEMPLATE:_rtsyn_port_descriptors_ */", &self.port_descriptors());
        self.insert_block(
            "/* RTSYN_TEMPLATE:_rtsyn_param_descriptors_ */",
            &self.value_descriptors("params", "PARAM", &self.spec.params),
        );
        self.insert_block(
            "/* RTSYN_TEMPLATE:_rtsyn_state_descriptors_ */",
            &self.value_descriptors("states", "STATE", &self.spec.states),
        );
        let descriptor_ports_pointer =
            self.descriptor_pointer(&self.spec.ports, "rtsyn__rtsyn_module_slug__ports");
        let descriptor_params_pointer =
            self.descriptor_pointer(&self.spec.params, "rtsyn__rtsyn_module_slug__params");
        let descriptor_states_pointer =
            self.descriptor_pointer(&self.spec.states, "rtsyn__rtsyn_module_slug__states");
        self.insert_block(
            "/* RTSYN_TEMPLATE:_rtsyn_descriptor_ports_pointer_ */ nullptr",
            &descriptor_ports_pointer,
        );
        self.insert_block(
            "/* RTSYN_TEMPLATE:_rtsyn_descriptor_params_pointer_ */ nullptr",
            &descriptor_params_pointer,
        );
        self.insert_block(
            "/* RTSYN_TEMPLATE:_rtsyn_descriptor_states_pointer_ */ nullptr",
            &descriptor_states_pointer,
        );
        self.insert_block(
            "    /* RTSYN_TEMPLATE:_rtsyn_create_initializers_ */\n    _rtsyn_module_slug_->reserved = 0;",
            &self.create_initializers(),
        );
        self.insert_block(
            "    /* RTSYN_TEMPLATE:_rtsyn_reset_assignments_ */\n    _rtsyn_module_slug_->reserved = 0;",
            &self.reset_assignments(),
        );
        self.insert_block(
            "    /* RTSYN_TEMPLATE:_rtsyn_set_param_cases_ */\n    default:\n        return RTSYN_ABI_STATUS_INVALID_ARGUMENT;",
            &self.set_param_cases(),
        );
        self.insert_block(
            "    /* RTSYN_TEMPLATE:_rtsyn_read_state_cases_ */\n    default:\n        return RTSYN_ABI_STATUS_INVALID_ARGUMENT;",
            &self.read_state_cases(),
        );
        self.insert_block("    /* RTSYN_TEMPLATE:_rtsyn_process_reads_ */", &self.process_reads());
        self.insert_block(
            "    /* RTSYN_TEMPLATE:_rtsyn_process_writes_ */\n    return RTSYN_ABI_STATUS_OK;",
            &self.process_writes(),
        );
        self.insert_block(
            "    /* RTSYN_TEMPLATE:_rtsyn_device_immediate_reads_ */",
            &self.device_immediate_reads(),
        );
        self.insert_block(
            "    /* RTSYN_TEMPLATE:_rtsyn_device_immediate_writes_ */",
            &self.process_write_statements(),
        );
        self.insert_block(
            "    /* RTSYN_TEMPLATE:_rtsyn_device_terminal_reads_ */",
            &self.process_reads(),
        );
        self.insert_block(
            "    /* RTSYN_TEMPLATE:_rtsyn_device_terminal_writes_ */",
            &self.device_terminal_writes(),
        );
        self.insert_block(
            "/* RTSYN_TEMPLATE:_rtsyn_port_slot_count_ */ 1",
            &self.port_slot_count().to_string(),
        );
        self.insert_block(
            "/* RTSYN_TEMPLATE:_rtsyn_host_value_byte_count_ */ sizeof(double)",
            &self.host_value_byte_count(),
        );
        self.insert_block(
            "    /* RTSYN_TEMPLATE:_rtsyn_test_port_value_size_cases_ */\n    (void)port_index;\n    return sizeof(double);",
            &self.test_port_value_size_cases(),
        );
        self.insert_block(
            "    /* RTSYN_TEMPLATE:_rtsyn_test_descriptor_expectations_ */",
            &self.test_descriptor_expectations(),
        );
    }

    fn insert(&mut self, key: &str, value: &str) {
        let rendered_value = value.replace("_rtsyn_module_slug_", &self.module.slug);
        self.replacements.insert(key.to_string(), rendered_value);
    }

    fn insert_block(&mut self, marker_with_fallback: &str, value: &str) {
        self.replacements
            .insert(marker_with_fallback.to_string(), value.to_string());
    }

    fn instance_fields(&self) -> String {
        let mut lines = Vec::new();
        for port in &self.spec.ports {
            lines.push(self.field_declaration(port.value_type, &port.name));
        }
        for param in &self.spec.params {
            lines.push(self.field_declaration(param.value_type, &param.name));
        }
        for state in &self.spec.states {
            lines.push(self.field_declaration(state.value_type, &state.name));
        }
        if lines.is_empty() {
            lines.push("    uint32_t reserved;".to_string());
        }
        lines.join("\n")
    }

    fn field_declaration(&self, value_type: crate::spec::ValueType, name: &str) -> String {
        if value_type.is_string() {
            format!("    char {}[16];", names::slug(name))
        } else {
            format!("    {} {};", value_type.c_name(), names::slug(name))
        }
    }

    fn port_enum_entries(&self) -> String {
        let mut ports = self.spec.ports.clone();
        ports.sort_by_key(|port| port.number);
        let mut lines = ports
            .iter()
            .map(|port| {
                format!(
                    "    RTSYN_{}_PORT_{} = {},",
                    self.module.upper,
                    names::upper(&port.name),
                    port.number
                )
            })
            .collect::<Vec<_>>();
        lines.push(format!(
            "    RTSYN_{}_PORT_COUNT = {},",
            self.module.upper,
            self.port_slot_count()
        ));
        lines.join("\n")
    }

    fn value_enum_entries(&self, prefix: &str, values: &[ValueSpec]) -> String {
        let mut values = values.to_vec();
        values.sort_by_key(|value| value.number);
        let mut lines = values
            .iter()
            .map(|value| {
                format!(
                    "    RTSYN_{}_{}_{} = {},",
                    self.module.upper,
                    prefix,
                    names::upper(&value.name),
                    value.number
                )
            })
            .collect::<Vec<_>>();
        lines.push(format!(
            "    RTSYN_{}_{}_COUNT = {},",
            self.module.upper,
            prefix,
            self.value_slot_count(&values)
        ));
        lines.join("\n")
    }

    fn port_descriptors(&self) -> String {
        if self.spec.ports.is_empty() {
            return String::new();
        }

        format!(
            "static const rtsyn_abi_port_descriptor_t rtsyn_{}_ports[RTSYN_{}_PORT_COUNT] = {{\n{}\n}};\n",
            self.module.slug,
            self.module.upper,
            self.spec
                .ports
                .iter()
                .map(|port| self.port_descriptor(port))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    fn c_string(value: &str) -> String {
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
    }

    fn port_descriptor(&self, port: &PortSpec) -> String {
        format!(
            "    [RTSYN_{}_PORT_{}] =\n        {{\n            .name = \"{}\",\n            .value_type = {},\n            .direction = {},\n        }},",
            self.module.upper,
            names::upper(&port.name),
            Self::c_string(&port.name),
            port.value_type.abi_name(),
            port.direction.abi_name()
        )
    }

    fn value_descriptors(&self, noun: &str, prefix: &str, values: &[ValueSpec]) -> String {
        if values.is_empty() {
            return String::new();
        }

        format!(
            "static const rtsyn_abi_{}_descriptor_t rtsyn_{}_{}[RTSYN_{}_{}_COUNT] = {{\n{}\n}};\n",
            if noun == "params" { "param" } else { "state" },
            self.module.slug,
            noun,
            self.module.upper,
            prefix,
            values
                .iter()
                .map(|value| self.value_descriptor(prefix, value))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    fn value_descriptor(&self, prefix: &str, value: &ValueSpec) -> String {
        format!(
            "    [RTSYN_{}_{}_{}] =\n        {{\n            .name = \"{}\",\n            .description = \"{}\",\n            .value_type = {},\n        }},",
            self.module.upper,
            prefix,
            names::upper(&value.name),
            Self::c_string(&value.name),
            Self::c_string(&value.description),
            value.value_type.abi_name()
        )
    }

    fn descriptor_pointer<T>(&self, values: &[T], symbol: &str) -> String {
        if values.is_empty() {
            "nullptr".to_string()
        } else {
            symbol.to_string()
        }
    }

    fn create_initializers(&self) -> String {
        let mut lines = Vec::new();
        for port in &self.spec.ports {
            lines.push(self.field_initializer(port.value_type, &port.name));
        }
        for param in &self.spec.params {
            let default_value = if param.value_type.is_string() {
                format!("\"{}\"", Self::c_string(&param.default_value))
            } else {
                param.default_value.clone()
            };
            lines.push(self.field_create_initializer(param.value_type, &param.name, &default_value));
        }
        for state in &self.spec.states {
            lines.push(self.field_initializer(state.value_type, &state.name));
        }
        lines.join("\n")
    }

    fn reset_assignments(&self) -> String {
        let mut lines = Vec::new();
        for port in &self.spec.ports {
            lines.push(self.field_initializer(port.value_type, &port.name));
        }
        for state in &self.spec.states {
            lines.push(self.field_initializer(state.value_type, &state.name));
        }
        if lines.is_empty() {
            lines.push(format!("    {}->reserved = 0;", self.module.slug));
        }
        lines.join("\n")
    }

    fn field_initializer(&self, value_type: crate::spec::ValueType, name: &str) -> String {
        self.field_create_initializer(value_type, name, value_type.zero_literal())
    }

    fn field_create_initializer(
        &self,
        value_type: crate::spec::ValueType,
        name: &str,
        value: &str,
    ) -> String {
        let field = names::slug(name);
        if value_type.is_string() {
            format!(
                "    strncpy({}->{}, {}, sizeof({}->{}) - 1);\n    {}->{}[sizeof({}->{}) - 1] = '\\0';",
                self.module.slug,
                field,
                value,
                self.module.slug,
                field,
                self.module.slug,
                field,
                self.module.slug,
                field
            )
        } else {
            format!("    {}->{} = {};", self.module.slug, field, value)
        }
    }

    fn set_param_cases(&self) -> String {
        if self.spec.params.is_empty() {
            return "    (void)param_index;\n    return RTSYN_ABI_STATUS_INVALID_ARGUMENT;".to_string();
        }

        let cases = self
            .spec
            .params
            .iter()
            .map(|param| {
                if param.value_type.is_string() {
                    format!(
                        "    case RTSYN_{}_PARAM_{}:\n        strncpy({}->{}, (const char *)value, sizeof({}->{}) - 1);\n        {}->{}[sizeof({}->{}) - 1] = '\\0';\n        return RTSYN_ABI_STATUS_OK;",
                        self.module.upper,
                        names::upper(&param.name),
                        self.module.slug,
                        names::slug(&param.name),
                        self.module.slug,
                        names::slug(&param.name),
                        self.module.slug,
                        names::slug(&param.name),
                        self.module.slug,
                        names::slug(&param.name)
                    )
                } else {
                    format!(
                        "    case RTSYN_{}_PARAM_{}:\n        {}->{} = *(const {} *)value;\n        return RTSYN_ABI_STATUS_OK;",
                        self.module.upper,
                        names::upper(&param.name),
                        self.module.slug,
                        names::slug(&param.name),
                        param.value_type.c_name()
                    )
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!("{cases}\n    default:\n        return RTSYN_ABI_STATUS_INVALID_ARGUMENT;")
    }

    fn read_state_cases(&self) -> String {
        if self.spec.states.is_empty() {
            return "    (void)state_index;\n    return RTSYN_ABI_STATUS_INVALID_ARGUMENT;".to_string();
        }

        let cases = self
            .spec
            .states
            .iter()
            .map(|state| {
                if state.value_type.is_string() {
                    format!(
                        "    case RTSYN_{}_STATE_{}:\n        strncpy((char *)out_value, {}->{}, sizeof({}->{}));\n        ((char *)out_value)[sizeof({}->{}) - 1] = '\\0';\n        return RTSYN_ABI_STATUS_OK;",
                        self.module.upper,
                        names::upper(&state.name),
                        self.module.slug,
                        names::slug(&state.name),
                        self.module.slug,
                        names::slug(&state.name),
                        self.module.slug,
                        names::slug(&state.name)
                    )
                } else {
                    format!(
                        "    case RTSYN_{}_STATE_{}:\n        *({} *)out_value = {}->{};\n        return RTSYN_ABI_STATUS_OK;",
                        self.module.upper,
                        names::upper(&state.name),
                        state.value_type.c_name(),
                        self.module.slug,
                        names::slug(&state.name)
                    )
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!("{cases}\n    default:\n        return RTSYN_ABI_STATUS_INVALID_ARGUMENT;")
    }

    fn process_reads(&self) -> String {
        let input_ports = self
            .spec
            .ports
            .iter()
            .filter(|port| port.direction == PortDirection::Input)
            .collect::<Vec<_>>();

        if input_ports.is_empty() {
            return String::new();
        }

        input_ports
            .iter()
            .map(|port| {
                let read_target = if port.value_type.is_string() {
                    format!("{}->{}", self.module.slug, names::slug(&port.name))
                } else {
                    format!("&{}->{}", self.module.slug, names::slug(&port.name))
                };
                let mut block = format!(
                    "    status = context->api->read(context, RTSYN_{}_PORT_{}, {});\n    if (status < 0)\n    {{\n        return status;\n    }}",
                    self.module.upper,
                    names::upper(&port.name),
                    read_target
                );
                if port.value_type.is_string() {
                    block.push_str(&format!(
                        "\n    {}->{}[sizeof({}->{}) - 1] = '\\0';",
                        self.module.slug,
                        names::slug(&port.name),
                        self.module.slug,
                        names::slug(&port.name)
                    ));
                }
                block
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn process_writes(&self) -> String {
        let statements = self.process_write_statements();
        if statements.is_empty() {
            return "    return RTSYN_ABI_STATUS_OK;".to_string();
        }

        format!("{statements}\n    return RTSYN_ABI_STATUS_OK;")
    }

    fn process_write_statements(&self) -> String {
        let output_ports = self
            .spec
            .ports
            .iter()
            .filter(|port| port.direction == PortDirection::Output)
            .collect::<Vec<_>>();

        if output_ports.is_empty() {
            return String::new();
        }

        let mut lines = Vec::new();
        lines.push("    /* TODO: update output fields before writing them to the host. */".to_string());
        for port in output_ports {
            lines.push(format!(
                "    status = context->api->write(context, RTSYN_{}_PORT_{}, &{}->{});\n    if (status < 0)\n    {{\n        return status;\n    }}",
                self.module.upper,
                names::upper(&port.name),
                self.module.slug,
                names::slug(&port.name)
            ));
        }
        lines.join("\n\n")
    }

    fn device_immediate_reads(&self) -> String {
        if self.spec.kind != ModuleKind::Device {
            return String::new();
        }

        "    /* TODO: acquire device inputs and update output fields. */".to_string()
    }

    fn device_terminal_writes(&self) -> String {
        if self.spec.kind != ModuleKind::Device {
            return String::new();
        }

        "    /* TODO: publish input fields to the device output side. */".to_string()
    }

    fn port_slot_count(&self) -> u32 {
        self.spec
            .ports
            .iter()
            .map(|port| port.number)
            .max()
            .map(|number| number + 1)
            .unwrap_or(0)
    }

    fn host_value_byte_count(&self) -> String {
        self.spec
            .ports
            .iter()
            .map(|port| port.value_type.c_size())
            .max_by_key(|size| numeric_size(size))
            .unwrap_or("sizeof(double)")
            .to_string()
    }

    fn test_port_value_size_cases(&self) -> String {
        if self.spec.ports.is_empty() {
            return "    (void)port_index;\n    return sizeof(double);".to_string();
        }

        let cases = self
            .spec
            .ports
            .iter()
            .map(|port| {
                format!(
                    "    case {}:\n        return {};",
                    port.number,
                    port.value_type.c_size()
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("    switch (port_index)\n    {{\n{cases}\n    default:\n        return 0;\n    }}")
    }

    fn value_slot_count(&self, values: &[ValueSpec]) -> u32 {
        values
            .iter()
            .map(|value| value.number)
            .max()
            .map(|number| number + 1)
            .unwrap_or(0)
    }

    fn test_descriptor_expectations(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            "    ASSERT_EQ(descriptor->port_count, {}U);",
            self.port_slot_count()
        ));
        for port in &self.spec.ports {
            lines.push(format!(
                "    EXPECT_STREQ(descriptor->ports[{}].name, \"{}\");",
                port.number, port.name
            ));
            lines.push(format!(
                "    EXPECT_EQ(descriptor->ports[{}].direction, {});",
                port.number,
                port.direction.abi_name()
            ));
        }
        lines.push(format!(
            "    ASSERT_EQ(descriptor->param_count, {}U);",
            self.value_slot_count(&self.spec.params)
        ));
        for param in &self.spec.params {
            lines.push(format!(
                "    EXPECT_STREQ(descriptor->params[{}].name, \"{}\");",
                param.number, param.name
            ));
        }
        lines.push(format!(
            "    ASSERT_EQ(descriptor->state_count, {}U);",
            self.value_slot_count(&self.spec.states)
        ));
        for state in &self.spec.states {
            lines.push(format!(
                "    EXPECT_STREQ(descriptor->states[{}].name, \"{}\");",
                state.number, state.name
            ));
        }
        lines.join("\n")
    }
}

fn numeric_size(size: &str) -> usize {
    match size {
        "sizeof(float)" => 4,
        "sizeof(double)" => 8,
        "sizeof(int)" => 4,
        "sizeof(uint64_t)" => 8,
        "16" => 16,
        _ => 0,
    }
}
