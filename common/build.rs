use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

fn to_pascal_case(s: &str) -> String {
    s.split(['_', '-'])
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + &chars.as_str().to_lowercase(),
                None => String::new(),
            }
        })
        .collect()
}

fn discover_languages(i18n_dir: &Path) -> Vec<String> {
    let mut languages = Vec::new();
    for entry in fs::read_dir(i18n_dir).expect("Could not read shared/i18n/ directory") {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "yaml") {
            let stem = path.file_stem().unwrap().to_str().unwrap();
            if stem != "config" {
                languages.push(stem.to_string());
            }
        }
    }
    languages.sort();
    languages
}

fn read_i18n_config(i18n_dir: &Path) -> String {
    let config_path = i18n_dir.join("config.yaml");
    let content = fs::read_to_string(&config_path).expect("Could not read i18n/config.yaml");
    let config: HashMap<String, String> =
        serde_yml::from_str(&content).expect("Could not parse i18n/config.yaml");
    config
        .get("default")
        .expect("Missing 'default' in i18n/config.yaml")
        .clone()
}

fn read_translation_files(i18n_dir: &Path) -> Vec<(String, HashMap<String, String>)> {
    let mut files = Vec::new();
    for entry in fs::read_dir(i18n_dir).expect("Could not read shared/i18n/ directory") {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "yaml") {
            let stem = path.file_stem().unwrap().to_str().unwrap();
            if stem == "config" {
                continue;
            }
            let content = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("Could not read {}", path.display()));
            let translations: HashMap<String, String> = serde_yml::from_str(&content)
                .unwrap_or_else(|_| panic!("Could not parse {}", path.display()));
            files.push((stem.to_string(), translations));
        }
    }
    files
}

fn validate_i18n(codes: &[String], i18n_dir: &Path) {
    let files = read_translation_files(i18n_dir);

    for (file_name, translations) in &files {
        for code in codes {
            if !translations.contains_key(code) {
                panic!("Missing translation for error code '{code}' in {file_name}.yaml");
            }
        }
    }

    if files.len() < 2 {
        return;
    }

    let (first_name, first_keys) = &files[0];
    let first_keys: std::collections::HashSet<&String> = first_keys.keys().collect();

    for (other_name, other_translations) in &files[1..] {
        let other_keys: std::collections::HashSet<&String> = other_translations.keys().collect();

        for key in &first_keys {
            if !other_keys.contains(key) {
                panic!(
                    "Key '{key}' exists in {first_name}.yaml but is missing in {other_name}.yaml"
                );
            }
        }

        for key in &other_keys {
            if !first_keys.contains(key) {
                panic!(
                    "Key '{key}' exists in {other_name}.yaml but is missing in {first_name}.yaml"
                );
            }
        }
    }
}

fn generate_language_enum(languages: &[String], default_language: &str, out_dir: &Path) {
    let variants: String = languages
        .iter()
        .map(|lang| {
            let variant = to_pascal_case(lang);
            if lang == default_language {
                format!("    #[default]\n    {variant},\n")
            } else {
                format!("    {variant},\n")
            }
        })
        .collect();

    let display_arms: String = languages
        .iter()
        .map(|lang| {
            let variant = to_pascal_case(lang);
            let short = lang.split('-').next().unwrap().to_uppercase();
            format!("            Language::{variant} => \"{short}\",\n")
        })
        .collect();

    let all_variants: String = languages
        .iter()
        .map(|lang| format!("Language::{}", to_pascal_case(lang)))
        .collect::<Vec<_>>()
        .join(", ");

    let generated = format!(
        "#[derive(Default, Clone, Copy, PartialEq)]\npub enum Language {{\n{variants}}}\n\n\
         impl Language {{\n\
         \x20   pub fn display_name(&self) -> &'static str {{\n\
         \x20       match self {{\n\
         {display_arms}\
         \x20       }}\n\
         \x20   }}\n\n\
         \x20   pub fn all() -> &'static [Language] {{\n\
         \x20       &[{all_variants}]\n\
         \x20   }}\n\
         }}\n"
    );

    let dest = out_dir.join("language_generated.rs");
    fs::write(dest, generated).unwrap();
}

