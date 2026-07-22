use {
    anyhow::{Context, Result},
    handlebars::Handlebars,
    heck::{ToKebabCase, ToPascalCase, ToSnakeCase},
    serde::Serialize,
    solana_keypair::{Keypair, Signer},
    std::{
        fs,
        path::{Path, PathBuf},
    },
};

#[derive(Serialize)]
pub struct TemplateContext {
    pub program_id: String,
    pub project_name_snake: String,
    pub project_name_kebab: String,
    pub project_name_pascal: String,
    pub program_name_snake: String,
    pub program_name_kebab: String,
    pub program_name_pascal: String,
    pub instruction_name_snake: String,
    pub instruction_name_pascal: String,
    pub typhoon_version: String,
    pub typhoon_builder_version: String,
    pub typhoon_idl_version: String,
}

impl TemplateContext {
    fn new(
        project_name: &str,
        program_name: &str,
        instruction_name: &str,
        versions: (String, String, String),
    ) -> Self {
        let (typhoon_version, typhoon_builder_version, typhoon_idl_version) = versions;
        Self {
            program_id: Keypair::new().pubkey().to_string(),
            project_name_snake: project_name.to_snake_case(),
            project_name_kebab: project_name.to_kebab_case(),
            project_name_pascal: project_name.to_pascal_case(),
            program_name_snake: program_name.to_snake_case(),
            program_name_kebab: program_name.to_kebab_case(),
            program_name_pascal: program_name.to_pascal_case(),
            instruction_name_snake: instruction_name.to_snake_case(),
            instruction_name_pascal: instruction_name.to_pascal_case(),
            typhoon_version,
            typhoon_builder_version,
            typhoon_idl_version,
        }
    }
}

pub struct Template {
    #[allow(dead_code)]
    pub name: String,
    pub files: Vec<TemplateFile>,
}

pub struct TemplateFile {
    pub path_template: String,
    pub content_template: String,
}

impl Template {
    pub fn generate_workspace(
        target_dir: &Path,
        project_name: &str,
        program_name: Option<String>,
        typhoon_path: Option<PathBuf>,
    ) -> Result<()> {
        let (typhoon_version, typhoon_builder_version, typhoon_idl_version) =
            if let Some(typhoon_path) = typhoon_path {
                // Convert typhoon_path to an absolute path
                let typhoon_path = typhoon_path.canonicalize().with_context(|| {
                    format!("Failed to canonicalize path: {}", typhoon_path.display())
                })?;
                (
                    format!(
                        "{{ path = \"{}\" }}",
                        typhoon_path.join("crates").join("lib").to_string_lossy()
                    ),
                    format!(
                        "{{ path = \"{}\" }}",
                        typhoon_path
                            .join("crates")
                            .join("instruction-builder")
                            .to_string_lossy()
                    ),
                    format!(
                        "{{ path = \"{}\" }}",
                        typhoon_path
                            .join("crates")
                            .join("idl-generator")
                            .to_string_lossy()
                    ),
                )
            } else {
                (
                    format!("\"{}\"", env!("CARGO_PKG_VERSION")).to_string(),
                    format!("\"{}\"", env!("CARGO_PKG_VERSION")).to_string(),
                    format!("\"{}\"", env!("CARGO_PKG_VERSION")).to_string(),
                )
            };

        let program_name = program_name.unwrap_or_else(|| project_name.to_string());
        let ctx = TemplateContext::new(
            project_name,
            &program_name,
            "",
            (
                typhoon_version,
                typhoon_builder_version,
                typhoon_idl_version,
            ),
        );

        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        for file in &Self::workspace(&program_name).files {
            // Render file path template
            let file_path = handlebars
                .render_template(&file.path_template, &ctx)
                .with_context(|| {
                    format!(
                        "Failed to render file path template: {}",
                        file.path_template
                    )
                })?;

            // Render file content template
            let file_content = handlebars
                .render_template(&file.content_template, &ctx)
                .with_context(|| {
                    format!(
                        "Failed to render file content template: {}",
                        file.path_template
                    )
                })?;

            let full_path = target_dir.join(&file_path);

            // Create parent directories if needed
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }

            fs::write(&full_path, file_content)
                .with_context(|| format!("Failed to write file: {}", full_path.display()))?;
        }

