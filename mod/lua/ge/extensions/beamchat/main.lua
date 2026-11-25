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

-- For retrying when connection fails
local retryIn = 0
local connected = false

-- Update management
local updateFreq = 10 -- 10hz
local updateIn = 0

local tcp, err
function M.onUpdate(dtReal)
    if connected then
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

            -- Add newline to determine packet borders
            local sent, err = tcp:send(json.encode(packet) .. "\n")
            if sent == nil then
                log("E", "beamchat", "tcp send failed, starting new connection: " .. tostring(err))
                tcp:close()
                connected = false
            end
        end

        return
    end

    -- If not connected
    retryIn = retryIn - dtReal
    if retryIn <= 0 then
        -- Create TCP stream
        tcp, err = assert(socket.tcp())
        if not tcp then
            log("E", "beamchat", "socket.tcp failed: " .. tostring(err))

            retryIn = 5
            return
        end

        tcp:settimeout(3)

        local ok, connectErr = tcp:connect(host, port)
        if not ok then
            log("E", "beamchat", "connect failed, retrying in 5 seconds: " .. tostring(connectErr))
            tcp:close()
            
            retryIn = 5
            return
        end

        log("I", "beamchat", "Connected to client")
        connected = true
    end
end

return M