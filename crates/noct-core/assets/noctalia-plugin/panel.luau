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

local function syncState(preset, color)
  if preset ~= nil then
    current = preset
  end
  tint = color
  noctalia.state.set("preset", current)
  noctalia.state.set("tint", tint or "")
end

local function refresh()
  run({ "noct", "effects", "current", "--json" }, function(ok, out)
    if ok then
      local decoded = noctalia.json.decode(out)
      if type(decoded) == "table" then
        syncState(decoded.preset, decoded.tint)
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
      syncState(id, tint)
      status = "Preset aplicado: " .. id
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
      syncState("mono", color == "none" and nil or color)
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
