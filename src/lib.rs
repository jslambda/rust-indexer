use std::fs;
use std::io::Write;
use std::path::Path;

use proc_macro2::Span;
use quote::ToTokens;
use serde::Serialize;
use syn::spanned::Spanned;
use syn::{Attribute, Expr, ExprLit, File, Item, Lit, Meta};
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
pub struct IndexEntry {
    pub kind: String,
    pub name: String,
    pub file: String,
    pub line_start: u32,
    pub line_end: u32,
    pub signature: String,
    pub doc_summary: Option<String>,
    pub doc: Option<String>,
}

pub fn build_index(project_root: &Path) -> Result<Vec<IndexEntry>, Box<dyn std::error::Error>> {
    let src_root = project_root.join("src");
    if !src_root.is_dir() {
        return Err(format!("{} is not a directory", src_root.display()).into());
    }

    let mut entries = Vec::new();

    for entry in WalkDir::new(&src_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_type().is_file() && e.path().extension().map(|ext| ext == "rs").unwrap_or(false)
        })
    {
        let path = entry.into_path();
        let relative = path
            .strip_prefix(project_root)
            .unwrap_or(&path)
            .to_path_buf();

        let content = fs::read_to_string(&path)?;
        let parsed: File = syn::parse_file(&content)?;

        let mut file_entries = index_file(&relative, &parsed);
        entries.append(&mut file_entries);
    }

    Ok(entries)
}

pub fn write_index_to<W: Write>(
    entries: &[IndexEntry],
    mut writer: W,
) -> Result<(), Box<dyn std::error::Error>> {
    serde_json::to_writer_pretty(&mut writer, entries)?;
    writer.flush()?;
    Ok(())
}

fn index_file(path: &Path, parsed: &File) -> Vec<IndexEntry> {
    let mut entries = Vec::new();

    for item in &parsed.items {
        match item {
            Item::Mod(item) => entries.push(build_entry(
                "module",
                &item.ident.to_string(),
                path,
                item.span(),
                &item.attrs,
                module_signature(item),
            )),
            Item::Struct(item) => entries.push(build_entry(
                "struct",
                &item.ident.to_string(),
                path,
                item.span(),
                &item.attrs,
                type_signature(&item.ident.to_string(), &item.generics),
            )),
            Item::Enum(item) => entries.push(build_entry(
                "enum",
                &item.ident.to_string(),
                path,
                item.span(),
                &item.attrs,
                type_signature(&item.ident.to_string(), &item.generics),
            )),
            Item::Trait(item) => entries.push(build_entry(
                "trait",
                &item.ident.to_string(),
                path,
                item.span(),
                &item.attrs,
                trait_signature(item),
            )),
            Item::Fn(item) => entries.push(build_entry(
                "fn",
                &item.sig.ident.to_string(),
                path,
                item.span(),
                &item.attrs,
                item.sig.to_token_stream().to_string(),
            )),
            Item::Impl(item) => entries.push(build_entry(
                "impl",
                &impl_name(item),
                path,
                item.span(),
                &item.attrs,
                impl_signature(item),
            )),
            _ => {}
        }
    }

    entries
}

fn build_entry(
    kind: &str,
    name: &str,
    file: &Path,
    span: Span,
    attrs: &[Attribute],
    signature: String,
) -> IndexEntry {
    let (doc_summary, doc) = extract_docs(attrs);
    let (line_start, line_end) = line_range(span);

    IndexEntry {
        kind: kind.to_string(),
        name: name.to_string(),
        file: file.to_string_lossy().into_owned(),
        line_start,
        line_end,
        signature,
        doc_summary,
        doc,
    }
}

fn line_range(span: Span) -> (u32, u32) {
    let start = span.start();
    let end = span.end();
    (start.line as u32, end.line as u32)
}

fn extract_docs(attrs: &[Attribute]) -> (Option<String>, Option<String>) {
    let mut docs = Vec::new();

    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }

        if let Meta::NameValue(meta) = &attr.meta {
            if let Expr::Lit(ExprLit {
                lit: Lit::Str(lit), ..
            }) = &meta.value
            {
                docs.push(lit.value());
            }
        }
    }

    if docs.is_empty() {
        return (None, None);
    }

    let doc = docs.join("\n");
    let doc_summary = docs.iter().find_map(|line| {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

    (doc_summary, Some(doc))
}

fn type_signature(name: &str, generics: &syn::Generics) -> String {
    if generics.params.is_empty() {
        name.to_string()
    } else {
        format!("{} {}", name, generics.to_token_stream())
    }
}

fn trait_signature(item: &syn::ItemTrait) -> String {
    if item.generics.params.is_empty() {
        format!("trait {}", item.ident)
    } else {
        format!("trait {} {}", item.ident, item.generics.to_token_stream())
    }
}

fn module_signature(item: &syn::ItemMod) -> String {
    if item.content.is_some() {
        format!("mod {}", item.ident)
    } else {
        format!("mod {};", item.ident)
    }
}

fn impl_signature(item: &syn::ItemImpl) -> String {
    let self_ty = item.self_ty.to_token_stream();
    let mut parts = vec!["impl".to_string()];

    if !item.generics.params.is_empty() {
        parts.push(item.generics.to_token_stream().to_string());
    }

    if let Some((_, path, _)) = &item.trait_ {
        parts.push(path.to_token_stream().to_string());
        parts.push("for".to_string());
    }

    parts.push(self_ty.to_string());

    parts.join(" ")
}

fn impl_name(item: &syn::ItemImpl) -> String {
    if let Some((_, path, _)) = &item.trait_ {
        format!(
            "{} for {}",
            path.to_token_stream(),
            item.self_ty.to_token_stream()
        )
    } else {
        item.self_ty.to_token_stream().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_docs() {
        let attrs: Vec<Attribute> = syn::parse_quote! {
            #[doc = " Line one "]
            #[doc = ""]
            #[doc = "Line two"]
        };

        let (summary, doc) = extract_docs(&attrs);
        assert_eq!(summary.as_deref(), Some("Line one"));
        assert_eq!(doc.as_deref(), Some(" Line one \n\nLine two"));
    }
}
