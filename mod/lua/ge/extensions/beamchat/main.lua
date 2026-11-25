local M = {}

local host, port = "127.0.0.1", 5555
local socket = require('socket')
local json = require('json')

function vec3toList(vec)
    return {vec.x, vec.y, vec.z}
end

function quatToList(quat)
    return {quat.x, quat.y, quat.z, quat.w}
end

function M.onExtensionLoaded()
    log("I", "beamchat", "BeamChat GELua extension loaded")
end

-- Update management
local updateFreq = 10 -- 10hz
local updateIn = 0

function M.onUpdate(dtReal)
    -- Next update timer
    updateIn = updateIn - dtReal
    if updateIn <= 0 then
        -- Prevents multiple successive updates after lag spike
        if updateIn > 2 * (1 / updateFreq) then updateIn = 0 end
        updateIn = updateIn + (1 / updateFreq)

        local vehicles = {}
        -- TODO: Find more efficient way to iterate just vehicles
        for i = 0, be:getObjectCount() - 1 do
            local vehicle = be:getObject(i)
            if vehicle then
                vehicles[#vehicles + 1] = {
                    id = vehicle:getId(),
                    position = vec3toList(vehicle:getPosition())
                }
            end
        end

        local packet = {
            camera = {
                position = vec3toList(core_camera.getPosition()),
                rotation = quatToList(core_camera.getQuat())
            },
            vehicles = vehicles
        }

        local udp = assert(socket.udp())
        udp:setpeername(host, port)
        udp:send(json.encode(packet))
    end

    return
end

return M