fn generate_i18n_enum(i18n_dir: &Path, out_dir: &Path) {
    let files = read_translation_files(i18n_dir);
    let keys = &files[0].1;
    let mut sorted_keys: Vec<&String> = keys.keys().collect();
    sorted_keys.sort();

    let variants: String = sorted_keys
        .iter()
        .map(|key| format!("    {},\n", to_pascal_case(key)))
        .collect();

    let match_arms: String = files
        .iter()
        .map(|(lang, translations)| {
            let lang_variant = to_pascal_case(lang);
            let arms: String = sorted_keys
                .iter()
                .map(|key| {
                    let variant = to_pascal_case(key);
                    let text = translations.get(*key).unwrap();
                    format!("                I18n::{variant} => \"{text}\",\n")
                })
                .collect();
            format!(
                "            Language::{lang_variant} => match self {{\n{arms}            }},\n"
            )
        })
        .collect();

    let generated = format!(
        "pub enum I18n {{\n{variants}}}\n\n\
         impl I18n {{\n\
         \x20   pub fn translate(&self, language: super::Language) -> &'static str {{\n\
         \x20       use super::Language;\n\
         \x20       match language {{\n\
         {match_arms}\
         \x20       }}\n\
         \x20   }}\n\
         }}\n"
    );
    fs::write(out_dir.join("i18n_generated.rs"), generated).unwrap();
}

fn generate_resource_enum(openapi_path: &Path, out_dir: &Path) {
    let content = fs::read_to_string(openapi_path).expect("Could not read openapi.yaml");
    let spec: serde_yml::Value =
        serde_yml::from_str(&content).expect("Could not parse openapi.yaml");

    let paths = spec["paths"]
        .as_mapping()
        .expect("Missing 'paths' in openapi.yaml");

    let mut resources: Vec<(String, String)> = paths
        .keys()
        .map(|path| {
            let path = path.as_str().unwrap();
            let variant = path
                .split('/')
                .filter(|s| !s.is_empty())
                .map(to_pascal_case)
                .collect::<String>();
            (variant, path.to_string())
        })
        .collect();

    resources.sort_by(|a, b| a.0.cmp(&b.0));

    let variants: String = resources
        .iter()
        .map(|(variant, _)| format!("    {variant},\n"))
        .collect();

    let path_arms: String = resources
        .iter()
        .map(|(variant, path)| format!("            Resource::{variant} => \"{path}\",\n"))
        .collect();

    let generated = format!(
        "#[derive(Debug, Clone, Copy, PartialEq)]\n\
         pub enum Resource {{\n{variants}}}\n\n\
         impl Resource {{\n\
         \x20   pub fn path(&self) -> &'static str {{\n\
         \x20       match self {{\n\
         {path_arms}\
         \x20       }}\n\
         \x20   }}\n\
         }}\n"
    );

    fs::write(out_dir.join("resource.rs"), generated).unwrap();
}

fn collect_env_vars(
    value: &serde_yml::Value,
    prefix: &str,
    result: &mut Vec<(String, String, String)>,
) {
    let mapping = value
        .as_mapping()
        .expect("Expected mapping in env vars yaml");
    for (key, val) in mapping {
        let key = key.as_str().unwrap();
        let path = if prefix.is_empty() {
            key.to_string()
        } else {
            format!("{prefix}_{key}")
        };

        if let Some(inner) = val.as_mapping() {
            let env_key = serde_yml::Value::String("env".to_string());
            if inner.contains_key(&env_key) {
                let default_key = serde_yml::Value::String("default".to_string());
                let env_name = inner[&env_key].as_str().unwrap();
                let default = inner[&default_key].as_str().unwrap();
                result.push((
                    to_pascal_case(&path.to_lowercase()),
                    env_name.to_string(),
                    default.to_string(),
                ));
            } else {
                collect_env_vars(val, &path, result);
            }
        }
    }
}

