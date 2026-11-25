--!nonstrict
-- Noct Effects panel: Umbriel presets, mix and Mono tint/intensity.

local function tr(key, args)
  local value = noctalia.tr(key, args)
  if type(value) == "string" and value ~= "" then
    return value
  end
  return key
end

local PRESETS = {
  { id = "neon",    label_key = "panel.presets.neon.name",    desc_key = "panel.presets.neon.desc",    tip_key = "panel.presets.neon.tip" },
  { id = "cinema",  label_key = "panel.presets.cinema.name",  desc_key = "panel.presets.cinema.desc",  tip_key = "panel.presets.cinema.tip" },
  { id = "matrix",  label_key = "panel.presets.matrix.name",  desc_key = "panel.presets.matrix.desc",  tip_key = "panel.presets.matrix.tip" },
  { id = "minimal", label_key = "panel.presets.minimal.name", desc_key = "panel.presets.minimal.desc", tip_key = "panel.presets.minimal.tip" },
  { id = "custom",  label_key = "panel.presets.custom.name",  desc_key = "panel.presets.custom.desc",  tip_key = "panel.presets.custom.tip" },
  { id = "mono",    label_key = "panel.presets.mono.name",    desc_key = "panel.presets.mono.desc",    tip_key = "panel.presets.mono.tip" },
  { id = "off",     label_key = "panel.presets.off.name",     desc_key = "panel.presets.off.desc",     tip_key = "panel.presets.off.tip" },
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

local function presetName(id)
  return tr("panel.presets." .. id .. ".name")
end

local function render()

  local rows = {}

  rows[#rows + 1] = ui.row({ align = "center", justify = "space_between" }, {
    ui.row({ gap = 8, align = "center" }, {
      ui.glyph({ name = "palette", size = 18, color = "primary" }),
      ui.label({ text = tr("panel.title"), fontSize = 16, fontWeight = "bold" }),
    }),
    ui.button({ glyph = "close", variant = "ghost", onClick = "closePanel" }),
  })

  rows[#rows + 1] = ui.label({
    text = busy and tr("panel.applying") or tr("panel.subtitle"),
    fontSize = 12,
    color = "on_surface/0.65",
  })
  rows[#rows + 1] = ui.separator({})

  for _, preset in ipairs(PRESETS) do
    local active = preset.id == current
    rows[#rows + 1] = ui.button({
      key = "preset-" .. preset.id,
      text = (active and "●  " or "○  ") .. tr(preset.label_key) .. "  ·  " .. tr(preset.desc_key),
      variant = active and "primary" or "ghost",
      contentAlign = "start",
      controlSize = "sm",
      tooltip = tr(preset.tip_key),
      enabled = not busy,
      onClick = function()
        applyPreset(preset.id)
      end,
    })
  end

  local overrideCount = countKeys(overrides)
  local mixLabel = tr("panel.mix_button")
  if overrideCount > 0 then
    mixLabel = tr("panel.mix_button_count", { count = overrideCount })
  end

  rows[#rows + 1] = ui.button({
    text = mixLabel,
    variant = "outline",
    controlSize = "sm",
    tooltip = tr("panel.mix_tip"),
    enabled = not busy,
    onClick = function()
      panel.close()
      noctalia.togglePanel("noct/effects:mix")
    end,
  })

  rows[#rows + 1] = ui.separator({})

  if not monoActive() then
    local screen = screenName() or "—"
    rows[#rows + 1] = ui.label({
      text = tr("panel.mono_inactive", { screen = screen }),
      fontSize = 11,
      color = "on_surface/0.6",
      maxWidth = 340,
    })
  end

  rows[#rows + 1] = ui.row({ align = "center", justify = "space_between",
    tooltip = tr("panel.tint_tip") }, {
    ui.label({ text = tr("panel.tint_title"), fontSize = 13, fontWeight = "bold" }),
    ui.label({
      text = tint or tr("panel.tint_pure"),
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
    text = tr("panel.tint_pure_button"),
    variant = (tint == nil) and "primary" or "outline",
    controlSize = "sm",
    tooltip = tr("panel.tint_tip"),
    enabled = not busy,
    onClick = function()
      setTint("none")
    end,
  })

  rows[#rows + 1] = ui.row({ align = "center", justify = "space_between",
    tooltip = tr("panel.intensity_tip") }, {
    ui.label({ text = tr("panel.intensity_title"), fontSize = 13, fontWeight = "bold" }),
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
    done(false, tr("panel.error", { message = "noct" }))
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
  status = tr("panel.applying_preset", { preset = presetName(id) })
  render()
  run({ "noct", "effects", "apply", id }, function(ok, out)
    busy = false
    if ok then
      syncState(id, tint, intensity)
      status = tr("panel.applied", { preset = presetName(id) })
      refresh()
    else
      status = tr("panel.error", { message = out })
      render()
    end
  end)
end

function setIntensity(value)
  if busy then
    return
  end
  busy = true
  status = tr("panel.intensity_applying", { value = value })
  render()
  run({ "noct", "effects", "intensity", tostring(value), "--json" }, function(ok, out)
    busy = false
    if ok then
      intensity = value
      syncState(nil, tint, intensity)
      status = tr("panel.intensity_applied", { value = value })
      refresh()
    else
      status = tr("panel.error", { message = out })
      render()
    end
  end)
end

function setTint(color)
  if busy then
    return
  end
  busy = true
  status = tr("panel.tint_applying", { color = color })
  render()
  run({ "noct", "effects", "tint", color, "--json" }, function(ok, out)
    busy = false
    if ok then
      syncState(nil, color == "none" and nil or color, intensity)
      status = tr("panel.tint_applied", { color = color })
      refresh()
    else
      status = tr("panel.error", { message = out })
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
