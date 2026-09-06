use std::fs;
use syn::visit::Visit;

mod common;
use common::{
    Citation, Corpus, Patterns, extract_citations, manifest_dir, rust_sources,
    unresolved_citation_findings,
};

const CITATION_SITE_FLOOR: usize = 46;

const RULE_REFERENCE_FLOOR: usize = 33;

const LIVE_ADR_FLOOR: usize = 27;

struct DocScan<'a> {
    site: &'a str,
    patterns: &'a Patterns,
    pending_bare_adr: Option<String>,
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
            if let Some(adr) = self.pending_bare_adr.take()
                && self.patterns.continuation.is_match(&text)
            {
                panic!(
                    "{}: a doc comment ends with `{adr}` and the next doc line opens with a rule \
                     id (`{}`). This guard reads one doc attribute at a time, so a citation \
                     wrapped across two lines would have its rule half checked against nothing. \
                     Put the ADR id and its rule id on one line. AFM-0037:R4 forbids passing on \
                     unreadable input",
                    self.site,
                    text.trim().chars().take(16).collect::<String>()
                );
            }
            let extracted = extract_citations(&text, self.patterns)
                .unwrap_or_else(|e| panic!("{}: {e}: {}", self.site, text.trim()));
            for (adr, rules) in extracted.citations {
                self.out.push(Citation {
                    site: self.site.to_owned(),
                    adr,
                    rules,
                    text: text.trim().to_owned(),
                });
            }
            if extracted.trailing_bare_adr {
                self.pending_bare_adr = self.out.last().map(|c| c.adr.clone());
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
        let patterns = Patterns::new();

        let mut citations = Vec::new();
        let mut counts = [0usize; 2];
        for (slot, tree) in ["src", "tests"].into_iter().enumerate() {
            let mut files = Vec::new();
            rust_sources(&root.join(tree), &mut files).expect("source directory is readable");
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
                    patterns: &patterns,
                    pending_bare_adr: None,
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
         count measured on this corpus after removing the unsupported retired-ADR claim from \
         src/output.rs. Extraction that silently stops matching is indistinguishable from a \
         corpus with nothing to check, so a shortfall is a broken guard and not a clean tree",
        scan.citations.len()
    );
    let rule_references: usize = scan.citations.iter().map(|c| c.rules.len()).sum();
    assert!(
        rule_references >= RULE_REFERENCE_FLOOR,
        "extracted {rule_references} rule reference(s) from those citations; the floor is \
         {RULE_REFERENCE_FLOOR}, the count measured on this corpus. The citation floor above \
         counts ADR tokens and stays green even when every rule suffix is dropped, so this \
         second floor is what notices a suffix parser that quietly stops matching"
    );

    let findings = unresolved_citation_findings(&scan.citations, &corpus);

    assert!(
        findings.is_empty(),
        "doc-attribute citations that do not resolve to a live corpus entry:\n{}",
        findings.join("\n")
    );
}

fn rules_of(text: &str) -> Vec<(String, Vec<u32>)> {
    extract_citations(text, &Patterns::new())
        .expect("doc text parses")
        .citations
        .into_iter()
        .map(|(adr, rules)| (adr, rules.into_iter().collect()))
        .collect()
}

fn rejection(text: &str) -> String {
    extract_citations(text, &Patterns::new())
        .err()
        .unwrap_or_else(|| panic!("expected `{text}` to be rejected, but it parsed"))
}

#[test]
fn rule_suffixes_the_corpus_spells_are_all_extracted() {
    assert_eq!(
        rules_of(" per AFM-0026:R5."),
        vec![("AFM-0026".to_owned(), vec![5])]
    );
    assert_eq!(
        rules_of(" per AFM-0006 R1 for it"),
        vec![("AFM-0006".to_owned(), vec![1])]
    );
    assert_eq!(
        rules_of(" AFM-0016 R1\u{2013}R3."),
        vec![("AFM-0016".to_owned(), vec![1, 2, 3])]
    );
    assert_eq!(
        rules_of(" AFM-0003 R1/R3 contract"),
        vec![("AFM-0003".to_owned(), vec![1, 3])]
    );
    assert_eq!(
        rules_of(" AFM-0036:R3 and R4 apply"),
        vec![("AFM-0036".to_owned(), vec![3, 4])]
    );
    assert_eq!(
        rules_of(" AFM-0026:R1, R7 are public"),
        vec![("AFM-0026".to_owned(), vec![1, 7])]
    );
    assert_eq!(
        rules_of(" AFM-0016 R3-R1."),
        vec![("AFM-0016".to_owned(), vec![1, 2, 3])]
    );
    assert_eq!(
        rules_of(" AFM-0032 alone."),
        vec![("AFM-0032".to_owned(), vec![])]
    );
    assert_eq!(
        rules_of(" AFM-0026:R5; AFM-0036:R3 and R4"),
        vec![
            ("AFM-0026".to_owned(), vec![5]),
            ("AFM-0036".to_owned(), vec![3, 4])
        ]
    );
}

#[test]
fn prose_that_merely_follows_a_citation_is_not_read_as_a_rule() {
    assert_eq!(
        rules_of(" AFM-0036:R2, its scope"),
        vec![("AFM-0036".to_owned(), vec![2])]
    );
    assert_eq!(
        rules_of(" AFM-0026 / CHE-0030: modules"),
        vec![("AFM-0026".to_owned(), vec![])]
    );
    assert_eq!(
        rules_of(" AFM-0003, all findings"),
        vec![("AFM-0003".to_owned(), vec![])]
    );
    assert_eq!(
        rules_of(" AFM-0004:R2 [L5] requires"),
        vec![("AFM-0004".to_owned(), vec![2])]
    );
}

#[test]
fn rule_suffix_forms_the_parser_cannot_read_are_rejected_rather_than_dropped() {
    assert!(rejection(" AFM-0026:Rx").contains("cannot parse"));
    assert!(rejection(" AFM-0026:R").contains("cannot parse"));
    assert!(rejection(" AFM-0026:R1, R7, 9").contains("does not parse"));
    assert!(rejection(" AFM-0026:R1 & R7").contains("does not parse"));
    assert!(rejection(" AFM-0026:R1-3").contains("does not parse"));
}

#[test]
fn a_citation_left_open_at_the_end_of_a_doc_line_is_flagged_for_the_wrap_check() {
    let patterns = Patterns::new();
    assert!(
        extract_citations(" see AFM-0016", &patterns)
            .expect("parses")
            .trailing_bare_adr,
        "a doc line ending in a bare ADR id is where a wrapped citation can hide"
    );
    assert!(
        !extract_citations(" see AFM-0016 R1", &patterns)
            .expect("parses")
            .trailing_bare_adr
    );
    assert!(
        !extract_citations(" see AFM-0016 alone.", &patterns)
            .expect("parses")
            .trailing_bare_adr
    );
    assert!(patterns.continuation.is_match(" R1\u{2013}R3 apply"));
    assert!(!patterns.continuation.is_match(" Rules apply"));
}
