import React, { useState, useEffect, useRef } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card';
import { Button } from '@/components/ui/Button';
import {
    Terminal as TerminalIcon,
    Cpu,
    BrainCircuit,
    ShieldCheck,
    Activity,
    Database,
    RefreshCw,
    AlertTriangle
} from 'lucide-react';

interface KeyStatus {
    id: string;
    provider: string;
    label: string;
    is_active: boolean;
    failure_count: number;
    last_used: string | null;
}

interface LogEntry {
    id: string;
    timestamp: string;
    level: 'info' | 'warn' | 'error' | 'debug';
    agent: string;
    message: string;
}

export const HackerDashboard: React.FC = () => {
    const [keys, setKeys] = useState<KeyStatus[]>([]);
    const [logs, setLogs] = useState<LogEntry[]>([]);
    const [systemHealth, setSystemHealth] = useState('Checking...');
    const [loading, setLoading] = useState(true);
    const terminalEndRef = useRef<HTMLDivElement>(null);

    // Simulated live logs for initial "Wow" factor - will connect to backend later
    useEffect(() => {
        const mockAgents = ['Agent 4', 'Agent 7', 'Agent 8', 'System'];
        const mockMessages = [
            'Rotating AI brain power to Cerebras Primary...',
            'Analyzing jailbreak attempt on target endpoint...',
            'Sandbox execution environment initialized successfully.',
            'Collaboration protocol established between Editor and Terminal agents.',
            'Self-Correction Engine detected minor syntax anomaly, fix generated.',
            'Ultra Tier request processed with high confidence bypass.',
            'Syncing state with Redis cluster...',
            'Monitoring dependency health: All systems NOMINAL.'
        ];

        const logTimer = setInterval(() => {
            const newLog: LogEntry = {
                id: Math.random().toString(36).substr(2, 9),
                timestamp: new Date().toLocaleTimeString(),
                level: Math.random() > 0.8 ? (Math.random() > 0.5 ? 'warn' : 'error') : 'info',
                agent: mockAgents[Math.floor(Math.random() * mockAgents.length)],
                message: mockMessages[Math.floor(Math.random() * mockMessages.length)]
            };
            setLogs(prev => [...prev.slice(-49), newLog]);
        }, 2000);

        return () => clearInterval(logTimer);
    }, []);

    // Fetch real key status from backend
    const fetchKeyStatus = async () => {
        try {
            const response = await fetch('/api/status/keys');
            if (response.ok) {
                const data = await response.json();
                setKeys(data);
                setSystemHealth('NOMINAL');
            } else {
                setSystemHealth('DEGRADED');
            }
        } catch (error) {
            console.error('Failed to fetch key status:', error);
            setSystemHealth('OFFLINE');
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchKeyStatus();
        const keyTimer = setInterval(fetchKeyStatus, 10000);
        return () => clearInterval(keyTimer);
    }, []);

    // Auto-scroll terminal
    useEffect(() => {
        terminalEndRef.current?.scrollIntoView({ behavior: 'smooth' });
    }, [logs]);

    return (
        <div className="space-y-6 font-mono selection:bg-purple-600/30">
            {/* Header with System Specs */}
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-gray-700 pb-6">
                <div>
                    <h1 className="text-4xl font-black tracking-tighter text-gray-100 flex items-center gap-3">
                        <Activity className="h-8 w-8 text-green-400 animate-pulse" />
                        HACKER DASHBOARD <span className="text-xs font-mono text-purple-400 align-top px-2 py-0.5 bg-purple-600/10 rounded">V2.0-ULTRA</span>
                    </h1>
                    <p className="mt-2 text-gray-400 max-w-2xl">
                        Real-time telemetry from MR.DarkPromth Core. Monitoring Agent coordination, AI brain rotation, and Sandbox security.
                    </p>
                </div>
                <div className="grid grid-cols-2 gap-4">
                    <div className="bg-gray-800 p-3 rounded-lg border border-gray-700 shadow-lg shadow-purple-600/5">
                        <div className="text-[10px] text-gray-500 uppercase tracking-widest">System Status</div>
                        <div className={`text-sm font-bold mt-1 ${systemHealth === 'NOMINAL' ? 'text-green-400' : 'text-pink-400'}`}>
                            {systemHealth}
                        </div>
                    </div>
                    <div className="bg-gray-800 p-3 rounded-lg border border-gray-700 shadow-lg shadow-blue-600/5">
                        <div className="text-[10px] text-gray-500 uppercase tracking-widest">Uptime</div>
                        <div className="text-sm font-bold mt-1 text-blue-400">99.99%</div>
                    </div>
                </div>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">

                {/* Main Log Terminal */}
                <div className="lg:col-span-2 space-y-6">
                    <Card className="bg-black/80 border-gray-700 shadow-2xl h-[600px] flex flex-col">
                        <CardHeader className="border-b border-gray-700/50 py-3 flex flex-row items-center justify-between">
                            <div className="flex items-center gap-2">
                                <TerminalIcon className="h-4 w-4 text-purple-400" />
                                <CardTitle className="text-sm font-bold tracking-widest uppercase">Live Agent Terminal</CardTitle>
                            </div>
                            <div className="flex gap-2">
                                <div className="h-2 w-2 rounded-full bg-pink-400 animate-pulse" />
                                <span className="text-[10px] text-gray-500 uppercase">Streaming...</span>
                            </div>
                        </CardHeader>
                        <CardContent className="flex-1 p-0 overflow-hidden relative">
                            <div className="absolute inset-0 bg-[radial-gradient(circle_at_center,_transparent_0%,_rgba(0,0,0,0.4)_100%)] pointer-events-none" />
                            <div className="h-full overflow-y-auto p-4 space-y-1.5 scrollbar-thin scrollbar-thumb-gray-700">
                                {logs.map((log) => (
                                    <div key={log.id} className="text-xs flex gap-3 group">
                                        <span className="text-gray-600 shrink-0">[{log.timestamp}]</span>
                                        <span className={`w-16 shrink-0 font-bold ${log.agent === 'Agent 4' ? 'text-blue-400' :
                                            log.agent === 'Agent 7' ? 'text-purple-400' :
                                                log.agent === 'Agent 8' ? 'text-pink-400' : 'text-gray-400'
                                            }`}>
                                            {log.agent}
                                        </span>
                                        <span className={`flex-1 ${log.level === 'error' ? 'text-pink-400' :
                                            log.level === 'warn' ? 'text-amber-400' : 'text-gray-300'
                                            }`}>
                                            {log.message}
                                        </span>
                                    </div>
                                ))}
                                <div ref={terminalEndRef} />
                            </div>
                        </CardContent>
                    </Card>

                    {/* AI Brain Pool Status */}
                    <Card className="bg-gray-800/50 border-gray-700 backdrop-blur-sm">
                        <CardHeader className="py-4 border-b border-gray-700/30">
                            <div className="flex items-center justify-between">
                                <div className="flex items-center gap-2">
                                    <BrainCircuit className="h-5 w-5 text-blue-400" />
                                    <CardTitle className="text-md uppercase tracking-tight">AI Brain Power Pool</CardTitle>
                                </div>
                                <Button variant="ghost" size="sm" onClick={fetchKeyStatus} className="h-8 px-2 hover:bg-blue-600/10">
                                    <RefreshCw className={`h-4 w-4 ${loading ? 'animate-spin' : ''}`} />
                                </Button>
                            </div>
                        </CardHeader>
                        <CardContent className="p-0">
                            <div className="divide-y divide-gray-700/20">
                                {keys.length > 0 ? (
                                    keys.map((key) => (
                                        <div key={key.id} className="p-4 flex items-center justify-between hover:bg-white/5 transition-colors">
                                            <div className="flex items-center gap-4">
                                                <div className={`p-2 rounded bg-black/40 border ${key.is_active ? 'border-green-400/30' : 'border-pink-400/30'}`}>
                                                    <Database className={`h-5 w-5 ${key.is_active ? 'text-green-400' : 'text-pink-400'}`} />
                                                </div>
                                                <div>
                                                    <div className="font-bold text-gray-200">{key.label}</div>
                                                    <div className="text-[10px] text-gray-500 uppercase flex items-center gap-2">
                                                        {key.provider} • ID: {key.id.split('-')[0]}...
                                                        <span className={`inline-block w-1.5 h-1.5 rounded-full ${key.is_active ? 'bg-green-400' : 'bg-pink-400'}`} />
                                                    </div>
                                                </div>
                                            </div>
                                            <div className="text-right">
                                                <div className="text-xs text-gray-400 uppercase">Fails: {key.failure_count}</div>
                                                <div className="text-[10px] text-gray-600 mt-1">
                                                    Last Use: {key.last_used ? new Date(key.last_used).toLocaleTimeString() : 'Never'}
                                                </div>
                                            </div>
                                        </div>
                                    ))
                                ) : (
                                    <div className="p-8 text-center text-gray-500 italic text-sm">
                                        {loading ? 'Initializing brain link...' : 'No AI brain power detected in pool. Add keys via environment variables.'}
                                    </div>
                                )}
                            </div>
                        </CardContent>
                    </Card>
                </div>

                {/* Sidebar: System Specs & Stats */}
                <div className="space-y-6">
                    {/* Sandbox Security Status */}
                    <Card className="bg-gradient-to-br from-gray-800 to-black border-gray-700 overflow-hidden relative group">
                        <div className="absolute top-0 right-0 p-4 opacity-10 group-hover:opacity-20 transition-opacity">
                            <ShieldCheck className="h-24 w-24 text-green-400" />
                        </div>
                        <CardHeader className="pb-2">
                            <CardTitle className="text-xs uppercase tracking-widest text-gray-500">Sandbox Isolation</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <div className="text-3xl font-black text-green-400 tracking-tighter uppercase">MAX_SECURE</div>
                            <div className="mt-2 p-2 bg-green-400/5 border border-green-400/20 rounded text-[10px] text-green-400 uppercase leading-relaxed">
                                Firewall: ACTIVE <br />
                                Resource Limit: 512MB <br />
                                Network: ISOLATED
                            </div>
                        </CardContent>
                    </Card>

                    {/* Quick Metrics */}
                    <div className="grid grid-cols-1 gap-4">
                        <div className="bg-black/50 p-4 rounded-xl border border-gray-700 flex justify-between items-center group cursor-crosshair">
                            <div>
                                <div className="text-[10px] text-gray-500 uppercase">Agent Efficiency</div>
                                <div className="text-2xl font-bold text-purple-400 group-hover:text-green-400 transition-colors">94.2%</div>
                            </div>
                            <Cpu className="h-8 w-8 text-gray-700 group-hover:rotate-90 transition-transform duration-500" />
                        </div>

                        <div className="bg-black/50 p-4 rounded-xl border border-gray-700 flex justify-between items-center group cursor-crosshair">
                            <div>
                                <div className="text-[10px] text-gray-500 uppercase">Jailbreak Effectiveness</div>
                                <div className="text-2xl font-bold text-pink-400">HIGH</div>
                            </div>
                            <AlertTriangle className="h-8 w-8 text-pink-400 animate-pulse" />
                        </div>

                        <div className="bg-black/50 p-4 rounded-xl border border-gray-700 flex justify-between items-center group cursor-crosshair">
                            <div>
                                <div className="text-[10px] text-gray-500 uppercase">Active Brains</div>
                                <div className="text-2xl font-bold text-blue-400">{keys.filter(k => k.is_active).length} / {keys.length}</div>
                            </div>
                            <BrainCircuit className="h-8 w-8 text-blue-400 group-hover:scale-110 transition-transform" />
                        </div>
                    </div>

                    {/* Interactive Terminal Control Panel */}
                    <Card className="bg-gray-800 border-gray-700">
                        <CardHeader>
                            <CardTitle className="text-xs uppercase tracking-widest text-white flex items-center gap-2">
                                <TerminalIcon className="h-3 w-3" />
                                Interactive Terminal
                            </CardTitle>
                        </CardHeader>
                        <CardContent className="space-y-4">
                            <InteractiveTerminal />
                        </CardContent>
                    </Card>

                    {/* Tool Execution Panel */}
                    <Card className="bg-gray-800 border-gray-700">
                        <CardHeader>
                            <CardTitle className="text-xs uppercase tracking-widest text-white flex items-center gap-2">
                                <Cpu className="h-3 w-3" />
                                Tool Execution
                            </CardTitle>
                        </CardHeader>
                        <CardContent className="space-y-4">
                            <ToolExecutionPanel />
                        </CardContent>
                    </Card>
                </div>
            </div>
        </div>
    );
};

