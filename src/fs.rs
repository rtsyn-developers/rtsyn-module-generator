use std::fs;
use std::path::{Path, PathBuf};

use crate::render::RenderContext;

const PLUGIN_TEMPLATE_FILES: &[(&str, &str)] = &[
    (".clang-format", include_str!("../templates/rtsyn-module-plugin-template/.clang-format")),
    (".gitignore", include_str!("../templates/rtsyn-module-plugin-template/.gitignore")),
    ("README.md", include_str!("../templates/rtsyn-module-plugin-template/README.md")),
    ("xmake.lua", include_str!("../templates/rtsyn-module-plugin-template/xmake.lua")),
    (
        "include/rtsyn/_rtsyn_module_slug_.h",
        include_str!("../templates/rtsyn-module-plugin-template/include/rtsyn/_rtsyn_module_slug_.h"),
    ),
    (
        "src/_rtsyn_module_slug_.c",
        include_str!("../templates/rtsyn-module-plugin-template/src/_rtsyn_module_slug_.c"),
    ),
    (
        "src/_rtsyn_module_slug_/create.c",
        include_str!("../templates/rtsyn-module-plugin-template/src/_rtsyn_module_slug_/create.c"),
    ),
    (
        "src/_rtsyn_module_slug_/destroy.c",
        include_str!("../templates/rtsyn-module-plugin-template/src/_rtsyn_module_slug_/destroy.c"),
    ),
    (
        "src/_rtsyn_module_slug_/process.c",
        include_str!("../templates/rtsyn-module-plugin-template/src/_rtsyn_module_slug_/process.c"),
    ),
    (
        "src/_rtsyn_module_slug_/read_state.c",
        include_str!("../templates/rtsyn-module-plugin-template/src/_rtsyn_module_slug_/read_state.c"),
    ),
    (
        "src/_rtsyn_module_slug_/set_param.c",
        include_str!("../templates/rtsyn-module-plugin-template/src/_rtsyn_module_slug_/set_param.c"),
    ),
    (
        "src/_rtsyn_module_slug_/start.c",
        include_str!("../templates/rtsyn-module-plugin-template/src/_rtsyn_module_slug_/start.c"),
    ),
    (
        "src/_rtsyn_module_slug_/stop.c",
        include_str!("../templates/rtsyn-module-plugin-template/src/_rtsyn_module_slug_/stop.c"),
    ),
    (
        "src/rtsyn/internal/_rtsyn_module_slug_.h",
        include_str!("../templates/rtsyn-module-plugin-template/src/rtsyn/internal/_rtsyn_module_slug_.h"),
    ),
    (
        "src/rtsyn/internal/_rtsyn_module_slug_/create.h",
        include_str!("../templates/rtsyn-module-plugin-template/src/rtsyn/internal/_rtsyn_module_slug_/create.h"),
    ),
    (
        "src/rtsyn/internal/_rtsyn_module_slug_/destroy.h",
        include_str!("../templates/rtsyn-module-plugin-template/src/rtsyn/internal/_rtsyn_module_slug_/destroy.h"),
    ),
    (
        "src/rtsyn/internal/_rtsyn_module_slug_/process.h",
        include_str!("../templates/rtsyn-module-plugin-template/src/rtsyn/internal/_rtsyn_module_slug_/process.h"),
    ),
    (
        "src/rtsyn/internal/_rtsyn_module_slug_/read_state.h",
        include_str!("../templates/rtsyn-module-plugin-template/src/rtsyn/internal/_rtsyn_module_slug_/read_state.h"),
    ),
    (
        "src/rtsyn/internal/_rtsyn_module_slug_/set_param.h",
        include_str!("../templates/rtsyn-module-plugin-template/src/rtsyn/internal/_rtsyn_module_slug_/set_param.h"),
    ),
    (
        "src/rtsyn/internal/_rtsyn_module_slug_/start.h",
        include_str!("../templates/rtsyn-module-plugin-template/src/rtsyn/internal/_rtsyn_module_slug_/start.h"),
    ),
    (
        "src/rtsyn/internal/_rtsyn_module_slug_/stop.h",
        include_str!("../templates/rtsyn-module-plugin-template/src/rtsyn/internal/_rtsyn_module_slug_/stop.h"),
    ),
    (
        "tests/_rtsyn_module_slug_.cpp",
        include_str!("../templates/rtsyn-module-plugin-template/tests/_rtsyn_module_slug_.cpp"),
    ),
    ("tests/module_loader.cpp", include_str!("../templates/rtsyn-module-plugin-template/tests/module_loader.cpp")),
];

