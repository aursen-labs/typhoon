use {
    crate::{templates::Template, utils::is_valid_name},
    heck::{ToKebabCase, ToSnakeCase},
    std::{
        fs,
        path::{Path, PathBuf},
    },
    toml_edit::{DocumentMut, Item},
};

pub fn program(project_dir: Option<PathBuf>, program: &str) -> anyhow::Result<()> {
    // Check if project directory already exists
    let project_dir = project_dir.unwrap_or_else(|| PathBuf::from("."));
    if !project_dir.exists() {
        anyhow::bail!("No Typhoon workspace found at '{}'", project_dir.display());
    }

    // Check if program already exists
    let program_dir = project_dir.join("programs").join(program.to_snake_case());
    if program_dir.exists() {
        anyhow::bail!(
            "Program '{}' already exists in the Typhoon workspace",
            program
        );
    }

    // Validate program name
    if !is_valid_name(program) {
        anyhow::bail!("Program name '{}' is not a valid name", program);
    }

    println!("Adding program {} to the Typhoon workspace...", program);
    println!("Location: {}", project_dir.display());

    // Generate program files
    Template::generate_program(&project_dir, program)?;

    // Insert dependencies and metadata into Cargo.toml files
    insert_toml_values(
        &project_dir.join("Cargo.toml"),
        &[(
            format!("workspace.dependencies.{}.path", program.to_kebab_case()).as_str(),
            format!("programs/{}", program.to_snake_case()).into(),
        )],
    )?;
    insert_toml_values(
        &project_dir.join("tests").join("Cargo.toml"),
        &[
            (
                format!(
                    "package.metadata.typhoon.builder-dependencies.{}.path",
                    program.to_snake_case()
                )
                .as_str(),
                format!("../programs/{}", program.to_snake_case()).into(),
            ),
            (
                format!("dev-dependencies.{}.workspace", program.to_kebab_case()).as_str(),
                true.into(),
            ),
        ],
    )?;

    println!("\n✅ Program added successfully!");

    Ok(())
}

pub fn handler(path: Option<PathBuf>, program: &str, instruction: &str) -> anyhow::Result<()> {
    let project_dir = path.unwrap_or_else(|| PathBuf::from("."));
    if !project_dir.exists() {
        anyhow::bail!("No Typhoon workspace found at '{}'", project_dir.display());
    }

    // Validate instruction name
    if !is_valid_name(instruction) {
        anyhow::bail!("Instruction name '{}' is not a valid name", instruction);
    }

    // Handlers live at `programs/<program>/src/handlers/<instruction>.rs`.
    let handler_path = project_dir
        .join("programs")
        .join(program.to_snake_case())
        .join("src")
        .join("handlers")
        .join(format!("{}.rs", instruction.to_snake_case()));
    if handler_path.exists() {
        anyhow::bail!(
            "Handler '{}' already exists in program '{}'",
            instruction,
            program
        );
    }

    // Generate handler files
    Template::generate_handler(&project_dir, program, instruction)?;

    // Insert dependencies
    update_mod_file(
        &project_dir
            .join("programs")
            .join(program.to_snake_case())
            .join("src")
            .join("contexts")
            .join("mod.rs"),
        &instruction.to_snake_case(),
    )?;
    update_mod_file(
        &project_dir
            .join("programs")
            .join(program.to_snake_case())
            .join("src")
            .join("handlers")
            .join("mod.rs"),
        &instruction.to_snake_case(),
    )?;

    // Add instruction to router
    let lib_path = project_dir
        .join("programs")
        .join(program.to_snake_case())
        .join("src")
        .join("lib.rs");
    let lib_content = fs::read_to_string(lib_path.clone())?;
    let mut lib_lines: Vec<String> = lib_content.lines().map(str::to_owned).collect();
    // Find the router and add the new instruction
    let router_line = lib_lines
        .iter()
        .position(|line| {
            line.trim_start()
                .starts_with("pub const ROUTER: EntryFn = basic_router! {")
        })
        .ok_or_else(|| anyhow::anyhow!("Router not found in lib.rs"))?;
    let router_end_line = router_line
        + lib_lines[router_line..]
            .iter()
            .position(|line| line.trim_start().starts_with("};"))
            .ok_or_else(|| anyhow::anyhow!("Router end not found in lib.rs"))?;
    let router_instr_regex = regex::Regex::new(r"^\s*(\d+)\s*=>\s*([\w_]+),").unwrap();
    let new_target = instruction.to_snake_case();
    let mut last_index = -1;
    for line in &lib_lines[router_line + 1..router_end_line] {
        if let Some(cap) = router_instr_regex.captures(line) {
            // Guard against a second, duplicate router entry for the same handler.
            if &cap[2] == new_target.as_str() {
                anyhow::bail!(
                    "Instruction '{}' is already registered in the router of program '{}'",
                    instruction,
                    program
                );
            }
            if let Ok(idx) = cap[1].parse::<i32>() {
                if idx > last_index {
                    last_index = idx;
                }
            }
        }
    }
    let next_index = last_index + 1;
    let new_instruction = format!("    {} => {},", next_index, instruction.to_snake_case());
    lib_lines.insert(router_end_line, new_instruction);
    fs::write(&lib_path, lib_lines.join("\n"))?;
    println!("\n✅ Handler added successfully!");

    Ok(())
}

fn insert_toml_values(
    toml_path: &Path,
    table_value_pairs: &[(&str, toml_edit::Value)],
) -> anyhow::Result<()> {
    fn set_path(item: &mut Item, path: &str, val: toml_edit::Value) -> anyhow::Result<()> {
        let mut current = item;

        for key in path.split('.') {
            current = current
                .get_mut(key)
                .ok_or_else(|| anyhow::anyhow!("Path '{}' not found in TOML file", path))?;
        }

        *current = Item::Value(val.clone());
        Ok(())
    }

    let contents = fs::read_to_string(toml_path)?;
    let mut doc = contents.parse::<DocumentMut>()?;
    for (key, val) in table_value_pairs {
        set_path(doc.as_item_mut(), key, val.clone())?;
    }
    fs::write(toml_path, doc.to_string())?;
    Ok(())
}

fn update_mod_file(mod_path: &Path, mod_name: &str) -> anyhow::Result<()> {
    let mod_content = fs::read_to_string(mod_path)?;

    if mod_content.contains(&format!("mod {};", mod_name.to_snake_case())) {
        return Ok(()); // Already exists
    }

    let mut mod_doc = mod_content.split("\n").collect::<Vec<&str>>();
    let new_line = format!("mod {};", mod_name.to_snake_case());
    mod_doc.insert(0, new_line.as_str());

    // Find last 'use' line and insert 'pub use *' after it
    let last_use_idx = mod_doc
        .iter()
        .rposition(|line| line.trim_start().starts_with("use "));
    let use_line = format!("pub use {}::*;", mod_name.to_snake_case());
    if let Some(idx) = last_use_idx {
        mod_doc.insert(idx + 1, &use_line);
    } else {
        // if no use found, insert after the first line
        mod_doc.insert(1, &use_line);
    }
    fs::write(mod_path, mod_doc.join("\n"))?;

    Ok(())
}