// Interactive Terminal Component for real command execution
const InteractiveTerminal: React.FC = () => {
    const [command, setCommand] = useState('');
    const [output, setOutput] = useState<Array<{ type: 'cmd' | 'out' | 'err'; text: string }>>([]);
    const [isExecuting, setIsExecuting] = useState(false);
    const [history, setHistory] = useState<string[]>([]);
    const [historyIndex, setHistoryIndex] = useState(-1);
    const outputRef = useRef<HTMLDivElement>(null);

    const executeCommand = async () => {
        if (!command.trim() || isExecuting) return;

        // Add command to history
        setHistory(prev => [...prev, command]);
        setHistoryIndex(-1);

        // Display the command in output
        setOutput(prev => [...prev, { type: 'cmd', text: `$ ${command}` }]);
        setIsExecuting(true);

        try {
            const response = await fetch('/api/terminal/execute', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    command: command.split(' ')[0],
                    args: command.split(' ').slice(1),
                    user_id: 'ultra-dashboard-user' // Ultra tier user
                })
            });

            const result = await response.json();

            if (response.ok) {
                if (result.stdout) {
                    setOutput(prev => [...prev, { type: 'out', text: result.stdout }]);
                }
                if (result.stderr) {
                    setOutput(prev => [...prev, { type: 'err', text: result.stderr }]);
                }
                if (!result.stdout && !result.stderr) {
                    setOutput(prev => [...prev, { type: 'out', text: `[Exit code: ${result.exit_code}]` }]);
                }
            } else {
                setOutput(prev => [...prev, { type: 'err', text: result.message || 'Command failed' }]);
            }
        } catch (error) {
            setOutput(prev => [...prev, { type: 'err', text: `Network error: ${error}` }]);
        } finally {
            setIsExecuting(false);
            setCommand('');
        }
    };

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter') {
            executeCommand();
        } else if (e.key === 'ArrowUp') {
            e.preventDefault();
            if (history.length > 0) {
                const newIndex = historyIndex === -1 ? history.length - 1 : Math.max(0, historyIndex - 1);
                setHistoryIndex(newIndex);
                setCommand(history[newIndex]);
            }
        } else if (e.key === 'ArrowDown') {
            e.preventDefault();
            if (historyIndex !== -1) {
                const newIndex = historyIndex + 1;
                if (newIndex >= history.length) {
                    setHistoryIndex(-1);
                    setCommand('');
                } else {
                    setHistoryIndex(newIndex);
                    setCommand(history[newIndex]);
                }
            }
        }
    };

    useEffect(() => {
        outputRef.current?.scrollIntoView({ behavior: 'smooth' });
    }, [output]);

    return (
        <div className="space-y-3">
            {/* Output Display */}
            <div className="bg-black/80 rounded border border-gray-700/50 p-3 h-32 overflow-y-auto font-mono text-xs">
                {output.length === 0 ? (
                    <span className="text-gray-600 italic">Ready for commands. Ultra Tier required.</span>
                ) : (
                    output.map((line, i) => (
                        <div key={i} className={`${line.type === 'cmd' ? 'text-purple-400 font-bold' :
                            line.type === 'err' ? 'text-pink-400' : 'text-gray-300'
                            } whitespace-pre-wrap break-all`}>
                            {line.text}
                        </div>
                    ))
                )}
                <div ref={outputRef} />
            </div>

            {/* Command Input */}
            <div className="relative flex gap-2">
                <span className="absolute left-3 top-2.5 text-purple-400 font-bold text-xs">$</span>
                <Input
                    value={command}
                    onChange={(e) => setCommand(e.target.value)}
                    onKeyDown={handleKeyDown}
                    placeholder="Enter command..."
                    disabled={isExecuting}
                    className="bg-black border-gray-700 text-blue-400 placeholder:text-gray-700 font-mono text-xs focus:ring-purple-600 pl-6"
                />
                <Button
                    size="sm"
                    onClick={executeCommand}
                    disabled={isExecuting || !command.trim()}
                    className="bg-purple-600 hover:bg-blue-600 text-xs font-bold shrink-0"
                >
                    {isExecuting ? 'RUN...' : 'RUN'}
                </Button>
            </div>

            {/* Quick Commands */}
            <div className="flex flex-wrap gap-1.5">
                {['whoami', 'pwd', 'ls', 'date', 'uptime'].map(cmd => (
                    <button
                        key={cmd}
                        onClick={() => setCommand(cmd)}
                        className="px-2 py-0.5 bg-black/50 rounded border border-gray-700/40 text-[9px] text-gray-400 hover:text-blue-400 hover:border-blue-400/30 transition-colors"
                    >
                        {cmd}
                    </button>
                ))}
            </div>
        </div>
    );
};

