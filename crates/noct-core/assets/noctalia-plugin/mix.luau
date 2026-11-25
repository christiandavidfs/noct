--!nonstrict
-- Mix panel: choose the effect of each slot without changing the preset.

local function tr(key, args)
  local value = noctalia.tr(key, args)
  if type(value) == "string" and value ~= "" then
    return value
  end
  return key
end

local SLOTS = {
  { key = "border",       label_key = "mix.slots.border.label",       tip_key = "mix.slots.border.tip" },
  { key = "cursor",       label_key = "mix.slots.cursor.label",       tip_key = "mix.slots.cursor.tip" },
  { key = "screen",       label_key = "mix.slots.screen.label",       tip_key = "mix.slots.screen.tip" },
  { key = "window",       label_key = "mix.slots.window.label",       tip_key = "mix.slots.window.tip" },
  { key = "windows_move", label_key = "mix.slots.windows_move.label", tip_key = "mix.slots.windows_move.tip" },
  { key = "windows_in",   label_key = "mix.slots.windows_in.label",   tip_key = "mix.slots.windows_in.tip" },
  { key = "windows_out",  label_key = "mix.slots.windows_out.label",  tip_key = "mix.slots.windows_out.tip" },
  { key = "workspaces",   label_key = "mix.slots.workspaces.label",   tip_key = "mix.slots.workspaces.tip" },
}

local EFFECT_TIPS = {
  border = {
    pulse = "mix.effects.border.pulse",
    audio = "mix.effects.border.audio",
  },
  cursor = {
    glow = "mix.effects.cursor.glow",
    trail = "mix.effects.cursor.trail",
    ["trail-path"] = "mix.effects.cursor.trail-path",
  },
  screen = {
    vignette = "mix.effects.screen.vignette",
    ["noct-mono"] = "mix.effects.screen.noct-mono",
  },
  window = {
    scanlines = "mix.effects.window.scanlines",
  },
  windows_in = {
    reveal = "mix.effects.windows_in.reveal",
  },
  windows_out = {
    reveal = "mix.effects.windows_out.reveal",
  },
  windows_move = {
    squash = "mix.effects.windows_move.squash",
  },
  workspaces = {
    workspace_wipe = "mix.effects.workspaces.workspace_wipe",
  },
}

local SLOT_LABELS = {}
for _, slot in ipairs(SLOTS) do
  SLOT_LABELS[slot.key] = slot.label_key
end

local preset = ""
local overrides = {}
local effective = {}
local choices = {}
local status = ""
local busy = false

local render
local run
local refresh

local function slotLabel(key)
  return tr(SLOT_LABELS[key] or key)
end

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
      return tr("mix.auto_active", { active = active })
    end
    return tr("mix.auto")
  elseif value == "none" then
    return tr("mix.off")
  end
  return value
end

local function labelForValue(value)
  if value == "auto" then
    return tr("mix.auto")
  elseif value == "none" then
    return tr("mix.off")
  end
  return value
end

local function describe(key, value)
  local tip_key = EFFECT_TIPS[key] and EFFECT_TIPS[key][value]
  if tip_key ~= nil then
    return tr(tip_key)
  end
  return value
end

local function valueTooltip(key, value)
  if value == "auto" then
    local active = effective[key]
    if type(active) == "string" and active ~= "" then
      return tr("mix.value_tip_auto_active", { active = active, desc = describe(key, active) })
    end
    return tr("mix.value_tip_auto")
  elseif value == "none" then
    return tr("mix.value_tip_off")
  end
  return tr("mix.value_tip", { name = value, desc = describe(key, value) })
end

local function slotTooltip(slot)
  local parts = { tr(slot.tip_key) }
  local options = choices[slot.key]
  if type(options) == "table" then
    for _, name in ipairs(options) do
      parts[#parts + 1] = name .. ": " .. describe(slot.key, name)
    end
  end
  parts[#parts + 1] = tr("mix.auto_off_hint")
  return table.concat(parts, "  ·  ")
end

local function setSlot(key, value)
  if busy then
    return
  end
  busy = true
  status = tr("mix.applying", { slot = slotLabel(key), value = labelForValue(value) })
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
      if value == "auto" then
        status = tr("mix.slot_reset", { slot = slotLabel(key) })
      else
        status = tr("mix.slot_set", { slot = slotLabel(key), value = labelForValue(value) })
      end
      refresh()
    else
      status = tr("mix.error", { message = out })
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
  status = tr("mix.clearing")
  render()
  run({ "noct", "effects", "mix", "clear" }, function(ok, out)
    busy = false
    if ok then
      status = tr("mix.cleared")
      refresh()
    else
      status = tr("mix.error", { message = out })
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
      ui.label({ text = tr("mix.title"), fontSize = 16, fontWeight = "bold" }),
    }),
    ui.button({ glyph = "close", variant = "ghost", onClick = "closePanel" }),
  })

  rows[#rows + 1] = ui.label({
    text = tr("mix.preset", { preset = preset ~= "" and preset or tr("mix.preset_none") }),
    fontSize = 12,
    color = "on_surface/0.65",
  })
  rows[#rows + 1] = ui.label({
    text = tr("mix.hint"),
    fontSize = 11,
    color = "on_surface/0.55",
    maxWidth = 340,
  })
  rows[#rows + 1] = ui.separator({})

  for _, slot in ipairs(SLOTS) do
    local value = overrideValue(slot.key)
    rows[#rows + 1] = ui.row({ align = "center", justify = "space_between",
      tooltip = slotTooltip(slot) }, {
      ui.label({ text = tr(slot.label_key), fontSize = 13, fontWeight = "bold" }),
      ui.button({
        text = displayValue(slot.key),
        variant = value == "auto" and "ghost" or "outline",
        controlSize = "sm",
        tooltip = valueTooltip(slot.key, value),
        enabled = not busy,
        onClick = function()
          cycle(slot.key)
        end,
      }),
    })
  end

  rows[#rows + 1] = ui.separator({})
  rows[#rows + 1] = ui.button({
    text = tr("mix.clear"),
    variant = "outline",
    controlSize = "sm",
    tooltip = tr("mix.clear_tip"),
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
    done(false, "noct")
  end
end

function onOpen(_context)
  render()
  refresh()
end

function closePanel()
  panel.close()
end