fn generate_env_var_enum(env_vars_path: &Path, out_dir: &Path) {
    let content = fs::read_to_string(env_vars_path)
        .expect("Could not read environment_variables_defaults.yaml");
    let spec: serde_yml::Value =
        serde_yml::from_str(&content).expect("Could not parse environment_variables_defaults.yaml");

    let mut vars = Vec::new();
    collect_env_vars(&spec, "", &mut vars);
    vars.sort_by(|a, b| a.0.cmp(&b.0));

    let variants: String = vars
        .iter()
        .map(|(variant, _, _)| format!("    {variant},\n"))
        .collect();

    let env_arms: String = vars
        .iter()
        .map(|(variant, env_name, _)| format!("            EnvVar::{variant} => \"{env_name}\",\n"))
        .collect();

    let default_arms: String = vars
        .iter()
        .map(|(variant, _, default)| format!("            EnvVar::{variant} => \"{default}\",\n"))
        .collect();

    let generated = format!(
        "#[derive(Debug, Clone, Copy, PartialEq)]\n\
         pub enum EnvVar {{\n{variants}}}\n\n\
         impl EnvVar {{\n\
         \x20   pub fn env_name(&self) -> &'static str {{\n\
         \x20       match self {{\n\
         {env_arms}\
         \x20       }}\n\
         \x20   }}\n\n\
         \x20   pub fn default_value(&self) -> &'static str {{\n\
         \x20       match self {{\n\
         {default_arms}\
         \x20       }}\n\
         \x20   }}\n\n\
         \x20   pub fn value(&self) -> String {{\n\
         \x20       std::env::var(self.env_name()).unwrap_or_else(|_| self.default_value().to_string())\n\
         \x20   }}\n\
         }}\n"
    );

    fs::write(out_dir.join("env_var.rs"), generated).unwrap();
}

fn extract_template_params(template: &str) -> Vec<String> {
    let mut params = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        if let Some(end) = rest[start..].find('}') {
            params.push(rest[start + 1..start + end].to_string());
            rest = &rest[start + end + 1..];
        } else {
            break;
        }
    }
    params
}

fn generate_error_enum(codes: &[String], i18n_dir: &Path, out_dir: &Path) {
    let files = read_translation_files(i18n_dir);
    let translations = &files[0].1;

    let mut structs = String::new();
    let mut enum_variants = String::new();
    let mut error_trait_impls = String::new();
    let mut match_key_arms = String::new();
    let mut match_params_arms = String::new();

    for code in codes {
        let variant = to_pascal_case(code);
        let struct_name = format!("{variant}Error");
        let template = translations.get(code).unwrap();
        let params = extract_template_params(template);

        // Enum variant
        enum_variants.push_str(&format!("    {variant}({struct_name}),\n"));

        // Struct
        if params.is_empty() {
            structs.push_str(&format!(
                "#[derive(Debug, Clone)]\npub struct {struct_name} {{\n    params: std::collections::HashMap<String, String>,\n}}\n\n\
                 impl {struct_name} {{\n\
                 \x20   pub fn new() -> Self {{\n\
                 \x20       {struct_name} {{ params: std::collections::HashMap::new() }}\n\
                 \x20   }}\n\
                 }}\n\n\
                 impl Default for {struct_name} {{\n\
                 \x20   fn default() -> Self {{\n\
                 \x20       Self::new()\n\
                 \x20   }}\n\
                 }}\n\n"
            ));
        } else {
            let constructor_args: String = params
                .iter()
                .map(|p| format!("{p}: String"))
                .collect::<Vec<_>>()
                .join(", ");
            let hashmap_entries: String = params
                .iter()
                .map(|p| format!("(\"{p}\".to_string(), {p})"))
                .collect::<Vec<_>>()
                .join(", ");
            structs.push_str(&format!(
                "#[derive(Debug, Clone)]\npub struct {struct_name} {{\n    params: std::collections::HashMap<String, String>,\n}}\n\n\
                 impl {struct_name} {{\n\
                 \x20   pub fn new({constructor_args}) -> Self {{\n\
                 \x20       let params = std::collections::HashMap::from([{hashmap_entries}]);\n\
                 \x20       {struct_name} {{ params }}\n\
                 \x20   }}\n\
                 }}\n\n"
            ));
        }

        // ErrorTrait impl for struct
        error_trait_impls.push_str(&format!(
            "impl ErrorTrait for {struct_name} {{\n\
             \x20   fn key(&self) -> super::ErrorCode {{\n\
             \x20       super::ErrorCode::{variant}\n\
             \x20   }}\n\
             \x20   fn params(&self) -> &std::collections::HashMap<String, String> {{\n\
             \x20       &self.params\n\
             \x20   }}\n\
             }}\n\n"
        ));

        // Match arms for Error impl
        match_key_arms.push_str(&format!("            Error::{variant}(e) => e.key(),\n"));
        match_params_arms.push_str(&format!("            Error::{variant}(e) => e.params(),\n"));
    }

    let generated = format!(
        "{structs}\
         {error_trait_impls}\
         #[derive(Debug, Clone)]\n\
         pub enum Error {{\n{enum_variants}}}\n\n\
         impl ErrorTrait for Error {{\n\
         \x20   fn key(&self) -> super::ErrorCode {{\n\
         \x20       match self {{\n\
         {match_key_arms}\
         \x20       }}\n\
         \x20   }}\n\n\
         \x20   fn params(&self) -> &std::collections::HashMap<String, String> {{\n\
         \x20       match self {{\n\
         {match_params_arms}\
         \x20       }}\n\
         \x20   }}\n\
         }}\n"
    );

    fs::write(out_dir.join("error.rs"), generated).unwrap();
}

