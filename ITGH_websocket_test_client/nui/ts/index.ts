/**
 * Interface representing a 3D position in the game world
 */
interface Position {
    /** X coordinate */
    x: number;
    /** Y coordinate */
    y: number;
    /** Z coordinate */
    z: number;
}

/**
 * Interface for WebSocket configuration received from Lua side
 */
interface WebSocketConfig {
    /** WebSocket server URL */
    Url: string;
    /** Interval in milliseconds for position tracking */
    PositionTrackingInterval: number;
}

/**
 * Client for handling WebSocket connections to the server
 */
class WebSocketClient {
    /** WebSocket connection instance */
    private socket: WebSocket | null = null;
    
    /** WebSocket server URL from config */
    private serverUrl: string = 'ws://127.0.0.1:63325';
    
    /** Heartbeat interval in milliseconds */
    private heartbeatInterval: number = 3000;
    
    /** Whether the client is connected to the WebSocket server */
    private isConnected: boolean = false;
    
    /** Player's license identifier */
    private playerLicense: string = '';

    /**
     * Initialize the WebSocket client
     */
    constructor() {
        this.setupNuiCallbacks();
        this.updateStatus(false);
        this.log('WebSocket client initialized');
    }

    /**
     * Set up NUI callbacks for FiveM
     * Handles messages from the Lua side
     */
    private setupNuiCallbacks(): void {
        window.addEventListener('message', (event) => {
            const data = event.data;
            
            if (!data || !data.action) return;

            switch (data.action) {
                case 'config':
                    // Handle configuration from Lua side
                    if (data.config) {
                        this.serverUrl = data.config.Url || this.serverUrl;
                        this.heartbeatInterval = data.config.PositionTrackingInterval || this.heartbeatInterval;
                        this.log(`Received config: URL=${this.serverUrl}, Heartbeat=${this.heartbeatInterval}ms`, 'system');
                    }
                    break;
                case 'connect':
                    this.connect(data.license || '');
                    break;
                case 'disconnect':
                    this.disconnect();
                    break;
                case 'sendPosition':
                    this.sendPosition(data.position);
                    break;
                default:
                    this.log(`Unknown action: ${data.action}`);
            }
        });
    }

    /**
     * Connect to the WebSocket server
     * @param license - Player's license identifier
     */
    private connect(license: string): void {
        if (this.isConnected) {
            this.log('Already connected to WebSocket server');
            return;
        }

        this.playerLicense = license;
        this.log(`Connecting to WebSocket server: ${this.serverUrl}`);
        
        try {
            this.socket = new WebSocket(this.serverUrl);
            
            /**
             * Handle WebSocket connection open event
             */
            this.socket.onopen = () => {
                this.isConnected = true;
                this.updateStatus(true);
                this.log('Connected to WebSocket server');
                this.sendNuiCallback('connected', { success: true });
            };
            
            /**
             * Handle WebSocket message event
             * @param event - WebSocket message event
             */
            this.socket.onmessage = (event) => {
                const message = event.data;
                this.log(`Received: ${message}`, 'receive');
                
                //=-- Handle heartbeat messages
                if (typeof message === 'string' && message.toLowerCase().includes('heartbeat')) {
                    this.sendHeartbeatResponse();
                }
            };
            
            /**
             * Handle WebSocket connection close event
             */
            this.socket.onclose = () => {
                this.isConnected = false;
                this.updateStatus(false);
                this.log('Disconnected from WebSocket server');
                this.sendNuiCallback('disconnected', { success: true });
            };
            
            /**
             * Handle WebSocket error event
             * @param error - WebSocket error event
             */
            this.socket.onerror = (error) => {
                this.log(`WebSocket error: ${error}`, 'error');
                this.sendNuiCallback('error', { message: 'WebSocket connection error' });
            };
        } catch (error) {
            this.log(`Failed to connect: ${error}`, 'error');
            this.sendNuiCallback('error', { message: 'Failed to connect to WebSocket server' });
        }
    }

    /**
     * Disconnect from the WebSocket server
     * Closes the connection if it's open
     */
    private disconnect(): void {
        if (!this.isConnected || !this.socket) {
            this.log('Not connected to WebSocket server');
            return;
        }
        
        this.log('Disconnecting from WebSocket server');
        this.socket.close();
    }

    /**
     * Send player position to the WebSocket server
     * @param position - Player's position in the game world
     */
    private sendPosition(position: Position): void {
        if (!this.isConnected || !this.socket) {
            this.log('Not connected to WebSocket server');
            return;
        }
        
        const positionArray = [position.x, position.y, position.z];
        this.log(`Sending position: [${positionArray.join(', ')}]`, 'send');
        this.socket.send(JSON.stringify(positionArray));
    }

    /**
     * Send heartbeat response with player license
     * Called when a heartbeat message is received from the server
     */
    private sendHeartbeatResponse(): void {
        if (!this.isConnected || !this.socket) {
            return;
        }
        
        this.log(`Sending heartbeat response with license: ${this.playerLicense}`, 'send');
        this.socket.send(this.playerLicense);
    }

    /**
     * Send callback to FiveM client
     * @param event - Event name to trigger on the Lua side
     * @param data - Data to send with the event
     */
    private sendNuiCallback(event: string, data: any): void {
        if ('Nui' in window) {
            // @ts-ignore - FiveM NUI object
            window.Nui.callback(event, data);
        } else {
            //=-- Use the correct format for FiveM NUI callbacks
            // @ts-ignore - FiveM specific function
            const resourceName = (window as any).GetParentResourceName ? (window as any).GetParentResourceName() : 'fivem-websocket-test';
            
            fetch(`https://${resourceName}/${event}`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json; charset=UTF-8',
                },
                body: JSON.stringify(data)
            }).catch(error => {
                //=-- Log fetch errors but don't throw
                console.log(`[WebSocket] NUI callback error: ${error}`);
            });
        }
    }

    /**
     * Update connection status in UI
     * @param connected - Whether the client is connected to the WebSocket server
     */
    private updateStatus(connected: boolean): void {
        const statusElement = document.getElementById('status');
        if (statusElement) {
            statusElement.className = `status ${connected ? 'connected' : 'disconnected'}`;
            statusElement.textContent = connected ? 'Connected' : 'Disconnected';
        }
    }

    /**
     * Log messages to the UI and console
     * @param message - Message to log
     * @param type - Type of message (info, error, send, receive)
     */
    private log(message: string, type: string = 'info'): void {
        console.log(`[WebSocket] ${message}`);
        
        const logElement = document.getElementById('log');
        if (logElement) {
            const entry = document.createElement('div');
            entry.className = `log-entry ${type}`;
            entry.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
            logElement.appendChild(entry);
            logElement.scrollTop = logElement.scrollHeight;
        }
    }
}

/**
 * Initialize the WebSocket client
 */
const client = new WebSocketClient();

/**
 * Expose client for debugging in browser
 */
(window as any).wsClient = client;
