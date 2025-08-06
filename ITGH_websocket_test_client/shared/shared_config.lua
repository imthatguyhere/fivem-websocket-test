--//=-- Shared configuration for WebSocket test client
---@class Config
---@field WebSocket WebSocketConfig WebSocket configuration settings
Config = {}

---@class WebSocketConfig
---@field Url string WebSocket server URL
---@field PositionTrackingInterval number Interval in milliseconds for position tracking
---@field AutoSendPosition boolean Whether to automatically send position updates
Config.WebSocket = {
    Url = "ws://127.0.0.1:63325",
    PositionTrackingInterval = 30000, -- 30 seconds
    AutoSendPosition = true -- Set to false to disable automatic position updates
}
