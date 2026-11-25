--!nonstrict
-- Panel de mezcla: elige el efecto de cada slot sin cambiar de preset.

local SLOTS = {
  { key = "border",       label = "Borde" },
  { key = "cursor",       label = "Cursor" },
  { key = "screen",       label = "Pantalla" },
  { key = "window",       label = "Ventana" },
  { key = "windows_move", label = "Anim. ventanas" },
  { key = "windows_in",   label = "Anim. apertura" },
  { key = "windows_out",  label = "Anim. cierre" },
  { key = "workspaces",   label = "Anim. workspaces" },
}

local preset = ""
local overrides = {}
local effective = {}
local choices = {}
local status = ""
local busy = false

local render
local run
local refresh

local function optionList(key)
  local list = { "auto", "none" }
  local seen = { auto = true, none = true }
  local items = choices[key]
  if type(items) == "table" then
    for _, value in ipairs(items) do
      if not seen[value] then
        list[#list + 1] = value
        seen[value] = true
      end
    end
  end
  return list
end

local function overrideValue(key)
  local value = overrides[key]
  if type(value) == "string" and value ~= "" then
    return value
  end
  return "auto"
end

local function displayValue(key)
  local value = overrideValue(key)
  if value == "auto" then
    local active = effective[key]
    if type(active) == "string" and active ~= "" then
      return "Auto · " .. active
    end
    return "Auto"
  elseif value == "none" then
    return "Off"
  end
  return value
end

local function setSlot(key, value)
  if busy then
    return
  end
  busy = true
  status = "Aplicando " .. key .. " = " .. value .. "…"
  render()

  local args
  if value == "auto" then
    args = { "noct", "effects", "mix", "clear", key }
  else
    args = { "noct", "effects", "mix", "set", key, value }
  end

  run(args, function(ok, out)
    busy = false
    if ok then
      status = value == "auto" and ("'" .. key .. "' vuelve al preset") or ("'" .. key .. "' = " .. value)
      refresh()
    else
      status = "Error: " .. out
      render()
    end
  end)
end

local function cycle(key)
  local list = optionList(key)
  local current = overrideValue(key)
  local index = 1
  for i, value in ipairs(list) do
    if value == current then
      index = i
      break
    end
  end
  local nextValue = list[(index % #list) + 1]
  setSlot(key, nextValue)
end

local function clearAll()
  if busy then
    return
  end
  busy = true
  status = "Limpiando mezcla…"
  render()
  run({ "noct", "effects", "mix", "clear" }, function(ok, out)
    busy = false
    if ok then
      status = "Mezcla limpiada"
      refresh()
    else
      status = "Error: " .. out
      render()
    end
  end)
end

function refresh()
  run({ "noct", "effects", "mix", "show", "--json" }, function(ok, out)
    if ok then
      local decoded = noctalia.json.decode(out)
      if type(decoded) == "table" then
        preset = decoded.preset or ""
        overrides = decoded.overrides or {}
        effective = decoded.effective or {}
        choices = decoded.choices or {}
      end
    end
    render()
  end)
end

function render()
  local rows = {}

  rows[#rows + 1] = ui.row({ align = "center", justify = "space_between" }, {
    ui.row({ gap = 8, align = "center" }, {
      ui.glyph({ name = "palette", size = 18, color = "primary" }),
      ui.label({ text = "Mezclar efectos", fontSize = 16, fontWeight = "bold" }),
    }),
    ui.button({ glyph = "close", variant = "ghost", onClick = "closePanel" }),
  })

  rows[#rows + 1] = ui.label({
    text = "Preset: " .. (preset ~= "" and preset or "ninguno"),
    fontSize = 12,
    color = "on_surface/0.65",
  })
  rows[#rows + 1] = ui.label({
    text = "Toca el valor para ciclar: Auto → Off → efectos instalados.",
    fontSize = 11,
    color = "on_surface/0.55",
    maxWidth = 340,
  })
  rows[#rows + 1] = ui.separator({})

  for _, slot in ipairs(SLOTS) do
    local value = overrideValue(slot.key)
    rows[#rows + 1] = ui.row({ align = "center", justify = "space_between" }, {
      ui.label({ text = slot.label, fontSize = 13, fontWeight = "bold" }),
      ui.button({
        text = displayValue(slot.key),
        variant = value == "auto" and "ghost" or "outline",
        controlSize = "sm",
        enabled = not busy,
        onClick = function()
          cycle(slot.key)
        end,
      }),
    })
  end

  rows[#rows + 1] = ui.separator({})
  rows[#rows + 1] = ui.button({
    text = "Limpiar toda la mezcla",
    variant = "outline",
    controlSize = "sm",
    enabled = not busy,
    onClick = function()
      clearAll()
    end,
  })

  if status ~= "" then
    rows[#rows + 1] = ui.label({
      text = status,
      fontSize = 12,
      color = "on_surface/0.75",
      maxWidth = 340,
    })
  end

  panel.render(ui.column({ padding = 14, gap = 8 }, rows))
end

function run(args, done)
  local started = noctalia.runAsync(args, function(result)
    local ok = result ~= nil and not result.timedOut and result.exitCode == 0
    local out = ""
    if result ~= nil then
      if ok then
        out = result.stdout or ""
      else
        out = (result.stderr ~= "" and result.stderr) or (result.stdout or "")
      end
    end
    done(ok, out)
  end)
  if not started then
    done(false, "no se pudo ejecutar noct")
  end
end

function onOpen(_context)
  render()
  refresh()
end

function closePanel()
  panel.close()
end
