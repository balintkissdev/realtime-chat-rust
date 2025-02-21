/**
 * Message payload is the same JSON event payload that is passed and serialized
 * by the server backend.
 */
interface Payload {
    event_type: PayloadEventType,
    username: string,
    message?: string
}

enum PayloadEventType {
    Connected = 'connected',
    Disconnected = 'disconnected',
    Message = 'message',
}

/**
 * Message view is the chat window displaying existing messages and having widgets
 * for sending messages.
 *
 * Establishes new WebSocket connection using browser window/tab acting as new client.
 * Fetches message history from REST API.
 */
export async function loadMessageView() {
    // Populate page
    const spaElement = document.getElementById('spa');
    if (!spaElement) {
        throw new Error('cannot find element "spa"');
    }
    spaElement.innerHTML = `
        <textarea id="messages-area" readonly></textarea>
        <div class="grid">
            <input id="send-input" type="text" placeholder="Write your message here" />
            <input id="send-button" type="button" value="Send" />
        </div>
    `;

    // Get message history
    const messageHistory = await queryHistory();
    messageHistory.forEach((payload: Payload) => {
        switch (payload.event_type) {
            case 'connected':
                appendMessage(`${payload.username} has joined the chat.`);
                break;
            case 'disconnected':
                // NOTE: Clicking the browser "Back" button also disconnects current user
                appendMessage(`${payload.username} has left the chat.`);
                break;
            case 'message':
                appendMessage(payload.message, payload.username);
                break;
        }
    });

    // Set user name
    const username = promptForUsername();

    // Establish WebSocket client connection
    const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
    const host = window.location.hostname;
    const port = '9001';    // TODO: Read from YAML
    const socket = new WebSocket(`${protocol}://${host}:${port}`);

    socket.onerror = function(e) {
        console.error('WebSocket error:', e);
        clearMessageAreaWith('Error: unable to connect to server.');
    }

    socket.onopen = function() {
        const payload = {
            event_type: 'connected',
            username: username,
        };
        socket.send(JSON.stringify(payload));
        appendMessage(`${username} has joined the chat.`);
    };

    socket.onclose = function(e) {
        console.error('WebSocket error:', e);
        clearMessageAreaWith('Error: connection to server was closed.');
    }

    socket.onmessage = function(e) {
        const payload = JSON.parse(e.data);
        switch (payload.event_type) {
            case PayloadEventType.Connected:
                appendMessage(`${payload.username} has joined the chat.`);
                break;
            case PayloadEventType.Disconnected:
                appendMessage(`${payload.username} has left the chat.`);
                break;
            case PayloadEventType.Message:
                appendMessage(payload.message, payload.username);
                break;
        }
    }

    // Message send widgets
    document.querySelector<HTMLButtonElement>('#send-input')!.onkeyup = (e) => {
        if (e.key === 'Enter') {
            sendChatMessage(socket, username);
        }
    }
    document.querySelector<HTMLButtonElement>('#send-button')!.onclick = (e) => {
        e.preventDefault();
        sendChatMessage(socket, username);
    }
}

/**
 * Query message log to populate chat window of newly joined users.
 */
async function queryHistory() {
    try {
        const url = `/api/history`;
        const resp = await fetch(url);
        if (!resp.ok) {
            throw new Error(`unable to query history: ${resp.status}`);
        }

        const payload = await resp.json();
        return payload;
    } catch (e: any) {
        if (e instanceof Error) {
            console.error(e.message);
        } else {
            console.error('unknown error happened during history query');
        }
        return [];
    }
}

/**
 * Mandatory prompt for setting username for current client.
 */
function promptForUsername() {
    let name;
    while (!name || name.trim() === '') {
        name = prompt('Please enter your name (required)');
        if (name === null) {
            name = prompt('Name is required to join the chat. Please enter your name:');
        }
    }
    name = name.trim();
    return name;
}

function sendChatMessage(socket: WebSocket, username: string) {
    const sendInput = document.querySelector<HTMLInputElement>('#send-input');
    if (!sendInput) {
        throw new Error('cannot find element "send-input"');
    }

    const msg = sendInput.value;
    if (msg.trim() === '') {
        sendInput.value = '';
        return;
    }

    const payload: Payload = {
        event_type: PayloadEventType.Message,
        username: username,
        message: msg,
    };
    socket.send(JSON.stringify(payload));
    sendInput.value = '';
    appendMessage(msg, username);
}

/**
 * Add message to chat window UI.
 */
function appendMessage(msg?: string, sender?: string) {
    if (!msg) {
        return;
    }

    const messagesArea = document.querySelector<HTMLTextAreaElement>('#messages-area');
    if (!messagesArea) {
        throw new Error('cannot find element "messages-area"');
    }
    const line = sender ? `[${sender}]: ${msg}` : msg;
    messagesArea.value += (messagesArea.value ? '\n' : '') + line;
    messagesArea.scrollTop = messagesArea.scrollHeight;
}

function clearMessageAreaWith(msg: string) {
    const messagesArea = document.querySelector<HTMLTextAreaElement>('#messages-area');
    if (!messagesArea) {
        throw new Error('cannot find element "messages-area"');
    }
    messagesArea.value = msg;
    messagesArea.scrollTop = messagesArea.scrollHeight;
}

loadMessageView();

