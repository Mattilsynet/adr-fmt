use std::fs;
use syn::punctuated::Punctuated;
use syn::visit::Visit;

mod common;
use common::{
    Citation, Corpus, Patterns, extract_citations, manifest_dir, rust_sources,
    unresolved_citation_findings,
};

const MESSAGE_CITATION_SITE_FLOOR: usize = 44;

const MESSAGE_RULE_REFERENCE_FLOOR: usize = 36;

const LIVE_ADR_FLOOR: usize = 27;

const DECLARED_RESIDUAL: &str = "\
This guard reads argument position, so it records as a claim only the message arguments of the \
macro and method forms listed in MESSAGE_MACROS and MESSAGE_METHODS. It does descend through \
the arguments of every other macro, so an in-scope macro nested inside an out-of-scope one is \
reached; but a body it cannot parse is a body it cannot descend into. Five kinds of citation \
are outside it and are not claimed to be covered: attribute argument streams such as the \
reason strings of an expect attribute, which the syntax-tree visitor never descends into; the \
second argument of matches, which is a pattern and defeats the expression parse this guard \
depends on; anything nested inside a macro body that parses as neither a comma-separated \
expression list nor a statement list, which on this corpus is matches and Token and nothing \
else; citations produced by macro expansion rather than written in the source; and citations \
sitting in operand or condition arguments, which this guard treats as data by construction.";

struct MessageMacro {
    name: &'static str,
    first_message_arg: usize,
}

const MESSAGE_MACROS: &[MessageMacro] = &[
    MessageMacro {
        name: "assert",
        first_message_arg: 1,
    },
    MessageMacro {
        name: "debug_assert",
        first_message_arg: 1,
    },
    MessageMacro {
        name: "assert_eq",
        first_message_arg: 2,
    },
    MessageMacro {
        name: "assert_ne",
        first_message_arg: 2,
    },
    MessageMacro {
        name: "debug_assert_eq",
        first_message_arg: 2,
    },
    MessageMacro {
        name: "debug_assert_ne",
        first_message_arg: 2,
    },
    MessageMacro {
        name: "panic",
        first_message_arg: 0,
    },
    MessageMacro {
        name: "unreachable",
        first_message_arg: 0,
    },
    MessageMacro {
        name: "todo",
        first_message_arg: 0,
    },
    MessageMacro {
        name: "unimplemented",
        first_message_arg: 0,
    },
];

const MESSAGE_METHODS: &[&str] = &["expect", "expect_err"];

struct Strings {
    out: Vec<String>,
}

impl<'ast> Visit<'ast> for Strings {
    fn visit_lit_str(&mut self, lit: &'ast syn::LitStr) {
        self.out.push(lit.value());
    }
}

fn strings_in(expr: &syn::Expr) -> Vec<String> {
    let mut collector = Strings { out: Vec::new() };
    collector.visit_expr(expr);
    collector.out
}

struct MessageScan<'a> {
    site: &'a str,
    patterns: &'a Patterns,
    out: &'a mut Vec<Citation>,
}

impl MessageScan<'_> {
    fn record(&mut self, context: &str, text: &str) {
        let extracted = extract_citations(text, self.patterns)
            .unwrap_or_else(|e| panic!("{}: {context}: {e}: {}", self.site, text.trim()));
        for (adr, rules) in extracted.citations {
            self.out.push(Citation {
                site: format!("{} ({context})", self.site),
                adr,
                rules,
                text: text.trim().to_owned(),
            });
        }
    }
}

enum MacroBody {
    Arguments(Punctuated<syn::Expr, syn::Token![,]>),
    Statements(Vec<syn::Stmt>),
    Opaque,
}

fn parse_macro_body(mac: &syn::Macro) -> MacroBody {
    if let Ok(args) = mac.parse_body_with(Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated)
    {
        return MacroBody::Arguments(args);
    }
    if let Ok(stmts) = mac.parse_body_with(syn::Block::parse_within) {
        return MacroBody::Statements(stmts);
    }
    MacroBody::Opaque
}

impl<'ast> Visit<'ast> for MessageScan<'_> {
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        let Some(name) = mac.path.segments.last().map(|s| s.ident.to_string()) else {
            return;
        };
        let entry = MESSAGE_MACROS.iter().find(|m| m.name == name);
        let first_message_arg = entry.map_or(usize::MAX, |m| m.first_message_arg);

        let body = match entry {
            Some(_) => MacroBody::Arguments(
                mac.parse_body_with(Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated)
                    .unwrap_or_else(|e| {
                        panic!(
                            "{}: the body of `{name}!` does not parse as a comma-separated \
                             expression list: {e}. This guard recovers argument position by \
                             that parse, so a body it cannot parse is a message argument it \
                             cannot read, and reporting it clean would be a false clean",
                            self.site
                        )
                    }),
            ),
            None => parse_macro_body(mac),
        };

        match body {
            MacroBody::Arguments(args) => {
                for (index, arg) in args.iter().enumerate() {
                    if index >= first_message_arg {
                        for text in strings_in(arg) {
                            self.record(&format!("{name}!#{index}"), &text);
                        }
                    }
                    syn::visit::visit_expr(self, arg);
                }
            }
            MacroBody::Statements(stmts) => {
                for stmt in &stmts {
                    syn::visit::visit_stmt(self, stmt);
                }
            }
            MacroBody::Opaque => {}
        }
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let method = call.method.to_string();
        if MESSAGE_METHODS.contains(&method.as_str())
            && let Some(arg) = call.args.first()
        {
            for text in strings_in(arg) {
                self.record(&format!("{method}#0"), &text);
            }
        }
        syn::visit::visit_expr_method_call(self, call);
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
                MessageScan {
                    site: &site,
                    patterns: &patterns,
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
fn assertion_message_citations_resolve_to_a_live_corpus_rule() {
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
        scan.citations.len() >= MESSAGE_CITATION_SITE_FLOOR,
        "extracted only {} citation(s) from assertion, panic and expect message arguments; the \
         floor is {MESSAGE_CITATION_SITE_FLOOR}, the count measured on this corpus. A scope \
         predicate that silently stops matching yields an empty set, which is indistinguishable \
         from a corpus with nothing to check, so a shortfall is a broken guard and not a clean \
         tree",
        scan.citations.len()
    );
    let rule_references: usize = scan.citations.iter().map(|c| c.rules.len()).sum();
    assert!(
        rule_references >= MESSAGE_RULE_REFERENCE_FLOOR,
        "extracted {rule_references} rule reference(s) from those citations; the floor is \
         {MESSAGE_RULE_REFERENCE_FLOOR}, the count measured on this corpus. The citation floor \
         above counts ADR tokens and stays green even when every rule suffix is dropped, so this \
         second floor is what notices a suffix parser that quietly stops matching"
    );

    let findings = unresolved_citation_findings(&scan.citations, &corpus);

    assert!(
        findings.is_empty(),
        "assertion, panic and expect message citations that do not resolve to a live corpus \
         entry:\n{}\n\nScope of this guard: {DECLARED_RESIDUAL}",
        findings.join("\n")
    );
}
