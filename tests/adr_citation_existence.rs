use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

const CITATION_SITE_FLOOR: usize = 46;

const LIVE_ADR_FLOOR: usize = 27;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("source directory is readable") {
        let path = entry.expect("readable directory entry").path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

fn markdown_files(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(dir)
        .expect("ADR directory is readable")
        .map(|entry| entry.expect("readable directory entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .collect();
    out.sort();
    out
}

struct Corpus {
    live: BTreeMap<String, BTreeSet<u32>>,
    stale: BTreeSet<String>,
}

impl Corpus {
    fn load() -> Self {
        let root = manifest_dir().join("docs").join("adr");
        let id_re = Regex::new(r"^(AFM-\d{4})").expect("ADR filename pattern compiles");
        let rule_re = Regex::new(r"(?m)^R(\d+)\b").expect("corpus rule-line pattern compiles");

        let mut live = BTreeMap::new();
        for path in markdown_files(&root.join("adr-fmt")) {
            let name = path
                .file_name()
                .expect("ADR file has a name")
                .to_string_lossy()
                .into_owned();
            let Some(id) = id_re.captures(&name) else {
                continue;
            };
            let text = fs::read_to_string(&path).expect("ADR file is readable");
            let rules = rule_re
                .captures_iter(&text)
                .map(|caps| caps[1].parse::<u32>().expect("rule digits fit a u32"))
                .collect();
            live.insert(id[1].to_owned(), rules);
        }

        let mut stale = BTreeSet::new();
        for path in markdown_files(&root.join("stale")) {
            let name = path
                .file_name()
                .expect("ADR file has a name")
                .to_string_lossy()
                .into_owned();
            if let Some(id) = id_re.captures(&name) {
                stale.insert(id[1].to_owned());
            }
        }

        Self { live, stale }
    }
}

struct Citation {
    site: String,
    adr: String,
    rules: BTreeSet<u32>,
    text: String,
}

struct DocScan<'a> {
    site: &'a str,
    adr_re: &'a Regex,
    tail_re: &'a Regex,
    more_re: &'a Regex,
    out: &'a mut Vec<Citation>,
}

fn doc_literal(site: &str, attr: &syn::Attribute) -> String {
    let syn::Meta::NameValue(pair) = &attr.meta else {
        panic!(
            "{site}: a `doc` attribute is not of the form `#[doc = \"literal\"]`. This guard \
             reads citations out of doc-attribute string literals only, so any other shape is \
             text it cannot see. AFM-0037:R4 forbids passing on unreadable input"
        )
    };
    let syn::Expr::Lit(lit) = &pair.value else {
        panic!(
            "{site}: a `doc` attribute's value is not a literal (for example \
             `#[doc = concat!(..)]`). Its text is produced after this guard runs, so the guard \
             cannot read a citation out of it. AFM-0037:R4 forbids passing on unreadable input"
        )
    };
    let syn::Lit::Str(text) = &lit.lit else {
        panic!(
            "{site}: a `doc` attribute's value is a literal but not a string literal, so there \
             is no documentation text for this guard to read. AFM-0037:R4 forbids passing on \
             unreadable input"
        )
    };
    text.value()
}

impl<'ast> Visit<'ast> for DocScan<'_> {
    fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
        if attr.path().is_ident("doc") {
            let text = doc_literal(self.site, attr);
            for caps in self.adr_re.captures_iter(&text) {
                let whole = caps.get(0).expect("match zero always exists");
                let mut rules = BTreeSet::new();
                if let Some(tail) = self.tail_re.captures(&text[whole.end()..]) {
                    let first: u32 = tail[1].parse().expect("rule digits fit a u32");
                    rules.insert(first);
                    let mut previous = first;
                    for extra in self.more_re.captures_iter(&tail[2]) {
                        let next: u32 = extra[2].parse().expect("rule digits fit a u32");
                        if matches!(&extra[1], "-" | "\u{2013}") {
                            rules.extend(previous.min(next)..=previous.max(next));
                        } else {
                            rules.insert(next);
                        }
                        previous = next;
                    }
                }
                self.out.push(Citation {
                    site: self.site.to_owned(),
                    adr: caps[1].to_owned(),
                    rules,
                    text: text.trim().to_owned(),
                });
            }
        }
        syn::visit::visit_attribute(self, attr);
    }
}

