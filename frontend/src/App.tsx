import { useEffect, useState, useRef } from 'react';

import { Payload, PayloadEventType, payloadToMessageLine } from './payload';
import './App.css';

/**
 * Chat window displaying existing messages and having widgets
 * for sending messages.
 *
 * Establishes new WebSocket connection using browser window/tab acting as new client.
 * Fetches message history from REST API.
 */
export default function App() {
    const [username, setUsername] = useState<string>('');
    const [messages, setMessages] = useState<string[]>([]);
    const [inputValue, setInputValue] = useState<string>('');
    const socketRef = useRef<WebSocket | null>(null);
    const messagesAreaRef = useRef<HTMLTextAreaElement>(null);

    const promptForUsername = () => {
        let name;
        while (!name || name.trim() === '') {
            name = prompt('Please enter your name (required)');
            if (name === null) {
                name = prompt('Name is required to join the chat. Please enter your name:');
            }
        }
        return name.trim();
    }

    const sendChatMessage = (e: React.FormEvent) => {
        e.preventDefault();

        const msg = inputValue;
        if (msg.trim() === '') {
            setInputValue('');
            return;
        }

        if (socketRef.current && socketRef.current.readyState === WebSocket.OPEN) {
            const payload: Payload = {
                event_type: PayloadEventType.Message,
                username: username,
                message: msg,
            };
            socketRef.current.send(JSON.stringify(payload));
            setMessages(prev => [...prev, payloadToMessageLine(payload)]);
            setInputValue('');
        }
    }

    // Scroll messages
    useEffect(() => {
        if (messagesAreaRef.current) {
            messagesAreaRef.current.scrollTop = messagesAreaRef.current.scrollHeight;
        }
    }, [messages]);

    useEffect(() => {
        // Set user name
        const name = promptForUsername();
        setUsername(name);

        const fetchHistory = async () => {
            try {
                const url = `/api/history`;
                const resp = await fetch(url);
                if (!resp.ok) {
                    throw new Error(`unable to query history: ${resp.status}`);
                }

                const historyPayload: Payload[] = await resp.json();
                const historyMessages: string[] = [];

                historyPayload.forEach((payload: Payload) => {
                    historyMessages.push(payloadToMessageLine(payload));
                });

                setMessages(historyMessages);
            } catch (e) {
                console.error('Error fetching history:', e);
                setMessages(['Error: Unable to fetch message history.']);
            }
        }

        const connectToServer = () => {
            const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
            const host = window.location.hostname;
            const port = '9001';    // TODO: Read from YAML
            const socket = new WebSocket(`${protocol}://${host}:${port}`);
            socketRef.current = socket;

            socket.onerror = (e) => {
                console.error('WebSocket error:', e);
                setMessages(['Error: unable to connect to server.']);
            }

            socket.onopen = () => {
                const payload = {
                    event_type: PayloadEventType.Connected,
                    username: name,
                };
                socket.send(JSON.stringify(payload));
                setMessages(prev => [...prev, payloadToMessageLine(payload)]);
            };

            socket.onclose = (e) => {
                console.error('WebSocket error:', e);
                setMessages(prev => [...prev, 'Error: connection to server was closed.']);
            }

            socket.onmessage = (e) => {
                const payload: Payload = JSON.parse(e.data);
                setMessages(prev => [...prev, payloadToMessageLine(payload)]);
            }
        }

        fetchHistory().then(() => connectToServer());

        return () => {
            if (socketRef.current && socketRef.current.readyState === WebSocket.OPEN) {
                const payload = {
                    event_type: PayloadEventType.Disconnected,
                    username: name,
                };
                socketRef.current.send(JSON.stringify(payload));
                socketRef.current.close();
            }
        };
    }, []);

    return (
        <>
            <textarea
                ref={messagesAreaRef}
                id="messages-area"
                readOnly
                value={messages.join('\n')}
            />
            <div className="grid">
                <input
                    id="send-input"
                    type="text"
                    placeholder="Write your message here"
                    value={inputValue}
                    onChange={(e) => setInputValue(e.target.value)}
                    onKeyUp={(e) => {
                        if (e.key === 'Enter') {
                            e.preventDefault();
                            sendChatMessage(e);
                        }
                    }}
                />
                <button
                    id="send-button"
                    type="button"
                    onClick={sendChatMessage}
                >
                    Send
                </button>
            </div>
        </>
    );
}

