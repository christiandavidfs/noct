--!nonstrict
-- Tile del control center: muestra el preset activo y abre el panel.

local SHORT = {
  neon = "Neon",
  cinema = "Cine",
  matrix = "Matrix",
  mono = "Mono",
  minimal = "Min",
  custom = "Libre",
  off = "Off",
}

local current = noctalia.state.get("preset") or "mono"

local function render()
  shortcut.setLabel(SHORT[current] or current)
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
