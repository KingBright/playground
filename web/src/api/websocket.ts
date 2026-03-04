export interface WebSocketMessage {
  type: string;
  data: any;
}

type Listener = (...args: any[]) => void;

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private url: string;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectTimeout = 1000;
  private listeners: Record<string, Listener[]> = {};

  constructor(url: string) {
    this.url = url;
  }

  on(event: string, listener: Listener) {
    if (!this.listeners[event]) {
      this.listeners[event] = [];
    }
    this.listeners[event].push(listener);
  }

  off(event: string, listener: Listener) {
    if (!this.listeners[event]) return;
    this.listeners[event] = this.listeners[event].filter(l => l !== listener);
  }

  private emit(event: string, ...args: any[]) {
    if (!this.listeners[event]) return;
    this.listeners[event].forEach(listener => listener(...args));
  }

  connect() {
    if (this.ws?.readyState === WebSocket.OPEN) return;

    this.ws = new WebSocket(this.url);

    this.ws.onopen = () => {
      console.log(`WebSocket connected to ${this.url}`);
      this.reconnectAttempts = 0;
      this.emit('connected');
    };

    this.ws.onmessage = (event) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data);
        this.emit('message', message);
        this.emit(message.type, message.data);
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };

    this.ws.onclose = () => {
      console.log(`WebSocket disconnected from ${this.url}`);
      this.emit('disconnected');
      this.attemptReconnect();
    };

    this.ws.onerror = (error) => {
      console.error(`WebSocket error on ${this.url}:`, error);
      this.emit('error', error);
    };
  }

  disconnect() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  private attemptReconnect() {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      this.reconnectAttempts++;
      setTimeout(() => {
        console.log(`Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`);
        this.connect();
      }, this.reconnectTimeout * Math.pow(2, this.reconnectAttempts - 1));
    } else {
      console.error('Max reconnect attempts reached');
    }
  }
}

const WS_BASE_URL = import.meta.env.VITE_WS_URL || 'ws://localhost:8080/api/ws';

export const createSessionWebSocket = (sessionId: string) => {
  return new WebSocketClient(`${WS_BASE_URL}/sessions/${sessionId}`);
};

export const createMissionsWebSocket = () => {
  return new WebSocketClient(`${WS_BASE_URL}/missions`);
};

export const createSystemWebSocket = () => {
  return new WebSocketClient(`${WS_BASE_URL}/system`);
};

import { useEffect, useState } from 'react';

export function useWebSocket<T>(
  clientGetter: () => WebSocketClient,
  messageType: string,
  initialData: T | null = null
) {
  const [data, setData] = useState<T | null>(initialData);
  const [isConnected, setIsConnected] = useState(false);

  useEffect(() => {
    const client = clientGetter();

    const handleMessage = (newData: T) => {
      setData(newData);
    };

    const handleConnect = () => setIsConnected(true);
    const handleDisconnect = () => setIsConnected(false);

    client.on(messageType, handleMessage);
    client.on('connected', handleConnect);
    client.on('disconnected', handleDisconnect);

    client.connect();

    return () => {
      client.off(messageType, handleMessage);
      client.off('connected', handleConnect);
      client.off('disconnected', handleDisconnect);
      client.disconnect();
    };
  }, [messageType]); // Removed clientGetter from dependencies to prevent infinite loop

  return { data, isConnected };
}
