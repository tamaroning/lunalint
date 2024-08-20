use std::sync::Arc;

use lunalint_core::diagnostics::{LintLevel, LintReport};
use lunalint_core::{full_moon, pass, Context};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::NONE),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            // Set true to lint on save
                            include_text: Some(true),
                        })),
                        ..TextDocumentSyncOptions::default()
                    },
                )),
                ..ServerCapabilities::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "lunalintd initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::INFO, "lunalintd shutdown")
            .await;
        Ok(())
    }

    async fn did_change(&self, _: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, format!("file changed, ignored"))
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        self.client
            .log_message(MessageType::INFO, format!("file opened: {}", uri))
            .await;
        self.lint(uri, params.text_document.text).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        self.client
            .log_message(MessageType::INFO, format!("file saved: {}", uri))
            .await;
        if let Some(src) = params.text {
            self.lint(uri, src).await;
        }
    }
}

impl Backend {
    async fn lint(&self, uri: Url, src: String) {
        let ast = match full_moon::parse(&src) {
            Ok(ast) => ast,
            Err(err) => {
                self.client
                    .log_message(MessageType::ERROR, format!("parse error: {}", err))
                    .await;
                return;
            }
        };
        self.client
            .log_message(MessageType::ERROR, format!("parse finished"))
            .await;

        let reports = {
            let mut ctx = Context::new(uri.to_file_path().unwrap(), src);
            ctx.resolver_mut().go(&ast);

            let ctx = Arc::new(ctx);
            let mut pass_manager = pass::PassManager::with_all_passes(Arc::clone(&ctx));
            pass_manager.run(&ast);

            let reports = ctx
                .reports()
                .iter()
                .map(|r| Arc::clone(r))
                .collect::<Vec<_>>();
            reports
        };

        let mut diags = vec![];
        for report in reports {
            diags.push(report_to_diag(&report));
        }
        self.client.publish_diagnostics(uri, diags, None).await;
    }
}

fn lintlevel_to_severity(level: &LintLevel) -> DiagnosticSeverity {
    match level {
        LintLevel::Error => DiagnosticSeverity::ERROR,
        LintLevel::Warning => DiagnosticSeverity::WARNING,
    }
}

fn lintloc_to_lsploc(loc: &lunalint_core::location::Location) -> Location {
    Location {
        uri: Url::from_file_path(loc.src().path()).unwrap(),
        range: Range {
            start: lintpos_to_lsppos(&loc.start()),
            end: lintpos_to_lsppos(&loc.end()),
        },
    }
}

fn lintpos_to_lsppos(pos: &lunalint_core::location::Position) -> Position {
    Position {
        line: pos.line() as u32 - 1,
        character: pos.character() as u32 - 1,
    }
}

fn report_to_diag(report: &LintReport) -> Diagnostic {
    let loc = report.loc();
    let start = lintpos_to_lsppos(&loc.start());
    let end = lintpos_to_lsppos(&loc.end());
    let severity = lintlevel_to_severity(&report.level());

    let mut related_information = None;
    if !report.labels().is_empty() {
        let mut infos = vec![];
        for label in report.labels() {
            let info = DiagnosticRelatedInformation {
                location: lintloc_to_lsploc(&label.loc()),
                message: label.msg().to_owned(),
            };
            infos.push(info);
        }
        related_information = Some(infos);
    }

    Diagnostic {
        range: Range { start, end },
        severity: Some(severity),
        code: None,
        code_description: None,
        source: Some("lunalint".to_owned()),
        message: report.msg().to_owned(),
        related_information,
        tags: None,
        data: None,
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
