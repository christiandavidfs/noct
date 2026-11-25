--!nonstrict
-- Control-center tile: shows the active preset and opens the panel.

local function tr(key)
  local value = noctalia.tr(key)
  if type(value) == "string" and value ~= "" then
    return value
  end
  return key
end

local SHORT = {
  neon = "shortcut.neon",
  cinema = "shortcut.cinema",
  matrix = "shortcut.matrix",
  mono = "shortcut.mono",
  minimal = "shortcut.minimal",
  custom = "shortcut.custom",
  off = "shortcut.off",
}

local current = noctalia.state.get("preset") or "mono"

local function render()
  shortcut.setLabel(tr(SHORT[current]) or current)
  shortcut.setIcon("palette")
  shortcut.setActive(current ~= "off")
end

function onClick()
  noctalia.togglePanel("noct/effects:panel")
end

noctalia.state.watch("preset", function(value)
  current = value or current
  render()
end)

noctalia.runAsync({ "noct", "effects", "current", "--json" }, function(result)
  if result ~= nil and not result.timedOut and result.exitCode == 0 then
    local decoded = noctalia.json.decode(result.stdout)
    if type(decoded) == "table" and type(decoded.preset) == "string" then
      current = decoded.preset
      noctalia.state.set("preset", current)
    end
  end
  render()
end)

render()
