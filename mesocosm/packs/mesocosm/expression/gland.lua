-- This Source Code Form is subject to the terms of the Mozilla Public
-- License, v. 2.0. If a copy of the MPL was not distributed with this
-- file, You can obtain one at https://mozilla.org/MPL/2.0/.
-- SPDX-License-Identifier: MPL-2.0
--
-- Where a line puts a gland. (PD4)
--
-- This is the whole of what an author writes: read the frozen request, decide
-- what the body should express, say so. It cannot spend, cannot place a cell,
-- cannot reach a world, and cannot be sure of being agreed with -- the host
-- prices the result, lays the tissue out, and runs the one validator, which is
-- free to refuse everything below.
--
-- Nothing here is hardcoded that the request already says. The definition's
-- own site requirement decides which shape to look for, the part's own cell
-- price decides whether the ground can charge what is being asked for, and the
-- candidate list decides whether this line may ask at all.

local GLAND = "mesocosm:secrete"

-- What the line is minded to spend, before the ground is consulted: four
-- cells, or five. The host drew the number; this only reads it.
local function appetite(entropy)
    return 4 + (entropy[1] % 2)
end

function express(request, entropy)
    -- A line can only express what it has come to. Nothing else in this file
    -- matters if this is false.
    local granted = false
    for _, id in ipairs(request.candidates) do
        if id == GLAND then
            granted = true
        end
    end
    if not granted then
        return { sites = {} }
    end

    -- The site requirement, read off the admitted definition rather than
    -- assumed. If this world admits the gland on some other shape, this script
    -- follows it there.
    local wants = nil
    for _, definition in ipairs(request.definitions) do
        if definition.id == GLAND then
            wants = definition.expressed_by[1]
        end
    end
    if wants == nil then
        return { sites = {} }
    end

    -- The first part of that shape, in part order, so the answer does not
    -- depend on which one was reached first.
    local target = nil
    for _, part in ipairs(request.parts) do
        if target == nil and part.role == wants then
            target = part
        end
    end
    if target == nil then
        return { sites = {} }
    end

    -- **The developmental context that actually decides the phenotype.** A
    -- gland is only ever as strong as the ground under the body can charge, and
    -- it costs its rent whether or not it is working. So on rich ground the
    -- line spends what it is minded to, and on lean ground it keeps a token
    -- gland and leaves the rest of the frond doing what it was doing.
    local ground = request.conditions.ground_mg or 0
    local cells = appetite(entropy)
    if ground < target.cell_mg * cells then
        cells = 1
    end
    if cells > target.cells then
        cells = target.cells
    end

    return {
        sites = {
            { part = target.part, process = GLAND, cells = cells },
        },
    }
end
