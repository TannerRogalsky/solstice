varying vec2 vPosition;
varying vec3 vColor;

#ifdef VERTEX
attribute vec4 position;
attribute vec3 color;

uniform mat4 uProjection;

void main() {
    vColor = color;
    vPosition = position.xy;
    gl_Position = uProjection * position;
}
#endif

#ifdef FRAGMENT
void main() {
    vec2 pos = normalize(vPosition);
    vec3 color = vec3(pos.x, pos.y, 1. - pos.y) * vColor;
    fragColor = vec4(color, 1.);
}
#endif