        Ok(())
    }

    pub fn generate_program(project_dir: &Path, program_name: &str) -> Result<()> {
        let v = env!("CARGO_PKG_VERSION");
        let ctx = TemplateContext::new(
            program_name,
            program_name,
            "",
            (v.to_string(), v.to_string(), v.to_string()),
        );

        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        for file in &Self::program(program_name).files {
            // Render file path template
            let file_path = handlebars
                .render_template(&file.path_template, &ctx)
                .with_context(|| {
                    format!(
                        "Failed to render file path template: {}",
                        file.path_template
                    )
                })?;

            // Render file content template
            let file_content = handlebars
                .render_template(&file.content_template, &ctx)
                .with_context(|| {
                    format!(
                        "Failed to render file content template: {}",
                        file.path_template
                    )
                })?;

            let full_path = project_dir.join(&file_path);

            // Create parent directories if needed
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }

            fs::write(&full_path, file_content)
                .with_context(|| format!("Failed to write file: {}", full_path.display()))?;
        }

        Ok(())
    }

    pub fn generate_handler(
        project_dir: &Path,
        program_name: &str,
        instruction_name: &str,
    ) -> Result<()> {
        let v = env!("CARGO_PKG_VERSION");
        let ctx = TemplateContext::new(
            program_name,
            program_name,
            instruction_name,
            (v.to_string(), v.to_string(), v.to_string()),
        );

        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        for file in &Self::handler(program_name, instruction_name).files {
            // Render file path template
            let file_path = handlebars
                .render_template(&file.path_template, &ctx)
                .with_context(|| {
                    format!(
                        "Failed to render file path template: {}",
                        file.path_template
                    )
                })?;

            // Render file content template
            let file_content = handlebars
                .render_template(&file.content_template, &ctx)
                .with_context(|| {
                    format!(
                        "Failed to render file content template: {}",
                        file.path_template
                    )
                })?;

            let full_path = project_dir.join(&file_path);

            // Create parent directories if needed
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }

            fs::write(&full_path, file_content)
                .with_context(|| format!("Failed to write file: {}", full_path.display()))?;
        }

        Ok(())
    }

    fn workspace(program_name: &str) -> Self {
        Self {
            name: "workspace".to_string(),
            files: vec![
                TemplateFile {
                    path_template: "Cargo.toml".to_string(),
                    content_template: include_str!("../templates/Cargo.toml.template").to_string(),
                },
                TemplateFile {
                    path_template: format!("programs/{}/Cargo.toml", program_name.to_snake_case())
                        .to_string(),
                    content_template: include_str!("../templates/programs/Cargo.toml.template")
                        .to_string(),
                },
                TemplateFile {
                    path_template: format!("programs/{}/build.rs", program_name.to_snake_case())
                        .to_string(),
                    content_template: include_str!("../templates/programs/build.rs.template")
                        .to_string(),
                },
                TemplateFile {
                    path_template: format!("programs/{}/src/lib.rs", program_name.to_snake_case())
                        .to_string(),
                    content_template: include_str!("../templates/programs/src/lib.rs.template")
                        .to_string(),
                },
                TemplateFile {
                    path_template: format!(
                        "programs/{}/src/state/mod.rs",
                        program_name.to_snake_case()
                    )
                    .to_string(),
                    content_template: include_str!(
                        "../templates/programs/src/state/mod.rs.template"
                    )
                    .to_string(),
                },
                TemplateFile {
                    path_template: format!(
                        "programs/{}/src/state/counter.rs",
                        program_name.to_snake_case()
                    )
                    .to_string(),
                    content_template: include_str!(
                        "../templates/programs/src/state/counter.rs.template"
                    )
                    .to_string(),
                },
                TemplateFile {
                    path_template: format!(
                        "programs/{}/src/handlers/mod.rs",
                        program_name.to_snake_case()
                    )
                    .to_string(),
                    content_template: include_str!(
                        "../templates/programs/src/handlers/mod.rs.template"
                    )
                    .to_string(),
                },
                TemplateFile {
                    path_template: format!(
                        "programs/{}/src/handlers/initialize.rs",
                        program_name.to_snake_case()
                    )
                    .to_string(),
                    content_template: include_str!(
                        "../templates/programs/src/handlers/initialize.rs.template"
                    )
                    .to_string(),
                },
                TemplateFile {
                    path_template: format!(
                        "programs/{}/src/handlers/close.rs",
                        program_name.to_snake_case()
                    )
                    .to_string(),
                    content_template: include_str!(
                        "../templates/programs/src/handlers/close.rs.template"
                    )
                    .to_string(),
                },
                TemplateFile {
                    path_template: format!(
                        "programs/{}/src/handlers/increment.rs",
                        program_name.to_snake_case()
                    )
                    .to_string(),
                    content_template: include_str!(
                        "../templates/programs/src/handlers/increment.rs.template"
                    )
                    .to_string(),
                },
                TemplateFile {
                    path_template: format!(
                        "programs/{}/src/contexts/mod.rs",
                        program_name.to_snake_case()
                    )
                    .to_string(),
                    content_template: include_str!(
                        "../templates/programs/src/contexts/mod.rs.template"
                    )
                    .to_string(),
                },
                TemplateFile {
                    path_template: "tests/Cargo.toml".to_string(),
                    content_template: include_str!("../templates/tests/Cargo.toml.template")
                        .to_string(),
                },
                TemplateFile {
                    path_template: format!("tests/tests/{}.rs", program_name.to_snake_case())
                        .to_string(),
                    content_template: include_str!(
                        "../templates/tests/tests/integration.rs.template"
                    )
                    .to_string(),
                },
            ],
        }
    }

    fn program(program_name: &str) -> Self {
        Self {
            name: "program".to_string(),
            files: vec![
                TemplateFile {
                    path_template: format!("programs/{}/Cargo.toml", program_name.to_snake_case())
                        .to_string(),
                    content_template: include_str!("../templates/programs/Cargo.toml.template")
                        .to_string(),
                },
                TemplateFile {
                    path_template: format!("programs/{}/build.rs", program_name.to_snake_case())
                        .to_string(),
                    content_template: include_str!("../templates/programs/build.rs.template")
                        .to_string(),
                },
                TemplateFile {
                    path_template: format!("programs/{}/src/lib.rs", program_name.to_snake_case())
                        .to_string(),
                    content_template: include_str!("../templates/programs/src/lib.rs.template")
                        .to_string(),
                },
                TemplateFile {
                    path_template: format!(
                        "programs/{}/src/state/mod.rs",
                        program_name.to_snake_case()
                    )
                    .to_string(),
                    content_template: include_str!(
                        "../templates/programs/src/state/mod.rs.template"
                    )
                    .to_string(),
                },
                TemplateFile {
                    path_template: format!(
                        "programs/{}/src/state/counter.rs",
                        program_name.to_snake_case()
                    )
                    .to_string(),
                    content_template: include_str!(
                        "../templates/programs/src/state/counter.rs.template"
                    )
                    .to_string(),
                },
                TemplateFile {
                    path_template: format!(
                        "programs/{}/src/handlers/mod.rs",
                        program_name.to_snake_case()
                    )
                    .to_string(),
                    content_template: include_str!(
                        "../templates/programs/src/handlers/mod.rs.template"
                    )
                    .to_string(),
                },
                TemplateFile {
                    path_template: format!(
                        "programs/{}/src/handlers/initialize.rs",
                        program_name.to_snake_case()
                    )
                    .to_string(),
                    content_template: include_str!(
                        "../templates/programs/src/handlers/initialize.rs.template"
                    )
                    .to_string(),
                },
                TemplateFile {
                    path_template: format!(
                        "programs/{}/src/handlers/close.rs",
                        program_name.to_snake_case()
                    )
                    .to_string(),
                    content_template: include_str!(
                        "../templates/programs/src/handlers/close.rs.template"
                    )
                    .to_string(),
                },
                TemplateFile {
                    path_template: format!(
                        "programs/{}/src/handlers/increment.rs",
                        program_name.to_snake_case()
                    )
                    .to_string(),
                    content_template: include_str!(
                        "../templates/programs/src/handlers/increment.rs.template"
                    )
                    .to_string(),
                },
                TemplateFile {
                    path_template: format!(
                        "programs/{}/src/contexts/mod.rs",
                        program_name.to_snake_case()
                    )
                    .to_string(),
                    content_template: include_str!(
                        "../templates/programs/src/contexts/mod.rs.template"
                    )
                    .to_string(),
                },
                TemplateFile {
                    path_template: format!("tests/tests/{}.rs", program_name.to_snake_case())
                        .to_string(),
                    content_template: include_str!(
                        "../templates/tests/tests/integration.rs.template"
                    )
                    .to_string(),
                },
            ],
        }
    }

    fn handler(program_name: &str, instruction_name: &str) -> Self {
        Self {
            name: "handler".to_string(),
            files: vec![
                TemplateFile {
                    path_template: format!(
                        "programs/{}/src/handlers/{}.rs",
                        program_name.to_snake_case(),
                        instruction_name.to_snake_case()
                    )
                    .to_string(),
                    content_template: include_str!(
                        "../templates/programs/src/handlers/generic.rs.template"
                    )
                    .to_string(),
                },
                TemplateFile {
                    path_template: format!(
                        "programs/{}/src/contexts/{}.rs",
                        program_name.to_snake_case(),
                        instruction_name.to_snake_case()
                    )
                    .to_string(),
                    content_template: include_str!(
                        "../templates/programs/src/contexts/generic.rs.template"
                    )
                    .to_string(),
                },
            ],
        }
    }
}
