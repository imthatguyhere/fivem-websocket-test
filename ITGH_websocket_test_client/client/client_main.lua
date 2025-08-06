--//=-- Unified function for sending messages to both chat and console
function SendMessage(source, message, messageType)
    --//=-- Define color based on message type
    local colors = {
        info = {255, 255, 0},    -- Yellow
        success = {0, 255, 0},   -- Green
        error = {255, 0, 0},     -- Red
        warning = {255, 165, 0},  -- Orange
        system = {0, 255, 255}    -- Cyan
    }
    
    --//=-- Default to info if type not specified
    local msgType = messageType or "info"
    local color = colors[msgType] or colors.info
    
    --//=-- Send to chat
    TriggerEvent('chat:addMessage', {
        color = color,
        multiline = true,
        args = {source or "WebSocket", message}
    })
    
    --//=-- Log to console with prefix based on type
    local prefix = "[" .. (source or "WebSocket") .. "] "
    if msgType == "error" then
        print(prefix .. "^1" .. message .. "^7") -- Red
    elseif msgType == "success" then
        print(prefix .. "^2" .. message .. "^7") -- Green
    elseif msgType == "warning" then
        print(prefix .. "^3" .. message .. "^7") -- Yellow
    elseif msgType == "system" then
        print(prefix .. "^5" .. message .. "^7") -- Cyan
    else
        print(prefix .. message) -- Default
    end
end

---@type boolean Whether the NUI is ready to receive messages
local isNuiReady = false

---@type boolean Whether the WebSocket connection is established
local websocketConnected = false

---@type string|nil The player's license identifier from the server
local playerLicense = nil -- Will store the player's license once received from server

---@class MessageType
---@field info string Yellow color
---@field success string Green color
---@field error string Red color
---@field warning string Orange color
---@field system string Cyan color

---@param source string The source of the message
---@param message string The message content
---@param messageType string The type of message (info, success, error, warning, system)
---@return nil
function SendMessage(source, message, messageType)
    --//=-- Define color based on message type
    local colors = {
        info = {255, 255, 0},    -- Yellow
        success = {0, 255, 0},   -- Green
        error = {255, 0, 0},     -- Red
        warning = {255, 165, 0},  -- Orange
        system = {0, 255, 255}    -- Cyan
    }
    
    --//=-- Default to info if type not specified
    local msgType = messageType or "info"
    local color = colors[msgType] or colors.info
    
    --//=-- Send to chat
    TriggerEvent('chat:addMessage', {
        color = color,
        multiline = true,
        args = {source or "WebSocket", message}
    })
    
    --//=-- Log to console with prefix based on type
    local prefix = "[" .. (source or "WebSocket") .. "] "
    if msgType == "error" then
        print(prefix .. "^1" .. message .. "^7") -- Red
    elseif msgType == "success" then
        print(prefix .. "^2" .. message .. "^7") -- Green
    elseif msgType == "warning" then
        print(prefix .. "^3" .. message .. "^7") -- Yellow
    elseif msgType == "system" then
        print(prefix .. "^5" .. message .. "^7") -- Cyan
    else
        print(prefix .. message) -- Default
    end
end

--//=-- Initialize the resource
Citizen.CreateThread(function()
    --//=-- Wait for NUI to be ready
    SetNuiFocus(false, false)
    
    --//=-- Send configuration to NUI using shared config values
    SendNUIMessage({
        action = "config",
        config = {
            Url = Config.WebSocket.Url,
            PositionTrackingInterval = Config.WebSocket.PositionTrackingInterval
        }
    })
    
    --//=-- Request player license from server
    RequestPlayerLicense()
    
    -- Register commands
    RegisterCommand("ws_connect", function(source, args, rawCommand)
        ConnectWebSocket()
    end, false)
    
    RegisterCommand("ws_disconnect", function(source, args, rawCommand)
        DisconnectWebSocket()
    end, false)
    
    RegisterCommand("ws_position", function(source, args, rawCommand)
        SendPlayerPosition()
    end, false)
    
    -- Send position periodically when connected (if enabled in config)
    Citizen.CreateThread(function()
        while true do
            -- Use the interval from config
            Citizen.Wait(Config.WebSocket.PositionTrackingInterval)
            -- Only send if connected and auto-send is enabled
            if websocketConnected and Config.WebSocket.AutoSendPosition then
                SendPlayerPosition()
            end
            Citizen.Wait(0)
        end
    end)
end)

--//=-- NUI Callbacks
RegisterNUICallback('connected', function(data, cb)
    websocketConnected = true
    SendMessage("WebSocket", "Connected to server", "success")
    cb('ok')
end)

RegisterNUICallback('disconnected', function(data, cb)
    websocketConnected = false
    SendMessage("WebSocket", "Disconnected from server", "error")
    cb('ok')
end)

RegisterNUICallback('error', function(data, cb)
    SendMessage("WebSocket", "Error: " .. (data.message or "Unknown error"), "error")
    cb('ok')
end)

---@description Connects to the WebSocket server using the player's license or a fallback ID
function ConnectWebSocket()
    --//=-- Use the license we got from the server, or fallback if not available yet
    local licenseToUse = playerLicense
    
    if licenseToUse == nil then
        --//=-- License not received yet, request it and use a temporary ID
        RequestPlayerLicense()
        licenseToUse = "client:" .. GetPlayerServerId(PlayerId())
        
        --//=-- Notify user that we're using a temporary ID
        SendMessage("WebSocket", "Using temporary ID. License not yet received from server.", "warning")
    end
    
    -- Send connect message to NUI
    SendNUIMessage({
        action = "connect",
        license = licenseToUse
    })
    
    SendMessage("WebSocket", "Connecting to server...", "info")
end

---@description Disconnects from the WebSocket server by sending a disconnect message to NUI
function DisconnectWebSocket()
    SendNUIMessage({
        action = "disconnect"
    })
    
    SendMessage("WebSocket", "Disconnecting from server...", "info")
end

---@description Sends the player's current position to the WebSocket server via NUI
function SendPlayerPosition()
    local playerPed = PlayerPedId()
    local coords = GetEntityCoords(playerPed)
    
    SendNUIMessage({
        action = "sendPosition",
        position = {
            x = coords.x,
            y = coords.y,
            z = coords.z
        }
    })
end

---@description Requests the player's license identifier from the server
---@return nil
function RequestPlayerLicense()
    --//=-- Trigger server event to get the license
    TriggerServerEvent('ITGH_websocket:getPlayerLicense')
    
    --//=-- Debug message
    print("Requesting player license from server")
end

--//=-- Event handler to receive player license from server
RegisterNetEvent('ITGH_websocket:setPlayerLicense')
AddEventHandler('ITGH_websocket:setPlayerLicense', function(license)
    --//=-- Store the license for the session
    playerLicense = license
    
    --//=-- Debug message
    print("Received player license: " .. playerLicense)
    
    --//=-- Notify in chat and console
    SendMessage("WebSocket", "Player license received", "system")
end)

--//=-- Debug: Print to console when resource starts
print("WebSocket Test Client loaded")
