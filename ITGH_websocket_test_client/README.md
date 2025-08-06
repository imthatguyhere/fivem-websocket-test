# FiveM WebSocket Test Client

This resource allows FiveM to communicate with a WebSocket server at `ws://127.0.0.1:63325` using NUI.

## Features

- Connect to a WebSocket server
- Disconnect from the WebSocket server
- Send player position data as a JSON array
- Automatically respond to heartbeat messages with the player's FiveM license
- Configurable position tracking interval and auto-sending

## Commands

- `/ws_connect` - Connect to the WebSocket server
- `/ws_disconnect` - Disconnect from the WebSocket server
- `/ws_position` - Send current player position to the WebSocket server

## Installation

1. Clone this repository to your FiveM resources folder
2. Build the NUI interface:
   ```
   cd nui
   npm install
   npm run build
   ```
3. Add `ensure ITGH_websocket_test_client` to your server.cfg
4. Start your FiveM server

## Development

For development, you can use the watch mode:

```
cd nui
npm run dev
```

This will automatically rebuild the NUI interface when changes are detected.

## Configuration

The resource can be configured by editing the `shared/shared_config.lua` file:

```lua
Config.WebSocket = {
    Url = "ws://127.0.0.1:63325",              -- WebSocket server URL
    PositionTrackingInterval = 30000,          -- Interval in milliseconds for position tracking (default: 30 seconds)
    AutoSendPosition = true                    -- Set to false to disable automatic position updates
}
```

### Configuration Options

- **Url**: The WebSocket server URL to connect to
- **PositionTrackingInterval**: How often (in milliseconds) to send position updates when connected
- **AutoSendPosition**: When true, automatically sends position updates at the specified interval. When false, position updates are only sent manually via the `/ws_position` command

## How It Works

1. The NUI interface creates a WebSocket connection to the specified server
2. When connected, it will respond to heartbeat messages with the player's license
3. The client can send the player's position data as a JSON array
4. All communication is logged in the NUI interface

## Requirements

- FiveM server
- WebSocket server running at ws://127.0.0.1:63325

## Author

Imthatguyhere (ITGH | Tyler)
