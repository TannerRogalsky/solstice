varying vec4 vLineColor;

#ifdef VERTEX
attribute vec3 point;
attribute vec3 position1, position2;
attribute float width1, width2;
attribute vec4 color1, color2;

uniform vec2 resolution;

vec4 pos(mat4 transform_projection, vec4 _vertex_position) {
    vLineColor = mix(color1, color2, point.z);

    vec4 clip0 = uProjection * uView * uModel * vec4(position1, 1.0);
    vec4 clip1 = uProjection * uView * uModel * vec4(position2, 1.0);
    vec2 screen0 = resolution * (0.5 * clip0.xy/clip0.w + 0.5);
    vec2 screen1 = resolution * (0.5 * clip1.xy/clip1.w + 0.5);
    vec2 xBasis = normalize(screen1 - screen0);
    vec2 yBasis = vec2(-xBasis.y, xBasis.x);
    vec2 pt0 = screen0 + width1 * (point.x * xBasis + point.y * yBasis);
    vec2 pt1 = screen1 + width2 * (point.x * xBasis + point.y * yBasis);
    vec2 pt = mix(pt0, pt1, point.z);
    vec4 clip = mix(clip0, clip1, point.z);
    return vec4(clip.w * (2.0 * pt/resolution - 1.0), clip.z, clip.w);
}
#endif

#ifdef FRAGMENT
vec4 effect(vec4 color, Image texture, vec2 texture_coords, vec2 screen_coords) {
    return vLineColor * uColor * Texel(texture, texture_coords);
}
#endif