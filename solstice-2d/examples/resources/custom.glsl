varying vec2 vPosition;

#ifdef VERTEX
vec4 pos(mat4 transform_projection, vec4 vertex_position) {
    vPosition = vertex_position.xy;
    return transform_projection * vertex_position;
}
#endif

#ifdef FRAGMENT
vec4 effect(vec4 color, Image texture, vec2 texture_coords, vec2 screen_coords) {
    vec2 pos = normalize(vPosition);
    return vec4(pos.x, pos.y, 1. - pos.y, 1.) * color;
}
#endif