fn main() {
    let yaml_path = "../shared/error-codes.yaml";
    let i18n_dir = Path::new("../shared/i18n");
    let openapi_path = Path::new("../shared/openapi.yaml");
    let env_vars_path = Path::new("../shared/environment_variables_defaults.yaml");
    println!("cargo::rerun-if-changed={yaml_path}");
    println!("cargo::rerun-if-changed={}", i18n_dir.display());
    println!("cargo::rerun-if-changed={}", openapi_path.display());
    println!("cargo::rerun-if-changed={}", env_vars_path.display());

    let yaml = fs::read_to_string(yaml_path).expect("Could not read error-codes.yaml");
    let codes: Vec<String> = serde_yml::from_str(&yaml).expect("Could not parse error-codes.yaml");

    let languages = discover_languages(i18n_dir);
    let default_language = read_i18n_config(i18n_dir);

    if !languages.iter().any(|l| l == &default_language) {
        panic!("Default language '{default_language}' has no YAML file in shared/i18n/");
    }

    validate_i18n(&codes, i18n_dir);

    let out_dir_str = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir_str);

    let error_variants: Vec<String> = codes.iter().map(|code| to_pascal_case(code)).collect();

    let variants: String = error_variants
        .iter()
        .map(|v| format!("    {v},\n"))
        .collect();

    let from_arms: String = error_variants
        .iter()
        .map(|v| format!("            ErrorCode::{v} => super::i18n::I18n::{v},\n"))
        .collect();

    let generated = format!(
        "use serde::{{Serialize, Deserialize}};\n\n\
         #[derive(Debug, PartialEq, Serialize, Deserialize)]\n\
         pub enum ErrorCode {{\n{variants}}}\n\n\
         impl From<ErrorCode> for super::i18n::I18n {{\n\
         \x20   fn from(code: ErrorCode) -> Self {{\n\
         \x20       match code {{\n\
         {from_arms}\
         \x20       }}\n\
         \x20   }}\n\
         }}\n"
    );
    fs::write(out_dir.join("error_code_generated.rs"), generated).unwrap();

    generate_language_enum(&languages, &default_language, out_dir);
    generate_i18n_enum(i18n_dir, out_dir);
    generate_resource_enum(openapi_path, out_dir);
    generate_env_var_enum(env_vars_path, out_dir);
    generate_error_enum(&codes, i18n_dir, out_dir);
}
