--!nonstrict
-- Panel de Noct Effects: presets de Umbriel y tinte del modo Mono.

local PRESETS = {
  { id = "neon",    label = "Neon",    desc = "pulse + glow + viñeta" },
  { id = "cinema",  label = "Cinema",  desc = "scanlines suaves + estela" },
  { id = "matrix",  label = "Matrix",  desc = "scanlines + pulse + viñeta" },
  { id = "mono",    label = "Mono",    desc = "todo gris o duotono" },
  { id = "minimal", label = "Minimal", desc = "solo borde vivo" },
  { id = "off",     label = "Off",     desc = "sin efectos" },
}

local current = noctalia.state.get("preset") or "mono"
local tint = noctalia.state.get("tint")
if tint == "" then
  tint = nil
end
local intensity = tonumber(noctalia.state.get("intensity")) or 100
local status = ""
local busy = false

local function render()

  local rows = {}

  rows[#rows + 1] = ui.row({ align = "center", justify = "space_between" }, {
    ui.row({ gap = 8, align = "center" }, {
      ui.glyph({ name = "palette", size = 18, color = "primary" }),
      ui.label({ text = "Noct Effects", fontSize = 16, fontWeight = "bold" }),
    }),
    ui.button({ glyph = "close", variant = "ghost", onClick = "closePanel" }),
  })

  rows[#rows + 1] = ui.label({
    text = busy and "Aplicando…" or "Efectos de Umbriel",
    fontSize = 12,
    color = "on_surface/0.65",
  })
  rows[#rows + 1] = ui.separator({})

  for _, preset in ipairs(PRESETS) do
    local active = preset.id == current
    rows[#rows + 1] = ui.button({
      key = "preset-" .. preset.id,
      text = (active and "●  " or "○  ") .. preset.label .. "  ·  " .. preset.desc,
      variant = active and "primary" or "ghost",
      contentAlign = "start",
      controlSize = "sm",
      enabled = not busy,
      onClick = function()
        applyPreset(preset.id)
      end,
    })
  end

  rows[#rows + 1] = ui.separator({})

  rows[#rows + 1] = ui.row({ align = "center", justify = "space_between" }, {
    ui.label({ text = "Tinte de Mono", fontSize = 13, fontWeight = "bold" }),
    ui.label({
      text = tint or "gris puro",
      fontSize = 12,
      color = "on_surface/0.65",
    }),
  })

  rows[#rows + 1] = ui.colorPicker({
    key = "tint",
    value = tint or "#ffffff",
    swatches = { "#7cffb2", "#7dd3fc", "#c4b5fd", "#fca5a5", "#fcd34d", "#ffffff" },
    inputs = "hex",
    width = 330,
    onChangeEnd = function(color)
      setTint(color)
    end,
  })

  rows[#rows + 1] = ui.button({
    text = "Gris puro",
    variant = (tint == nil) and "primary" or "outline",
    controlSize = "sm",
    enabled = not busy,
    onClick = function()
      setTint("none")
    end,
  })

  rows[#rows + 1] = ui.row({ align = "center", justify = "space_between" }, {
    ui.label({ text = "Intensidad de Mono", fontSize = 13, fontWeight = "bold" }),
    ui.label({
      text = tostring(intensity) .. "%",
      fontSize = 12,
      color = "on_surface/0.65",
    }),
  })

  rows[#rows + 1] = ui.slider({
    key = "intensity",
    min = 0,
    max = 100,
    step = 1,
    value = intensity,
    enabled = not busy,
    onChange = function(value)
      intensity = math.floor(tonumber(value) or intensity)
      render()
    end,
    onDragEnd = function(value)
      setIntensity(math.floor(tonumber(value) or intensity))
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

local function run(args, done)
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

local function syncState(preset, color, level)
  if preset ~= nil then
    current = preset
  end
  tint = color
  if level ~= nil then
    intensity = math.floor(tonumber(level) or intensity)
  end
  noctalia.state.set("preset", current)
  noctalia.state.set("tint", tint or "")
  noctalia.state.set("intensity", intensity)
end

local function refresh()
  run({ "noct", "effects", "current", "--json" }, function(ok, out)
    if ok then
      local decoded = noctalia.json.decode(out)
      if type(decoded) == "table" then
        syncState(decoded.preset, decoded.tint, decoded.intensity)
      end
    end
    render()
  end)
end

function applyPreset(id)
  if busy then
    return
  end
  busy = true
  status = "Aplicando " .. id .. "…"
  render()
  run({ "noct", "effects", "apply", id }, function(ok, out)
    busy = false
    if ok then
      syncState(id, tint, intensity)
      status = "Preset aplicado: " .. id
      refresh()
    else
      status = "Error: " .. out
      render()
    end
  end)
end

function setIntensity(value)
  if busy then
    return
  end
  busy = true
  status = "Aplicando intensidad " .. value .. "%…"
  render()
  run({ "noct", "effects", "intensity", tostring(value) }, function(ok, out)
    busy = false
    if ok then
      intensity = value
      syncState("mono", tint, intensity)
      status = "Intensidad aplicada: " .. value .. "%"
      refresh()
    else
      status = "Error: " .. out
      render()
    end
  end)
end

function setTint(color)
  if busy then
    return
  end
  busy = true
  status = "Aplicando tinte " .. color .. "…"
  render()
  run({ "noct", "effects", "tint", color }, function(ok, out)
    busy = false
    if ok then
      syncState("mono", color == "none" and nil or color, intensity)
      status = "Tinte aplicado: " .. color
      refresh()
    else
      status = "Error: " .. out
      render()
    end
  end)
end

function onOpen(_context)
  render()
  refresh()
end

function closePanel()
  panel.close()
end
