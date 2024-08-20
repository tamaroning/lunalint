use std::sync::Arc;

use lunalint_core::diagnostics::{LintLevel, LintReport};
use lunalint_core::{full_moon, pass, Context};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Arc<Client>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, format!("file changed"))
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        self.client
            .log_message(MessageType::INFO, format!("file opened: {}", uri))
            .await;
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

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        ])))
    }

    async fn hover(&self, _: HoverParams) -> Result<Option<Hover>> {
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String("You're hovering!".to_string())),
            range: None,
        }))
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
            let mut pass_manager = pass::PassManager::new();
            pass_manager.add_pass(Box::new(pass::CountDownLoop::new(Arc::clone(&ctx))));
            pass_manager.add_pass(Box::new(pass::GlobalInNilEnv::new(Arc::clone(&ctx))));
            pass_manager.add_pass(Box::new(pass::UnicodeName::new(Arc::clone(&ctx))));
            pass_manager.add_pass(Box::new(pass::UndefinedGlobal::new(Arc::clone(&ctx))));
            pass_manager.add_pass(Box::new(pass::LowercaseGlobal::new(Arc::clone(&ctx))));
            pass_manager.run(&ast);

            let reports = ctx
                .reports()
                .iter()
                .map(|r| Arc::clone(r))
                .collect::<Vec<_>>();
            reports
        };

        let mut handles = vec![];
        for report in reports {
            // spawn below
            //show_report(&self.client, report, uri.clone()).await;
            let client = Arc::clone(&self.client);
            let report = Arc::clone(&report);
            let uri = uri.clone();
            handles.push(tokio::spawn(async move {
                show_report(&client, &report, uri.clone()).await;
            }));
        }
        for handle in handles {
            handle.await.unwrap();
        }
    }
}

fn lintlevel_to_severity(level: &LintLevel) -> DiagnosticSeverity {
    match level {
        LintLevel::Error => DiagnosticSeverity::ERROR,
        LintLevel::Warning => DiagnosticSeverity::WARNING,
    }
}

fn lintpos_to_lsppos(pos: &lunalint_core::location::Position) -> Position {
    Position {
        line: pos.line() as u32,
        character: pos.character() as u32,
    }
}

async fn show_report(client: &Client, report: &LintReport, uri: Url) {
    let start = lintpos_to_lsppos(&report.loc().start());
    let end = lintpos_to_lsppos(&report.loc().end());
    let severity = lintlevel_to_severity(&report.level());

    client
        .publish_diagnostics(
            uri,
            vec![Diagnostic {
                range: Range { start, end },
                severity: Some(severity),
                code: None,
                code_description: None,
                source: Some("lunalint".to_owned()),
                message: report.msg().to_owned(),
                related_information: None,
                tags: None,
                data: None,
            }],
            None,
        )
        .await;
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client: Arc::new(client),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