const DEVICE_TEMPLATE_FILES: &[(&str, &str)] = &[
    (".clang-format", include_str!("../templates/rtsyn-module-device-template/.clang-format")),
    (".gitignore", include_str!("../templates/rtsyn-module-device-template/.gitignore")),
    ("README.md", include_str!("../templates/rtsyn-module-device-template/README.md")),
    ("xmake.lua", include_str!("../templates/rtsyn-module-device-template/xmake.lua")),
    (
        "include/rtsyn/_rtsyn_module_slug_.h",
        include_str!("../templates/rtsyn-module-device-template/include/rtsyn/_rtsyn_module_slug_.h"),
    ),
    (
        "src/_rtsyn_module_slug_.c",
        include_str!("../templates/rtsyn-module-device-template/src/_rtsyn_module_slug_.c"),
    ),
    (
        "src/_rtsyn_module_slug_/create.c",
        include_str!("../templates/rtsyn-module-device-template/src/_rtsyn_module_slug_/create.c"),
    ),
    (
        "src/_rtsyn_module_slug_/destroy.c",
        include_str!("../templates/rtsyn-module-device-template/src/_rtsyn_module_slug_/destroy.c"),
    ),
    (
        "src/_rtsyn_module_slug_/process.c",
        include_str!("../templates/rtsyn-module-device-template/src/_rtsyn_module_slug_/process.c"),
    ),
    (
        "src/_rtsyn_module_slug_/read_state.c",
        include_str!("../templates/rtsyn-module-device-template/src/_rtsyn_module_slug_/read_state.c"),
    ),
    (
        "src/_rtsyn_module_slug_/set_param.c",
        include_str!("../templates/rtsyn-module-device-template/src/_rtsyn_module_slug_/set_param.c"),
    ),
    (
        "src/_rtsyn_module_slug_/start.c",
        include_str!("../templates/rtsyn-module-device-template/src/_rtsyn_module_slug_/start.c"),
    ),
    (
        "src/_rtsyn_module_slug_/stop.c",
        include_str!("../templates/rtsyn-module-device-template/src/_rtsyn_module_slug_/stop.c"),
    ),
    (
        "src/rtsyn/internal/_rtsyn_module_slug_.h",
        include_str!("../templates/rtsyn-module-device-template/src/rtsyn/internal/_rtsyn_module_slug_.h"),
    ),
    (
        "src/rtsyn/internal/_rtsyn_module_slug_/create.h",
        include_str!("../templates/rtsyn-module-device-template/src/rtsyn/internal/_rtsyn_module_slug_/create.h"),
    ),
    (
        "src/rtsyn/internal/_rtsyn_module_slug_/destroy.h",
        include_str!("../templates/rtsyn-module-device-template/src/rtsyn/internal/_rtsyn_module_slug_/destroy.h"),
    ),
    (
        "src/rtsyn/internal/_rtsyn_module_slug_/process.h",
        include_str!("../templates/rtsyn-module-device-template/src/rtsyn/internal/_rtsyn_module_slug_/process.h"),
    ),
    (
        "src/rtsyn/internal/_rtsyn_module_slug_/read_state.h",
        include_str!("../templates/rtsyn-module-device-template/src/rtsyn/internal/_rtsyn_module_slug_/read_state.h"),
    ),
    (
        "src/rtsyn/internal/_rtsyn_module_slug_/set_param.h",
        include_str!("../templates/rtsyn-module-device-template/src/rtsyn/internal/_rtsyn_module_slug_/set_param.h"),
    ),
    (
        "src/rtsyn/internal/_rtsyn_module_slug_/start.h",
        include_str!("../templates/rtsyn-module-device-template/src/rtsyn/internal/_rtsyn_module_slug_/start.h"),
    ),
    (
        "src/rtsyn/internal/_rtsyn_module_slug_/stop.h",
        include_str!("../templates/rtsyn-module-device-template/src/rtsyn/internal/_rtsyn_module_slug_/stop.h"),
    ),
    (
        "tests/_rtsyn_module_slug_.cpp",
        include_str!("../templates/rtsyn-module-device-template/tests/_rtsyn_module_slug_.cpp"),
    ),
    ("tests/module_loader.cpp", include_str!("../templates/rtsyn-module-device-template/tests/module_loader.cpp")),
];

#[derive(Clone, Copy)]
pub enum PrepareOutput {
    Create,
    Replace,
}

pub fn copy_rendered_templates(
    context: &RenderContext,
    output_dir: &Path,
    prepare_output: PrepareOutput,
) -> Result<(), String> {
    prepare_output_dir(output_dir, prepare_output)?;

    let template_files = match context.template_dir() {
        "rtsyn-module-plugin-template" => PLUGIN_TEMPLATE_FILES,
        "rtsyn-module-device-template" => DEVICE_TEMPLATE_FILES,
        _ => return Err("unsupported module template".to_string()),
    };

    for (template_path, template) in template_files {
        let output_path = output_path_for(context, Path::new(template_path), output_dir);
        let rendered = context.render_template(template)?;
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create `{}`: {error}", parent.display()))?;
        }
        fs::write(&output_path, rendered)
            .map_err(|error| format!("failed to write `{}`: {error}", output_path.display()))?;
    }

    Ok(())
}

fn prepare_output_dir(output_dir: &Path, prepare_output: PrepareOutput) -> Result<(), String> {
    if output_dir.exists() {
        match prepare_output {
            PrepareOutput::Create => {
                return Err(format!(
                    "output directory `{}` already exists; pass --force to replace it",
                    output_dir.display()
                ));
            }
            PrepareOutput::Replace => {
                fs::remove_dir_all(output_dir)
                    .map_err(|error| format!("failed to remove `{}`: {error}", output_dir.display()))?;
            }
        }
    }

    fs::create_dir_all(output_dir)
        .map_err(|error| format!("failed to create `{}`: {error}", output_dir.display()))
}

fn output_path_for(context: &RenderContext, relative: &Path, output_dir: &Path) -> PathBuf {
    let mut output = output_dir.to_path_buf();
    for component in relative.components() {
        let component = component.as_os_str().to_string_lossy();
        output.push(context.render_path_component(&component));
    }
    output
}