struct Scan {
    citations: Vec<Citation>,
    src_files: usize,
    test_files: usize,
}

impl Scan {
    fn run() -> Self {
        let root = manifest_dir();
        let adr_re = Regex::new(r"(AFM-\d{4})").expect("citation pattern compiles");
        let tail_re = Regex::new(r"^[: ]R(\d+)((?:\s*(?:[-\x{2013}/]|and)\s*R?\d+)*)")
            .expect("rule-suffix pattern compiles");
        let more_re =
            Regex::new(r"\s*([-\x{2013}/]|and)\s*R?(\d+)").expect("multi-rule pattern compiles");

        let mut citations = Vec::new();
        let mut counts = [0usize; 2];
        for (slot, tree) in ["src", "tests"].into_iter().enumerate() {
            let mut files = Vec::new();
            rust_sources(&root.join(tree), &mut files);
            files.sort();
            counts[slot] = files.len();
            for file in &files {
                let site = file
                    .strip_prefix(&root)
                    .unwrap_or(file)
                    .display()
                    .to_string();
                let text = fs::read_to_string(file).expect("source file is readable");
                let ast = syn::parse_file(&text)
                    .unwrap_or_else(|e| panic!("{site}: source does not parse: {e}"));
                DocScan {
                    site: &site,
                    adr_re: &adr_re,
                    tail_re: &tail_re,
                    more_re: &more_re,
                    out: &mut citations,
                }
                .visit_file(&ast);
            }
        }

        Self {
            citations,
            src_files: counts[0],
            test_files: counts[1],
        }
    }
}

#[test]
fn doc_attribute_citations_resolve_to_a_live_corpus_rule() {
    let scan = Scan::run();
    let corpus = Corpus::load();

    assert!(
        scan.src_files > 0 && scan.test_files > 0,
        "parsed {} file(s) under src/ and {} under tests/. A tree that contributes no files \
         contributes no citations either, so this guard would report a clean corpus without \
         having looked at one",
        scan.src_files,
        scan.test_files
    );
    assert!(
        corpus.live.len() >= LIVE_ADR_FLOOR,
        "loaded only {} live ADR(s) from docs/adr/adr-fmt/. Every citation resolves against \
         this map, so an empty or truncated one turns real violations into passes",
        corpus.live.len()
    );
    assert!(
        !corpus.stale.is_empty(),
        "loaded no ADRs from docs/adr/stale/. Without them a citation of a retired ADR is \
         reported as merely absent, losing the AFM-0022 distinction this guard draws"
    );
    assert!(
        scan.citations.len() >= CITATION_SITE_FLOOR,
        "extracted only {} doc-attribute citation(s); the floor is {CITATION_SITE_FLOOR}, the \
         count measured on this corpus after removing the unsupported AFM-0021 claim from \
         src/output.rs. Extraction that silently stops matching is indistinguishable from a \
         corpus with nothing to check, so a shortfall is a broken guard and not a clean tree",
        scan.citations.len()
    );

    let mut findings = Vec::new();
    for citation in &scan.citations {
        let Some(rules) = corpus.live.get(&citation.adr) else {
            if corpus.stale.contains(&citation.adr) {
                findings.push(format!(
                    "{}: cites {}, which is retired to docs/adr/stale/. Per AFM-0022 a stale \
                     ADR is a non-authoritative pointer and carries no binding rule: {}",
                    citation.site, citation.adr, citation.text
                ));
            } else {
                findings.push(format!(
                    "{}: cites {}, which is no ADR in this corpus, live or stale: {}",
                    citation.site, citation.adr, citation.text
                ));
            }
            continue;
        };
        for rule in &citation.rules {
            if !rules.contains(rule) {
                findings.push(format!(
                    "{}: cites {}:R{rule}, but {} declares no rule R{rule}; it declares {:?}: {}",
                    citation.site, citation.adr, citation.adr, rules, citation.text
                ));
            }
        }
    }

    assert!(
        findings.is_empty(),
        "doc-attribute citations that do not resolve to a live corpus entry:\n{}",
        findings.join("\n")
    );
}
