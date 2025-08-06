--//=-- Server-side script for WebSocket test client
--//=-- This is a minimal server script since most functionality is in the client/NUI

---@description Event handler to retrieve player license and send it to the client
---@return nil
RegisterNetEvent('ITGH_websocket:getPlayerLicense')
AddEventHandler('ITGH_websocket:getPlayerLicense', function()
    local src = source
    local identifiers = GetPlayerIdentifiers(src)
    local license = "license:unknown"
    
    --//=-- Find the license identifier
    for _, v in pairs(identifiers) do
        if string.sub(v, 1, string.len("license:")) == "license:" then
            license = v
            break
        end
    end
    
    --//=-- Send the license back to the client
    TriggerClientEvent('ITGH_websocket:setPlayerLicense', src, license)
end)

--//=-- Print to console when resource starts
AddEventHandler('onResourceStart', function(resourceName)
    if (GetCurrentResourceName() ~= resourceName) then
        return
    end
    print(resourceName .. ' has been started.')
end)

--//=-- Print to console when resource stops
AddEventHandler('onResourceStop', function(resourceName)
    if (GetCurrentResourceName() ~= resourceName) then
        return
    end
    print(resourceName .. ' has been stopped.')
end)
