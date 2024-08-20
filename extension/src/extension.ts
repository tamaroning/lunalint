import * as path from 'path';
import * as vscode from 'vscode';
import { workspace } from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';

let client: LanguageClient | null = null;

export function activate(context: vscode.ExtensionContext) {
	console.log('lunalint is now active!');

	const serverExecutable = context.asAbsolutePath(path.join('..', 'target', 'debug', 'lunalintd.exe'));
	console.log('serverExecutable:', serverExecutable);
	const srcfilePath = vscode.window.activeTextEditor?.document.fileName;

	const disposable = vscode.commands.registerCommand('lunalint.helloWorld', () => {
		vscode.window.showInformationMessage('Hello World from lunalint!');
	});

	context.subscriptions.push(disposable);

	const serverOptions: ServerOptions = {
		command: serverExecutable,
		args: [],
		transport: TransportKind.stdio
	};

	const clientOptions: LanguageClientOptions = {
		documentSelector: [{
			scheme: 'file', language: 'lua',
			pattern: '**/*.lua'
		}],
		synchronize: {
			fileEvents: workspace.createFileSystemWatcher('**/.clientrc')
		}
	};

	client = new LanguageClient(
		'lunalint',
		'Lunalint',
		serverOptions,
		clientOptions
	);

	client.start();
}


export function deactivate(): Thenable<void> | undefined {
	console.log('lunalint is deactivating');
	/*
	if (!client) {
		return undefined;
	}
	return client.stop();
	*/
	return undefined;
}
