import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
    const serverPath = vscode.workspace.getConfiguration('flint').get<string>('serverPath') || 'flint-lsp';
    const trace = vscode.workspace.getConfiguration('flint').get<string>('trace.server') || 'off';

    const serverOptions: ServerOptions = {
        command: serverPath,
        args: [],
        options: { env: { ...process.env } }
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ language: 'flint' }],
        synchronize: {
            configurationSection: 'flint'
        },
        outputChannelName: 'Flint Language Server',
        traceOutput: trace === 'verbose' ? 'verbose' : trace === 'messages' ? 'message' : 'off'
    };

    client = new LanguageClient('flint-lsp', 'Flint Language Server', serverOptions, clientOptions);

    context.subscriptions.push(
        vscode.commands.registerCommand('flint.formatDocument', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor || editor.document.languageId !== 'flint') return;
            
            const doc = editor.document;
            const text = doc.getText();
            
            try {
                const edit = new vscode.WorkspaceEdit();
                edit.replace(doc.uri, new vscode.Range(0, 0, doc.lineCount, 0), formatFlintCode(text));
                await vscode.workspace.applyEdit(edit);
            } catch (e) {
                vscode.window.showErrorMessage('Format failed: ' + e);
            }
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('flint.runFile', async () => {
            const doc = vscode.window.activeTextEditor?.document;
            if (!doc) return;
            
            const terminal = vscode.window.createTerminal('Flint Run');
            terminal.sendText(`flint run "${doc.uri.fsPath}"`);
            terminal.show();
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('flint.openRepl', () => {
            const terminal = vscode.window.createTerminal('Flint REPL');
            terminal.sendText('flint repl');
            terminal.show();
        })
    );

    context.subscriptions.push(
        vscode.commands.registerCommand('flint.runTests', async () => {
            const doc = vscode.window.activeTextEditor?.document;
            if (!doc) return;
            
            const terminal = vscode.window.createTerminal('Flint Test');
            terminal.sendText(`flint test "${doc.uri.fsPath}"`);
            terminal.show();
        })
    );

    client.start();
}

export function deactivate() {
    if (client) {
        client.stop();
    }
}

function formatFlintCode(code: string): string {
    const lines = code.split('\n');
    let indent = 0;
    const result: string[] = [];

    for (let line of lines) {
        line = line.trim();
        
        if (!line) {
            result.push('');
            continue;
        }

        if (line.startsWith('elif ') || line.startsWith('else') || line === 'end') {
            indent = Math.max(0, indent - 1);
        }

        result.push('  '.repeat(indent) + line);

        if (line.endsWith(':') && !line.startsWith('//')) {
            indent++;
        }
    }

    return result.join('\n');
}

function provideCompletionItems(document: vscode.TextDocument, position: vscode.Position): vscode.CompletionItem[] {
    const items: vscode.CompletionItem[] = [];
    const keywords = [
        'let', 'var', 'const', 'fn', 'return', 'if', 'else', 'elif',
        'for', 'while', 'match', 'in', 'is', 'not', 'and', 'or',
        'true', 'false', 'null', 'type', 'interface', 'trait', 'impl',
        'enum', 'struct', 'import', 'export', 'async', 'await', 'spawn'
    ];

    for (const kw of keywords) {
        const item = new vscode.CompletionItem(kw, vscode.CompletionItemKind.Keyword);
        item.detail = `keyword: ${kw}`;
        items.push(item);
    }

    const types = ['Int', 'Float', 'Bool', 'Str', 'Char', 'Array', 'Map', 'Option', 'Result'];
    for (const ty of types) {
        const item = new vscode.CompletionItem(ty, vscode.CompletionItemKind.TypeParameter);
        item.detail = `type: ${ty}`;
        items.push(item);
    }

    return items;
}

function provideHover(document: vscode.TextDocument, position: vscode.Position): vscode.Hover | null {
    const range = document.getWordRangeAtPosition(position);
    const word = document.getText(range);
    
    const docs: Record<string, string> = {
        'let': 'Declare an immutable variable',
        'var': 'Declare a mutable variable',
        'fn': 'Define a function',
        'if': 'Conditional expression',
        'match': 'Pattern matching expression',
        'for': 'For loop iteration',
        'while': 'While loop'
    };

    if (docs[word]) {
        return new vscode.Hover(docs[word], range);
    }

    return null;
}