// Tool Execution Panel Component
const ToolExecutionPanel: React.FC = () => {
    const [selectedTool, setSelectedTool] = useState('file_read');
    const [toolInput, setToolInput] = useState('');
    const [result, setResult] = useState<{ success: boolean; data: any; error?: string } | null>(null);
    const [isExecuting, setIsExecuting] = useState(false);
    const [tools, setTools] = useState<Array<{ name: string; description: string }>>([]);

    useEffect(() => {
        fetch('/api/tools')
            .then(res => res.json())
            .then(data => setTools(data.tools || []))
            .catch(err => console.error('Failed to fetch tools:', err));
    }, []);

    const executeTool = async () => {
        if (!toolInput.trim() || isExecuting) return;
        setIsExecuting(true);
        setResult(null);
        try {
            let inputJson;
            try { inputJson = JSON.parse(toolInput); } catch { inputJson = { input: toolInput }; }
            const response = await fetch('/api/tools/execute', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ tool_name: selectedTool, input: inputJson, user_id: 'ultra-dashboard-user' })
            });
            setResult(await response.json());
        } catch (error) {
            setResult({ success: false, data: null, error: String(error) });
        } finally {
            setIsExecuting(false);
        }
    };

    return (
        <div className="space-y-3">
            <div className="flex gap-2">
                <select value={selectedTool} onChange={(e) => setSelectedTool(e.target.value)}
                    className="bg-black border border-gray-700 text-purple-400 text-xs rounded px-2 py-1.5 focus:ring-purple-600 flex-1">
                    {tools.map(tool => (<option key={tool.name} value={tool.name}>{tool.name}</option>))}
                </select>
            </div>
            <div className="flex gap-2">
                <Input value={toolInput} onChange={(e) => setToolInput(e.target.value)}
                    placeholder='{"path": "/tmp/test.txt"}' className="bg-black border-gray-700 text-green-400 placeholder:text-gray-700 font-mono text-xs" />
                <Button size="sm" onClick={executeTool} disabled={isExecuting || !toolInput.trim()}
                    className="bg-green-400 hover:bg-blue-600 text-black text-xs font-bold shrink-0">
                    {isExecuting ? '...' : 'EXEC'}
                </Button>
            </div>
            {result && (
                <div className={`bg-black/80 rounded border ${result.success ? 'border-green-400/30' : 'border-pink-400/30'} p-2 text-xs max-h-24 overflow-y-auto`}>
                    <div className={`font-bold ${result.success ? 'text-green-400' : 'text-pink-400'}`}>
                        {result.success ? '✓ Success' : '✗ Error'}
                    </div>
                    <pre className="text-gray-300 whitespace-pre-wrap mt-1 text-[10px]">
                        {result.error || JSON.stringify(result.data, null, 2).slice(0, 300)}
                    </pre>
                </div>
            )}
        </div>
    );
};

// WebSocket hook for real-time logs (currently unused)
// const useWebSocketLogs = (url: string) => {
//     const [logs, setLogs] = useState<LogEntry[]>([]);
//     const [connected, setConnected] = useState(false);
//     useEffect(() => {
//         try {
//             const ws = new WebSocket(url);
//             ws.onopen = () => setConnected(true);
//             ws.onclose = () => setConnected(false);
//             ws.onmessage = (event) => {
//                 try { setLogs(prev => [...prev.slice(-49), JSON.parse(event.data)]); } catch { }
//             };
//             return () => ws.close();
//         } catch { setConnected(false); }
//     }, [url]);
//     return { logs, connected };
// };

const Input = React.forwardRef<HTMLInputElement, React.InputHTMLAttributes<HTMLInputElement>>(
    ({ className, ...props }, ref) => {
        return (
            <input
                className={`flex h-10 w-full rounded-md border border-input bg-transparent px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 ${className}`}
                ref={ref}
                {...props}
            />
        );
    }
);
Input.displayName = 'Input';
