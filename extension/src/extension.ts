import * as path from 'path';
import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind } from 'vscode-languageclient/node';

let client: LanguageClient | null = null;

export function activate(context: vscode.ExtensionContext) {
	console.log('lunalint is now active');

	const serverExecutable = context.asAbsolutePath(path.join('..', 'target', 'debug', 'lunalintd.exe'));
	console.log('path:', serverExecutable);
	const srcfilePath = vscode.window.activeTextEditor?.document.fileName;

	const disposable = vscode.commands.registerCommand('lunalint.activate', () => { });

	context.subscriptions.push(disposable);

	const serverOptions: ServerOptions = {
		command: serverExecutable,
		args: [],
		transport: TransportKind.stdio
	};

	const clientOptions: LanguageClientOptions = {
		documentSelector: [{
			scheme: 'file', pattern: '**/*.lua'
		}],
		synchronize: {
			fileEvents: vscode.workspace.createFileSystemWatcher('**/*.lua')
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
	if (!client) {
		console.error('Error: lunalint client is not defined');
		return undefined;
	}
	return client.stop();
}
