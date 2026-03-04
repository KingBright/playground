/**
 * WebSocket Client for real-time updates
 */

const WS_BASE_URL = 'ws://localhost:8080/api/ws';

export type WebSocketMessage =
  | { type: 'connected'; session_id?: string; message: string }
  | { type: 'session_status'; session_id: string; status: string; agent_count: number; snapshot_count: number; timestamp: string }
  | { type: 'heartbeat'; timestamp: string; active_missions: number }
  | { type: 'error'; message: string }
  | { type: 'ack'; received: string }
  | { type: 'subscribed'; channel: unknown };

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private reconnectInterval = 5000;
  private messageHandlers: ((msg: WebSocketMessage) => void)[] = [];
  private isConnected = false;

  constructor(private url: string) {}

  connect(): void {
    try {
      this.ws = new WebSocket(this.url);

      this.ws.onopen = () => {
        console.log('WebSocket connected:', this.url);
        this.isConnected = true;
      };

      this.ws.onmessage = (event) => {
        try {
          const msg: WebSocketMessage = JSON.parse(event.data);
          this.messageHandlers.forEach((handler) => handler(msg));
        } catch (e) {
          console.error('Failed to parse WebSocket message:', e);
        }
      };

      this.ws.onclose = () => {
        console.log('WebSocket disconnected, reconnecting in', this.reconnectInterval, 'ms');
        this.isConnected = false;
        setTimeout(() => this.connect(), this.reconnectInterval);
      };

      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };
    } catch (error) {
      console.error('Failed to connect WebSocket:', error);
      setTimeout(() => this.connect(), this.reconnectInterval);
    }
  }

  disconnect(): void {
    this.ws?.close();
    this.ws = null;
    this.isConnected = false;
  }

  send(message: unknown): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    } else {
      console.warn('WebSocket is not open');
    }
  }

  onMessage(handler: (msg: WebSocketMessage) => void): () => void {
    this.messageHandlers.push(handler);
    return () => {
      const index = this.messageHandlers.indexOf(handler);
      if (index > -1) {
        this.messageHandlers.splice(index, 1);
      }
    };
  }

  get connected(): boolean {
    return this.isConnected;
  }
}

// Session WebSocket - 订阅特定Session的状态更新
export function createSessionWebSocket(sessionId: string): WebSocketClient {
  const client = new WebSocketClient(`${WS_BASE_URL}/sessions/${sessionId}`);
  client.connect();
  return client;
}

// Missions WebSocket - 订阅任务更新
export function createMissionsWebSocket(): WebSocketClient {
  const client = new WebSocketClient(`${WS_BASE_URL}/missions`);
  client.connect();
  return client;
}

// React Hook for WebSocket
export function useWebSocket(url: string) {
  const [messages, setMessages] = React.useState<WebSocketMessage[]>([]);
  const [connected, setConnected] = React.useState(false);
  const clientRef = React.useRef<WebSocketClient | null>(null);

  React.useEffect(() => {
    const client = new WebSocketClient(url);
    clientRef.current = client;

    const unsubscribe = client.onMessage((msg) => {
      setMessages((prev) => [...prev, msg]);
      if (msg.type === 'connected') {
        setConnected(true);
      }
    });

    client.connect();

    return () => {
      unsubscribe();
      client.disconnect();
    };
  }, [url]);

  return {
    messages,
    connected,
    send: (msg: unknown) => clientRef.current?.send(msg),
    clearMessages: () => setMessages([]),
  };
}

// Need to import React for the hook
import React from 'react';
