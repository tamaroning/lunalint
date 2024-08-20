import * as vscode from 'vscode';
import { workspace } from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';

let client: LanguageClient | null = null;

export function activate(context: vscode.ExtensionContext) {

	console.log('lunalint is now active!');

	const disposable = vscode.commands.registerCommand('lunalint.helloWorld', () => {
		vscode.window.showInformationMessage('Hello World from lunalint!');
	});

	context.subscriptions.push(disposable);

	const serverExecutable = '<path-to-your-binary>';

	const serverOptions: ServerOptions = {
		command: serverExecutable,
		args: [],
		transport: TransportKind.stdio
	};

	const clientOptions: LanguageClientOptions = {
		documentSelector: [{ scheme: 'file', language: 'lua' }],
		synchronize: {
			fileEvents: workspace.createFileSystemWatcher('**/.clientrc')
		}
	};

	client = new LanguageClient(
		'lunalint',
		'A Lua linter',
		serverOptions,
		clientOptions
	);

	client.start();
}


export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}
