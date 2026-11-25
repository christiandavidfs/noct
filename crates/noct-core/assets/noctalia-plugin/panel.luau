--!nonstrict
-- Panel de Noct Effects: presets de Umbriel y tinte del modo Mono.

local PRESETS = {
  { id = "neon",    label = "Neon",    desc = "pulse + glow + viñeta",
    tip = "Borde pulsante, halo de cursor, viñeta y animaciones con rebote." },
  { id = "cinema",  label = "Cinema",  desc = "scanlines suaves + estela",
    tip = "Viñeta de pantalla, scanlines por ventana y estela corta de cursor." },
  { id = "matrix",  label = "Matrix",  desc = "scanlines + pulse + viñeta",
    tip = "Scanlines, borde pulsante, rastro largo de cursor y barrido de workspaces." },
  { id = "minimal", label = "Minimal", desc = "solo borde vivo",
    tip = "Solo un borde vivo alrededor de la ventana enfocada." },
  { id = "custom",  label = "Libre",   desc = "solo lo que mezcles",
    tip = "Empieza sin efectos y añade solo los que elijas en Mezclar." },
  { id = "mono",    label = "Mono",    desc = "todo gris o duotono",
    tip = "Todo el escritorio en gris con scanlines y viñeta; aquí aplican tinte e intensidad." },
  { id = "off",     label = "Off",     desc = "sin efectos",
    tip = "Desactiva todos los efectos. La mezcla guardada se conserva." },
}

local current = noctalia.state.get("preset") or "mono"
local tint = noctalia.state.get("tint")
if tint == "" then
  tint = nil
end
local intensity = tonumber(noctalia.state.get("intensity")) or 100
local overrides = {}
local effects = {}
local status = ""
local busy = false

local function countKeys(tableValue)
  local count = 0
  if type(tableValue) == "table" then
    for _ in pairs(tableValue) do
      count = count + 1
    end
  end
  return count
end

local function screenName()
  for _, entry in ipairs(effects) do
    local kind, name = string.match(entry, "^([^=]+)=(.*)$")
    if kind == "screen" then
      return name
    end
  end
  return nil
end

local function monoActive()
  return screenName() == "noct-mono"
end

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
      tooltip = preset.tip,
      enabled = not busy,
      onClick = function()
        applyPreset(preset.id)
      end,
    })
  end

  local mixLabel = "Mezclar efectos…"
  if countKeys(overrides) > 0 then
    mixLabel = "Mezclar efectos (" .. countKeys(overrides) .. ")…"
  end

  rows[#rows + 1] = ui.button({
    text = mixLabel,
    variant = "outline",
    controlSize = "sm",
    tooltip = "Elige el efecto de cada slot: borde, cursor, pantalla, ventana y animaciones.",
    enabled = not busy,
    onClick = function()
      panel.close()
      noctalia.togglePanel("noct/effects:mix")
    end,
  })

  rows[#rows + 1] = ui.separator({})

  if not monoActive() then
    local screen = screenName()
    rows[#rows + 1] = ui.label({
      text = "Mono no está activo"
        .. (screen and (" · Pantalla = " .. screen) or "")
        .. ". Ajusta el tinte o la intensidad para activarlo.",
      fontSize = 11,
      color = "on_surface/0.6",
      maxWidth = 340,
    })
  end

  rows[#rows + 1] = ui.row({ align = "center", justify = "space_between",
    tooltip = "El tinte solo afecta a Mono. Si Mono no está activo, cambiarlo lo activa." }, {
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
    tooltip = "Quita el tinte y deja Mono en blanco y negro.",
    enabled = not busy,
    onClick = function()
      setTint("none")
    end,
  })

  rows[#rows + 1] = ui.row({ align = "center", justify = "space_between",
    tooltip = "La intensidad solo afecta a Mono. Si Mono no está activo, moverla lo activa." }, {
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

local function syncState(preset, color, level, ov, fx)
  if preset ~= nil then
    current = preset
  end
  tint = color
  if level ~= nil then
    intensity = math.floor(tonumber(level) or intensity)
  end
  if type(ov) == "table" then
    overrides = ov
  end
  if type(fx) == "table" then
    effects = fx
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
        syncState(decoded.preset, decoded.tint, decoded.intensity, decoded.overrides, decoded.effects)
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
  run({ "noct", "effects", "intensity", tostring(value), "--json" }, function(ok, out)
    busy = false
    if ok then
      local decoded = noctalia.json.decode(out)
      if type(decoded) == "table" and type(decoded.message) == "string" then
        status = decoded.message
      else
        status = "Intensidad aplicada: " .. value .. "%"
      end
      intensity = value
      syncState(nil, tint, intensity)
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
  run({ "noct", "effects", "tint", color, "--json" }, function(ok, out)
    busy = false
    if ok then
      local decoded = noctalia.json.decode(out)
      if type(decoded) == "table" and type(decoded.message) == "string" then
        status = decoded.message
      else
        status = "Tinte aplicado: " .. color
      end
      syncState(nil, color == "none" and nil or color, intensity)
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
