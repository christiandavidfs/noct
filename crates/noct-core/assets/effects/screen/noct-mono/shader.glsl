// Noct Mono: grayscale + scanlines + vignette over the whole output.
// __NOCT_TINT__ se sustituye al aplicar el preset.
const vec3 NOCT_TINT = vec3(__R__, __G__, __B__);
const float NOCT_TINT_MIX = __MIX__;

vec4 screen(vec2 uv) {
  vec4 c = umbriel_sample(uv);

  // Luminance perceptual (Rec. 709).
  float luma = dot(c.rgb, vec3(0.2126, 0.7152, 0.0722));

  // Ligero contraste en S para que el blanco y negro no se vea plano.
  float mono = smoothstep(0.0, 1.0, luma);
  mono = mix(mono, mono * mono * (3.0 - 2.0 * mono), 0.40);

  // Scanlines cada 3 píxeles lógicos.
  float line = mod(floor(uv.y * umbriel_size.y), 3.0) == 0.0 ? 0.78 : 1.0;

  // Viñeta suave en los bordes.
  vec2 p = uv - 0.5;
  float falloff = smoothstep(0.32, 0.95, length(p) * 1.25);
  float dim = (1.0 - 0.55 * falloff) * line;

  // Tinte opcional: negro → color → blanco.
  vec3 base = vec3(mono);
  vec3 tinted = NOCT_TINT * mono;
  vec3 color = mix(base, tinted, NOCT_TINT_MIX) * dim;

  return vec4(color, c.a);
}
